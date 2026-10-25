use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct JudicialProceedingRow {
    pub id: Uuid,
    pub building_id: Uuid,
    pub proceeding_type: String,
    pub status: String,
    pub subject: String,
    pub claimant: String,
    pub defendant: String,
    pub jurisdiction: Option<String>,
    pub case_reference: Option<String>,
    pub opened_at: NaiveDate,
    pub closed_at: Option<NaiveDate>,
    pub outcome: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProceedingInput {
    pub proceeding_type: String,
    pub subject: String,
    pub claimant: String,
    pub defendant: String,
    pub jurisdiction: Option<String>,
    pub case_reference: Option<String>,
    pub opened_at: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProceedingInput {
    pub proceeding_type: String,
    pub status: String,
    pub subject: String,
    pub claimant: String,
    pub defendant: String,
    pub jurisdiction: Option<String>,
    pub case_reference: Option<String>,
    pub opened_at: NaiveDate,
    pub closed_at: Option<NaiveDate>,
    pub outcome: Option<String>,
}

pub const PROCEEDING_STATUS_ONGOING: &str = "ongoing";
pub const PROCEEDING_STATUS_SUSPENDED: &str = "suspended";
pub const PROCEEDING_STATUS_CLOSED: &str = "closed";
const VALID_TYPES: &[&str] = &["debt_recovery", "works_dispute", "liability", "other"];
const VALID_STATUSES: &[&str] = &[
    PROCEEDING_STATUS_ONGOING,
    PROCEEDING_STATUS_SUSPENDED,
    PROCEEDING_STATUS_CLOSED,
];

const LIST_QUERY: &str = concat!(
    "SELECT id, building_id, proceeding_type::text AS proceeding_type,",
    " status::text AS status, subject, claimant, defendant, jurisdiction,",
    " case_reference, opened_at, closed_at, outcome, created_by, created_at, updated_at",
    " FROM judicial_proceeding WHERE building_id = $1",
    " ORDER BY opened_at DESC, created_at DESC",
);

const LIST_ONGOING_QUERY: &str = concat!(
    "SELECT id, building_id, proceeding_type::text AS proceeding_type,",
    " status::text AS status, subject, claimant, defendant, jurisdiction,",
    " case_reference, opened_at, closed_at, outcome, created_by, created_at, updated_at",
    " FROM judicial_proceeding WHERE building_id = $1 AND status <> $2::text::judicial_proceeding_status",
    " ORDER BY opened_at DESC",
);

const GET_QUERY: &str = concat!(
    "SELECT id, building_id, proceeding_type::text AS proceeding_type,",
    " status::text AS status, subject, claimant, defendant, jurisdiction,",
    " case_reference, opened_at, closed_at, outcome, created_by, created_at, updated_at",
    " FROM judicial_proceeding WHERE id = $1 AND building_id = $2",
);

fn validate_payload(
    proceeding_type: &str,
    status: Option<&str>,
    subject: &str,
    claimant: &str,
    defendant: &str,
    opened_at: NaiveDate,
    closed_at: Option<NaiveDate>,
) -> AppResult<()> {
    if subject.trim().is_empty() {
        return Err(AppError::validation_code(
            "errors.judicial_proceedings.subject_required",
        ));
    }
    if claimant.trim().is_empty() {
        return Err(AppError::validation_code(
            "errors.judicial_proceedings.claimant_required",
        ));
    }
    if defendant.trim().is_empty() {
        return Err(AppError::validation_code(
            "errors.judicial_proceedings.defendant_required",
        ));
    }
    if !VALID_TYPES.contains(&proceeding_type) {
        return Err(AppError::validation_code(
            "errors.judicial_proceedings.invalid_type",
        ));
    }
    if let Some(status) = status {
        if !VALID_STATUSES.contains(&status) {
            return Err(AppError::validation_code(
                "errors.judicial_proceedings.invalid_status",
            ));
        }
        let is_closed = status == PROCEEDING_STATUS_CLOSED;
        if is_closed && closed_at.is_none() {
            return Err(AppError::validation_code(
                "errors.judicial_proceedings.closed_requires_closed_at",
            ));
        }
        if !is_closed && closed_at.is_some() {
            return Err(AppError::validation_code(
                "errors.judicial_proceedings.closed_at_only_when_closed",
            ));
        }
    }
    if let Some(closed_at) = closed_at
        && closed_at < opened_at
    {
        return Err(AppError::validation_code(
            "errors.judicial_proceedings.closed_before_opened",
        ));
    }
    Ok(())
}

pub async fn list(pool: &PgPool, building_id: Uuid) -> AppResult<Vec<JudicialProceedingRow>> {
    sqlx::query_as::<_, JudicialProceedingRow>(LIST_QUERY)
        .bind(building_id)
        .bind(PROCEEDING_STATUS_CLOSED)
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

pub async fn list_ongoing(
    pool: &PgPool,
    building_id: Uuid,
) -> AppResult<Vec<JudicialProceedingRow>> {
    sqlx::query_as::<_, JudicialProceedingRow>(LIST_ONGOING_QUERY)
        .bind(building_id)
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

pub async fn get(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
) -> AppResult<Option<JudicialProceedingRow>> {
    sqlx::query_as::<_, JudicialProceedingRow>(GET_QUERY)
        .bind(id)
        .bind(building_id)
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
}

pub async fn create(
    pool: &PgPool,
    building_id: Uuid,
    created_by: Uuid,
    input: CreateProceedingInput,
) -> AppResult<JudicialProceedingRow> {
    validate_payload(
        &input.proceeding_type,
        None,
        &input.subject,
        &input.claimant,
        &input.defendant,
        input.opened_at,
        None,
    )?;

    let new_id: Uuid = sqlx::query_scalar(
        r#"INSERT INTO judicial_proceeding
               (building_id, proceeding_type, subject, claimant, defendant,
                jurisdiction, case_reference, opened_at, created_by)
           VALUES ($1, $2::text::judicial_proceeding_type, $3, $4, $5, $6, $7, $8, $9)
           RETURNING id"#,
    )
    .bind(building_id)
    .bind(&input.proceeding_type)
    .bind(input.subject.trim())
    .bind(input.claimant.trim())
    .bind(input.defendant.trim())
    .bind(input.jurisdiction.as_deref().map(str::trim))
    .bind(input.case_reference.as_deref().map(str::trim))
    .bind(input.opened_at)
    .bind(created_by)
    .fetch_one(pool)
    .await?;

    get(pool, new_id, building_id)
        .await?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("created proceeding not found")))
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    input: UpdateProceedingInput,
) -> AppResult<Option<JudicialProceedingRow>> {
    validate_payload(
        &input.proceeding_type,
        Some(&input.status),
        &input.subject,
        &input.claimant,
        &input.defendant,
        input.opened_at,
        input.closed_at,
    )?;

    let result = sqlx::query(
        r#"UPDATE judicial_proceeding SET
               proceeding_type = $3::text::judicial_proceeding_type,
               status = $4::text::judicial_proceeding_status,
               subject = $5,
               claimant = $6,
               defendant = $7,
               jurisdiction = $8,
               case_reference = $9,
               opened_at = $10,
               closed_at = $11,
               outcome = $12,
               updated_at = NOW()
           WHERE id = $1 AND building_id = $2"#,
    )
    .bind(id)
    .bind(building_id)
    .bind(&input.proceeding_type)
    .bind(&input.status)
    .bind(input.subject.trim())
    .bind(input.claimant.trim())
    .bind(input.defendant.trim())
    .bind(input.jurisdiction.as_deref().map(str::trim))
    .bind(input.case_reference.as_deref().map(str::trim))
    .bind(input.opened_at)
    .bind(input.closed_at)
    .bind(input.outcome.as_deref().map(str::trim))
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Ok(None);
    }
    get(pool, id, building_id).await
}

pub async fn delete(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<bool> {
    let result = sqlx::query("DELETE FROM judicial_proceeding WHERE id = $1 AND building_id = $2")
        .bind(id)
        .bind(building_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn rejects_blank_subject() {
        let err = validate_payload(
            "other",
            None,
            "  ",
            "Syndicat",
            "M. Dupont",
            date(2026, 1, 1),
            None,
        );
        assert!(err.is_err());
    }

    #[test]
    fn rejects_closed_without_closed_at() {
        let err = validate_payload(
            "debt_recovery",
            Some("closed"),
            "Recouvrement",
            "Syndicat",
            "M. Dupont",
            date(2026, 1, 1),
            None,
        );
        assert!(err.is_err());
    }

    #[test]
    fn rejects_closed_at_when_not_closed() {
        let err = validate_payload(
            "debt_recovery",
            Some("ongoing"),
            "Recouvrement",
            "Syndicat",
            "M. Dupont",
            date(2026, 1, 1),
            Some(date(2026, 2, 1)),
        );
        assert!(err.is_err());
    }

    #[test]
    fn rejects_closed_before_opened() {
        let err = validate_payload(
            "debt_recovery",
            Some("closed"),
            "Recouvrement",
            "Syndicat",
            "M. Dupont",
            date(2026, 2, 1),
            Some(date(2026, 1, 1)),
        );
        assert!(err.is_err());
    }

    #[test]
    fn accepts_valid_closed_proceeding() {
        let ok = validate_payload(
            "liability",
            Some("closed"),
            "Dégât des eaux",
            "Syndicat",
            "Assureur",
            date(2026, 1, 1),
            Some(date(2026, 3, 1)),
        );
        assert!(ok.is_ok());
    }
}
