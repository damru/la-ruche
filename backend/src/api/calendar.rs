// ── api/calendar.rs — Calendrier copropriété ────────────────
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
use std::{str::FromStr, sync::Arc};

use crate::{
    db::{self, AppState},
    domain::{
        AuthUser,
        calendar::{CalendarEvent, CalendarSource, effective_sources},
    },
    errors::{AppError, AppResult},
};

const MAX_RANGE_DAYS: i64 = 366;
const EXPORT_DEFAULT_RANGE_DAYS: u64 = 365;
const ICS_PRODID: &str = "-//La Ruche//Calendrier//FR";

#[derive(Deserialize)]
pub struct EventsQuery {
    pub from: NaiveDate,
    pub to: NaiveDate,
    /// Optional CSV filter: `types=meeting,contract,...`.
    /// Absent or empty → every source visible for the caller's roles.
    pub types: Option<String>,
}

/// Parses the CSV `types` query parameter into sources.
/// An empty value is treated as absent; an unknown value is a 400.
fn parse_types(types: Option<&str>) -> AppResult<Option<Vec<CalendarSource>>> {
    let Some(types) = types else {
        return Ok(None);
    };
    let requested = types
        .split(',')
        .filter(|part| !part.is_empty())
        .map(CalendarSource::from_str)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|()| AppError::validation_code("errors.calendar.invalid_type"))?;
    // `types=` (empty value) is treated as absent.
    Ok((!requested.is_empty()).then_some(requested))
}

/// GET /calendar/events?from=YYYY-MM-DD&to=YYYY-MM-DD&types=meeting,contract
///
/// All authenticated roles can call this endpoint — filtering happens by
/// visible sources, not by a 403. The `types` filter is intersected with
/// role visibility: it can only narrow.
pub async fn events(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<EventsQuery>,
) -> AppResult<Json<Vec<CalendarEvent>>> {
    if params.from > params.to || (params.to - params.from).num_days() > MAX_RANGE_DAYS {
        return Err(AppError::validation_code("errors.calendar.invalid_range"));
    }

    let requested = parse_types(params.types.as_deref())?;
    let sources = effective_sources(&auth.roles, requested.as_deref());
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
/// Same visibility pipeline as `events` serialized as ICS.
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
