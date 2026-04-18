use std::sync::Arc;
use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{db::{self, AppState}, domain::AuthUser, errors::{AppError, AppResult}};

#[derive(Deserialize, Validate, ToSchema)]
pub struct GetUploadUrlInput {
    #[validate(length(min = 1, max = 10))]
    pub extension: String,
    #[validate(length(min = 1))]
    pub content_type: String,
    pub size_bytes: i64,
}

#[derive(Serialize, ToSchema)]
pub struct UploadUrlResponse {
    pub url: String,
    pub key: String,
    pub bucket: String,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct ConfirmUploadInput {
    #[validate(length(min = 1))]
    pub key: String,
    #[validate(length(min = 1))]
    pub bucket: String,
    #[validate(length(min = 1))]
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub visible_roles: Vec<String>,
}

/// POST /api/v1/documents/upload-url
/// Generates a pre-signed PUT URL for direct S3 upload — file never passes through backend
pub async fn get_upload_url(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(input): Json<GetUploadUrlInput>,
) -> AppResult<Json<UploadUrlResponse>> {
    input.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // Sanitize extension (prevent path traversal)
    let ext = input.extension.trim_start_matches('.').to_lowercase();
    if ext.contains('/') || ext.contains('\\') {
        return Err(AppError::Validation("Invalid extension".into()));
    }

    let upload = state.storage
        .presigned_upload_url(auth.building_id, &ext, &input.content_type, state.cfg.s3.presigned_ttl_secs)
        .await?;

    Ok(Json(UploadUrlResponse { url: upload.url, key: upload.key, bucket: upload.bucket }))
}

/// POST /api/v1/documents — confirms a successful S3 upload and records in DB
pub async fn confirm_upload(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(input): Json<ConfirmUploadInput>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    input.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // Verify the key belongs to the building's bucket
    let expected_bucket = state.storage.bucket_name(auth.building_id);
    if input.bucket != expected_bucket {
        return Err(AppError::Forbidden);
    }

    let doc = db::document::create(
        &state.pool,
        auth.building_id,
        auth.user_id,
        &input.original_name,
        &input.key,
        &input.bucket,
        &input.mime_type,
        input.size_bytes,
        &input.visible_roles,
    ).await?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "id": doc.id,
        "original_name": doc.original_name,
        "size_bytes": doc.size_bytes,
        "created_at": doc.created_at,
    }))))
}

/// GET /api/v1/documents
pub async fn list(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<serde_json::Value>>> {
    // Filtre par rôles de l'utilisateur courant
    let roles: Vec<String> = auth.roles.iter().map(|r| format!("{:?}", r).to_lowercase()).collect();
    let docs = db::document::list(&state.pool, auth.building_id, &roles).await?;
    Ok(Json(docs.into_iter().map(|d| serde_json::json!({
        "id": d.id,
        "original_name": d.original_name,
        "mime_type": d.mime_type,
        "size_bytes": d.size_bytes,
        "created_at": d.created_at,
    })).collect()))
}

/// GET /api/v1/documents/:id/download-url
/// Generates a pre-signed GET URL — never store on client side
pub async fn get_download_url(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let doc = db::document::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let url = state.storage
        .presigned_download_url(&doc.bucket, &doc.storage_path, &doc.original_name, state.cfg.s3.presigned_ttl_secs)
        .await?;

    Ok(Json(serde_json::json!({ "url": url })))
}

/// DELETE /api/v1/documents/:id
pub async fn delete(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    auth.require_role(&crate::domain::Role::Manager)?;

    let deleted = db::document::delete(&state.pool, id, auth.building_id).await?;
    if let Some((bucket, key)) = deleted {
        let storage = state.storage.clone();
        tokio::spawn(async move {
            if let Err(e) = storage.delete_object(&bucket, &key).await {
                tracing::error!(error = %e, "S3 delete error");
            }
        });
    }

    Ok(StatusCode::NO_CONTENT)
}
