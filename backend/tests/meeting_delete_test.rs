// Integration tests for EP-003-S12 — DELETE /meetings/{id} on a draft AG.
// A draft is deleted atomically with its children (resolutions, votes,
// attendance, documents); a non-draft or foreign meeting behaves as
// not found (404), and a non-manager role is refused (403).

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use laruche::domain::Role;
use serde_json::{Value, json};
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

/// Second building for the same manager, for cross-building isolation checks.
async fn setup_other_building(pool: &sqlx::PgPool, manager_id: Uuid) -> Uuid {
    let building_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total)
         VALUES ($1, 'OtherBuilding', '2 rue Test', '75002', 'Paris', 1000)",
    )
    .bind(building_id)
    .execute(pool)
    .await
    .expect("other building");
    sqlx::query(
        "INSERT INTO user_building (user_id, building_id, roles)
         VALUES ($1, $2, ARRAY[$3]::user_role[])",
    )
    .bind(manager_id)
    .bind(building_id)
    .bind(Role::Manager.to_string())
    .execute(pool)
    .await
    .expect("other user_building");
    building_id
}

fn token_for(
    user_id: Uuid,
    building_id: Uuid,
    role: Role,
    cfg: &laruche::config::AppConfig,
) -> String {
    laruche::auth::jwt::generate_access_token(
        user_id,
        building_id,
        vec![role],
        &cfg.auth.jwt_secret,
        cfg.auth.access_token_ttl_secs,
    )
    .expect("token")
}

async fn create_meeting(server: &TestServer, auth: &str) -> Uuid {
    let resp = server
        .post("/api/v1/meetings")
        .add_header("Authorization", auth)
        .json(&json!({
            "meeting_type": "annual",
            "meeting_date": "2027-06-15T18:00:00Z",
            "location": "Salle commune",
            "agenda": "Approbation du budget",
        }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::CREATED);
    resp.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap()
}

async fn create_resolution(server: &TestServer, auth: &str, meeting_id: Uuid, number: i32) -> Uuid {
    let resp = server
        .post(&format!("/api/v1/meetings/{meeting_id}/resolutions"))
        .add_header("Authorization", auth)
        .json(&json!({
            "number": number,
            "title": format!("Résolution {number}"),
            "vote_type": "simple_majority",
        }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::CREATED);
    resp.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap()
}

async fn create_document(server: &TestServer, auth: &str, building_id: Uuid) -> Uuid {
    let bucket = format!("laruche-{}", building_id.simple());
    let resp = server
        .post("/api/v1/documents")
        .add_header("Authorization", auth)
        .json(&json!({
            "key": format!("uploads/{}.pdf", Uuid::new_v4()),
            "bucket": bucket,
            "original_name": "convocation.pdf",
            "mime_type": "application/pdf",
            "size_bytes": 1024,
            "visible_roles": ["manager"],
            "category": "autre",
        }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::CREATED);
    resp.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap()
}

/// Full convocation path; leaves the meeting in `called` status.
async fn convene(server: &TestServer, auth: &str, building_id: Uuid, meeting_id: Uuid) {
    let document_id = create_document(server, auth, building_id).await;

    let attach = server
        .post(&format!("/api/v1/meetings/{meeting_id}/documents"))
        .add_header("Authorization", auth)
        .json(&json!({ "document_id": document_id, "kind": "notice" }))
        .await;
    assert_eq!(attach.status_code(), StatusCode::NO_CONTENT);

    let call = server
        .post(&format!("/api/v1/meetings/{meeting_id}/call"))
        .add_header("Authorization", auth)
        .json(&json!({ "channel": "email" }))
        .await;
    assert_eq!(call.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn draft_meeting_delete_cascades_and_returns_404_afterwards() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, Role::Manager, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let meeting_id = create_meeting(&server, &auth).await;
    let r1 = create_resolution(&server, &auth, meeting_id, 1).await;
    let document_id = create_document(&server, &auth, f.building_id).await;
    let attach = server
        .post(&format!("/api/v1/meetings/{meeting_id}/documents"))
        .add_header("Authorization", &auth)
        .json(&json!({ "document_id": document_id, "kind": "attachment" }))
        .await;
    assert_eq!(attach.status_code(), StatusCode::NO_CONTENT);

    let delete = server
        .delete(&format!("/api/v1/meetings/{meeting_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(delete.status_code(), StatusCode::NO_CONTENT);

    let get = server
        .get(&format!("/api/v1/meetings/{meeting_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(get.status_code(), StatusCode::NOT_FOUND);

    let resolution_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM resolution WHERE id = $1")
        .bind(r1)
        .fetch_one(&pool)
        .await
        .expect("resolution count");
    assert_eq!(resolution_count, 0);

    let document_link_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM general_meeting_document WHERE meeting_id = $1")
            .bind(meeting_id)
            .fetch_one(&pool)
            .await
            .expect("document link count");
    assert_eq!(document_link_count, 0);

    let meeting_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM general_meeting WHERE id = $1")
            .bind(meeting_id)
            .fetch_one(&pool)
            .await
            .expect("meeting count");
    assert_eq!(meeting_count, 0);

    // Double DELETE -> 404 (idempotent from the client's perspective).
    let again = server
        .delete(&format!("/api/v1/meetings/{meeting_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(again.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_refused_once_convened_and_for_other_building() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let other_building_id = setup_other_building(&pool, f.manager_id).await;
    let token = token_for(f.manager_id, f.building_id, Role::Manager, &cfg);
    let server = common::test_server_with_pool(cfg.clone(), pool.clone()).await;
    let auth = format!("Bearer {token}");

    // Non-draft meeting -> 404, nothing deleted.
    let meeting_id = create_meeting(&server, &auth).await;
    convene(&server, &auth, f.building_id, meeting_id).await;
    let delete = server
        .delete(&format!("/api/v1/meetings/{meeting_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(delete.status_code(), StatusCode::NOT_FOUND);
    let meeting_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM general_meeting WHERE id = $1")
            .bind(meeting_id)
            .fetch_one(&pool)
            .await
            .expect("meeting count");
    assert_eq!(meeting_count, 1);

    // Meeting from another building -> 404 (tenant isolation).
    let other_token = token_for(f.manager_id, other_building_id, Role::Manager, &cfg);
    let other_auth = format!("Bearer {other_token}");
    let cross_delete = server
        .delete(&format!("/api/v1/meetings/{meeting_id}"))
        .add_header("Authorization", &other_auth)
        .await;
    assert_eq!(cross_delete.status_code(), StatusCode::NOT_FOUND);
    let meeting_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM general_meeting WHERE id = $1")
            .bind(meeting_id)
            .fetch_one(&pool)
            .await
            .expect("meeting count");
    assert_eq!(meeting_count, 1);
}

#[tokio::test]
async fn delete_requires_manager_role() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, Role::Manager, &cfg);
    let server = common::test_server_with_pool(cfg.clone(), pool.clone()).await;
    let auth = format!("Bearer {token}");

    let meeting_id = create_meeting(&server, &auth).await;

    let coowner_token = token_for(Uuid::new_v4(), f.building_id, Role::CoOwner, &cfg);
    let resp = server
        .delete(&format!("/api/v1/meetings/{meeting_id}"))
        .add_header("Authorization", format!("Bearer {coowner_token}"))
        .await;
    assert_eq!(resp.status_code(), StatusCode::FORBIDDEN);

    let meeting_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM general_meeting WHERE id = $1")
            .bind(meeting_id)
            .fetch_one(&pool)
            .await
            .expect("meeting count");
    assert_eq!(meeting_count, 1);
}
