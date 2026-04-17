use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json as JsonResponse,
    },
    routing::{get, post},
    Router,
};
use futures::StreamExt;
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};
use tracing::{error, info, warn};

mod history;
mod prompts;

use history::History;

const MAX_WORD_LEN: usize = 200;
const MAX_BODY_BYTES: usize = 8 * 1024; // 8 KB — explain payloads are tiny
const VALID_LENSES: &[&str] = &["simple", "learning", "game", "cyberpunk", "poetic"];
const CACHE_MAX_CAPACITY: u64 = 500;
const OLLAMA_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct AppState {
    http: reqwest::Client,
    ollama_generate_url: String, // pre-built once at startup
    model: String,               // read once at startup
    cache: Cache<(String, String), String>,
    history: Arc<History>,
}

// ── Request / response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ExplainRequest {
    word: String,
    lens: String,
    #[serde(default)]
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct HistoryQuery {
    #[serde(default = "default_limit")]
    limit: usize,
}
fn default_limit() -> usize {
    20
}

#[derive(Debug, Serialize)]
struct ExplainResponse {
    explanation: String,
    lens: String,
    word: String,
    cached: bool,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

/// Typed shape of a single Ollama streaming chunk.
#[derive(Deserialize)]
struct OllamaChunk {
    #[serde(default)]
    response: String,
    #[serde(default)]
    done: bool,
}

// ── Validation & prompt ───────────────────────────────────────────────────────

fn validate_request(payload: &ExplainRequest) -> Result<(), String> {
    let word = payload.word.trim();
    if word.is_empty() {
        return Err("Word cannot be empty.".to_string());
    }
    if word.len() > MAX_WORD_LEN {
        return Err(format!("Word is too long (max {MAX_WORD_LEN} characters)."));
    }
    if !VALID_LENSES.contains(&payload.lens.as_str()) {
        return Err(format!(
            "Unknown lens '{}'. Valid options: {}.",
            payload.lens,
            VALID_LENSES.join(", ")
        ));
    }
    Ok(())
}

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

fn lens_token_limit(lens: &str) -> u32 {
    match lens {
        "simple"    => 110,
        "learning"  => 360,
        "game"      => 280,
        "cyberpunk" => 200,
        "poetic"    => 230,
        _           => 220,
    }
}

fn ollama_body(model: &str, prompt: &str, stream: bool, num_predict: u32) -> serde_json::Value {
    serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": stream,
        "options": {
            "num_predict": num_predict,
            "num_ctx": 512,
            "temperature": 0.7,
            "top_p": 0.9,
            "repeat_penalty": 1.1
        }
    })
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn health() -> impl IntoResponse {
    JsonResponse(serde_json::json!({ "status": "ok" }))
}

async fn get_history(
    State(state): State<Arc<AppState>>,
    Query(q): Query<HistoryQuery>,
) -> impl IntoResponse {
    let limit = q.limit.min(50);
    JsonResponse(state.history.recent(limit))
}

async fn explain(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ExplainRequest>,
) -> impl IntoResponse {
    if let Err(e) = validate_request(&payload) {
        warn!(word = %payload.word, lens = %payload.lens, error = %e, "invalid request");
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            JsonResponse(ErrorResponse { error: e }),
        )
            .into_response();
    }

    info!(
        word = %payload.word.trim(),
        lens = %payload.lens,
        stream = payload.stream,
        "explain request"
    );

    if payload.stream {
        explain_stream(state, payload).await.into_response()
    } else {
        explain_json(state, payload).await.into_response()
    }
}

