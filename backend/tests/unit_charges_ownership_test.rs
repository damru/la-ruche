mod common;

use axum::http::StatusCode;
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn co_owner_charges_are_limited_to_current_ownership_period() {
    let (cfg, pool) = common::test_db().await;

    let building_id = Uuid::new_v4();
    let unit_id = Uuid::new_v4();
    let old_owner_id = Uuid::new_v4();
    let new_owner_id = Uuid::new_v4();
    let manager_id = Uuid::new_v4();
    let budget_id = Uuid::new_v4();
    let levy_before_id = Uuid::new_v4();
    let levy_after_id = Uuid::new_v4();

    let transfer_date = chrono::NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid transfer date");
    let old_start = chrono::NaiveDate::from_ymd_opt(2025, 1, 1).expect("valid old start");
    let old_levy_due = chrono::NaiveDate::from_ymd_opt(2026, 2, 15).expect("valid old due date");
    let new_levy_due = chrono::NaiveDate::from_ymd_opt(2026, 4, 15).expect("valid new due date");

    sqlx::query(
        r#"INSERT INTO building (id, name, address_line1, postal_code, city, shares_total)
           VALUES ($1, 'Residence Charges Ownership', '1 rue des Tests', '75001', 'Paris', 1000)"#,
    )
    .bind(building_id)
    .execute(&pool)
    .await
    .expect("insert building");

    let unit_number = format!("T-{}", &Uuid::new_v4().simple().to_string()[..8]);
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
        r#"INSERT INTO user_account (id, email, last_name, first_name, password_hash)
           VALUES ($1, $2, 'Old', 'Owner', 'hash')"#,
    )
    .bind(old_owner_id)
    .bind(format!("old-owner-{}@test.local", Uuid::new_v4()))
    .execute(&pool)
    .await
    .expect("insert old owner");

    sqlx::query(
        r#"INSERT INTO user_account (id, email, last_name, first_name, password_hash)
           VALUES ($1, $2, 'New', 'Owner', 'hash')"#,
    )
    .bind(new_owner_id)
    .bind(format!("new-owner-{}@test.local", Uuid::new_v4()))
    .execute(&pool)
    .await
    .expect("insert new owner");

    sqlx::query(
        r#"INSERT INTO user_account (id, email, last_name, first_name, password_hash)
           VALUES ($1, $2, 'Test', 'Manager', 'hash')"#,
    )
    .bind(manager_id)
    .bind(format!("manager-{}@test.local", Uuid::new_v4()))
    .execute(&pool)
    .await
    .expect("insert manager");

    sqlx::query(
        r#"INSERT INTO unit_owner (unit_id, user_id, start_date, end_date)
           VALUES ($1, $2, $3, $4)"#,
    )
    .bind(unit_id)
    .bind(old_owner_id)
    .bind(old_start)
    .bind(transfer_date)
    .execute(&pool)
    .await
    .expect("insert old ownership");

    sqlx::query(
        r#"INSERT INTO unit_owner (unit_id, user_id, start_date, end_date)
           VALUES ($1, $2, $3, NULL)"#,
    )
    .bind(unit_id)
    .bind(new_owner_id)
    .bind(transfer_date)
    .execute(&pool)
    .await
    .expect("insert new ownership");

    sqlx::query(
        r#"INSERT INTO budget (id, building_id, fiscal_year, status)
           VALUES ($1, $2, 2026, 'approved'::budget_status)"#,
    )
    .bind(budget_id)
    .bind(building_id)
    .execute(&pool)
    .await
    .expect("insert budget");

    sqlx::query(
        r#"INSERT INTO levy_call (id, budget_id, call_label, due_date, charge_amount, works_fund_amount, issued_at, status)
           VALUES ($1, $2, 'Appel avant mutation', $3, 120.00, 0.00, '2026-02-10T10:00:00Z'::timestamptz, 'issued'::levy_status)"#,
    )
    .bind(levy_before_id)
    .bind(budget_id)
    .bind(old_levy_due)
    .execute(&pool)
    .await
    .expect("insert levy before transfer");

    sqlx::query(
        r#"INSERT INTO levy_call (id, budget_id, call_label, due_date, charge_amount, works_fund_amount, issued_at, status)
           VALUES ($1, $2, 'Appel après mutation', $3, 180.00, 0.00, '2026-03-20T10:00:00Z'::timestamptz, 'issued'::levy_status)"#,
    )
    .bind(levy_after_id)
    .bind(budget_id)
    .bind(new_levy_due)
    .execute(&pool)
    .await
    .expect("insert levy after transfer");

    sqlx::query(
        r#"INSERT INTO levy_call_unit (levy_call_id, unit_id, budget_line_id, key_id, amount, status)
           VALUES ($1, $2, NULL, NULL, 120.00, 'issued'::levy_status)"#,
    )
    .bind(levy_before_id)
    .bind(unit_id)
    .execute(&pool)
    .await
    .expect("insert levy unit before transfer");

    sqlx::query(
        r#"INSERT INTO levy_call_unit (levy_call_id, unit_id, budget_line_id, key_id, amount, status)
           VALUES ($1, $2, NULL, NULL, 180.00, 'issued'::levy_status)"#,
    )
    .bind(levy_after_id)
    .bind(unit_id)
    .execute(&pool)
    .await
    .expect("insert levy unit after transfer");

    let co_owner_token = laruche::auth::jwt::generate_access_token(
        new_owner_id,
        building_id,
        vec![laruche::domain::Role::CoOwner],
        &cfg.auth.jwt_secret,
        cfg.auth.access_token_ttl_secs,
    )
    .expect("co-owner token");

    let manager_token = laruche::auth::jwt::generate_access_token(
        manager_id,
        building_id,
        vec![laruche::domain::Role::Manager],
        &cfg.auth.jwt_secret,
        cfg.auth.access_token_ttl_secs,
    )
    .expect("manager token");

    let server = common::test_server().await;

    let co_owner_resp = server
        .get(&format!("/api/v1/units/{}/charges", unit_id))
        .add_header("Authorization", format!("Bearer {}", co_owner_token))
        .await;
    assert_eq!(co_owner_resp.status_code(), StatusCode::OK);

    let co_owner_body: Value = co_owner_resp.json();
    let co_owner_levies = co_owner_body["levies"].as_array().expect("levies array");
    assert_eq!(
        co_owner_levies.len(),
        1,
        "new owner must only see levies from current ownership period"
    );
    assert_eq!(co_owner_levies[0]["call_label"], "Appel après mutation");

    let manager_resp = server
        .get(&format!("/api/v1/units/{}/charges", unit_id))
        .add_header("Authorization", format!("Bearer {}", manager_token))
        .await;
    assert_eq!(manager_resp.status_code(), StatusCode::OK);

    let manager_body: Value = manager_resp.json();
    let manager_levies = manager_body["levies"].as_array().expect("levies array");
    assert_eq!(
        manager_levies.len(),
        2,
        "manager must keep full historical visibility"
    );
    assert_eq!(
        manager_body["pending_transfer"], false,
        "unit with an active owner must not be pending_transfer"
    );
}

