use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UnitRow {
    pub id: Uuid,
    pub building_id: Uuid,
    pub unit_number: String,
    pub unit_type: String,
    pub block: Option<String>,
    pub staircase: Option<String>,
    pub floor: Option<String>,
    pub description: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUnitInput {
    pub unit_number: String,
    pub unit_type: String,
    pub block: Option<String>,
    pub staircase: Option<String>,
    pub floor: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUnitInput {
    pub unit_number: String,
    pub unit_type: String,
    pub block: Option<String>,
    pub staircase: Option<String>,
    pub floor: Option<String>,
    pub description: Option<String>,
}

pub async fn list(pool: &PgPool, building_id: Uuid) -> crate::errors::AppResult<Vec<UnitRow>> {
    Ok(sqlx::query_as!(
        UnitRow,
        r#"SELECT id, building_id, unit_number, unit_type::text as "unit_type!",
           block, staircase, floor, description, active, created_at
           FROM unit WHERE building_id = $1 AND active = true ORDER BY unit_number"#,
        building_id
    )
    .fetch_all(pool)
    .await?)
}

pub async fn is_archived(
    pool: &PgPool,
    building_id: Uuid,
    unit_number: &str,
) -> crate::errors::AppResult<bool> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM unit WHERE building_id = $1 AND unit_number = $2 AND active = false",
    )
    .bind(building_id)
    .bind(unit_number)
    .fetch_one(pool)
    .await?;
    Ok(count > 0)
}

