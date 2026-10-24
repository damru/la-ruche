use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::PgPool;
use sqlx::types::Decimal;
use uuid::Uuid;

use crate::domain::etat_date::{EtatDateInputs, ExerciceQuotePart, ProcedureSummary};
use crate::errors::{AppError, AppResult};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct EtatDateRequestRow {
    pub id: Uuid,
    pub unit_id: Uuid,
    pub requested_at: NaiveDate,
    pub due_date: NaiveDate,
    pub requester_name: String,
    pub status: String,
    pub snapshot: Option<serde_json::Value>,
    pub generated_at: Option<DateTime<Utc>>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

pub const STATUS_REQUESTED: &str = "requested";
pub const STATUS_GENERATED: &str = "generated";
pub const STATUS_SENT: &str = "sent";
pub const STATUS_CANCELLED: &str = "cancelled";

const GET_QUERY: &str = concat!(
    "SELECT edr.id, edr.unit_id, edr.requested_at, edr.due_date, edr.requester_name,",
    " edr.status::text AS status, edr.snapshot, edr.generated_at, edr.created_by, edr.created_at",
    " FROM etat_date_request edr",
    " JOIN unit u ON u.id = edr.unit_id AND u.building_id = $2",
    " WHERE edr.id = $1",
);

const LIST_QUERY: &str = concat!(
    "SELECT edr.id, edr.unit_id, edr.requested_at, edr.due_date, edr.requester_name,",
    " edr.status::text AS status, edr.snapshot, edr.generated_at, edr.created_by, edr.created_at",
    " FROM etat_date_request edr",
    " JOIN unit u ON u.id = edr.unit_id AND u.building_id = $1",
    " ORDER BY edr.due_date ASC, edr.created_at DESC",
);

const STORE_SNAPSHOT_QUERY: &str = concat!(
    "UPDATE etat_date_request edr SET",
    " status = $5::text::etat_date_status,",
    " snapshot = $3,",
    " generated_at = NOW()",
    " FROM unit u",
    " WHERE edr.id = $1 AND u.id = edr.unit_id AND u.building_id = $2",
    "   AND edr.status <> $4::text::etat_date_status",
    " RETURNING edr.id, edr.unit_id, edr.requested_at, edr.due_date, edr.requester_name,",
    " edr.status::text AS status, edr.snapshot, edr.generated_at, edr.created_by, edr.created_at",
);

const UPDATE_STATUS_QUERY: &str = concat!(
    "UPDATE etat_date_request edr SET status = $3::etat_date_status",
    " FROM unit u",
    " WHERE edr.id = $1 AND u.id = edr.unit_id AND u.building_id = $2",
    " RETURNING edr.id, edr.unit_id, edr.requested_at, edr.due_date, edr.requester_name,",
    " edr.status::text AS status, edr.snapshot, edr.generated_at, edr.created_by, edr.created_at",
);

pub async fn create_request(
    pool: &PgPool,
    unit_id: Uuid,
    building_id: Uuid,
    requested_at: NaiveDate,
    requester_name: &str,
    created_by: Uuid,
) -> AppResult<EtatDateRequestRow> {
    // Ensure the unit belongs to this building before creating the request.
    let unit_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM unit WHERE id = $1 AND building_id = $2)")
            .bind(unit_id)
            .bind(building_id)
            .fetch_one(pool)
            .await?;
    if !unit_exists {
        return Err(AppError::NotFound);
    }

    let due_date = crate::domain::etat_date::compute_due_date(requested_at);

    let new_id: Uuid = sqlx::query_scalar(
        r#"INSERT INTO etat_date_request
               (unit_id, requested_at, due_date, requester_name, created_by)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id"#,
    )
    .bind(unit_id)
    .bind(requested_at)
    .bind(due_date)
    .bind(requester_name.trim())
    .bind(created_by)
    .fetch_one(pool)
    .await?;

    get(pool, new_id, building_id)
        .await?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("created etat_date_request not found")))
}

pub async fn get(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
) -> AppResult<Option<EtatDateRequestRow>> {
    sqlx::query_as::<_, EtatDateRequestRow>(GET_QUERY)
        .bind(id)
        .bind(building_id)
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
}

