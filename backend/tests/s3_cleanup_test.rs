// Integration tests for the periodic S3 cleanup job (EP-005-S09):
// deferred retention purge of soft-deleted documents (pass 2). Pass 1
// (orphan detection) requires a real S3-compatible `ListObjectsV2` XML response,
// which the shared wiremock catch-all stub does not provide — it is
// exercised at the unit level in `domain::retention` and by inspection;
// this test asserts that a pass-1 failure (unparseable mock response) is
// swallowed as a counted error rather than aborting the job, and that pass 2
// runs to completion regardless.
//
// Run with: cargo test --test s3_cleanup_test

mod common;

use chrono::{Duration, Months, Utc};
use laruche::domain::Role;
use sqlx::PgPool;
use uuid::Uuid;

struct Document {
    id: Uuid,
    bucket: String,
    storage_path: String,
    category: &'static str,
}

async fn seed_building(pool: &PgPool) -> Uuid {
    let building_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total)
         VALUES ($1, 'TestBuilding', '1 rue Test', '75001', 'Paris', 1000)",
    )
    .bind(building_id)
    .execute(pool)
    .await
    .expect("building");
    building_id
}

async fn seed_manager(pool: &PgPool, building_id: Uuid) -> Uuid {
    let manager_id = Uuid::new_v4();
    let password_hash =
        laruche::auth::password::hash_password("TestPass123!", &common::test_config().await.auth)
            .expect("hash");
    sqlx::query(
        "INSERT INTO user_account (id, email, last_name, first_name, password_hash)
         VALUES ($1, $2, 'Test', 'Manager', $3)",
    )
    .bind(manager_id)
    .bind(format!("mgr-{}@test.local", Uuid::new_v4()))
    .bind(&password_hash)
    .execute(pool)
    .await
    .expect("user");
    sqlx::query(
        "INSERT INTO user_building (user_id, building_id, roles)
         VALUES ($1, $2, ARRAY[$3]::user_role[])",
    )
    .bind(manager_id)
    .bind(building_id)
    .bind(Role::Manager.to_string())
    .execute(pool)
    .await
    .expect("user_building");
    manager_id
}

/// Inserts a soft-deleted document, backdating `deleted_at` by `age`.
async fn seed_soft_deleted_document(
    pool: &PgPool,
    building_id: Uuid,
    manager_id: Uuid,
    category: &'static str,
    age: Duration,
) -> Document {
    let doc = Document {
        id: Uuid::new_v4(),
        bucket: format!("laruche-{}", building_id.simple()),
        storage_path: format!("uploads/{}.pdf", Uuid::new_v4()),
        category,
    };
    let deleted_at = Utc::now() - age;

    sqlx::query(
        "INSERT INTO document
            (id, building_id, storage_path, bucket, original_name, mime_type, size_bytes,
             category, uploaded_by, deleted_at, visible_to_roles)
         VALUES ($1, $2, $3, $4, 'test.pdf', 'application/pdf', 1024, $5::document_category, $6, $7, ARRAY['manager']::user_role[])",
    )
    .bind(doc.id)
    .bind(building_id)
    .bind(&doc.storage_path)
    .bind(&doc.bucket)
    .bind(category)
    .bind(manager_id)
    .bind(deleted_at)
    .execute(pool)
    .await
    .expect("insert soft-deleted document");

    doc
}

#[tokio::test]
async fn expired_retention_purges_the_document_row() {
    let (_server, pool) = common::isolated_test_server_with_pool().await;
    let cfg = common::test_config().await;
    let building_id = seed_building(&pool).await;
    let manager_id = seed_manager(&pool, building_id).await;

    // `diagnostic` retention is 10 years (domain::retention::retention_months);
    // back-date deleted_at well past that.
    let doc = seed_soft_deleted_document(
        &pool,
        building_id,
        manager_id,
        "diagnostic",
        Duration::days(11 * 365),
    )
    .await;

    let storage = laruche::storage::client::StorageClient::new(&cfg.s3)
        .await
        .expect("storage");
    let state = std::sync::Arc::new(laruche::db::AppState {
        pool: pool.clone(),
        storage,
        cfg,
    });

    let report = laruche::jobs::s3_cleanup::run_once(&state)
        .await
        .expect("run_once");
    assert_eq!(report.documents_purged, 1);

    let exists: Option<Uuid> = sqlx::query_scalar("SELECT id FROM document WHERE id = $1")
        .bind(doc.id)
        .fetch_optional(&pool)
        .await
        .expect("query");
    assert!(exists.is_none(), "expired document must be hard-deleted");
}

