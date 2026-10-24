// Integration tests for the budget rectificatif workflow (EP-018-S06, Pattern B).
//
// The invariants under test come from ADR-009 and LEGAL.md §11:
//   - an approved budget is a legal act: its lines are frozen and it stays in
//     force, untouched, until a rectificatif is voted;
//   - accounting history is never rewritten nor erased;
//   - at most one rectificatif in preparation, and one budget in force, per year.
//
// Run with: cargo test --test budget_rectificatif_test

mod common;

use axum::http::StatusCode;
use laruche::domain::Role;
use serde_json::{Value, json};
use uuid::Uuid;

struct Fixture {
    building_id: Uuid,
    manager_id: Uuid,
    budget_id: Uuid,
    fiscal_year: i32,
    meeting_id: Uuid,
    key_id_1: Uuid,
    #[allow(dead_code)]
    key_id_2: Uuid,
}

/// Creates a building, a manager, a closed AG, and a budget in the requested
/// status holding two active lines.
async fn setup_budget(pool: &sqlx::PgPool, status: &str) -> Fixture {
    let building_id = Uuid::new_v4();
    let manager_id = Uuid::new_v4();
    let budget_id = Uuid::new_v4();
    let meeting_id = Uuid::new_v4();
    let key_id_1 = Uuid::new_v4();
    let key_id_2 = Uuid::new_v4();
    let fiscal_year = 2025;

    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total)
         VALUES ($1, 'TestBuilding', '1 rue Test', '75001', 'Paris', 1000)",
    )
    .bind(building_id)
    .execute(pool)
    .await
    .expect("building");

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

    sqlx::query(
        "INSERT INTO user_building (user_id, building_id, roles)
         VALUES ($1, $2, ARRAY[$3]::user_role[])",
    )
    .bind(manager_id)
    .bind(building_id)
    .bind(Role::Manager.to_string())
    .execute(pool)
    .await
    .expect("user_building");

    sqlx::query(
        "INSERT INTO general_meeting (id, building_id, meeting_type, status, meeting_date, created_by)
         VALUES ($1, $2, 'annual', 'closed', NOW(), $3)",
    )
    .bind(meeting_id)
    .bind(building_id)
    .bind(manager_id)
    .execute(pool)
    .await
    .expect("meeting");

    // `approval_meeting_id` is only set once the budget is approved.
    let approval_meeting_id = if status == "approved" {
        Some(meeting_id)
    } else {
        None
    };
    sqlx::query(
        "INSERT INTO budget (id, building_id, fiscal_year, status, approval_meeting_id)
         VALUES ($1, $2, $3, $4::budget_status, $5)",
    )
    .bind(budget_id)
    .bind(building_id)
    .bind(fiscal_year)
    .bind(status)
    .bind(approval_meeting_id)
    .execute(pool)
    .await
    .expect("budget");

    for (key_id, name) in [(key_id_1, "Test Key 1"), (key_id_2, "Test Key 2")] {
        sqlx::query(
            "INSERT INTO allocation_key (id, building_id, name, key_type)
             VALUES ($1, $2, $3, 'special')",
        )
        .bind(key_id)
        .bind(building_id)
        .bind(name)
        .execute(pool)
        .await
        .expect("allocation_key");
    }

    // Two lines of the same kind: the case that used to break the line copy.
    for (key_id, label, amount) in [(key_id_1, "Charge 1", 50000), (key_id_2, "Charge 2", 30000)] {
        sqlx::query(
            "INSERT INTO budget_line (budget_id, label, key_id, budgeted_amount, account_code)
             VALUES ($1, $2, $3, $4, '602')",
        )
        .bind(budget_id)
        .bind(label)
        .bind(key_id)
        .bind(sqlx::types::Decimal::from(amount))
        .execute(pool)
        .await
        .expect("budget_line");
    }

    Fixture {
        building_id,
        manager_id,
        budget_id,
        fiscal_year,
        meeting_id,
        key_id_1,
        key_id_2,
    }
}

fn token_for(user_id: Uuid, building_id: Uuid, cfg: &laruche::config::AppConfig) -> String {
    laruche::auth::jwt::generate_access_token(
        user_id,
        building_id,
        vec![Role::Manager],
        &cfg.auth.jwt_secret,
        cfg.auth.access_token_ttl_secs,
    )
    .expect("token")
}

