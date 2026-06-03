use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    db::{self, AppState},
    domain::{AuthUser, Role, owners},
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

#[derive(sqlx::FromRow)]
struct LevyUnitChargeRow {
    levy_call_id: Uuid,
    fiscal_year: i32,
    call_label: String,
    due_date: chrono::NaiveDate,
    issued_at: DateTime<Utc>,
    total_amount: rust_decimal::Decimal,
    charge_amount: rust_decimal::Decimal,
    works_fund_amount: rust_decimal::Decimal,
    status: String,
    paid_at: Option<DateTime<Utc>>,
}

#[derive(sqlx::FromRow)]
struct LevyBreakdownRow {
    levy_call_id: Uuid,
    budget_line_id: Option<Uuid>,
    budget_line_label: Option<String>,
    key_id: Option<Uuid>,
    key_name: Option<String>,
    shares_snapshot: Option<i32>,
    shares_total_snapshot: Option<i32>,
    amount: rust_decimal::Decimal,
    is_works_fund: bool,
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
        .map_err(|_| AppError::validation_code("errors.units.invalid_payload"))?;

    // Pre-check: is the number already taken by an archived unit?
    if db::unit::is_archived(&state.pool, auth.building_id, &input.unit_number).await? {
        return Err(AppError::conflict_code("errors.units.unit_number_archived"));
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
            AppError::conflict_code("errors.units.unit_number_exists")
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
        .map_err(|_| AppError::validation_code("errors.units.invalid_payload"))?;

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

/// GET /units/{id}/owners — ownership history (EP-002-S02, US-GP-007)
pub async fn owners(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<db::unit::OwnerRecord>>> {
    // Managers and board members only (AC-GP-007-B)
    if !auth.roles.iter().any(|r| *r == Role::Manager || *r == Role::BoardMember) {
        return Err(AppError::Forbidden);
    }
    // Verify unit belongs to this building
    db::unit::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let records = db::unit::list_owners(&state.pool, id, auth.building_id).await?;
    Ok(Json(records))
}

#[derive(Deserialize, Validate)]
pub struct TransferInput {
    pub user_id: Uuid,
    pub start_date: NaiveDate,
}

/// POST /units/{id}/transfer — ownership transfer (EP-002-S02, US-GP-003)
pub async fn transfer(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<TransferInput>,
) -> AppResult<StatusCode> {
    require_manager(&auth)?;

    let today = chrono::Utc::now().naive_utc().date();
    owners::validate_transfer_date(input.start_date, today)
        .map_err(AppError::validation_code)?;

    // Verify unit belongs to this building
    db::unit::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Verify new owner exists and belongs to this building
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM user_building WHERE user_id = $1 AND building_id = $2)",
    )
    .bind(input.user_id)
    .bind(auth.building_id)
    .fetch_one(&state.pool)
    .await?;

    if !exists {
        return Err(AppError::validation_code("errors.owners.new_owner_not_found"));
    }

    db::unit::transfer_ownership(&state.pool, id, auth.building_id, input.user_id, input.start_date)
        .await?;

    Ok(StatusCode::NO_CONTENT)
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

    let rows = sqlx::query_as::<_, LevyUnitChargeRow>(
        r#"SELECT
               lc.id                                                         AS levy_call_id,
               b.fiscal_year,
               lc.call_label,
               lc.due_date,
               lc.issued_at,
               SUM(lcu.amount)                                               AS total_amount,
               SUM(lcu.amount) FILTER (WHERE lcu.budget_line_id IS NOT NULL) AS charge_amount,
               SUM(lcu.amount) FILTER (WHERE lcu.budget_line_id IS NULL)     AS works_fund_amount,
               CASE
                   WHEN bool_or(lcu.status::text = 'overdue') THEN 'overdue'
                   WHEN bool_and(lcu.status::text = 'paid')   THEN 'paid'
                   ELSE 'issued'
               END                                                           AS status,
               MAX(lcu.paid_at)                                              AS paid_at
           FROM levy_call lc
           JOIN budget b           ON b.id             = lc.budget_id AND b.building_id = $2
           JOIN levy_call_unit lcu ON lcu.levy_call_id = lc.id        AND lcu.unit_id   = $1
           GROUP BY lc.id, b.fiscal_year, lc.call_label, lc.due_date, lc.issued_at
           ORDER BY lc.due_date DESC"#,
    )
    .bind(id)
    .bind(auth.building_id)
    .fetch_all(&state.pool)
    .await?;

    let breakdown_rows = sqlx::query_as::<_, LevyBreakdownRow>(
        r#"SELECT
               lcu.levy_call_id,
               lcu.budget_line_id,
               bl.label                      AS budget_line_label,
               lcu.key_id,
               ak.name                       AS key_name,
               lcu.shares_snapshot,
               lcu.shares_total_snapshot,
               lcu.amount,
               (lcu.budget_line_id IS NULL)  AS is_works_fund
           FROM levy_call_unit lcu
           JOIN levy_call lc ON lc.id = lcu.levy_call_id
           JOIN budget b     ON b.id  = lc.budget_id AND b.building_id = $2
           LEFT JOIN budget_line bl  ON bl.id  = lcu.budget_line_id
           LEFT JOIN allocation_key ak ON ak.id = lcu.key_id
           WHERE lcu.unit_id = $1
           ORDER BY lcu.levy_call_id, bl.label NULLS LAST"#,
    )
    .bind(id)
    .bind(auth.building_id)
    .fetch_all(&state.pool)
    .await?;

    let mut breakdown_map: HashMap<Uuid, Vec<serde_json::Value>> = HashMap::new();
    for b in breakdown_rows {
        breakdown_map
            .entry(b.levy_call_id)
            .or_default()
            .push(serde_json::json!({
                "budget_line_id":        b.budget_line_id,
                "budget_line_label":     b.budget_line_label,
                "key_id":               b.key_id,
                "key_name":             b.key_name,
                "shares_snapshot":      b.shares_snapshot,
                "shares_total_snapshot": b.shares_total_snapshot,
                "amount":               b.amount,
                "is_works_fund":        b.is_works_fund,
            }));
    }

    let total_issued: rust_decimal::Decimal = rows.iter().map(|r| r.total_amount).sum();
    let total_paid: rust_decimal::Decimal = rows
        .iter()
        .filter(|r| r.status == "paid")
        .map(|r| r.total_amount)
        .sum();
    let balance = total_issued - total_paid;

    let levies: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            let breakdown = breakdown_map.remove(&r.levy_call_id).unwrap_or_default();
            serde_json::json!({
                "id":                 r.levy_call_id,
                "fiscal_year":        r.fiscal_year,
                "total_amount":       r.total_amount,
                "charge_amount":      r.charge_amount,
                "works_fund_amount":  r.works_fund_amount,
                "call_label":         r.call_label,
                "due_date":           r.due_date,
                "status":             r.status,
                "issued_at":          r.issued_at,
                "paid_at":            r.paid_at,
                "breakdown":          breakdown,
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
