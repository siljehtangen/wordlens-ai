use axum::{
    http::StatusCode,
    response::{IntoResponse, Json as JsonResponse},
};

use crate::types::ErrorResponse;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    InvalidRequest(String),
    #[error("{0}")]
    OllamaUnreachable(String),
    #[error("{0}")]
    OllamaBadGateway(String),
    #[error("{0}")]
    OllamaParseError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::InvalidRequest(msg)    => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            AppError::OllamaUnreachable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            AppError::OllamaBadGateway(msg)  => (StatusCode::BAD_GATEWAY, msg),
            AppError::OllamaParseError(msg)  => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        (status, JsonResponse(ErrorResponse { error: message })).into_response()
    }
}