fn array_of(body: Value) -> Vec<Value> {
    body.as_array()
        .expect("response must be a JSON array")
        .clone()
}

// ── Approval ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn approve_promotes_draft_and_records_the_meeting() {
    let (cfg, pool) = common::test_db().await;
    let f = setup_budget(&pool, "draft").await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;

    let resp = server
        .post(&format!("/api/v1/budgets/{}/approve", f.budget_id))
        .add_header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "approval_meeting_id": f.meeting_id }))
        .await;

    assert_eq!(resp.status_code(), StatusCode::OK);
    let body: Value = resp.json();
    assert_eq!(body["status"].as_str().unwrap(), "approved");
    assert_eq!(
        body["approval_meeting_id"].as_str().unwrap(),
        f.meeting_id.to_string()
    );
}

#[tokio::test]
async fn approve_rejects_a_meeting_that_has_not_been_held() {
    let (cfg, pool) = common::test_db().await;
    let f = setup_budget(&pool, "draft").await;

    let open_meeting = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO general_meeting (id, building_id, meeting_type, status, meeting_date, created_by)
         VALUES ($1, $2, 'annual', 'called', NOW(), $3)",
    )
    .bind(open_meeting)
    .bind(f.building_id)
    .bind(f.manager_id)
    .execute(&pool)
    .await
    .expect("meeting");

    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;

    let resp = server
        .post(&format!("/api/v1/budgets/{}/approve", f.budget_id))
        .add_header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "approval_meeting_id": open_meeting }))
        .await;

    assert_eq!(resp.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn approve_rejects_an_unknown_meeting() {
    let (cfg, pool) = common::test_db().await;
    let f = setup_budget(&pool, "draft").await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;

    let resp = server
        .post(&format!("/api/v1/budgets/{}/approve", f.budget_id))
        .add_header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "approval_meeting_id": Uuid::new_v4() }))
        .await;

    assert_eq!(resp.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

// ── Freezing (Pattern B) ────────────────────────────────────────────────────

#[tokio::test]
async fn approved_budget_lines_cannot_be_created_updated_or_deleted() {
    let (cfg, pool) = common::test_db().await;
    let f = setup_budget(&pool, "approved").await;
    let token = token_for(f.manager_id, f.building_id, &cfg);

    let existing: (Uuid, i32) =
        sqlx::query_as("SELECT id, version FROM budget_line WHERE budget_id = $1 LIMIT 1")
            .bind(f.budget_id)
            .fetch_one(&pool)
            .await
            .expect("line");

    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let create = server
        .post(&format!("/api/v1/budgets/{}/lines", f.budget_id))
        .add_header("Authorization", &auth)
        .json(&json!({
            "label": "Nouvelle charge",
            "key_id": f.key_id_1,
            "budgeted_amount": "100.00",
        }))
        .await;
    assert_eq!(create.status_code(), StatusCode::UNPROCESSABLE_ENTITY);

    let update = server
        .put(&format!(
            "/api/v1/budgets/{}/lines/{}",
            f.budget_id, existing.0
        ))
        .add_header("Authorization", &auth)
        .json(&json!({
            "label": "Charge modifiée",
            "key_id": f.key_id_1,
            "budgeted_amount": "999.00",
            "version": existing.1,
        }))
        .await;
    assert_eq!(update.status_code(), StatusCode::UNPROCESSABLE_ENTITY);

    let delete = server
        .delete(&format!(
            "/api/v1/budgets/{}/lines/{}",
            f.budget_id, existing.0
        ))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(delete.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

// ── Rectificatif creation ───────────────────────────────────────────────────

#[tokio::test]
async fn rectificatif_requires_an_approved_parent() {
    let (cfg, pool) = common::test_db().await;
    let f = setup_budget(&pool, "draft").await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;

    let resp = server
        .post(&format!("/api/v1/budgets/{}/rectificatif", f.budget_id))
        .add_header("Authorization", format!("Bearer {token}"))
        .await;

    assert_eq!(resp.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

/// The regression that matters most: creating a draft rectificatif must leave the
/// budget in force completely untouched (AC-BC-009-B).
#[tokio::test]
async fn creating_a_rectificatif_leaves_the_amended_budget_in_force() {
    let (cfg, pool) = common::test_db().await;
    let f = setup_budget(&pool, "approved").await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let resp = server
        .post(&format!("/api/v1/budgets/{}/rectificatif", f.budget_id))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(resp.status_code(), StatusCode::CREATED);

    let rect: Value = resp.json();
    let rect_id: Uuid = rect["id"].as_str().unwrap().parse().unwrap();
    assert_eq!(rect["status"].as_str().unwrap(), "draft");
    assert_eq!(rect["fiscal_year"].as_i64().unwrap(), f.fiscal_year as i64);
    assert_eq!(
        rect["parent_budget_id"].as_str().unwrap(),
        f.budget_id.to_string()
    );

    // The amended budget is still approved, still has its two active lines, and
    // still reports a non-zero forecast that reconciles with them.
    let parent: Value = server
        .get(&format!("/api/v1/budgets/{}", f.budget_id))
        .add_header("Authorization", &auth)
        .await
        .json();
    assert_eq!(parent["status"].as_str().unwrap(), "approved");
    assert_eq!(parent["total_forecast"].as_str().unwrap(), "80000.00");

    let parent_lines = array_of(
        server
            .get(&format!("/api/v1/budgets/{}/lines", f.budget_id))
            .add_header("Authorization", &auth)
            .await
            .json(),
    );
    assert_eq!(parent_lines.len(), 2);
    for line in &parent_lines {
        assert!(line["valid_to"].is_null(), "parent lines must stay open");
    }

    // Both lines were copied — a budget with two lines of the same kind used to
    // fail here on a unique-index violation.
    let rect_lines = array_of(
        server
            .get(&format!("/api/v1/budgets/{rect_id}/lines"))
            .add_header("Authorization", &auth)
            .await
            .json(),
    );
    assert_eq!(rect_lines.len(), 2);
    for line in &rect_lines {
        assert_eq!(line["rectificatif_version"].as_i64().unwrap(), 1);
        assert!(
            line["actual_amount"].is_null(),
            "realised amounts must not be carried into a new budget act"
        );
    }
}

#[tokio::test]
async fn only_one_rectificatif_may_be_in_preparation() {
    let (cfg, pool) = common::test_db().await;
    let f = setup_budget(&pool, "approved").await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let first = server
        .post(&format!("/api/v1/budgets/{}/rectificatif", f.budget_id))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(first.status_code(), StatusCode::CREATED);

    let second = server
        .post(&format!("/api/v1/budgets/{}/rectificatif", f.budget_id))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(second.status_code(), StatusCode::CONFLICT);
}

// ── Rectificatif approval ───────────────────────────────────────────────────

#[tokio::test]
async fn approving_a_rectificatif_closes_the_amended_budget_and_dates_out_its_lines() {
    let (cfg, pool) = common::test_db().await;
    let f = setup_budget(&pool, "approved").await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let rect: Value = server
        .post(&format!("/api/v1/budgets/{}/rectificatif", f.budget_id))
        .add_header("Authorization", &auth)
        .await
        .json();
    let rect_id: Uuid = rect["id"].as_str().unwrap().parse().unwrap();

    let approved = server
        .post(&format!("/api/v1/budgets/{rect_id}/approve"))
        .add_header("Authorization", &auth)
        .json(&json!({ "approval_meeting_id": f.meeting_id }))
        .await;
    assert_eq!(approved.status_code(), StatusCode::OK);

    let parent: Value = server
        .get(&format!("/api/v1/budgets/{}", f.budget_id))
        .add_header("Authorization", &auth)
        .await
        .json();
    assert_eq!(parent["status"].as_str().unwrap(), "closed");

    // The amended budget's lines are dated out but never deleted: history is
    // preserved, and the validity interval must not be inverted even though the
    // rectificatif was approved the same day the lines were created.
    let closed: Vec<(Option<chrono::NaiveDate>, chrono::NaiveDate)> = sqlx::query_as(
        "SELECT valid_to, valid_from FROM budget_line WHERE budget_id = $1 ORDER BY label",
    )
    .bind(f.budget_id)
    .fetch_all(&pool)
    .await
    .expect("closed lines");
    assert_eq!(closed.len(), 2);
    for (valid_to, valid_from) in closed {
        let valid_to = valid_to.expect("line must be dated out");
        assert!(
            valid_to >= valid_from,
            "validity interval must not be inverted: {valid_to} < {valid_from}"
        );
    }
}

#[tokio::test]
async fn rectificatif_numbering_increments_along_the_chain() {
    let (cfg, pool) = common::test_db().await;
    let f = setup_budget(&pool, "approved").await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let first: Value = server
        .post(&format!("/api/v1/budgets/{}/rectificatif", f.budget_id))
        .add_header("Authorization", &auth)
        .await
        .json();
    let first_id: Uuid = first["id"].as_str().unwrap().parse().unwrap();

    server
        .post(&format!("/api/v1/budgets/{first_id}/approve"))
        .add_header("Authorization", &auth)
        .json(&json!({ "approval_meeting_id": f.meeting_id }))
        .await;

    let second: Value = server
        .post(&format!("/api/v1/budgets/{first_id}/rectificatif"))
        .add_header("Authorization", &auth)
        .await
        .json();
    let second_id: Uuid = second["id"].as_str().unwrap().parse().unwrap();

    let lines = array_of(
        server
            .get(&format!("/api/v1/budgets/{second_id}/lines"))
            .add_header("Authorization", &auth)
            .await
            .json(),
    );
    assert_eq!(lines.len(), 2);
    for line in &lines {
        assert_eq!(line["rectificatif_version"].as_i64().unwrap(), 2);
    }

    // The chain is exposed root first so the UI can label the amendments.
    let chain = array_of(
        server
            .get(&format!("/api/v1/budgets/{second_id}/rectificatifs"))
            .add_header("Authorization", &auth)
            .await
            .json(),
    );
    assert_eq!(chain.len(), 3);
    assert_eq!(chain[0]["id"].as_str().unwrap(), f.budget_id.to_string());
    assert_eq!(chain[1]["id"].as_str().unwrap(), first_id.to_string());
    assert_eq!(chain[2]["id"].as_str().unwrap(), second_id.to_string());
}

// ── Draft line editing ──────────────────────────────────────────────────────

#[tokio::test]
async fn update_line_bumps_the_version_and_rejects_a_stale_one() {
    let (cfg, pool) = common::test_db().await;
    let f = setup_budget(&pool, "draft").await;
    let token = token_for(f.manager_id, f.building_id, &cfg);

    let (line_id, version): (Uuid, i32) = sqlx::query_as(
        "SELECT id, version FROM budget_line WHERE budget_id = $1 AND label = 'Charge 1'",
    )
    .bind(f.budget_id)
    .fetch_one(&pool)
    .await
    .expect("line");

    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");
    let body = json!({
        "label": "Charge 1",
        "key_id": f.key_id_1,
        "budgeted_amount": "12345.00",
        "version": version,
    });

    let ok = server
        .put(&format!(
            "/api/v1/budgets/{}/lines/{}",
            f.budget_id, line_id
        ))
        .add_header("Authorization", &auth)
        .json(&body)
        .await;
    assert_eq!(ok.status_code(), StatusCode::OK);
    let updated: Value = ok.json();
    assert_eq!(updated["budgeted_amount"].as_str().unwrap(), "12345.00");
    assert_eq!(
        updated["version"].as_i64().unwrap(),
        (version + 1) as i64,
        "the trigger must bump the optimistic-locking token"
    );

    // Replaying the same version is a lost update and must be refused.
    let stale = server
        .put(&format!(
            "/api/v1/budgets/{}/lines/{}",
            f.budget_id, line_id
        ))
        .add_header("Authorization", &auth)
        .json(&body)
        .await;
    assert_eq!(stale.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn duplicate_active_line_is_rejected() {
    let (cfg, pool) = common::test_db().await;
    let f = setup_budget(&pool, "draft").await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;

    // Same (key_id, label) as the fixture's first line.
    let resp = server
        .post(&format!("/api/v1/budgets/{}/lines", f.budget_id))
        .add_header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "label": "Charge 1",
            "key_id": f.key_id_1,
            "budgeted_amount": "10.00",
        }))
        .await;

    assert_eq!(resp.status_code(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn a_closed_historical_line_cannot_be_updated_or_deleted() {
    let (cfg, pool) = common::test_db().await;
    let f = setup_budget(&pool, "approved").await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let rect: Value = server
        .post(&format!("/api/v1/budgets/{}/rectificatif", f.budget_id))
        .add_header("Authorization", &auth)
        .await
        .json();
    let rect_id: Uuid = rect["id"].as_str().unwrap().parse().unwrap();
    server
        .post(&format!("/api/v1/budgets/{rect_id}/approve"))
        .add_header("Authorization", &auth)
        .json(&json!({ "approval_meeting_id": f.meeting_id }))
        .await;

    // The amended budget is now closed, so its lines are unreachable for
    // mutation at the budget level as well as the line level.
    let (line_id, version): (Uuid, i32) = sqlx::query_as(
        "SELECT id, version FROM budget_line WHERE budget_id = $1 AND label = 'Charge 1'",
    )
    .bind(f.budget_id)
    .fetch_one(&pool)
    .await
    .expect("closed line");

    let update = server
        .put(&format!(
            "/api/v1/budgets/{}/lines/{}",
            f.budget_id, line_id
        ))
        .add_header("Authorization", &auth)
        .json(&json!({
            "label": "Charge 1",
            "key_id": f.key_id_1,
            "budgeted_amount": "1.00",
            "version": version,
        }))
        .await;
    assert_eq!(update.status_code(), StatusCode::UNPROCESSABLE_ENTITY);

    let delete = server
        .delete(&format!(
            "/api/v1/budgets/{}/lines/{}",
            f.budget_id, line_id
        ))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(delete.status_code(), StatusCode::UNPROCESSABLE_ENTITY);

    // Nothing was erased.
    let still_there: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM budget_line WHERE budget_id = $1")
            .bind(f.budget_id)
            .fetch_one(&pool)
            .await
            .expect("count");
    assert_eq!(still_there, 2);
}

// ── Cross-tenant isolation ──────────────────────────────────────────────────

#[tokio::test]
async fn a_budget_of_another_building_is_not_reachable() {
    let (cfg, pool) = common::test_db().await;
    let mine = setup_budget(&pool, "approved").await;
    let theirs = setup_budget(&pool, "approved").await;
    let token = token_for(mine.manager_id, mine.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let resp = server
        .post(&format!(
            "/api/v1/budgets/{}/rectificatif",
            theirs.budget_id
        ))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(resp.status_code(), StatusCode::NOT_FOUND);

    let chain = server
        .get(&format!(
            "/api/v1/budgets/{}/rectificatifs",
            theirs.budget_id
        ))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(chain.status_code(), StatusCode::NOT_FOUND);
}

// ── RBAC matrix — budget ─────────────────────────────────────────────────────
// Administration (create/update/close/approve/rectificatif) is manager-only;
// everyone authenticated may view; board member may list levies and mark paid.

#[tokio::test]
async fn board_member_cannot_administer_budgets() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.board_member_jwt);

    let created = server
        .post("/api/v1/budgets")
        .add_header("Authorization", &auth)
        .json(&json!({ "fiscal_year": 2026 }))
        .await;
    assert_eq!(created.status_code(), StatusCode::FORBIDDEN);

    let listed = server
        .get("/api/v1/budgets")
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(listed.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn residents_can_view_but_not_administer_budgets() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;

    for jwt in [&ctx.coowner_jwt, &ctx.tenant_jwt] {
        let auth = format!("Bearer {jwt}");
        let listed = server
            .get("/api/v1/budgets")
            .add_header("Authorization", &auth)
            .await;
        assert_eq!(listed.status_code(), StatusCode::OK);

        let created = server
            .post("/api/v1/budgets")
            .add_header("Authorization", &auth)
            .json(&json!({ "fiscal_year": 2026 }))
            .await;
        assert_eq!(created.status_code(), StatusCode::FORBIDDEN);
    }
}

#[tokio::test]
async fn manager_can_create_budget() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;

    let created = server
        .post("/api/v1/budgets")
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .json(&json!({ "fiscal_year": 2027 }))
        .await;
    assert_eq!(created.status_code(), StatusCode::CREATED);
}
