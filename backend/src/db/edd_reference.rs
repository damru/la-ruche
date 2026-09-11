use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct EddReferenceRow {
    pub id: Uuid,
    pub building_id: Uuid,
    pub label: String,
    pub edd_date: NaiveDate,
    pub document_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct EddReferenceShareRow {
    pub unit_id: Uuid,
    pub allocation_key_id: Uuid,
    pub share_value: Decimal,
}

pub struct NewEddReferenceShare {
    pub unit_id: Uuid,
    pub allocation_key_id: Uuid,
    pub share_value: Decimal,
}

/// Creates an EDD reference and its shares in a single transaction.
pub async fn create(
    pool: &PgPool,
    building_id: Uuid,
    label: &str,
    edd_date: NaiveDate,
    created_by: Uuid,
    shares: &[NewEddReferenceShare],
) -> crate::errors::AppResult<EddReferenceRow> {
    let mut tx = pool.begin().await?;

    let reference = sqlx::query_as::<_, EddReferenceRow>(
        "INSERT INTO edd_reference (building_id, label, edd_date, created_by) \
         VALUES ($1, $2, $3, $4) \
         RETURNING id, building_id, label, edd_date, document_id, created_by, created_at",
    )
    .bind(building_id)
    .bind(label)
    .bind(edd_date)
    .bind(created_by)
    .fetch_one(&mut *tx)
    .await?;

    for share in shares {
        sqlx::query(
            "INSERT INTO edd_reference_share (edd_reference_id, unit_id, allocation_key_id, share_value) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(reference.id)
        .bind(share.unit_id)
        .bind(share.allocation_key_id)
        .bind(share.share_value)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(reference)
}

pub async fn list(
    pool: &PgPool,
    building_id: Uuid,
) -> crate::errors::AppResult<Vec<EddReferenceRow>> {
    Ok(sqlx::query_as::<_, EddReferenceRow>(
        "SELECT id, building_id, label, edd_date, document_id, created_by, created_at \
         FROM edd_reference WHERE building_id = $1 \
         ORDER BY edd_date DESC, created_at DESC",
    )
    .bind(building_id)
    .fetch_all(pool)
    .await?)
}

/// Latest EDD reference for a building, ordered by `edd_date` then `created_at`.
pub async fn get_latest(
    pool: &PgPool,
    building_id: Uuid,
) -> crate::errors::AppResult<Option<EddReferenceRow>> {
    Ok(sqlx::query_as::<_, EddReferenceRow>(
        "SELECT id, building_id, label, edd_date, document_id, created_by, created_at \
         FROM edd_reference WHERE building_id = $1 \
         ORDER BY edd_date DESC, created_at DESC LIMIT 1",
    )
    .bind(building_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_shares(
    pool: &PgPool,
    edd_reference_id: Uuid,
) -> crate::errors::AppResult<Vec<EddReferenceShareRow>> {
    Ok(sqlx::query_as::<_, EddReferenceShareRow>(
        "SELECT unit_id, allocation_key_id, share_value \
         FROM edd_reference_share WHERE edd_reference_id = $1",
    )
    .bind(edd_reference_id)
    .fetch_all(pool)
    .await?)
}

/// Currently configured tantièmes for a building, across all active keys and units.
#[derive(Debug, sqlx::FromRow)]
pub struct CurrentShareRow {
    pub unit_id: Uuid,
    pub allocation_key_id: Uuid,
    pub share_value: Decimal,
}

pub async fn list_current_shares(
    pool: &PgPool,
    building_id: Uuid,
) -> crate::errors::AppResult<Vec<CurrentShareRow>> {
    Ok(sqlx::query_as::<_, CurrentShareRow>(
        "SELECT uak.unit_id, uak.key_id AS allocation_key_id, uak.share_value::numeric AS share_value \
         FROM unit_allocation_key uak \
         JOIN unit u ON u.id = uak.unit_id \
         JOIN allocation_key ak ON ak.id = uak.key_id \
         WHERE u.building_id = $1 AND u.active = true \
           AND uak.valid_to IS NULL AND ak.deleted_at IS NULL",
    )
    .bind(building_id)
    .fetch_all(pool)
    .await?)
}
