use crate::errors::{AppError, AppResult};
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
    pub category: String,
    pub supersedes_id: Option<Uuid>,
    pub is_current: bool,
    pub replacement_reason: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
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
    pub linked_report_titles: Vec<String>,
    pub linked_claim_titles: Vec<String>,
    pub linked_report_count: i64,
    pub category: String,
    pub supersedes_id: Option<Uuid>,
    pub is_current: bool,
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
                   COALESCE(
                       array_remove(
                           array_agg(DISTINCT r.title ORDER BY r.title),
                           NULL
                       ),
                       ARRAY[]::text[]
                   ) AS linked_report_titles,
                   COALESCE(
                       array_remove(
                           array_agg(DISTINCT c.title ORDER BY c.title),
                           NULL
                       ),
                       ARRAY[]::text[]
                   ) AS linked_claim_titles,
                   COUNT(DISTINCT rd.report_id) AS linked_report_count,
                   d.category::text AS category,
                   d.supersedes_id,
                   d.is_current,
                   d.created_at
            FROM document d
            LEFT JOIN contract_document cd ON cd.document_id = d.id
            LEFT JOIN vendor_contract vc ON vc.id = cd.contract_id AND vc.building_id = $1
            LEFT JOIN report_document rd ON rd.document_id = d.id
            LEFT JOIN report r ON r.id = rd.report_id AND r.building_id = $1
            LEFT JOIN claim_document cld ON cld.document_id = d.id
            LEFT JOIN claim c ON c.id = cld.claim_id AND c.building_id = $1
            WHERE d.building_id = $1
               AND d.visible_to_roles::text[] && $2
               AND d.deleted_at IS NULL
            GROUP BY d.id, d.original_name, d.mime_type, d.size_bytes, d.visible_to_roles,
                     d.category, d.supersedes_id, d.is_current, d.created_at
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
    category: &str,
) -> AppResult<DocumentRow> {
    let mut tx = pool.begin().await?;
    let doc = sqlx::query_as::<_, DocumentRow>(
        r#"INSERT INTO document (building_id, uploaded_by, original_name, storage_path,
           bucket, mime_type, size_bytes, visible_to_roles, category)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8::user_role[], $9::text::document_category)
           RETURNING id, building_id, original_name, storage_path, bucket,
           mime_type, size_bytes, uploaded_by,
           extraction_status::text AS extraction_status,
           visible_to_roles::text[] AS visible_roles,
           category::text AS category, supersedes_id, is_current,
           replacement_reason, deleted_at,
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
    .bind(category)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query!(
        r#"INSERT INTO document_event (building_id, document_id, event_type, actor_id, payload)
           VALUES ($1, $2, 'uploaded', $3, $4)"#,
        building_id,
        doc.id,
        user_id,
        serde_json::json!({
            "original_name": doc.original_name,
            "category": doc.category,
            "visible_roles": doc.visible_roles,
        }),
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(doc)
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
           category::text AS category, supersedes_id, is_current,
           replacement_reason, deleted_at,
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
           category::text AS category, supersedes_id, is_current,
           replacement_reason, deleted_at,
           created_at
           FROM document WHERE id = $1 AND building_id = $2 AND deleted_at IS NULL"#,
    )
    .bind(id)
    .bind(building_id)
    .fetch_optional(pool)
    .await?)
}

