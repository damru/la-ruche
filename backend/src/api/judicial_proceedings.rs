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
    let proceedings = db::judicial_proceeding::list(&state.pool, auth.building_id).await?;
    json_value(proceedings)
}

pub async fn get(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let proceeding = db::judicial_proceeding::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    json_value(proceeding)
}

pub async fn create(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(input): Json<db::judicial_proceeding::CreateProceedingInput>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    auth.require_role(&Role::Manager)?;
    let proceeding =
        db::judicial_proceeding::create(&state.pool, auth.building_id, auth.user_id, input).await?;
    Ok((StatusCode::CREATED, json_value(proceeding)?))
}

pub async fn update(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<db::judicial_proceeding::UpdateProceedingInput>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_role(&Role::Manager)?;
    let proceeding = db::judicial_proceeding::update(&state.pool, id, auth.building_id, input)
        .await?
        .ok_or(AppError::NotFound)?;
    json_value(proceeding)
}

pub async fn delete(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    let deleted = db::judicial_proceeding::delete(&state.pool, id, auth.building_id).await?;
    if !deleted {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
