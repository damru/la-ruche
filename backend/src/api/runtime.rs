use std::sync::Arc;

use axum::{Json, extract::State};
use serde::Serialize;
use utoipa::ToSchema;

use crate::db::AppState;

#[derive(Serialize, ToSchema)]
pub struct RuntimeStatusResponse {
    pub demo_mode: bool,
}

pub async fn status(State(state): State<Arc<AppState>>) -> Json<RuntimeStatusResponse> {
    Json(RuntimeStatusResponse {
        demo_mode: state.cfg.app.demo_mode,
    })
}
