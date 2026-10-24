// ── api/logbook.rs — Carnet d'entretien (EP-009) ─────────────────
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Decimal;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    db::{
        self, AppState,
        contract::{contract_status, contract_type},
    },
    domain::{AuthUser, Role},
    errors::{AppError, AppResult, json_value},
};

/// Input for manual logbook entry creation.
#[derive(Deserialize)]
pub struct CreateLogbookInput {
    pub title: String,
    pub description: Option<String>,
    pub intervention_date: NaiveDate,
    pub cost: Option<Decimal>,
    pub vendor_name: Option<String>,
    /// Optional link to a vendor contract (EP-009-S05).
    pub contract_id: Option<Uuid>,
    /// Optional link to a task (manual association).
    pub task_id: Option<Uuid>,
}

/// Input for updating a manual logbook entry.
#[derive(Deserialize)]
pub struct UpdateLogbookInput {
    pub title: String,
    pub description: Option<String>,
    pub intervention_date: NaiveDate,
    pub cost: Option<Decimal>,
    pub vendor_name: Option<String>,
    /// Optional link to a vendor contract (EP-009-S05).
    pub contract_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct LogbookBuildingInfo {
    pub name: String,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub postal_code: String,
    pub city: String,
}

#[derive(Serialize)]
pub struct LogbookManagerInfo {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
}

#[derive(Serialize)]
pub struct LogbookContractInfo {
    pub id: Uuid,
    pub vendor_name: String,
    pub subject: String,
    pub contract_type: String,
    pub end_date: Option<NaiveDate>,
    pub status: String,
}

#[derive(Serialize)]
pub struct LogbookSummaryResponse {
    pub building: LogbookBuildingInfo,
    pub current_manager: Option<LogbookManagerInfo>,
    pub insurance_contracts: Vec<LogbookContractInfo>,
    pub maintenance_contracts: Vec<LogbookContractInfo>,
}

fn is_contract_active(contract: &db::contract::VendorContractRow, today: NaiveDate) -> bool {
    contract.status == contract_status::ACTIVE && contract.end_date.is_none_or(|date| date >= today)
}

/// `contract_type` values classified as maintenance contracts for the logbook summary.
const MAINTENANCE_CONTRACT_TYPES: &[&str] = &[
    contract_type::MAINTENANCE,
    contract_type::SERVICE,
    contract_type::ENERGY,
    contract_type::WATER,
    contract_type::CLEANING,
    contract_type::GREEN_SPACES,
];

fn is_maintenance_contract(contract_type: &str) -> bool {
    MAINTENANCE_CONTRACT_TYPES.contains(&contract_type)
}

fn map_contract(contract: db::contract::VendorContractRow) -> LogbookContractInfo {
    LogbookContractInfo {
        id: contract.id,
        vendor_name: contract.vendor_name,
        subject: contract.subject,
        contract_type: contract.contract_type,
        end_date: contract.end_date,
        status: contract.status,
    }
}

fn parse_manager_info(user: &serde_json::Value) -> AppResult<Option<LogbookManagerInfo>> {
    let roles = user
        .get("roles")
        .and_then(serde_json::Value::as_array)
        .map(|roles| {
            roles
                .iter()
                .filter_map(serde_json::Value::as_str)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if !roles.contains(&Role::Manager.as_str()) {
        return Ok(None);
    }

    let require_field = |field: &str| {
        user.get(field)
            .and_then(serde_json::Value::as_str)
            .map(ToString::to_string)
            .ok_or_else(|| {
                AppError::Internal(anyhow::anyhow!(
                    "manager row missing '{}' in logbook summary",
                    field
                ))
            })
    };

    Ok(Some(LogbookManagerInfo {
        first_name: require_field("first_name")?,
        last_name: require_field("last_name")?,
        email: require_field("email")?,
        phone: user
            .get("phone")
            .and_then(serde_json::Value::as_str)
            .map(ToString::to_string),
    }))
}

/// All authenticated users can list the maintenance log.
pub async fn list(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let rows = db::maintenance_log::list(&state.pool, auth.building_id).await?;
    json_value(rows)
}

/// All authenticated users can access the regulatory summary of the logbook.
pub async fn summary(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<LogbookSummaryResponse>> {
    let building = db::building::get(&state.pool, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let users = db::user::list(&state.pool, auth.building_id).await?;
    let today = Utc::now().date_naive();

    let current_manager = users
        .iter()
        .map(parse_manager_info)
        .find_map(|manager| manager.transpose())
        .transpose()?;

    let contracts = db::contract::list(&state.pool, auth.building_id).await?;
    let active_contracts = contracts
        .into_iter()
        .filter(|contract| is_contract_active(contract, today))
        .collect::<Vec<_>>();

    let insurance_contracts = active_contracts
        .iter()
        .filter(|contract| contract.contract_type == contract_type::INSURANCE)
        .cloned()
        .map(map_contract)
        .collect();

    let maintenance_contracts = active_contracts
        .into_iter()
        .filter(|contract| is_maintenance_contract(&contract.contract_type))
        .map(map_contract)
        .collect();

    Ok(Json(LogbookSummaryResponse {
        building: LogbookBuildingInfo {
            name: building.name,
            address_line1: building.address_line1,
            address_line2: building.address_line2,
            postal_code: building.postal_code,
            city: building.city,
        },
        current_manager,
        insurance_contracts,
        maintenance_contracts,
    }))
}

/// All authenticated users can view a single log entry.
pub async fn get(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let entry = db::maintenance_log::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    json_value(entry)
}

/// Only managers and board members can manually create log entries.
/// `source` is always forced to `manual` — automatic entries come from task closure.
pub async fn create(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateLogbookInput>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;

    if input.title.trim().is_empty() {
        return Err(AppError::validation_code("errors.logbook.title_required"));
    }

    let entry = db::maintenance_log::create(
        &state.pool,
        auth.building_id,
        auth.user_id,
        &input.title,
        input.description.as_deref(),
        input.intervention_date,
        input.cost,
        input.vendor_name.as_deref(),
        input.contract_id,
        input.task_id,
    )
    .await?;

    Ok((StatusCode::CREATED, json_value(entry)?))
}

/// Only managers and board members can edit manual log entries.
/// Automatic entries (source = 'automatic') are frozen — assert_mutable enforces this.
pub async fn update(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateLogbookInput>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    let existing = db::maintenance_log::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    crate::domain::immutability::assert_mutable(
        &crate::domain::immutability::Entity::MaintenanceLog,
        &existing.source,
    )?;
    if input.title.trim().is_empty() {
        return Err(AppError::validation_code("errors.logbook.title_required"));
    }

    let entry = db::maintenance_log::update(
        &state.pool,
        id,
        auth.building_id,
        &input.title,
        input.description.as_deref(),
        input.intervention_date,
        input.cost,
        input.vendor_name.as_deref(),
        input.contract_id,
    )
    .await?
    .ok_or(AppError::NotFound)?;

    json_value(entry)
}

/// Only managers can delete logbook entries. Automatic entries are frozen.
pub async fn delete(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    let existing = db::maintenance_log::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    crate::domain::immutability::assert_mutable(
        &crate::domain::immutability::Entity::MaintenanceLog,
        &existing.source,
    )?;
    db::maintenance_log::delete(&state.pool, id, auth.building_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Input for a correction entry (append-only amendment of a frozen manual entry).
#[derive(Deserialize)]
pub struct CorrectionInput {
    pub title: String,
    pub description: Option<String>,
    pub intervention_date: NaiveDate,
    pub cost: Option<Decimal>,
    pub vendor_name: Option<String>,
    pub contract_id: Option<Uuid>,
}

/// Creates a correction entry referencing an existing manual log entry (EP-018-S07 Pattern C).
pub async fn correction(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<CorrectionInput>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;

    if input.title.trim().is_empty() {
        return Err(AppError::validation_code("errors.logbook.title_required"));
    }

    let entry = db::maintenance_log::create_correction(
        &state.pool,
        id,
        auth.building_id,
        auth.user_id,
        &input.title,
        input.description.as_deref(),
        input.intervention_date,
        input.cost,
        input.vendor_name.as_deref(),
        input.contract_id,
    )
    .await?;

    Ok((StatusCode::CREATED, json_value(entry)?))
}
