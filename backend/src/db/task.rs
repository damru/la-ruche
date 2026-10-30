use crate::errors::AppResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, types::Decimal};
use uuid::Uuid;

/// Full internal row — keeps vendor_token for token generation endpoints.
/// Never serialized directly; use `TaskListItem` for API responses.
#[derive(Debug, sqlx::FromRow)]
pub struct TaskRow {
    pub id: Uuid,
    pub building_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub contract_id: Option<Uuid>,
    pub contractor_id: Option<Uuid>,
    pub contractor_name: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub created_by: Uuid,
    /// Raw token UUID — capability secret, never exposed in list/get responses.
    pub vendor_token: Option<Uuid>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub completion_cost: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub document_count: i64,
    /// True when at least one maintenance_log entry references this task.
    pub is_in_logbook: bool,
}

/// Safe DTO for list/get API responses.
/// Exposes token metadata (expiry, presence) without leaking the raw token UUID.
#[derive(Debug, Serialize)]
pub struct TaskListItem {
    pub id: Uuid,
    pub building_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub contract_id: Option<Uuid>,
    pub contractor_id: Option<Uuid>,
    pub contractor_name: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub created_by: Uuid,
    pub has_vendor_token: bool,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub completion_cost: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub document_count: i64,
    /// True when at least one maintenance_log entry references this task.
    pub is_in_logbook: bool,
}

