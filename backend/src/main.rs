mod error;
mod handlers;
mod history;
mod ollama;
mod prompts;
mod ratelimit;
mod state;
mod types;

use std::sync::Arc;
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, Any, CorsLayer},
    services::{ServeDir, ServeFile},
};
use tracing::info;

use handlers::{explain, get_history, health, shutdown_signal};
use ratelimit::{rate_limit_middleware, RateLimiter};
use state::AppState;

const MAX_BODY_BYTES: usize = 8 * 1024;
const CACHE_MAX_CAPACITY: u64 = 500;
/// Global fixed-window cap: 120 requests per 60 s across all clients.
const RATE_LIMIT_REQUESTS: u64 = 120;
const RATE_LIMIT_WINDOW: std::time::Duration = std::time::Duration::from_secs(60);

struct Config {
    ollama_url: String,
    model: String,
    frontend_dist: String,
    bind_addr: String,
    /// Comma-separated allowed CORS origins, or "*" to allow any.
    cors_origins: String,
}

impl Config {
    fn from_env() -> Self {
        Self {
            ollama_url: std::env::var("OLLAMA_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:11434".to_string()),
            model: std::env::var("OLLAMA_MODEL")
                .unwrap_or_else(|_| "llama3".to_string()),
            frontend_dist: std::env::var("FRONTEND_DIST")
                .unwrap_or_else(|_| "../frontend/dist".to_string()),
            bind_addr: std::env::var("BIND_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:3001".to_string()),
            cors_origins: std::env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "wordlens_backend=debug,tower_http=debug".into()),
        )
        .init();

    let cfg = Config::from_env();
    let ollama_generate_url = format!("{}/api/generate", cfg.ollama_url);

    info!(model = %cfg.model, url = %ollama_generate_url, "Ollama config");
    info!(origins = %cfg.cors_origins, "CORS config");

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
        model: cfg.model,
        cache,
        history: Arc::new(history::History::default()),
    });

    let allow_origin: AllowOrigin = if cfg.cors_origins.trim() == "*" {
        AllowOrigin::any()
    } else {
        let origins = cfg
            .cors_origins
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect::<Vec<axum::http::HeaderValue>>();
        AllowOrigin::list(origins)
    };

    let cors = CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods(Any)
        .allow_headers(Any);

    let limiter = RateLimiter::new(RATE_LIMIT_REQUESTS, RATE_LIMIT_WINDOW);

    info!("serving frontend from {}", cfg.frontend_dist);

    let serve_dir = ServeDir::new(&cfg.frontend_dist)
        .not_found_service(ServeFile::new(format!("{}/index.html", cfg.frontend_dist)));

    let app = axum::Router::new()
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
        .fallback_service(serve_dir);

    let listener = tokio::net::TcpListener::bind(&cfg.bind_addr).await.unwrap_or_else(|e| {
        eprintln!("ERROR: failed to bind to {}: {e}", cfg.bind_addr);
        std::process::exit(1);
    });

    info!("WordLens backend listening on http://{}", cfg.bind_addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}
