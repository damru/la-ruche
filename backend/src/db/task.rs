use crate::errors::AppResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
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
    pub assigned_to: Option<Uuid>,
    pub created_by: Uuid,
    /// Raw token UUID — capability secret, never exposed in list/get responses.
    pub vendor_token: Option<Uuid>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
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
    pub assigned_to: Option<Uuid>,
    pub created_by: Uuid,
    pub has_vendor_token: bool,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
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
            assigned_to: r.assigned_to,
            created_by: r.created_by,
            token_expires_at: r.token_expires_at,
            closed_at: r.closed_at,
            created_at: r.created_at,
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
    pub assigned_to: Option<Uuid>,
}

/// Valid task status values for update requests.
const VALID_STATUSES: &[&str] = &[
    "open",
    "in_progress",
    "awaiting_vendor",
    "closed",
    "cancelled",
];

pub fn validate_status(status: &str) -> bool {
    VALID_STATUSES.contains(&status)
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
        assigned_to: r.try_get("assigned_to")?,
        created_by: r.try_get("created_by")?,
        vendor_token: r.try_get("vendor_token")?,
        token_expires_at: r.try_get("token_expires_at")?,
        closed_at: r.try_get("closed_at")?,
        created_at: r.try_get("created_at")?,
    })
}

pub async fn list(pool: &PgPool, building_id: Uuid) -> AppResult<Vec<TaskRow>> {
    let rows = sqlx::query(
        r#"SELECT id, building_id, title, description,
           status::text AS status, priority::text AS priority,
           contract_id, contractor_id, assigned_to, created_by, vendor_token,
           token_expires_at, closed_at, created_at
           FROM task WHERE building_id = $1
           ORDER BY CASE status::text WHEN 'closed' THEN 1 ELSE 0 END ASC,
                    CASE priority::text WHEN 'urgent' THEN 1 WHEN 'high' THEN 2
                    WHEN 'normal' THEN 3 ELSE 4 END, created_at DESC"#,
    )
    .bind(building_id)
    .fetch_all(pool)
    .await?;
    rows.iter()
        .map(task_row_from_query)
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

pub async fn get(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<Option<TaskRow>> {
    let row = sqlx::query(
        r#"SELECT id, building_id, title, description,
           status::text AS status, priority::text AS priority,
           contract_id, contractor_id, assigned_to, created_by, vendor_token,
           token_expires_at, closed_at, created_at
           FROM task WHERE id = $1 AND building_id = $2"#,
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
    let priority = input.priority.as_deref().unwrap_or("normal");
    let row = sqlx::query(
        r#"INSERT INTO task (building_id, title, description, priority, contract_id, contractor_id, assigned_to, created_by)
           VALUES ($1, $2, $3, $4::text::task_priority, $5, $6, $7, $8)
           RETURNING id, building_id, title, description,
           status::text AS status, priority::text AS priority,
           contract_id, contractor_id, assigned_to, created_by, vendor_token,
           token_expires_at, closed_at, created_at"#,
    )
    .bind(building_id)
    .bind(&input.title)
    .bind(&input.description)
    .bind(priority)
    .bind(input.contract_id)
    .bind(input.contractor_id)
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
    let closed_at = if status == "closed" {
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
         status = 'awaiting_vendor'::task_status, updated_at = NOW()
         WHERE id = $1 AND building_id = $2",
    )
    .bind(id)
    .bind(building_id)
    .bind(token)
    .bind(expires_at)
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

/// Deletes a task. Returns 404 if not found.
pub async fn delete(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM task WHERE id = $1 AND building_id = $2")
        .bind(id)
        .bind(building_id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(crate::errors::AppError::NotFound);
    }
    Ok(())
}

/// Updates mutable task fields. `None` values leave the existing field unchanged.
pub async fn update_fields(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    title: Option<&str>,
    description: Option<Option<&str>>,
    priority: Option<&str>,
    status: Option<&str>,
    contractor_id: Option<Option<Uuid>>,
) -> AppResult<()> {
    let current = get(pool, id, building_id)
        .await?
        .ok_or(crate::errors::AppError::NotFound)?;

    let new_title = title.unwrap_or(&current.title).to_string();
    let new_description = match description {
        Some(d) => d.map(|s| s.to_string()),
        None => current.description,
    };
    let new_priority = priority.unwrap_or(&current.priority).to_string();
    let new_status = status.unwrap_or(&current.status).to_string();
    let new_contractor_id = match contractor_id {
        Some(c) => c,
        None => current.contractor_id,
    };
    let closed_at = if new_status == "closed" && current.closed_at.is_none() {
        Some(Utc::now())
    } else if new_status != "closed" {
        None
    } else {
        current.closed_at
    };

    let result = sqlx::query(
        "UPDATE task SET title = $3, description = $4,
         priority = $5::text::task_priority,
         status = $6::text::task_status,
         contractor_id = $7,
         closed_at = $8, updated_at = NOW()
         WHERE id = $1 AND building_id = $2",
    )
    .bind(id)
    .bind(building_id)
    .bind(&new_title)
    .bind(&new_description)
    .bind(&new_priority)
    .bind(&new_status)
    .bind(new_contractor_id)
    .bind(closed_at)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(crate::errors::AppError::NotFound);
    }
    Ok(())
}

/// Returns 404 if the task doesn't exist.
pub async fn revoke_token(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE task SET vendor_token = NULL, token_expires_at = NULL, updated_at = NOW()
         WHERE id = $1 AND building_id = $2",
    )
    .bind(id)
    .bind(building_id)
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
        r#"SELECT id, building_id, title, description,
           status::text AS status, priority::text AS priority,
           contract_id, contractor_id, assigned_to, created_by, vendor_token,
           token_expires_at, closed_at, created_at
           FROM task
           WHERE vendor_token = $1 AND token_expires_at > NOW()
           AND status != 'closed'"#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;
    row.as_ref()
        .map(task_row_from_query)
        .transpose()
        .map_err(Into::into)
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
