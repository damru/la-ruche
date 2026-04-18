use sqlx::PgPool;
use uuid::Uuid;
use chrono::NaiveDate;
use crate::errors::AppResult;

pub async fn list(pool: &PgPool, building_id: Uuid) -> AppResult<Vec<serde_json::Value>> {
    let rows = sqlx::query!(
        r#"SELECT id, title, description, declared_at, status::text as status,
           unit_id, created_at FROM claim WHERE building_id = $1 ORDER BY created_at DESC"#,
        building_id
    ).fetch_all(pool).await?;
    Ok(rows.into_iter().map(|r| serde_json::json!({
        "id": r.id, "title": r.title, "description": r.description,
        "declared_at": r.declared_at, "status": r.status,
        "unit_id": r.unit_id, "created_at": r.created_at,
    })).collect())
}

pub async fn create(
    pool: &PgPool, building_id: Uuid, user_id: Uuid,
    title: &str, description: Option<&str>,
    declared_at: NaiveDate, unit_id: Option<Uuid>,
) -> AppResult<serde_json::Value> {
    let row = sqlx::query!(
        r#"INSERT INTO claim (building_id, created_by, title, description, declared_at, unit_id)
           VALUES ($1,$2,$3,$4,$5,$6) RETURNING id, status::text as status"#,
        building_id, user_id, title, description, declared_at, unit_id
    ).fetch_one(pool).await?;
    Ok(serde_json::json!({ "id": row.id, "status": row.status }))
}

pub async fn update_status(pool: &PgPool, id: Uuid, building_id: Uuid, status: &str) -> AppResult<()> {
    sqlx::query!(
        "UPDATE claim SET status=$3::text::claim_status, updated_at=NOW() WHERE id=$1 AND building_id=$2",
        id, building_id, status
    ).execute(pool).await?;
    Ok(())
}
