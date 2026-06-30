// Integration tests — EP-002-S11: governance register (registre des mandats).
// AC1: CRUD mandates (manager-only). AC2: single active syndic invariant enforced.
// Requires a PostgreSQL test database (DATABASE_URL) + MinIO.
// Run with: cargo test --test governance_register_test

mod common;

use axum::http::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

struct Ctx {
    manager_jwt: String,
    coowner_jwt: String,
}

async fn setup() -> Ctx {
    dotenvy::dotenv().ok();
    let cfg = laruche::config::AppConfig::load().expect("config");
    let pool = sqlx::PgPool::connect(&cfg.db.url).await.expect("db");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations");

    let building_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total) \
         VALUES ($1, 'GovTestBuilding', '1 rue Gov', '75001', 'Paris', 1000)",
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
    let email = format!("gov-{role}-{}@test.local", Uuid::new_v4());
    let password_hash =
        laruche::auth::password::hash_password("MotDePasse123!", &cfg.auth).expect("hash");
    sqlx::query(
        "INSERT INTO user_account (id, email, last_name, first_name, password_hash) \
         VALUES ($1, $2, 'Gov', 'User', $3)",
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
async fn governance_mandates_requires_auth() {
    let server = common::test_server().await;
    let resp = server.get("/api/v1/governance/mandates").await;
    assert_eq!(resp.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn manager_can_crud_mandate() {
    let ctx = setup().await;
    let server = common::test_server().await;

    let created = server
        .post("/api/v1/governance/mandates")
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .json(&json!({
            "role": "council_member",
            "holder_name": "Mme Bernard",
            "start_date": "2026-01-01"
        }))
        .await;
    assert_eq!(created.status_code(), StatusCode::CREATED);
    let body: Value = created.json();
    let id = body["id"].as_str().expect("id").to_string();
    assert_eq!(body["role"], "council_member");

    let listed = server
        .get("/api/v1/governance/mandates")
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(listed.status_code(), StatusCode::OK);
    assert_eq!(listed.json::<Value>().as_array().expect("array").len(), 1);

    let updated = server
        .put(&format!("/api/v1/governance/mandates/{id}"))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .json(&json!({
            "role": "council_president",
            "holder_name": "Mme Bernard",
            "start_date": "2026-01-01",
            "end_date": "2026-12-31"
        }))
        .await;
    assert_eq!(updated.status_code(), StatusCode::OK);
    assert_eq!(updated.json::<Value>()["role"], "council_president");

    let deleted = server
        .delete(&format!("/api/v1/governance/mandates/{id}"))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(deleted.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn co_owner_cannot_create_mandate() {
    let ctx = setup().await;
    let server = common::test_server().await;
    let resp = server
        .post("/api/v1/governance/mandates")
        .add_header("Authorization", format!("Bearer {}", ctx.coowner_jwt))
        .json(&json!({
            "role": "syndic",
            "holder_name": "M. Petit",
            "start_date": "2026-01-01"
        }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn overlapping_active_syndic_is_rejected() {
    let ctx = setup().await;
    let server = common::test_server().await;

    let first = server
        .post("/api/v1/governance/mandates")
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .json(&json!({
            "role": "syndic",
            "holder_name": "M. Premier",
            "start_date": "2026-01-01"
        }))
        .await;
    assert_eq!(first.status_code(), StatusCode::CREATED);

    // Second open-ended syndic mandate overlaps the first → rejected.
    let second = server
        .post("/api/v1/governance/mandates")
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .json(&json!({
            "role": "syndic",
            "holder_name": "M. Second",
            "start_date": "2026-06-01"
        }))
        .await;
    assert_eq!(second.status_code(), StatusCode::BAD_REQUEST);
}
