use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::types::Decimal;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    config::AppSettings,
    db::{self, AppState},
    domain::{AuthUser, Role},
    errors::{AppError, AppResult, json_value},
};

/// Builds the public vendor submission URL for a given token.
fn vendor_url(cfg: &AppSettings, token: Uuid) -> String {
    format!("{}/vendor/{}", cfg.public_url, token)
}

const DEFAULT_LIST_LIMIT: i64 = 50;
const MAX_LIST_LIMIT: i64 = 200;

#[derive(Debug, Deserialize)]
pub struct ListTasksQuery {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub contractor_id: Option<Uuid>,
    pub q: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListTasksQuery>,
) -> AppResult<Json<serde_json::Value>> {
    if let Some(ref status) = query.status
        && !db::task::validate_status(status)
    {
        return Err(AppError::validation_code("errors.tasks.invalid_filter"));
    }
    if let Some(ref priority) = query.priority
        && !db::task::validate_priority(priority)
    {
        return Err(AppError::validation_code("errors.tasks.invalid_filter"));
    }
    let limit = query
        .limit
        .unwrap_or(DEFAULT_LIST_LIMIT)
        .clamp(1, MAX_LIST_LIMIT);
    let offset = query.offset.unwrap_or(0).max(0);
    let filters = db::task::TaskListFilters {
        status: query.status,
        priority: query.priority,
        assigned_to: query.assigned_to,
        contractor_id: query.contractor_id,
        q: query.q,
        limit,
        offset,
    };
    let (rows, total) = db::task::list(&state.pool, auth.building_id, &filters).await?;
    let items: Vec<db::task::TaskListItem> = rows.into_iter().map(Into::into).collect();
    json_value(serde_json::json!({ "items": items, "total": total }))
}

pub async fn get(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let t = db::task::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    json_value(db::task::TaskListItem::from(t))
}

pub async fn list_documents(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    db::task::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let documents = db::task::list_documents(&state.pool, id, auth.building_id).await?;
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
        db::task::attach_document(&state.pool, id, input.document_id, auth.building_id).await?;
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
        db::task::detach_document(&state.pool, id, document_id, auth.building_id).await?;
    if !detached {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn create(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(input): Json<db::task::CreateTaskInput>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    if input.title.trim().is_empty() {
        return Err(AppError::validation_code("errors.tasks.title_required"));
    }
    if let Some(ref p) = input.priority
        && !db::task::validate_priority(p)
    {
        return Err(AppError::validation_code("errors.tasks.invalid_priority"));
    }
    let t = db::task::create(&state.pool, auth.building_id, auth.user_id, input).await?;
    Ok((
        StatusCode::CREATED,
        json_value(db::task::TaskListItem::from(t))?,
    ))
}

/// Serde helper: correctly maps absent → None, null → Some(None), value → Some(Some(v)).
/// Standard `Option<Option<T>>` treats both absent and null as None, which
/// breaks the "patch: keep if absent, clear if null" semantics.
mod double_option {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
    where
        T: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        Ok(Some(Option::deserialize(deserializer)?))
    }
}

#[derive(Deserialize)]
pub struct UpdateTaskInput {
    pub status: Option<String>,
    pub title: Option<String>,
    #[serde(default, deserialize_with = "double_option::deserialize")]
    pub description: Option<Option<String>>,
    pub priority: Option<String>,
    #[serde(default, deserialize_with = "double_option::deserialize")]
    pub contractor_id: Option<Option<Uuid>>,
    #[serde(default, deserialize_with = "double_option::deserialize")]
    pub contractor_name: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option::deserialize")]
    pub completion_cost: Option<Option<Decimal>>,
}

pub async fn update(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateTaskInput>,
) -> AppResult<StatusCode> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    let task = db::task::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    crate::domain::immutability::assert_mutable(
        &crate::domain::immutability::Entity::Task,
        &task.status,
    )?;
    if let Some(ref t) = input.title
        && t.trim().is_empty()
    {
        return Err(AppError::validation_code("errors.tasks.title_required"));
    }
    if let Some(ref s) = input.status
        && !db::task::validate_status(s)
    {
        return Err(AppError::validation_code("errors.tasks.invalid_status"));
    }
    if let Some(ref p) = input.priority
        && !db::task::validate_priority(p)
    {
        return Err(AppError::validation_code("errors.tasks.invalid_priority"));
    }
    db::task::update_fields(
        &state.pool,
        id,
        auth.building_id,
        input.title.as_deref(),
        input.description.as_ref().map(|d| d.as_deref()),
        input.priority.as_deref(),
        input.status.as_deref(),
        input.contractor_id,
        input.contractor_name.as_ref().map(|value| value.as_deref()),
        input.completion_cost,
        auth.user_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_task(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    let task = db::task::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    crate::domain::immutability::assert_mutable(
        &crate::domain::immutability::Entity::Task,
        &task.status,
    )?;
    db::task::delete(&state.pool, id, auth.building_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_vendor_token_url(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_role(&Role::Manager)?;
    let result = db::task::get_vendor_token(&state.pool, id, auth.building_id).await?;
    match result {
        Some((token, expires_at)) => {
            let url = vendor_url(&state.cfg.app, token);
            Ok(Json(
                serde_json::json!({ "url": url, "expires_at": expires_at }),
            ))
        }
        None => Err(AppError::NotFound),
    }
}

pub async fn generate_token(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_role(&Role::Manager)?;
    let (token, expires_at) = db::task::generate_token(
        &state.pool,
        id,
        auth.building_id,
        state.cfg.auth.vendor_token_ttl_days,
    )
    .await?;
    let url = vendor_url(&state.cfg.app, token);
    Ok(Json(
        serde_json::json!({ "token": token, "expires_at": expires_at, "url": url }),
    ))
}

pub async fn revoke_token(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    db::task::revoke_token(&state.pool, id, auth.building_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
