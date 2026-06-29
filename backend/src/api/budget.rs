use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{Datelike, NaiveDate};
use serde::Deserialize;
use sqlx::types::Decimal;
use std::collections::HashMap;
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

fn legal_minimum_works_fund_rate_pct() -> Decimal {
    Decimal::new(5, 0)
}

fn percentage_divisor() -> Decimal {
    Decimal::new(100, 0)
}

fn works_fund_target_amount(total_charges: Decimal, contribution_rate_pct: Decimal) -> Decimal {
    (total_charges * contribution_rate_pct / percentage_divisor()).round_dp(2)
}

fn serialize_works_fund(
    works_fund: db::budget::WorksFundRow,
    minimum_legal_rate_pct: Decimal,
) -> serde_json::Value {
    serde_json::json!({
        "id": works_fund.id,
        "fiscal_year": works_fund.fiscal_year,
        "levied_amount": works_fund.levied_amount,
        "available_amount": works_fund.available_amount,
        "contribution_rate_pct": works_fund.contribution_rate_pct,
        "minimum_legal_rate_pct": minimum_legal_rate_pct,
        "is_rate_compliant": works_fund.contribution_rate_pct >= minimum_legal_rate_pct,
    })
}

fn can_see_draft(auth: &AuthUser) -> bool {
    auth.roles
        .iter()
        .any(|r| matches!(r, Role::Manager | Role::BoardMember))
}

