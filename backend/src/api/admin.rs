use std::sync::Arc;

use axum::{extract::State, http::StatusCode};

use crate::{
    db::AppState,
    domain::{AuthUser, Role},
    errors::{AppError, AppResult},
};

// Embed the demo SQL at compile time — no runtime file dependency.
const DEMO_SQL: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/seeds/demo.sql"));

/// Truncates all application tables in dependency order, leaving the schema intact.
/// User accounts and tokens are preserved.
const TRUNCATE_SQL: &str = "TRUNCATE TABLE \
    maintenance_log_document, maintenance_log, \
    report_document, report, \
    vendor_submission_document, vendor_submission, \
    task_document, \
    task, \
    contract_document, contract_reminder, vendor_contract, \
    claim_document, claim, \
    vote, resolution, general_meeting, \
    levy_call_unit, levy_call_line, levy_call, budget_line, budget, works_fund, \
    consultation_response_choice, consultation_response, consultation_choice, consultation, \
    contractor, \
    unit_allocation_key, \
    allocation_key, \
    governance_mandate, \
    unit_owner, \
    unit, \
    document \
CASCADE";

/// POST /admin/reset-demo
///
/// Truncates all tables and re-seeds the database with demo data.
/// Restricted to users with the `manager` role.
pub async fn reset_demo(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    if !state.cfg.app.demo_mode {
        return Err(AppError::Forbidden);
    }

    sqlx::raw_sql(DEMO_SQL).execute(&state.pool).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /admin/reset-data
///
/// Truncates all tables without re-seeding — preserves user accounts.
/// Restricted to users with the `manager` role.
pub async fn reset_data(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    if !state.cfg.app.demo_mode {
        return Err(AppError::Forbidden);
    }

    sqlx::raw_sql(TRUNCATE_SQL).execute(&state.pool).await?;

    Ok(StatusCode::NO_CONTENT)
}
