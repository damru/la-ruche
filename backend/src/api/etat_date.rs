use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::NaiveDate;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    db::{self, AppState},
    domain::{AuthUser, Role},
    errors::{AppError, AppResult, json_value},
};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateEtatDateRequestInput {
    pub requested_at: NaiveDate,
    #[validate(length(min = 1, max = 200))]
    pub requester_name: String,
}

/// POST /units/{id}/etat-date-requests — manager only.
pub async fn create(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(unit_id): Path<Uuid>,
    Json(input): Json<CreateEtatDateRequestInput>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    auth.require_role(&Role::Manager)?;
    input
        .validate()
        .map_err(|_| AppError::validation_code("errors.etat_date.invalid_request_date"))?;

    let today = chrono::Utc::now().naive_utc().date();
    if input.requested_at > today {
        return Err(AppError::validation_code(
            "errors.etat_date.invalid_request_date",
        ));
    }

    let request = db::etat_date::create_request(
        &state.pool,
        unit_id,
        auth.building_id,
        input.requested_at,
        &input.requester_name,
        auth.user_id,
    )
    .await?;

    Ok((StatusCode::CREATED, json_value(request)?))
}

/// GET /etat-date-requests — list with J+15 deadlines, sorted by due_date.
pub async fn list(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let requests = db::etat_date::list_for_building(&state.pool, auth.building_id).await?;
    json_value(requests)
}

/// GET /etat-date-requests/{id} — returns the frozen snapshot, or 409 if not yet generated.
pub async fn get(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let request = db::etat_date::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if request.snapshot.is_none() {
        return Err(AppError::conflict_code("errors.etat_date.not_generated"));
    }

    json_value(request)
}

/// POST /etat-date-requests/{id}/generate — assembles and freezes the document.
/// Regeneration allowed while `status != 'sent'`.
pub async fn generate(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_role(&Role::Manager)?;

    let request = db::etat_date::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if request.status == db::etat_date::STATUS_SENT {
        return Err(AppError::conflict_code("errors.etat_date.already_sent"));
    }

    let inputs = db::etat_date::load_inputs(&state.pool, request.unit_id, auth.building_id).await?;
    let etat_date = crate::domain::etat_date::assemble(inputs);
    let snapshot = serde_json::to_value(&etat_date).map_err(|e| AppError::Internal(e.into()))?;

    let updated = db::etat_date::store_snapshot(&state.pool, id, auth.building_id, &snapshot)
        .await?
        .ok_or_else(|| AppError::conflict_code("errors.etat_date.already_sent"))?;

    json_value(updated)
}

/// GET /units/{id}/pre-etat-date — manager only. Generated on demand, no
/// workflow/snapshot (EP-002-S07, LEGAL.md §7.2).
pub async fn pre_etat_date(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(unit_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_role(&Role::Manager)?;

    db::unit::get(&state.pool, unit_id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let inputs = db::etat_date::load_inputs(&state.pool, unit_id, auth.building_id).await?;
    let extras = db::etat_date::load_pre_etat_date_extras(&state.pool, auth.building_id).await?;
    let pre_etat_date = crate::domain::etat_date::assemble_pre_etat_date(
        inputs,
        extras.impayes_globaux_syndicat,
        extras.fonds_travaux_derniere_cotisation,
    );

    json_value(pre_etat_date)
}
