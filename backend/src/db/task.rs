use crate::errors::AppResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TaskRow {
    pub id: Uuid,
    pub building_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub contract_id: Option<Uuid>,
    pub assigned_to: Option<Uuid>,
    pub vendor_token: Option<Uuid>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskInput {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub contract_id: Option<Uuid>,
    pub assigned_to: Option<Uuid>,
}

pub async fn list(pool: &PgPool, building_id: Uuid) -> AppResult<Vec<TaskRow>> {
    Ok(sqlx::query_as!(
        TaskRow,
        r#"SELECT id, building_id, title, description,
           status::text as "status!", priority::text as "priority!",
           contract_id, assigned_to, vendor_token, token_expires_at, closed_at, created_at
           FROM task WHERE building_id = $1
           ORDER BY CASE priority::text WHEN 'urgent' THEN 1 WHEN 'high' THEN 2
                    WHEN 'normal' THEN 3 ELSE 4 END, created_at DESC"#,
        building_id
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<Option<TaskRow>> {
    Ok(sqlx::query_as!(
        TaskRow,
        r#"SELECT id, building_id, title, description,
           status::text as "status!", priority::text as "priority!",
           contract_id, assigned_to, vendor_token, token_expires_at, closed_at, created_at
           FROM task WHERE id = $1 AND building_id = $2"#,
        id,
        building_id
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn create(
    pool: &PgPool,
    building_id: Uuid,
    user_id: Uuid,
    input: CreateTaskInput,
) -> AppResult<TaskRow> {
    let priority = input.priority.as_deref().unwrap_or("normal");
    Ok(sqlx::query_as!(TaskRow,
        r#"INSERT INTO task (building_id, title, description, priority, contract_id, assigned_to, created_by)
           VALUES ($1, $2, $3, $4::text::task_priority, $5, $6, $7)
           RETURNING id, building_id, title, description,
           status::text as "status!", priority::text as "priority!",
           contract_id, assigned_to, vendor_token, token_expires_at, closed_at, created_at"#,
        building_id, input.title, input.description, priority,
        input.contract_id, input.assigned_to, user_id
    ).fetch_one(pool).await?)
}

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
    sqlx::query!(
        "UPDATE task SET status = $3::text::task_status, closed_at = $4, updated_at = NOW() WHERE id = $1 AND building_id = $2",
        id, building_id, status, closed_at
    ).execute(pool).await?;
    Ok(())
}

pub async fn generate_token(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    ttl_days: u64,
) -> AppResult<(Uuid, DateTime<Utc>)> {
    let token = uuid::Uuid::new_v4();
    let expires_at = Utc::now() + chrono::Duration::days(ttl_days as i64);

    sqlx::query!(
        "UPDATE task SET vendor_token = $3, token_expires_at = $4,
         status = 'awaiting_vendor'::task_status, updated_at = NOW()
         WHERE id = $1 AND building_id = $2",
        id,
        building_id,
        token,
        expires_at
    )
    .execute(pool)
    .await?;

    Ok((token, expires_at))
}

pub async fn revoke_token(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<()> {
    sqlx::query!(
        "UPDATE task SET vendor_token = NULL, token_expires_at = NULL, updated_at = NOW() WHERE id = $1 AND building_id = $2",
        id, building_id
    ).execute(pool).await?;
    Ok(())
}

pub async fn get_by_token(pool: &PgPool, token: Uuid) -> AppResult<Option<TaskRow>> {
    Ok(sqlx::query_as!(
        TaskRow,
        r#"SELECT id, building_id, title, description,
           status::text as "status!", priority::text as "priority!",
           contract_id, assigned_to, vendor_token, token_expires_at, closed_at, created_at
           FROM task
           WHERE vendor_token = $1 AND token_expires_at > NOW()
           AND status != 'closed'"#,
        token
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn create_submission(
    pool: &PgPool,
    task_id: Uuid,
    comment: Option<String>,
) -> AppResult<Uuid> {
    let id = sqlx::query_scalar!(
        "INSERT INTO vendor_submission (task_id, comment) VALUES ($1, $2) RETURNING id",
        task_id,
        comment
    )
    .fetch_one(pool)
    .await?;
    Ok(id)
}

pub async fn attach_submission_document(
    pool: &PgPool,
    submission_id: Uuid,
    document_id: Uuid,
) -> AppResult<()> {
    sqlx::query!(
        "INSERT INTO vendor_submission_document (submission_id, document_id) VALUES ($1, $2)",
        submission_id,
        document_id
    )
    .execute(pool)
    .await?;
    Ok(())
}
