use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{config::Credentials, Client};

use crate::{config::S3Settings, errors::{AppError, AppResult}};

#[derive(Clone)]
pub struct StorageClient {
    pub(crate) inner: Client,
    pub(crate) cfg: S3Settings,
}

impl StorageClient {
    pub async fn new(cfg: &S3Settings) -> AppResult<Self> {
        let credentials = Credentials::new(
            &cfg.access_key_id,
            &cfg.secret_access_key,
            None, None, "laruche-static",
        );

        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(cfg.region.clone()))
            .credentials_provider(credentials)
            .endpoint_url(&cfg.endpoint_url)
            .load()
            .await;

        let inner = Client::new(&config);
        Ok(Self { inner, cfg: cfg.clone() })
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
    /// Called on building creation.
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

    /// Supprime un fichier du bucket.
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
}
