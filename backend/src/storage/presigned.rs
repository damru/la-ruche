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

    /// Generates a pre-signed URL for download (GET), forcing the
    /// `attachment` disposition. Called on each download request — never stored.
    pub async fn presigned_download_url(
        &self,
        bucket: &str,
        key: &str,
        filename: &str,
        ttl_secs: u64,
    ) -> AppResult<String> {
        let disposition = format!("attachment; filename=\"{}\"", sanitize_filename(filename));
        self.presigned_get_url(bucket, key, &disposition, ttl_secs)
            .await
    }

    /// Generates a pre-signed URL for in-app rendering (GET) of a PDF,
    /// with the `inline` disposition and an RFC 5987-encoded filename.
    /// Same bucket, same short TTL, same access path as
    /// `presigned_download_url` — only the disposition differs.
    pub async fn presigned_inline_url(
        &self,
        bucket: &str,
        key: &str,
        filename: &str,
        ttl_secs: u64,
    ) -> AppResult<String> {
        let disposition = format!(
            "inline; filename=\"{}\"; filename*=UTF-8''{}",
            sanitize_filename(filename),
            encode_rfc5987(filename)
        );
        self.presigned_get_url(bucket, key, &disposition, ttl_secs)
            .await
    }

    async fn presigned_get_url(
        &self,
        bucket: &str,
        key: &str,
        disposition: &str,
        ttl_secs: u64,
    ) -> AppResult<String> {
        let presigning_cfg = PresigningConfig::expires_in(Duration::from_secs(ttl_secs))
            .map_err(|e| AppError::Storage(e.to_string()))?;

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

/// Percent-encodes a filename for the RFC 5987 `filename*` parameter
/// (UTF-8 attribute-value): keeps RFC 5987 `attr-char`, encodes everything else.
const RFC5987_ATTR_CHAR: &percent_encoding::AsciiSet = &percent_encoding::NON_ALPHANUMERIC
    .remove(b'!')
    .remove(b'#')
    .remove(b'$')
    .remove(b'&')
    .remove(b'+')
    .remove(b'-')
    .remove(b'.')
    .remove(b'^')
    .remove(b'_')
    .remove(b'`')
    .remove(b'|')
    .remove(b'~');

fn encode_rfc5987(name: &str) -> String {
    percent_encoding::utf8_percent_encode(name, RFC5987_ATTR_CHAR).to_string()
}

#[cfg(test)]
mod tests {
    use super::{encode_rfc5987, sanitize_filename};

    #[test]
    fn rfc5987_keeps_attr_chars() {
        assert_eq!(encode_rfc5987("PV_AG_2026-01.pdf"), "PV_AG_2026-01.pdf");
    }

    #[test]
    fn rfc5987_encodes_spaces_and_accents() {
        assert_eq!(
            encode_rfc5987("PV AG été.pdf"),
            "PV%20AG%20%C3%A9t%C3%A9.pdf"
        );
    }

    #[test]
    fn rfc5987_encodes_semicolon_and_quote() {
        assert_eq!(encode_rfc5987("a;b\"c.pdf"), "a%3Bb%22c.pdf");
    }

    #[test]
    fn sanitize_replaces_dangerous_chars() {
        assert_eq!(sanitize_filename("a\"b\\c;d.pdf"), "a_b_c_d.pdf");
    }
}
