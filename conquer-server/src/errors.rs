use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

/// API error type
#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

impl From<conquer_db::DbError> for ApiError {
    fn from(err: conquer_db::DbError) -> Self {
        match err {
            conquer_db::DbError::NotFound(msg) => ApiError::NotFound(msg),
            conquer_db::DbError::AlreadyExists(msg) => ApiError::Conflict(msg),
            conquer_db::DbError::InvalidState(msg) => ApiError::BadRequest(msg),
            conquer_db::DbError::AuthError(msg) => ApiError::Unauthorized(msg),
            conquer_db::DbError::Unauthorized(msg) => ApiError::Forbidden(msg),
            conquer_db::DbError::GameFull => ApiError::Conflict("Game full".to_string()),
            conquer_db::DbError::SerializationError(msg) => ApiError::Internal(msg),
            conquer_db::DbError::Internal(msg) => ApiError::Internal(msg),
        }
    }
}
