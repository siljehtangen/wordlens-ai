use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json as JsonResponse,
    },
    routing::post,
    Router,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

// ── Shared app state ─────────────────────────────────────────────────────────

#[derive(Clone)]
struct AppState {
    http: reqwest::Client,
}

// ── Request / Response types ────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ExplainRequest {
    word: String,
    lens: String,
    /// Set to true to receive a streaming SSE response instead of JSON.
    #[serde(default)]
    stream: bool,
}

#[derive(Debug, Serialize)]
struct ExplainResponse {
    explanation: String,
    lens: String,
    word: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

mod prompts;

// ── Prompt builder ───────────────────────────────────────────────────────────

fn build_prompt(word: &str, lens: &str) -> String {
    let word = word.trim();
    let template = match lens {
        "simple"    => prompts::PROMPT_SIMPLE,
        "learning"  => prompts::PROMPT_LEARNING,
        "game"      => prompts::PROMPT_GAME,
        "cyberpunk" => prompts::PROMPT_CYBERPUNK,
        "poetic"    => prompts::PROMPT_POETIC,
        _           => "Explain '{word}' clearly and concisely.",
    };
    template.replace("{word}", word)
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// Single handler that returns either streaming SSE or a JSON blob,
/// depending on `payload.stream`.
async fn explain(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ExplainRequest>,
) -> impl IntoResponse {
    if payload.stream {
        explain_stream(state, payload).await.into_response()
    } else {
        explain_json(state, payload).await.into_response()
    }
}

/// Non-streaming path — waits for the full Ollama response.
async fn explain_json(
    state: Arc<AppState>,
    payload: ExplainRequest,
) -> Result<JsonResponse<ExplainResponse>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let prompt = build_prompt(&payload.word, &payload.lens);

    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3".to_string());
    let body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
        "options": {
            "num_predict": 200,
            "num_ctx": 1024,
            "temperature": 0.7
        }
    });

    let resp = state
        .http
        .post("http://127.0.0.1:11434/api/generate")
        .json(&body)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                JsonResponse(ErrorResponse {
                    error: format!(
                        "Cannot reach Ollama at localhost:11434. \
                         Make sure Ollama is running with `ollama serve`. Error: {e}"
                    ),
                }),
            )
        })?;

    if !resp.status().is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            JsonResponse(ErrorResponse {
                error: format!("Ollama returned HTTP {}", resp.status()),
            }),
        ));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            JsonResponse(ErrorResponse {
                error: format!("Failed to parse Ollama response: {e}"),
            }),
        )
    })?;

    let explanation = json["response"]
        .as_str()
        .unwrap_or("No response generated.")
        .trim()
        .to_string();

    Ok(JsonResponse(ExplainResponse {
        explanation,
        lens: payload.lens,
        word: payload.word,
    }))
}

/// Streaming path — forwards Ollama token-by-token over SSE.
async fn explain_stream(
    state: Arc<AppState>,
    payload: ExplainRequest,
) -> Result<Sse<impl futures::Stream<Item = Result<Event, axum::Error>>>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let prompt = build_prompt(&payload.word, &payload.lens);

    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3".to_string());
    let body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": true,
        "options": {
            "num_predict": 200,
            "num_ctx": 1024,
            "temperature": 0.7
        }
    });

    let resp = state
        .http
        .post("http://127.0.0.1:11434/api/generate")
        .json(&body)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                JsonResponse(ErrorResponse {
                    error: format!("Cannot reach Ollama: {e}"),
                }),
            )
        })?;

    // Ollama streams newline-delimited JSON objects.
    // We re-emit each token as an SSE `data` event, and a final `done` event.
    let byte_stream = resp.bytes_stream();

    let event_stream = byte_stream
        .map(|chunk| {
            let chunk = chunk.map_err(|_| ())?;
            let line = std::str::from_utf8(&chunk).unwrap_or("").trim().to_string();
            if line.is_empty() {
                return Err(());
            }
            let json: serde_json::Value = serde_json::from_str(&line).map_err(|_| ())?;
            let token = json["response"].as_str().unwrap_or("").to_string();
            let done = json["done"].as_bool().unwrap_or(false);
            Ok((token, done))
        })
        .filter_map(|r| async move { r.ok() })
        .flat_map(|(token, done)| {
            let mut events: Vec<Result<Event, axum::Error>> = Vec::new();
            if !token.is_empty() {
                events.push(Ok(Event::default().data(token)));
            }
            if done {
                events.push(Ok(Event::default().event("done").data("")));
            }
            futures::stream::iter(events)
        });

    Ok(Sse::new(event_stream).keep_alive(KeepAlive::default()))
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "wordlens_backend=debug,tower_http=debug".into()),
        )
        .init();

    let state = Arc::new(AppState {
        http: reqwest::Client::builder()
            .pool_max_idle_per_host(4)
            .build()
            .expect("Failed to build HTTP client"),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/explain", post(explain))
        .with_state(state)
        .layer(cors);

    let addr = "0.0.0.0:3001";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("WordLens backend listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
