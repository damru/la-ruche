// Integration tests — EP-014-S01: aggregated calendar feed.
// Requires a PostgreSQL test database (DATABASE_URL).
// Run with: cargo test --test calendar_test

mod common;

use axum::http::StatusCode;
use chrono::{Duration, Utc};
use serde_json::Value;
use uuid::Uuid;

struct Ctx {
    building_id: Uuid,
    manager_jwt: String,
    coowner_jwt: String,
    manager_id: Uuid,
}

async fn setup() -> Ctx {
    let (cfg, pool) = common::test_db().await;
    setup_with(cfg, pool).await
}

async fn insert_user(
    pool: &sqlx::PgPool,
    cfg: &laruche::config::AppConfig,
    building_id: Uuid,
    role: &str,
) -> Uuid {
    let user_id = Uuid::new_v4();
    let email = format!("cal-{role}-{}@test.local", Uuid::new_v4());
    let password_hash =
        laruche::auth::password::hash_password("MotDePasse123!", &cfg.auth).expect("hash");
    sqlx::query(
        "INSERT INTO user_account (id, email, last_name, first_name, password_hash) \
         VALUES ($1, $2, 'Cal', 'User', $3)",
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

/// Seeds one event per source, all dated "today", so a single [from, to]
/// range covers every fixture without depending on wall-clock day boundaries.
async fn seed_all_sources(pool: &sqlx::PgPool, ctx: &Ctx) {
    let meeting_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO general_meeting (id, building_id, meeting_type, status, meeting_date, created_by) \
         VALUES ($1, $2, 'annual', 'called', NOW(), $3)",
    )
    .bind(meeting_id)
    .bind(ctx.building_id)
    .bind(ctx.manager_id)
    .execute(pool)
    .await
    .expect("meeting");

    let contract_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO vendor_contract (id, building_id, vendor_name, subject, contract_type, start_date, status, created_by) \
         VALUES ($1, $2, 'ACME', 'Entretien ascenseur', 'maintenance', CURRENT_DATE, 'active', $3)",
    )
    .bind(contract_id)
    .bind(ctx.building_id)
    .bind(ctx.manager_id)
    .execute(pool)
    .await
    .expect("contract");

    sqlx::query(
        "INSERT INTO contract_reminder (id, contract_id, reminder_type, reminder_date, status) \
         VALUES ($1, $2, 'renewal', CURRENT_DATE, 'pending')",
    )
    .bind(Uuid::new_v4())
    .bind(contract_id)
    .execute(pool)
    .await
    .expect("contract_reminder");

    let task_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO task (id, building_id, title, status, created_by, closed_at) \
         VALUES ($1, $2, 'Réparation fuite', 'closed', $3, NOW())",
    )
    .bind(task_id)
    .bind(ctx.building_id)
    .bind(ctx.manager_id)
    .execute(pool)
    .await
    .expect("task");

    sqlx::query(
        "INSERT INTO claim (id, building_id, title, declared_at, status, created_by) \
         VALUES ($1, $2, 'Dégât des eaux', CURRENT_DATE, 'open', $3)",
    )
    .bind(Uuid::new_v4())
    .bind(ctx.building_id)
    .bind(ctx.manager_id)
    .execute(pool)
    .await
    .expect("claim");

    sqlx::query(
        "INSERT INTO maintenance_log (id, building_id, title, intervention_date, source, created_by) \
         VALUES ($1, $2, 'Ramonage', CURRENT_DATE, 'manual', $3)",
    )
    .bind(Uuid::new_v4())
    .bind(ctx.building_id)
    .bind(ctx.manager_id)
    .execute(pool)
    .await
    .expect("maintenance_log");
}

fn today_range() -> (String, String) {
    let today = Utc::now().date_naive();
    let from = (today - Duration::days(1)).to_string();
    let to = (today + Duration::days(1)).to_string();
    (from, to)
}

#[tokio::test]
async fn calendar_events_requires_auth() {
    let server = common::test_server().await;
    let resp = server
        .get("/api/v1/calendar/events?from=2026-01-01&to=2026-01-31")
        .await;
    assert_eq!(resp.status_code(), StatusCode::UNAUTHORIZED);
}

