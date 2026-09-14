// Integration tests — EP-002-S06: état daté (LEGAL.md §7.1, 15-day deadline).
// AC1: request tracking (J+15 due date, status). AC2: generation freezes a
// 3-part regulatory document (snapshot). AC3: regeneration allowed until
// `sent`; manager-only writes.
// Requires a PostgreSQL test database (DATABASE_URL) + MinIO.
// Run with: cargo test --test etat_date_test

mod common;

use axum::http::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

struct Ctx {
    manager_jwt: String,
    coowner_jwt: String,
    unit_id: Uuid,
}

async fn setup() -> Ctx {
    let (cfg, pool) = common::test_db().await;

    let building_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total) \
         VALUES ($1, 'EDTestBuilding', '1 rue ED', '75001', 'Paris', 1000)",
    )
    .bind(building_id)
    .execute(&pool)
    .await
    .expect("building");

    let unit_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO unit (id, building_id, unit_number, unit_type) \
         VALUES ($1, $2, 'ED-101', 'apartment'::unit_type)",
    )
    .bind(unit_id)
    .bind(building_id)
    .execute(&pool)
    .await
    .expect("unit");

    let manager_id = insert_user(&pool, &cfg, building_id, "manager").await;
    let coowner_id = insert_user(&pool, &cfg, building_id, "co_owner").await;

    let manager_jwt = laruche::auth::jwt::generate_access_token(
        manager_id,
        building_id,
        vec![laruche::domain::Role::Manager],
        &cfg.auth.jwt_secret,
        cfg.auth.access_token_ttl_secs,
    )
    .expect("manager token");

    let coowner_jwt = laruche::auth::jwt::generate_access_token(
        coowner_id,
        building_id,
        vec![laruche::domain::Role::CoOwner],
        &cfg.auth.jwt_secret,
        cfg.auth.access_token_ttl_secs,
    )
    .expect("coowner token");

    Ctx {
        manager_jwt,
        coowner_jwt,
        unit_id,
    }
}

async fn insert_user(
    pool: &sqlx::PgPool,
    cfg: &laruche::config::AppConfig,
    building_id: Uuid,
    role: &str,
) -> Uuid {
    let user_id = Uuid::new_v4();
    let email = format!("ed-{role}-{}@test.local", Uuid::new_v4());
    let password_hash =
        laruche::auth::password::hash_password("MotDePasse123!", &cfg.auth).expect("hash");
    sqlx::query(
        "INSERT INTO user_account (id, email, last_name, first_name, password_hash) \
         VALUES ($1, $2, 'ED', 'User', $3)",
    )
    .bind(user_id)
    .bind(&email)
    .bind(&password_hash)
    .execute(pool)
    .await
    .expect("user");
    sqlx::query(
        "INSERT INTO user_building (user_id, building_id, roles) \
         VALUES ($1, $2, ARRAY[$3]::user_role[])",
    )
    .bind(user_id)
    .bind(building_id)
    .bind(role)
    .execute(pool)
    .await
    .expect("role");
    user_id
}

#[tokio::test]
async fn etat_date_requires_auth() {
    let server = common::test_server().await;
    let resp = server.get("/api/v1/etat-date-requests").await;
    assert_eq!(resp.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn co_owner_cannot_create_request() {
    let ctx = setup().await;
    let server = common::test_server().await;
    let resp = server
        .post(&format!("/api/v1/units/{}/etat-date-requests", ctx.unit_id))
        .add_header("Authorization", format!("Bearer {}", ctx.coowner_jwt))
        .json(&json!({
            "requested_at": "2026-01-01",
            "requester_name": "Étude Notaire X"
        }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn manager_can_request_generate_and_read_snapshot() {
    let ctx = setup().await;
    let server = common::test_server().await;

    // create the request → J+15 due date computed by domain
    let created = server
        .post(&format!("/api/v1/units/{}/etat-date-requests", ctx.unit_id))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .json(&json!({
            "requested_at": "2026-01-01",
            "requester_name": "Étude Notaire X"
        }))
        .await;
    assert_eq!(
        created.status_code(),
        StatusCode::CREATED,
        "body: {}",
        created.text()
    );
    let body: Value = created.json();
    let id = body["id"].as_str().expect("id").to_string();
    assert_eq!(body["due_date"], "2026-01-16");
    assert_eq!(body["status"], "requested");

    // not yet generated → 409 on GET
    let not_generated = server
        .get(&format!("/api/v1/etat-date-requests/{id}"))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(not_generated.status_code(), StatusCode::CONFLICT);

    // list shows the pending request
    let listed = server
        .get("/api/v1/etat-date-requests")
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(listed.status_code(), StatusCode::OK);
    let arr: Value = listed.json();
    assert_eq!(arr.as_array().expect("array").len(), 1);

    // generate → lot vierge: zeroed lines present, never omitted
    let generated = server
        .post(&format!("/api/v1/etat-date-requests/{id}/generate"))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(generated.status_code(), StatusCode::OK);
    let gbody: Value = generated.json();
    assert_eq!(gbody["status"], "generated");
    let snapshot = &gbody["snapshot"];
    assert_eq!(
        snapshot["partie1_sommes_dues_par_cedant"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        snapshot["emprunts_collectifs"],
        "néant — emprunts collectifs non gérés"
    );
    assert!(snapshot["procedures"].as_array().unwrap().is_empty());

    // now generated → GET returns it
    let fetched = server
        .get(&format!("/api/v1/etat-date-requests/{id}"))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(fetched.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn coowner_cannot_generate() {
    let ctx = setup().await;
    let server = common::test_server().await;

    let created = server
        .post(&format!("/api/v1/units/{}/etat-date-requests", ctx.unit_id))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .json(&json!({
            "requested_at": "2026-01-01",
            "requester_name": "Étude Notaire X"
        }))
        .await;
    let id = created.json::<Value>()["id"].as_str().unwrap().to_string();

    let resp = server
        .post(&format!("/api/v1/etat-date-requests/{id}/generate"))
        .add_header("Authorization", format!("Bearer {}", ctx.coowner_jwt))
        .await;
    assert_eq!(resp.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn future_request_date_is_rejected() {
    let ctx = setup().await;
    let server = common::test_server().await;

    let resp = server
        .post(&format!("/api/v1/units/{}/etat-date-requests", ctx.unit_id))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .json(&json!({
            "requested_at": "2999-01-01",
            "requester_name": "Étude Notaire X"
        }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn regeneration_is_refused_once_sent() {
    let ctx = setup().await;
    let (_, pool) = common::test_db().await;
    let server = common::test_server().await;

    let created = server
        .post(&format!("/api/v1/units/{}/etat-date-requests", ctx.unit_id))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .json(&json!({
            "requested_at": "2026-01-01",
            "requester_name": "Étude Notaire X"
        }))
        .await;
    let id: Uuid = created.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    let generated = server
        .post(&format!("/api/v1/etat-date-requests/{id}/generate"))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(generated.status_code(), StatusCode::OK);

    // Directly flip to 'sent' — no transmission endpoint exists yet (S07 scope).
    sqlx::query("UPDATE etat_date_request SET status = 'sent' WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .expect("mark sent");

    let regen = server
        .post(&format!("/api/v1/etat-date-requests/{id}/generate"))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(regen.status_code(), StatusCode::CONFLICT);
}
