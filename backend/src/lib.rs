pub mod error;
pub mod handlers;
pub mod history;
pub mod ollama;
pub mod prompts;
pub mod ratelimit;
pub mod state;
pub mod types;

use std::sync::Arc;

use axum::Router;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};

use handlers::{explain, get_history, health};
use ratelimit::{rate_limit_middleware, RateLimiter};
use state::AppState;

pub const MAX_BODY_BYTES: usize = 8 * 1024;
pub const CACHE_MAX_CAPACITY: u64 = 500;
pub const RATE_LIMIT_REQUESTS: u64 = 120;
pub const RATE_LIMIT_WINDOW: std::time::Duration = std::time::Duration::from_secs(60);
pub const DEFAULT_HISTORY_LIMIT: usize = 20;

pub fn build_app(state: Arc<AppState>, limiter: RateLimiter, cors: CorsLayer) -> Router {
    Router::new()
        .route("/health", axum::routing::get(health))
        .route("/api/explain", axum::routing::post(explain))
        .route("/api/history", axum::routing::get(get_history))
        .with_state(state)
        .layer(axum::middleware::from_fn_with_state(
            limiter,
            rate_limit_middleware,
        ))
        .layer(axum::extract::DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(CompressionLayer::new())
        .layer(cors)
}

pub fn build_app_with_static(
    state: Arc<AppState>,
    limiter: RateLimiter,
    cors: CorsLayer,
    frontend_dist: &str,
) -> Router {
    let serve_dir = ServeDir::new(frontend_dist)
        .not_found_service(ServeFile::new(format!("{frontend_dist}/index.html")));

    build_app(state, limiter, cors).fallback_service(serve_dir)
}

/// Builds an AppState suitable for tests: Ollama points at port 1
/// (guaranteed immediate ECONNREFUSED) with a short connect timeout.
pub fn test_state() -> Arc<AppState> {
    Arc::new(AppState {
        http: reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(1))
            .build()
            .expect("test http client"),
        ollama_generate_url: "http://127.0.0.1:1/api/generate".to_string(),
        model: "test-model".to_string(),
        cache: moka::future::Cache::builder()
            .max_capacity(10)
            .time_to_live(std::time::Duration::from_secs(60))
            .build(),
        history: Arc::new(history::History::default()),
    })
}
