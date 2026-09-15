// ── api/calendar.rs — Calendrier copropriété (EP-014-S01) ────────────────
//
// Handler with no business logic: validates the query range, resolves the
// visible sources for the caller's roles, and delegates to `db::calendar`.

use axum::{Json, extract::State};
use chrono::NaiveDate;
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    db::{self, AppState},
    domain::{AuthUser, calendar::CalendarEvent},
    errors::{AppError, AppResult},
};

const MAX_RANGE_DAYS: i64 = 366;

#[derive(Deserialize)]
pub struct EventsQuery {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

/// GET /calendar/events?from=YYYY-MM-DD&to=YYYY-MM-DD
///
/// All authenticated roles can call this endpoint — filtering happens by
/// visible sources (AC-CAL-001-C), not by a 403.
pub async fn events(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<EventsQuery>,
) -> AppResult<Json<Vec<CalendarEvent>>> {
    if params.from > params.to || (params.to - params.from).num_days() > MAX_RANGE_DAYS {
        return Err(AppError::validation_code("errors.calendar.invalid_range"));
    }

    let sources = crate::domain::calendar::visible_sources(&auth.roles);
    let events = db::calendar::events(
        &state.pool,
        auth.building_id,
        params.from,
        params.to,
        &sources,
        &auth.roles,
    )
    .await?;

    Ok(Json(events))
}
