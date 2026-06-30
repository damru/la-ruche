// Integration tests — EP-002-S10: judicial proceedings register.
// AC1: CRUD proceedings (manager-only). AC2: feeds état daté (list_ongoing covered in unit tests).
// Requires a PostgreSQL test database (DATABASE_URL) + MinIO.
// Run with: cargo test --test judicial_proceedings_test

mod common;

use axum::http::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

struct Ctx {
    building_id: Uuid,
    manager_jwt: String,
    coowner_jwt: String,
}

async fn setup() -> Ctx {
    let (cfg, pool) = common::test_db().await;

    let building_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total) \
         VALUES ($1, 'JPTestBuilding', '1 rue JP', '75001', 'Paris', 1000)",
    )
    .bind(building_id)
    .execute(&pool)
    .await
    .expect("building");

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
        building_id,
        manager_jwt,
        coowner_jwt,
    }
}

async fn insert_user(
    pool: &sqlx::PgPool,
    cfg: &laruche::config::AppConfig,
    building_id: Uuid,
    role: &str,
) -> Uuid {
    let user_id = Uuid::new_v4();
    let email = format!("jp-{role}-{}@test.local", Uuid::new_v4());
    let password_hash =
        laruche::auth::password::hash_password("MotDePasse123!", &cfg.auth).expect("hash");
    sqlx::query(
        "INSERT INTO user_account (id, email, last_name, first_name, password_hash) \
         VALUES ($1, $2, 'JP', 'User', $3)",
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
async fn judicial_proceedings_requires_auth() {
    let server = common::test_server().await;
    let resp = server.get("/api/v1/judicial-proceedings").await;
    assert_eq!(resp.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn manager_can_crud_proceeding() {
    let ctx = setup().await;
    let server = common::test_server().await;

    // create
    let created = server
        .post("/api/v1/judicial-proceedings")
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .json(&json!({
            "proceeding_type": "debt_recovery",
            "subject": "Recouvrement charges lot 12",
            "claimant": "Syndicat des copropriétaires",
            "defendant": "M. Dupont",
            "jurisdiction": "Tribunal judiciaire de Paris",
            "case_reference": "RG 24/12345",
            "opened_at": "2026-01-15"
        }))
        .await;
    assert_eq!(created.status_code(), StatusCode::CREATED);
    let body: Value = created.json();
    let id = body["id"].as_str().expect("id").to_string();
    assert_eq!(body["status"], "ongoing");

    // list scoped to building
    let listed = server
        .get("/api/v1/judicial-proceedings")
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(listed.status_code(), StatusCode::OK);
    let arr: Value = listed.json();
    assert_eq!(arr.as_array().expect("array").len(), 1);

    // update → close
    let updated = server
        .put(&format!("/api/v1/judicial-proceedings/{id}"))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .json(&json!({
            "proceeding_type": "debt_recovery",
            "status": "closed",
            "subject": "Recouvrement charges lot 12",
            "claimant": "Syndicat des copropriétaires",
            "defendant": "M. Dupont",
            "jurisdiction": "Tribunal judiciaire de Paris",
            "case_reference": "RG 24/12345",
            "opened_at": "2026-01-15",
            "closed_at": "2026-05-01",
            "outcome": "Condamnation au paiement"
        }))
        .await;
    assert_eq!(updated.status_code(), StatusCode::OK);
    let ubody: Value = updated.json();
    assert_eq!(ubody["status"], "closed");

    // delete
    let deleted = server
        .delete(&format!("/api/v1/judicial-proceedings/{id}"))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(deleted.status_code(), StatusCode::NO_CONTENT);

    let _ = ctx.building_id;
}

#[tokio::test]
async fn co_owner_cannot_create_proceeding() {
    let ctx = setup().await;
    let server = common::test_server().await;
    let resp = server
        .post("/api/v1/judicial-proceedings")
        .add_header("Authorization", format!("Bearer {}", ctx.coowner_jwt))
        .json(&json!({
            "proceeding_type": "other",
            "subject": "Litige",
            "claimant": "Syndicat",
            "defendant": "Tiers",
            "opened_at": "2026-01-15"
        }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn closed_status_requires_closed_at() {
    let ctx = setup().await;
    let server = common::test_server().await;

    let created = server
        .post("/api/v1/judicial-proceedings")
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .json(&json!({
            "proceeding_type": "liability",
            "subject": "Dégât",
            "claimant": "Syndicat",
            "defendant": "Assureur",
            "opened_at": "2026-01-15"
        }))
        .await;
    let id = created.json::<Value>()["id"].as_str().unwrap().to_string();

    let resp = server
        .put(&format!("/api/v1/judicial-proceedings/{id}"))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .json(&json!({
            "proceeding_type": "liability",
            "status": "closed",
            "subject": "Dégât",
            "claimant": "Syndicat",
            "defendant": "Assureur",
            "opened_at": "2026-01-15",
            "closed_at": null,
            "outcome": null
        }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
}
