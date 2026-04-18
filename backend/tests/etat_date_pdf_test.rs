// Integration tests — EP-002-S06/S07: regulatory PDF export (ADR-011).
// AC: the frozen état daté snapshot and the on-demand pré-état daté are
// rendered server-side as `application/pdf` attachments; access mirrors the
// JSON endpoints (état daté: any authenticated member; pré-état daté: manager).
// Requires a PostgreSQL test database (embedded or DATABASE_URL).
// Run with: cargo test --test etat_date_pdf_test

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
         VALUES ($1, 'PDF Résidence', '1 rue PDF', '75011', 'Paris', 1000)",
    )
    .bind(building_id)
    .execute(&pool)
    .await
    .expect("building");

    let unit_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO unit (id, building_id, unit_number, unit_type) \
         VALUES ($1, $2, 'PDF-101', 'apartment'::unit_type)",
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
    let email = format!("pdf-{role}-{}@test.local", Uuid::new_v4());
    let password_hash =
        laruche::auth::password::hash_password("MotDePasse123!", &cfg.auth).expect("hash");
    sqlx::query(
        "INSERT INTO user_account (id, email, last_name, first_name, password_hash) \
         VALUES ($1, $2, 'PDF', 'User', $3)",
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

async fn create_and_generate(server: &axum_test::TestServer, ctx: &Ctx) -> String {
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
        "{}",
        created.text()
    );
    let id = created.json::<Value>()["id"].as_str().unwrap().to_string();

    let generated = server
        .post(&format!("/api/v1/etat-date-requests/{id}/generate"))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(
        generated.status_code(),
        StatusCode::OK,
        "{}",
        generated.text()
    );
    id
}

#[tokio::test]
async fn etat_date_pdf_requires_generation() {
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

    let res = server
        .get(&format!("/api/v1/etat-date-requests/{id}/pdf"))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(res.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn etat_date_pdf_is_served_as_application_pdf() {
    let ctx = setup().await;
    let server = common::test_server().await;
    let id = create_and_generate(&server, &ctx).await;

    let res = server
        .get(&format!("/api/v1/etat-date-requests/{id}/pdf"))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(res.status_code(), StatusCode::OK);
    assert_eq!(
        res.headers().get("content-type").unwrap(),
        "application/pdf"
    );
    assert!(
        res.headers()
            .get("content-disposition")
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("attachment; filename=\"etat-date.pdf\"")
    );
    // axum_test decodes text via UTF-8; PDF is binary — assert via bytes length.
    assert!(res.as_bytes().len() > 4);
    assert_eq!(&res.as_bytes()[..4], b"%PDF");
}

#[tokio::test]
async fn pre_etat_date_pdf_is_manager_only() {
    let ctx = setup().await;
    let server = common::test_server().await;

    let manager_res = server
        .get(&format!("/api/v1/units/{}/pre-etat-date/pdf", ctx.unit_id))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(manager_res.status_code(), StatusCode::OK);
    assert_eq!(
        manager_res.headers().get("content-type").unwrap(),
        "application/pdf"
    );
    assert_eq!(&manager_res.as_bytes()[..4], b"%PDF");

    let coowner_res = server
        .get(&format!("/api/v1/units/{}/pre-etat-date/pdf", ctx.unit_id))
        .add_header("Authorization", format!("Bearer {}", ctx.coowner_jwt))
        .await;
    assert_eq!(coowner_res.status_code(), StatusCode::FORBIDDEN);
}
