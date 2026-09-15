// ── api/calendar_events.rs — Événements manuels du calendrier (EP-014-S03)
//
// Handlers with no business logic, guard `require_any_role` on writes;
// reads are open to every authenticated role (AC-CAL-005-D).

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{NaiveDate, NaiveTime};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    db::{self, AppState, calendar_event::CalendarEventCategory},
    domain::{AuthUser, Role},
    errors::{AppError, AppResult, json_value},
};

#[derive(Deserialize)]
pub struct CreateCalendarEventInput {
    pub title: String,
    pub description: Option<String>,
    pub event_date: NaiveDate,
    pub event_time: Option<NaiveTime>,
    pub category: Option<CalendarEventCategory>,
}

#[derive(Deserialize)]
pub struct UpdateCalendarEventInput {
    pub title: String,
    pub description: Option<String>,
    pub event_date: NaiveDate,
    pub event_time: Option<NaiveTime>,
    pub category: Option<CalendarEventCategory>,
}

/// All authenticated roles can list manual events (AC-CAL-005-D).
pub async fn list(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let events = db::calendar_event::list(&state.pool, auth.building_id).await?;
    json_value(events)
}

/// All authenticated roles can view a single manual event.
pub async fn get(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let event = db::calendar_event::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    json_value(event)
}

/// Only Manager and BoardMember can create manual events (AC-CAL-005-E).
pub async fn create(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateCalendarEventInput>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;

    if input.title.trim().is_empty() {
        return Err(AppError::validation_code(
            "errors.calendar_event.title_required",
        ));
    }

    let event = db::calendar_event::create(
        &state.pool,
        auth.building_id,
        auth.user_id,
        &input.title,
        input.description.as_deref(),
        input.event_date,
        input.event_time,
        input.category,
    )
    .await?;

    Ok((StatusCode::CREATED, json_value(event)?))
}

/// Manager and BoardMember can edit any manual event of their building
/// (AC-CAL-005-E) — not only the ones they created.
pub async fn update(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateCalendarEventInput>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;

    if input.title.trim().is_empty() {
        return Err(AppError::validation_code(
            "errors.calendar_event.title_required",
        ));
    }

    let event = db::calendar_event::update(
        &state.pool,
        id,
        auth.building_id,
        &input.title,
        input.description.as_deref(),
        input.event_date,
        input.event_time,
        input.category,
    )
    .await?
    .ok_or(AppError::NotFound)?;

    json_value(event)
}

/// Manager and BoardMember can delete any manual event of their building.
pub async fn remove(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    db::calendar_event::delete(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_documents(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    db::calendar_event::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let documents = db::calendar_event::list_documents(&state.pool, id, auth.building_id).await?;
    json_value(documents)
}

#[derive(Deserialize)]
pub struct AttachDocumentInput {
    pub document_id: Uuid,
}

pub async fn attach_document(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<AttachDocumentInput>,
) -> AppResult<StatusCode> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    let attached =
        db::calendar_event::attach_document(&state.pool, id, input.document_id, auth.building_id)
            .await?;
    if !attached {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn detach_document(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, document_id)): Path<(Uuid, Uuid)>,
) -> AppResult<StatusCode> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    let detached =
        db::calendar_event::detach_document(&state.pool, id, document_id, auth.building_id).await?;
    if !detached {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
