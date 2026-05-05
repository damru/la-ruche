use std::sync::Arc;

use axum::{extract::State, http::StatusCode};

use crate::{
    db::AppState,
    domain::{AuthUser, Role},
    errors::AppResult,
};

// Embed the demo SQL at compile time — no runtime file dependency.
const DEMO_SQL: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/seeds/demo.sql"));

/// POST /admin/reset-demo
///
/// Truncates all tables and re-seeds the database with demo data.
/// Restricted to users with the `manager` role.
pub async fn reset_demo(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;

    sqlx::raw_sql(DEMO_SQL).execute(&state.pool).await?;

    Ok(StatusCode::NO_CONTENT)
}
