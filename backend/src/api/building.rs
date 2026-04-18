use std::sync::Arc;

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::{
    db::{self, AppState},
    domain::AuthUser,
    errors::{AppError, AppResult},
};

// ── DTOs ──────────────────────────────────────────────────────────

#[derive(Serialize, ToSchema)]
pub struct BuildingResponse {
    pub id: String,
    pub name: String,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub postal_code: String,
    pub city: String,
    pub manager_siret: Option<String>,
    pub shares_total: String,
    pub bylaws_date: Option<String>,
    pub created_at: String,
}

#[derive(Serialize, ToSchema)]
pub struct BuildingStatsResponse {
    pub name: String,
    pub nb_lots: i64,
    pub nb_users: i64,
    pub nb_tenants: i64,
    pub shares_total: String,
    pub works_fund_available: String,
    pub works_fund_pct: Option<i32>,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct UpdateBuildingInput {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[validate(length(min = 1, max = 200))]
    pub address_line1: String,
    pub address_line2: Option<String>,
    #[validate(length(min = 1, max = 10))]
    pub postal_code: String,
    #[validate(length(min = 1, max = 100))]
    pub city: String,
    pub manager_siret: Option<String>,
    pub bylaws_date: Option<chrono::NaiveDate>,
}

// ── Handlers ──────────────────────────────────────────────────────

/// GET /building
pub async fn get(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> AppResult<Json<BuildingResponse>> {
    let row = db::building::get(&state.pool, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(BuildingResponse {
        id:             row.id.to_string(),
        name:           row.name,
        address_line1:  row.address_line1,
        address_line2:  row.address_line2,
        postal_code:    row.postal_code,
        city:           row.city,
        manager_siret:  row.manager_siret,
        shares_total:   row.shares_total.to_string(),
        bylaws_date:    row.bylaws_date.map(|d: chrono::NaiveDate| d.to_string()),
        created_at:     row.created_at.to_rfc3339(),
    }))
}

/// GET /building/stats
pub async fn stats(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> AppResult<Json<BuildingStatsResponse>> {
    let building = db::building::get(&state.pool, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let s = db::building::get_stats(&state.pool, auth.building_id).await?;

    Ok(Json(BuildingStatsResponse {
        name:                 building.name,
        nb_lots:              s.nb_units,
        nb_users:             s.nb_users,
        nb_tenants:           s.nb_tenants,
        shares_total:         s.shares_total.to_string(),
        works_fund_available: s.works_fund_available.to_string(),
        works_fund_pct:       s.works_fund_pct,
    }))
}

/// PUT /building
pub async fn update(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(input): Json<UpdateBuildingInput>,
) -> AppResult<Json<BuildingResponse>> {
    if !auth.roles.iter().any(|r| *r == crate::domain::Role::Manager) {
        return Err(AppError::Forbidden);
    }

    input.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    let row = db::building::update(
        &state.pool,
        auth.building_id,
        db::building::UpdateBuildingInput {
            name:          input.name,
            address_line1: input.address_line1,
            address_line2: input.address_line2,
            postal_code:   input.postal_code,
            city:          input.city,
            manager_siret: input.manager_siret,
            bylaws_date:   input.bylaws_date,
        },
    )
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(BuildingResponse {
        id:             row.id.to_string(),
        name:           row.name,
        address_line1:  row.address_line1,
        address_line2:  row.address_line2,
        postal_code:    row.postal_code,
        city:           row.city,
        manager_siret:  row.manager_siret,
        shares_total:   row.shares_total.to_string(),
        bylaws_date:    row.bylaws_date.map(|d: chrono::NaiveDate| d.to_string()),
        created_at:     row.created_at.to_rfc3339(),
    }))
}

// ── Allocation keys (stubs — to be implemented) ────────────────────

pub async fn list_keys() -> AppResult<axum::http::StatusCode> {
    Ok(axum::http::StatusCode::NOT_IMPLEMENTED)
}

pub async fn create_key() -> AppResult<axum::http::StatusCode> {
    Ok(axum::http::StatusCode::NOT_IMPLEMENTED)
}

pub async fn update_key() -> AppResult<axum::http::StatusCode> {
    Ok(axum::http::StatusCode::NOT_IMPLEMENTED)
}

pub async fn delete_key() -> AppResult<axum::http::StatusCode> {
    Ok(axum::http::StatusCode::NOT_IMPLEMENTED)
}