pub async fn list(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let budgets = db::budget::list(&state.pool, auth.building_id).await?;
    let visible: Vec<_> = if can_see_draft(&auth) {
        budgets
    } else {
        budgets
            .into_iter()
            .filter(|b| b.status != "draft")
            .collect()
    };
    Ok(Json(
        serde_json::to_value(visible).map_err(|e| AppError::Internal(e.into()))?,
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
    if b.status == "draft" && !can_see_draft(&auth) {
        return Err(AppError::NotFound);
    }
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
    let budget = db::budget::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let works_fund =
        db::budget::get_or_create_works_fund(&state.pool, auth.building_id, budget.fiscal_year)
            .await?;
    let legal_minimum_rate_pct = legal_minimum_works_fund_rate_pct();

    if works_fund.contribution_rate_pct < legal_minimum_rate_pct {
        return Err(AppError::conflict_code(
            "errors.budget.works_fund_rate_below_legal_minimum",
        ));
    }

    let works_fund_target =
        works_fund_target_amount(budget.total_forecast, works_fund.contribution_rate_pct);
    if works_fund.levied_amount < works_fund_target {
        return Err(AppError::conflict_code(
            "errors.budget.works_fund_target_unmet",
        ));
    }

    let closed = db::budget::close(&state.pool, id, auth.building_id).await?;
    if closed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::conflict_code(
            "errors.budget.exercise_already_closed",
        ))
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

#[derive(Deserialize)]
pub struct UpdateWorksFundInput {
    pub contribution_rate_pct: Decimal,
}

fn quarterly_default_call_label(now: chrono::DateTime<chrono::Utc>) -> String {
    let quarter = (now.month0() / 3) + 1;
    format!("Quarterly Q{quarter} {}", now.year())
}

pub async fn generate_levies(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<GenerateLeviesInput>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_role(&Role::Manager)?;
    db::budget::ensure_budget_is_mutable(&state.pool, id, auth.building_id).await?;

    let budget = db::budget::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let unbalanced_keys =
        db::allocation_key::list_unbalanced_active_keys(&state.pool, auth.building_id).await?;
    if !unbalanced_keys.is_empty() {
        return Err(AppError::conflict_code(
            "errors.budget.allocation_keys_unbalanced",
        ));
    }

    let lines = db::budget::list_lines(&state.pool, id, auth.building_id).await?;
    let works_fund =
        db::budget::get_or_create_works_fund(&state.pool, auth.building_id, budget.fiscal_year)
            .await?;

    let total_charges: Decimal = lines.iter().map(|l| l.budgeted_amount).sum();

    let legal_minimum_rate_pct = legal_minimum_works_fund_rate_pct();
    if input.include_works_fund && works_fund.contribution_rate_pct < legal_minimum_rate_pct {
        return Err(AppError::conflict_code(
            "errors.budget.works_fund_rate_below_legal_minimum",
        ));
    }

    let works_fund_amount = if input.include_works_fund {
        works_fund_target_amount(total_charges, works_fund.contribution_rate_pct)
    } else {
        Decimal::ZERO
    };

    let levy_totals = db::budget::levy_amount_totals(&state.pool, id, auth.building_id).await?;
    let charge_amount_totals_by_line =
        db::budget::charge_amount_totals_by_line(&state.pool, id, auth.building_id).await?;
    let target_charge_amount = total_charges;
    let remaining_charge_amount =
        (target_charge_amount - levy_totals.charge_amount).max(Decimal::ZERO);
    let remaining_works_fund_amount = if input.include_works_fund {
        (works_fund_amount - levy_totals.works_fund_amount).max(Decimal::ZERO)
    } else {
        Decimal::ZERO
    };
    let already_called_total = if input.include_works_fund {
        levy_totals.total_amount
    } else {
        levy_totals.charge_amount
    };
    let total_amount = if input.include_works_fund {
        total_charges + works_fund_amount
    } else {
        total_charges
    };
    let remaining_amount = remaining_charge_amount + remaining_works_fund_amount;

    if remaining_amount <= Decimal::ZERO {
        return Ok(Json(serde_json::json!({
            "generated": 0,
            "total_amount": total_amount,
            "already_called": already_called_total,
            "remaining_amount": Decimal::ZERO,
            "due_date": input.due_date,
        })));
    }

    let mut levy_inserts_by_unit_key: HashMap<(Uuid, Uuid), db::budget::LevyInsert> =
        HashMap::new();
    let mut units_with_shares_by_key: HashMap<Uuid, (Vec<UnitWithShares>, i64)> = HashMap::new();

    for line in &lines {
        let already_called_for_line = charge_amount_totals_by_line
            .get(&line.id)
            .copied()
            .unwrap_or(Decimal::ZERO);
        let remaining_for_line =
            (line.budgeted_amount - already_called_for_line).max(Decimal::ZERO);

        if remaining_for_line <= Decimal::ZERO {
            continue;
        }

        let (units_with_shares, key_total_shares) = if let Some(cached) =
            units_with_shares_by_key.get(&line.key_id)
        {
            cached.clone()
        } else {
            let key_shares =
                db::allocation_key::list_shares(&state.pool, line.key_id, auth.building_id).await?;
            let units_with_shares: Vec<UnitWithShares> = key_shares
                .into_iter()
                .filter(|row| row.share_value > 0)
                .map(|row| UnitWithShares {
                    unit_id: row.unit_id,
                    key_id: row.key_id,
                    shares: row.share_value as i64,
                })
                .collect();
            let key_total_shares: i64 = units_with_shares.iter().map(|unit| unit.shares).sum();
            units_with_shares_by_key
                .insert(line.key_id, (units_with_shares.clone(), key_total_shares));
            (units_with_shares, key_total_shares)
        };

        for levy in compute_levies(remaining_for_line, &units_with_shares, key_total_shares) {
            if levy.amount <= Decimal::ZERO {
                continue;
            }

            levy_inserts_by_unit_key
                .entry((levy.unit_id, levy.key_id))
                .and_modify(|insert| {
                    insert.amount += levy.amount;
                    insert.charge_amount += levy.amount;
                    insert.line_allocations.push(db::budget::LevyInsertLine {
                        budget_line_id: line.id,
                        amount: levy.amount,
                    });
                })
                .or_insert(db::budget::LevyInsert {
                    unit_id: levy.unit_id,
                    key_id: levy.key_id,
                    amount: levy.amount,
                    charge_amount: levy.amount,
                    works_fund_amount: Decimal::ZERO,
                    shares_snapshot: levy.shares_snapshot,
                    shares_total_snapshot: levy.shares_total_snapshot,
                    line_allocations: vec![db::budget::LevyInsertLine {
                        budget_line_id: line.id,
                        amount: levy.amount,
                    }],
                });
        }
    }

    let general_units_data =
        db::allocation_key::list_general_shares(&state.pool, auth.building_id).await?;
    let general_units_with_shares: Vec<UnitWithShares> = general_units_data
        .iter()
        .map(|row| UnitWithShares {
            unit_id: row.unit_id,
            key_id: row.key_id,
            shares: row.shares as i64,
        })
        .collect();
    let general_total_shares: i64 = general_units_with_shares
        .iter()
        .map(|unit| unit.shares)
        .sum();

    if input.include_works_fund && remaining_works_fund_amount > Decimal::ZERO {
        for levy in compute_levies(
            remaining_works_fund_amount,
            &general_units_with_shares,
            general_total_shares,
        ) {
            if levy.amount <= Decimal::ZERO {
                continue;
            }

            levy_inserts_by_unit_key
                .entry((levy.unit_id, levy.key_id))
                .and_modify(|insert| {
                    insert.amount += levy.amount;
                    insert.works_fund_amount += levy.amount;
                })
                .or_insert(db::budget::LevyInsert {
                    unit_id: levy.unit_id,
                    key_id: levy.key_id,
                    amount: levy.amount,
                    charge_amount: Decimal::ZERO,
                    works_fund_amount: levy.amount,
                    shares_snapshot: levy.shares_snapshot,
                    shares_total_snapshot: levy.shares_total_snapshot,
                    line_allocations: Vec::new(),
                });
        }
    }

    let default_label = quarterly_default_call_label(chrono::Utc::now());
    let call_label = input
        .call_label
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&default_label);
    let levy_inserts: Vec<db::budget::LevyInsert> = levy_inserts_by_unit_key
        .into_values()
        .filter(|insert| insert.amount > Decimal::ZERO)
        .collect();

    let nb = db::budget::generate_levies(
        &state.pool,
        id,
        auth.building_id,
        input.due_date,
        call_label,
        &levy_inserts,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "generated": nb,
        "total_amount": total_amount,
        "already_called": already_called_total,
        "remaining_amount": remaining_amount,
        "call_label": call_label,
        "due_date": input.due_date,
    })))
}

