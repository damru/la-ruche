use chrono::{DateTime, Datelike, Days, NaiveDate, Utc, Weekday};
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::{PgPool, Row, types::Decimal};
use std::str::FromStr;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

#[derive(Debug, Serialize)]
pub struct ClaimRow {
    pub id: Uuid,
    pub building_id: Uuid,
    pub created_by: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub claim_type: String,
    pub declared_at: Option<NaiveDate>,
    pub published_at: Option<NaiveDate>,
    pub status: String,
    pub unit_id: Option<Uuid>,
    pub transmitted_at: Option<NaiveDate>,
    pub reimbursement_date: Option<NaiveDate>,
    pub reimbursement_amount: Option<Decimal>,
    pub legal_deadline_at: Option<NaiveDate>,
    pub days_remaining: Option<i64>,
    pub is_deadline_approaching: bool,
    pub is_deadline_missed: bool,
    pub is_transmitted_late: bool,
    pub document_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateClaimInput {
    pub title: String,
    pub description: Option<String>,
    pub claim_type: String,
    pub published_at: Option<NaiveDate>,
    pub unit_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateClaimInput {
    pub title: String,
    pub description: Option<String>,
    pub claim_type: String,
    pub declared_at: Option<NaiveDate>,
    pub published_at: Option<NaiveDate>,
    pub status: String,
    pub unit_id: Option<Uuid>,
    pub transmitted_at: Option<NaiveDate>,
    pub reimbursement_date: Option<NaiveDate>,
    #[serde(default, deserialize_with = "deserialize_optional_decimal")]
    pub reimbursement_amount: Option<Decimal>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum FlexibleDecimalInput {
    String(String),
    Number(f64),
}

fn deserialize_optional_decimal<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<FlexibleDecimalInput>::deserialize(deserializer)?;
    value
        .map(|input| {
            let raw = match input {
                FlexibleDecimalInput::String(value) => value,
                FlexibleDecimalInput::Number(value) => value.to_string(),
            };
            Decimal::from_str(&raw).map_err(serde::de::Error::custom)
        })
        .transpose()
}

pub const CLAIM_TYPE_CURRENT: &str = "current";
pub const CLAIM_TYPE_NATURAL_DISASTER: &str = "natural_disaster";
pub const CLAIM_TYPE_THEFT: &str = "theft";
pub const VALID_CLAIM_TYPES: &[&str] = &[
    CLAIM_TYPE_CURRENT,
    CLAIM_TYPE_NATURAL_DISASTER,
    CLAIM_TYPE_THEFT,
];

pub const CLAIM_STATUS_DRAFT: &str = "draft";
pub const CLAIM_STATUS_OPEN: &str = "open";
pub const CLAIM_STATUS_IN_PROGRESS: &str = "in_progress";
pub const CLAIM_STATUS_CLOSED: &str = "closed";
pub const VALID_STATUSES: &[&str] = &[
    CLAIM_STATUS_DRAFT,
    CLAIM_STATUS_OPEN,
    CLAIM_STATUS_IN_PROGRESS,
    CLAIM_STATUS_CLOSED,
];

pub fn can_declare(status: &str) -> bool {
    status == CLAIM_STATUS_DRAFT
}

const LIST_QUERY: &str = concat!(
    "SELECT c.id, c.building_id, c.created_by, c.title, c.description,",
    " c.claim_type::text AS claim_type, c.declared_at, c.published_at,",
    " c.status::text AS status, c.unit_id, c.transmitted_at,",
    " c.reimbursement_date, c.reimbursement_amount,",
    " (SELECT COUNT(*) FROM claim_document cd",
    "   JOIN document d ON d.id = cd.document_id AND d.deleted_at IS NULL",
    "   WHERE cd.claim_id = c.id) AS document_count,",
    " c.created_at, c.updated_at",
    " FROM claim c WHERE c.building_id = $1",
    " AND ($2 OR c.status <> 'draft')",
    " ORDER BY c.updated_at DESC",
);

const GET_QUERY: &str = concat!(
    "SELECT c.id, c.building_id, c.created_by, c.title, c.description,",
    " c.claim_type::text AS claim_type, c.declared_at, c.published_at,",
    " c.status::text AS status, c.unit_id, c.transmitted_at,",
    " c.reimbursement_date, c.reimbursement_amount,",
    " (SELECT COUNT(*) FROM claim_document cd",
    "   JOIN document d ON d.id = cd.document_id AND d.deleted_at IS NULL",
    "   WHERE cd.claim_id = c.id) AS document_count,",
    " c.created_at, c.updated_at",
    " FROM claim c WHERE c.id = $1 AND c.building_id = $2",
);

pub fn validate_claim_type(claim_type: &str) -> bool {
    VALID_CLAIM_TYPES.contains(&claim_type)
}

pub fn validate_status(status: &str) -> bool {
    VALID_STATUSES.contains(&status)
}

pub fn validate_payload(
    title: &str,
    claim_type: &str,
    status: Option<&str>,
    declared_at: Option<NaiveDate>,
    published_at: Option<NaiveDate>,
    transmitted_at: Option<NaiveDate>,
    reimbursement_date: Option<NaiveDate>,
    reimbursement_amount: Option<Decimal>,
) -> AppResult<()> {
    if title.trim().is_empty() {
        return Err(AppError::validation_code("errors.claims.title_required"));
    }
    if !validate_claim_type(claim_type) {
        return Err(AppError::validation_code("errors.claims.invalid_type"));
    }
    if let Some(status) = status
        && !validate_status(status)
    {
        return Err(AppError::validation_code("errors.claims.invalid_status"));
    }
    if claim_type == CLAIM_TYPE_NATURAL_DISASTER {
        if published_at.is_none() {
            return Err(AppError::validation_code(
                "errors.claims.published_at_required_for_natural_disaster",
            ));
        }
    } else if published_at.is_some() {
        return Err(AppError::validation_code(
            "errors.claims.published_at_only_for_natural_disaster",
        ));
    }
    if let (Some(declared), Some(transmitted)) = (declared_at, transmitted_at)
        && transmitted < declared
    {
        return Err(AppError::validation_code(
            "errors.claims.transmitted_before_reference_date",
        ));
    }
    if reimbursement_amount.is_some_and(|amount| amount <= Decimal::ZERO) {
        return Err(AppError::validation_code(
            "errors.claims.reimbursement_amount_positive",
        ));
    }
    if status.is_none() && (reimbursement_date.is_some() || reimbursement_amount.is_some()) {
        return Err(AppError::validation_code(
            "errors.claims.reimbursement_requires_status",
        ));
    }
    if matches!(status, Some(CLAIM_STATUS_OPEN | CLAIM_STATUS_IN_PROGRESS))
        && (reimbursement_date.is_some() || reimbursement_amount.is_some())
    {
        return Err(AppError::validation_code(
            "errors.claims.reimbursement_only_for_closed",
        ));
    }
    Ok(())
}

fn add_business_days(mut date: NaiveDate, days: u32) -> NaiveDate {
    let mut remaining = days;
    while remaining > 0 {
        date = date
            .succ_opt()
            .expect("valid business-day progression for claim deadline");
        if !matches!(date.weekday(), Weekday::Sat | Weekday::Sun) {
            remaining -= 1;
        }
    }
    date
}

fn reference_date(
    claim_type: &str,
    declared_at: NaiveDate,
    published_at: Option<NaiveDate>,
) -> NaiveDate {
    if claim_type == CLAIM_TYPE_NATURAL_DISASTER {
        published_at.unwrap_or(declared_at)
    } else {
        declared_at
    }
}

fn legal_deadline_at(
    claim_type: &str,
    declared_at: NaiveDate,
    published_at: Option<NaiveDate>,
) -> NaiveDate {
    match claim_type {
        CLAIM_TYPE_CURRENT => add_business_days(declared_at, 5),
        CLAIM_TYPE_THEFT => add_business_days(declared_at, 2),
        CLAIM_TYPE_NATURAL_DISASTER => {
            reference_date(claim_type, declared_at, published_at) + Days::new(10)
        }
        _ => add_business_days(declared_at, 5),
    }
}

fn approaching_threshold_days(claim_type: &str) -> i64 {
    match claim_type {
        CLAIM_TYPE_THEFT => 1,
        CLAIM_TYPE_NATURAL_DISASTER => 3,
        _ => 2,
    }
}

fn row_to_claim(row: &sqlx::postgres::PgRow) -> Result<ClaimRow, sqlx::Error> {
    let claim_type: String = row.try_get("claim_type")?;
    let declared_at: Option<NaiveDate> = row.try_get("declared_at")?;
    let published_at: Option<NaiveDate> = row.try_get("published_at")?;
    let transmitted_at: Option<NaiveDate> = row.try_get("transmitted_at")?;
    let reimbursement_date: Option<NaiveDate> = row.try_get("reimbursement_date")?;
    let reimbursement_amount: Option<Decimal> = row.try_get("reimbursement_amount")?;
    let today = Utc::now().date_naive();

    let (
        deadline,
        days_remaining,
        is_transmitted_late,
        is_deadline_missed,
        is_deadline_approaching,
    ) = match declared_at {
        Some(declared) => {
            let deadline = legal_deadline_at(&claim_type, declared, published_at);
            let days = transmitted_at.map_or_else(|| Some((deadline - today).num_days()), |_| None);
            let late = transmitted_at.is_some_and(|date| date > deadline);
            let missed = transmitted_at.is_none() && today > deadline;
            let approaching = transmitted_at.is_none()
                && days.is_some_and(|d| d >= 0 && d <= approaching_threshold_days(&claim_type));
            (Some(deadline), days, late, missed, approaching)
        }
        None => (None, None, false, false, false),
    };

    Ok(ClaimRow {
        id: row.try_get("id")?,
        building_id: row.try_get("building_id")?,
        created_by: row.try_get("created_by")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        claim_type,
        declared_at,
        published_at,
        status: row.try_get("status")?,
        unit_id: row.try_get("unit_id")?,
        transmitted_at,
        reimbursement_date,
        reimbursement_amount,
        legal_deadline_at: deadline,
        days_remaining,
        is_deadline_approaching,
        is_deadline_missed,
        is_transmitted_late,
        document_count: row.try_get("document_count")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub async fn list(
    pool: &PgPool,
    building_id: Uuid,
    include_drafts: bool,
) -> AppResult<Vec<ClaimRow>> {
    let rows = sqlx::query(LIST_QUERY)
        .bind(building_id)
        .bind(include_drafts)
        .fetch_all(pool)
        .await?;
    rows.iter()
        .map(row_to_claim)
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

pub async fn get(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<Option<ClaimRow>> {
    let row = sqlx::query(GET_QUERY)
        .bind(id)
        .bind(building_id)
        .fetch_optional(pool)
        .await?;
    row.as_ref()
        .map(row_to_claim)
        .transpose()
        .map_err(Into::into)
}

pub async fn create(
    pool: &PgPool,
    building_id: Uuid,
    user_id: Uuid,
    input: CreateClaimInput,
) -> AppResult<ClaimRow> {
    validate_payload(
        &input.title,
        &input.claim_type,
        None,
        None,
        input.published_at,
        None,
        None,
        None,
    )?;

    let row = sqlx::query(
        r#"INSERT INTO claim
           (building_id, title, description, claim_type, published_at, unit_id, created_by)
           VALUES ($1, $2, $3, $4::text::claim_type, $5, $6, $7)
           RETURNING id"#,
    )
    .bind(building_id)
    .bind(input.title.trim())
    .bind(input.description)
    .bind(input.claim_type)
    .bind(input.published_at)
    .bind(input.unit_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let claim_id: Uuid = row.try_get("id")?;
    get(pool, claim_id, building_id)
        .await?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("created claim not found")))
}

pub async fn declare(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    declared_at: Option<NaiveDate>,
) -> AppResult<ClaimRow> {
    let today = Utc::now().date_naive();
    let effective_date = declared_at.unwrap_or(today);

    if effective_date > today {
        return Err(AppError::validation_code(
            "errors.claims.declared_at_in_future",
        ));
    }

    let row = sqlx::query(
        "SELECT status::text AS status FROM claim WHERE id = $1 AND building_id = $2 FOR UPDATE",
    )
    .bind(id)
    .bind(building_id)
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Err(AppError::NotFound);
    };

    let status: String = row.try_get("status")?;
    if !can_declare(&status) {
        return Err(AppError::validation_code("errors.claims.not_declarable"));
    }

    sqlx::query(
        "UPDATE claim SET status = 'open', declared_at = $3, updated_at = NOW()
         WHERE id = $1 AND building_id = $2",
    )
    .bind(id)
    .bind(building_id)
    .bind(effective_date)
    .execute(pool)
    .await?;

    get(pool, id, building_id)
        .await?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("declared claim not found")))
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    input: UpdateClaimInput,
) -> AppResult<Option<ClaimRow>> {
    validate_payload(
        &input.title,
        &input.claim_type,
        Some(&input.status),
        input.declared_at,
        input.published_at,
        input.transmitted_at,
        input.reimbursement_date,
        input.reimbursement_amount,
    )?;

    let result = sqlx::query(
        r#"UPDATE claim SET
           title = $3,
           description = $4,
           claim_type = $5::text::claim_type,
           published_at = $6,
           status = $7::text::claim_status,
           unit_id = $8,
           transmitted_at = $9,
           reimbursement_date = $10,
           reimbursement_amount = $11,
           updated_at = NOW()
           WHERE id = $1 AND building_id = $2"#,
    )
    .bind(id)
    .bind(building_id)
    .bind(input.title.trim())
    .bind(input.description)
    .bind(input.claim_type)
    .bind(input.published_at)
    .bind(input.status)
    .bind(input.unit_id)
    .bind(input.transmitted_at)
    .bind(input.reimbursement_date)
    .bind(input.reimbursement_amount)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Ok(None);
    }

    get(pool, id, building_id).await
}

pub async fn list_documents(
    pool: &PgPool,
    claim_id: Uuid,
    building_id: Uuid,
) -> AppResult<Vec<serde_json::Value>> {
    let rows = sqlx::query(
        r#"SELECT d.id, d.original_name, d.mime_type, d.size_bytes, d.created_at
           FROM claim_document cd
           JOIN document d ON d.id = cd.document_id AND d.building_id = $2
           WHERE cd.claim_id = $1 AND d.deleted_at IS NULL
           ORDER BY d.created_at ASC"#,
    )
    .bind(claim_id)
    .bind(building_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.try_get::<Uuid, _>("id").unwrap_or_default(),
                "original_name": row.try_get::<String, _>("original_name").unwrap_or_default(),
                "mime_type": row.try_get::<String, _>("mime_type").unwrap_or_default(),
                "size_bytes": row.try_get::<i64, _>("size_bytes").unwrap_or_default(),
                "created_at": row.try_get::<DateTime<Utc>, _>("created_at").unwrap_or_default(),
            })
        })
        .collect())
}

