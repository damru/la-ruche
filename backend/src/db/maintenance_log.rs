use crate::errors::AppResult;
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::types::Decimal;
use sqlx::{Executor, PgPool, Postgres, Row};
use uuid::Uuid;

/// Typed row for maintenance log entries.
#[derive(Debug, Serialize)]
pub struct MaintenanceLogRow {
    pub id: Uuid,
    pub building_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub intervention_date: NaiveDate,
    pub cost: Option<Decimal>,
    pub vendor_name: Option<String>,
    pub contract_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

fn row_from_pg(r: &sqlx::postgres::PgRow) -> Result<MaintenanceLogRow, sqlx::Error> {
    Ok(MaintenanceLogRow {
        id: r.try_get("id")?,
        building_id: r.try_get("building_id")?,
        title: r.try_get("title")?,
        description: r.try_get("description")?,
        intervention_date: r.try_get("intervention_date")?,
        cost: r.try_get("cost")?,
        vendor_name: r.try_get("vendor_name")?,
        contract_id: r.try_get("contract_id")?,
        task_id: r.try_get("task_id")?,
        source: r.try_get("source")?,
        created_at: r.try_get("created_at")?,
    })
}

pub async fn list(pool: &PgPool, building_id: Uuid) -> AppResult<Vec<MaintenanceLogRow>> {
    let rows = sqlx::query(
        r#"SELECT id, building_id, title, description, intervention_date, cost,
           vendor_name, contract_id, task_id,
           source::text AS source, created_at
           FROM maintenance_log WHERE building_id = $1
           ORDER BY intervention_date DESC, created_at DESC"#,
    )
    .bind(building_id)
    .fetch_all(pool)
    .await?;
    rows.iter()
        .map(row_from_pg)
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

pub async fn get(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
) -> AppResult<Option<MaintenanceLogRow>> {
    let row = sqlx::query(
        r#"SELECT id, building_id, title, description, intervention_date, cost,
           vendor_name, contract_id, task_id,
           source::text AS source, created_at
           FROM maintenance_log WHERE id = $1 AND building_id = $2"#,
    )
    .bind(id)
    .bind(building_id)
    .fetch_optional(pool)
    .await?;
    row.as_ref()
        .map(row_from_pg)
        .transpose()
        .map_err(Into::into)
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    pool: &PgPool,
    building_id: Uuid,
    user_id: Uuid,
    title: &str,
    description: Option<&str>,
    intervention_date: NaiveDate,
    cost: Option<Decimal>,
    vendor_name: Option<&str>,
    contract_id: Option<Uuid>,
    task_id: Option<Uuid>,
) -> AppResult<MaintenanceLogRow> {
    let row = sqlx::query(
        r#"INSERT INTO maintenance_log
           (building_id, created_by, title, description, intervention_date,
            cost, vendor_name, contract_id, task_id, source)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,'manual'::log_source)
           RETURNING id, building_id, title, description, intervention_date, cost,
           vendor_name, contract_id, task_id, source::text AS source, created_at"#,
    )
    .bind(building_id)
    .bind(user_id)
    .bind(title)
    .bind(description)
    .bind(intervention_date)
    .bind(cost)
    .bind(vendor_name)
    .bind(contract_id)
    .bind(task_id)
    .fetch_one(pool)
    .await?;
    row_from_pg(&row).map_err(Into::into)
}

#[allow(clippy::too_many_arguments)]
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    title: &str,
    description: Option<&str>,
    intervention_date: NaiveDate,
    cost: Option<Decimal>,
    vendor_name: Option<&str>,
    contract_id: Option<Uuid>,
) -> AppResult<Option<MaintenanceLogRow>> {
    let row = sqlx::query(
        r#"UPDATE maintenance_log
           SET title=$3, description=$4, intervention_date=$5, cost=$6,
               vendor_name=$7, contract_id=$8, updated_at=NOW()
           WHERE id=$1 AND building_id=$2 AND source='manual'
           RETURNING id, building_id, title, description, intervention_date, cost,
           vendor_name, contract_id, task_id, source::text AS source, created_at"#,
    )
    .bind(id)
    .bind(building_id)
    .bind(title)
    .bind(description)
    .bind(intervention_date)
    .bind(cost)
    .bind(vendor_name)
    .bind(contract_id)
    .fetch_optional(pool)
    .await?;
    row.as_ref()
        .map(row_from_pg)
        .transpose()
        .map_err(Into::into)
}

pub async fn delete(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<()> {
    sqlx::query("DELETE FROM maintenance_log WHERE id=$1 AND building_id=$2")
        .bind(id)
        .bind(building_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Automatic creation from a closed task.
pub async fn create_automatic_from_task<'e, E>(
    executor: E,
    task_id: Uuid,
    building_id: Uuid,
    user_id: Uuid,
) -> AppResult<()>
where
    E: Executor<'e, Database = Postgres>,
{
    let result = sqlx::query(
        r#"INSERT INTO maintenance_log
           (building_id, created_by, title, task_id, intervention_date, source, vendor_name, cost)
           SELECT t.building_id, $3, t.title, t.id, CURRENT_DATE, 'automatic'::log_source,
                  COALESCE(c.name, t.contractor_name) AS vendor_name, t.completion_cost
            FROM task t
            LEFT JOIN contractor c ON c.id = t.contractor_id
            WHERE t.id = $1 AND t.building_id = $2 AND t.status = 'closed'::task_status"#,
    )
    .bind(task_id)
    .bind(building_id)
    .bind(user_id)
    .execute(executor)
    .await?;

    if result.rows_affected() == 0 {
        return Err(crate::errors::AppError::NotFound);
    }
    Ok(())
}

pub async fn from_task(
    pool: &PgPool,
    task_id: Uuid,
    building_id: Uuid,
    user_id: Uuid,
) -> AppResult<()> {
    create_automatic_from_task(pool, task_id, building_id, user_id).await?;
    Ok(())
}
