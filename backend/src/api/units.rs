use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    db::{self, AppState},
    domain::{AuthUser, Role},
    errors::{AppError, AppResult},
};

// ── DTOs ──────────────────────────────────────────────────────────

#[derive(Serialize, ToSchema)]
pub struct OwnerSummary {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(Serialize, ToSchema)]
pub struct UnitResponse {
    pub id: Uuid,
    pub building_id: Uuid,
    pub unit_number: String,
    pub unit_type: String,
    pub block: Option<String>,
    pub staircase: Option<String>,
    pub floor: Option<String>,
    pub description: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub current_owner: Option<OwnerSummary>,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct CreateUnitInput {
    #[validate(length(min = 1, max = 20))]
    pub unit_number: String,
    pub unit_type: String,
    pub block: Option<String>,
    pub staircase: Option<String>,
    pub floor: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct UpdateUnitInput {
    #[validate(length(min = 1, max = 20))]
    pub unit_number: String,
    pub unit_type: String,
    pub block: Option<String>,
    pub staircase: Option<String>,
    pub floor: Option<String>,
    pub description: Option<String>,
}

fn to_response(row: db::unit::UnitRow) -> UnitResponse {
    UnitResponse {
        id: row.id,
        building_id: row.building_id,
        unit_number: row.unit_number,
        unit_type: row.unit_type,
        block: row.block,
        staircase: row.staircase,
        floor: row.floor,
        description: row.description,
        active: row.active,
        created_at: row.created_at,
        current_owner: None,
    }
}

fn require_manager(auth: &AuthUser) -> AppResult<()> {
    if auth.roles.contains(&Role::Manager) {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

// ── Handlers ──────────────────────────────────────────────────────

/// GET /units
pub async fn list(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> AppResult<Json<Vec<UnitResponse>>> {
    let rows = db::unit::list(&state.pool, auth.building_id).await?;
    let unit_ids: Vec<Uuid> = rows.iter().map(|r| r.id).collect();
    let owner_rows = db::unit::list_current_owners(&state.pool, &unit_ids).await?;
    let mut owner_map: HashMap<Uuid, OwnerSummary> = owner_rows
        .into_iter()
        .map(|o| {
            (
                o.unit_id,
                OwnerSummary {
                    id: o.user_id,
                    first_name: o.first_name,
                    last_name: o.last_name,
                    email: o.email,
                },
            )
        })
        .collect();
    Ok(Json(
        rows.into_iter()
            .map(|row| {
                let id = row.id;
                let mut resp = to_response(row);
                resp.current_owner = owner_map.remove(&id);
                resp
            })
            .collect(),
    ))
}

/// POST /units
pub async fn create(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(input): Json<CreateUnitInput>,
) -> AppResult<(StatusCode, Json<UnitResponse>)> {
    require_manager(&auth)?;
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Pre-check: is the number already taken by an archived unit?
    if db::unit::is_archived(&state.pool, auth.building_id, &input.unit_number).await? {
        return Err(AppError::Conflict("UNIT_NUMBER_ARCHIVED".into()));
    }

    let row = db::unit::create(
        &state.pool,
        auth.building_id,
        db::unit::CreateUnitInput {
            unit_number: input.unit_number.clone(),
            unit_type: input.unit_type,
            block: input.block,
            staircase: input.staircase,
            floor: input.floor,
            description: input.description,
        },
    )
    .await
    .map_err(|e| match &e {
        AppError::Database(db_err)
            if db_err.as_database_error().and_then(|d| d.code()).as_deref() == Some("23505") =>
        {
            AppError::Conflict("UNIT_NUMBER_EXISTS".into())
        }
        _ => e,
    })?;

    Ok((StatusCode::CREATED, Json(to_response(row))))
}

/// GET /units/{id}
pub async fn get(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<UnitResponse>> {
    let row = db::unit::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(to_response(row)))
}

/// PUT /units/{id}
pub async fn update(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateUnitInput>,
) -> AppResult<Json<UnitResponse>> {
    require_manager(&auth)?;
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let row = db::unit::update(
        &state.pool,
        id,
        auth.building_id,
        db::unit::UpdateUnitInput {
            unit_number: input.unit_number,
            unit_type: input.unit_type,
            block: input.block,
            staircase: input.staircase,
            floor: input.floor,
            description: input.description,
        },
    )
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(to_response(row)))
}

/// DELETE /units/{id}
pub async fn archive(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    require_manager(&auth)?;
    db::unit::archive(&state.pool, id, auth.building_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /units/{id}/owners — EP-002
pub async fn owners(
    _state: State<Arc<AppState>>,
    _auth: AuthUser,
    _id: Path<Uuid>,
) -> AppResult<StatusCode> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

/// POST /units/{id}/transfer — EP-002
pub async fn transfer(
    _state: State<Arc<AppState>>,
    _auth: AuthUser,
    _id: Path<Uuid>,
) -> AppResult<StatusCode> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

/// GET /units/{id}/charges — EP-004-S06
/// Individual charge statement: all levy_calls for this unit with snapshot data.
/// Co-owners can only access units they currently own; managers and board members
/// can access any unit in their building.
pub async fn charges(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    // Verify the unit belongs to the caller's building
    let unit = db::unit::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Co-owners (without manager/board_member) may only view their own unit's charges
    let is_privileged = auth
        .roles
        .iter()
        .any(|r| *r == Role::Manager || *r == Role::BoardMember);
    if !is_privileged {
        let owns: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM unit_owner WHERE unit_id = $1 AND user_id = $2 AND end_date IS NULL)"
        )
        .bind(id)
        .bind(auth.user_id)
        .fetch_one(&state.pool)
        .await?;
        if !owns {
            return Err(AppError::Forbidden);
        }
    }

    let rows = sqlx::query!(
        r#"SELECT
               lc.id,
               b.fiscal_year,
               lc.amount,
               lc.due_date,
               lc.status::text            AS status,
               lc.issued_at,
               lc.paid_at,
               lc.key_id,
               ak.name                    AS key_name,
               lc.shares_snapshot,
               lc.shares_total_snapshot
           FROM levy_call lc
           JOIN budget b ON b.id = lc.budget_id AND b.building_id = $2
           LEFT JOIN allocation_key ak ON ak.id = lc.key_id
           WHERE lc.unit_id = $1
           ORDER BY lc.due_date DESC"#,
        id,
        auth.building_id
    )
    .fetch_all(&state.pool)
    .await?;

    let total_issued: rust_decimal::Decimal = rows.iter().map(|r| r.amount).sum();
    let total_paid: rust_decimal::Decimal = rows
        .iter()
        .filter(|r| r.status.as_deref() == Some("paid"))
        .map(|r| r.amount)
        .sum();
    let balance = total_issued - total_paid;

    let levies: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id":                    r.id,
                "fiscal_year":           r.fiscal_year,
                "amount":                r.amount,
                "due_date":              r.due_date,
                "status":                r.status,
                "issued_at":             r.issued_at,
                "paid_at":               r.paid_at,
                "key_id":                r.key_id,
                "key_name":              r.key_name,
                "shares_snapshot":       r.shares_snapshot,
                "shares_total_snapshot": r.shares_total_snapshot,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "unit": {
            "id":          unit.id,
            "unit_number": unit.unit_number,
            "unit_type":   unit.unit_type,
        },
        "levies":       levies,
        "total_issued": total_issued,
        "total_paid":   total_paid,
        "balance":      balance,
    })))
}
