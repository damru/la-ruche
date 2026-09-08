use crate::errors::{AppError, AppResult};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Full report row enriched with author name (JOIN with user_account)
/// and document count (subquery on report_document).
/// Returned by all list/get API responses.
#[derive(Debug, Serialize)]
pub struct ReportRow {
    pub id: Uuid,
    pub building_id: Uuid,
    pub unit_id: Option<Uuid>,
    pub author_id: Uuid,
    pub author_first_name: String,
    pub author_last_name: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub task_id: Option<Uuid>,
    pub task_status: Option<String>,
    pub document_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

const LIST_QUERY: &str = concat!(
    "SELECT r.id, r.building_id, r.unit_id, r.author_id,",
    " u.first_name AS author_first_name, u.last_name AS author_last_name,",
    " r.title, r.description, r.status::text AS status,",
    " r.task_id, t.status::text AS task_status,",
    " (SELECT COUNT(*) FROM report_document rd WHERE rd.report_id = r.id) AS document_count,",
    " r.created_at, r.updated_at",
    " FROM report r",
    " JOIN user_account u ON u.id = r.author_id",
    " LEFT JOIN task t ON t.id = r.task_id AND t.building_id = r.building_id",
    " WHERE r.building_id = $1 ORDER BY r.updated_at DESC",
);

const GET_QUERY: &str = concat!(
    "SELECT r.id, r.building_id, r.unit_id, r.author_id,",
    " u.first_name AS author_first_name, u.last_name AS author_last_name,",
    " r.title, r.description, r.status::text AS status,",
    " r.task_id, t.status::text AS task_status,",
    " (SELECT COUNT(*) FROM report_document rd WHERE rd.report_id = r.id) AS document_count,",
    " r.created_at, r.updated_at",
    " FROM report r",
    " JOIN user_account u ON u.id = r.author_id",
    " LEFT JOIN task t ON t.id = r.task_id AND t.building_id = r.building_id",
    " WHERE r.id = $1 AND r.building_id = $2",
);

fn row_to_report(r: &sqlx::postgres::PgRow) -> Result<ReportRow, sqlx::Error> {
    Ok(ReportRow {
        id: r.try_get("id")?,
        building_id: r.try_get("building_id")?,
        unit_id: r.try_get("unit_id")?,
        author_id: r.try_get("author_id")?,
        author_first_name: r.try_get("author_first_name")?,
        author_last_name: r.try_get("author_last_name")?,
        title: r.try_get("title")?,
        description: r.try_get("description")?,
        status: r.try_get("status")?,
        task_id: r.try_get("task_id")?,
        task_status: r.try_get("task_status")?,
        document_count: r.try_get("document_count")?,
        created_at: r.try_get("created_at")?,
        updated_at: r.try_get("updated_at")?,
    })
}

/// Valid report status values — mirrors the `report_status` PostgreSQL enum.
pub const VALID_STATUSES: &[&str] = &["open", "in_progress", "closed"];
pub const STATUS_CLOSED: &str = "closed";
pub const STATUS_IN_PROGRESS: &str = "in_progress";

pub fn validate_status(status: &str) -> bool {
    VALID_STATUSES.contains(&status)
}

pub async fn list(pool: &PgPool, building_id: Uuid) -> AppResult<Vec<ReportRow>> {
    let rows = sqlx::query(LIST_QUERY)
        .bind(building_id)
        .fetch_all(pool)
        .await?;
    rows.iter()
        .map(row_to_report)
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

pub async fn get(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<Option<ReportRow>> {
    let row = sqlx::query(GET_QUERY)
        .bind(id)
        .bind(building_id)
        .fetch_optional(pool)
        .await?;
    row.as_ref()
        .map(row_to_report)
        .transpose()
        .map_err(Into::into)
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateReportInput {
    pub title: String,
    pub description: Option<String>,
    pub unit_id: Option<Uuid>,
}

pub async fn create(
    pool: &PgPool,
    building_id: Uuid,
    author_id: Uuid,
    input: CreateReportInput,
) -> AppResult<ReportRow> {
    // Step 1: insert and obtain the new row's id.
    // Using a plain INSERT … RETURNING id (not a CTE) so that the INSERT is never
    // committed while the subsequent enriched SELECT fails, which would save the
    // report in the DB but return a "Database error" to the caller.
    let row = sqlx::query(
        "INSERT INTO report (building_id, author_id, title, description, unit_id) \
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(building_id)
    .bind(author_id)
    .bind(&input.title)
    .bind(&input.description)
    .bind(input.unit_id)
    .fetch_one(pool)
    .await?;
    let id: Uuid = row.try_get("id")?;

    // Step 2: fetch the enriched row (JOINs user_account + document_count subquery)
    // reusing the same query used by the GET endpoint.
    get(pool, id, building_id)
        .await?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("created report not found")))
}

pub async fn update_status(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    status: &str,
) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE report SET status = $3::text::report_status, updated_at = NOW() WHERE id = $1 AND building_id = $2",
    )
    .bind(id)
    .bind(building_id)
    .bind(status)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

/// Converts a report into a task (EP-011-S03).
/// Returns 404 if the report does not exist in the building.
/// Returns 409 Conflict if the report was already converted (task_id IS NOT NULL).
/// On success: inserts a task, links it back to the report via task_id, and returns the task UUID.
pub async fn convert_to_task(
    pool: &PgPool,
    report_id: Uuid,
    building_id: Uuid,
    created_by: Uuid,
    priority: &str,
) -> AppResult<Uuid> {
    let mut tx = pool.begin().await?;

    // Step 1: check report exists and hasn't been converted yet.
    let existing: Option<Option<Uuid>> =
        sqlx::query_scalar("SELECT task_id FROM report WHERE id = $1 AND building_id = $2")
            .bind(report_id)
            .bind(building_id)
            .fetch_optional(&mut *tx)
            .await?;

    match existing {
        None => return Err(AppError::NotFound),
        Some(Some(_)) => {
            return Err(AppError::conflict_code(
                "errors.reports.already_converted_to_task",
            ));
        }
        Some(None) => {}
    }

    // Step 2: load the source report payload inside the transaction.
    let report_row =
        sqlx::query("SELECT title, description FROM report WHERE id = $1 AND building_id = $2")
            .bind(report_id)
            .bind(building_id)
            .fetch_one(&mut *tx)
            .await?;
    let title: String = report_row.try_get("title")?;
    let description: Option<String> = report_row.try_get("description")?;

    // Step 3: create the task with the selected priority.
    let row = sqlx::query(
        r#"INSERT INTO task (building_id, title, description, priority, created_by)
           VALUES ($1, $2, $3, $4::text::task_priority, $5)
           RETURNING id"#,
    )
    .bind(building_id)
    .bind(&title)
    .bind(&description)
    .bind(priority)
    .bind(created_by)
    .fetch_one(&mut *tx)
    .await?;
    let task_id: Uuid = row.try_get("id")?;

    // Step 4: link the task back to the report and mark the report in progress.
    sqlx::query(
        "UPDATE report SET task_id = $3, status = $4::text::report_status, updated_at = NOW() \
         WHERE id = $1 AND building_id = $2",
    )
    .bind(report_id)
    .bind(building_id)
    .bind(task_id)
    .bind(STATUS_IN_PROGRESS)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(task_id)
}

pub async fn close_linked_to_task<'e, E>(
    executor: E,
    task_id: Uuid,
    building_id: Uuid,
) -> AppResult<()>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query(
        "UPDATE report
         SET status = $3::text::report_status, updated_at = NOW()
         WHERE task_id = $1 AND building_id = $2 AND status != $3::text::report_status",
    )
    .bind(task_id)
    .bind(building_id)
    .bind(STATUS_CLOSED)
    .execute(executor)
    .await?;
    Ok(())
}

