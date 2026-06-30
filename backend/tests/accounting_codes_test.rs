// Integration tests — EP-004-S09: plan comptable copropriété mapping.
// Verifies AC1: budget_line carries an account_code; accounting codes list is accessible.
// Requires a PostgreSQL test database (DATABASE_URL).
// Run with: cargo test --test accounting_codes_test

mod common;

use axum::http::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

/// GET /accounting-codes requires authentication.
#[tokio::test]
async fn accounting_codes_requires_auth() {
    let server = common::test_server().await;
    let resp = server.get("/api/v1/accounting-codes").await;
    assert_eq!(resp.status_code(), StatusCode::UNAUTHORIZED);
}

/// Seeded accounting codes are returned as a non-empty list.
#[tokio::test]
async fn accounting_codes_list_is_nonempty() {
    dotenvy::dotenv().ok();
    let cfg = laruche::config::AppConfig::load().expect("config");
    let pool = sqlx::PgPool::connect(&cfg.db.url).await.expect("db");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations");

    let building_id = Uuid::new_v4();
    let manager_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total) \
         VALUES ($1, 'ACTestBuilding', '1 rue AC', '75001', 'Paris', 1000)",
    )
    .bind(building_id)
    .execute(&pool)
    .await
    .expect("building");

    let unique_email = format!("mgr-ac-{}@test.local", Uuid::new_v4());
    let password_hash =
        laruche::auth::password::hash_password("MotDePasse123!", &cfg.auth).expect("hash");
    sqlx::query(
        "INSERT INTO user_account (id, email, last_name, first_name, password_hash) \
         VALUES ($1, $2, 'AC', 'Mgr', $3)",
    )
    .bind(manager_id)
    .bind(&unique_email)
    .bind(&password_hash)
    .execute(&pool)
    .await
    .expect("user");

    sqlx::query(
        "INSERT INTO user_building (user_id, building_id, roles) \
         VALUES ($1, $2, ARRAY['manager']::user_role[])",
    )
    .bind(manager_id)
    .bind(building_id)
    .execute(&pool)
    .await
    .expect("role");

    let jwt = laruche::auth::jwt::generate_access_token(
        manager_id,
        building_id,
        vec![laruche::domain::Role::Manager],
        &cfg.auth.jwt_secret,
        cfg.auth.access_token_ttl_secs,
    )
    .expect("manager token");

    let server = common::test_server().await;
    let resp = server
        .get("/api/v1/accounting-codes")
        .add_header("Authorization", format!("Bearer {jwt}"))
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);

    let codes: Value = resp.json();
    let arr = codes.as_array().expect("expected array");
    assert!(
        !arr.is_empty(),
        "accounting codes must be non-empty (seeded)"
    );

    // Verify expected structure on first entry
    let first = &arr[0];
    assert!(first.get("code").is_some(), "code field required");
    assert!(first.get("label").is_some(), "label field required");
    assert!(first.get("category").is_some(), "category field required");
}

/// A budget line created with an account_code is returned with that code.
#[tokio::test]
async fn budget_line_stores_account_code() {
    dotenvy::dotenv().ok();
    let cfg = laruche::config::AppConfig::load().expect("config");
    let pool = sqlx::PgPool::connect(&cfg.db.url).await.expect("db");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations");

    let building_id = Uuid::new_v4();
    let manager_id = Uuid::new_v4();
    let key_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total) \
         VALUES ($1, 'ACLineTest', '2 rue AC', '75001', 'Paris', 1000)",
    )
    .bind(building_id)
    .execute(&pool)
    .await
    .expect("building");

    let unique_email = format!("mgr-acl-{}@test.local", Uuid::new_v4());
    let password_hash =
        laruche::auth::password::hash_password("MotDePasse123!", &cfg.auth).expect("hash");
    sqlx::query(
        "INSERT INTO user_account (id, email, last_name, first_name, password_hash) \
         VALUES ($1, $2, 'ACL', 'Mgr', $3)",
    )
    .bind(manager_id)
    .bind(&unique_email)
    .bind(&password_hash)
    .execute(&pool)
    .await
    .expect("user");

    sqlx::query(
        "INSERT INTO user_building (user_id, building_id, roles) \
         VALUES ($1, $2, ARRAY['manager']::user_role[])",
    )
    .bind(manager_id)
    .bind(building_id)
    .execute(&pool)
    .await
    .expect("role");

    sqlx::query(
        "INSERT INTO allocation_key (id, building_id, name, key_type) \
         VALUES ($1, $2, 'Charges Générales', 'general'::allocation_key_type)",
    )
    .bind(key_id)
    .bind(building_id)
    .execute(&pool)
    .await
    .expect("key");

    let budget_id: Uuid = sqlx::query_scalar(
        "INSERT INTO budget (building_id, fiscal_year, status) \
         VALUES ($1, 2026, 'draft'::budget_status) RETURNING id",
    )
    .bind(building_id)
    .fetch_one(&pool)
    .await
    .expect("budget");

    let jwt = laruche::auth::jwt::generate_access_token(
        manager_id,
        building_id,
        vec![laruche::domain::Role::Manager],
        &cfg.auth.jwt_secret,
        cfg.auth.access_token_ttl_secs,
    )
    .expect("manager token");

    let server = common::test_server().await;

    // Create a budget line with account_code '6152' (entretien immobilier)
    let create_resp = server
        .post(&format!("/api/v1/budgets/{budget_id}/lines"))
        .add_header("Authorization", format!("Bearer {jwt}"))
        .json(&json!({
            "label": "Entretien parties communes",
            "key_id": key_id,
            "budgeted_amount": 3000.00,
            "account_code": "6152"
        }))
        .await;
    assert_eq!(
        create_resp.status_code(),
        StatusCode::CREATED,
        "create line failed: {:?}",
        create_resp.text()
    );

    let line: Value = create_resp.json();
    assert_eq!(line["account_code"], "6152", "account_code must be stored");

    // List lines — account_code must be present
    let list_resp = server
        .get(&format!("/api/v1/budgets/{budget_id}/lines"))
        .add_header("Authorization", format!("Bearer {jwt}"))
        .await;
    assert_eq!(list_resp.status_code(), StatusCode::OK);
    let lines: Value = list_resp.json();
    let lines_arr = lines.as_array().expect("array");
    assert_eq!(lines_arr.len(), 1);
    assert_eq!(lines_arr[0]["account_code"], "6152");
}
