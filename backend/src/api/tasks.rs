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
    let rows = db::task::list(&state.pool, auth.building_id).await?;
    json_value(rows)
}

pub async fn get(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let t = db::task::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    json_value(t)
}

pub async fn create(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(input): Json<db::task::CreateTaskInput>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    let t = db::task::create(&state.pool, auth.building_id, auth.user_id, input).await?;
    Ok((StatusCode::CREATED, json_value(t)?))
}

#[derive(Deserialize)]
pub struct UpdateTaskInput {
    pub status: String,
}

pub async fn update(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateTaskInput>,
) -> AppResult<StatusCode> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    db::task::update_status(&state.pool, id, auth.building_id, &input.status).await?;
    Ok(StatusCode::NO_CONTENT)
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
    let url = format!("{}/vendor/{}", state.cfg.app.public_url, token);
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
