// Integration tests for budget rectificatif versioning workflow (EP-018-S05)
// Verifies the complete workflow of creating rectificatif budgets.
//
// Run with: cargo test --test budget_rectificatif_versioning_test

mod common;

use axum::http::StatusCode;
use laruche::domain::Role;
use serde_json::{Value, json};
use sqlx::types::Decimal;
use uuid::Uuid;

/// Helper to create a test building, user, and approved budget
async fn setup_approved_budget(pool: &sqlx::PgPool) -> (Uuid, Uuid, Uuid, i32) {
    let building_id = Uuid::new_v4();
    let manager_id = Uuid::new_v4();
    let budget_id = Uuid::new_v4();
    let fiscal_year = 2025;
    let approval_meeting_id = Uuid::new_v4();

    // Create building
    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total) 
         VALUES ($1, 'TestBuilding', '1 rue Test', '75001', 'Paris', 1000)",
    )
    .bind(building_id)
    .execute(pool)
    .await
    .expect("building");

    // Create user
    let password_hash =
        laruche::auth::password::hash_password("TestPass123!", &common::test_config().await.auth)
            .expect("hash");
    sqlx::query(
        "INSERT INTO user_account (id, email, last_name, first_name, password_hash) 
         VALUES ($1, $2, 'Test', 'Manager', $3)",
    )
    .bind(manager_id)
    .bind(format!("mgr-{}@test.local", Uuid::new_v4()))
    .bind(&password_hash)
    .execute(pool)
    .await
    .expect("user");

    // Link user to building
    sqlx::query(
        "INSERT INTO user_building (user_id, building_id, roles) \
         VALUES ($1, $2, ARRAY[$3]::user_role[])",
    )
    .bind(manager_id)
    .bind(building_id)
    .bind(Role::Manager.to_string())
    .execute(pool)
    .await
    .expect("user_building");

    // Create approval meeting
    sqlx::query(
        "INSERT INTO general_meeting (id, building_id, meeting_type, status, meeting_date, created_by) \
         VALUES ($1, $2, 'annual', 'closed', NOW(), $3)",
    )
    .bind(approval_meeting_id)
    .bind(building_id)
    .bind(manager_id)
    .execute(pool)
    .await
    .expect("approval_meeting");

    // Create budget with approved status
    sqlx::query(
        "INSERT INTO budget (id, building_id, fiscal_year, status, approval_meeting_id) \
         VALUES ($1, $2, $3, 'approved', $4)",
    )
    .bind(budget_id)
    .bind(building_id)
    .bind(fiscal_year)
    .bind(approval_meeting_id)
    .execute(pool)
    .await
    .expect("budget");

    // Create budget lines
    let key_id1 = Uuid::new_v4();
    let key_id2 = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO allocation_key (id, building_id, name, key_type) \
         VALUES ($1, $2, 'Test Key 1', 'special')",
    )
    .bind(key_id1)
    .bind(building_id)
    .execute(pool)
    .await
    .expect("allocation_key_1");

    sqlx::query(
        "INSERT INTO allocation_key (id, building_id, name, key_type) \
         VALUES ($1, $2, 'Test Key 2', 'special')",
    )
    .bind(key_id2)
    .bind(building_id)
    .execute(pool)
    .await
    .expect("allocation_key_2");

    sqlx::query(
        "INSERT INTO budget_line (budget_id, label, key_id, budgeted_amount, account_code, year, line_type, rectificatif_version, parent_budget_id, valid_from, valid_to) 
         VALUES ($1, 'Charge 1', $2, 50000, '602', $3, 'charge', NULL, NULL, CURRENT_DATE, NULL)",
    )
    .bind(budget_id)
    .bind(key_id1)
    .bind(fiscal_year)
    .execute(pool)
    .await
    .expect("budget_line_1");

    sqlx::query(
        "INSERT INTO budget_line (budget_id, label, key_id, budgeted_amount, account_code, year, line_type, rectificatif_version, parent_budget_id, valid_from, valid_to) 
         VALUES ($1, 'Charge 2', $2, 30000, '602', $3, 'charge', NULL, NULL, CURRENT_DATE, NULL)",
    )
    .bind(budget_id)
    .bind(key_id2)
    .bind(fiscal_year)
    .execute(pool)
    .await
    .expect("budget_line_2");

    (building_id, manager_id, budget_id, fiscal_year)
}

