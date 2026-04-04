use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("conversion failed: {0}")]
    ConversionFailed(String),

    #[error("internal error: {0}")]
    InternalError(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, code, message) = match self {
            AppError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, "invalid_request", msg),
            AppError::ConversionFailed(msg) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "conversion_failed", msg)
            }
            AppError::InternalError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                err.to_string(),
            ),
        };
        (status, Json(json!({ "error": code, "message": message }))).into_response()
    }
}
