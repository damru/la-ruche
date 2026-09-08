use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    db::{self, AppState},
    domain::AuthUser,
    errors::{AppError, AppResult},
};

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

#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentListItem {
    pub id: Uuid,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub visible_roles: Vec<String>,
    pub linked_contract_names: Vec<String>,
    pub linked_report_titles: Vec<String>,
    pub linked_claim_titles: Vec<String>,
    pub linked_report_count: i64,
    pub category: String,
    pub supersedes_id: Option<Uuid>,
    pub is_current: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<db::document::DocumentListRow> for DocumentListItem {
    fn from(document: db::document::DocumentListRow) -> Self {
        Self {
            id: document.id,
            original_name: document.original_name,
            mime_type: document.mime_type,
            size_bytes: document.size_bytes,
            visible_roles: document.visible_roles,
            linked_contract_names: document.linked_contract_names,
            linked_report_titles: document.linked_report_titles,
            linked_claim_titles: document.linked_claim_titles,
            linked_report_count: document.linked_report_count,
            category: document.category,
            supersedes_id: document.supersedes_id,
            is_current: document.is_current,
            created_at: document.created_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DocumentVersionItem {
    pub id: Uuid,
    pub original_name: String,
    pub category: String,
    pub supersedes_id: Option<Uuid>,
    pub is_current: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<db::document::DocumentRow> for DocumentVersionItem {
    fn from(document: db::document::DocumentRow) -> Self {
        Self {
            id: document.id,
            original_name: document.original_name,
            category: document.category,
            supersedes_id: document.supersedes_id,
            is_current: document.is_current,
            created_at: document.created_at,
        }
    }
}

/// Valid `document_category` enum values (EP-005-S06/S07, LEGAL §1.2, §3.4, §6.1-6.3).
const VALID_CATEGORIES: &[&str] = &[
    "reglement_copropriete",
    "edd",
    "pv_ag",
    "comptabilite",
    "appel_fonds",
    "budget",
    "contrat",
    "diagnostic",
    "sinistre",
    "tache",
    "libre",
    "signalement",
    "carnet_entretien",
    "prestataire",
    "autre",
];

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
    /// Founding document category (`reglement_copropriete` | `edd` | `autre`); defaults to `autre`.
    #[serde(default)]
    pub category: Option<String>,
    /// When set, publishes a new version replacing this document (must be the current version, same category).
    #[serde(default)]
    pub supersedes_id: Option<Uuid>,
    /// Justification, required when replacing a founding document
    /// (`reglement_copropriete` | `edd`) via `supersedes_id` (US-DOC-007).
    #[serde(default)]
    pub reason: Option<String>,
}

/// POST /api/v1/documents/upload-url
/// Generates a pre-signed PUT URL for direct S3 upload — file never passes through backend
pub async fn get_upload_url(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(input): Json<GetUploadUrlInput>,
) -> AppResult<Json<UploadUrlResponse>> {
    input
        .validate()
        .map_err(|_| AppError::validation_code("errors.documents.invalid_upload_request"))?;

    // Sanitize extension (prevent path traversal)
    let ext = input.extension.trim_start_matches('.').to_lowercase();
    if ext.contains('/') || ext.contains('\\') {
        return Err(AppError::validation_code(
            "errors.documents.invalid_extension",
        ));
    }

    // Ensure bucket exists — idempotent, handles seeded data or transient S3 failures.
    // Non-fatal: a dispatch error here must not block the presigned URL generation;
    // the bucket may already exist and the PUT will succeed regardless.
    if let Err(e) = state.storage.ensure_bucket(auth.building_id).await {
        tracing::warn!(building_id = %auth.building_id, error = %e, "Could not ensure S3 bucket — proceeding anyway");
    }

    let upload = state
        .storage
        .presigned_upload_url(
            auth.building_id,
            &ext,
            &input.content_type,
            state.cfg.s3.presigned_ttl_secs,
        )
        .await?;

    Ok(Json(UploadUrlResponse {
        url: upload.url,
        key: upload.key,
        bucket: upload.bucket,
    }))
}

/// POST /api/v1/documents — confirms a successful S3 upload and records in DB
pub async fn confirm_upload(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(input): Json<ConfirmUploadInput>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    input
        .validate()
        .map_err(|_| AppError::validation_code("errors.documents.invalid_confirm_request"))?;

    // Verify the key belongs to the building's bucket
    let expected_bucket = state.storage.bucket_name(auth.building_id);
    if input.bucket != expected_bucket {
        return Err(AppError::Forbidden);
    }

    let category = input.category.as_deref().unwrap_or("autre");
    if !VALID_CATEGORIES.contains(&category) {
        return Err(AppError::validation_code(
            "errors.documents.invalid_category",
        ));
    }

    let doc = if let Some(supersedes_id) = input.supersedes_id {
        db::document::create_version(
            &state.pool,
            auth.building_id,
            auth.user_id,
            &input.original_name,
            &input.key,
            &input.bucket,
            &input.mime_type,
            input.size_bytes,
            &input.visible_roles,
            category,
            supersedes_id,
            input.reason.as_deref(),
        )
        .await?
    } else {
        db::document::create(
            &state.pool,
            auth.building_id,
            auth.user_id,
            &input.original_name,
            &input.key,
            &input.bucket,
            &input.mime_type,
            input.size_bytes,
            &input.visible_roles,
            category,
        )
        .await
        .map_err(|e| match e {
            AppError::Database(ref db_err)
                if db_err
                    .as_database_error()
                    .and_then(|d| d.constraint())
                    .map(|c| c == "document_current_founding_unique")
                    .unwrap_or(false) =>
            {
                AppError::unprocessable_entity_code("errors.documents.current_already_exists")
            }
            other => other,
        })?
    };

    if let Err(e) = state
        .storage
        .set_original_name_metadata(&input.bucket, &input.key, &input.original_name)
        .await
    {
        tracing::warn!(key = %input.key, error = %e, "Could not set S3 original-name metadata — non-fatal");
    }

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": doc.id,
            "original_name": doc.original_name,
            "size_bytes": doc.size_bytes,
            "category": doc.category,
            "is_current": doc.is_current,
            "supersedes_id": doc.supersedes_id,
            "created_at": doc.created_at,
        })),
    ))
}

