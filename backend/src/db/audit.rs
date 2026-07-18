//! Append-only audit log repository (ADR-009).
//! No UPDATE or DELETE — every row is a permanent accounting record.

use serde_json::Value;
use sqlx::{PgPool, Postgres};
use uuid::Uuid;

use crate::errors::AppResult;

#[derive(Debug)]
pub struct NewAuditEntry<'a> {
    pub building_id: Uuid,
    pub entity_type: &'a str,
    pub entity_id: Uuid,
    pub operation: &'a str, // "UPDATE" | "DELETE"
    pub changed_by: Option<Uuid>,
    pub old_values: Option<Value>,
    pub new_values: Option<Value>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct AuditEntry {
    pub id: Uuid,
    pub building_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub operation: String,
    pub changed_by: Option<Uuid>,
    pub changed_at: chrono::DateTime<chrono::Utc>,
    pub old_values: Option<Value>,
    pub new_values: Option<Value>,
}

/// Records one mutation event. Returns the new row's id.
///
/// Generic over `Executor` so callers inside an open transaction can pass
/// `&mut *tx` — the audit row must land atomically with the mutation it
/// documents (same commit/rollback boundary), never on a separate pool
/// connection.
pub async fn insert<'e, E>(executor: E, entry: NewAuditEntry<'_>) -> AppResult<Uuid>
where
    E: sqlx::Executor<'e, Database = Postgres>,
{
    let id = sqlx::query_scalar(
        r#"INSERT INTO audit_log
               (building_id, entity_type, entity_id, operation, changed_by, old_values, new_values)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id"#,
    )
    .bind(entry.building_id)
    .bind(entry.entity_type)
    .bind(entry.entity_id)
    .bind(entry.operation)
    .bind(entry.changed_by)
    .bind(entry.old_values)
    .bind(entry.new_values)
    .fetch_one(executor)
    .await?;

    Ok(id)
}

/// Lists all audit entries for a specific entity, newest first.
pub async fn list_by_entity(
    pool: &PgPool,
    building_id: Uuid,
    entity_type: &str,
    entity_id: Uuid,
) -> AppResult<Vec<AuditEntry>> {
    let rows = sqlx::query_as::<_, AuditEntry>(
        r#"SELECT id, building_id, entity_type, entity_id, operation,
                  changed_by, changed_at, old_values, new_values
           FROM audit_log
           WHERE building_id = $1 AND entity_type = $2 AND entity_id = $3
           ORDER BY changed_at DESC"#,
    )
    .bind(building_id)
    .bind(entity_type)
    .bind(entity_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
