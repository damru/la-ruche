// ── api/calendar.rs — Calendrier copropriété (EP-014-S01) ────────────────
//
// Handler with no business logic: validates the query range, resolves the
// visible sources for the caller's roles, and delegates to `db::calendar`.

use axum::{
    Json,
    extract::State,
    http::{HeaderValue, header},
    response::{IntoResponse, Response},
};
use chrono::{Days, NaiveDate, Utc};
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    db::{self, AppState},
    domain::{AuthUser, calendar::CalendarEvent},
    errors::{AppError, AppResult},
};

const MAX_RANGE_DAYS: i64 = 366;
const EXPORT_DEFAULT_RANGE_DAYS: u64 = 365;
const ICS_PRODID: &str = "-//La Ruche//Calendrier//FR";

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

#[derive(Deserialize)]
pub struct ExportIcsQuery {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}

/// GET /calendar/export.ics?from=YYYY-MM-DD&to=YYYY-MM-DD
///
/// Same visibility pipeline as `events` (AC-CAL-003-A) serialized as ICS.
/// Defaults to today → +365 days when `from`/`to` are omitted (subscription
/// export has no meaningful default scoped to the current screen).
pub async fn export_ics(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<ExportIcsQuery>,
) -> AppResult<Response> {
    let today = Utc::now().date_naive();
    let from = params.from.unwrap_or(today);
    let to = params
        .to
        .unwrap_or_else(|| from + Days::new(EXPORT_DEFAULT_RANGE_DAYS));

    if from > to || (to - from).num_days() > MAX_RANGE_DAYS {
        return Err(AppError::validation_code("errors.calendar.invalid_range"));
    }

    let sources = crate::domain::calendar::visible_sources(&auth.roles);
    let events = db::calendar::events(
        &state.pool,
        auth.building_id,
        from,
        to,
        &sources,
        &auth.roles,
    )
    .await?;

    let base_url = state.cfg.app.public_url.trim_end_matches('/');
    let body = crate::domain::ics::to_ics(&events, ICS_PRODID, base_url);

    let mut response = body.into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/calendar; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"laruche-calendar.ics\""),
    );
    Ok(response)
}
