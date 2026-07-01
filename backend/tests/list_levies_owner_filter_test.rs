mod common;

use axum::http::StatusCode;
use serde_json::Value;
use uuid::Uuid;

/// EP-002-S19: Manager can filter levies by historical owner.
/// Verifies AC-GP-009-A (default = current owner) and AC-GP-009-B (filter by past owner).
#[tokio::test]
async fn manager_can_filter_budget_levies_by_historical_owner() {
    let (cfg, pool) = common::test_db().await;

    let building_id = Uuid::new_v4();
    let unit_id = Uuid::new_v4();
    let old_owner_id = Uuid::new_v4();
    let new_owner_id = Uuid::new_v4();
    let manager_id = Uuid::new_v4();
    let budget_id = Uuid::new_v4();
    let levy_before_id = Uuid::new_v4();
    let levy_after_id = Uuid::new_v4();

    let transfer_date = chrono::NaiveDate::from_ymd_opt(2026, 3, 1).expect("date");
    let old_start = chrono::NaiveDate::from_ymd_opt(2025, 1, 1).expect("date");
    let levy_before_due = chrono::NaiveDate::from_ymd_opt(2026, 2, 15).expect("date");
    let levy_after_due = chrono::NaiveDate::from_ymd_opt(2026, 4, 15).expect("date");

    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total) \
         VALUES ($1, 'LevyFilterTest', '1 rue Test', '75001', 'Paris', 1000)",
    )
    .bind(building_id)
    .execute(&pool)
    .await
    .expect("insert building");

    let unit_number = format!("LFT-{}", &Uuid::new_v4().simple().to_string()[..8]);
    sqlx::query(
        "INSERT INTO unit (id, building_id, unit_number, unit_type) \
         VALUES ($1, $2, $3, 'apartment'::unit_type)",
    )
    .bind(unit_id)
    .bind(building_id)
    .bind(&unit_number)
    .execute(&pool)
    .await
    .expect("insert unit");

    for (uid, last, first, email_prefix) in [
        (old_owner_id, "Ancien", "Proprio", "old"),
        (new_owner_id, "Nouveau", "Proprio", "new"),
        (manager_id, "Gestion", "Syndic", "mgr"),
    ] {
        sqlx::query(
            "INSERT INTO user_account (id, email, last_name, first_name, password_hash) \
             VALUES ($1, $2, $3, $4, 'hash')",
        )
        .bind(uid)
        .bind(format!(
            "{}-lft-{}@test.local",
            email_prefix,
            Uuid::new_v4()
        ))
        .bind(last)
        .bind(first)
        .execute(&pool)
        .await
        .expect("insert user");
    }

    sqlx::query(
        "INSERT INTO unit_owner (unit_id, user_id, start_date, end_date) VALUES ($1, $2, $3, $4)",
    )
    .bind(unit_id)
    .bind(old_owner_id)
    .bind(old_start)
    .bind(transfer_date)
    .execute(&pool)
    .await
    .expect("old ownership");

    sqlx::query(
        "INSERT INTO unit_owner (unit_id, user_id, start_date, end_date) VALUES ($1, $2, $3, NULL)",
    )
    .bind(unit_id)
    .bind(new_owner_id)
    .bind(transfer_date)
    .execute(&pool)
    .await
    .expect("new ownership");

    sqlx::query(
        "INSERT INTO budget (id, building_id, fiscal_year, status) \
         VALUES ($1, $2, 2026, 'approved'::budget_status)",
    )
    .bind(budget_id)
    .bind(building_id)
    .execute(&pool)
    .await
    .expect("budget");

    for (lid, label, due, issued) in [
        (
            levy_before_id,
            "Appel avant mutation",
            levy_before_due,
            "2026-02-01T10:00:00Z",
        ),
        (
            levy_after_id,
            "Appel après mutation",
            levy_after_due,
            "2026-03-20T10:00:00Z",
        ),
    ] {
        sqlx::query(
            "INSERT INTO levy_call (id, budget_id, call_label, due_date, charge_amount, \
             works_fund_amount, issued_at, status) \
             VALUES ($1, $2, $3, $4, 100.00, 0.00, $5::timestamptz, 'issued'::levy_status)",
        )
        .bind(lid)
        .bind(budget_id)
        .bind(label)
        .bind(due)
        .bind(issued)
        .execute(&pool)
        .await
        .expect("levy_call");

        sqlx::query(
            "INSERT INTO levy_call_unit (levy_call_id, unit_id, budget_line_id, key_id, \
             amount, status) VALUES ($1, $2, NULL, NULL, 100.00, 'issued'::levy_status)",
        )
        .bind(lid)
        .bind(unit_id)
        .execute(&pool)
        .await
        .expect("levy_call_unit");
    }

    let manager_token = laruche::auth::jwt::generate_access_token(
        manager_id,
        building_id,
        vec![laruche::domain::Role::Manager],
        &cfg.auth.jwt_secret,
        cfg.auth.access_token_ttl_secs,
    )
    .expect("token");

    let server = common::test_server().await;
    let auth_header = format!("Bearer {}", manager_token);

    // No filter — both levies visible, current owner is new_owner
    let resp = server
        .get(&format!("/api/v1/budgets/{}/levies", budget_id))
        .add_header("Authorization", &auth_header)
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let body: Value = resp.json();
    let levies = body.as_array().expect("array");
    assert_eq!(levies.len(), 2, "unfiltered: both levies");
    // Current owner shown for both rows
    assert!(
        levies
            .iter()
            .all(|l| l["owner_id"] == new_owner_id.to_string()),
        "unfiltered: current owner on all rows"
    );

    // Filter by old owner — only pre-transfer levy
    let resp = server
        .get(&format!(
            "/api/v1/budgets/{}/levies?owner_id={}",
            budget_id, old_owner_id
        ))
        .add_header("Authorization", &auth_header)
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let body: Value = resp.json();
    let levies = body.as_array().expect("array");
    assert_eq!(levies.len(), 1, "old-owner filter: one levy");
    assert_eq!(
        levies[0]["call_label"], "Appel avant mutation",
        "old-owner filter: correct levy"
    );
    assert_eq!(
        levies[0]["owner_id"],
        old_owner_id.to_string(),
        "old-owner filter: correct owner shown"
    );

    // Filter by new owner — only post-transfer levy
    let resp = server
        .get(&format!(
            "/api/v1/budgets/{}/levies?owner_id={}",
            budget_id, new_owner_id
        ))
        .add_header("Authorization", &auth_header)
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let body: Value = resp.json();
    let levies = body.as_array().expect("array");
    assert_eq!(levies.len(), 1, "new-owner filter: one levy");
    assert_eq!(
        levies[0]["call_label"], "Appel après mutation",
        "new-owner filter: correct levy"
    );
    assert_eq!(
        levies[0]["owner_id"],
        new_owner_id.to_string(),
        "new-owner filter: correct owner shown"
    );
}
