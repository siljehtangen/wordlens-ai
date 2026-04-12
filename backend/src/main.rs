use axum::{
    extract::Json,
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
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

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

// ── Prompt builder ───────────────────────────────────────────────────────────

fn build_prompt(word: &str, lens: &str) -> String {
    let word = word.trim();
    match lens {
        "simple" => format!(
            "Explain '{word}' in simple, clear, friendly terms that anyone can understand. \
             Use 2-3 sentences. No jargon, no bullet points — just plain prose."
        ),
        "learning" => format!(
            "Give a structured educational explanation of '{word}'. \
             Cover: what it is, why it matters, a concrete real-world example, and one surprising fact. \
             Use clear paragraphs with a logical flow."
        ),
        "game" => format!(
            "Explain '{word}' as if it is a mechanic or core system in a video game world. \
             Use game-design vocabulary — stats, abilities, spawn rates, power-ups, whatever fits. \
             Make it sound exciting, interactive, and playable!"
        ),
        "cyberpunk" => format!(
            "Explain '{word}' through a cyberpunk lens: neon-lit megacities, neural implants, \
             rogue AI, corporate dystopia, and hacker culture. \
             Use evocative, atmospheric tech-noir language. Keep it sharp and electric."
        ),
        "poetic" => format!(
            "Explain '{word}' in a poetic, metaphorical way. \
             Use vivid imagery, sensory detail, and emotional resonance. \
             Write it as flowing prose poetry — let it breathe and sing."
        ),
        _ => format!("Explain '{word}' clearly and concisely."),
    }
}

// ── Handlers ─────────────────────────────────────────────────────────────────

/// Single handler that returns either streaming SSE or a JSON blob,
/// depending on `payload.stream`.
async fn explain(Json(payload): Json<ExplainRequest>) -> impl IntoResponse {
    if payload.stream {
        explain_stream(payload).await.into_response()
    } else {
        explain_json(payload).await.into_response()
    }
}

/// Non-streaming path — waits for the full Ollama response.
async fn explain_json(
    payload: ExplainRequest,
) -> Result<JsonResponse<ExplainResponse>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let prompt = build_prompt(&payload.word, &payload.lens);

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": "llama3",
        "prompt": prompt,
        "stream": false
    });

    let resp = client
        .post("http://127.0.0.1:11434/api/generate")
        .json(&body)
        .timeout(std::time::Duration::from_secs(180))
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
    payload: ExplainRequest,
) -> Result<Sse<impl futures::Stream<Item = Result<Event, axum::Error>>>, (StatusCode, JsonResponse<ErrorResponse>)> {
    let prompt = build_prompt(&payload.word, &payload.lens);

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": "llama3",
        "prompt": prompt,
        "stream": true
    });

    let resp = client
        .post("http://127.0.0.1:11434/api/generate")
        .json(&body)
        .timeout(std::time::Duration::from_secs(180))
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

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/explain", post(explain))
        .layer(cors);

    let addr = "0.0.0.0:3001";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("WordLens backend listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}
