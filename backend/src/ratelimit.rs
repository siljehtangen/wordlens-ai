use std::sync::Arc;
use axum::{
    extract::{Request, State},
    http::StatusCode,
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

    fn is_allowed(&self) -> bool {
        let mut g = self.0.lock();
        let now = std::time::Instant::now();
        if now >= g.window_end {
            g.window_end = now + g.window;
            g.count = 1;
            return true;
        }
        g.count += 1;
        g.count <= g.limit
    }
}

pub async fn rate_limit_middleware(
    State(limiter): State<RateLimiter>,
    req: Request,
    next: Next,
) -> Response {
    if limiter.is_allowed() {
        next.run(req).await
    } else {
        StatusCode::TOO_MANY_REQUESTS.into_response()
    }
}
