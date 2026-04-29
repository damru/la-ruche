use crate::errors::AppResult;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

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
    pub visible_roles: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DocumentListRow {
    pub id: Uuid,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub visible_roles: Vec<String>,
    pub linked_contract_names: Vec<String>,
    pub linked_report_count: i64,
    pub created_at: DateTime<Utc>,
}

pub async fn list(
    pool: &PgPool,
    building_id: Uuid,
    roles: &[String],
) -> AppResult<Vec<DocumentListRow>> {
    Ok(sqlx::query_as::<_, DocumentListRow>(
        r#"SELECT d.id,
                  d.original_name,
                  d.mime_type,
                  d.size_bytes,
                  d.visible_to_roles::text[] AS visible_roles,
                  COALESCE(
                      array_remove(
                          array_agg(DISTINCT vc.vendor_name ORDER BY vc.vendor_name),
                          NULL
                      ),
                      ARRAY[]::text[]
                  ) AS linked_contract_names,
                  COUNT(DISTINCT rd.report_id) AS linked_report_count,
                  d.created_at
           FROM document d
           LEFT JOIN contract_document cd ON cd.document_id = d.id
           LEFT JOIN vendor_contract vc ON vc.id = cd.contract_id AND vc.building_id = $1
           LEFT JOIN report_document rd ON rd.document_id = d.id
           WHERE d.building_id = $1
              AND d.visible_to_roles::text[] && $2
           GROUP BY d.id, d.original_name, d.mime_type, d.size_bytes, d.visible_to_roles, d.created_at
           ORDER BY d.created_at DESC"#,
    )
    .bind(building_id)
    .bind(roles)
    .fetch_all(pool)
    .await?)
}

#[allow(clippy::too_many_arguments)]
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
    Ok(sqlx::query_as::<_, DocumentRow>(
        r#"INSERT INTO document (building_id, uploaded_by, original_name, storage_path,
           bucket, mime_type, size_bytes, visible_to_roles)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8::user_role[])
           RETURNING id, building_id, original_name, storage_path, bucket,
           mime_type, size_bytes, uploaded_by,
           extraction_status::text AS extraction_status,
           visible_to_roles::text[] AS visible_roles,
           created_at"#,
    )
    .bind(building_id)
    .bind(user_id)
    .bind(original_name)
    .bind(storage_path)
    .bind(bucket)
    .bind(mime_type)
    .bind(size_bytes)
    .bind(visible_to_roles)
    .fetch_one(pool)
    .await?)
}

/// Same as `create` but executes within an existing transaction.
#[allow(clippy::too_many_arguments)]
pub async fn create_tx(
    tx: &mut Transaction<'_, Postgres>,
    building_id: Uuid,
    user_id: Uuid,
    original_name: &str,
    storage_path: &str,
    bucket: &str,
    mime_type: &str,
    size_bytes: i64,
    visible_to_roles: &[String],
) -> AppResult<DocumentRow> {
    Ok(sqlx::query_as::<_, DocumentRow>(
        r#"INSERT INTO document (building_id, uploaded_by, original_name, storage_path,
           bucket, mime_type, size_bytes, visible_to_roles)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8::user_role[])
           RETURNING id, building_id, original_name, storage_path, bucket,
           mime_type, size_bytes, uploaded_by,
           extraction_status::text AS extraction_status,
           visible_to_roles::text[] AS visible_roles,
           created_at"#,
    )
    .bind(building_id)
    .bind(user_id)
    .bind(original_name)
    .bind(storage_path)
    .bind(bucket)
    .bind(mime_type)
    .bind(size_bytes)
    .bind(visible_to_roles)
    .fetch_one(&mut **tx)
    .await?)
}

pub async fn get(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<Option<DocumentRow>> {
    Ok(sqlx::query_as::<_, DocumentRow>(
        r#"SELECT id, building_id, original_name, storage_path, bucket,
           mime_type, size_bytes, uploaded_by,
           extraction_status::text AS extraction_status,
           visible_to_roles::text[] AS visible_roles,
           created_at
           FROM document WHERE id = $1 AND building_id = $2"#,
    )
    .bind(id)
    .bind(building_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn delete(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
) -> AppResult<Option<(String, String)>> {
    let row = sqlx::query_as::<_, (String, String)>(
        r#"
        WITH deleted_document AS (
            DELETE FROM document
            WHERE id = $1 AND building_id = $2
            RETURNING id, bucket, storage_path
        ),
        deleted_links AS (
            DELETE FROM contract_document
            WHERE document_id IN (SELECT id FROM deleted_document)
        )
        SELECT bucket, storage_path
        FROM deleted_document
        "#,
    )
    .bind(id)
    .bind(building_id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}
