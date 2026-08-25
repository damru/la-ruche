// Integration tests for budget_line rectification constraints (EP-018-S05, EP-018-S06)
// Verifies:
// - Disallow duplicate non-rectified lines per year
// - Allow parent_budget_id assignment only to rectified lines
// - Enforce version bump on change
//
// Run with: cargo test --test budget_line_rectification_test

mod common;

use axum::http::StatusCode;
use laruche::domain::Role;
use serde_json::{Value, json};
use sqlx::types::Decimal;
use uuid::Uuid;

/// Helper to create a test building, user, and budget
async fn setup_test_context(pool: &sqlx::PgPool) -> (Uuid, Uuid, Uuid, i32, Uuid) {
    let building_id = Uuid::new_v4();
    let manager_id = Uuid::new_v4();
    let budget_id = Uuid::new_v4();
    let fiscal_year = 2025;

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

    // Create budget
    sqlx::query(
        "INSERT INTO budget (id, building_id, fiscal_year, status) \
         VALUES ($1, $2, $3, 'draft')",
    )
    .bind(budget_id)
    .bind(building_id)
    .bind(fiscal_year)
    .execute(pool)
    .await
    .expect("budget");

    // Create allocation key (referenced by budget_line.key_id FK)
    let key_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO allocation_key (id, building_id, name, key_type) \
         VALUES ($1, $2, 'Test Key', 'special')",
    )
    .bind(key_id)
    .bind(building_id)
    .execute(pool)
    .await
    .expect("allocation_key");

    (building_id, manager_id, budget_id, fiscal_year, key_id)
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
// TEST: Disallow duplicate non-rectified lines per year
// ============================================================================