pub async fn delete(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<Option<()>> {
    let mut tx = pool.begin().await?;
    let linked_document_ids = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT cd.document_id
        FROM claim_document cd
        JOIN claim c ON c.id = cd.claim_id
        WHERE cd.claim_id = $1 AND c.building_id = $2
        "#,
    )
    .bind(id)
    .bind(building_id)
    .fetch_all(&mut *tx)
    .await?;

    let result = sqlx::query("DELETE FROM claim WHERE id = $1 AND building_id = $2")
        .bind(id)
        .bind(building_id)
        .execute(&mut *tx)
        .await?;

    if result.rows_affected() == 0 {
        return Ok(None);
    }

    crate::db::document::delete_orphaned_tx(&mut tx, &linked_document_ids, building_id).await?;
    tx.commit().await?;
    Ok(Some(()))
}

pub async fn attach_document(
    pool: &PgPool,
    claim_id: Uuid,
    document_id: Uuid,
    building_id: Uuid,
) -> AppResult<bool> {
    let pair_exists: bool = sqlx::query_scalar(
        r#"SELECT EXISTS(
               SELECT 1 FROM claim c, document d
               WHERE c.id = $1 AND c.building_id = $3
                 AND d.id = $2 AND d.building_id = $3
           )"#,
    )
    .bind(claim_id)
    .bind(document_id)
    .bind(building_id)
    .fetch_one(pool)
    .await?;

    if !pair_exists {
        return Ok(false);
    }

    sqlx::query(
        "INSERT INTO claim_document (claim_id, document_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(claim_id)
    .bind(document_id)
    .execute(pool)
    .await?;

    Ok(true)
}

