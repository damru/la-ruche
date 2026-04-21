use aws_sdk_s3::{presigning::PresigningConfig, types::MetadataDirective};
use std::time::Duration;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    storage::client::StorageClient,
};

#[derive(Debug)]
pub struct UploadUrl {
    /// Pre-signed URL for direct PUT from client
    pub url: String,
    /// Generated S3 key (UUID + extension) — store in DB after confirmation
    pub key: String,
    /// Target bucket
    pub bucket: String,
}

impl StorageClient {
    /// Generates a pre-signed URL for direct browser upload (PUT).
    ///
    /// Security:
    /// - Key is a UUID — never the original filename
    /// - Short TTL (configurable, default 15 min)
    /// - Bucket is private: this URL is the only upload path
    /// - Content-type is constrained to the provided value
    pub async fn presigned_upload_url(
        &self,
        building_id: Uuid,
        extension: &str,
        content_type: &str,
        ttl_secs: u64,
    ) -> AppResult<UploadUrl> {
        let bucket = self.bucket_name(building_id);
        let key = format!("{}.{}", Uuid::new_v4(), extension.trim_start_matches('.'));

        let presigning_cfg = PresigningConfig::expires_in(Duration::from_secs(ttl_secs))
            .map_err(|e| AppError::Storage(e.to_string()))?;

        let url = self
            .presign_client
            .put_object()
            .bucket(&bucket)
            .key(&key)
            .content_type(content_type)
            .presigned(presigning_cfg)
            .await
            .map_err(|e| AppError::Storage(e.to_string()))?
            .uri()
            .to_string();

        Ok(UploadUrl { url, key, bucket })
    }

    /// Generates a pre-signed URL for download (GET).
    /// Called on each download request — never stored.
    pub async fn presigned_download_url(
        &self,
        bucket: &str,
        key: &str,
        filename: &str,
        ttl_secs: u64,
    ) -> AppResult<String> {
        let presigning_cfg = PresigningConfig::expires_in(Duration::from_secs(ttl_secs))
            .map_err(|e| AppError::Storage(e.to_string()))?;

        // Content-Disposition to force download with original filename
        let disposition = format!("attachment; filename=\"{}\"", sanitize_filename(filename));

        let url = self
            .presign_client
            .get_object()
            .bucket(bucket)
            .key(key)
            .response_content_disposition(disposition)
            .presigned(presigning_cfg)
            .await
            .map_err(|e| AppError::Storage(e.to_string()))?
            .uri()
            .to_string();

        Ok(url)
    }

    /// Pre-signed URL for upload to staging bucket (vendors).
    /// File is moved to final bucket after backend validation.
    #[allow(dead_code)]
    pub async fn presigned_staging_upload_url(
        &self,
        extension: &str,
        content_type: &str,
        ttl_secs: u64,
    ) -> AppResult<UploadUrl> {
        let bucket = self.cfg.staging_bucket.clone();
        let key = format!("{}.{}", Uuid::new_v4(), extension.trim_start_matches('.'));

        let presigning_cfg = PresigningConfig::expires_in(Duration::from_secs(ttl_secs))
            .map_err(|e| AppError::Storage(e.to_string()))?;

        let url = self
            .presign_client
            .put_object()
            .bucket(&bucket)
            .key(&key)
            .content_type(content_type)
            .presigned(presigning_cfg)
            .await
            .map_err(|e| AppError::Storage(e.to_string()))?
            .uri()
            .to_string();

        Ok(UploadUrl { url, key, bucket })
    }

    /// Moves a file from staging to the final bucket (copy + delete).
    pub async fn move_from_staging(
        &self,
        staging_key: &str,
        building_id: Uuid,
        extension: &str,
    ) -> AppResult<(String, String)> {
        let dest_bucket = self.bucket_name(building_id);
        let dest_key = format!("{}.{}", Uuid::new_v4(), extension.trim_start_matches('.'));

        let copy_source = format!("{}/{}", self.cfg.staging_bucket, staging_key);

        self.inner
            .copy_object()
            .copy_source(&copy_source)
            .bucket(&dest_bucket)
            .key(&dest_key)
            .send()
            .await
            .map_err(|e| AppError::Storage(e.to_string()))?;

        // Delete from staging after successful copy
        self.delete_object(&self.cfg.staging_bucket, staging_key)
            .await?;

        Ok((dest_bucket, dest_key))
    }

    /// Tags an already-uploaded S3 object with the original filename as user metadata.
    /// Uses a same-source copy with MetadataDirective::Replace — the only S3 way to
    /// update metadata without re-uploading the object body.
    /// Non-fatal: the object is usable even if this call fails.
    pub async fn set_original_name_metadata(
        &self,
        bucket: &str,
        key: &str,
        original_name: &str,
    ) -> AppResult<()> {
        let copy_source = format!("{}/{}", bucket, key);
        self.inner
            .copy_object()
            .copy_source(&copy_source)
            .bucket(bucket)
            .key(key)
            .metadata_directive(MetadataDirective::Replace)
            .metadata("original-name", sanitize_filename(original_name))
            .send()
            .await
            .map_err(|e| AppError::Storage(e.to_string()))?;
        Ok(())
    }
}

/// Sanitizes a filename for the Content-Disposition header.
/// Removes dangerous characters.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || "._ -".contains(c) {
                c
            } else {
                '_'
            }
        })
        .collect()
}
