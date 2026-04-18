pub mod assembly;
pub mod budget;
pub mod maintenance_log;
pub mod consultation;
pub mod contract;
pub mod building;
pub mod document;
pub mod unit;
pub mod report;
pub mod claim;
pub mod task;
pub mod user;

use sqlx::PgPool;

use crate::{config::AppConfig, storage::client::StorageClient};

/// Shared state injected into all handlers via Arc<AppState>
pub struct AppState {
    pub pool: PgPool,
    pub storage: StorageClient,
    pub cfg: AppConfig,
}
