// Integration tests — EP-002-S07: pré-état daté (LEGAL.md §7.2, art. L721-2 CCH).
// AC1: GET /units/{id}/pre-etat-date returns the L721-2 synthesis directly
// (no request/workflow, no snapshot). AC2: 404 for unknown lot, 403 for
// non-manager role. AC3: lot without calls still returns 200 with zeroed
// rubriques.
// Requires a PostgreSQL test database (DATABASE_URL) + S3-compatible storage.
// Run with: cargo test --test pre_etat_date_test

mod common;

use axum::http::StatusCode;
use serde_json::Value;
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
         VALUES ($1, 'PEDTestBuilding', '1 rue PED', '75001', 'Paris', 1000)",
    )
    .bind(building_id)
    .execute(&pool)
    .await
    .expect("building");

    let unit_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO unit (id, building_id, unit_number, unit_type) \
         VALUES ($1, $2, 'PED-101', 'apartment'::unit_type)",
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
    let email = format!("ped-{role}-{}@test.local", Uuid::new_v4());
    let password_hash =
        laruche::auth::password::hash_password("MotDePasse123!", &cfg.auth).expect("hash");
    sqlx::query(
        "INSERT INTO user_account (id, email, last_name, first_name, password_hash) \
         VALUES ($1, $2, 'PED', 'User', $3)",
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
async fn pre_etat_date_requires_auth() {
    let server = common::test_server().await;
    let resp = server
        .get(&format!("/api/v1/units/{}/pre-etat-date", Uuid::new_v4()))
        .await;
    assert_eq!(resp.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn coowner_cannot_read_pre_etat_date() {
    let ctx = setup().await;
    let server = common::test_server().await;
    let resp = server
        .get(&format!("/api/v1/units/{}/pre-etat-date", ctx.unit_id))
        .add_header("Authorization", format!("Bearer {}", ctx.coowner_jwt))
        .await;
    assert_eq!(resp.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn unknown_unit_returns_404() {
    let ctx = setup().await;
    let server = common::test_server().await;
    let resp = server
        .get(&format!("/api/v1/units/{}/pre-etat-date", Uuid::new_v4()))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(resp.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn manager_reads_pre_etat_date_lot_vierge() {
    let ctx = setup().await;
    let server = common::test_server().await;

    let resp = server
        .get(&format!("/api/v1/units/{}/pre-etat-date", ctx.unit_id))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK, "body: {}", resp.text());

    let body: Value = resp.json();
    // Lot vierge: no exercice recorded → rubriques explicitly null, never omitted.
    assert!(body["charges_courantes_n1"].is_null());
    assert!(body["charges_courantes_n2"].is_null());
    assert!(body["dette_fournisseurs"].is_null());
    assert_eq!(body["sommes_dues_par_cedant"], "0");
    assert_eq!(body["impayes_globaux_syndicat"], "0");
    assert_eq!(body["procedures_en_cours"], 0);
    // No workflow: calling twice returns the current snapshot each time (no
    // stored state, reproducible at will).
    let resp2 = server
        .get(&format!("/api/v1/units/{}/pre-etat-date", ctx.unit_id))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(resp2.status_code(), StatusCode::OK);
}