pub async fn list_for_building(
    pool: &PgPool,
    building_id: Uuid,
) -> AppResult<Vec<EtatDateRequestRow>> {
    sqlx::query_as::<_, EtatDateRequestRow>(LIST_QUERY)
        .bind(building_id)
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

/// Freezes the assembled document as `snapshot`, moving status to `generated`.
/// Refused once `sent` (probative value: a transmitted document is no longer
/// regenerable — see errors.etat_date.already_sent at the API layer).
pub async fn store_snapshot(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    snapshot: &serde_json::Value,
) -> AppResult<Option<EtatDateRequestRow>> {
    sqlx::query_as::<_, EtatDateRequestRow>(STORE_SNAPSHOT_QUERY)
        .bind(id)
        .bind(building_id)
        .bind(snapshot)
        .bind(STATUS_SENT)
        .bind(STATUS_GENERATED)
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
}

pub async fn update_status(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    status: &str,
) -> AppResult<Option<EtatDateRequestRow>> {
    sqlx::query_as::<_, EtatDateRequestRow>(UPDATE_STATUS_QUERY)
        .bind(id)
        .bind(building_id)
        .bind(status)
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
}

#[derive(Debug, sqlx::FromRow)]
struct ExerciceRow {
    fiscal_year: i32,
    charge_amount: Decimal,
    works_fund_amount: Decimal,
}

#[derive(Debug, sqlx::FromRow)]
struct ProcedureRow {
    subject: String,
    proceeding_type: String,
    opened_at: NaiveDate,
    jurisdiction: Option<String>,
}

/// Loads and shapes every data source the état daté depends on for a given
/// lot: outstanding charges (EP-004), works fund quote-part (EP-004-S03) and
/// ongoing judicial proceedings (EP-002-S10, registre `judicial_proceeding`).
/// Pure SQL boundary — `domain::etat_date::assemble` never touches the DB.
pub async fn load_inputs(
    pool: &PgPool,
    unit_id: Uuid,
    building_id: Uuid,
) -> AppResult<EtatDateInputs> {
    // Sommes dues par le cédant : charges émises et non soldées (hors fonds travaux).
    let arrears: Decimal = sqlx::query_scalar(
        r#"SELECT COALESCE(SUM(lcu.amount), 0)::numeric
           FROM levy_call_unit lcu
           JOIN levy_call lc ON lc.id = lcu.levy_call_id
           JOIN budget b     ON b.id = lc.budget_id AND b.building_id = $2
           WHERE lcu.unit_id = $1
             AND lc.status = $3::text::levy_status
             AND lcu.status <> $4::text::levy_status
             AND lcu.budget_line_id IS NOT NULL
             AND lcu.amount > 0"#,
    )
    .bind(unit_id)
    .bind(building_id)
    .bind(crate::db::budget::levy_status::ISSUED)
    .bind(crate::db::budget::levy_status::PAID)
    .fetch_one(pool)
    .await?;

    // Sommes dont le syndicat est débiteur : avoirs (montants négatifs) émis pour ce lot.
    let avances_syndicat: Decimal = sqlx::query_scalar(
        r#"SELECT COALESCE(SUM(-lcu.amount), 0)::numeric
           FROM levy_call_unit lcu
           JOIN levy_call lc ON lc.id = lcu.levy_call_id
           JOIN budget b     ON b.id = lc.budget_id AND b.building_id = $2
           WHERE lcu.unit_id = $1
             AND lc.status = $3::text::levy_status
             AND lcu.budget_line_id IS NOT NULL
             AND lcu.amount < 0"#,
    )
    .bind(unit_id)
    .bind(building_id)
    .bind(crate::db::budget::levy_status::ISSUED)
    .fetch_one(pool)
    .await?;

    // Quote-part fonds travaux du lot : part du lot dans le fonds disponible
    // de l'exercice en cours, au prorata de la clé générale (LEGAL.md §4.4).
    let fonds_travaux_quote_part: Decimal = sqlx::query_scalar(
        r#"SELECT COALESCE(
               (SELECT wf.available_amount * uak.share_value / NULLIF(b.shares_total, 0)
                FROM works_fund wf
                JOIN unit_allocation_key uak
                    ON uak.unit_id = $1 AND uak.valid_to IS NULL
                JOIN allocation_key ak
                    ON ak.id = uak.key_id AND ak.key_type = 'general' AND ak.deleted_at IS NULL
                JOIN building b ON b.id = wf.building_id
                WHERE wf.building_id = $2
                ORDER BY wf.fiscal_year DESC
                LIMIT 1),
               0
           )::numeric"#,
    )
    .bind(unit_id)
    .bind(building_id)
    .fetch_one(pool)
    .await?;

    // Annexe : quote-part du lot sur les 2 derniers exercices ayant émis des appels.
    let exercice_rows = sqlx::query_as::<_, ExerciceRow>(
        r#"SELECT
               b.fiscal_year,
               COALESCE(SUM(lcu.amount) FILTER (WHERE lcu.budget_line_id IS NOT NULL), 0)::numeric AS charge_amount,
               COALESCE(SUM(lcu.amount) FILTER (WHERE lcu.budget_line_id IS NULL), 0)::numeric     AS works_fund_amount
           FROM levy_call_unit lcu
           JOIN levy_call lc ON lc.id = lcu.levy_call_id
           JOIN budget b     ON b.id = lc.budget_id AND b.building_id = $2
           WHERE lcu.unit_id = $1 AND lc.status = $3::text::levy_status
           GROUP BY b.fiscal_year
           ORDER BY b.fiscal_year DESC
           LIMIT 2"#,
    )
    .bind(unit_id)
    .bind(building_id)
    .bind(crate::db::budget::levy_status::ISSUED)
    .fetch_all(pool)
    .await?;

    let exercices = exercice_rows
        .into_iter()
        .map(|r| ExerciceQuotePart {
            fiscal_year: r.fiscal_year,
            charge_amount: r.charge_amount,
            works_fund_amount: r.works_fund_amount,
            total_amount: r.charge_amount + r.works_fund_amount,
        })
        .collect();

    // Procédures en cours (registre EP-002-S10) — alimente obligatoirement l'état daté.
    let procedure_rows = sqlx::query_as::<_, ProcedureRow>(
        r#"SELECT subject, proceeding_type::text AS proceeding_type, opened_at, jurisdiction
           FROM judicial_proceeding
           WHERE building_id = $1 AND status = $2::text::judicial_proceeding_status
           ORDER BY opened_at DESC"#,
    )
    .bind(building_id)
    .bind(crate::db::judicial_proceeding::PROCEEDING_STATUS_ONGOING)
    .fetch_all(pool)
    .await?;

    let procedures = procedure_rows
        .into_iter()
        .map(|r| ProcedureSummary {
            subject: r.subject,
            proceeding_type: r.proceeding_type,
            opened_at: r.opened_at,
            jurisdiction: r.jurisdiction,
        })
        .collect();

    Ok(EtatDateInputs {
        arrears,
        avances_syndicat,
        provisions_a_venir: Decimal::ZERO,
        fonds_travaux_quote_part,
        exercices,
        procedures,
    })
}

