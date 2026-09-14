use axum::{
    Json,
    extract::{Query, State},
};
use chrono::NaiveDate;
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    db::{self, AppState},
    domain::{AuthUser, calendar::visible_sources},
    errors::{AppError, AppResult, json_value},
};

#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

const MAX_RANGE_DAYS: i64 = 366;

pub async fn events(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(params): Query<EventsQuery>,
) -> AppResult<Json<serde_json::Value>> {
    if params.from > params.to || (params.to - params.from).num_days() > MAX_RANGE_DAYS {
        return Err(AppError::validation_code("errors.calendar.invalid_range"));
    }

    let sources = visible_sources(&auth.roles);
    let events = db::calendar::events(
        &state.pool,
        auth.building_id,
        params.from,
        params.to,
        &sources,
        &auth.roles,
    )
    .await?;
    json_value(events)
}
