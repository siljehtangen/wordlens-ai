use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;
use tower_http::cors::CorsLayer;
use wordlens_backend::{build_app, ratelimit::RateLimiter, test_state, RATE_LIMIT_REQUESTS, RATE_LIMIT_WINDOW};

fn app() -> axum::Router {
    build_app(test_state(), RateLimiter::new(RATE_LIMIT_REQUESTS, RATE_LIMIT_WINDOW), CorsLayer::permissive())
}

fn json_post(uri: &str, body: &'static str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap()
}

// ── /health ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn health_returns_200() {
    let resp = app()
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ── /api/history ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn history_starts_empty() {
    let resp = app()
        .oneshot(Request::builder().uri("/api/history").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
    let val: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(val, serde_json::json!([]));
}

// ── /api/explain validation (no Ollama needed) ────────────────────────────────

#[tokio::test]
async fn empty_word_returns_422() {
    let resp = app()
        .oneshot(json_post("/api/explain", r#"{"word":"","lens":"simple","stream":false}"#))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn whitespace_word_returns_422() {
    let resp = app()
        .oneshot(json_post("/api/explain", r#"{"word":"   ","lens":"simple","stream":false}"#))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn word_too_long_returns_422() {
    let word = "a".repeat(201);
    let body = format!(r#"{{"word":"{word}","lens":"simple","stream":false}}"#);
    let resp = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/explain")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn invalid_lens_returns_422() {
    let resp = app()
        .oneshot(json_post("/api/explain", r#"{"word":"apple","lens":"invalid","stream":false}"#))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn missing_content_type_returns_415() {
    let resp = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/explain")
                .body(Body::from(r#"{"word":"apple","lens":"simple","stream":false}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

// ── Ollama unreachable ────────────────────────────────────────────────────────

#[tokio::test]
async fn ollama_unreachable_returns_503() {
    // test_state points Ollama at 127.0.0.1:1 — immediate ECONNREFUSED.
    let resp = app()
        .oneshot(json_post("/api/explain", r#"{"word":"quantum","lens":"simple","stream":false}"#))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
}

// ── Rate limiter ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn rate_limit_triggers_429() {
    // Tight window: 2 requests per minute.
    let tight = build_app(
        test_state(),
        RateLimiter::new(2, std::time::Duration::from_secs(60)),
        CorsLayer::permissive(),
    );

    let req = || {
        Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap()
    };

    assert_eq!(tight.clone().oneshot(req()).await.unwrap().status(), StatusCode::OK);
    assert_eq!(tight.clone().oneshot(req()).await.unwrap().status(), StatusCode::OK);
    assert_eq!(tight.oneshot(req()).await.unwrap().status(), StatusCode::TOO_MANY_REQUESTS);
}
