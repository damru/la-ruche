use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::NaiveDate;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    db::{self, AppState},
    domain::{AuthUser, Role, etat_date::EtatDate},
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

/// GET /etat-date-requests/{id}/pdf — renders the frozen snapshot as a PDF.
/// Access matches `get` (any authenticated building member): the snapshot is
/// the probative record, the PDF only renders it.
pub async fn export_pdf(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Response> {
    let request = db::etat_date::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let snapshot = request
        .snapshot
        .ok_or_else(|| AppError::conflict_code("errors.etat_date.not_generated"))?;
    let etat_date: EtatDate =
        serde_json::from_value(snapshot).map_err(|e| AppError::Internal(e.into()))?;

    let header = build_header(&state.pool, auth.building_id, request.unit_id).await?;
    let mut header = header;
    header.requester_name = Some(request.requester_name);
    header.requested_at = Some(request.requested_at);
    header.due_date = Some(request.due_date);
    header.generated_at = request.generated_at.map(|dt| dt.date_naive());

    let bytes = crate::domain::legal_documents::render_etat_date(&header, &etat_date)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    Ok(pdf_response(bytes, "etat-date"))
}

/// GET /units/{id}/pre-etat-date/pdf — renders the pré-état daté as a PDF.
/// Manager only, mirroring the JSON `pre_etat_date` endpoint.
pub async fn pre_etat_date_pdf(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(unit_id): Path<Uuid>,
) -> AppResult<Response> {
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

    let mut header = build_header(&state.pool, auth.building_id, unit_id).await?;
    header.edited_at = Some(chrono::Utc::now().date_naive());

    let bytes = crate::domain::legal_documents::render_pre_etat_date(&header, &pre_etat_date)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    Ok(pdf_response(bytes, "pre-etat-date"))
}

/// Assembles the residence + syndic header block shared by both documents.
/// Data shaping only — no business logic.
async fn build_header(
    pool: &sqlx::PgPool,
    building_id: Uuid,
    unit_id: Uuid,
) -> AppResult<crate::domain::legal_documents::LegalDocumentHeader> {
    let building = db::building::get(pool, building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let unit = db::unit::get(pool, unit_id, building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    // Syndic name comes from the active governance mandate; SIRET comes from
    // `building.manager_siret` (governance_mandate has no SIRET column). This is
    // a known data-model coupling: a syndic change must also update manager_siret
    // (documented in ADR-011 amendment).
    let syndic_name = db::governance_mandate::current_syndic_name(pool, building_id).await?;
    // Extra context for the questionnaire annex (all pre-existing queries):
    // current owner of the lot, syndic mandate start, last/next general
    // meeting, last works-fund levy of the building.
    let owner_name = db::unit::list_owners(pool, unit_id, building_id)
        .await?
        .into_iter()
        .find(|o| o.end_date.is_none())
        .map(|o| o.display_name);
    let syndic_mandate_start = db::governance_mandate::list(pool, building_id)
        .await?
        .into_iter()
        .find(|m| m.role == db::governance_mandate::ROLE_SYNDIC && m.end_date.is_none())
        .map(|m| m.start_date);
    let now = chrono::Utc::now().date_naive();
    let meetings = db::assembly::list(pool, building_id).await?;
    let ag_dates: Vec<chrono::NaiveDate> = meetings
        .iter()
        .map(|m| m.meeting_date.date_naive())
        .collect();
    let last_ag_date = ag_dates.iter().copied().filter(|d| *d <= now).max();
    let next_ag_date = ag_dates.iter().copied().filter(|d| *d > now).min();
    let fonds_travaux_derniere_cotisation =
        db::etat_date::load_pre_etat_date_extras(pool, building_id)
            .await?
            .fonds_travaux_derniere_cotisation;

    let mut address = building.address_line1.clone();
    if let Some(line2) = building
        .address_line2
        .as_deref()
        .filter(|s| !s.trim().is_empty())
    {
        address.push_str(", ");
        address.push_str(line2);
    }
    address.push_str(", ");
    address.push_str(&building.postal_code);
    address.push(' ');
    address.push_str(&building.city);

    Ok(crate::domain::legal_documents::LegalDocumentHeader {
        building_name: building.name,
        building_address: address,
        syndic_name,
        syndic_siret: building.manager_siret,
        unit_number: unit.unit_number,
        requester_name: None,
        requested_at: None,
        due_date: None,
        generated_at: None,
        edited_at: None,
        owner_name,
        bylaws_date: building.bylaws_date,
        syndic_mandate_start,
        last_ag_date,
        next_ag_date,
        fonds_travaux_derniere_cotisation: Some(fonds_travaux_derniere_cotisation),
    })
}

/// Wraps PDF bytes as an `application/pdf` attachment download.
fn pdf_response(bytes: Vec<u8>, base_name: &str) -> Response {
    let mut response = bytes.into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{base_name}.pdf\""))
            .unwrap_or(HeaderValue::from_static("attachment")),
    );
    response
}
