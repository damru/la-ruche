//! Business activity log repository. Append-only: insert + read only.
//! Distinct from `audit_log` (accounting-grade trail): this table feeds the
//! user-facing report/task lifecycle timelines.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, Postgres};
use uuid::Uuid;

use crate::errors::AppResult;

/// `activity_log.entity_type` values (TEXT column).
pub const ENTITY_REPORT: &str = "report";
pub const ENTITY_TASK: &str = "task";

/// Closed vocabulary of `activity_log.action` values (TEXT column).
pub mod actions {
    pub const CREATED: &str = "created";
    pub const STATUS_CHANGED: &str = "status_changed";
    pub const PHOTO_ADDED: &str = "photo_added";
    pub const CONVERTED_TO_TASK: &str = "converted_to_task";
    pub const ASSIGNEE_CHANGED: &str = "assignee_changed";
    pub const CONTRACTOR_ASSIGNED: &str = "contractor_assigned";
    pub const VENDOR_LINK_GENERATED: &str = "vendor_link_generated";
    pub const DOCUMENT_UPLOADED: &str = "document_uploaded";
    pub const VENDOR_SUBMISSION_RECEIVED: &str = "vendor_submission_received";
    pub const CLOSED: &str = "closed";
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct ActivityLogRow {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub action: String,
    pub actor_id: Option<Uuid>,
    pub payload: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// One timeline event with the actor's display name resolved.
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TimelineEventRow {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub action: String,
    pub actor_id: Option<Uuid>,
    pub actor_name: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Generic over the executor so callers can join an open transaction.
pub async fn insert<'e, E>(
    executor: E,
    entity_type: &str,
    entity_id: Uuid,
    action: &str,
    actor_id: Option<Uuid>,
    payload: Option<serde_json::Value>,
) -> AppResult<()>
where
    E: sqlx::Executor<'e, Database = Postgres>,
{
    sqlx::query(
        "INSERT INTO activity_log (entity_type, entity_id, action, actor_id, payload) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(entity_type)
    .bind(entity_id)
    .bind(action)
    .bind(actor_id)
    .bind(payload)
    .execute(executor)
    .await?;
    Ok(())
}

/// Events of ONE entity, chronological ascending (deterministic tie-break on id).
pub async fn list_for_entity(
    pool: &PgPool,
    entity_type: &str,
    entity_id: Uuid,
) -> AppResult<Vec<ActivityLogRow>> {
    let rows = sqlx::query_as::<_, ActivityLogRow>(
        "SELECT id, entity_type, entity_id, action, actor_id, payload, created_at \
         FROM activity_log \
         WHERE entity_type = $1 AND entity_id = $2 \
         ORDER BY created_at ASC, id ASC",
    )
    .bind(entity_type)
    .bind(entity_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Merged events for a (report?, task?) chain, chronological ascending.
/// `actor_name` comes from user_account (NULL for anonymous vendor submissions).
pub async fn list_for_chain(
    pool: &PgPool,
    report_id: Option<Uuid>,
    task_id: Option<Uuid>,
) -> AppResult<Vec<TimelineEventRow>> {
    let rows = sqlx::query_as::<_, TimelineEventRow>(
        r#"SELECT a.id, a.entity_type, a.entity_id, a.action, a.actor_id,
                  u.first_name || ' ' || u.last_name AS actor_name,
                  a.payload, a.created_at
           FROM activity_log a
           LEFT JOIN user_account u ON u.id = a.actor_id
           WHERE (a.entity_type = 'report' AND a.entity_id = $1)
              OR (a.entity_type = 'task' AND a.entity_id = $2)
           ORDER BY a.created_at ASC, a.id ASC"#,
    )
    .bind(report_id)
    .bind(task_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Post-commit emission: a logging failure must never fail the mutation that
/// already succeeded. The timeline is a UX feed, not an evidentiary trail.
pub async fn emit_best_effort(
    pool: &PgPool,
    entity_type: &str,
    entity_id: Uuid,
    action: &str,
    actor_id: Option<Uuid>,
    payload: Option<serde_json::Value>,
) {
    if let Err(e) = insert(pool, entity_type, entity_id, action, actor_id, payload).await {
        tracing::warn!("activity_log emission failed: {e}");
    }
}