impl From<TaskRow> for TaskListItem {
    fn from(r: TaskRow) -> Self {
        Self {
            has_vendor_token: r.vendor_token.is_some(),
            id: r.id,
            building_id: r.building_id,
            title: r.title,
            description: r.description,
            status: r.status,
            priority: r.priority,
            contract_id: r.contract_id,
            contractor_id: r.contractor_id,
            contractor_name: r.contractor_name,
            assigned_to: r.assigned_to,
            created_by: r.created_by,
            token_expires_at: r.token_expires_at,
            closed_at: r.closed_at,
            completion_cost: r.completion_cost,
            created_at: r.created_at,
            updated_at: r.updated_at,
            document_count: r.document_count,
            is_in_logbook: r.is_in_logbook,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskInput {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub contract_id: Option<Uuid>,
    pub contractor_id: Option<Uuid>,
    pub contractor_name: Option<String>,
    pub assigned_to: Option<Uuid>,
}

/// Trims optional text input and converts blank values to `None` so nullable
/// text columns are stored consistently without whitespace-only content.
fn normalize_optional_text(value: Option<&str>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

/// `task_status` values (PG enum `task_status`).
pub mod status {
    pub const OPEN: &str = "open";
    pub const IN_PROGRESS: &str = "in_progress";
    pub const AWAITING_VENDOR: &str = "awaiting_vendor";
    pub const AWAITING_VALIDATION: &str = "awaiting_validation";
    pub const CLOSED: &str = "closed";
    pub const CANCELLED: &str = "cancelled";
}

/// `task_priority` values (PG enum `task_priority`).
pub mod priority {
    pub const LOW: &str = "low";
    pub const NORMAL: &str = "normal";
    pub const HIGH: &str = "high";
    pub const URGENT: &str = "urgent";
}

/// Valid task status values for update requests.
const VALID_STATUSES: &[&str] = &[
    status::OPEN,
    status::IN_PROGRESS,
    status::AWAITING_VENDOR,
    status::AWAITING_VALIDATION,
    status::CLOSED,
    status::CANCELLED,
];

pub const DEFAULT_PRIORITY: &str = priority::NORMAL;
const VALID_PRIORITIES: &[&str] = &[
    priority::LOW,
    priority::NORMAL,
    priority::HIGH,
    priority::URGENT,
];

pub fn validate_status(status: &str) -> bool {
    VALID_STATUSES.contains(&status)
}

pub fn validate_priority(priority: &str) -> bool {
    VALID_PRIORITIES.contains(&priority)
}

fn task_row_from_query(r: &sqlx::postgres::PgRow) -> Result<TaskRow, sqlx::Error> {
    use sqlx::Row;
    Ok(TaskRow {
        id: r.try_get("id")?,
        building_id: r.try_get("building_id")?,
        title: r.try_get("title")?,
        description: r.try_get("description")?,
        status: r.try_get("status")?,
        priority: r.try_get("priority")?,
        contract_id: r.try_get("contract_id")?,
        contractor_id: r.try_get("contractor_id")?,
        contractor_name: r.try_get("contractor_name")?,
        assigned_to: r.try_get("assigned_to")?,
        created_by: r.try_get("created_by")?,
        vendor_token: r.try_get("vendor_token")?,
        token_expires_at: r.try_get("token_expires_at")?,
        closed_at: r.try_get("closed_at")?,
        completion_cost: r.try_get("completion_cost")?,
        created_at: r.try_get("created_at")?,
        updated_at: r.try_get("updated_at")?,
        document_count: r.try_get("document_count")?,
        is_in_logbook: r.try_get("is_in_logbook")?,
    })
}

/// Optional server-side filters for the task list. `None` means "no filter".
#[derive(Debug, Default)]
pub struct TaskListFilters {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub contractor_id: Option<Uuid>,
    pub q: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

/// Escapes ILIKE metacharacters so a user-provided search term is matched literally.
fn escape_ilike(term: &str) -> String {
    term.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// Per-status task counts for a building (tab counters), ignoring other filters.
pub async fn status_counts(pool: &PgPool, building_id: Uuid) -> AppResult<Vec<(String, i64)>> {
    let rows = sqlx::query(
        "SELECT t.status::text AS status, COUNT(*) AS count
           FROM task t
           WHERE t.building_id = $1
           GROUP BY t.status::text",
    )
    .bind(building_id)
    .fetch_all(pool)
    .await?;
    use sqlx::Row;
    rows.iter()
        .map(|row| {
            Ok((
                row.try_get::<String, _>("status")?,
                row.try_get::<i64, _>("count")?,
            ))
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(Into::into)
}

/// Lists tasks for a building with optional filters and pagination.
/// Returns the page of rows plus the total number of matching rows.
pub async fn list(
    pool: &PgPool,
    building_id: Uuid,
    filters: &TaskListFilters,
) -> AppResult<(Vec<TaskRow>, i64)> {
    let search = filters
        .q
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(escape_ilike);
    let rows = sqlx::query(
        r#"SELECT t.id, t.building_id, t.title, t.description,
           t.status::text AS status, t.priority::text AS priority,
           t.contract_id, t.contractor_id, t.contractor_name, t.assigned_to, t.created_by, t.vendor_token,
           t.token_expires_at, t.closed_at, t.completion_cost, t.created_at, t.updated_at,
            (
              SELECT COUNT(*)
              FROM (
                SELECT td.document_id
                FROM task_document td
                WHERE td.task_id = t.id
                UNION
                SELECT vsd.document_id
                FROM vendor_submission_document vsd
                JOIN vendor_submission vs ON vs.id = vsd.submission_id
                WHERE vs.task_id = t.id
              ) AS task_documents
              JOIN document d ON d.id = task_documents.document_id AND d.deleted_at IS NULL
            ) AS document_count,
           EXISTS(SELECT 1 FROM maintenance_log WHERE task_id = t.id) AS is_in_logbook,
           COUNT(*) OVER () AS total_count
           FROM task t
           WHERE t.building_id = $1
             AND ($2::text IS NULL OR t.status::text = $2)
             AND ($3::text IS NULL OR t.priority::text = $3)
             AND ($4::uuid IS NULL OR t.assigned_to = $4)
             AND ($5::uuid IS NULL OR t.contractor_id = $5)
             AND ($6::text IS NULL OR t.title ILIKE '%' || $6 || '%' ESCAPE '\' OR t.description ILIKE '%' || $6 || '%' ESCAPE '\')
           ORDER BY t.updated_at DESC
           LIMIT $7 OFFSET $8"#,
    )
    .bind(building_id)
    .bind(filters.status.as_deref())
    .bind(filters.priority.as_deref())
    .bind(filters.assigned_to)
    .bind(filters.contractor_id)
    .bind(search.as_deref())
    .bind(filters.limit)
    .bind(filters.offset)
    .fetch_all(pool)
    .await?;
    let total = rows
        .first()
        .and_then(|row| {
            use sqlx::Row;
            row.try_get::<i64, _>("total_count").ok()
        })
        .unwrap_or(0);
    let items = rows
        .iter()
        .map(task_row_from_query)
        .collect::<Result<Vec<_>, _>>()?;
    Ok((items, total))
}

pub async fn get(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<Option<TaskRow>> {
    let row = sqlx::query(
        r#"SELECT t.id, t.building_id, t.title, t.description,
           t.status::text AS status, t.priority::text AS priority,
           t.contract_id, t.contractor_id, t.contractor_name, t.assigned_to, t.created_by, t.vendor_token,
           t.token_expires_at, t.closed_at, t.completion_cost, t.created_at, t.updated_at,
            (
              SELECT COUNT(*)
              FROM (
                SELECT td.document_id
                FROM task_document td
                WHERE td.task_id = t.id
                UNION
                SELECT vsd.document_id
                FROM vendor_submission_document vsd
                JOIN vendor_submission vs ON vs.id = vsd.submission_id
                WHERE vs.task_id = t.id
              ) AS task_documents
              JOIN document d ON d.id = task_documents.document_id AND d.deleted_at IS NULL
            ) AS document_count,
           EXISTS(SELECT 1 FROM maintenance_log WHERE task_id = t.id) AS is_in_logbook
           FROM task t WHERE t.id = $1 AND t.building_id = $2"#,
    )
    .bind(id)
    .bind(building_id)
    .fetch_optional(pool)
    .await?;
    row.as_ref()
        .map(task_row_from_query)
        .transpose()
        .map_err(Into::into)
}

pub async fn create(
    pool: &PgPool,
    building_id: Uuid,
    user_id: Uuid,
    input: CreateTaskInput,
) -> AppResult<TaskRow> {
    let priority = input.priority.as_deref().unwrap_or(DEFAULT_PRIORITY);
    let contractor_name = normalize_optional_text(input.contractor_name.as_deref());
    let row = sqlx::query(
        r#"INSERT INTO task (building_id, title, description, priority, contract_id, contractor_id, contractor_name, assigned_to, created_by)
           VALUES ($1, $2, $3, $4::text::task_priority, $5, $6, $7, $8, $9)
           RETURNING id, building_id, title, description,
           status::text AS status, priority::text AS priority,
           contract_id, contractor_id, contractor_name, assigned_to, created_by, vendor_token,
           token_expires_at, closed_at, completion_cost, created_at, updated_at,
           0::bigint AS document_count,
           FALSE AS is_in_logbook"#,
    )
    .bind(building_id)
    .bind(&input.title)
    .bind(&input.description)
    .bind(priority)
    .bind(input.contract_id)
    .bind(input.contractor_id)
    .bind(contractor_name)
    .bind(input.assigned_to)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    task_row_from_query(&row).map_err(Into::into)
}

/// Returns an error if the task was not found (0 rows affected).
pub async fn update_status(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    status: &str,
) -> AppResult<()> {
    let closed_at = if status == status::CLOSED {
        Some(Utc::now())
    } else {
        None
    };
    let result = sqlx::query(
        "UPDATE task SET status = $3::text::task_status, closed_at = $4, updated_at = NOW()
         WHERE id = $1 AND building_id = $2",
    )
    .bind(id)
    .bind(building_id)
    .bind(status)
    .bind(closed_at)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(crate::errors::AppError::NotFound);
    }
    Ok(())
}

/// Returns `(token_uuid, expires_at)`. Returns 404 if the task doesn't exist.
pub async fn generate_token(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    ttl_days: u64,
) -> AppResult<(Uuid, DateTime<Utc>)> {
    let token = Uuid::new_v4();
    let expires_at = Utc::now() + chrono::Duration::days(ttl_days as i64);

    let result = sqlx::query(
        "UPDATE task SET vendor_token = $3, token_expires_at = $4,
         status = $5::text::task_status, updated_at = NOW()
         WHERE id = $1 AND building_id = $2",
    )
    .bind(id)
    .bind(building_id)
    .bind(token)
    .bind(expires_at)
    .bind(status::AWAITING_VENDOR)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(crate::errors::AppError::NotFound);
    }
    Ok((token, expires_at))
}

/// Returns the raw vendor_token UUID and its expiry for an active token, or `None` if absent.
pub async fn get_vendor_token(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
) -> AppResult<Option<(Uuid, DateTime<Utc>)>> {
    use sqlx::Row;
    let row = sqlx::query(
        "SELECT vendor_token, token_expires_at FROM task WHERE id = $1 AND building_id = $2",
    )
    .bind(id)
    .bind(building_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(|r| {
        let token: Option<Uuid> = r.try_get("vendor_token").ok().flatten();
        let expires_at: Option<DateTime<Utc>> = r.try_get("token_expires_at").ok().flatten();
        token.zip(expires_at)
    }))
}

/// Deletes a task; documents that become orphaned are soft-deleted.
pub async fn delete(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<()> {
    let mut tx = pool.begin().await?;

    let linked_document_ids = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT document_id
        FROM (
            SELECT td.document_id
            FROM task_document td
            WHERE td.task_id = $1
            UNION
            SELECT vsd.document_id
            FROM vendor_submission_document vsd
            JOIN vendor_submission vs ON vs.id = vsd.submission_id
            JOIN task t ON t.id = vs.task_id
            WHERE t.id = $1 AND t.building_id = $2
        ) AS linked_documents
        "#,
    )
    .bind(id)
    .bind(building_id)
    .fetch_all(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        DELETE FROM vendor_submission
        WHERE task_id = $1
          AND EXISTS (
              SELECT 1
              FROM task t
              WHERE t.id = $1
                AND t.building_id = $2
          )
        "#,
    )
    .bind(id)
    .bind(building_id)
    .execute(&mut *tx)
    .await?;

    let result = sqlx::query("DELETE FROM task WHERE id = $1 AND building_id = $2")
        .bind(id)
        .bind(building_id)
        .execute(&mut *tx)
        .await?;
    if result.rows_affected() == 0 {
        return Err(crate::errors::AppError::NotFound);
    }

    crate::db::document::delete_orphaned_tx(&mut tx, &linked_document_ids, building_id).await?;
    tx.commit().await?;
    Ok(())
}

/// Updates mutable task fields. `None` values leave the existing field unchanged.
/// `completion_cost_update` uses patch semantics:
/// - `None` keeps the current value
/// - `Some(None)` clears the value
/// - `Some(Some(x))` replaces the value
pub async fn update_fields(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    title: Option<&str>,
    description: Option<Option<&str>>,
    priority: Option<&str>,
    status: Option<&str>,
    contractor_id: Option<Option<Uuid>>,
    contractor_name: Option<Option<&str>>,
    completion_cost_update: Option<Option<Decimal>>,
    actor_user_id: Uuid,
) -> AppResult<()> {
    let current = get(pool, id, building_id)
        .await?
        .ok_or(crate::errors::AppError::NotFound)?;

    let current_status = current.status.clone();
    let current_completion_cost = current.completion_cost;

    let new_title = title
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| current.title.clone());
    let new_description = match description {
        Some(d) => d.map(|s| s.to_string()),
        None => current.description.clone(),
    };
    let new_priority = priority
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| current.priority.clone());
    let new_status = status
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| current.status.clone());
    let new_contractor_id = match contractor_id {
        Some(c) => c,
        None => current.contractor_id,
    };
    let new_contractor_name = match contractor_name {
        Some(value) => normalize_optional_text(value),
        None => current.contractor_name.clone(),
    };
    let new_completion_cost = match completion_cost_update {
        Some(cost) => cost,
        None => current_completion_cost,
    };
    let closed_at = if new_status == status::CLOSED && current.closed_at.is_none() {
        Some(Utc::now())
    } else if new_status != status::CLOSED {
        None
    } else {
        current.closed_at
    };
    let create_log_entry = current_status != status::CLOSED && new_status == status::CLOSED;

    let mut tx = pool.begin().await?;
    let result = sqlx::query(
        "UPDATE task SET title = $3, description = $4,
          priority = $5::text::task_priority,
          status = $6::text::task_status,
          contractor_id = $7,
          contractor_name = $8,
          closed_at = $9,
          completion_cost = $10,
          updated_at = NOW()
          WHERE id = $1 AND building_id = $2",
    )
    .bind(id)
    .bind(building_id)
    .bind(&new_title)
    .bind(&new_description)
    .bind(&new_priority)
    .bind(&new_status)
    .bind(new_contractor_id)
    .bind(&new_contractor_name)
    .bind(closed_at)
    .bind(new_completion_cost)
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() == 0 {
        return Err(crate::errors::AppError::NotFound);
    }
    if create_log_entry {
        crate::db::maintenance_log::create_automatic_from_task(
            &mut *tx,
            id,
            building_id,
            actor_user_id,
        )
        .await?;
        crate::db::report::close_linked_to_task(&mut *tx, id, building_id).await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Returns 404 if the task doesn't exist.
pub async fn revoke_token(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<()> {
    // Also reset status to 'open' when the task was awaiting a vendor, so the
    // manager can decide to take a different path (self-handle or re-delegate).
    let result = sqlx::query(
        "UPDATE task
         SET vendor_token = NULL,
             token_expires_at = NULL,
             status = CASE WHEN status = $3::text::task_status THEN $4::text::task_status ELSE status END,
             updated_at = NOW()
         WHERE id = $1 AND building_id = $2",
    )
    .bind(id)
    .bind(building_id)
    .bind(status::AWAITING_VENDOR)
    .bind(status::OPEN)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(crate::errors::AppError::NotFound);
    }
    Ok(())
}

/// Looks up a task by its vendor token. Only valid (non-expired, non-closed) tokens match.
pub async fn get_by_token(pool: &PgPool, token: Uuid) -> AppResult<Option<TaskRow>> {
    let row = sqlx::query(
        r#"SELECT t.id, t.building_id, t.title, t.description,
           t.status::text AS status, t.priority::text AS priority,
           t.contract_id, t.contractor_id, t.contractor_name, t.assigned_to, t.created_by, t.vendor_token,
           t.token_expires_at, t.closed_at, t.completion_cost, t.created_at, t.updated_at,
            (
              SELECT COUNT(*)
              FROM (
                SELECT td.document_id
                FROM task_document td
                WHERE td.task_id = t.id
                UNION
                SELECT vsd.document_id
                FROM vendor_submission_document vsd
                JOIN vendor_submission vs ON vs.id = vsd.submission_id
                WHERE vs.task_id = t.id
              ) AS task_documents
              JOIN document d ON d.id = task_documents.document_id AND d.deleted_at IS NULL
            ) AS document_count,
           EXISTS(SELECT 1 FROM maintenance_log WHERE task_id = t.id) AS is_in_logbook
           FROM task t
           WHERE t.vendor_token = $1 AND t.token_expires_at > NOW()
           AND t.status != $2::text::task_status"#,
    )
    .bind(token)
    .bind(status::CLOSED)
    .fetch_optional(pool)
    .await?;
    row.as_ref()
        .map(task_row_from_query)
        .transpose()
        .map_err(Into::into)
}

pub async fn list_documents(
    pool: &PgPool,
    task_id: Uuid,
    building_id: Uuid,
) -> AppResult<Vec<serde_json::Value>> {
    use sqlx::Row;

    let rows = sqlx::query(
        r#"SELECT DISTINCT d.id, d.original_name, d.mime_type, d.size_bytes, d.created_at
           FROM document d
           WHERE d.building_id = $2
             AND d.deleted_at IS NULL
             AND d.id IN (
               SELECT td.document_id
               FROM task_document td
               WHERE td.task_id = $1
               UNION
               SELECT vsd.document_id
               FROM vendor_submission_document vsd
               JOIN vendor_submission vs ON vs.id = vsd.submission_id
               WHERE vs.task_id = $1
             )
           ORDER BY d.created_at ASC"#,
    )
    .bind(task_id)
    .bind(building_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.try_get::<Uuid, _>("id").unwrap_or_default(),
                "original_name": row.try_get::<String, _>("original_name").unwrap_or_default(),
                "mime_type": row.try_get::<String, _>("mime_type").unwrap_or_default(),
                "size_bytes": row.try_get::<i64, _>("size_bytes").unwrap_or_default(),
                "created_at": row.try_get::<DateTime<Utc>, _>("created_at").unwrap_or_default(),
            })
        })
        .collect())
}

pub async fn attach_document(
    pool: &PgPool,
    task_id: Uuid,
    document_id: Uuid,
    building_id: Uuid,
) -> AppResult<bool> {
    let pair_exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM task t
            WHERE t.id = $1
              AND t.building_id = $3
        ) AND EXISTS(
            SELECT 1
            FROM document d
            WHERE d.id = $2
              AND d.building_id = $3
        )
        "#,
    )
    .bind(task_id)
    .bind(document_id)
    .bind(building_id)
    .fetch_one(pool)
    .await?;

    if !pair_exists {
        return Ok(false);
    }

    sqlx::query(
        r#"
        INSERT INTO task_document (task_id, document_id)
        VALUES ($1, $2)
        ON CONFLICT (task_id, document_id) DO NOTHING
        "#,
    )
    .bind(task_id)
    .bind(document_id)
    .execute(pool)
    .await?;

    Ok(true)
}

pub async fn detach_document(
    pool: &PgPool,
    task_id: Uuid,
    document_id: Uuid,
    building_id: Uuid,
) -> AppResult<bool> {
    let result = sqlx::query(
        r#"
        DELETE FROM task_document td
        USING task t
        WHERE td.task_id = $1
          AND td.document_id = $2
          AND t.id = td.task_id
          AND t.building_id = $3
        "#,
    )
    .bind(task_id)
    .bind(document_id)
    .bind(building_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn create_submission(
    pool: &PgPool,
    task_id: Uuid,
    comment: Option<String>,
) -> AppResult<Uuid> {
    use sqlx::Row;
    let row = sqlx::query(
        "INSERT INTO vendor_submission (task_id, comment) VALUES ($1, $2) RETURNING id",
    )
    .bind(task_id)
    .bind(comment)
    .fetch_one(pool)
    .await?;
    Ok(row.try_get("id")?)
}

pub async fn attach_submission_document(
    pool: &PgPool,
    submission_id: Uuid,
    document_id: Uuid,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO vendor_submission_document (submission_id, document_id) VALUES ($1, $2)",
    )
    .bind(submission_id)
    .bind(document_id)
    .execute(pool)
    .await?;
    Ok(())
}
