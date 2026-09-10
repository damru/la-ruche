// Integration tests for the document-event outbox-lite (EP-005-S10, PRD
// US-DOC-009). Every successful upload writes exactly one `document_event`
// row atomically with the document, and the polling job marks it processed.
//
// Run with: cargo test --test document_events_test

mod common;

use axum::http::StatusCode;
use laruche::domain::Role;
use laruche::jobs::document_events::process_events;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

struct Fixture {
    building_id: Uuid,
    manager_id: Uuid,
}

async fn setup(pool: &sqlx::PgPool) -> Fixture {
    let building_id = Uuid::new_v4();
    let manager_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total)
         VALUES ($1, 'TestBuilding', '1 rue Test', '75001', 'Paris', 1000)",
    )
    .bind(building_id)
    .execute(pool)
    .await
    .expect("building");

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

    Fixture {
        building_id,
        manager_id,
    }
}

fn token_for(user_id: Uuid, building_id: Uuid, cfg: &laruche::config::AppConfig) -> String {
    laruche::auth::jwt::generate_access_token(
        user_id,
        building_id,
        vec![Role::Manager],
        &cfg.auth.jwt_secret,
        cfg.auth.access_token_ttl_secs,
    )
    .expect("token")
}

async fn confirm_upload(
    server: &axum_test::TestServer,
    auth: &str,
    building_id: Uuid,
    original_name: &str,
) -> axum_test::TestResponse {
    let bucket = format!("laruche-{}", building_id.simple());
    let body = json!({
        "key": format!("uploads/{}.pdf", Uuid::new_v4()),
        "bucket": bucket,
        "original_name": original_name,
        "mime_type": "application/pdf",
        "size_bytes": 1024,
        "visible_roles": ["manager"],
        "category": "libre",
    });

    server
        .post("/api/v1/documents")
        .add_header("Authorization", auth)
        .json(&body)
        .await
}

#[tokio::test]
async fn upload_creates_one_uploaded_event_atomically() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let resp = confirm_upload(&server, &auth, f.building_id, "note.pdf").await;
    assert_eq!(resp.status_code(), StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    let document_id: Uuid = body["id"].as_str().unwrap().parse().unwrap();

    let events = sqlx::query!(
        r#"SELECT event_type, document_id, processed_at FROM document_event WHERE document_id = $1"#,
        document_id
    )
    .fetch_all(&pool)
    .await
    .expect("query events");

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, "uploaded");
    assert_eq!(events[0].document_id, document_id);
    assert!(events[0].processed_at.is_none());
}

#[tokio::test]
async fn two_uploads_create_two_events_ordered_by_created_at() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let first = confirm_upload(&server, &auth, f.building_id, "first.pdf").await;
    assert_eq!(first.status_code(), StatusCode::CREATED);
    let second = confirm_upload(&server, &auth, f.building_id, "second.pdf").await;
    assert_eq!(second.status_code(), StatusCode::CREATED);

    let first_id: Uuid = first.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let second_id: Uuid = second.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    let events = sqlx::query!(
        r#"SELECT document_id, created_at FROM document_event
           WHERE document_id = ANY($1) ORDER BY created_at"#,
        &[first_id, second_id]
    )
    .fetch_all(&pool)
    .await
    .expect("query events");

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].document_id, first_id);
    assert_eq!(events[1].document_id, second_id);
}

#[tokio::test]
async fn job_marks_pending_events_processed_and_ignores_already_processed() {
    let (_cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;

    // Insert a document + a pending event directly.
    let document_id = sqlx::query_scalar::<_, Uuid>(
        r#"INSERT INTO document (building_id, uploaded_by, original_name, storage_path,
           bucket, mime_type, size_bytes, visible_to_roles, category)
           VALUES ($1, $2, 'x.pdf', 'k', 'b', 'application/pdf', 10, ARRAY['manager']::user_role[], 'libre')
           RETURNING id"#,
    )
    .bind(f.building_id)
    .bind(f.manager_id)
    .fetch_one(&pool)
    .await
    .expect("insert document");

    let pending_event_id = sqlx::query_scalar::<_, Uuid>(
        r#"INSERT INTO document_event (building_id, document_id, event_type, actor_id, payload)
           VALUES ($1, $2, 'uploaded', $3, '{}') RETURNING id"#,
    )
    .bind(f.building_id)
    .bind(document_id)
    .bind(f.manager_id)
    .fetch_one(&pool)
    .await
    .expect("insert pending event");

    // A previously-processed event: process_events must not touch it again.
    let already_processed_event_id = sqlx::query_scalar::<_, Uuid>(
        r#"INSERT INTO document_event (building_id, document_id, event_type, actor_id, payload, processed_at)
           VALUES ($1, $2, 'uploaded', $3, '{}', NOW() - INTERVAL '1 hour') RETURNING id"#,
    )
    .bind(f.building_id)
    .bind(document_id)
    .bind(f.manager_id)
    .fetch_one(&pool)
    .await
    .expect("insert processed event");

    let processed_at_before = sqlx::query_scalar::<_, Option<chrono::DateTime<chrono::Utc>>>(
        "SELECT processed_at FROM document_event WHERE id = $1",
    )
    .bind(already_processed_event_id)
    .fetch_one(&pool)
    .await
    .expect("read before");

    let storage = laruche::storage::client::StorageClient::new(&common::test_config().await.s3)
        .await
        .expect("storage");
    let state = Arc::new(laruche::db::AppState {
        pool: pool.clone(),
        storage,
        cfg: common::test_config().await,
    });

    process_events(&state).await.expect("process_events");

    let pending_processed_at = sqlx::query_scalar::<_, Option<chrono::DateTime<chrono::Utc>>>(
        "SELECT processed_at FROM document_event WHERE id = $1",
    )
    .bind(pending_event_id)
    .fetch_one(&pool)
    .await
    .expect("read pending after");
    assert!(pending_processed_at.is_some());

    let already_processed_at_after =
        sqlx::query_scalar::<_, Option<chrono::DateTime<chrono::Utc>>>(
            "SELECT processed_at FROM document_event WHERE id = $1",
        )
        .bind(already_processed_event_id)
        .fetch_one(&pool)
        .await
        .expect("read already-processed after");
    assert_eq!(processed_at_before, already_processed_at_after);
}
