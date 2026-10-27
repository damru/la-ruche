// ── api/logbook.rs — Carnet d'entretien (EP-009) ─────────────────
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
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
    /// Optional category (defaults to `autre`).
    pub category: Option<String>,
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
    /// Optional category (defaults to `autre`).
    pub category: Option<String>,
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
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub amount: Option<Decimal>,
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

/// Resolves the requested category against the `maintenance_category` enum,
/// defaulting to `autre`. Invalid values are rejected with a validation error
/// instead of leaking a raw SQL cast error.
fn resolve_category(category: Option<&str>) -> AppResult<&'static str> {
    match category {
        None => Ok(db::maintenance_log::category::OTHER),
        Some(value) => db::maintenance_log::category_from_str(value)
            .ok_or_else(|| AppError::validation_code("errors.maintenance.invalid_category")),
    }
}

fn map_contract(contract: db::contract::VendorContractRow) -> LogbookContractInfo {
    LogbookContractInfo {
        id: contract.id,
        vendor_name: contract.vendor_name,
        subject: contract.subject,
        contract_type: contract.contract_type,
        start_date: contract.start_date,
        end_date: contract.end_date,
        amount: contract.amount,
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
    let category = resolve_category(input.category.as_deref())?;

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
        category,
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
    let category = resolve_category(input.category.as_deref())?;

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
        category,
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

/// GET /logbook/export/pdf — renders the regulatory logbook (décret n°2001-477,
/// art. 1) as a PDF using the shared document template (EP-009-S09).
/// Access matches `summary` (any authenticated building member): the logbook is
/// a regulatory record every member may consult.
pub async fn export_pdf(auth: AuthUser, State(state): State<Arc<AppState>>) -> AppResult<Response> {
    let building = db::building::get(&state.pool, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let syndic_name =
        db::governance_mandate::current_syndic_name(&state.pool, auth.building_id).await?;

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

    // Syndic contact: current manager user (email/phone) if any.
    let users = db::user::list(&state.pool, auth.building_id).await?;
    let today = Utc::now().date_naive();
    let syndic_contact = users
        .iter()
        .filter_map(|user| {
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
                return None;
            }
            let contact = [user.get("email"), user.get("phone")]
                .into_iter()
                .flatten()
                .filter_map(serde_json::Value::as_str)
                .collect::<Vec<_>>()
                .join(" — ");
            (!contact.is_empty()).then_some(contact)
        })
        .next();

    let contracts = db::contract::list(&state.pool, auth.building_id).await?;
    let active_contracts = contracts
        .into_iter()
        .filter(|contract| is_contract_active(contract, today))
        .collect::<Vec<_>>();

    let to_line = |contract: &db::contract::VendorContractRow, type_label: Option<String>| {
        crate::domain::logbook_pdf::LogbookContractLine {
            subject: contract.subject.clone(),
            vendor_name: contract.vendor_name.clone(),
            contract_type_label: type_label,
            start_date: contract.start_date,
            end_date: contract.end_date,
            amount: contract.amount,
        }
    };

    let insurance_contracts = active_contracts
        .iter()
        .filter(|c| c.contract_type == contract_type::INSURANCE)
        .map(|c| to_line(c, None))
        .collect();
    let maintenance_contracts = active_contracts
        .iter()
        .filter(|c| is_maintenance_contract(&c.contract_type))
        .map(|c| to_line(c, Some(contract_type_label(&c.contract_type))))
        .collect();

    let entries = db::maintenance_log::list(&state.pool, auth.building_id).await?;
    let mut entry_lines = entries
        .iter()
        .map(|row| crate::domain::logbook_pdf::LogbookEntryLine {
            intervention_date: row.intervention_date,
            title: row.title.clone(),
            description: row.description.clone(),
            vendor_name: row.vendor_name.clone(),
            category_label: category_label(&row.category),
            source_label: source_label(&row.source),
            cost: row.cost,
        })
        .collect::<Vec<_>>();
    // Chronologie ascendante pour le document imprimé (la liste API est DESC).
    entry_lines.sort_by_key(|entry| entry.intervention_date);

    let building_name_for_doc = building.name.clone();
    let year = today.format("%Y").to_string();
    let edition_line = format!(
        "Réf. LR-CARNET-{year} · Édition du {}",
        crate::domain::pdf_template::format_date(today)
    );
    let letter_paragraphs = vec![
        format!(
            "Conformément au décret n°2001-477 (loi SRU), le syndic tient à jour le carnet d'entretien de l'immeuble. Vous trouverez en annexe du présent document l'édition du {} du carnet : contrats d'assurance et de maintenance en cours, puis chronologie complète des interventions.",
            crate::domain::pdf_template::format_date_long(today)
        ),
        "Cette édition est établie à sa date d'émission et annule et remplace toute édition antérieure.".to_string(),
        "Nous restons à votre disposition pour toute question relative au contenu de ce document.".to_string(),
    ];
    let data = crate::domain::logbook_pdf::LogbookPdfData {
        identity: crate::domain::pdf_template::DocumentIdentity {
            building_name: building.name,
            building_address: address,
            syndic_name,
            syndic_siret: building.manager_siret,
        },
        edited_at: today,
        syndic_contact,
        insurance_contracts,
        maintenance_contracts,
        entries: entry_lines,
        labels: crate::domain::logbook_pdf::LogbookPdfLabels {
            doc_title: "Carnet d'entretien de l'immeuble".to_string(),
            issuer_heading: "Le syndic des copropriétaires".to_string(),
            issuer_not_set: "Syndic non renseigné".to_string(),
            recipient_label: "Destinataire".to_string(),
            objet: format!("Carnet d'entretien de l'immeuble — édition {year}"),
            reference: Some(edition_line),
            salutation: "Madame, Monsieur,".to_string(),
            letter_paragraphs,
            closing_formula: "Je vous prie d'agréer, Madame, Monsieur, l'expression de mes salutations distinguées.".to_string(),
            signer_closing: "Pour le syndic,".to_string(),
            signer_role: Some(format!("Syndic en exercice — {building_name_for_doc}")),
            place: building.city.clone(),
            annex_kicker: "Annexe".to_string(),
            insurance_section: "Contrats d'assurance en cours".to_string(),
            maintenance_section: "Contrats d'entretien et de maintenance en cours".to_string(),
            chronology_section: "Chronologie des interventions".to_string(),
            no_contracts: "Aucun contrat en cours.".to_string(),
            no_entries: "Aucune intervention enregistrée.".to_string(),
            col_date: "Date".to_string(),
            col_intervention: "Intervention".to_string(),
            col_vendor: "Prestataire".to_string(),
            col_category: "Catégorie".to_string(),
            col_source: "Source".to_string(),
            col_cost: "Coût".to_string(),
            count_one: "1 contrat".to_string(),
            count_other: "{count} contrats".to_string(),
            entries_count_one: "1 intervention".to_string(),
            entries_count_other: "{count} interventions".to_string(),
        },
    };

    let bytes = crate::domain::logbook_pdf::render_logbook(&data)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    let mut response = bytes.into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str("attachment; filename=\"carnet-entretien.pdf\"")
            .unwrap_or(HeaderValue::from_static("attachment")),
    );
    Ok(response)
}

/// FR label for a `contract_type` enum value.
fn contract_type_label(contract_type: &str) -> String {
    match contract_type {
        contract_type::MAINTENANCE => "Maintenance",
        contract_type::SERVICE => "Prestation",
        contract_type::ENERGY => "Énergie",
        contract_type::WATER => "Eau",
        contract_type::CLEANING => "Nettoyage",
        contract_type::GREEN_SPACES => "Espaces verts",
        _ => "Autre",
    }
    .to_string()
}

/// FR label for a `maintenance_category` enum value.
fn category_label(category: &str) -> String {
    match category {
        db::maintenance_log::category::WORKS => "Travaux",
        db::maintenance_log::category::CONTRACT => "Contrat",
        db::maintenance_log::category::CLAIM => "Sinistre",
        _ => "Autre",
    }
    .to_string()
}

/// FR label for a `log_source` enum value.
fn source_label(source: &str) -> String {
    match source {
        db::maintenance_log::log_source::AUTOMATIC => "Généré automatiquement",
        _ => "Saisie manuelle",
    }
    .to_string()
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
    /// Optional category (defaults to `autre`).
    pub category: Option<String>,
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
    let category = resolve_category(input.category.as_deref())?;

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
        category,
    )
    .await?;

    Ok((StatusCode::CREATED, json_value(entry)?))
}