// ── Update / Delete ───────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
pub struct UpdateReportInput {
    pub title: String,
    pub description: Option<String>,
    pub unit_id: Option<Uuid>,
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    input: UpdateReportInput,
) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE report SET title = $3, description = $4, unit_id = $5, updated_at = NOW() \
         WHERE id = $1 AND building_id = $2",
    )
    .bind(id)
    .bind(building_id)
    .bind(&input.title)
    .bind(&input.description)
    .bind(input.unit_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub async fn delete(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<()> {
    let mut tx = pool.begin().await?;
    let linked_document_ids = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT rd.document_id
        FROM report_document rd
        JOIN report r ON r.id = rd.report_id
        WHERE rd.report_id = $1 AND r.building_id = $2
        "#,
    )
    .bind(id)
    .bind(building_id)
    .fetch_all(&mut *tx)
    .await?;

    // report_document rows are removed automatically via ON DELETE CASCADE (migration 0010).
    let result = sqlx::query("DELETE FROM report WHERE id = $1 AND building_id = $2")
        .bind(id)
        .bind(building_id)
        .execute(&mut *tx)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    crate::db::document::delete_orphaned_tx(&mut tx, &linked_document_ids, building_id).await?;
    tx.commit().await?;
    Ok(())
}

// ── Document attachment ───────────────────────────────────────────

/// List documents attached to a report (id, original_name, mime_type, size_bytes, created_at).
pub async fn list_documents(
    pool: &PgPool,
    report_id: Uuid,
    building_id: Uuid,
) -> AppResult<Vec<serde_json::Value>> {
    let rows = sqlx::query(
        r#"SELECT d.id, d.original_name, d.mime_type, d.size_bytes, d.created_at
           FROM report_document rd
           JOIN document d ON d.id = rd.document_id AND d.building_id = $2
           WHERE rd.report_id = $1 AND d.deleted_at IS NULL
           ORDER BY d.created_at ASC"#,
    )
    .bind(report_id)
    .bind(building_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.try_get::<Uuid, _>("id").unwrap_or_default(),
                "original_name": r.try_get::<String, _>("original_name").unwrap_or_default(),
                "mime_type": r.try_get::<String, _>("mime_type").unwrap_or_default(),
                "size_bytes": r.try_get::<i64, _>("size_bytes").unwrap_or_default(),
                "created_at": r.try_get::<DateTime<Utc>, _>("created_at").unwrap_or_default(),
            })
        })
        .collect())
}

