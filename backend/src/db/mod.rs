pub mod allocation_key;
pub mod assembly;
pub mod budget;
pub mod building;
pub mod claim;
pub mod consultation;
pub mod contract;
pub mod document;
pub mod maintenance_log;
pub mod report;
pub mod task;
pub mod unit;
pub mod user;

use sqlx::PgPool;

use crate::{config::AppConfig, storage::client::StorageClient};

/// Shared state injected into all handlers via Arc<AppState>
pub struct AppState {
    pub pool: PgPool,
    pub storage: StorageClient,
    pub cfg: AppConfig,
}
