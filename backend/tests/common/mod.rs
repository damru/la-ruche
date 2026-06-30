// Shared helpers for La Ruche integration tests.
// Each integration test file imports this module with `mod common;`.
//
// The harness is fully self-contained: it boots a real ephemeral PostgreSQL
// instance (postgresql_embedded — binary cached after first download) and a
// wiremock HTTP server standing in for S3/MinIO. No external services or
// environment configuration are required to run the tests.
#![allow(dead_code)]

use std::sync::Arc;

use axum_test::TestServer;
use laruche::config::{AppConfig, AppSettings, AuthSettings, DbSettings, S3Settings};
use postgresql_embedded::PostgreSQL;
use serde_json::json;
use sqlx::PgPool;
use tokio::sync::OnceCell;
use wiremock::matchers::any;
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Process-wide test infrastructure, initialised once per test binary.
/// The embedded PostgreSQL server and the wiremock server are kept alive for
/// the whole process lifetime by holding them here.
struct Infra {
    cfg: AppConfig,
    _pg: PostgreSQL,
    _s3: MockServer,
}

static INFRA: OnceCell<Infra> = OnceCell::const_new();

async fn infra() -> &'static Infra {
    INFRA
        .get_or_init(|| async {
            // ── Embedded PostgreSQL ───────────────────────────────────
            let mut pg = PostgreSQL::default();
            pg.setup().await.expect("embedded postgres setup");
            pg.start().await.expect("embedded postgres start");
            let db_name = "laruche_test";
            pg.create_database(db_name)
                .await
                .expect("create test database");
            let db_url = pg.settings().url(db_name);

            // Run migrations once against the shared test database.
            let pool = PgPool::connect(&db_url)
                .await
                .expect("connect for migrations");
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .expect("run migrations");
            pool.close().await;

            // ── HTTP mock standing in for S3/MinIO ────────────────────
            // A catch-all 200 covers the bucket/object operations performed
            // during tests (CreateBucket, CopyObject, DeleteObject, …).
            // Presigning is computed locally and makes no HTTP call.
            let s3 = MockServer::start().await;
            Mock::given(any())
                .respond_with(ResponseTemplate::new(200))
                .mount(&s3)
                .await;
            let s3_uri = s3.uri();

            let cfg = test_config(db_url, s3_uri);
            Infra {
                cfg,
                _pg: pg,
                _s3: s3,
            }
        })
        .await
}

/// Builds a fully programmatic test configuration — no config file or env vars.
fn test_config(db_url: String, s3_uri: String) -> AppConfig {
    AppConfig {
        app: AppSettings {
            host: "127.0.0.1".to_string(),
            port: 0,
            public_url: "http://localhost".to_string(),
            name: "La Ruche Test".to_string(),
            demo_mode: false,
        },
        db: DbSettings {
            url: db_url,
            max_connections: 5,
        },
        s3: S3Settings {
            endpoint_url: s3_uri.clone(),
            region: "us-east-1".to_string(),
            access_key_id: "test".to_string(),
            secret_access_key: "test".to_string(),
            bucket_prefix: "laruche".to_string(),
            staging_bucket: "laruche-staging".to_string(),
            presigned_ttl_secs: 900,
            public_url: Some(s3_uri),
        },
        auth: AuthSettings {
            // 256-bit secret (min required by the jwt module).
            jwt_secret: "test-secret-test-secret-test-secret-0123456789ab".to_string(),
            access_token_ttl_secs: 3600,
            refresh_token_ttl_days: 7,
            invitation_ttl_days: 7,
            reset_token_ttl_secs: 3600,
            vendor_token_ttl_days: 30,
            // Minimal Argon2 cost — fast hashing for tests.
            argon2_memory_kb: 8,
            argon2_iterations: 1,
        },
    }
}

/// Returns the shared test config and a fresh pool to the embedded database.
/// Both `test_db()` and `test_server()` target the same database, so data
/// seeded via the returned pool is visible to the running application.
pub async fn test_db() -> (AppConfig, PgPool) {
    let infra = infra().await;
    let pool = PgPool::connect(&infra.cfg.db.url)
        .await
        .expect("connect to test database");
    (infra.cfg.clone(), pool)
}

/// Creates a test server with the full application stack, wired to the
/// embedded PostgreSQL database and the wiremock S3 stub. Cookie persistence
/// is enabled so authenticated flows (login/bootstrap) carry over between
/// requests on the same server instance.
pub async fn test_server() -> TestServer {
    let (cfg, pool) = test_db().await;
    let storage = laruche::storage::client::StorageClient::new(&cfg.s3)
        .await
        .expect("storage");

    let state = Arc::new(laruche::db::AppState { pool, storage, cfg });
    let webauthn = laruche::auth::passkey::build_webauthn(&state.cfg.app).expect("webauthn");
    let app = laruche::api::router(state, webauthn);

    let mut server = TestServer::new(app);
    server.save_cookies();
    server
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
