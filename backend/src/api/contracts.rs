use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    api::documents::DocumentListItem,
    db::{self, AppState},
    domain::{AuthUser, Role},
    errors::{AppError, AppResult, json_value},
};

#[derive(Deserialize)]
pub struct ListContractsQuery {
    pub status: Option<String>,
    pub contract_type: Option<String>,
    pub vendor: Option<String>,
}

pub async fn list(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListContractsQuery>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    let filter = db::contract::ListContractsFilter {
        status: query.status,
        contract_type: query.contract_type,
        vendor_name: query.vendor,
    };
    json_value(db::contract::list(&state.pool, auth.building_id, &filter).await?)
}

pub async fn list_vendors(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<String>>> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    Ok(Json(
        db::contract::vendor_names(&state.pool, auth.building_id).await?,
    ))
}

pub async fn list_types(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<String>>> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    Ok(Json(db::contract::contract_types(&state.pool).await?))
}

pub async fn list_all_reminders(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    json_value(db::contract::list_all_reminders(&state.pool, auth.building_id).await?)
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
    let current = db::contract::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;

    use crate::domain::immutability::{Entity, is_mutable};
    if !is_mutable(&Entity::VendorContract, &current.status) {
        let frozen_field_changed = input.amount != current.amount
            || input.start_date != current.start_date
            || input.end_date != current.end_date
            || input.notice_days != current.notice_days
            || input.contract_type != current.contract_type
            || input.payment_frequency != current.payment_frequency
            || input.auto_renewal != current.auto_renewal;
        if frozen_field_changed {
            return Err(AppError::validation_code("errors.immutability.frozen"));
        }
    }

    let c = db::contract::update(&state.pool, id, auth.building_id, input)
        .await?
        .ok_or(AppError::NotFound)?;
    json_value(c)
}

pub async fn create_amendment(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<db::contract::CreateAmendmentInput>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    auth.require_role(&Role::Manager)?;
    let created =
        db::contract::create_amendment(&state.pool, id, auth.building_id, auth.user_id, input)
            .await?
            .ok_or(AppError::NotFound)?;
    Ok((StatusCode::CREATED, json_value(created)?))
}

pub async fn activate(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_role(&Role::Manager)?;
    let activated = db::contract::activate(&state.pool, id, auth.building_id, auth.user_id)
        .await?
        .ok_or(AppError::NotFound)?;
    json_value(activated)
}

pub async fn amendment_chain(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    json_value(db::contract::amendment_chain(&state.pool, id, auth.building_id).await?)
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

pub async fn delete(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    db::contract::delete(&state.pool, id, auth.building_id).await?;
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
) -> AppResult<Json<Vec<DocumentListItem>>> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    let roles: Vec<String> = auth.roles.iter().map(ToString::to_string).collect();
    let documents = db::contract::list_documents(&state.pool, id, auth.building_id, &roles).await?;
    Ok(Json(
        documents.into_iter().map(DocumentListItem::from).collect(),
    ))
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