#[tokio::test]
async fn test_duplicate_non_rectified_lines_rejected() {
    let (cfg, pool) = common::test_db().await;
    let (building_id, manager_id, budget_id, fiscal_year, key_id) = setup_test_context(&pool).await;
    let token = create_auth_token(&pool, manager_id, building_id).await;

    let server = common::test_server_with_pool(cfg, pool).await;

    let label = "Test Charge";

    // Create first non-rectified line - should succeed
    let create_input = json!({
        "label": label,
        "key_id": key_id,
        "budgeted_amount": Decimal::new(1000, 0),
        "account_code": "602",
        "year": fiscal_year,
        "line_type": "charge",
        "rectificatif_version": Value::Null,
        "parent_budget_id": Value::Null,
    });

    let resp = server
        .post(&format!("/api/v1/budgets/{}/lines", budget_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&create_input)
        .await;

    assert_eq!(
        resp.status_code(),
        StatusCode::CREATED,
        "First line creation failed"
    );

    // Try to create duplicate non-rectified line - should fail
    let resp2 = server
        .post(&format!("/api/v1/budgets/{}/lines", budget_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&create_input)
        .await;

    assert_eq!(
        resp2.status_code(),
        StatusCode::BAD_REQUEST,
        "Duplicate non-rectified line should be rejected"
    );
    assert!(
        resp2.json::<Value>()["message"]
            .as_str()
            .unwrap()
            .contains("duplicate_non_rectified")
    );
}

#[tokio::test]
async fn test_duplicate_rectified_lines_allowed() {
    let (cfg, pool) = common::test_db().await;
    let (building_id, manager_id, budget_id, fiscal_year, key_id) = setup_test_context(&pool).await;
    let token = create_auth_token(&pool, manager_id, building_id).await;

    let server = common::test_server_with_pool(cfg, pool).await;

    let label = "Test Charge";

    // Create first rectified line (rectificatif_version = 1) - should succeed
    let create_input1 = json!({
        "label": label,
        "key_id": key_id,
        "budgeted_amount": Decimal::new(1000, 0),
        "account_code": "602",
        "year": fiscal_year,
        "line_type": "charge",
        "rectificatif_version": 1,
        "parent_budget_id": Value::Null,
    });

    let resp1 = server
        .post(&format!("/api/v1/budgets/{}/lines", budget_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&create_input1)
        .await;

    assert_eq!(resp1.status_code(), StatusCode::CREATED);

    // Create second rectified line with different rectificatif_version - should succeed
    let create_input2 = json!({
        "label": label,
        "key_id": key_id,
        "budgeted_amount": Decimal::new(1500, 0),
        "account_code": "602",
        "year": fiscal_year,
        "line_type": "charge",
        "rectificatif_version": 2,
        "parent_budget_id": Value::Null,
    });

    let resp2 = server
        .post(&format!("/api/v1/budgets/{}/lines", budget_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&create_input2)
        .await;

    assert_eq!(
        resp2.status_code(),
        StatusCode::CREATED,
        "Different rectification numbers should be allowed"
    );
}

// ============================================================================
// TEST: parent_budget_id only allowed for rectified lines
// ============================================================================

#[tokio::test]
async fn test_parent_budget_id_requires_rectificatif_version() {
    let (cfg, pool) = common::test_db().await;
    let (building_id, manager_id, budget_id, fiscal_year, key_id) = setup_test_context(&pool).await;
    let token = create_auth_token(&pool, manager_id, building_id).await;

    let server = common::test_server_with_pool(cfg, pool).await;

    // Try to create non-rectified line with parent_budget_id - should fail
    let create_input = json!({
        "label": "Test Charge",
        "key_id": key_id,
        "budgeted_amount": Decimal::new(1000, 0),
        "account_code": "602",
        "year": fiscal_year,
        "line_type": "charge",
        "rectificatif_version": Value::Null,
        "parent_budget_id": budget_id,
    });

    let resp = server
        .post(&format!("/api/v1/budgets/{}/lines", budget_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&create_input)
        .await;

    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
    assert!(
        resp.json::<Value>()["message"]
            .as_str()
            .unwrap()
            .contains("parent_budget_id_requires_rectification")
    );
}

#[tokio::test]
async fn test_parent_budget_id_allowed_for_rectified() {
    let (cfg, pool) = common::test_db().await;
    let (building_id, manager_id, budget_id, fiscal_year, key_id) = setup_test_context(&pool).await;
    let token = create_auth_token(&pool, manager_id, building_id).await;

    let server = common::test_server_with_pool(cfg, pool).await;

    // Create rectified line with parent_budget_id - should succeed
    let create_input = json!({
        "label": "Test Charge",
        "key_id": key_id,
        "budgeted_amount": Decimal::new(1000, 0),
        "account_code": "602",
        "year": fiscal_year,
        "line_type": "charge",
        "rectificatif_version": 1,
        "parent_budget_id": budget_id,
    });

    let resp = server
        .post(&format!("/api/v1/budgets/{}/lines", budget_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&create_input)
        .await;

    assert_eq!(
        resp.status_code(),
        StatusCode::CREATED,
        "Rectified line with parent_budget_id should be allowed"
    );
}

// ============================================================================
// TEST: Version bump on change
// ============================================================================

#[tokio::test]
async fn test_version_bump_on_update() {
    let (cfg, pool) = common::test_db().await;
    let (building_id, manager_id, budget_id, fiscal_year, key_id) = setup_test_context(&pool).await;
    let token = create_auth_token(&pool, manager_id, building_id).await;

    let server = common::test_server_with_pool(cfg, pool).await;

    // Create a line
    let create_input = json!({
        "label": "Test Charge",
        "key_id": key_id,
        "budgeted_amount": Decimal::new(1000, 0),
        "account_code": "602",
        "year": fiscal_year,
        "line_type": "charge",
        "rectificatif_version": Value::Null,
        "parent_budget_id": Value::Null,
    });

    let create_resp = server
        .post(&format!("/api/v1/budgets/{}/lines", budget_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&create_input)
        .await;

    assert_eq!(create_resp.status_code(), StatusCode::CREATED);
    let created_line: Value = create_resp.json();
    let line_id = created_line["id"].as_str().unwrap();
    let initial_version = created_line["version"].as_i64().unwrap();

    // Update the line with correct version
    let update_input = json!({
        "label": "Updated Charge",
        "key_id": key_id,
        "budgeted_amount": Decimal::new(1500, 0),
        "account_code": "602",
        "version": initial_version,
        "year": fiscal_year,
        "line_type": "charge",
        "rectificatif_version": Value::Null,
        "parent_budget_id": Value::Null,
    });

    let update_resp = server
        .put(&format!("/api/v1/budgets/{}/lines/{}", budget_id, line_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&update_input)
        .await;

    assert_eq!(update_resp.status_code(), StatusCode::OK);
    let updated_line: Value = update_resp.json();
    let updated_version = updated_line["version"].as_i64().unwrap();

    assert_eq!(
        updated_version,
        initial_version + 1,
        "Version should be incremented on update"
    );

    // Try to update with old version - should fail with conflict
    let stale_update_input = json!({
        "label": "Stale Update",
        "key_id": key_id,
        "budgeted_amount": Decimal::new(2000, 0),
        "account_code": "602",
        "version": initial_version,
        "year": fiscal_year,
        "line_type": "charge",
        "rectificatif_version": Value::Null,
        "parent_budget_id": Value::Null,
    });

    let stale_resp = server
        .put(&format!("/api/v1/budgets/{}/lines/{}", budget_id, line_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&stale_update_input)
        .await;

    assert_eq!(
        stale_resp.status_code(),
        StatusCode::CONFLICT,
        "Update with stale version should be rejected"
    );
}

// ============================================================================
// TEST: Complex rectification scenario
// ============================================================================

#[tokio::test]
async fn test_rectification_chain() {
    let (cfg, pool) = common::test_db().await;
    let (building_id, manager_id, budget_id, fiscal_year, key_id) = setup_test_context(&pool).await;
    let token = create_auth_token(&pool, manager_id, building_id).await;

    let server = common::test_server_with_pool(cfg, pool).await;

    let label = "Works Fund";

    // Create root budget line (non-rectified)
    let root_input = json!({
        "label": label,
        "key_id": key_id,
        "budgeted_amount": Decimal::new(10000, 0),
        "account_code": "10300",
        "year": fiscal_year,
        "line_type": "fonds_travaux",
        "rectificatif_version": Value::Null,
        "parent_budget_id": Value::Null,
    });

    let root_resp = server
        .post(&format!("/api/v1/budgets/{}/lines", budget_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&root_input)
        .await;

    assert_eq!(root_resp.status_code(), StatusCode::CREATED);
    let root_line: Value = root_resp.json();
    let _root_line_id = root_line["id"].as_str().unwrap();

    // Create first rectificatif (rectificatif_version = 1) with parent_budget_id pointing to root
    // Note: parent_budget_id should reference the budget, not the budget_line
    // For this test, we'll use the budget_id as parent_budget_id
    let rectif1_input = json!({
        "label": label,
        "key_id": key_id,
        "budgeted_amount": Decimal::new(12000, 0),
        "account_code": "10300",
        "year": fiscal_year,
        "line_type": "fonds_travaux",
        "rectificatif_version": 1,
        "parent_budget_id": budget_id,
    });

    let rectif1_resp = server
        .post(&format!("/api/v1/budgets/{}/lines", budget_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&rectif1_input)
        .await;

    assert_eq!(
        rectif1_resp.status_code(),
        StatusCode::CREATED,
        "First rectificatif should be created"
    );

    // Create second rectificatif (rectificatif_version = 2)
    let rectif2_input = json!({
        "label": label,
        "key_id": key_id,
        "budgeted_amount": Decimal::new(15000, 0),
        "account_code": "10300",
        "year": fiscal_year,
        "line_type": "fonds_travaux",
        "rectificatif_version": 2,
        "parent_budget_id": budget_id,
    });

    let rectif2_resp = server
        .post(&format!("/api/v1/budgets/{}/lines", budget_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&rectif2_input)
        .await;

    assert_eq!(
        rectif2_resp.status_code(),
        StatusCode::CREATED,
        "Second rectificatif should be created"
    );

    // Verify we can list all lines
    let list_resp = server
        .get(&format!("/api/v1/budgets/{}/lines", budget_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .await;

    assert_eq!(list_resp.status_code(), StatusCode::OK);
    let lines: Value = list_resp.json();
    assert!(
        lines.as_array().unwrap().len() >= 3,
        "Should have at least 3 lines (root + 2 rectificatifs)"
    );
}

// ============================================================================
// TEST: parent_budget_id enforces same fiscal year
// ============================================================================

#[tokio::test]
async fn test_parent_budget_id_enforces_same_fiscal_year() {
    let (cfg, pool) = common::test_db().await;
    let (building_id, manager_id, budget_id, fiscal_year, key_id) = setup_test_context(&pool).await;
    let token = create_auth_token(&pool, manager_id, building_id).await;

    let server = common::test_server_with_pool(cfg, pool.clone()).await;

    // Create a second budget with a different fiscal year
    let budget_id_2026 = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO budget (id, building_id, fiscal_year, status) \
         VALUES ($1, $2, 2026, 'draft')",
    )
    .bind(budget_id_2026)
    .bind(building_id)
    .execute(&pool)
    .await
    .expect("budget_2026");

    // Try to create a line in budget 2025 with parent_budget_id pointing to budget 2026
    // This should fail because fiscal years don't match
    let create_input = json!({
        "label": "Test Charge",
        "key_id": key_id,
        "budgeted_amount": Decimal::new(1000, 0),
        "account_code": "602",
        "year": fiscal_year,
        "line_type": "charge",
        "rectificatif_version": 1,
        "parent_budget_id": budget_id_2026,
    });

    let resp = server
        .post(&format!("/api/v1/budgets/{}/lines", budget_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&create_input)
        .await;

    assert_eq!(
        resp.status_code(),
        StatusCode::BAD_REQUEST,
        "Creating line with parent_budget_id from different fiscal year should be rejected"
    );
    assert!(
        resp.json::<Value>()["message"]
            .as_str()
            .unwrap()
            .contains("parent_fiscal_year_mismatch")
    );
}

// ============================================================================
// TEST: parent_budget_id with same fiscal year succeeds
// ============================================================================

#[tokio::test]
async fn test_parent_budget_id_same_fiscal_year_succeeds() {
    let (cfg, pool) = common::test_db().await;
    let (building_id, manager_id, budget_id, fiscal_year, key_id) = setup_test_context(&pool).await;
    let token = create_auth_token(&pool, manager_id, building_id).await;

    let server = common::test_server_with_pool(cfg, pool.clone()).await;

    // Create a second budget with the SAME fiscal year (as a budget-level
    // rectificatif of the root budget, since only one root budget per
    // building/fiscal_year is allowed — see migration 0040).
    let budget_id_2025_2 = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO budget (id, building_id, fiscal_year, status, parent_budget_id) \
         VALUES ($1, $2, $3, 'draft', $4)",
    )
    .bind(budget_id_2025_2)
    .bind(building_id)
    .bind(fiscal_year)
    .bind(budget_id)
    .execute(&pool)
    .await
    .expect("budget_2025_2");

    // Create a line in budget 2025 with parent_budget_id pointing to another budget 2025
    // This should succeed because fiscal years match
    let create_input = json!({
        "label": "Test Charge",
        "key_id": key_id,
        "budgeted_amount": Decimal::new(1000, 0),
        "account_code": "602",
        "year": fiscal_year,
        "line_type": "charge",
        "rectificatif_version": 1,
        "parent_budget_id": budget_id_2025_2,
    });

    let resp = server
        .post(&format!("/api/v1/budgets/{}/lines", budget_id))
        .add_header("Authorization", format!("Bearer {}", token))
        .json(&create_input)
        .await;

    assert_eq!(
        resp.status_code(),
        StatusCode::CREATED,
        "Creating line with parent_budget_id from same fiscal year should succeed"
    );
}
