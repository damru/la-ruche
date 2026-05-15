use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{Datelike, NaiveDate};
use serde::Deserialize;
use sqlx::types::Decimal;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    db::{self, AppState},
    domain::{
        AuthUser, Role,
        charges::{UnitWithShares, compute_levies},
    },
    errors::{AppError, AppResult},
};

pub async fn list(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let budgets = db::budget::list(&state.pool, auth.building_id).await?;
    Ok(Json(
        serde_json::to_value(budgets).map_err(|e| AppError::Internal(e.into()))?,
    ))
}

pub async fn get(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let b = db::budget::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(
        serde_json::to_value(b).map_err(|e| AppError::Internal(e.into()))?,
    ))
}

#[derive(Deserialize)]
pub struct CreateBudgetInput {
    pub fiscal_year: i32,
}

pub async fn create(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateBudgetInput>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    auth.require_role(&Role::Manager)?;
    let b = db::budget::create(&state.pool, auth.building_id, input.fiscal_year).await?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(b).map_err(|e| AppError::Internal(e.into()))?),
    ))
}

pub async fn update(
    auth: AuthUser,
    State(_): State<Arc<AppState>>,
    Path(_id): Path<Uuid>,
    Json(_input): Json<serde_json::Value>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn close(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    let closed = db::budget::close(&state.pool, id, auth.building_id).await?;
    if closed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::conflict_code("errors.budget.exercise_already_closed"))
    }
}

pub async fn list_lines(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let lines = db::budget::list_lines(&state.pool, id, auth.building_id).await?;
    Ok(Json(
        serde_json::to_value(lines).map_err(|e| AppError::Internal(e.into()))?,
    ))
}

#[derive(Deserialize)]
pub struct CreateLineInput {
    pub label: String,
    pub key_id: Uuid,
    pub budgeted_amount: Decimal,
}

pub async fn create_line(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateLineInput>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    auth.require_role(&Role::Manager)?;
    db::budget::ensure_budget_is_mutable(&state.pool, id, auth.building_id).await?;
    let l = db::budget::create_line(
        &state.pool,
        id,
        &input.label,
        input.key_id,
        input.budgeted_amount,
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(l).map_err(|e| AppError::Internal(e.into()))?),
    ))
}

pub async fn update_line(
    auth: AuthUser,
    State(_): State<Arc<AppState>>,
    Path((_, _)): Path<(Uuid, Uuid)>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_line(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((budget_id, line_id)): Path<(Uuid, Uuid)>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    db::budget::ensure_budget_is_mutable(&state.pool, budget_id, auth.building_id).await?;
    db::budget::delete_line(&state.pool, line_id, budget_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/budgets/:id/appels/generer
/// Generates levy calls by shares via domain/charges.rs and inserts them
#[derive(Deserialize)]
pub struct GenerateLeviesInput {
    pub due_date: NaiveDate,
    pub include_works_fund: bool,
    pub call_label: Option<String>,
}

pub async fn generate_levies(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<GenerateLeviesInput>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_role(&Role::Manager)?;
    db::budget::ensure_budget_is_mutable(&state.pool, id, auth.building_id).await?;

    let _budget = db::budget::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let lines = db::budget::list_lines(&state.pool, id, auth.building_id).await?;

    let units_data = db::allocation_key::list_general_shares(&state.pool, auth.building_id).await?;

    let total_shares: i64 = units_data.iter().map(|u| u.shares as i64).sum();

    let total_charges: Decimal = lines.iter().map(|l| l.budgeted_amount).sum();

    let works_fund_amount = if input.include_works_fund {
        total_charges * Decimal::new(5, 2)
    } else {
        Decimal::ZERO
    };

    let total_amount = total_charges + works_fund_amount;
    let already_called = db::budget::total_levy_amount(&state.pool, id, auth.building_id).await?;
    let remaining_amount = (total_amount - already_called).max(Decimal::ZERO);

    if remaining_amount <= Decimal::ZERO {
        return Ok(Json(serde_json::json!({
            "generated": 0,
            "total_amount": total_amount,
            "already_called": already_called,
            "remaining_amount": Decimal::ZERO,
            "due_date": input.due_date,
        })));
    }

    let units_with_shares: Vec<UnitWithShares> = units_data
        .iter()
        .map(|u| UnitWithShares {
            unit_id: u.unit_id,
            key_id: u.key_id,
            shares: u.shares as i64,
        })
        .collect();

    let computed_levies = compute_levies(remaining_amount, &units_with_shares, total_shares);
    let now = chrono::Utc::now();
    let default_label = format!(
        "Trimestriel T{} {}",
        ((now.month0() / 3) + 1),
        now.year()
    );
    let call_label = input
        .call_label
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&default_label);
    let levy_inserts: Vec<db::budget::LevyInsert> = computed_levies
        .iter()
        .map(|l| db::budget::LevyInsert {
            unit_id: l.unit_id,
            key_id: l.key_id,
            amount: l.amount,
            shares_snapshot: l.shares_snapshot,
            shares_total_snapshot: l.shares_total_snapshot,
        })
        .collect();

    let nb = db::budget::generate_levies(
        &state.pool,
        id,
        auth.building_id,
        input.due_date,
        call_label,
        &levy_inserts,
    )
    .await?
    .len();

    Ok(Json(serde_json::json!({
        "generated": nb,
        "total_amount": total_amount,
        "already_called": already_called,
        "remaining_amount": remaining_amount,
        "call_label": call_label,
        "due_date": input.due_date,
    })))
}

pub async fn list_levies(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let levies = db::budget::list_levies(&state.pool, id, auth.building_id).await?;
    Ok(Json(
        serde_json::to_value(levies).map_err(|e| AppError::Internal(e.into()))?,
    ))
}

pub async fn mark_paid(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    db::budget::ensure_levy_budget_is_mutable(&state.pool, id, auth.building_id).await?;
    db::budget::mark_paid(&state.pool, id, auth.building_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_works_fund(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let fiscal_year = chrono::Utc::now()
        .format("%Y")
        .to_string()
        .parse::<i32>()
        .unwrap_or(2025);
    let ft = db::budget::get_works_fund(&state.pool, auth.building_id, fiscal_year).await?;
    Ok(Json(ft.unwrap_or(serde_json::json!(null))))
}

pub async fn get_works_fund(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(fiscal_year): Path<i32>,
) -> AppResult<Json<serde_json::Value>> {
    let ft = db::budget::get_works_fund(&state.pool, auth.building_id, fiscal_year).await?;
    Ok(Json(ft.ok_or(AppError::NotFound)?))
}
