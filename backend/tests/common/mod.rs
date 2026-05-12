// Shared helpers for La Ruche integration tests.
// Each integration test file imports this module with `mod common;`.
#![allow(dead_code)]

use axum_test::TestServer;
use serde_json::json;

/// Creates a test server with the full application stack and runs pending
/// migrations against the test database.
pub async fn test_server() -> TestServer {
    dotenvy::dotenv().ok();
    let cfg = laruche::config::AppConfig::load().expect("config");
    let pool = sqlx::PgPool::connect(&cfg.db.url).await.expect("db");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations");

    let storage = laruche::storage::client::StorageClient::new(&cfg.s3)
        .await
        .expect("storage");

    let state = std::sync::Arc::new(laruche::db::AppState { pool, storage, cfg });
    let webauthn = laruche::auth::passkey::build_webauthn(&state.cfg.app).expect("webauthn");
    let app = laruche::api::router(state, webauthn);

    TestServer::new(app)
}

/// Minimal valid bootstrap payload used across onboarding and assembly tests.
pub fn bootstrap_payload() -> serde_json::Value {
    json!({
        "email": "syndic@test.fr",
        "last_name": "Martin",
        "first_name": "Sophie",
        "password": "MotDePasse123!",
        "building_name": "Résidence Les Tests",
        "address_line1": "1 rue des Tests",
        "postal_code": "75001",
        "city": "Paris",
        "shares_total": 1000
    })
}
