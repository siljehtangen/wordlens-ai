mod handlers;
mod history;
mod ollama;
mod prompts;
mod state;
mod types;

use std::sync::Arc;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};
use tracing::info;

use handlers::{explain, get_history, health, shutdown_signal};
use state::AppState;

const MAX_BODY_BYTES: usize = 8 * 1024;
const CACHE_MAX_CAPACITY: u64 = 500;

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

    let cache = moka::future::Cache::builder()
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
        history: Arc::new(history::History::default()),
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

    let app = axum::Router::new()
        .route("/health", axum::routing::get(health))
        .route("/api/explain", axum::routing::post(explain))
        .route("/api/history", axum::routing::get(get_history))
        .with_state(state)
        .layer(axum::extract::DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(CompressionLayer::new())
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