/// Publishes a new version of a founding document, chained to the one it
/// replaces. Fails if the predecessor is not the current version, or if the
/// category doesn't match (EP-005-S06, LEGAL §1.2).
#[allow(clippy::too_many_arguments)]
pub async fn create_version(
    pool: &PgPool,
    building_id: Uuid,
    user_id: Uuid,
    original_name: &str,
    storage_path: &str,
    bucket: &str,
    mime_type: &str,
    size_bytes: i64,
    visible_to_roles: &[String],
    category: &str,
    supersedes_id: Uuid,
    reason: Option<&str>,
) -> AppResult<DocumentRow> {
    let mut tx = pool.begin().await?;

    let predecessor = sqlx::query_as::<_, (String, bool)>(
        r#"SELECT category::text, is_current FROM document
           WHERE id = $1 AND building_id = $2 FOR UPDATE"#,
    )
    .bind(supersedes_id)
    .bind(building_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::NotFound)?;

    if predecessor.0 != category {
        return Err(AppError::unprocessable_entity_code(
            "errors.documents.category_mismatch",
        ));
    }
    if !predecessor.1 {
        return Err(AppError::unprocessable_entity_code(
            "errors.documents.not_current_version",
        ));
    }
    if crate::domain::immutability::document_requires_replacement_reason(category)
        && reason.unwrap_or("").trim().is_empty()
    {
        return Err(AppError::unprocessable_entity_code(
            "errors.documents.replacement_reason_required",
        ));
    }

    sqlx::query("UPDATE document SET is_current = FALSE WHERE id = $1")
        .bind(supersedes_id)
        .execute(&mut *tx)
        .await?;

    let row = sqlx::query_as::<_, DocumentRow>(
        r#"INSERT INTO document (building_id, uploaded_by, original_name, storage_path,
           bucket, mime_type, size_bytes, visible_to_roles, category, supersedes_id, is_current,
           replacement_reason)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8::user_role[], $9::text::document_category, $10, TRUE, $11)
           RETURNING id, building_id, original_name, storage_path, bucket,
           mime_type, size_bytes, uploaded_by,
           extraction_status::text AS extraction_status,
           visible_to_roles::text[] AS visible_roles,
           category::text AS category, supersedes_id, is_current,
           replacement_reason, deleted_at,
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
    .bind(category)
    .bind(supersedes_id)
    .bind(reason)
    .fetch_one(&mut *tx)
    .await?;

    // Emit 'replaced' event for the new version
    sqlx::query!(
        r#"INSERT INTO document_event (building_id, document_id, event_type, actor_id, payload)
           VALUES ($1, $2, 'replaced', $3, $4)"#,
        building_id,
        row.id,
        user_id,
        serde_json::json!({
            "original_name": row.original_name,
            "category": row.category,
            "visible_roles": row.visible_roles,
            "supersedes_id": row.supersedes_id,
            "replacement_reason": row.replacement_reason,
        }),
    )
    .execute(&mut *tx)
    .await?;

    crate::db::audit::insert(
        &mut *tx,
        crate::db::audit::NewAuditEntry {
            building_id,
            entity_type: "document",
            entity_id: row.id,
            operation: "UPDATE",
            changed_by: Some(user_id),
            old_values: Some(serde_json::json!({ "id": supersedes_id })),
            new_values: Some(serde_json::json!({ "id": row.id, "reason": reason })),
        },
    )
    .await?;

    tx.commit().await?;
    Ok(row)
}

/// Returns the full version chain a document belongs to (ascending +
/// descending via `supersedes_id`), ordered chronologically — identical
/// whichever link of the chain is queried.
pub async fn version_chain(
    pool: &PgPool,
    document_id: Uuid,
    building_id: Uuid,
) -> AppResult<Vec<DocumentRow>> {
    Ok(sqlx::query_as::<_, DocumentRow>(
        r#"
        WITH RECURSIVE ancestors AS (
            SELECT id, supersedes_id FROM document WHERE id = $1 AND building_id = $2
            UNION
            SELECT d.id, d.supersedes_id
            FROM document d
            JOIN ancestors a ON d.id = a.supersedes_id
        ),
        descendants AS (
            SELECT id FROM document WHERE id = $1 AND building_id = $2
            UNION
            SELECT d.id
            FROM document d
            JOIN descendants dsc ON d.supersedes_id = dsc.id
        ),
        chain AS (
            SELECT id FROM ancestors
            UNION
            SELECT id FROM descendants
        )
        SELECT d.id, d.building_id, d.original_name, d.storage_path, d.bucket,
               d.mime_type, d.size_bytes, d.uploaded_by,
               d.extraction_status::text AS extraction_status,
               d.visible_to_roles::text[] AS visible_roles,
               d.category::text AS category, d.supersedes_id, d.is_current,
               d.replacement_reason, d.deleted_at,
               d.created_at
        FROM document d
        JOIN chain c ON c.id = d.id
        WHERE d.building_id = $2
          AND d.deleted_at IS NULL
        ORDER BY d.created_at
        "#,
    )
    .bind(document_id)
    .bind(building_id)
    .fetch_all(pool)
    .await?)
}