// Test 1 (spec §7): manager sees every source in range, sorted.
#[tokio::test]
async fn manager_sees_all_sources_in_range() {
    let (cfg, pool) = common::test_db().await;
    let ctx = setup_with(cfg, pool.clone()).await;
    seed_all_sources(&pool, &ctx).await;

    let server = common::test_server().await;
    let (from, to) = today_range();
    let resp = server
        .get(&format!("/api/v1/calendar/events?from={from}&to={to}"))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let body: Value = resp.json();
    let events = body.as_array().expect("array");
    let sources: Vec<&str> = events
        .iter()
        .map(|e| e["source"].as_str().expect("source"))
        .collect();
    for expected in ["meeting", "contract", "task", "claim", "maintenance"] {
        assert!(
            sources.contains(&expected),
            "expected source {expected} in {sources:?}"
        );
    }

    // sorted by (date, time)
    let dates: Vec<&str> = events.iter().map(|e| e["date"].as_str().unwrap()).collect();
    let mut sorted = dates.clone();
    sorted.sort();
    assert_eq!(dates, sorted, "events must be sorted by date");
}

// Test 2 (spec §7): co-owner only sees the meeting date, not contract/task/claim/maintenance.
#[tokio::test]
async fn co_owner_only_sees_meeting_source() {
    let (cfg, pool) = common::test_db().await;
    let ctx = setup_with(cfg, pool.clone()).await;
    seed_all_sources(&pool, &ctx).await;

    let server = common::test_server().await;
    let (from, to) = today_range();
    let resp = server
        .get(&format!("/api/v1/calendar/events?from={from}&to={to}"))
        .add_header("Authorization", format!("Bearer {}", ctx.coowner_jwt))
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let body: Value = resp.json();
    let events = body.as_array().expect("array");
    assert!(!events.is_empty(), "co-owner must still see the meeting");
    for event in events {
        assert_eq!(event["source"], "meeting");
    }
}

// Test 3 (spec §7): events outside [from, to] are absent.
#[tokio::test]
async fn events_outside_range_are_absent() {
    let (cfg, pool) = common::test_db().await;
    let ctx = setup_with(cfg, pool.clone()).await;
    seed_all_sources(&pool, &ctx).await;

    let server = common::test_server().await;
    let today = Utc::now().date_naive();
    let far_from = (today + Duration::days(100)).to_string();
    let far_to = (today + Duration::days(110)).to_string();
    let resp = server
        .get(&format!(
            "/api/v1/calendar/events?from={far_from}&to={far_to}"
        ))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let body: Value = resp.json();
    assert_eq!(body.as_array().expect("array").len(), 0);
}

// Test 4 (spec §7): from > to → 400 invalid_range.
#[tokio::test]
async fn invalid_range_returns_400() {
    let ctx = setup().await;
    let server = common::test_server().await;
    let resp = server
        .get("/api/v1/calendar/events?from=2026-02-01&to=2026-01-01")
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
        resp.json::<Value>()["message"],
        "errors.calendar.invalid_range"
    );
}

// Test 5 (spec §7): building_id isolation — another building's events never leak.
#[tokio::test]
async fn events_are_isolated_by_building() {
    let (cfg, pool) = common::test_db().await;
    let ctx = setup_with(cfg, pool.clone()).await;
    seed_all_sources(&pool, &ctx).await;

    // Second, unrelated building + manager.
    let other_building_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total) \
         VALUES ($1, 'OtherBuilding', '2 rue Autre', '75002', 'Paris', 1000)",
    )
    .bind(other_building_id)
    .execute(&pool)
    .await
    .expect("other building");
    let other_manager_id = insert_user(
        &pool,
        &common::test_config().await,
        other_building_id,
        "manager",
    )
    .await;
    let other_manager_jwt = laruche::auth::jwt::generate_access_token(
        other_manager_id,
        other_building_id,
        vec![laruche::domain::Role::Manager],
        &common::test_config().await.auth.jwt_secret,
        common::test_config().await.auth.access_token_ttl_secs,
    )
    .expect("other manager token");

    let server = common::test_server().await;
    let (from, to) = today_range();
    let resp = server
        .get(&format!("/api/v1/calendar/events?from={from}&to={to}"))
        .add_header("Authorization", format!("Bearer {other_manager_jwt}"))
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let body: Value = resp.json();
    assert_eq!(
        body.as_array().expect("array").len(),
        0,
        "other building must not see ctx's events"
    );
}

/// Same as `setup`, but reuses a caller-provided (cfg, pool) so the caller can
/// seed data against the same pool afterwards.
async fn setup_with(cfg: laruche::config::AppConfig, pool: sqlx::PgPool) -> Ctx {
    let building_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total) \
         VALUES ($1, 'CalTestBuilding', '1 rue Cal', '75001', 'Paris', 1000)",
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
        building_id,
        manager_jwt,
        coowner_jwt,
        manager_id,
    }
}