#[derive(Deserialize)]
pub struct ListLeviesQuery {
    owner_id: Option<Uuid>,
}

pub async fn list_levies(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(params): Query<ListLeviesQuery>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    let levies =
        db::budget::list_levies(&state.pool, id, auth.building_id, params.owner_id).await?;
    Ok(Json(
        serde_json::to_value(levies).map_err(|e| AppError::Internal(e.into()))?,
    ))
}

pub async fn issue_levy_call(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(levy_call_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    db::budget::issue_levy_call(&state.pool, levy_call_id, auth.building_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn credit_note(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(levy_call_id): Path<Uuid>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    auth.require_role(&Role::Manager)?;
    let new_id =
        db::budget::create_credit_note(&state.pool, levy_call_id, auth.building_id).await?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": new_id })),
    ))
}

pub async fn mark_paid(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((levy_call_id, unit_id)): Path<(Uuid, Uuid)>,
) -> AppResult<StatusCode> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    db::budget::mark_paid(&state.pool, levy_call_id, unit_id, auth.building_id).await?;
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
    let ft = match db::budget::get_works_fund(&state.pool, auth.building_id, fiscal_year).await? {
        Some(works_fund) => Some(works_fund),
        None => {
            if db::budget::get_by_fiscal_year(&state.pool, auth.building_id, fiscal_year)
                .await?
                .is_some()
            {
                Some(
                    db::budget::get_or_create_works_fund(
                        &state.pool,
                        auth.building_id,
                        fiscal_year,
                    )
                    .await?,
                )
            } else {
                None
            }
        }
    };

    Ok(Json(
        ft.map(|works_fund| serialize_works_fund(works_fund, legal_minimum_works_fund_rate_pct()))
            .unwrap_or(serde_json::json!(null)),
    ))
}

pub async fn get_works_fund(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(fiscal_year): Path<i32>,
) -> AppResult<Json<serde_json::Value>> {
    let works_fund = match db::budget::get_works_fund(&state.pool, auth.building_id, fiscal_year)
        .await?
    {
        Some(works_fund) => works_fund,
        None => {
            db::budget::get_by_fiscal_year(&state.pool, auth.building_id, fiscal_year)
                .await?
                .ok_or(AppError::NotFound)?;
            db::budget::get_or_create_works_fund(&state.pool, auth.building_id, fiscal_year).await?
        }
    };

    Ok(Json(serialize_works_fund(
        works_fund,
        legal_minimum_works_fund_rate_pct(),
    )))
}

pub async fn update_works_fund(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(fiscal_year): Path<i32>,
    Json(input): Json<UpdateWorksFundInput>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_role(&Role::Manager)?;

    if input.contribution_rate_pct < Decimal::ZERO
        || input.contribution_rate_pct > percentage_divisor()
    {
        return Err(AppError::validation_code(
            "errors.budget.works_fund_rate_invalid",
        ));
    }

    db::budget::get_by_fiscal_year(&state.pool, auth.building_id, fiscal_year)
        .await?
        .ok_or(AppError::NotFound)?;
    db::budget::get_or_create_works_fund(&state.pool, auth.building_id, fiscal_year).await?;

    let works_fund = db::budget::update_works_fund_rate(
        &state.pool,
        auth.building_id,
        fiscal_year,
        input.contribution_rate_pct.round_dp(2),
    )
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(serialize_works_fund(
        works_fund,
        legal_minimum_works_fund_rate_pct(),
    )))
}
