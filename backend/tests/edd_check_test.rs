// EP-001-S09: EDD coherence check (LEGAL.md §1.2).
// DB-level test for edd_reference create/get_latest/list_shares/list_current_shares,
// plus API E2E for POST /building/edd-references and GET /building/edd-check.

mod common;

use axum::http::StatusCode;
use rust_decimal::Decimal;
use serde_json::{Value, json};
use uuid::Uuid;

#[tokio::test]
async fn edd_reference_create_and_get_latest_roundtrip() {
    let (_cfg, pool) = common::test_db().await;

    let building_id = Uuid::new_v4();
    let unit_id = Uuid::new_v4();
    let key_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let unit_number = format!("E-{}", &Uuid::new_v4().simple().to_string()[..8]);
    let key_name = format!("EDD key {}", &Uuid::new_v4().simple().to_string()[..8]);
    let email = format!("edd-{}@test.fr", &Uuid::new_v4().simple().to_string()[..8]);

    sqlx::query(
        r#"INSERT INTO building (id, name, address_line1, postal_code, city, shares_total)
           VALUES ($1, 'Residence EDD Test', '1 rue EDD', '75001', 'Paris', 1000)"#,
    )
    .bind(building_id)
    .execute(&pool)
    .await
    .expect("insert building");

    sqlx::query(
        r#"INSERT INTO user_account (id, email, last_name, first_name)
           VALUES ($1, $2, 'Test', 'Manager')"#,
    )
    .bind(user_id)
    .bind(&email)
    .execute(&pool)
    .await
    .expect("insert user");

    sqlx::query(
        r#"INSERT INTO unit (id, building_id, unit_number, unit_type)
           VALUES ($1, $2, $3, 'apartment'::unit_type)"#,
    )
    .bind(unit_id)
    .bind(building_id)
    .bind(&unit_number)
    .execute(&pool)
    .await
    .expect("insert unit");

    sqlx::query(
        r#"INSERT INTO allocation_key (id, building_id, name, key_type)
           VALUES ($1, $2, $3, 'special'::allocation_key_type)"#,
    )
    .bind(key_id)
    .bind(building_id)
    .bind(&key_name)
    .execute(&pool)
    .await
    .expect("insert allocation key");

    sqlx::query(
        r#"INSERT INTO unit_allocation_key (unit_id, key_id, share_value)
           VALUES ($1, $2, 100)"#,
    )
    .bind(unit_id)
    .bind(key_id)
    .execute(&pool)
    .await
    .expect("insert unit allocation key");

    let shares = vec![laruche::db::edd_reference::NewEddReferenceShare {
        unit_id,
        allocation_key_id: key_id,
        share_value: Decimal::from(90),
    }];

    let reference = laruche::db::edd_reference::create(
        &pool,
        building_id,
        "EDD notariée 2024",
        chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        user_id,
        &shares,
    )
    .await
    .expect("create edd reference");

    let latest = laruche::db::edd_reference::get_latest(&pool, building_id)
        .await
        .expect("get latest")
        .expect("reference exists");
    assert_eq!(latest.id, reference.id);
    assert_eq!(latest.label, "EDD notariée 2024");

    let ref_shares = laruche::db::edd_reference::list_shares(&pool, reference.id)
        .await
        .expect("list shares");
    assert_eq!(ref_shares.len(), 1);
    assert_eq!(ref_shares[0].share_value, Decimal::from(90));

    let current_shares = laruche::db::edd_reference::list_current_shares(&pool, building_id)
        .await
        .expect("list current shares");
    assert_eq!(current_shares.len(), 1);
    assert_eq!(current_shares[0].share_value, Decimal::from(100));

    // Pure domain comparison surfaces the value mismatch (90 referenced vs 100 configured).
    let report = laruche::domain::edd_check::compare(
        &ref_shares
            .into_iter()
            .map(|s| laruche::domain::edd_check::EddRefShare {
                unit_id: s.unit_id,
                allocation_key_id: s.allocation_key_id,
                share_value: s.share_value,
            })
            .collect::<Vec<_>>(),
        &current_shares
            .into_iter()
            .map(|s| laruche::domain::edd_check::CurrentShare {
                unit_id: s.unit_id,
                allocation_key_id: s.allocation_key_id,
                share_value: s.share_value,
            })
            .collect::<Vec<_>>(),
    );
    assert!(!report.is_empty());
}

#[tokio::test]
async fn api_edd_check_without_reference_returns_404() {
    let mut server = common::isolated_test_server().await;

    let bootstrap = server
        .post("/api/v1/onboarding/bootstrap")
        .json(&common::bootstrap_payload())
        .await;
    assert_eq!(bootstrap.status_code(), StatusCode::CREATED);
    server.add_cookies(bootstrap.cookies());

    let check = server.get("/api/v1/building/edd-check").await;
    assert_eq!(check.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn api_edd_reference_create_then_check_reports_discrepancies() {
    let mut server = common::isolated_test_server().await;

    let bootstrap = server
        .post("/api/v1/onboarding/bootstrap")
        .json(&common::bootstrap_payload())
        .await;
    assert_eq!(bootstrap.status_code(), StatusCode::CREATED);
    server.add_cookies(bootstrap.cookies());

    let create = server
        .post("/api/v1/building/edd-references")
        .json(&json!({
            "label": "EDD notariée",
            "edd_date": "2024-01-01",
            "shares": []
        }))
        .await;
    assert_eq!(create.status_code(), StatusCode::CREATED);

    let check = server.get("/api/v1/building/edd-check").await;
    assert_eq!(check.status_code(), StatusCode::OK);
    let body: Value = check.json();
    assert!(body["discrepancies"].is_array());
}
