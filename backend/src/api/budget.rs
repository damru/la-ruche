use std::sync::Arc;
use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::Deserialize;
use uuid::Uuid;
use chrono::NaiveDate;
use sqlx::types::Decimal;

use crate::{
    db::{self, AppState},
    domain::{AuthUser, Role, charges::{compute_levies, UnitWithShares}},
    errors::{AppError, AppResult},
};

pub async fn list(auth: AuthUser, State(state): State<Arc<AppState>>) -> AppResult<Json<serde_json::Value>> {
    let budgets = db::budget::list(&state.pool, auth.building_id).await?;
    Ok(Json(serde_json::to_value(budgets).map_err(|e| AppError::Internal(e.into()))?))
}

pub async fn get(auth: AuthUser, State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> AppResult<Json<serde_json::Value>> {
    let b = db::budget::get(&state.pool, id, auth.building_id).await?.ok_or(AppError::NotFound)?;
    Ok(Json(serde_json::to_value(b).map_err(|e| AppError::Internal(e.into()))?))
}

#[derive(Deserialize)]
pub struct CreateBudgetInput { pub fiscal_year: i32 }

pub async fn create(auth: AuthUser, State(state): State<Arc<AppState>>, Json(input): Json<CreateBudgetInput>) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    auth.require_role(&Role::Manager)?;
    let b = db::budget::create(&state.pool, auth.building_id, input.fiscal_year).await?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(b).map_err(|e| AppError::Internal(e.into()))?)))
}

pub async fn update(auth: AuthUser, State(_): State<Arc<AppState>>, Path(_id): Path<Uuid>, Json(_input): Json<serde_json::Value>) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_lines(auth: AuthUser, State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> AppResult<Json<serde_json::Value>> {
    let lines = db::budget::list_lines(&state.pool, id, auth.building_id).await?;
    Ok(Json(serde_json::to_value(lines).map_err(|e| AppError::Internal(e.into()))?))
}

#[derive(Deserialize)]
pub struct CreateLineInput {
    pub label: String,
    pub key_id: Uuid,
    pub budgeted_amount: Decimal,
}

pub async fn create_line(auth: AuthUser, State(state): State<Arc<AppState>>, Path(id): Path<Uuid>, Json(input): Json<CreateLineInput>) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    auth.require_role(&Role::Manager)?;
    let l = db::budget::create_line(&state.pool, id, &input.label, input.key_id, input.budgeted_amount).await?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(l).map_err(|e| AppError::Internal(e.into()))?)))
}

pub async fn update_line(auth: AuthUser, State(_): State<Arc<AppState>>, Path((_, _)): Path<(Uuid, Uuid)>) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_line(auth: AuthUser, State(state): State<Arc<AppState>>, Path((budget_id, line_id)): Path<(Uuid, Uuid)>) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    db::budget::delete_line(&state.pool, line_id, budget_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/budgets/:id/appels/generer
/// Generates levy calls by shares via domain/charges.rs and inserts them
#[derive(Deserialize)]
pub struct GenerateLeviesInput {
    pub due_date: NaiveDate,
    pub include_works_fund: bool,
}

pub async fn generate_levies(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<GenerateLeviesInput>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_role(&Role::Manager)?;

    let _budget = db::budget::get(&state.pool, id, auth.building_id).await?.ok_or(AppError::NotFound)?;
    let lines = db::budget::list_lines(&state.pool, id, auth.building_id).await?;

    let units_data = sqlx::query!(
        r#"SELECT u.id as unit_id, uak.share_value as shares
           FROM unit u
           JOIN unit_allocation_key uak ON uak.unit_id = u.id
           JOIN allocation_key ak ON ak.id = uak.key_id AND ak.name = 'General'
           WHERE u.building_id = $1 AND u.active = true"#,
        auth.building_id
    ).fetch_all(&state.pool).await?;

    let total_shares: Decimal = units_data.iter().map(|u| u.shares).sum();

    let total_charges: Decimal = lines.iter().map(|l| l.budgeted_amount).sum();

    let works_fund_amount = if input.include_works_fund {
        total_charges * Decimal::new(5, 2)
    } else {
        Decimal::ZERO
    };

    let total_amount = total_charges + works_fund_amount;

    let units_with_shares: Vec<UnitWithShares> = units_data.iter().map(|u| UnitWithShares {
        unit_id: u.unit_id,
        shares: u.shares,
    }).collect();

    let computed_levies = compute_levies(total_amount, &units_with_shares, total_shares);
    let levy_inserts: Vec<(Uuid, Decimal)> = computed_levies.iter().map(|l| (l.unit_id, l.amount)).collect();

    let nb = db::budget::generate_levies(&state.pool, id, auth.building_id, input.due_date, &levy_inserts).await?.len();

    Ok(Json(serde_json::json!({
        "generated": nb,
        "total_amount": total_amount,
        "due_date": input.due_date,
    })))
}

pub async fn list_levies(auth: AuthUser, State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> AppResult<Json<serde_json::Value>> {
    let levies = db::budget::list_levies(&state.pool, id, auth.building_id).await?;
    Ok(Json(serde_json::to_value(levies).map_err(|e| AppError::Internal(e.into()))?))
}

pub async fn mark_paid(auth: AuthUser, State(state): State<Arc<AppState>>, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    db::budget::mark_paid(&state.pool, id, auth.building_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_works_fund(auth: AuthUser, State(state): State<Arc<AppState>>) -> AppResult<Json<serde_json::Value>> {
    let fiscal_year = chrono::Utc::now().format("%Y").to_string().parse::<i32>().unwrap_or(2025);
    let ft = db::budget::get_works_fund(&state.pool, auth.building_id, fiscal_year).await?;
    Ok(Json(ft.unwrap_or(serde_json::json!(null))))
}

pub async fn get_works_fund(auth: AuthUser, State(state): State<Arc<AppState>>, Path(fiscal_year): Path<i32>) -> AppResult<Json<serde_json::Value>> {
    let ft = db::budget::get_works_fund(&state.pool, auth.building_id, fiscal_year).await?;
    Ok(Json(ft.ok_or(AppError::NotFound)?))
}