/// Attach an existing document to a report.
/// Verifies both the report and the document belong to the building before linking.
/// Returns false when either does not exist.
pub async fn attach_document(
    pool: &PgPool,
    report_id: Uuid,
    document_id: Uuid,
    building_id: Uuid,
) -> AppResult<bool> {
    let pair_exists: bool = sqlx::query_scalar(
        r#"SELECT EXISTS(
               SELECT 1 FROM report r, document d
               WHERE r.id = $1 AND r.building_id = $3
                 AND d.id = $2 AND d.building_id = $3
           )"#,
    )
    .bind(report_id)
    .bind(document_id)
    .bind(building_id)
    .fetch_one(pool)
    .await?;

    if !pair_exists {
        return Ok(false);
    }

    sqlx::query(
        "INSERT INTO report_document (report_id, document_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(report_id)
    .bind(document_id)
    .execute(pool)
    .await?;

    Ok(true)
}

/// Detach a document from a report.
/// Scoped to the building. Returns false if the link did not exist.
pub async fn detach_document(
    pool: &PgPool,
    report_id: Uuid,
    document_id: Uuid,
    building_id: Uuid,
) -> AppResult<bool> {
    let result = sqlx::query(
        r#"DELETE FROM report_document rd
           USING report r
           WHERE rd.report_id = $1
             AND rd.document_id = $2
             AND r.id = rd.report_id
             AND r.building_id = $3"#,
    )
    .bind(report_id)
    .bind(document_id)
    .bind(building_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
