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

pub struct ActionDef {
    pub name: &'static str,
    pub sensitive: bool,
}

/// Closed vocabulary of `activity_log.action` values (TEXT column).
pub mod actions {
    pub struct ActionDef {
        pub name: &'static str,
        pub sensitive: bool,
    }

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

    /// Every action of the closed vocabulary. The `sensitive` flag is the
    /// single source of the visibility rule: an action added here must be
    /// classified in the same literal.
    pub const ALL: &[ActionDef] = &[
        ActionDef {
            name: CREATED,
            sensitive: false,
        },
        ActionDef {
            name: STATUS_CHANGED,
            sensitive: false,
        },
        ActionDef {
            name: PHOTO_ADDED,
            sensitive: false,
        },
        ActionDef {
            name: CONVERTED_TO_TASK,
            sensitive: false,
        },
        ActionDef {
            name: ASSIGNEE_CHANGED,
            sensitive: true,
        },
        ActionDef {
            name: CONTRACTOR_ASSIGNED,
            sensitive: false,
        },
        ActionDef {
            name: VENDOR_LINK_GENERATED,
            sensitive: true,
        },
        ActionDef {
            name: DOCUMENT_UPLOADED,
            sensitive: false,
        },
        ActionDef {
            name: VENDOR_SUBMISSION_RECEIVED,
            sensitive: false,
        },
        ActionDef {
            name: CLOSED,
            sensitive: false,
        },
    ];

    pub fn is_sensitive(action: &str) -> bool {
        ALL.iter().any(|def| def.name == action && def.sensitive)
    }
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct ActivityLogRow {
    pub id: Uuid,
    pub building_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub action: String,
    pub actor_id: Option<Uuid>,
    pub payload: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TimelineEventRow {
    pub id: Uuid,
    pub building_id: Uuid,
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
    building_id: Uuid,
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
        "INSERT INTO activity_log (building_id, entity_type, entity_id, action, actor_id, payload) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(building_id)
    .bind(entity_type)
    .bind(entity_id)
    .bind(action)
    .bind(actor_id)
    .bind(payload)
    .execute(executor)
    .await?;
    Ok(())
}

/// Emits a status transition: `closed` for task closure, `status_changed`
/// otherwise (report closure stays `status_changed`). Callers guard no-ops.
pub async fn emit_status_change<'e, E>(
    executor: E,
    building_id: Uuid,
    entity_type: &str,
    entity_id: Uuid,
    old_status: &str,
    new_status: &str,
    actor_id: Option<Uuid>,
) -> AppResult<()>
where
    E: sqlx::Executor<'e, Database = Postgres>,
{
    let (action, payload) = if entity_type == ENTITY_TASK && new_status == "closed" {
        (
            actions::CLOSED,
            serde_json::json!({ "old_status": old_status }),
        )
    } else {
        (
            actions::STATUS_CHANGED,
            serde_json::json!({ "old_status": old_status, "new_status": new_status }),
        )
    };
    insert(
        executor,
        building_id,
        entity_type,
        entity_id,
        action,
        actor_id,
        Some(payload),
    )
    .await
}

/// Post-commit variant: same event, logging failure only warned.
pub async fn emit_status_change_best_effort(
    pool: &PgPool,
    building_id: Uuid,
    entity_type: &str,
    entity_id: Uuid,
    old_status: &str,
    new_status: &str,
    actor_id: Option<Uuid>,
) {
    if let Err(e) = emit_status_change(
        pool,
        building_id,
        entity_type,
        entity_id,
        old_status,
        new_status,
        actor_id,
    )
    .await
    {
        tracing::warn!("activity_log emission failed: {e}");
    }
}

/// Events of ONE entity, chronological ascending (deterministic tie-break on id).
pub async fn list_for_entity(
    pool: &PgPool,
    building_id: Uuid,
    entity_type: &str,
    entity_id: Uuid,
) -> AppResult<Vec<ActivityLogRow>> {
    let rows = sqlx::query_as::<_, ActivityLogRow>(
        "SELECT id, building_id, entity_type, entity_id, action, actor_id, payload, created_at \
         FROM activity_log \
         WHERE building_id = $1 AND entity_type = $2 AND entity_id = $3 \
         ORDER BY created_at ASC, id ASC",
    )
    .bind(building_id)
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
    building_id: Uuid,
    report_id: Option<Uuid>,
    task_id: Option<Uuid>,
) -> AppResult<Vec<TimelineEventRow>> {
    let rows = sqlx::query_as::<_, TimelineEventRow>(
        r#"SELECT a.id, a.building_id, a.entity_type, a.entity_id, a.action, a.actor_id,
                  u.first_name || ' ' || u.last_name AS actor_name,
                  a.payload, a.created_at
           FROM activity_log a
           LEFT JOIN user_account u ON u.id = a.actor_id
           WHERE a.building_id = $1
             AND ((a.entity_type = 'report' AND a.entity_id = $2)
               OR (a.entity_type = 'task' AND a.entity_id = $3))
           ORDER BY a.created_at ASC, a.id ASC"#,
    )
    .bind(building_id)
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
    building_id: Uuid,
    entity_type: &str,
    entity_id: Uuid,
    action: &str,
    actor_id: Option<Uuid>,
    payload: Option<serde_json::Value>,
) {
    if let Err(e) = insert(
        pool,
        building_id,
        entity_type,
        entity_id,
        action,
        actor_id,
        payload,
    )
    .await
    {
        tracing::warn!("activity_log emission failed: {e}");
    }
}