#[tokio::test]
async fn charges_pending_transfer_flag_when_no_active_owner() {
    let (cfg, pool) = common::test_db().await;

    let building_id = Uuid::new_v4();
    let manager_id = Uuid::new_v4();
    let old_owner_id = Uuid::new_v4();
    let unit_id = Uuid::new_v4();

    let unit_number = format!("PT-{}", &Uuid::new_v4().simple().to_string()[..8]);
    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total) VALUES ($1, 'PT Building', '1 rue PT', '75001', 'Paris', 1000)"
    ).bind(building_id).execute(&pool).await.expect("building");

    sqlx::query(
        "INSERT INTO unit (id, building_id, unit_number, unit_type) VALUES ($1, $2, $3, 'apartment'::unit_type)"
    ).bind(unit_id).bind(building_id).bind(&unit_number).execute(&pool).await.expect("unit");

    sqlx::query(
        "INSERT INTO user_account (id, email, last_name, first_name, password_hash) VALUES ($1, $2, 'Old', 'Owner', 'hash')"
    ).bind(old_owner_id)
     .bind(format!("old-pt-{}@test.local", Uuid::new_v4()))
     .execute(&pool).await.expect("old owner");

    sqlx::query(
        "INSERT INTO user_account (id, email, last_name, first_name, password_hash) VALUES ($1, $2, 'Mgr', 'PT', 'hash')"
    ).bind(manager_id)
     .bind(format!("mgr-pt-{}@test.local", Uuid::new_v4()))
     .execute(&pool).await.expect("manager");

    // Old owner sold — all ownerships have an end_date → pending_transfer state
    sqlx::query(
        "INSERT INTO unit_owner (unit_id, user_id, start_date, end_date) VALUES ($1, $2, '2025-01-01', '2026-06-01')"
    ).bind(unit_id).bind(old_owner_id).execute(&pool).await.expect("closed ownership");

    let token = laruche::auth::jwt::generate_access_token(
        manager_id,
        building_id,
        vec![laruche::domain::Role::Manager],
        &cfg.auth.jwt_secret,
        cfg.auth.access_token_ttl_secs,
    )
    .expect("token");

    let server = common::test_server().await;
    let resp = server
        .get(&format!("/api/v1/units/{}/charges", unit_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .await;
    assert_eq!(resp.status_code(), axum::http::StatusCode::OK);

    let body: Value = resp.json();
    assert_eq!(
        body["pending_transfer"], true,
        "unit with no active owner must have pending_transfer=true"
    );
}