async fn explain_json(
    state: Arc<AppState>,
    payload: ExplainRequest,
) -> Result<JsonResponse<ExplainResponse>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let word = payload.word.trim().to_lowercase();
    let lens = payload.lens.clone();
    let cache_key = (word.clone(), lens.clone());

    if let Some(cached) = state.cache.get(&cache_key).await {
        info!(word = %word, lens = %lens, "cache hit");
        return Ok(JsonResponse(ExplainResponse {
            explanation: cached,
            lens,
            word: payload.word,
            cached: true,
        }));
    }

    let prompt = build_prompt(&payload.word, &payload.lens);
    let body = ollama_body(&state.model, &prompt, false, lens_token_limit(&payload.lens));

    let resp = state
        .http
        .post(&state.ollama_generate_url)
        .json(&body)
        .timeout(OLLAMA_TIMEOUT)
        .send()
        .await
        .map_err(|e| {
            error!(error = %e, "cannot reach Ollama (json path)");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                JsonResponse(ErrorResponse {
                    error: format!(
                        "Cannot reach Ollama at {}. \
                         Make sure Ollama is running with `ollama serve`. Error: {e}",
                        state.ollama_generate_url
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
        error!(error = %e, "failed to parse Ollama response");
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

    state.cache.insert(cache_key, explanation.clone()).await;
    state.history.push(word, lens.clone(), &explanation);

    Ok(JsonResponse(ExplainResponse {
        explanation,
        lens,
        word: payload.word,
        cached: false,
    }))
}

async fn explain_stream(
    state: Arc<AppState>,
    payload: ExplainRequest,
) -> Result<
    Sse<futures::stream::BoxStream<'static, Result<Event, axum::Error>>>,
    (StatusCode, JsonResponse<ErrorResponse>),
> {
    let word = payload.word.trim().to_lowercase();
    let lens = payload.lens.clone();
    let cache_key = (word.clone(), lens.clone());

    // Cache hit: burst the full cached text as a single SSE event — no Ollama round-trip.
    if let Some(cached) = state.cache.get(&cache_key).await {
        info!(word = %word, lens = %lens, "stream cache hit");
        let events: Vec<Result<Event, axum::Error>> = vec![
            Ok(Event::default().data(cached)),
            Ok(Event::default().event("done").data("")),
        ];
        return Ok(Sse::new(futures::stream::iter(events).boxed()).keep_alive(KeepAlive::default()));
    }

    let prompt = build_prompt(&payload.word, &payload.lens);
    let body = ollama_body(&state.model, &prompt, true, lens_token_limit(&payload.lens));

    let resp = state
        .http
        .post(&state.ollama_generate_url)
        .json(&body)
        .timeout(OLLAMA_TIMEOUT)
        .send()
        .await
        .map_err(|e| {
            error!(error = %e, "cannot reach Ollama (stream path)");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                JsonResponse(ErrorResponse {
                    error: format!("Cannot reach Ollama: {e}"),
                }),
            )
        })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        error!(%status, %body, "Ollama error (stream path)");
        return Err((
            StatusCode::BAD_GATEWAY,
            JsonResponse(ErrorResponse {
                error: format!("Ollama returned HTTP {status}: {body}"),
            }),
        ));
    }

    let history = Arc::clone(&state.history);
    let cache = state.cache.clone();
    let mut full_text = String::new();

    // Each reqwest chunk may contain one or more newline-delimited JSON objects.
    // We split on newlines, parse each as OllamaChunk, and emit SSE events.
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
                        history.push(word.clone(), lens.clone(), &full_text);
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

    Ok(Sse::new(event_stream).keep_alive(KeepAlive::default()))
}

// ── Graceful shutdown ─────────────────────────────────────────────────────────

async fn shutdown_signal() {
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

// ── Main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "wordlens_backend=debug,tower_http=debug".into()),
        )
        .init();

    let ollama_base = std::env::var("OLLAMA_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:11434".to_string());
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3".to_string());
    let ollama_generate_url = format!("{ollama_base}/api/generate");

    info!(%model, url = %ollama_generate_url, "Ollama config");

    let cache: Cache<(String, String), String> = Cache::builder()
        .max_capacity(CACHE_MAX_CAPACITY)
        .time_to_live(std::time::Duration::from_secs(3600))
        .build();

    let state = Arc::new(AppState {
        http: reqwest::Client::builder()
            .pool_max_idle_per_host(4)
            .build()
            .expect("failed to build HTTP client"),
        ollama_generate_url,
        model,
        cache,
        history: Arc::new(History::default()),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let frontend_dist = std::env::var("FRONTEND_DIST")
        .unwrap_or_else(|_| "../frontend/dist".to_string());
    info!("serving frontend from {frontend_dist}");

    let serve_dir = ServeDir::new(&frontend_dist)
        .not_found_service(ServeFile::new(format!("{frontend_dist}/index.html")));

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/explain", post(explain))
        .route("/api/history", get(get_history))
        .with_state(state)
        // Innermost: cap request body before it reaches handlers
        .layer(axum::extract::DefaultBodyLimit::max(MAX_BODY_BYTES))
        // Compress responses (gzip/deflate) — free bandwidth win
        .layer(CompressionLayer::new())
        // Outermost: CORS headers on every response including errors
        .layer(cors)
        .fallback_service(serve_dir);

    let addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3001".to_string());

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap_or_else(|e| {
        eprintln!("ERROR: failed to bind to {addr}: {e}");
        std::process::exit(1);
    });

    info!("WordLens backend listening on http://{addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}
