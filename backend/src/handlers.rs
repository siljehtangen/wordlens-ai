use axum::{
    extract::{Json, Query, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json as JsonResponse,
    },
};
use futures::StreamExt;
use std::sync::Arc;
use tokio::time::timeout;
use tracing::{error, info, warn, Span};

use crate::error::AppError;
use crate::ollama::{build_prompt, lens_token_limit, ollama_body, validate_request, OLLAMA_TIMEOUT};
use crate::state::AppState;
use crate::types::{ExplainRequest, ExplainResponse, HistoryQuery, OllamaChunk};

const STREAM_CHUNK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn health() -> impl IntoResponse {
    JsonResponse(serde_json::json!({ "status": "ok" }))
}

pub async fn get_history(
    State(state): State<Arc<AppState>>,
    Query(q): Query<HistoryQuery>,
) -> impl IntoResponse {
    let limit = q.limit.min(50);
    JsonResponse(state.history.recent(limit))
}

pub async fn explain(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ExplainRequest>,
) -> impl IntoResponse {
    if let Err(e) = validate_request(&payload) {
        warn!(word = %payload.word, lens = ?payload.lens, error = %e, "invalid request");
        return AppError::InvalidRequest(e).into_response();
    }

    info!(
        word = %payload.word.trim(),
        lens = ?payload.lens,
        stream = payload.stream,
        "explain request"
    );

    if payload.stream {
        explain_stream(state, payload).await.into_response()
    } else {
        explain_json(state, payload).await.into_response()
    }
}

#[tracing::instrument(skip_all, fields(word = %payload.word.trim(), lens = ?payload.lens, cached = false))]
async fn explain_json(
    state: Arc<AppState>,
    payload: ExplainRequest,
) -> Result<JsonResponse<ExplainResponse>, AppError> {
    let word = payload.word.trim().to_lowercase();
    let lens = payload.lens;
    let cache_key = (word.clone(), lens);

    if let Some(cached) = state.cache.get(&cache_key).await {
        Span::current().record("cached", true);
        info!("cache hit");
        return Ok(JsonResponse(ExplainResponse {
            explanation: cached,
            lens,
            word: payload.word,
            cached: true,
        }));
    }

    let prompt = build_prompt(&payload.word, lens);
    let body = ollama_body(&state.model, &prompt, false, lens_token_limit(lens));

    let resp = state
        .http
        .post(&state.ollama_generate_url)
        .json(&body)
        .timeout(OLLAMA_TIMEOUT)
        .send()
        .await
        .map_err(|e| {
            error!(error = %e, "cannot reach Ollama (json path)");
            AppError::OllamaUnreachable(format!(
                "Cannot reach Ollama at {}. \
                 Make sure Ollama is running with `ollama serve`. Error: {e}",
                state.ollama_generate_url
            ))
        })?;

    if !resp.status().is_success() {
        return Err(AppError::OllamaBadGateway(format!(
            "Ollama returned HTTP {}",
            resp.status()
        )));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| {
        error!(error = %e, "failed to parse Ollama response");
        AppError::OllamaParseError(format!("Failed to parse Ollama response: {e}"))
    })?;

    let explanation = json["response"]
        .as_str()
        .unwrap_or("No response generated.")
        .trim()
        .to_string();

    state.cache.insert(cache_key, explanation.clone()).await;
    state.history.push(word, lens, &explanation);

    Ok(JsonResponse(ExplainResponse {
        explanation,
        lens,
        word: payload.word,
        cached: false,
    }))
}

#[tracing::instrument(skip_all, fields(word = %payload.word.trim(), lens = ?payload.lens, cached = false))]
async fn explain_stream(
    state: Arc<AppState>,
    payload: ExplainRequest,
) -> Result<Sse<futures::stream::BoxStream<'static, Result<Event, axum::Error>>>, AppError> {
    let word = payload.word.trim().to_lowercase();
    let lens = payload.lens;
    let cache_key = (word.clone(), lens);

    // Cache hit: burst the full cached text as a single SSE event — no Ollama round-trip.
    if let Some(cached) = state.cache.get(&cache_key).await {
        Span::current().record("cached", true);
        info!("stream cache hit");
        let events: Vec<Result<Event, axum::Error>> = vec![
            Ok(Event::default().data(cached)),
            Ok(Event::default().event("done").data("")),
        ];
        return Ok(Sse::new(futures::stream::iter(events).boxed()).keep_alive(KeepAlive::default()));
    }

    let prompt = build_prompt(&payload.word, lens);
    let body = ollama_body(&state.model, &prompt, true, lens_token_limit(lens));

    let resp = state
        .http
        .post(&state.ollama_generate_url)
        .json(&body)
        .timeout(OLLAMA_TIMEOUT)
        .send()
        .await
        .map_err(|e| {
            error!(error = %e, "cannot reach Ollama (stream path)");
            AppError::OllamaUnreachable(format!("Cannot reach Ollama: {e}"))
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        error!(%status, %body, "Ollama error (stream path)");
        return Err(AppError::OllamaBadGateway(format!(
            "Ollama returned HTTP {status}: {body}"
        )));
    }

    let history = Arc::clone(&state.history);
    let cache = state.cache.clone();
    let mut full_text = String::new();

    // Each reqwest chunk may contain one or more newline-delimited JSON objects.
    // We split on newlines, parse each as OllamaChunk, and emit SSE events.
    // STREAM_CHUNK_TIMEOUT guards against Ollama stalling mid-response.
    let event_stream = resp
        .bytes_stream()
        .flat_map(move |chunk| {
            let bytes = match chunk {
                Ok(b) => b,
                Err(e) => {
                    error!(error = %e, "error reading Ollama stream chunk");
                    return futures::stream::iter(vec![]);
                }
            };

            let events: Vec<Result<Event, axum::Error>> = std::str::from_utf8(&bytes)
                .unwrap_or("")
                .lines()
                .filter(|l| !l.trim().is_empty())
                .filter_map(|line| serde_json::from_str::<OllamaChunk>(line).ok())
                .flat_map(|chunk| {
                    let mut evs: Vec<Result<Event, axum::Error>> = Vec::new();
                    if !chunk.response.is_empty() {
                        full_text.push_str(&chunk.response);
                        evs.push(Ok(Event::default().data(chunk.response)));
                    }
                    if chunk.done {
                        history.push(word.clone(), lens, &full_text);
                        let text = full_text.clone();
                        let key = cache_key.clone();
                        let cache = cache.clone();
                        tokio::spawn(async move { cache.insert(key, text).await });
                        evs.push(Ok(Event::default().event("done").data("")));
                    }
                    evs
                })
                .collect();

            futures::stream::iter(events)
        })
        .boxed();

    // Wrap each poll with a deadline so a stalled Ollama doesn't hold the connection open.
    let guarded_stream = futures::stream::unfold(event_stream, |mut s| async move {
        match timeout(STREAM_CHUNK_TIMEOUT, s.next()).await {
            Ok(Some(item)) => Some((item, s)),
            Ok(None) => None,
            Err(_) => {
                error!("stream chunk timeout — Ollama stalled, closing SSE");
                None
            }
        }
    })
    .boxed();

    Ok(Sse::new(guarded_stream).keep_alive(KeepAlive::default()))
}

// ── Graceful shutdown ─────────────────────────────────────────────────────────

pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("shutdown signal received, draining connections");
}
