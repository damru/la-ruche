use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
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
    let mandates = db::governance_mandate::list(&state.pool, auth.building_id).await?;
    json_value(mandates)
}

pub async fn get(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let mandate = db::governance_mandate::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    json_value(mandate)
}

pub async fn create(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(input): Json<db::governance_mandate::CreateMandateInput>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    auth.require_role(&Role::Manager)?;
    let mandate =
        db::governance_mandate::create(&state.pool, auth.building_id, auth.user_id, input).await?;
    Ok((StatusCode::CREATED, json_value(mandate)?))
}

pub async fn update(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<db::governance_mandate::UpdateMandateInput>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_role(&Role::Manager)?;
    let mandate = db::governance_mandate::update(&state.pool, id, auth.building_id, input)
        .await?
        .ok_or(AppError::NotFound)?;
    json_value(mandate)
}

pub async fn delete(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    let deleted = db::governance_mandate::delete(&state.pool, id, auth.building_id).await?;
    if !deleted {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