pub async fn detach_document(
    pool: &PgPool,
    claim_id: Uuid,
    document_id: Uuid,
    building_id: Uuid,
) -> AppResult<bool> {
    let result = sqlx::query(
        r#"DELETE FROM claim_document cd
           USING claim c
           WHERE cd.claim_id = $1
             AND cd.document_id = $2
             AND c.id = cd.claim_id
             AND c.building_id = $3"#,
    )
    .bind(claim_id)
    .bind(document_id)
    .bind(building_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_declare_only_from_draft() {
        assert!(can_declare(CLAIM_STATUS_DRAFT));
        assert!(!can_declare(CLAIM_STATUS_OPEN));
        assert!(!can_declare(CLAIM_STATUS_IN_PROGRESS));
        assert!(!can_declare(CLAIM_STATUS_CLOSED));
    }

    #[test]
    fn rejects_reimbursement_before_closed_status() {
        let declared_at = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
        let result = validate_payload(
            "Dégât des eaux",
            CLAIM_TYPE_CURRENT,
            Some(CLAIM_STATUS_IN_PROGRESS),
            Some(declared_at),
            None,
            Some(declared_at),
            None,
            Some(Decimal::new(15000, 2)),
        );
        assert!(result.is_err());
    }

    #[test]
    fn accepts_closed_claim_with_optional_reimbursement() {
        let declared_at = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
        let reimbursement_date = NaiveDate::from_ymd_opt(2026, 5, 20).unwrap();
        let result = validate_payload(
            "Vol cave",
            CLAIM_TYPE_THEFT,
            Some(CLAIM_STATUS_CLOSED),
            Some(declared_at),
            None,
            Some(declared_at),
            Some(reimbursement_date),
            Some(Decimal::new(9500, 2)),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn accepts_create_payload_without_declared_at() {
        let result = validate_payload(
            "Fuite hall B",
            CLAIM_TYPE_CURRENT,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!(result.is_ok());
    }
}
