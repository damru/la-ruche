use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::NaiveDate;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    db::{self, AppState, claim::CLAIM_STATUS_OPEN},
    domain::{AuthUser, Role},
    errors::{AppError, AppResult, json_value},
};

pub async fn list(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let include_drafts = auth.has_role(&Role::Manager);
    let claims = db::claim::list(&state.pool, auth.building_id, include_drafts).await?;
    json_value(claims)
}

pub async fn get(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let claim = db::claim::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    // Drafts are invisible to non-managers (404, not 403 — do not reveal existence).
    if claim.status == db::claim::CLAIM_STATUS_DRAFT && !auth.has_role(&Role::Manager) {
        return Err(AppError::NotFound);
    }
    json_value(claim)
}

pub async fn create(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(mut input): Json<db::claim::CreateClaimInput>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    input.title = input.title.trim().to_string();
    let claim = db::claim::create(&state.pool, auth.building_id, auth.user_id, input).await?;
    Ok((StatusCode::CREATED, json_value(claim)?))
}

/// Asserts the claim is mutable (not transmitted) — shared by update/delete.
fn assert_claim_mutable(existing: &db::claim::ClaimRow) -> AppResult<()> {
    let imm_status = if existing.transmitted_at.is_some() {
        "transmitted"
    } else {
        CLAIM_STATUS_OPEN
    };
    crate::domain::immutability::assert_mutable(
        &crate::domain::immutability::Entity::Claim,
        imm_status,
    )
}

pub async fn update(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(mut input): Json<db::claim::UpdateClaimInput>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_role(&Role::Manager)?;
    let existing = db::claim::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    assert_claim_mutable(&existing)?;
    input.title = input.title.trim().to_string();
    let claim = db::claim::update(&state.pool, id, auth.building_id, input)
        .await?
        .ok_or(AppError::NotFound)?;
    json_value(claim)
}

pub async fn delete(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    let existing = db::claim::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    assert_claim_mutable(&existing)?;
    let deleted = db::claim::delete(&state.pool, id, auth.building_id).await?;
    if deleted.is_none() {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct DeclareClaimInput {
    pub declared_at: Option<NaiveDate>,
}

pub async fn declare(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    body: Option<Json<DeclareClaimInput>>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_role(&Role::Manager)?;
    let declared_at = body.and_then(|Json(input)| input.declared_at);
    let claim = db::claim::declare(&state.pool, id, auth.building_id, declared_at).await?;
    json_value(claim)
}

pub async fn list_documents(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    db::claim::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let documents = db::claim::list_documents(&state.pool, id, auth.building_id).await?;
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
        db::claim::attach_document(&state.pool, id, input.document_id, auth.building_id).await?;
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
        db::claim::detach_document(&state.pool, id, document_id, auth.building_id).await?;
    if !detached {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct CreateNoteInput {
    pub body: String,
}

pub async fn create_note(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateNoteInput>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    let note = db::claim_note::create(&state.pool, id, auth.building_id, auth.user_id, &input.body)
        .await?;
    Ok((StatusCode::CREATED, json_value(note)?))
}

pub async fn list_notes(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    let notes = db::claim_note::list(&state.pool, id, auth.building_id).await?;
    json_value(notes)
}
