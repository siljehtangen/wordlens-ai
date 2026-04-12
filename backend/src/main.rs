use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Json as JsonResponse,
    },
    routing::{get, post},
    Router,
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};

const MAX_WORD_LEN: usize = 200;
const VALID_LENSES: &[&str] = &["simple", "learning", "game", "cyberpunk", "poetic"];

#[derive(Clone)]
struct AppState {
    http: reqwest::Client,
    ollama_url: String,
}

#[derive(Debug, Deserialize)]
struct ExplainRequest {
    word: String,
    lens: String,
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

async fn health() -> impl IntoResponse {
    JsonResponse(serde_json::json!({ "status": "ok" }))
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
        .post(format!("{}/api/generate", state.ollama_url))
        .json(&body)
        .timeout(std::time::Duration::from_secs(60))
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
                        state.ollama_url
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

    Ok(JsonResponse(ExplainResponse {
        explanation,
        lens: payload.lens,
        word: payload.word,
    }))
}

async fn explain_stream(
    state: Arc<AppState>,
    payload: ExplainRequest,
) -> Result<
    Sse<impl futures::Stream<Item = Result<Event, axum::Error>>>,
    (StatusCode, JsonResponse<ErrorResponse>),
> {
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
        .post(format!("{}/api/generate", state.ollama_url))
        .json(&body)
        .timeout(std::time::Duration::from_secs(60))
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

    let event_stream = resp
        .bytes_stream()
        .map(|chunk| {
            let chunk = chunk.map_err(|e| {
                error!(error = %e, "error reading Ollama stream chunk");
            })?;
            let line = std::str::from_utf8(&chunk).unwrap_or("").trim().to_string();
            if line.is_empty() {
                return Err(());
            }
            let json: serde_json::Value = serde_json::from_str(&line).map_err(|e| {
                error!(error = %e, raw = %line, "failed to parse Ollama stream chunk");
            })?;
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "wordlens_backend=debug,tower_http=debug".into()),
        )
        .init();

    let ollama_url = std::env::var("OLLAMA_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:11434".to_string());

    info!("Using Ollama at {ollama_url}");

    let state = Arc::new(AppState {
        http: reqwest::Client::builder()
            .pool_max_idle_per_host(4)
            .build()
            .expect("Failed to build HTTP client"),
        ollama_url,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/explain", post(explain))
        .with_state(state)
        .layer(cors);

    let addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3001".to_string());

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap_or_else(|e| {
        eprintln!("ERROR: Failed to bind to {addr}: {e}");
        eprintln!("       Is another process already using that port?");
        std::process::exit(1);
    });

    info!("WordLens backend listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
