use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

/// Type d'erreur unifié — toute erreur interne remonte ici
/// puis est convertie en réponse HTTP JSON structurée.
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
