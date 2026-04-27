// ── api/contractor.rs ─────────────────────────────────────────────
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    db::{self, AppState},
    errors::{AppError, AppResult},
};

/// GET /vendor/:token — public access, no JWT
pub async fn get_task(
    State(state): State<Arc<AppState>>,
    Path(token): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    use sqlx::Row;

    let row = sqlx::query(
        r#"SELECT t.id, t.title, t.description, t.status::text AS status, t.token_expires_at,
               b.name AS building_name,
               b.address_line1, b.address_line2, b.postal_code, b.city,
               u.first_name AS manager_first_name, u.last_name AS manager_last_name,
               u.email AS manager_email, u.phone AS manager_phone
           FROM task t
           JOIN building b ON b.id = t.building_id
           JOIN user_account u ON u.id = t.created_by
           WHERE t.vendor_token = $1 AND t.token_expires_at > NOW() AND t.status != 'closed'"#,
    )
    .bind(token)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let address_line1: String = row.try_get("address_line1")?;
    let address_line2: Option<String> = row.try_get("address_line2").ok().flatten();
    let postal_code: String = row.try_get("postal_code")?;
    let city: String = row.try_get("city")?;
    let building_address = match address_line2 {
        Some(line2) if !line2.is_empty() => {
            format!("{address_line1}, {line2}, {postal_code} {city}")
        }
        _ => format!("{address_line1}, {postal_code} {city}"),
    };

    Ok(Json(serde_json::json!({
        "id": row.try_get::<Uuid, _>("id")?,
        "title": row.try_get::<String, _>("title")?,
        "description": row.try_get::<Option<String>, _>("description")?,
        "status": row.try_get::<String, _>("status")?,
        "token_expires_at": row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("token_expires_at")?,
        "building_name": row.try_get::<String, _>("building_name")?,
        "building_address": building_address,
        "manager_name": format!(
            "{} {}",
            row.try_get::<String, _>("manager_first_name")?,
            row.try_get::<String, _>("manager_last_name")?
        ),
        "manager_email": row.try_get::<String, _>("manager_email")?,
        "manager_phone": row.try_get::<Option<String>, _>("manager_phone")?,
    })))
}

#[derive(Serialize)]
pub struct StagingUploadUrlResponse {
    pub url: String,
    pub key: String,
}

#[derive(Deserialize)]
pub struct StagingUploadUrlInput {
    pub extension: String,
    pub content_type: String,
}

/// POST /vendor/:token/upload-url — obtain a presigned PUT URL for staging bucket
pub async fn upload_url(
    State(state): State<Arc<AppState>>,
    Path(token): Path<Uuid>,
    Json(input): Json<StagingUploadUrlInput>,
) -> AppResult<Json<StagingUploadUrlResponse>> {
    // Validate token before issuing a presigned URL
    let _ = db::task::get_by_token(&state.pool, token)
        .await?
        .ok_or(AppError::NotFound)?;

    if input.extension.trim().is_empty() || input.extension.len() > 10 {
        return Err(AppError::Validation("Invalid file extension".into()));
    }

    let upload = state
        .storage
        .presigned_staging_upload_url(
            &input.extension,
            &input.content_type,
            state.cfg.s3.presigned_ttl_secs,
        )
        .await?;

    Ok(Json(StagingUploadUrlResponse {
        url: upload.url,
        key: upload.key,
    }))
}

/// Metadata for one document being submitted.
#[derive(Deserialize)]
pub struct SubmissionDocumentMeta {
    /// Key in the staging bucket (returned by upload-url endpoint).
    pub key: String,
    /// Original filename provided by the vendor.
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
}

#[derive(Deserialize)]
pub struct SubmissionInput {
    pub comment: Option<String>,
    pub documents: Vec<SubmissionDocumentMeta>,
}

/// POST /vendor/:token/submission
pub async fn submit(
    State(state): State<Arc<AppState>>,
    Path(token): Path<Uuid>,
    Json(input): Json<SubmissionInput>,
) -> AppResult<StatusCode> {
    let task = db::task::get_by_token(&state.pool, token)
        .await?
        .ok_or(AppError::NotFound)?;

    // Use a DB transaction so that partial failures don't leave inconsistent state
    let mut tx = state.pool.begin().await?;

    // Create submission record
    let submission_id = {
        use sqlx::Row;
        let row = sqlx::query(
            "INSERT INTO vendor_submission (task_id, comment) VALUES ($1, $2) RETURNING id",
        )
        .bind(task.id)
        .bind(&input.comment)
        .fetch_one(&mut *tx)
        .await?;
        row.try_get::<Uuid, _>("id")?
    };

    // Move each file from staging to the building's final bucket and create document records
    for doc_meta in &input.documents {
        let ext = doc_meta.key.rsplit('.').next().unwrap_or("bin");

        let (bucket, final_key) = state
            .storage
            .move_from_staging(&doc_meta.key, task.building_id, ext)
            .await?;

        // Vendor submissions have no authenticated user.
        // task.created_by (the manager who opened the task) is used as the
        // uploaded_by proxy to satisfy the FK constraint and maintain an audit trail.
        let doc = db::document::create_tx(
            &mut tx,
            task.building_id,
            task.created_by,
            &doc_meta.original_name,
            &final_key,
            &bucket,
            &doc_meta.mime_type,
            doc_meta.size_bytes,
            &["manager".to_string(), "board_member".to_string()],
        )
        .await?;

        sqlx::query(
            "INSERT INTO vendor_submission_document (submission_id, document_id) VALUES ($1, $2)",
        )
        .bind(submission_id)
        .bind(doc.id)
        .execute(&mut *tx)
        .await?;
    }

    // Close the task
    sqlx::query(
        "UPDATE task SET status = 'closed'::task_status, closed_at = NOW(), updated_at = NOW()
         WHERE id = $1 AND building_id = $2",
    )
    .bind(task.id)
    .bind(task.building_id)
    .execute(&mut *tx)
    .await?;

    // Auto-feed the maintenance log (idempotent via unique index on task_id)
    // Includes contractor name if the task has one assigned.
    sqlx::query(
        r#"INSERT INTO maintenance_log (building_id, created_by, title, task_id, intervention_date, source, vendor_name)
           SELECT t.building_id, $3, t.title, t.id, CURRENT_DATE, 'automatic'::log_source,
                  c.name AS vendor_name
           FROM task t
           LEFT JOIN contractor c ON c.id = t.contractor_id
           WHERE t.id = $1 AND t.building_id = $2
           ON CONFLICT DO NOTHING"#,
    )
    .bind(task.id)
    .bind(task.building_id)
    .bind(task.created_by)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(StatusCode::CREATED)
}
