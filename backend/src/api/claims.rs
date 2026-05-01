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
    let claims = db::claim::list(&state.pool, auth.building_id).await?;
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

pub async fn update(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(mut input): Json<db::claim::UpdateClaimInput>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
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
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    let deleted_documents = db::claim::delete(&state.pool, id, auth.building_id).await?;
    let Some(deleted_documents) = deleted_documents else {
        return Err(AppError::NotFound);
    };
    let storage = state.storage.clone();
    tokio::spawn(async move {
        for (bucket, key) in deleted_documents {
            if let Err(error) = storage.delete_object(&bucket, &key).await {
                tracing::error!(error = %error, "S3 delete error");
            }
        }
    });
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_documents(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
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