/// Soft-deletes a non-founding document (US-DOC-004): the row and the S3
/// object are kept, only `deleted_at`/`deleted_by`/`deletion_reason` are set.
/// Founding categories (`reglement_copropriete`, `edd`) must go through
/// `create_version` with a reason instead — see
/// `document_requires_replacement_reason`.
pub async fn delete(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    deleted_by: Uuid,
    deletion_reason: Option<&str>,
) -> AppResult<Option<()>> {
    let doc = sqlx::query_scalar::<_, String>(
        "SELECT category::text
         FROM document WHERE id = $1 AND building_id = $2 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(building_id)
    .fetch_optional(pool)
    .await?;

    let Some(category) = doc else {
        return Ok(None);
    };

    if crate::domain::immutability::document_requires_replacement_reason(&category) {
        return Err(AppError::unprocessable_entity_code(
            "errors.documents.replacement_required",
        ));
    }

    // A document with a successor must be preserved to keep the chain intact.
    let has_successor = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM document WHERE supersedes_id = $1)",
    )
    .bind(id)
    .fetch_one(pool)
    .await?;
    if has_successor {
        return Err(AppError::unprocessable_entity_code(
            "errors.documents.founding_locked",
        ));
    }

    let updated = sqlx::query(
        r#"
        WITH deleted_document AS (
            UPDATE document
            SET deleted_at = now(), deleted_by = $3, deletion_reason = $4
            WHERE id = $1 AND building_id = $2 AND deleted_at IS NULL
            RETURNING id
        ),
        cleared_meeting_minutes AS (
            UPDATE general_meeting
            SET minutes_document_id = NULL
            WHERE minutes_document_id IN (SELECT id FROM deleted_document)
        )
        SELECT id FROM deleted_document
        "#,
    )
    .bind(id)
    .bind(building_id)
    .bind(deleted_by)
    .bind(deletion_reason)
    .execute(pool)
    .await?;

    Ok((updated.rows_affected() > 0).then_some(()))
}

/// Soft-deletes documents with no remaining references (called from
/// cascading deletes of a contract/task/claim/report). Founding categories
/// are excluded — they can never be orphan-deleted, only replaced.
pub async fn delete_orphaned_tx(
    tx: &mut Transaction<'_, Postgres>,
    document_ids: &[Uuid],
    building_id: Uuid,
) -> AppResult<()> {
    if document_ids.is_empty() {
        return Ok(());
    }

    sqlx::query(
        r#"
        UPDATE document d
        SET deleted_at = now()
        WHERE d.id = ANY($1)
          AND d.building_id = $2
          AND d.deleted_at IS NULL
          AND d.category NOT IN ('reglement_copropriete', 'edd')
          AND NOT EXISTS (
              SELECT 1 FROM document successor WHERE successor.supersedes_id = d.id
          )
          AND NOT EXISTS (
              SELECT 1 FROM contract_document cd WHERE cd.document_id = d.id
          )
          AND NOT EXISTS (
              SELECT 1 FROM general_meeting_document gmd WHERE gmd.document_id = d.id
          )
          AND NOT EXISTS (
              SELECT 1 FROM general_meeting gm WHERE gm.minutes_document_id = d.id
          )
          AND NOT EXISTS (
              SELECT 1 FROM report_document rd WHERE rd.document_id = d.id
          )
          AND NOT EXISTS (
              SELECT 1 FROM claim_document cld WHERE cld.document_id = d.id
          )
          AND NOT EXISTS (
              SELECT 1 FROM task_document td WHERE td.document_id = d.id
          )
          AND NOT EXISTS (
              SELECT 1 FROM maintenance_log_document mld WHERE mld.document_id = d.id
          )
          AND NOT EXISTS (
              SELECT 1 FROM vendor_submission_document vsd WHERE vsd.document_id = d.id
          )
        "#,
    )
    .bind(document_ids)
    .bind(building_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

// End of module.
