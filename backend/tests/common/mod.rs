// Shared helpers for La Ruche integration tests.
// Each integration test file imports this module with `mod common;`.
//
// Database: when `DATABASE_URL` is set (CI, docker-compose) the harness uses
// that PostgreSQL instance directly. Otherwise it boots a real ephemeral
// PostgreSQL (postgresql_embedded — binary cached after first download) so the
// suite stays runnable with zero external services on a developer machine.
// S3/MinIO is always replaced by an in-process wiremock HTTP stub.
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
    // Kept alive for the whole process when we booted an embedded server;
    // `None` when an external `DATABASE_URL` (CI / docker) is used.
    _pg: Option<PostgreSQL>,
    _s3: MockServer,
}

static INFRA: OnceCell<Infra> = OnceCell::const_new();

async fn infra() -> &'static Infra {
    INFRA
        .get_or_init(|| async {
            // ── PostgreSQL ────────────────────────────────────────────
            // Prefer an externally provided database (CI, docker-compose);
            // fall back to an embedded server for local zero-setup runs.
            let (db_url, pg) = match std::env::var("DATABASE_URL") {
                Ok(url) if !url.is_empty() => (url, None),
                _ => {
                    let mut pg = PostgreSQL::default();
                    pg.setup().await.expect("embedded postgres setup");
                    pg.start().await.expect("embedded postgres start");
                    let db_name = "laruche_test";
                    pg.create_database(db_name)
                        .await
                        .expect("create test database");
                    let url = pg.settings().url(db_name);
                    (url, Some(pg))
                }
            };

            // Run migrations once against the test database.
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

/// Returns the shared test config and a fresh pool to the test database.
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
    server_for(cfg, pool).await
}

/// Like [`test_server`], but wired to a brand-new, freshly-migrated database
/// dedicated to the caller. Use this for flows that require a pristine instance
/// (e.g. first-run onboarding, which asserts an empty `user_account` and relies
/// on the bootstrap first-run guard): the shared test database is polluted by
/// every other integration binary, so order-independent isolation is mandatory.
pub async fn isolated_test_server() -> TestServer {
    let infra = infra().await;

    // Carve out a uniquely-named database on the same PostgreSQL server.
    // The name is a server-generated UUID identifier — no user input — so the
    // `AssertSqlSafe` wrapper required by sqlx 0.9 carries no injection risk.
    let db_name = format!("laruche_test_{}", uuid::Uuid::new_v4().simple());
    let admin = PgPool::connect(&infra.cfg.db.url)
        .await
        .expect("connect to create isolated database");
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "CREATE DATABASE \"{db_name}\""
    )))
    .execute(&admin)
    .await
    .expect("create isolated database");
    admin.close().await;

    // Point a config clone at the new database and migrate it.
    let mut url = url::Url::parse(&infra.cfg.db.url).expect("parse test database url");
    url.set_path(&format!("/{db_name}"));
    let db_url = url.to_string();

    let pool = PgPool::connect(&db_url)
        .await
        .expect("connect to isolated database");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations on isolated database");

    let mut cfg = infra.cfg.clone();
    cfg.db.url = db_url;
    server_for(cfg, pool).await
}

/// Like [`isolated_test_server`], but also returns a second pool connected to
/// the same isolated database, for tests that need to assert on rows written
/// as a side effect of an API call (e.g. the audit trail) without an HTTP
/// surface to read them back through.
pub async fn isolated_test_server_with_pool() -> (TestServer, PgPool) {
    let infra = infra().await;

    let db_name = format!("laruche_test_{}", uuid::Uuid::new_v4().simple());
    let admin = PgPool::connect(&infra.cfg.db.url)
        .await
        .expect("connect to create isolated database");
    sqlx::query(sqlx::AssertSqlSafe(format!(
        "CREATE DATABASE \"{db_name}\""
    )))
    .execute(&admin)
    .await
    .expect("create isolated database");
    admin.close().await;

    let mut url = url::Url::parse(&infra.cfg.db.url).expect("parse test database url");
    url.set_path(&format!("/{db_name}"));
    let db_url = url.to_string();

    let pool = PgPool::connect(&db_url)
        .await
        .expect("connect to isolated database");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations on isolated database");

    let audit_pool = pool.clone();

    let mut cfg = infra.cfg.clone();
    cfg.db.url = db_url;
    let server = server_for(cfg, pool).await;
    (server, audit_pool)
}

/// Assembles the application stack into a cookie-persisting `TestServer`.
async fn server_for(cfg: AppConfig, pool: PgPool) -> TestServer {
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
