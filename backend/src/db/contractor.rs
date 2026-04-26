// ── db/contractor.rs ─────────────────────────────────────────────────────────
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::errors::AppResult;

#[derive(Debug, Serialize)]
pub struct ContractorRow {
    pub id: Uuid,
    pub building_id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub domain: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn row_to_contractor(r: &sqlx::postgres::PgRow) -> Result<ContractorRow, sqlx::Error> {
    Ok(ContractorRow {
        id: r.try_get("id")?,
        building_id: r.try_get("building_id")?,
        name: r.try_get("name")?,
        email: r.try_get("email")?,
        phone: r.try_get("phone")?,
        domain: r.try_get("domain")?,
        created_at: r.try_get("created_at")?,
        updated_at: r.try_get("updated_at")?,
    })
}

const SELECT: &str =
    "SELECT id, building_id, name, email, phone, domain, created_at, updated_at FROM contractor";

/// List all contractors for a building, ordered by name.
pub async fn list(pool: &PgPool, building_id: Uuid) -> AppResult<Vec<ContractorRow>> {
    let rows = sqlx::query(&format!("{SELECT} WHERE building_id = $1 ORDER BY name"))
        .bind(building_id)
        .fetch_all(pool)
        .await?;
    rows.iter()
        .map(row_to_contractor)
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

/// Search contractors by name (case-insensitive, ILIKE), limited to 10 results.
/// Used for autocomplete when assigning a contractor to a task.
pub async fn search(pool: &PgPool, building_id: Uuid, q: &str) -> AppResult<Vec<ContractorRow>> {
    let rows = sqlx::query(&format!(
        "{SELECT} WHERE building_id = $1 AND name ILIKE $2 ORDER BY name LIMIT 10"
    ))
    .bind(building_id)
    .bind(format!("%{q}%"))
    .fetch_all(pool)
    .await?;
    rows.iter()
        .map(row_to_contractor)
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

/// Fetch a single contractor by id, scoped to the building.
pub async fn get(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<Option<ContractorRow>> {
    let row = sqlx::query(&format!("{SELECT} WHERE id = $1 AND building_id = $2"))
        .bind(id)
        .bind(building_id)
        .fetch_optional(pool)
        .await?;
    row.as_ref()
        .map(row_to_contractor)
        .transpose()
        .map_err(Into::into)
}

#[derive(Debug, Deserialize)]
pub struct CreateContractorInput {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub domain: Option<String>,
}

pub async fn create(
    pool: &PgPool,
    building_id: Uuid,
    input: CreateContractorInput,
) -> AppResult<ContractorRow> {
    let row = sqlx::query(
        r#"INSERT INTO contractor (building_id, name, email, phone, domain)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, building_id, name, email, phone, domain, created_at, updated_at"#,
    )
    .bind(building_id)
    .bind(&input.name)
    .bind(&input.email)
    .bind(&input.phone)
    .bind(&input.domain)
    .fetch_one(pool)
    .await?;
    row_to_contractor(&row).map_err(Into::into)
}

#[derive(Debug, Deserialize)]
pub struct UpdateContractorInput {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub domain: Option<String>,
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    input: UpdateContractorInput,
) -> AppResult<Option<ContractorRow>> {
    let row = sqlx::query(
        r#"UPDATE contractor
           SET name       = COALESCE($3, name),
               email      = COALESCE($4, email),
               phone      = COALESCE($5, phone),
               domain     = COALESCE($6, domain),
               updated_at = NOW()
           WHERE id = $1 AND building_id = $2
           RETURNING id, building_id, name, email, phone, domain, created_at, updated_at"#,
    )
    .bind(id)
    .bind(building_id)
    .bind(&input.name)
    .bind(&input.email)
    .bind(&input.phone)
    .bind(&input.domain)
    .fetch_optional(pool)
    .await?;
    row.as_ref()
        .map(row_to_contractor)
        .transpose()
        .map_err(Into::into)
}

pub async fn delete(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<bool> {
    let result = sqlx::query("DELETE FROM contractor WHERE id = $1 AND building_id = $2")
        .bind(id)
        .bind(building_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