/// Extra inputs for the pré-état daté (EP-002-S07) not carried by
/// `EtatDateInputs`: the syndicat's global unpaid balance (all lots, not just
/// the cédant's) and the works fund's last contribution call amount.
pub struct PreEtatDateExtras {
    pub impayes_globaux_syndicat: Decimal,
    pub fonds_travaux_derniere_cotisation: Decimal,
}

pub async fn load_pre_etat_date_extras(
    pool: &PgPool,
    building_id: Uuid,
) -> AppResult<PreEtatDateExtras> {
    // État global des impayés du syndicat : toutes les charges émises et non
    // soldées, tous lots confondus (hors fonds travaux).
    let impayes_globaux_syndicat: Decimal = sqlx::query_scalar(
        r#"SELECT COALESCE(SUM(lcu.amount), 0)::numeric
           FROM levy_call_unit lcu
           JOIN levy_call lc ON lc.id = lcu.levy_call_id
           JOIN budget b     ON b.id = lc.budget_id AND b.building_id = $1
           WHERE lc.status = $2::text::levy_status
             AND lcu.status <> $3::text::levy_status
             AND lcu.budget_line_id IS NOT NULL
             AND lcu.amount > 0"#,
    )
    .bind(building_id)
    .bind(crate::db::budget::levy_status::ISSUED)
    .bind(crate::db::budget::levy_status::PAID)
    .fetch_one(pool)
    .await?;

    // Dernière cotisation au fonds travaux : montant du dernier appel émis
    // comportant une part fonds travaux.
    let fonds_travaux_derniere_cotisation: Decimal = sqlx::query_scalar(
        r#"SELECT COALESCE(
               (SELECT lc.works_fund_amount
                FROM levy_call lc
                JOIN budget b ON b.id = lc.budget_id AND b.building_id = $1
                WHERE lc.works_fund_amount > 0
                ORDER BY lc.issued_at DESC
                LIMIT 1),
               0
           )::numeric"#,
    )
    .bind(building_id)
    .fetch_one(pool)
    .await?;

    Ok(PreEtatDateExtras {
        impayes_globaux_syndicat,
        fonds_travaux_derniere_cotisation,
    })
}