pub async fn get(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<Option<UnitRow>> {
    Ok(sqlx::query_as!(
        UnitRow,
        r#"SELECT id, building_id, unit_number, unit_type::text as "unit_type!",
           block, staircase, floor, description, active, created_at
           FROM unit WHERE id = $1 AND building_id = $2"#,
        id,
        building_id
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn create(
    pool: &PgPool,
    building_id: Uuid,
    input: CreateUnitInput,
) -> crate::errors::AppResult<UnitRow> {
    Ok(sqlx::query_as!(UnitRow,
        r#"INSERT INTO unit (building_id, unit_number, unit_type, block, staircase, floor, description)
           VALUES ($1, $2, $3::text::unit_type, $4, $5, $6, $7)
           RETURNING id, building_id, unit_number, unit_type::text as "unit_type!",
           block, staircase, floor, description, active, created_at"#,
        building_id, input.unit_number, input.unit_type,
        input.block, input.staircase, input.floor, input.description
    ).fetch_one(pool).await?)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    input: UpdateUnitInput,
) -> crate::errors::AppResult<Option<UnitRow>> {
    Ok(sqlx::query_as::<_, UnitRow>(
        r#"UPDATE unit SET
               unit_number = $3, unit_type = $4::text::unit_type,
               block = $5, staircase = $6, floor = $7, description = $8,
               updated_at = NOW()
           WHERE id = $1 AND building_id = $2 AND active = true
           RETURNING id, building_id, unit_number, unit_type::text as unit_type,
           block, staircase, floor, description, active, created_at"#,
    )
    .bind(id)
    .bind(building_id)
    .bind(&input.unit_number)
    .bind(&input.unit_type)
    .bind(&input.block)
    .bind(&input.staircase)
    .bind(&input.floor)
    .bind(&input.description)
    .fetch_optional(pool)
    .await?)
}

pub async fn archive(pool: &PgPool, id: Uuid, building_id: Uuid) -> crate::errors::AppResult<()> {
    let mut tx = pool.begin().await?;

    let archived = sqlx::query(
        "UPDATE unit SET active = false, updated_at = NOW() WHERE id = $1 AND building_id = $2",
    )
    .bind(id)
    .bind(building_id)
    .execute(&mut *tx)
    .await?;

    if archived.rows_affected() > 0 {
        sqlx::query(
            "UPDATE unit_allocation_key SET valid_to = CURRENT_DATE WHERE unit_id = $1 AND valid_to IS NULL",
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct OwnerRecord {
    pub id: Uuid,
    pub unit_id: Uuid,
    pub user_id: Option<Uuid>,
    pub display_name: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub anonymized: bool,
}

pub async fn list_owners(
    pool: &PgPool,
    unit_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<Vec<OwnerRecord>> {
    let rows = sqlx::query!(
        r#"
        SELECT uo.id, uo.unit_id, uo.user_id,
               COALESCE(uo.anonymized_name, u.first_name || ' ' || u.last_name, 'Unknown owner') as display_name,
               uo.start_date, uo.end_date,
               (uo.anonymized_at IS NOT NULL) as anonymized
        FROM unit_owner uo
        JOIN unit u2 ON u2.id = uo.unit_id AND u2.building_id = $2
        LEFT JOIN user_account u ON u.id = uo.user_id
        WHERE uo.unit_id = $1
        ORDER BY uo.start_date DESC
        "#,
        unit_id, building_id
    ).fetch_all(pool).await?;

    Ok(rows
        .into_iter()
        .map(|r| OwnerRecord {
            id: r.id,
            unit_id: r.unit_id,
            user_id: r.user_id,
            display_name: r.display_name.unwrap_or_default(),
            start_date: r.start_date,
            end_date: r.end_date,
            anonymized: r.anonymized.unwrap_or(false),
        })
        .collect())
}

#[derive(Debug, sqlx::FromRow)]
pub struct CurrentOwnerRow {
    pub unit_id: Uuid,
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
}

/// Fetches the current owner (end_date IS NULL) for each of the given unit IDs.
/// Used by the list handler to enrich unit responses in a single extra query.
pub async fn list_current_owners(
    pool: &PgPool,
    unit_ids: &[Uuid],
) -> crate::errors::AppResult<Vec<CurrentOwnerRow>> {
    if unit_ids.is_empty() {
        return Ok(vec![]);
    }
    Ok(sqlx::query_as::<_, CurrentOwnerRow>(
        r#"
        SELECT uo.unit_id, ua.id as user_id, ua.first_name, ua.last_name, ua.email
        FROM unit_owner uo
        JOIN user_account ua ON ua.id = uo.user_id
        WHERE uo.unit_id = ANY($1::uuid[])
          AND uo.end_date IS NULL
          AND uo.user_id IS NOT NULL
        "#,
    )
    .bind(unit_ids)
    .fetch_all(pool)
    .await?)
}

pub async fn transfer_ownership(
    pool: &PgPool,
    unit_id: Uuid,
    building_id: Uuid,
    new_owner_id: Uuid,
    start_date: NaiveDate,
) -> crate::errors::AppResult<()> {
    let mut tx = pool.begin().await?;

    sqlx::query!(
        "UPDATE unit_owner SET end_date = $1 WHERE unit_id = $2 AND end_date IS NULL",
        start_date,
        unit_id
    )
    .execute(&mut *tx)
    .await?;

    let _ = sqlx::query_scalar!(
        "SELECT id FROM unit WHERE id = $1 AND building_id = $2",
        unit_id,
        building_id
    )
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query!(
        "INSERT INTO unit_owner (unit_id, user_id, start_date) VALUES ($1, $2, $3)",
        unit_id,
        new_owner_id,
        start_date
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn set_shares(
    pool: &PgPool,
    unit_id: Uuid,
    key_id: Uuid,
    share_value: i32,
) -> crate::errors::AppResult<()> {
    sqlx::query!(
        r#"INSERT INTO unit_allocation_key (unit_id, key_id, share_value) VALUES ($1, $2, $3)
           ON CONFLICT (unit_id, key_id) DO UPDATE SET share_value = EXCLUDED.share_value"#,
        unit_id,
        key_id,
        share_value
    )
    .execute(pool)
    .await?;
    Ok(())
}
