use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::errors::AppResult;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DocumentRow {
    pub id: Uuid,
    pub building_id: Uuid,
    pub original_name: String,
    pub storage_path: String,
    pub bucket: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub uploaded_by: Option<Uuid>,
    pub extraction_status: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub async fn list(pool: &PgPool, building_id: Uuid, roles: &[String]) -> AppResult<Vec<DocumentRow>> {
    Ok(sqlx::query_as!(DocumentRow,
        r#"SELECT id, building_id, original_name, storage_path, bucket,
           mime_type, size_bytes, uploaded_by,
           extraction_status::text as extraction_status, created_at
           FROM document
           WHERE building_id = $1
           AND visible_to_roles::text[] && $2
           ORDER BY created_at DESC"#,
        building_id, roles as _
    ).fetch_all(pool).await?)
}

pub async fn create(
    pool: &PgPool,
    building_id: Uuid,
    user_id: Uuid,
    original_name: &str,
    storage_path: &str,
    bucket: &str,
    mime_type: &str,
    size_bytes: i64,
    visible_to_roles: &[String],
) -> AppResult<DocumentRow> {
    Ok(sqlx::query_as!(DocumentRow,
        r#"INSERT INTO document (building_id, uploaded_by, original_name, storage_path,
           bucket, mime_type, size_bytes, visible_to_roles)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8::user_role[])
           RETURNING id, building_id, original_name, storage_path, bucket,
           mime_type, size_bytes, uploaded_by,
           extraction_status::text as extraction_status, created_at"#,
        building_id, user_id, original_name, storage_path,
        bucket, mime_type, size_bytes, visible_to_roles as _
    ).fetch_one(pool).await?)
}

pub async fn get(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<Option<DocumentRow>> {
    Ok(sqlx::query_as!(DocumentRow,
        r#"SELECT id, building_id, original_name, storage_path, bucket,
           mime_type, size_bytes, uploaded_by,
           extraction_status::text as extraction_status, created_at
           FROM document WHERE id = $1 AND building_id = $2"#,
        id, building_id
    ).fetch_optional(pool).await?)
}

pub async fn delete(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<Option<(String, String)>> {
    let row = sqlx::query!(
        "DELETE FROM document WHERE id = $1 AND building_id = $2 RETURNING bucket, storage_path",
        id, building_id
    ).fetch_optional(pool).await?;
    Ok(row.map(|r| (r.bucket, r.storage_path)))
}
