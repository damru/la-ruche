use sqlx::PgPool;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, NaiveDate, Utc};

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
pub struct CreateUnitInput {
    pub unit_number: String,
    pub unit_type: String,
    pub block: Option<String>,
    pub staircase: Option<String>,
    pub floor: Option<String>,
    pub description: Option<String>,
}

pub async fn list(pool: &PgPool, building_id: Uuid) -> crate::errors::AppResult<Vec<UnitRow>> {
    Ok(sqlx::query_as!(UnitRow,
        r#"SELECT id, building_id, unit_number, unit_type::text as "unit_type!",
           block, staircase, floor, description, active, created_at
           FROM unit WHERE building_id = $1 AND active = true ORDER BY unit_number"#,
        building_id
    ).fetch_all(pool).await?)
}

pub async fn get(pool: &PgPool, id: Uuid, building_id: Uuid) -> crate::errors::AppResult<Option<UnitRow>> {
    Ok(sqlx::query_as!(UnitRow,
        r#"SELECT id, building_id, unit_number, unit_type::text as "unit_type!",
           block, staircase, floor, description, active, created_at
           FROM unit WHERE id = $1 AND building_id = $2"#,
        id, building_id
    ).fetch_optional(pool).await?)
}

pub async fn create(pool: &PgPool, building_id: Uuid, input: CreateUnitInput) -> crate::errors::AppResult<UnitRow> {
    Ok(sqlx::query_as!(UnitRow,
        r#"INSERT INTO unit (building_id, unit_number, unit_type, block, staircase, floor, description)
           VALUES ($1, $2, $3::text::unit_type, $4, $5, $6, $7)
           RETURNING id, building_id, unit_number, unit_type::text as "unit_type!",
           block, staircase, floor, description, active, created_at"#,
        building_id, input.unit_number, input.unit_type,
        input.block, input.staircase, input.floor, input.description
    ).fetch_one(pool).await?)
}

pub async fn archive(pool: &PgPool, id: Uuid, building_id: Uuid) -> crate::errors::AppResult<()> {
    sqlx::query!(
        "UPDATE unit SET active = false, updated_at = NOW() WHERE id = $1 AND building_id = $2",
        id, building_id
    ).execute(pool).await?;
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

pub async fn list_owners(pool: &PgPool, unit_id: Uuid, building_id: Uuid) -> crate::errors::AppResult<Vec<OwnerRecord>> {
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

    Ok(rows.into_iter().map(|r| OwnerRecord {
        id: r.id,
        unit_id: r.unit_id,
        user_id: r.user_id,
        display_name: r.display_name.unwrap_or_default(),
        start_date: r.start_date,
        end_date: r.end_date,
        anonymized: r.anonymized.unwrap_or(false),
    }).collect())
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
        start_date, unit_id
    ).execute(&mut *tx).await?;

    let _ = sqlx::query_scalar!(
        "SELECT id FROM unit WHERE id = $1 AND building_id = $2",
        unit_id, building_id
    ).fetch_one(&mut *tx).await?;

    sqlx::query!(
        "INSERT INTO unit_owner (unit_id, user_id, start_date) VALUES ($1, $2, $3)",
        unit_id, new_owner_id, start_date
    ).execute(&mut *tx).await?;

    tx.commit().await?;
    Ok(())
}

pub async fn set_shares(
    pool: &PgPool,
    unit_id: Uuid,
    key_id: Uuid,
    share_value: sqlx::types::Decimal,
) -> crate::errors::AppResult<()> {
    sqlx::query!(
        r#"INSERT INTO unit_allocation_key (unit_id, key_id, share_value) VALUES ($1, $2, $3)
           ON CONFLICT (unit_id, key_id) DO UPDATE SET share_value = EXCLUDED.share_value"#,
        unit_id, key_id, share_value
    ).execute(pool).await?;
    Ok(())
}