/// Helper to create an auth token for the test user
async fn create_auth_token(_pool: &sqlx::PgPool, user_id: Uuid, building_id: Uuid) -> String {
    use laruche::auth::jwt;
    let cfg = common::test_config().await;
    jwt::generate_access_token(
        user_id,
        building_id,
        vec![Role::Manager],
        &cfg.auth.jwt_secret,
        cfg.auth.access_token_ttl_secs,
    )
    .expect("token")
}

// ============================================================================
// TEST 1: Create rectificatif budget, verify line_number reuse with version bump
// ============================================================================

#[tokio::test]
async fn test_create_rectificatif_budget_with_line_versioning() {
    let (cfg, pool) = common::test_db().await;
    let (building_id, manager_id, original_budget_id, fiscal_year) =
        setup_approved_budget(&pool).await;
    let token = create_auth_token(&pool, manager_id, building_id).await;

    let server = common::test_server_with_pool(cfg, pool).await;

    // Create rectificatif budget
    let create_rect_input = json!({
        "approval_meeting_id": Uuid::new_v4()
    });

    let rect_resp = server
        .post(&format!(
            "/api/v1/budgets/{}/rectificatif",
            original_budget_id
        ))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&create_rect_input)
        .await;

    assert_eq!(
        rect_resp.status_code(),
        StatusCode::CREATED,
        "Rectificatif budget creation should succeed"
    );

    let rect_budget: Value = rect_resp.json();
    let rect_budget_id: Uuid = rect_budget["id"].as_str().unwrap().parse().unwrap();

    // Verify rectificatif budget properties
    assert_eq!(
        rect_budget["status"].as_str().unwrap(),
        "draft",
        "Rectificatif budget should be in draft status"
    );
    assert_eq!(
        rect_budget["fiscal_year"].as_i64().unwrap(),
        fiscal_year as i64,
        "Rectificatif budget should have same fiscal year as original"
    );
    assert_eq!(
        rect_budget["parent_budget_id"].as_str().unwrap(),
        original_budget_id.to_string(),
        "Rectificatif budget should reference original budget via parent_budget_id"
    );

    // Verify original budget lines were closed (valid_to set to yesterday)
    let lines_resp = server
        .get(&format!("/api/v1/budgets/{}/lines", original_budget_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    assert_eq!(lines_resp.status_code(), StatusCode::OK);
    let original_lines: Value = lines_resp.json();

    if let Value::Array(lines_array) = original_lines {
        for line in lines_array {
            assert!(
                line["valid_to"].is_string(),
                "Original budget lines should have valid_to set after rectificatif creation"
            );
        }
    }

    // Verify rectificatif budget has copied lines with new version
    let rect_lines_resp = server
        .get(&format!("/api/v1/budgets/{}/lines", rect_budget_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    assert_eq!(rect_lines_resp.status_code(), StatusCode::OK);
    let rect_lines: Value = rect_lines_resp.json();

    if let Value::Array(lines_array) = rect_lines {
        assert!(
            lines_array.len() >= 2,
            "Rectificatif should have copied all lines from original budget"
        );

        for line in lines_array {
            assert!(
                line["rectificatif_version"].is_number(),
                "Rectificatif lines should have rectificatif_version set"
            );
            assert_eq!(
                line["rectificatif_version"].as_i64().unwrap(),
                1,
                "First rectificatif lines should have version 1"
            );

            assert_eq!(
                line["parent_budget_id"].as_str().unwrap(),
                original_budget_id.to_string(),
                "Rectificatif lines should reference original budget via parent_budget_id"
            );

            assert!(
                line["valid_from"].is_string(),
                "Rectificatif lines should have valid_from set"
            );

            assert!(
                line["valid_to"].is_null(),
                "Rectificatif lines should have valid_to NULL (active)"
            );
        }
    }

    // Verify line_number reuse: create lines with same key_id but different rectificatif_version
    let key_id = Uuid::new_v4();
    let label = "Reused Charge";

    // Create line in rectificatif budget with version 1
    let line1_input = json!({
        "label": label,
        "key_id": key_id,
        "budgeted_amount": Decimal::new(1000, 0),
        "account_code": "602",
        "year": fiscal_year,
        "line_type": "charge",
        "rectificatif_version": 1,
        "parent_budget_id": original_budget_id,
    });

    let line1_resp = server
        .post(&format!("/api/v1/budgets/{}/lines", rect_budget_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&line1_input)
        .await;

    assert_eq!(
        line1_resp.status_code(),
        StatusCode::CREATED,
        "First version of line should be created in rectificatif"
    );

    // Create line with same key_id but version 2 - should succeed (line_number reuse)
    let line2_input = json!({
        "label": label,
        "key_id": key_id,
        "budgeted_amount": Decimal::new(1500, 0),
        "account_code": "602",
        "year": fiscal_year,
        "line_type": "charge",
        "rectificatif_version": 2,
        "parent_budget_id": original_budget_id,
    });

    let line2_resp = server
        .post(&format!("/api/v1/budgets/{}/lines", rect_budget_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&line2_input)
        .await;

    assert_eq!(
        line2_resp.status_code(),
        StatusCode::CREATED,
        "Second version of line with same key_id should be allowed (line_number reuse with version bump)"
    );
}

// ============================================================================
// TEST 2: Create second rectificatif, verify version increments
// ============================================================================

#[tokio::test]
async fn test_second_rectificatif_increments_version() {
    let (cfg, pool) = common::test_db().await;
    let (building_id, manager_id, original_budget_id, fiscal_year) =
        setup_approved_budget(&pool).await;
    let token = create_auth_token(&pool, manager_id, building_id).await;

    let server = common::test_server_with_pool(cfg, pool).await;

    // Create first rectificatif
    let create_rect_input = json!({
        "approval_meeting_id": Uuid::new_v4()
    });

    let rect1_resp = server
        .post(&format!(
            "/api/v1/budgets/{}/rectificatif",
            original_budget_id
        ))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&create_rect_input)
        .await;

    assert_eq!(rect1_resp.status_code(), StatusCode::CREATED);
    let rect1_budget: Value = rect1_resp.json();
    let rect1_budget_id: Uuid = rect1_budget["id"].as_str().unwrap().parse().unwrap();

    // Approve first rectificatif so we can create a second one
    let approve_input = json!({
        "approval_meeting_id": Uuid::new_v4()
    });

    let approve_resp = server
        .post(&format!("/api/v1/budgets/{}/approve", rect1_budget_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&approve_input)
        .await;

    assert_eq!(approve_resp.status_code(), StatusCode::OK);

    // Create second rectificatif from the first rectificatif
    let rect2_resp = server
        .post(&format!("/api/v1/budgets/{}/rectificatif", rect1_budget_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&create_rect_input)
        .await;

    assert_eq!(
        rect2_resp.status_code(),
        StatusCode::CREATED,
        "Second rectificatif budget creation should succeed"
    );

    let rect2_budget: Value = rect2_resp.json();
    let rect2_budget_id: Uuid = rect2_budget["id"].as_str().unwrap().parse().unwrap();

    // Verify second rectificatif has parent_budget_id pointing to first rectificatif
    assert_eq!(
        rect2_budget["parent_budget_id"].as_str().unwrap(),
        rect1_budget_id.to_string(),
        "Second rectificatif should reference first rectificatif via parent_budget_id"
    );

    // Verify second rectificatif lines have version 2
    let rect2_lines_resp = server
        .get(&format!("/api/v1/budgets/{}/lines", rect2_budget_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    assert_eq!(rect2_lines_resp.status_code(), StatusCode::OK);
    let rect2_lines: Value = rect2_lines_resp.json();

    if let Value::Array(lines_array) = rect2_lines {
        for line in lines_array {
            assert_eq!(
                line["rectificatif_version"].as_i64().unwrap(),
                2,
                "Second rectificatif lines should have version 2"
            );
        }
    }
}
