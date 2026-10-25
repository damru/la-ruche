use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    db::{self, AppState},
    domain::{AuthUser, Role},
    errors::{AppError, AppResult, json_value},
};

/// List all reports for the authenticated user's building.
/// All authenticated roles can view reports.
pub async fn list(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let rows = db::report::list(&state.pool, auth.building_id).await?;
    json_value(rows)
}

/// Get a single report by id.
pub async fn get(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let row = db::report::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    json_value(row)
}

/// Create a new report. Any authenticated user (all roles) can open a report.
pub async fn create(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(input): Json<db::report::CreateReportInput>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    if input.title.trim().is_empty() {
        return Err(AppError::validation_code("errors.reports.title_required"));
    }
    let row = db::report::create(&state.pool, auth.building_id, auth.user_id, input).await?;
    Ok((StatusCode::CREATED, json_value(row)?))
}

#[derive(Deserialize)]
pub struct UpdateStatusInput {
    pub status: String,
}

#[derive(Deserialize, Default)]
pub struct ConvertToTaskInput {
    pub priority: Option<String>,
}

/// Update report status. Only the manager (syndic) may change a report's status.
pub async fn update_status(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateStatusInput>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    if !db::report::validate_status(&input.status) {
        return Err(AppError::validation_code("errors.reports.invalid_status"));
    }
    db::report::update_status(&state.pool, id, auth.building_id, &input.status).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Convert a report into a task (EP-011-S03).
/// Only the manager (syndic) may convert a report into a task.
/// Returns the newly created task id.
pub async fn convert_to_task(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    input: Option<Json<ConvertToTaskInput>>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_role(&Role::Manager)?;
    let priority = input
        .and_then(|Json(body)| body.priority)
        .unwrap_or_else(|| db::task::DEFAULT_PRIORITY.to_string());
    if !db::task::validate_priority(&priority) {
        return Err(AppError::validation_code("errors.reports.invalid_priority"));
    }
    let task_id =
        db::report::convert_to_task(&state.pool, id, auth.building_id, auth.user_id, &priority)
            .await?;
    json_value(serde_json::json!({ "task_id": task_id }))
}

/// Update a report's title, description, and unit. Manager/board only.
pub async fn update(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(mut input): Json<db::report::UpdateReportInput>,
) -> AppResult<StatusCode> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    let trimmed = input.title.trim().to_string();
    if trimmed.is_empty() {
        return Err(AppError::validation_code("errors.reports.title_required"));
    }
    input.title = trimmed;
    db::report::update(&state.pool, id, auth.building_id, input).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Delete a report. Manager/board only.
pub async fn delete(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    db::report::delete(&state.pool, id, auth.building_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Document (photo) attachment ───────────────────────────────────

/// GET /reports/{id}/documents — list documents attached to a report.
/// All authenticated roles can list documents.
pub async fn list_documents(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    // Verify the report belongs to the building
    db::report::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let docs = db::report::list_documents(&state.pool, id, auth.building_id).await?;
    json_value(docs)
}

#[derive(Deserialize)]
pub struct AttachDocumentInput {
    pub document_id: Uuid,
}

/// POST /reports/{id}/documents — attach an existing document to a report.
/// Any authenticated role (author of the report may attach their own photos).
pub async fn attach_document(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<AttachDocumentInput>,
) -> AppResult<StatusCode> {
    let attached =
        db::report::attach_document(&state.pool, id, input.document_id, auth.building_id).await?;
    if !attached {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /reports/{id}/documents/{doc_id} — detach a document from a report.
/// Only managers and board members can detach documents.
pub async fn detach_document(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, doc_id)): Path<(Uuid, Uuid)>,
) -> AppResult<StatusCode> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    let detached = db::report::detach_document(&state.pool, id, doc_id, auth.building_id).await?;
    if !detached {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
