use std::sync::Arc;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tracing::{info, warn};

use wordlens_backend::{
    build_app_with_static, history::History, ratelimit::RateLimiter, state::AppState,
    CACHE_MAX_CAPACITY, RATE_LIMIT_REQUESTS, RATE_LIMIT_WINDOW,
};
use wordlens_backend::handlers::shutdown_signal;

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
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "wordlens_backend=debug,tower_http=debug".into());

    if std::env::var("LOG_FORMAT").unwrap_or_default().eq_ignore_ascii_case("json") {
        tracing_subscriber::fmt().json().with_env_filter(env_filter).init();
    } else {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    }

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
        history: Arc::new(History::default()),
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
    if !std::path::Path::new(&cfg.frontend_dist).exists() {
        warn!(
            path = %cfg.frontend_dist,
            "FRONTEND_DIST path does not exist — set FRONTEND_DIST or run `trunk build` first"
        );
    }

    let app = build_app_with_static(state, limiter, cors, &cfg.frontend_dist);

    let listener = tokio::net::TcpListener::bind(&cfg.bind_addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("ERROR: failed to bind to {}: {e}", cfg.bind_addr);
            std::process::exit(1);
        });

    info!("WordLens backend listening on http://{}", cfg.bind_addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}
