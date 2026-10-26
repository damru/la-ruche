use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

#[derive(Debug, Serialize)]
pub struct ClaimNoteRow {
    pub id: Uuid,
    pub claim_id: Uuid,
    pub author_id: Uuid,
    pub author_name: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

pub async fn create(
    pool: &PgPool,
    claim_id: Uuid,
    building_id: Uuid,
    author_id: Uuid,
    body: &str,
) -> AppResult<ClaimNoteRow> {
    let body = body.trim();
    if body.is_empty() {
        return Err(AppError::validation_code("errors.claims.note_body_empty"));
    }

    // Verify the claim belongs to this building (404 otherwise).
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM claim WHERE id = $1 AND building_id = $2)")
            .bind(claim_id)
            .bind(building_id)
            .fetch_one(pool)
            .await?;
    if !exists {
        return Err(AppError::NotFound);
    }

    let row = sqlx::query(
        r#"INSERT INTO claim_note (claim_id, author_id, body)
           VALUES ($1, $2, $3)
           RETURNING id, claim_id, author_id, body, created_at"#,
    )
    .bind(claim_id)
    .bind(author_id)
    .bind(body)
    .fetch_one(pool)
    .await?;

    let id: Uuid = row.try_get("id")?;
    let author_name: String =
        sqlx::query_scalar("SELECT first_name || ' ' || last_name FROM user_account WHERE id = $1")
            .bind(author_id)
            .fetch_one(pool)
            .await?;

    Ok(ClaimNoteRow {
        id,
        claim_id,
        author_id,
        author_name,
        body: row.try_get("body")?,
        created_at: row.try_get("created_at")?,
    })
}

pub async fn list(
    pool: &PgPool,
    claim_id: Uuid,
    building_id: Uuid,
) -> AppResult<Vec<ClaimNoteRow>> {
    // Verify the claim belongs to this building (404 otherwise).
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM claim WHERE id = $1 AND building_id = $2)")
            .bind(claim_id)
            .bind(building_id)
            .fetch_one(pool)
            .await?;
    if !exists {
        return Err(AppError::NotFound);
    }

    let rows = sqlx::query(
        r#"SELECT n.id, n.claim_id, n.author_id,
                  u.first_name || ' ' || u.last_name AS author_name,
                  n.body, n.created_at
           FROM claim_note n
           JOIN user_account u ON u.id = n.author_id
           WHERE n.claim_id = $1
           ORDER BY n.created_at ASC"#,
    )
    .bind(claim_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|row| {
            Ok(ClaimNoteRow {
                id: row.try_get("id")?,
                claim_id: row.try_get("claim_id")?,
                author_id: row.try_get("author_id")?,
                author_name: row.try_get("author_name")?,
                body: row.try_get("body")?,
                created_at: row.try_get("created_at")?,
            })
        })
        .collect()
}
