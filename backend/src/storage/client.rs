use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{
    Client,
    config::{Builder, Credentials},
};

use crate::{
    config::S3Settings,
    errors::{AppError, AppResult},
};

#[derive(Clone)]
pub struct StorageClient {
    pub(crate) inner: Client,
    pub(crate) presign_client: Client,
    pub(crate) cfg: S3Settings,
}

/// Builds an S3 client with path-style URLs forced.
/// Required for MinIO which does not support virtual-hosted-style by default.
fn build_client(base: &aws_config::SdkConfig, force_path_style: bool) -> Client {
    let s3_config = Builder::from(base)
        .force_path_style(force_path_style)
        .build();
    Client::from_conf(s3_config)
}

impl StorageClient {
    pub async fn new(cfg: &S3Settings) -> AppResult<Self> {
        let credentials = Credentials::new(
            &cfg.access_key_id,
            &cfg.secret_access_key,
            None,
            None,
            "laruche-static",
        );

        // Force path-style when endpoint_url is set (MinIO / self-hosted).
        // AWS S3 itself does not use path-style, so this is safe to always
        // enable when a custom endpoint is configured.
        let force_path_style = true;

        let inner_base = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(cfg.region.clone()))
            .credentials_provider(credentials.clone())
            .endpoint_url(&cfg.endpoint_url)
            .load()
            .await;

        // When public_url differs from endpoint_url (Docker dev), build a
        // dedicated client that signs presigned URLs with the public hostname.
        let presign_endpoint = cfg.public_url.as_deref().unwrap_or(&cfg.endpoint_url);
        let presign_base = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(cfg.region.clone()))
            .credentials_provider(credentials)
            .endpoint_url(presign_endpoint)
            .load()
            .await;

        Ok(Self {
            inner: build_client(&inner_base, force_path_style),
            presign_client: build_client(&presign_base, force_path_style),
            cfg: cfg.clone(),
        })
    }

    /// Bucket name for a building.
    /// Format: {prefix}-{building_id_no_dashes}
    pub fn bucket_name(&self, building_id: uuid::Uuid) -> String {
        format!(
            "{}-{}",
            self.cfg.bucket_prefix,
            building_id.to_string().replace('-', "")
        )
    }

    /// Creates the building bucket if it does not exist.
    pub async fn ensure_bucket(&self, building_id: uuid::Uuid) -> AppResult<()> {
        let bucket = self.bucket_name(building_id);
        match self.inner.create_bucket().bucket(&bucket).send().await {
            Ok(_) => Ok(()),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("BucketAlreadyOwnedByYou") || msg.contains("BucketAlreadyExists") {
                    Ok(())
                } else {
                    tracing::error!(error = %e, bucket = %bucket, "Error creating bucket");
                    Err(AppError::Storage(msg))
                }
            }
        }
    }

    /// Deletes a file from its bucket.
    pub async fn delete_object(&self, bucket: &str, key: &str) -> AppResult<()> {
        self.inner
            .delete_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map(|_| ())
            .map_err(|e| AppError::Storage(e.to_string()))
    }

    /// Lists every object key in a bucket, with its `last_modified` timestamp.
    /// Paginates through the SDK's continuation token. A missing/nonexistent
    /// bucket (no upload has happened yet for that building) is treated as
    /// empty rather than an error (EP-005-S09).
    pub async fn list_objects(
        &self,
        bucket: &str,
    ) -> AppResult<Vec<(String, Option<aws_sdk_s3::primitives::DateTime>)>> {
        let mut objects = Vec::new();
        let mut continuation_token: Option<String> = None;

        loop {
            let mut req = self.inner.list_objects_v2().bucket(bucket);
            if let Some(token) = continuation_token.take() {
                req = req.continuation_token(token);
            }

            let resp = match req.send().await {
                Ok(resp) => resp,
                Err(e) => {
                    let msg = e.to_string();
                    if msg.contains("NoSuchBucket") {
                        return Ok(Vec::new());
                    }
                    return Err(AppError::Storage(msg));
                }
            };

            for object in resp.contents() {
                if let Some(key) = object.key() {
                    objects.push((key.to_string(), object.last_modified().copied()));
                }
            }

            match resp.next_continuation_token() {
                Some(token) => continuation_token = Some(token.to_string()),
                None => break,
            }
        }

        Ok(objects)
    }
}
