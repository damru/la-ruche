// ── api/notification_preferences.rs — GET/PUT/DELETE /me/notification-preferences ──

use std::collections::HashMap;
use std::sync::Arc;

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::{
    db::{self, AppState},
    domain::{AuthUser, notification_catalog},
    errors::{AppError, AppResult},
};

#[derive(Serialize)]
pub struct NotifEventState {
    pub key: &'static str,
    pub mandatory: bool,
    pub enabled: bool,
}

#[derive(Serialize)]
pub struct NotificationPreferencesResponse {
    pub events: Vec<NotifEventState>,
}

#[derive(Deserialize)]
pub struct UpdateNotificationPreferencesInput {
    pub prefs: HashMap<String, bool>,
}

/// Merges the canonical catalog with the user's stored overrides:
/// mandatory events are always enabled, optional ones use the override
/// when present, otherwise the catalog default. Catalog order is preserved.
fn merged_events(overrides: &HashMap<String, bool>) -> Vec<NotifEventState> {
    notification_catalog::NOTIF_EVENTS
        .iter()
        .map(|event| NotifEventState {
            key: event.key,
            mandatory: event.mandatory,
            enabled: if event.mandatory {
                true
            } else {
                overrides
                    .get(event.key)
                    .copied()
                    .unwrap_or(event.default_enabled)
            },
        })
        .collect()
}

/// GET /me/notification-preferences — catalog fused with the user's values.
pub async fn get(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> AppResult<Json<NotificationPreferencesResponse>> {
    let rows = db::notification_preference::list(&state.pool, auth.user_id).await?;
    let overrides: HashMap<String, bool> = rows.into_iter().collect();
    Ok(Json(NotificationPreferencesResponse {
        events: merged_events(&overrides),
    }))
}

/// DELETE /me/notification-preferences — drops all stored overrides, back to
/// catalog defaults. Idempotent; response has the same shape as GET.
pub async fn reset(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> AppResult<Json<NotificationPreferencesResponse>> {
    db::notification_preference::delete_all(&state.pool, auth.user_id).await?;
    Ok(Json(NotificationPreferencesResponse {
        events: merged_events(&HashMap::new()),
    }))
}

/// PUT /me/notification-preferences — replaces the values of optional events.
pub async fn update(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(input): Json<UpdateNotificationPreferencesInput>,
) -> AppResult<Json<NotificationPreferencesResponse>> {
    // Validate before writing anything: unknown keys and mandatory events are rejected.
    for key in input.prefs.keys() {
        let event = notification_catalog::find(key).ok_or_else(|| {
            AppError::bad_request_code("errors.notification_preferences.unknown_event")
        })?;
        if event.mandatory {
            return Err(AppError::bad_request_code(
                "errors.notification_preferences.mandatory_event",
            ));
        }
    }

    let prefs: Vec<(String, bool)> = input.prefs.into_iter().collect();
    db::notification_preference::upsert(&state.pool, auth.user_id, &prefs).await?;

    // Re-read so the response reflects the persisted state (same shape as GET).
    let rows = db::notification_preference::list(&state.pool, auth.user_id).await?;
    let overrides: HashMap<String, bool> = rows.into_iter().collect();
    Ok(Json(NotificationPreferencesResponse {
        events: merged_events(&overrides),
    }))
}
