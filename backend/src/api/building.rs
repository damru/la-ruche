use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    db::{self, AppState},
    domain::AuthUser,
    errors::{AppError, AppResult},
};

// ── DTOs ──────────────────────────────────────────────────────────

#[derive(Serialize, ToSchema)]
pub struct BuildingResponse {
    pub id: Uuid,
    pub name: String,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub postal_code: String,
    pub city: String,
    pub manager_siret: Option<String>,
    pub shares_total: i32,
    pub bylaws_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct BuildingStatsResponse {
    pub name: String,
    pub nb_lots: i64,
    pub nb_users: i64,
    pub nb_tenants: i64,
    pub shares_total: i32,
    #[schema(value_type = f64)]
    pub works_fund_available: Decimal,
    pub works_fund_pct: Option<i32>,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct UpdateBuildingInput {
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    #[validate(length(min = 1, max = 200))]
    pub address_line1: String,
    pub address_line2: Option<String>,
    #[validate(length(min = 1, max = 10))]
    pub postal_code: String,
    #[validate(length(min = 1, max = 100))]
    pub city: String,
    pub manager_siret: Option<String>,
    pub shares_total: i32,
    pub bylaws_date: Option<chrono::NaiveDate>,
}

// ── Handlers ──────────────────────────────────────────────────────

/// GET /building
pub async fn get(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> AppResult<Json<BuildingResponse>> {
    let row = db::building::get(&state.pool, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(BuildingResponse {
        id: row.id,
        name: row.name,
        address_line1: row.address_line1,
        address_line2: row.address_line2,
        postal_code: row.postal_code,
        city: row.city,
        manager_siret: row.manager_siret,
        shares_total: row.shares_total,
        bylaws_date: row.bylaws_date,
        created_at: row.created_at,
    }))
}

/// GET /building/stats
pub async fn stats(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> AppResult<Json<BuildingStatsResponse>> {
    let building = db::building::get(&state.pool, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let s = db::building::get_stats(&state.pool, auth.building_id).await?;

    Ok(Json(BuildingStatsResponse {
        name: building.name,
        nb_lots: s.nb_units,
        nb_users: s.nb_users,
        nb_tenants: s.nb_tenants,
        shares_total: s.shares_total,
        works_fund_available: s.works_fund_available,
        works_fund_pct: s.works_fund_pct,
    }))
}

/// PUT /building
pub async fn update(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(input): Json<UpdateBuildingInput>,
) -> AppResult<Json<BuildingResponse>> {
    if !auth.roles.contains(&crate::domain::Role::Manager) {
        return Err(AppError::Forbidden);
    }

    input
        .validate()
        .map_err(|_| AppError::validation_code("errors.building.invalid_payload"))?;

    let row = db::building::update(
        &state.pool,
        auth.building_id,
        db::building::UpdateBuildingInput {
            name: input.name,
            address_line1: input.address_line1,
            address_line2: input.address_line2,
            postal_code: input.postal_code,
            city: input.city,
            manager_siret: input.manager_siret,
            shares_total: input.shares_total,
            bylaws_date: input.bylaws_date,
        },
    )
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(BuildingResponse {
        id: row.id,
        name: row.name,
        address_line1: row.address_line1,
        address_line2: row.address_line2,
        postal_code: row.postal_code,
        city: row.city,
        manager_siret: row.manager_siret,
        shares_total: row.shares_total,
        bylaws_date: row.bylaws_date,
        created_at: row.created_at,
    }))
}

// ── Allocation keys ───────────────────────────────────────────────

#[derive(Serialize, ToSchema)]
pub struct AllocationKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub key_type: String,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct AllocationKeyInput {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub description: Option<String>,
    #[validate(length(max = 500))]
    pub justification: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct UnitShareResponse {
    pub unit_id: Uuid,
    pub unit_number: String,
    pub share_value: i32,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct SetSharesInput {
    pub shares: Vec<UnitShareItem>,
    #[validate(length(max = 500))]
    pub justification: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct UnitShareItem {
    pub unit_id: Uuid,
    pub share_value: i32,
}

#[derive(Deserialize, ToSchema)]
pub struct DeleteKeyInput {
    #[serde(default)]
    pub justification: Option<String>,
}

fn require_manager(auth: &crate::domain::AuthUser) -> crate::errors::AppResult<()> {
    if auth.roles.contains(&crate::domain::Role::Manager) {
        Ok(())
    } else {
        Err(crate::errors::AppError::Forbidden)
    }
}

/// GET /building/keys
pub async fn list_keys(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> AppResult<Json<Vec<AllocationKeyResponse>>> {
    let rows = db::allocation_key::list(&state.pool, auth.building_id).await?;
    Ok(Json(
        rows.into_iter()
            .map(|r| AllocationKeyResponse {
                id: r.id,
                name: r.name,
                description: r.description,
                key_type: r.key_type,
            })
            .collect(),
    ))
}

/// POST /building/keys
pub async fn create_key(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(input): Json<AllocationKeyInput>,
) -> AppResult<(StatusCode, Json<AllocationKeyResponse>)> {
    require_manager(&auth)?;
    input
        .validate()
        .map_err(|_| AppError::validation_code("errors.allocation_keys.invalid_payload"))?;

    let row = db::allocation_key::create(
        &state.pool,
        auth.building_id,
        &input.name,
        input.description.as_deref(),
        auth.user_id,
        input.justification.as_deref(),
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(AllocationKeyResponse {
            id: row.id,
            name: row.name,
            description: row.description,
            key_type: row.key_type,
        }),
    ))
}

/// PUT /building/keys/{id}
pub async fn update_key(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<AllocationKeyInput>,
) -> AppResult<Json<AllocationKeyResponse>> {
    require_manager(&auth)?;
    input
        .validate()
        .map_err(|_| AppError::validation_code("errors.allocation_keys.invalid_payload"))?;

    let row = db::allocation_key::update(
        &state.pool,
        id,
        auth.building_id,
        &input.name,
        input.description.as_deref(),
        auth.user_id,
        input.justification.as_deref(),
    )
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(AllocationKeyResponse {
        id: row.id,
        name: row.name,
        description: row.description,
        key_type: row.key_type,
    }))
}

/// DELETE /building/keys/{id}
pub async fn delete_key(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<DeleteKeyInput>,
) -> AppResult<StatusCode> {
    require_manager(&auth)?;
    if let Some(j) = &input.justification {
        if j.len() > 500 {
            return Err(AppError::validation_code(
                "errors.allocation_keys.justification_too_long",
            ));
        }
    }
    let deleted = db::allocation_key::delete(
        &state.pool,
        id,
        auth.building_id,
        auth.user_id,
        input.justification.as_deref(),
    )
    .await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

/// GET /building/keys/{id}/shares
pub async fn list_key_shares(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let rows = db::allocation_key::list_shares(&state.pool, id, auth.building_id).await?;
    let building = db::building::get(&state.pool, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let shares_sum: i32 = rows.iter().map(|r| r.share_value).sum();

    let shares: Vec<UnitShareResponse> = rows
        .into_iter()
        .map(|r| UnitShareResponse {
            unit_id: r.unit_id,
            unit_number: r.unit_number,
            share_value: r.share_value,
        })
        .collect();

    Ok(Json(serde_json::json!({
        "shares": shares,
        "shares_sum": shares_sum,
        "shares_total": building.shares_total,
    })))
}

/// GET /building/keys/{id}/balance
/// Returns current shares sum vs building total and a `balanced` flag.
/// Used by the frontend to show the imbalance alert (EP-001-S07).
pub async fn key_balance(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let (sum, total) =
        db::allocation_key::shares_balance(&state.pool, id, auth.building_id).await?;
    Ok(Json(serde_json::json!({
        "shares_sum":   sum,
        "shares_total": total,
        "balanced":     sum == total as i64,
        "delta":        total as i64 - sum,
    })))
}

/// PUT /building/keys/{id}/shares
pub async fn set_key_shares(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<SetSharesInput>,
) -> AppResult<StatusCode> {
    require_manager(&auth)?;
    body.validate()
        .map_err(|_| AppError::validation_code("errors.allocation_keys.justification_too_long"))?;

    // Validate all values first
    let mut parsed: Vec<(Uuid, i32)> = Vec::with_capacity(body.shares.len());
    for item in &body.shares {
        if item.share_value < 0 {
            return Err(AppError::validation_code(
                "errors.allocation_keys.share_value_negative",
            ));
        }
        parsed.push((item.unit_id, item.share_value));
    }

    // Warn-level check: sum vs building total (not blocking — partial allocation is valid)
    let building = db::building::get(&state.pool, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let shares_sum: i32 = parsed.iter().map(|(_, v)| v).sum();
    if shares_sum > building.shares_total {
        return Err(AppError::validation_code(
            "errors.allocation_keys.shares_exceed_total",
        ));
    }

    for (unit_id, share_value) in parsed {
        db::allocation_key::set_share(
            &state.pool,
            unit_id,
            id,
            share_value,
            auth.building_id,
            auth.user_id,
            body.justification.as_deref(),
        )
        .await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

/// GET /building/keys/{id}/audit
pub async fn key_audit(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<crate::db::audit::AuditEntry>>> {
    auth.require_any_role(&[crate::domain::Role::Manager, crate::domain::Role::BoardMember])?;
    let entries =
        db::audit::list_by_entity(&state.pool, auth.building_id, "allocation_key", id).await?;
    Ok(Json(entries))
}

// ── EDD coherence check (EP-001-S09, LEGAL §1.2) ─────────────────

#[derive(Serialize, ToSchema)]
pub struct EddReferenceResponse {
    pub id: Uuid,
    pub label: String,
    pub edd_date: NaiveDate,
    pub document_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct CreateEddReferenceInput {
    #[validate(length(min = 1, max = 200))]
    pub label: String,
    pub edd_date: NaiveDate,
    pub shares: Vec<EddReferenceShareInput>,
}

#[derive(Deserialize, ToSchema)]
pub struct EddReferenceShareInput {
    pub unit_id: Uuid,
    pub allocation_key_id: Uuid,
    #[schema(value_type = f64)]
    pub share_value: Decimal,
}

fn to_edd_reference_response(row: db::edd_reference::EddReferenceRow) -> EddReferenceResponse {
    EddReferenceResponse {
        id: row.id,
        label: row.label,
        edd_date: row.edd_date,
        document_id: row.document_id,
        created_at: row.created_at,
    }
}

/// POST /building/edd-references
pub async fn create_edd_reference(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(input): Json<CreateEddReferenceInput>,
) -> AppResult<(StatusCode, Json<EddReferenceResponse>)> {
    require_manager(&auth)?;
    input
        .validate()
        .map_err(|_| AppError::validation_code("errors.edd.shares_required"))?;

    for share in &input.shares {
        if share.share_value < Decimal::ZERO {
            return Err(AppError::validation_code("errors.edd.invalid_share_value"));
        }
    }

    let shares: Vec<db::edd_reference::NewEddReferenceShare> = input
        .shares
        .into_iter()
        .map(|s| db::edd_reference::NewEddReferenceShare {
            unit_id: s.unit_id,
            allocation_key_id: s.allocation_key_id,
            share_value: s.share_value,
        })
        .collect();

    let row = db::edd_reference::create(
        &state.pool,
        auth.building_id,
        &input.label,
        input.edd_date,
        auth.user_id,
        &shares,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(to_edd_reference_response(row))))
}

/// GET /building/edd-references
pub async fn list_edd_references(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> AppResult<Json<Vec<EddReferenceResponse>>> {
    let rows = db::edd_reference::list(&state.pool, auth.building_id).await?;
    Ok(Json(
        rows.into_iter().map(to_edd_reference_response).collect(),
    ))
}

/// GET /building/edd-check
/// Compares the latest EDD reference with currently configured tantièmes and
/// returns a non-blocking discrepancy report (EP-001-S09, LEGAL §1.2).
pub async fn edd_check(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    let reference = db::edd_reference::get_latest(&state.pool, auth.building_id)
        .await?
        .ok_or_else(|| AppError::not_found_code("errors.edd.no_reference"))?;

    let reference_shares = db::edd_reference::list_shares(&state.pool, reference.id).await?;
    let current_shares =
        db::edd_reference::list_current_shares(&state.pool, auth.building_id).await?;

    let report = crate::domain::edd_check::compare(
        &reference_shares
            .into_iter()
            .map(|s| crate::domain::edd_check::EddRefShare {
                unit_id: s.unit_id,
                allocation_key_id: s.allocation_key_id,
                share_value: s.share_value,
            })
            .collect::<Vec<_>>(),
        &current_shares
            .into_iter()
            .map(|s| crate::domain::edd_check::CurrentShare {
                unit_id: s.unit_id,
                allocation_key_id: s.allocation_key_id,
                share_value: s.share_value,
            })
            .collect::<Vec<_>>(),
    );

    Ok(Json(serde_json::json!({
        "reference": to_edd_reference_response(reference),
        "discrepancies": report.discrepancies,
    })))
}

// ── Registry (consolidated view) ─────────────────────────────────

/// GET /building/registry
/// Returns building info + all active units + all allocation keys + shares matrix
pub async fn registry(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    let building = db::building::get(&state.pool, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let units = db::unit::list(&state.pool, auth.building_id).await?;
    let keys = db::allocation_key::list(&state.pool, auth.building_id).await?;

    // Fetch shares for each key
    let mut keys_with_shares = Vec::new();
    for key in &keys {
        let shares = db::allocation_key::list_shares(&state.pool, key.id, auth.building_id).await?;
        let shares_sum: i32 = shares.iter().map(|s| s.share_value).sum();
        keys_with_shares.push(serde_json::json!({
            "id": key.id,
            "name": key.name,
            "description": key.description,
            "shares_sum": shares_sum,
            "shares": shares.iter().map(|s| serde_json::json!({
                "unit_id": s.unit_id,
                "unit_number": s.unit_number,
                "share_value": s.share_value,
            })).collect::<Vec<_>>(),
        }));
    }

    Ok(Json(serde_json::json!({
        "building": {
            "id": building.id,
            "name": building.name,
            "address_line1": building.address_line1,
            "postal_code": building.postal_code,
            "city": building.city,
            "shares_total": building.shares_total,
        },
        "units": units.iter().map(|u| serde_json::json!({
            "id": u.id,
            "unit_number": u.unit_number,
            "unit_type": u.unit_type,
            "block": u.block,
            "staircase": u.staircase,
            "floor": u.floor,
            "active": u.active,
        })).collect::<Vec<_>>(),
        "keys": keys_with_shares,
    })))
}
