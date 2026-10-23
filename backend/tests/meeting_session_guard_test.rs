// EP-013-S11 — session guard: votes and attendance are rejected unless the
// meeting is `ongoing`, and stay frozen once `closed` (LEGAL.md §11.1).
//
// AC1: vote + attendance on `draft` / `called` → 400 errors.meetings.not_ongoing.
// AC2: both succeed while `ongoing` (non-regression).
// AC3: after `/close`, vote + attendance → 400 and existing tallies are unchanged.
// AC4: unknown meeting / other building → 404 (unchanged).

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use laruche::domain::Role;
use serde_json::{Value, json};
use uuid::Uuid;

struct Fixture {
    building_id: Uuid,
    manager_id: Uuid,
    unit_id: Uuid,
}

async fn setup(pool: &sqlx::PgPool) -> Fixture {
    let building_id = Uuid::new_v4();
    let manager_id = Uuid::new_v4();
    let unit_id = Uuid::new_v4();
    let key_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total)
         VALUES ($1, 'GuardBuilding', '1 rue Garde', '75001', 'Paris', 1000)",
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
    .bind(format!("guard-mgr-{}@test.local", Uuid::new_v4()))
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

    sqlx::query(
        "INSERT INTO unit (id, building_id, unit_number, unit_type)
         VALUES ($1, $2, 'G-101', 'apartment'::unit_type)",
    )
    .bind(unit_id)
    .bind(building_id)
    .execute(pool)
    .await
    .expect("unit");

    sqlx::query(
        "INSERT INTO allocation_key (id, building_id, name, key_type)
         VALUES ($1, $2, 'Charges générales', 'general'::allocation_key_type)",
    )
    .bind(key_id)
    .bind(building_id)
    .execute(pool)
    .await
    .expect("allocation_key");

    sqlx::query(
        "INSERT INTO unit_allocation_key (unit_id, key_id, share_value)
         VALUES ($1, $2, 1000)",
    )
    .bind(unit_id)
    .bind(key_id)
    .execute(pool)
    .await
    .expect("unit_allocation_key");

    Fixture {
        building_id,
        manager_id,
        unit_id,
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

async fn create_resolution(server: &TestServer, auth: &str, meeting_id: Uuid) -> Uuid {
    let resp = server
        .post(&format!("/api/v1/meetings/{meeting_id}/resolutions"))
        .add_header("Authorization", auth)
        .json(&json!({
            "number": 1,
            "title": "Résolution 1",
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

fn vote_payload(unit_id: Uuid) -> Value {
    json!({ "unit_id": unit_id, "position": "in_favor" })
}

fn attendance_payload(unit_id: Uuid) -> Value {
    json!({
        "entries": [{ "unit_id": unit_id, "mode": "present", "representative_name": null }]
    })
}

async fn cast_vote(
    server: &TestServer,
    auth: &str,
    meeting_id: Uuid,
    resolution_id: Uuid,
    unit_id: Uuid,
) -> axum_test::TestResponse {
    server
        .post(&format!(
            "/api/v1/meetings/{meeting_id}/resolutions/{resolution_id}/vote"
        ))
        .add_header("Authorization", auth)
        .json(&vote_payload(unit_id))
        .await
}

async fn save_attendance(
    server: &TestServer,
    auth: &str,
    meeting_id: Uuid,
    unit_id: Uuid,
) -> axum_test::TestResponse {
    server
        .put(&format!("/api/v1/meetings/{meeting_id}/attendance"))
        .add_header("Authorization", auth)
        .json(&attendance_payload(unit_id))
        .await
}

fn assert_not_ongoing(resp: &axum_test::TestResponse) {
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
        resp.json::<Value>()["message"],
        "errors.meetings.not_ongoing"
    );
}

async fn set_status(pool: &sqlx::PgPool, meeting_id: Uuid, status: &str) {
    sqlx::query("UPDATE general_meeting SET status = $2::meeting_status WHERE id = $1")
        .bind(meeting_id)
        .bind(status)
        .execute(pool)
        .await
        .expect("set status");
}

#[tokio::test]
async fn vote_and_attendance_rejected_outside_ongoing_and_frozen_after_close() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let meeting_id = create_meeting(&server, &auth).await;
    let resolution_id = create_resolution(&server, &auth, meeting_id).await;

    // draft: both writes refused.
    assert_not_ongoing(&cast_vote(&server, &auth, meeting_id, resolution_id, f.unit_id).await);
    assert_not_ongoing(&save_attendance(&server, &auth, meeting_id, f.unit_id).await);

    // called: still refused.
    set_status(&pool, meeting_id, "called").await;
    assert_not_ongoing(&cast_vote(&server, &auth, meeting_id, resolution_id, f.unit_id).await);
    assert_not_ongoing(&save_attendance(&server, &auth, meeting_id, f.unit_id).await);

    // ongoing: both writes succeed (non-regression).
    let start = server
        .post(&format!("/api/v1/meetings/{meeting_id}/start"))
        .add_header("Authorization", &auth)
        .json(&json!({
            "president_name": "Président",
            "secretary_name": "Secrétaire",
            "scrutineers": [],
        }))
        .await;
    assert_eq!(start.status_code(), StatusCode::NO_CONTENT);

    let vote = cast_vote(&server, &auth, meeting_id, resolution_id, f.unit_id).await;
    assert_eq!(vote.status_code(), StatusCode::NO_CONTENT);
    let attendance = save_attendance(&server, &auth, meeting_id, f.unit_id).await;
    assert_eq!(attendance.status_code(), StatusCode::NO_CONTENT);

    // Snapshot the recorded state before closing.
    let attendance_before = server
        .get(&format!("/api/v1/meetings/{meeting_id}/attendance"))
        .add_header("Authorization", &auth)
        .await
        .json::<Value>();
    let votes_before = server
        .get(&format!(
            "/api/v1/meetings/{meeting_id}/resolutions/{resolution_id}/votes"
        ))
        .add_header("Authorization", &auth)
        .await
        .json::<Vec<Value>>();

    // closed: both writes refused, frozen forever.
    let close = server
        .post(&format!("/api/v1/meetings/{meeting_id}/close"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(close.status_code(), StatusCode::NO_CONTENT);

    assert_not_ongoing(&cast_vote(&server, &auth, meeting_id, resolution_id, f.unit_id).await);
    assert_not_ongoing(&save_attendance(&server, &auth, meeting_id, f.unit_id).await);

    // Existing tallies are unchanged.
    let attendance_after = server
        .get(&format!("/api/v1/meetings/{meeting_id}/attendance"))
        .add_header("Authorization", &auth)
        .await
        .json::<Value>();
    assert_eq!(attendance_after, attendance_before);
    let votes_after = server
        .get(&format!(
            "/api/v1/meetings/{meeting_id}/resolutions/{resolution_id}/votes"
        ))
        .add_header("Authorization", &auth)
        .await
        .json::<Vec<Value>>();
    assert_eq!(votes_after, votes_before);
    assert_eq!(votes_after.len(), 1);
}

#[tokio::test]
async fn vote_on_unknown_meeting_is_not_found() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let meeting_id = create_meeting(&server, &auth).await;
    let resolution_id = create_resolution(&server, &auth, meeting_id).await;

    // Meeting of another building → 404, not 400.
    let resp = cast_vote(&server, &auth, Uuid::new_v4(), resolution_id, f.unit_id).await;
    assert_eq!(resp.status_code(), StatusCode::NOT_FOUND);

    // Attendance on unknown meeting → 404.
    let resp = save_attendance(&server, &auth, Uuid::new_v4(), f.unit_id).await;
    assert_eq!(resp.status_code(), StatusCode::NOT_FOUND);
}
