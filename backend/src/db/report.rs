use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::errors::AppResult;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ReportRow {
    pub id: Uuid,
    pub building_id: Uuid,
    pub unit_id: Option<Uuid>,
    pub author_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn list(pool: &PgPool, building_id: Uuid) -> AppResult<Vec<ReportRow>> {
    Ok(sqlx::query_as!(ReportRow,
        r#"SELECT id, building_id, unit_id, author_id, title, description,
           status::text as "status!", created_at, updated_at
           FROM report WHERE building_id = $1 ORDER BY created_at DESC"#,
        building_id
    ).fetch_all(pool).await?)
}

pub async fn create(
    pool: &PgPool,
    building_id: Uuid,
    author_id: Uuid,
    title: &str,
    description: Option<&str>,
    unit_id: Option<Uuid>,
) -> AppResult<ReportRow> {
    Ok(sqlx::query_as!(ReportRow,
        r#"INSERT INTO report (building_id, author_id, title, description, unit_id)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, building_id, unit_id, author_id, title, description,
           status::text as "status!", created_at, updated_at"#,
        building_id, author_id, title, description, unit_id
    ).fetch_one(pool).await?)
}

pub async fn update_status(pool: &PgPool, id: Uuid, building_id: Uuid, status: &str) -> AppResult<()> {
    sqlx::query!(
        "UPDATE report SET status = $3::text::report_status, updated_at = NOW() WHERE id=$1 AND building_id=$2",
        id, building_id, status
    ).execute(pool).await?;
    Ok(())
}