/// GET /api/v1/documents
pub async fn list(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<DocumentListItem>>> {
    // Filter by current user's roles
    let roles: Vec<String> = auth.roles.iter().map(ToString::to_string).collect();
    let rows = db::document::list(&state.pool, auth.building_id, &roles).await?;

    Ok(Json(rows.into_iter().map(DocumentListItem::from).collect()))
}

/// GET /api/v1/documents/:id/versions — full version chain (EP-005-S06)
pub async fn versions(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<DocumentVersionItem>>> {
    let rows = db::document::version_chain(&state.pool, id, auth.building_id).await?;
    Ok(Json(
        rows.into_iter().map(DocumentVersionItem::from).collect(),
    ))
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

    let url = state
        .storage
        .presigned_download_url(
            &doc.bucket,
            &doc.storage_path,
            &doc.original_name,
            state.cfg.s3.presigned_ttl_secs,
        )
        .await?;

    Ok(Json(serde_json::json!({ "url": url })))
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct DeleteDocumentInput {
    #[serde(default)]
    pub reason: Option<String>,
}

/// DELETE /api/v1/documents/:id
pub async fn delete(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<DeleteDocumentInput>,
) -> AppResult<StatusCode> {
    auth.require_role(&crate::domain::Role::Manager)?;

    let deleted = db::document::delete(
        &state.pool,
        id,
        auth.building_id,
        auth.user_id,
        input.reason.as_deref(),
    )
    .await?;
    deleted.ok_or(AppError::NotFound)?;

    Ok(StatusCode::NO_CONTENT)
}
