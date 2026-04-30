// ── api/logbook.rs — Carnet d'entretien (EP-009) ─────────────────
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::NaiveDate;
use serde::Deserialize;
use sqlx::types::Decimal;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    db::{self, AppState},
    domain::{AuthUser, Role},
    errors::{AppError, AppResult, json_value},
};

/// Input for manual logbook entry creation.
#[derive(Deserialize)]
pub struct CreateLogbookInput {
    pub title: String,
    pub description: Option<String>,
    pub intervention_date: NaiveDate,
    pub cost: Option<Decimal>,
    pub vendor_name: Option<String>,
    /// Optional link to a vendor contract (EP-009-S05).
    pub contract_id: Option<Uuid>,
    /// Optional link to a task (manual association).
    pub task_id: Option<Uuid>,
}

/// Input for updating a manual logbook entry.
#[derive(Deserialize)]
pub struct UpdateLogbookInput {
    pub title: String,
    pub description: Option<String>,
    pub intervention_date: NaiveDate,
    pub cost: Option<Decimal>,
    pub vendor_name: Option<String>,
    /// Optional link to a vendor contract (EP-009-S05).
    pub contract_id: Option<Uuid>,
}

/// All authenticated users can list the maintenance log.
pub async fn list(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let rows = db::maintenance_log::list(&state.pool, auth.building_id).await?;
    json_value(rows)
}

/// All authenticated users can view a single log entry.
pub async fn get(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let entry = db::maintenance_log::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    json_value(entry)
}

/// Only managers and board members can manually create log entries.
/// `source` is always forced to `manual` — automatic entries come from task closure.
pub async fn create(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateLogbookInput>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;

    if input.title.trim().is_empty() {
        return Err(AppError::Validation("Title is required".into()));
    }

    let entry = db::maintenance_log::create(
        &state.pool,
        auth.building_id,
        auth.user_id,
        &input.title,
        input.description.as_deref(),
        input.intervention_date,
        input.cost,
        input.vendor_name.as_deref(),
        input.contract_id,
        input.task_id,
    )
    .await?;

    Ok((StatusCode::CREATED, json_value(entry)?))
}

/// Only managers and board members can edit manual log entries.
/// Automatic entries (source = 'automatic') cannot be edited through this endpoint.
pub async fn update(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateLogbookInput>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;

    if input.title.trim().is_empty() {
        return Err(AppError::Validation("Title is required".into()));
    }

    let entry = db::maintenance_log::update(
        &state.pool,
        id,
        auth.building_id,
        &input.title,
        input.description.as_deref(),
        input.intervention_date,
        input.cost,
        input.vendor_name.as_deref(),
        input.contract_id,
    )
    .await?
    .ok_or(AppError::NotFound)?;

    json_value(entry)
}

/// Only managers can delete logbook entries.
pub async fn delete(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    db::maintenance_log::delete(&state.pool, id, auth.building_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