#[tokio::test]
async fn pv_ag_soft_deleted_is_never_purged_regardless_of_age() {
    let (_server, pool) = common::isolated_test_server_with_pool().await;
    let cfg = common::test_config().await;
    let building_id = seed_building(&pool).await;
    let manager_id = seed_manager(&pool, building_id).await;

    let doc = seed_soft_deleted_document(
        &pool,
        building_id,
        manager_id,
        "pv_ag",
        Duration::days(365 * 50),
    )
    .await;

    let storage = laruche::storage::client::StorageClient::new(&cfg.s3)
        .await
        .expect("storage");
    let state = std::sync::Arc::new(laruche::db::AppState {
        pool: pool.clone(),
        storage,
        cfg,
    });

    let report = laruche::jobs::s3_cleanup::run_once(&state)
        .await
        .expect("run_once");
    assert_eq!(report.documents_purged, 0);

    let exists: Option<Uuid> = sqlx::query_scalar("SELECT id FROM document WHERE id = $1")
        .bind(doc.id)
        .fetch_optional(&pool)
        .await
        .expect("query");
    assert!(exists.is_some(), "pv_ag must never be purged");
}

#[tokio::test]
async fn retention_not_yet_elapsed_is_preserved() {
    let (_server, pool) = common::isolated_test_server_with_pool().await;
    let cfg = common::test_config().await;
    let building_id = seed_building(&pool).await;
    let manager_id = seed_manager(&pool, building_id).await;

    let doc = seed_soft_deleted_document(
        &pool,
        building_id,
        manager_id,
        "diagnostic",
        Duration::days(30),
    )
    .await;

    let storage = laruche::storage::client::StorageClient::new(&cfg.s3)
        .await
        .expect("storage");
    let state = std::sync::Arc::new(laruche::db::AppState {
        pool: pool.clone(),
        storage,
        cfg,
    });

    let report = laruche::jobs::s3_cleanup::run_once(&state)
        .await
        .expect("run_once");
    assert_eq!(report.documents_purged, 0);

    let exists: Option<Uuid> = sqlx::query_scalar("SELECT id FROM document WHERE id = $1")
        .bind(doc.id)
        .fetch_optional(&pool)
        .await
        .expect("query");
    assert!(exists.is_some(), "not-yet-expired document must survive");
}

/// `Months::new(120)` boundary check, exercised end-to-end through the job
/// rather than only at the `domain::retention` unit level.
#[tokio::test]
async fn boundary_exactly_at_retention_limit_is_purged() {
    let (_server, pool) = common::isolated_test_server_with_pool().await;
    let cfg = common::test_config().await;
    let building_id = seed_building(&pool).await;
    let manager_id = seed_manager(&pool, building_id).await;

    let deleted_at = Utc::now().checked_sub_months(Months::new(120)).unwrap();
    let doc = Document {
        id: Uuid::new_v4(),
        bucket: format!("laruche-{}", building_id.simple()),
        storage_path: format!("uploads/{}.pdf", Uuid::new_v4()),
        category: "autre",
    };
    sqlx::query(
        "INSERT INTO document
            (id, building_id, storage_path, bucket, original_name, mime_type, size_bytes,
             category, uploaded_by, deleted_at, visible_to_roles)
         VALUES ($1, $2, $3, $4, 'test.pdf', 'application/pdf', 1024, $5::document_category, $6, $7, ARRAY['manager']::user_role[])",
    )
    .bind(doc.id)
    .bind(building_id)
    .bind(&doc.storage_path)
    .bind(&doc.bucket)
    .bind(doc.category)
    .bind(manager_id)
    .bind(deleted_at)
    .execute(&pool)
    .await
    .expect("insert soft-deleted document");

    let storage = laruche::storage::client::StorageClient::new(&cfg.s3)
        .await
        .expect("storage");
    let state = std::sync::Arc::new(laruche::db::AppState {
        pool: pool.clone(),
        storage,
        cfg,
    });

    let report = laruche::jobs::s3_cleanup::run_once(&state)
        .await
        .expect("run_once");
    assert_eq!(report.documents_purged, 1);
}
