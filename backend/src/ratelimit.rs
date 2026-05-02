use std::sync::Arc;
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use parking_lot::Mutex;

/// Fixed-window rate limiter shared across all requests.
#[derive(Clone)]
pub struct RateLimiter(Arc<Mutex<Inner>>);

struct Inner {
    count: u64,
    window_end: std::time::Instant,
    limit: u64,
    window: std::time::Duration,
}

impl RateLimiter {
    pub fn new(limit: u64, window: std::time::Duration) -> Self {
        Self(Arc::new(Mutex::new(Inner {
            count: 0,
            window_end: std::time::Instant::now() + window,
            limit,
            window,
        })))
    }

    fn check(&self) -> Result<(), u64> {
        let mut g = self.0.lock();
        let now = std::time::Instant::now();
        if now >= g.window_end {
            g.window_end = now + g.window;
            g.count = 0;
        }
        g.count += 1;
        if g.count <= g.limit {
            Ok(())
        } else {
            Err((g.window_end - now).as_secs().max(1))
        }
    }
}

pub async fn rate_limit_middleware(
    State(limiter): State<RateLimiter>,
    req: Request,
    next: Next,
) -> Response {
    match limiter.check() {
        Ok(()) => next.run(req).await,
        Err(retry_after) => (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
        )
            .into_response(),
    }
}
