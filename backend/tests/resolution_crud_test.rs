// Integration tests for resolution CRUD and reorder.
// Agenda frozen at convocation, except the second-vote passerelle
// (art. 25-1/26-1) which may still inscribe a resolution while `ongoing`.

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

async fn create_meeting(server: &TestServer, auth: &str) -> Uuid {
    let resp = server
        .post("/api/v1/meetings")
        .add_header("Authorization", auth)
        .json(&json!({
            "meeting_type": "annual",
            "meeting_date": "2027-06-15T18:00:00Z",
            "location": "Salle commune",
            "agenda": "Approbation du budget\nTravaux de toiture",
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
            "description": "Description de test",
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

async fn list_resolutions(server: &TestServer, auth: &str, meeting_id: Uuid) -> Vec<Value> {
    server
        .get(&format!("/api/v1/meetings/{meeting_id}/resolutions"))
        .add_header("Authorization", auth)
        .await
        .json::<Vec<Value>>()
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
async fn draft_resolution_delete_and_reorder_flow() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let meeting_id = create_meeting(&server, &auth).await;
    let r1 = create_resolution(&server, &auth, meeting_id, 1).await;
    let r2 = create_resolution(&server, &auth, meeting_id, 2).await;
    let r3 = create_resolution(&server, &auth, meeting_id, 3).await;

    let delete = server
        .delete(&format!("/api/v1/meetings/{meeting_id}/resolutions/{r2}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(delete.status_code(), StatusCode::NO_CONTENT);

    let after_delete = list_resolutions(&server, &auth, meeting_id).await;
    assert_eq!(after_delete.len(), 2);
    assert_eq!(after_delete[0]["id"].as_str().unwrap(), r1.to_string());
    assert_eq!(after_delete[1]["id"].as_str().unwrap(), r3.to_string());

    let reorder = server
        .post(&format!(
            "/api/v1/meetings/{meeting_id}/resolutions/reorder"
        ))
        .add_header("Authorization", &auth)
        .json(&json!({ "ordered_ids": [r3, r1] }))
        .await;
    assert_eq!(reorder.status_code(), StatusCode::OK);
    let reordered = reorder.json::<Vec<Value>>();
    assert_eq!(reordered.len(), 2);
    assert_eq!(reordered[0]["id"].as_str().unwrap(), r3.to_string());
    assert_eq!(reordered[0]["number"], 1);
    assert_eq!(reordered[1]["id"].as_str().unwrap(), r1.to_string());
    assert_eq!(reordered[1]["number"], 2);

    let listed = list_resolutions(&server, &auth, meeting_id).await;
    assert_eq!(listed[0]["id"].as_str().unwrap(), r3.to_string());
    assert_eq!(listed[0]["number"], 1);
    assert_eq!(listed[1]["id"].as_str().unwrap(), r1.to_string());
    assert_eq!(listed[1]["number"], 2);

    let missing = server
        .post(&format!(
            "/api/v1/meetings/{meeting_id}/resolutions/reorder"
        ))
        .add_header("Authorization", &auth)
        .json(&json!({ "ordered_ids": [r1] }))
        .await;
    assert_eq!(missing.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
        missing.json::<Value>()["message"],
        "errors.resolutions.reorder_mismatch"
    );

    let duplicated = server
        .post(&format!(
            "/api/v1/meetings/{meeting_id}/resolutions/reorder"
        ))
        .add_header("Authorization", &auth)
        .json(&json!({ "ordered_ids": [r1, r1] }))
        .await;
    assert_eq!(duplicated.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
        duplicated.json::<Value>()["message"],
        "errors.resolutions.reorder_mismatch"
    );
}

#[tokio::test]
async fn structural_mutations_refused_after_convocation() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let meeting_id = create_meeting(&server, &auth).await;
    let r1 = create_resolution(&server, &auth, meeting_id, 1).await;
    convene(&server, &auth, f.building_id, meeting_id).await;

    let delete = server
        .delete(&format!("/api/v1/meetings/{meeting_id}/resolutions/{r1}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(delete.status_code(), StatusCode::NOT_FOUND);

    let reorder = server
        .post(&format!(
            "/api/v1/meetings/{meeting_id}/resolutions/reorder"
        ))
        .add_header("Authorization", &auth)
        .json(&json!({ "ordered_ids": [r1] }))
        .await;
    assert_eq!(reorder.status_code(), StatusCode::CONFLICT);
    assert_eq!(
        reorder.json::<Value>()["message"],
        "errors.meetings.not_in_draft_status"
    );

    let create = server
        .post(&format!("/api/v1/meetings/{meeting_id}/resolutions"))
        .add_header("Authorization", &auth)
        .json(&json!({
            "number": 2,
            "title": "Résolution tardive",
            "vote_type": "simple_majority",
        }))
        .await;
    assert_eq!(create.status_code(), StatusCode::CONFLICT);
    assert_eq!(
        create.json::<Value>()["message"],
        "errors.meetings.meeting_not_mutable"
    );
}

#[tokio::test]
async fn ongoing_session_keeps_vote_patch_and_passerelle() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let meeting_id = create_meeting(&server, &auth).await;
    let r1 = create_resolution(&server, &auth, meeting_id, 1).await;
    convene(&server, &auth, f.building_id, meeting_id).await;

    let start = server
        .post(&format!("/api/v1/meetings/{meeting_id}/start"))
        .add_header("Authorization", &auth)
        .json(&json!({
            "president_name": "Présidente",
            "secretary_name": "Secrétaire",
            "scrutineers": [],
        }))
        .await;
    assert_eq!(start.status_code(), StatusCode::NO_CONTENT);

    let patch = server
        .patch(&format!("/api/v1/meetings/{meeting_id}/resolutions/{r1}"))
        .add_header("Authorization", &auth)
        .json(&json!({ "description": "Mise à jour en séance" }))
        .await;
    assert_eq!(patch.status_code(), StatusCode::OK);
    assert_eq!(
        patch.json::<Value>()["description"],
        "Mise à jour en séance"
    );

    let passerelle = server
        .post(&format!("/api/v1/meetings/{meeting_id}/resolutions"))
        .add_header("Authorization", &auth)
        .json(&json!({
            "number": 2,
            "title": "Re-délibération (Art. 24)",
            "vote_type": "simple_majority",
        }))
        .await;
    assert_eq!(passerelle.status_code(), StatusCode::CREATED);

    let delete = server
        .delete(&format!("/api/v1/meetings/{meeting_id}/resolutions/{r1}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(delete.status_code(), StatusCode::NOT_FOUND);
}
