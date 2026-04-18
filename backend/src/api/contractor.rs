// ── api/contractor.rs ─────────────────────────────────────────────
use std::sync::Arc;
use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::Deserialize;
use uuid::Uuid;

use crate::{db::{self, AppState}, errors::{AppError, AppResult}};

/// GET /vendor/:token — public access, no JWT
pub async fn get_task(
    State(state): State<Arc<AppState>>,
    Path(token): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let task = db::task::get_by_token(&state.pool, token)
        .await?
        .ok_or(AppError::NotFound)?;

    // Only expose information needed by the vendor
    Ok(Json(serde_json::json!({
        "id": task.id,
        "title": task.title,
        "description": task.description,
        "status": task.status,
        "token_expires_at": task.token_expires_at,
    })))
}

#[derive(Deserialize)]
pub struct SubmissionInput {
    pub comment: Option<String>,
    pub document_keys: Vec<String>, // keys in staging bucket
}

/// POST /vendor/:token/submission
pub async fn submit(
    State(state): State<Arc<AppState>>,
    Path(token): Path<Uuid>,
    Json(input): Json<SubmissionInput>,
) -> AppResult<StatusCode> {
    let task = db::task::get_by_token(&state.pool, token)
        .await?
        .ok_or(AppError::NotFound)?;

    let submission_id = db::task::create_submission(&state.pool, task.id, input.comment).await?;

    // Move each file from staging to the building's final bucket
    for key in &input.document_keys {
        let ext = key.rsplit('.').next().unwrap_or("bin");
        let (bucket, final_key) = state.storage
            .move_from_staging(key, task.building_id, ext)
            .await?;

        let doc = db::document::create(
            &state.pool,
            task.building_id,
            task.id, // uploaded_by = task id as proxy (no user)
            key, &final_key, &bucket,
            "application/octet-stream", 0,
            &["manager".to_string(), "board_member".to_string()],
        ).await?;

        db::task::attach_submission_document(&state.pool, submission_id, doc.id).await?;
    }

    // Automatically close the task after submission
    db::task::update_status(&state.pool, task.id, task.building_id, "closed").await?;

    // Automatically feed the maintenance log
    db::maintenance_log::from_task(&state.pool, task.id, task.building_id, task.id).await?;

    Ok(StatusCode::CREATED)
}
