// ── api/contractors.rs ───────────────────────────────────────────────────────
use axum::{
    Json,
    extract::{Path, Query, State},
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

/// GET /contractors?q=… — list or search contractors (all authenticated users)
#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

pub async fn list(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let rows = if let Some(ref q) = params.q {
        if q.trim().is_empty() {
            db::contractor::list(&state.pool, auth.building_id).await?
        } else {
            db::contractor::search(&state.pool, auth.building_id, q).await?
        }
    } else {
        db::contractor::list(&state.pool, auth.building_id).await?
    };
    json_value(rows)
}

/// GET /contractors/:id — get a single contractor (all authenticated users)
pub async fn get(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let c = db::contractor::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    json_value(c)
}

/// POST /contractors — create a contractor (Manager / BoardMember only)
pub async fn create(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(input): Json<db::contractor::CreateContractorInput>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Name is required".into()));
    }
    let c = db::contractor::create(&state.pool, auth.building_id, input).await?;
    Ok((StatusCode::CREATED, json_value(c)?))
}

/// PUT /contractors/:id — update a contractor (Manager / BoardMember only)
pub async fn update(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<db::contractor::UpdateContractorInput>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    let c = db::contractor::update(&state.pool, id, auth.building_id, input)
        .await?
        .ok_or(AppError::NotFound)?;
    json_value(c)
}

/// DELETE /contractors/:id — delete a contractor (Manager / BoardMember only)
pub async fn delete(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    if !db::contractor::delete(&state.pool, id, auth.building_id).await? {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
