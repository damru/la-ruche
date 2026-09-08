use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

/// Unified error type — all internal errors bubble up here
/// and are converted into a structured JSON HTTP response.
#[derive(Debug, Error)]
pub enum AppError {
    // ── 400 ─────────────────────────────────────────────────────
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    // ── 401 ─────────────────────────────────────────────────────
    #[error("Unauthenticated")]
    Unauthorized,

    #[error("Token expired or invalid")]
    TokenInvalid,

    // ── 403 ─────────────────────────────────────────────────────
    #[error("Access forbidden")]
    Forbidden,

    // ── 404 ─────────────────────────────────────────────────────
    #[error("Resource not found")]
    NotFound,

    // ── 409 ─────────────────────────────────────────────────────
    #[error("Conflict: {0}")]
    Conflict(String),

    // ── 422 ─────────────────────────────────────────────────────
    #[error("Unprocessable entity: {0}")]
    UnprocessableEntity(String),

    // ── 500 ─────────────────────────────────────────────────────
    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Storage error")]
    Storage(String),

    #[error("Internal error")]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    pub fn validation_code(code: &'static str) -> Self {
        Self::Validation(code.into())
    }

    pub fn bad_request_code(code: &'static str) -> Self {
        Self::BadRequest(code.into())
    }

    pub fn conflict_code(code: &'static str) -> Self {
        Self::Conflict(code.into())
    }

    pub fn unprocessable_entity_code(code: &'static str) -> Self {
        Self::UnprocessableEntity(code.into())
    }
}

/// JSON response body for errors
#[derive(Serialize)]
struct ErrorBody {
    error: &'static str,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Log 5xx errors
        if matches!(
            self,
            AppError::Database(_) | AppError::Internal(_) | AppError::Storage(_)
        ) {
            tracing::error!(error = %self, "Internal error");
        }

        let (status, code, message) = match &self {
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, "validation_error", msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg.clone()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", self.to_string()),
            AppError::TokenInvalid => (StatusCode::UNAUTHORIZED, "token_invalid", self.to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "forbidden", self.to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "not_found", self.to_string()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg.clone()),
            AppError::UnprocessableEntity(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "unprocessable_entity",
                msg.clone(),
            ),
            // 5xx: generic message to client, details in logs
            AppError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "db_error",
                "Database error".into(),
            ),
            AppError::Storage(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "storage_error",
                "Storage error".into(),
            ),
            AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Internal error".into(),
            ),
        };

        (
            status,
            Json(ErrorBody {
                error: code,
                message,
            }),
        )
            .into_response()
    }
}

/// Result alias to reduce signature noise
pub type AppResult<T> = Result<T, AppError>;

pub fn json_value<T: Serialize>(value: T) -> AppResult<Json<serde_json::Value>> {
    Ok(Json(
        serde_json::to_value(value).map_err(|e| AppError::Internal(e.into()))?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_code_builds_validation_variant_with_message() {
        let err = AppError::validation_code("errors.foo.bar");
        match err {
            AppError::Validation(msg) => assert_eq!(msg, "errors.foo.bar"),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn bad_request_code_builds_bad_request_variant_with_message() {
        let err = AppError::bad_request_code("errors.foo.bad");
        match err {
            AppError::BadRequest(msg) => assert_eq!(msg, "errors.foo.bad"),
            other => panic!("expected BadRequest, got {other:?}"),
        }
    }

    #[test]
    fn conflict_code_builds_conflict_variant_with_message() {
        let err = AppError::conflict_code("errors.foo.conflict");
        match err {
            AppError::Conflict(msg) => assert_eq!(msg, "errors.foo.conflict"),
            other => panic!("expected Conflict, got {other:?}"),
        }
    }

    #[test]
    fn unprocessable_entity_code_builds_variant_with_message() {
        let err = AppError::unprocessable_entity_code("errors.foo.unprocessable");
        match err {
            AppError::UnprocessableEntity(msg) => assert_eq!(msg, "errors.foo.unprocessable"),
            other => panic!("expected UnprocessableEntity, got {other:?}"),
        }
    }

    #[test]
    fn into_response_maps_each_variant_to_expected_status() {
        let cases = [
            (AppError::Validation("x".into()), StatusCode::BAD_REQUEST),
            (AppError::BadRequest("x".into()), StatusCode::BAD_REQUEST),
            (AppError::Unauthorized, StatusCode::UNAUTHORIZED),
            (AppError::TokenInvalid, StatusCode::UNAUTHORIZED),
            (AppError::Forbidden, StatusCode::FORBIDDEN),
            (AppError::NotFound, StatusCode::NOT_FOUND),
            (AppError::Conflict("x".into()), StatusCode::CONFLICT),
            (
                AppError::UnprocessableEntity("x".into()),
                StatusCode::UNPROCESSABLE_ENTITY,
            ),
            (
                AppError::Database(sqlx::Error::RowNotFound),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            (
                AppError::Storage("x".into()),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            (
                AppError::Internal(anyhow::anyhow!("boom")),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        ];
        for (err, expected) in cases {
            assert_eq!(err.into_response().status(), expected);
        }
    }

    #[test]
    fn json_value_serializes_value_into_ok_json() {
        let result = json_value(serde_json::json!({ "a": 1 }));
        assert!(result.is_ok());
    }
}
