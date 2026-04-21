use std::sync::Arc;

use axum::{Json, extract::State};
use serde_json::Value;

use crate::{
    db::{self, AppState},
    domain::AuthUser,
    errors::AppResult,
};

/// GET /users — list all active users of the current building
pub async fn list(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> AppResult<Json<Vec<Value>>> {
    let users = db::user::list(&state.pool, auth.building_id).await?;
    Ok(Json(users))
}
