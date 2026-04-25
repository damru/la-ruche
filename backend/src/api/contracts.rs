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

pub async fn list(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    json_value(db::contract::list(&state.pool, auth.building_id).await?)
}

pub async fn get(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    let c = db::contract::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    json_value(c)
}

pub async fn create(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(input): Json<db::contract::CreateVendorContractInput>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    auth.require_role(&Role::Manager)?;
    let c = db::contract::create(&state.pool, auth.building_id, auth.user_id, input).await?;
    Ok((StatusCode::CREATED, json_value(c)?))
}

pub async fn update(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<db::contract::CreateVendorContractInput>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_role(&Role::Manager)?;
    let c = db::contract::update(&state.pool, id, auth.building_id, input)
        .await?
        .ok_or(AppError::NotFound)?;
    json_value(c)
}

pub async fn terminate(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    db::contract::terminate(&state.pool, id, auth.building_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_reminders(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    json_value(db::contract::list_reminders(&state.pool, id, auth.building_id).await?)
}

pub async fn acknowledge_reminder(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, rid)): Path<(Uuid, Uuid)>,
) -> AppResult<StatusCode> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    db::contract::acknowledge_reminder(&state.pool, rid, id, auth.building_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_documents(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    json_value(db::contract::list_documents(&state.pool, id, auth.building_id).await?)
}

#[derive(Deserialize)]
pub struct AttachContractDocumentInput {
    pub document_id: Uuid,
}

pub async fn attach_document(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<AttachContractDocumentInput>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    let attached =
        db::contract::attach_document(&state.pool, id, input.document_id, auth.building_id).await?;
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
    auth.require_role(&Role::Manager)?;
    let detached =
        db::contract::detach_document(&state.pool, id, document_id, auth.building_id).await?;
    if !detached {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
