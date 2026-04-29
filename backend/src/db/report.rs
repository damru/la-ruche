use crate::errors::{AppError, AppResult};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Full report row enriched with author name (JOIN with user_account).
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

const SELECT: &str = r#"
    SELECT r.id, r.building_id, r.unit_id, r.author_id,
           u.first_name AS author_first_name, u.last_name AS author_last_name,
           r.title, r.description, r.status::text AS status,
           r.created_at, r.updated_at
    FROM report r
    JOIN user_account u ON u.id = r.author_id
"#;

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
        created_at: r.try_get("created_at")?,
        updated_at: r.try_get("updated_at")?,
    })
}

/// Valid report status values for update requests.
const VALID_STATUSES: &[&str] = &["open", "in_progress", "closed"];

pub fn validate_status(status: &str) -> bool {
    VALID_STATUSES.contains(&status)
}

pub async fn list(pool: &PgPool, building_id: Uuid) -> AppResult<Vec<ReportRow>> {
    let rows = sqlx::query(&format!(
        "{SELECT} WHERE r.building_id = $1 ORDER BY r.created_at DESC"
    ))
    .bind(building_id)
    .fetch_all(pool)
    .await?;
    rows.iter()
        .map(row_to_report)
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

pub async fn get(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<Option<ReportRow>> {
    let row = sqlx::query(&format!(
        "{SELECT} WHERE r.id = $1 AND r.building_id = $2"
    ))
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
    let row = sqlx::query(
        r#"INSERT INTO report (building_id, author_id, title, description, unit_id)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id"#,
    )
    .bind(building_id)
    .bind(author_id)
    .bind(&input.title)
    .bind(&input.description)
    .bind(input.unit_id)
    .fetch_one(pool)
    .await?;
    let new_id: Uuid = row.try_get("id")?;
    get(pool, new_id, building_id)
        .await?
        .ok_or(AppError::NotFound)
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
/// Creates the task and returns its new UUID. The report itself is NOT deleted.
pub async fn convert_to_task(
    pool: &PgPool,
    report_id: Uuid,
    building_id: Uuid,
    created_by: Uuid,
) -> AppResult<Uuid> {
    // Fetch report title/description — error if not found or wrong building
    let row = sqlx::query(
        "SELECT title, description FROM report WHERE id = $1 AND building_id = $2",
    )
    .bind(report_id)
    .bind(building_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let title: String = row.try_get("title")?;
    let description: Option<String> = row.try_get("description")?;

    let task_row = sqlx::query(
        r#"INSERT INTO task (building_id, title, description, priority, created_by)
           VALUES ($1, $2, $3, 'normal'::task_priority, $4)
           RETURNING id"#,
    )
    .bind(building_id)
    .bind(&title)
    .bind(&description)
    .bind(created_by)
    .fetch_one(pool)
    .await?;

    Ok(task_row.try_get("id")?)
}
