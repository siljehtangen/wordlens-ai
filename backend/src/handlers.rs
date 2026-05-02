use axum::{
    extract::{Json, Query, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json as JsonResponse, Response,
    },
};
use futures::StreamExt;
use std::sync::Arc;
use tokio::time::timeout;
use tracing::{error, info, warn, Span};

use crate::error::AppError;
use crate::ollama::{build_prompt, lens_token_limit, ollama_body, validate_request, OllamaChunk, OllamaRequest, OLLAMA_TIMEOUT};
use crate::state::AppState;
use crate::types::{ExplainRequest, ExplainResponse, HistoryQuery, Lens};

const STREAM_CHUNK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn health() -> impl IntoResponse {
    JsonResponse(serde_json::json!({ "status": "ok" }))
}

pub async fn get_history(
    State(state): State<Arc<AppState>>,
    Query(q): Query<HistoryQuery>,
) -> impl IntoResponse {
    let limit = q.limit.min(crate::MAX_HISTORY_ENTRIES);
    JsonResponse(state.history.recent(limit))
}

pub async fn explain(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ExplainRequest>,
) -> Result<Response, AppError> {
    validate_request(&payload).map_err(|e| {
        warn!(word = %payload.word, lens = ?payload.lens, error = %e, "invalid request");
        e
    })?;

    info!(
        word = %payload.word.trim(),
        lens = ?payload.lens,
        stream = payload.stream,
        "explain request"
    );

    let cache_key = (payload.word.trim().to_lowercase(), payload.lens);

    if payload.stream {
        Ok(explain_stream(state, payload, cache_key).await?.into_response())
    } else {
        Ok(explain_json(state, payload, cache_key).await?.into_response())
    }
}

#[tracing::instrument(skip_all, fields(word = %payload.word.trim(), lens = ?payload.lens, cached = false))]
async fn explain_json(
    state: Arc<AppState>,
    payload: ExplainRequest,
    cache_key: (String, Lens),
) -> Result<JsonResponse<ExplainResponse>, AppError> {
    let lens = payload.lens;

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

    let resp = send_to_ollama(&state.http, &state.ollama_generate_url, &body).await?;

    let chunk: OllamaChunk = resp.json().await.map_err(|e| {
        error!(error = %e, "failed to parse Ollama response");
        AppError::OllamaParseError(format!("Failed to parse Ollama response: {e}"))
    })?;

    let raw = chunk.response.trim().to_string();
    let explanation = if raw.is_empty() { "No response generated.".to_string() } else { raw };

    state.history.push(&cache_key.0, lens, &explanation);
    state.cache.insert(cache_key, explanation.clone()).await;

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
    cache_key: (String, Lens),
) -> Result<Sse<futures::stream::BoxStream<'static, Result<Event, axum::Error>>>, AppError> {
    let lens = payload.lens;

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

    let resp = send_to_ollama(&state.http, &state.ollama_generate_url, &body).await?;

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
                    return futures::stream::iter(vec![
                        Ok(Event::default().event("error").data(format!("Stream read error: {e}"))),
                    ]);
                }
            };

            let text = std::str::from_utf8(&bytes).unwrap_or_else(|e| {
                warn!(error = %e, "invalid UTF-8 in Ollama stream chunk — skipping");
                ""
            });
            let events: Vec<Result<Event, axum::Error>> = text
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
                        history.push(&cache_key.0, lens, &full_text);
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
    // State is Option<stream>: None signals the stream is done after an error event.
    let guarded_stream = futures::stream::unfold(Some(event_stream), |state| async move {
        let mut s = state?;
        match timeout(STREAM_CHUNK_TIMEOUT, s.next()).await {
            Ok(Some(item)) => Some((item, Some(s))),
            Ok(None) => None,
            Err(_) => {
                error!("stream chunk timeout — Ollama stalled, closing SSE");
                Some((Ok(Event::default().event("error").data("Stream timed out")), None))
            }
        }
    })
    .boxed();

    Ok(Sse::new(guarded_stream).keep_alive(KeepAlive::default()))
}

// ── Shared helpers ────────────────────────────────────────────────────────────

async fn send_to_ollama(
    http: &reqwest::Client,
    url: &str,
    body: &OllamaRequest,
) -> Result<reqwest::Response, AppError> {
    let resp = http
        .post(url)
        .json(body)
        .timeout(OLLAMA_TIMEOUT)
        .send()
        .await
        .map_err(|e| {
            error!(error = %e, "cannot reach Ollama");
            AppError::OllamaUnreachable(format!(
                "Cannot reach Ollama at {url}. \
                 Make sure Ollama is running with `ollama serve`. Error: {e}"
            ))
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        error!(%status, body = %text, "Ollama error response");
        return Err(AppError::OllamaBadGateway(format!(
            "Ollama returned HTTP {status}: {text}"
        )));
    }

    Ok(resp)
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
