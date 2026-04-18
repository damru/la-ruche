use sqlx::PgPool;
use uuid::Uuid;
use chrono::NaiveDate;
use sqlx::types::Decimal;
use crate::errors::AppResult;

pub async fn list(pool: &PgPool, building_id: Uuid) -> AppResult<Vec<serde_json::Value>> {
    let rows = sqlx::query!(
        r#"SELECT id, title, description, intervention_date, cost,
           vendor_name, contract_id, task_id,
           source::text as source, created_at
           FROM maintenance_log WHERE building_id = $1 ORDER BY intervention_date DESC"#,
        building_id
    ).fetch_all(pool).await?;
    Ok(rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "title": r.title, "description": r.description,
        "intervention_date": r.intervention_date, "cost": r.cost,
        "vendor_name": r.vendor_name, "contract_id": r.contract_id,
        "task_id": r.task_id, "source": r.source, "created_at": r.created_at,
    })).collect())
}

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
    source: &str,
) -> AppResult<serde_json::Value> {
    let id = sqlx::query_scalar!(
        r#"INSERT INTO maintenance_log
           (building_id, created_by, title, description, intervention_date,
            cost, vendor_name, contract_id, task_id, source)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10::log_source)
           RETURNING id"#,
        building_id, user_id, title, description, intervention_date,
        cost, vendor_name, contract_id, task_id, source as _
    ).fetch_one(pool).await?;
    Ok(serde_json::json!({ "id": id }))
}

pub async fn delete(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<()> {
    sqlx::query!(
        "DELETE FROM maintenance_log WHERE id=$1 AND building_id=$2",
        id, building_id
    ).execute(pool).await?;
    Ok(())
}

/// Automatic creation from a closed task
pub async fn from_task(pool: &PgPool, task_id: Uuid, building_id: Uuid, user_id: Uuid) -> AppResult<()> {
    sqlx::query!(
        r#"INSERT INTO maintenance_log (building_id, created_by, title, task_id, intervention_date, source)
           SELECT building_id, $3, title, id, CURRENT_DATE, 'automatic'::log_source
           FROM task WHERE id = $1 AND building_id = $2
           ON CONFLICT DO NOTHING"#,
        task_id, building_id, user_id
    ).execute(pool).await?;
    Ok(())
}
