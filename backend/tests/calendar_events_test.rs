// EP-014-S01: `GET /calendar/events` aggregated feed — role filtering,
// range validation, out-of-range exclusion, building_id isolation.

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use laruche::domain::Role;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

async fn calendar_test_server() -> (TestServer, PgPool, Uuid) {
    let (mut server, pool) = common::isolated_test_server_with_pool().await;
    let bootstrap = server
        .post("/api/v1/onboarding/bootstrap")
        .json(&common::bootstrap_payload())
        .await;
    assert_eq!(bootstrap.status_code(), StatusCode::CREATED);
    let building_id: Uuid = bootstrap.json::<Value>()["building_id"]
        .as_str()
        .expect("building_id")
        .parse()
        .expect("valid uuid");
    server.add_cookies(bootstrap.cookies());
    (server, pool, building_id)
}

async fn manager_id(pool: &PgPool, building_id: Uuid) -> Uuid {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id FROM user_building WHERE building_id = $1 LIMIT 1",
    )
    .bind(building_id)
    .fetch_one(pool)
    .await
    .expect("manager id")
}

async fn seed_meeting(
    pool: &PgPool,
    building_id: Uuid,
    manager_id: Uuid,
    meeting_date: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO general_meeting (id, building_id, meeting_type, status, meeting_date, created_by)
         VALUES ($1, $2, 'annual', 'closed', $3::timestamptz, $4)",
    )
    .bind(id)
    .bind(building_id)
    .bind(meeting_date)
    .bind(manager_id)
    .execute(pool)
    .await
    .expect("insert meeting");
    id
}

async fn seed_contract_reminder(
    pool: &PgPool,
    building_id: Uuid,
    manager_id: Uuid,
    reminder_date: &str,
) -> Uuid {
    let contract_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO vendor_contract (id, building_id, vendor_name, subject, contract_type, start_date, created_by)
         VALUES ($1, $2, 'Acme', 'Contrat test', 'maintenance', '2020-01-01', $3)",
    )
    .bind(contract_id)
    .bind(building_id)
    .bind(manager_id)
    .execute(pool)
    .await
    .expect("insert contract");

    let reminder_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO contract_reminder (id, contract_id, reminder_type, reminder_date, status)
         VALUES ($1, $2, 'renewal', $3::date, 'pending')",
    )
    .bind(reminder_id)
    .bind(contract_id)
    .bind(reminder_date)
    .execute(pool)
    .await
    .expect("insert reminder");
    reminder_id
}

async fn seed_closed_task(
    pool: &PgPool,
    building_id: Uuid,
    manager_id: Uuid,
    closed_at: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO task (id, building_id, title, status, created_by, closed_at)
         VALUES ($1, $2, 'Réparation ascenseur', 'closed', $3, $4::timestamptz)",
    )
    .bind(id)
    .bind(building_id)
    .bind(manager_id)
    .bind(closed_at)
    .execute(pool)
    .await
    .expect("insert task");
    id
}

async fn seed_claim(pool: &PgPool, building_id: Uuid, manager_id: Uuid, declared_at: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO claim (id, building_id, title, declared_at, created_by)
         VALUES ($1, $2, 'Dégât des eaux', $3::date, $4)",
    )
    .bind(id)
    .bind(building_id)
    .bind(declared_at)
    .bind(manager_id)
    .execute(pool)
    .await
    .expect("insert claim");
    id
}

async fn seed_maintenance_log(
    pool: &PgPool,
    building_id: Uuid,
    manager_id: Uuid,
    intervention_date: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO maintenance_log (id, building_id, title, intervention_date, created_by)
         VALUES ($1, $2, 'Entretien chaudière', $3::date, $4)",
    )
    .bind(id)
    .bind(building_id)
    .bind(intervention_date)
    .bind(manager_id)
    .execute(pool)
    .await
    .expect("insert maintenance log");
    id
}

fn co_owner_token(building_id: Uuid, cfg: &laruche::config::AppConfig) -> String {
    laruche::auth::jwt::generate_access_token(
        Uuid::new_v4(),
        building_id,
        vec![Role::CoOwner],
        &cfg.auth.jwt_secret,
        cfg.auth.access_token_ttl_secs,
    )
    .expect("co-owner token")
}

#[tokio::test]
async fn manager_sees_every_source_within_range_sorted() {
    let (server, pool, building_id) = calendar_test_server().await;
    let mgr = manager_id(&pool, building_id).await;

    seed_meeting(&pool, building_id, mgr, "2026-06-10T10:00:00Z").await;
    seed_contract_reminder(&pool, building_id, mgr, "2026-06-05").await;
    seed_closed_task(&pool, building_id, mgr, "2026-06-15T09:00:00Z").await;
    seed_claim(&pool, building_id, mgr, "2026-06-02").await;
    seed_maintenance_log(&pool, building_id, mgr, "2026-06-20").await;

    let res = server
        .get("/api/v1/calendar/events?from=2026-06-01&to=2026-06-30")
        .await;
    assert_eq!(res.status_code(), StatusCode::OK);
    let events: Vec<Value> = res.json();

    // Meeting also generates J+30/J+60 legal deadline events (out of this
    // range), so we just assert all 5 seeded sources are present.
    let sources: Vec<&str> = events
        .iter()
        .map(|e| e["source"].as_str().unwrap())
        .collect();
    assert!(sources.contains(&"meeting"));
    assert!(sources.contains(&"contract"));
    assert!(sources.contains(&"task"));
    assert!(sources.contains(&"claim"));
    assert!(sources.contains(&"maintenance"));

    // Sorted by date ascending.
    let dates: Vec<&str> = events.iter().map(|e| e["date"].as_str().unwrap()).collect();
    let mut sorted = dates.clone();
    sorted.sort();
    assert_eq!(dates, sorted);
}

#[tokio::test]
async fn co_owner_only_sees_meeting_dates() {
    let (_server, pool, building_id) = calendar_test_server().await;
    let mgr = manager_id(&pool, building_id).await;
    let cfg = common::test_config().await;

    seed_meeting(&pool, building_id, mgr, "2026-06-10T10:00:00Z").await;
    seed_contract_reminder(&pool, building_id, mgr, "2026-06-05").await;
    seed_closed_task(&pool, building_id, mgr, "2026-06-15T09:00:00Z").await;
    seed_claim(&pool, building_id, mgr, "2026-06-02").await;
    seed_maintenance_log(&pool, building_id, mgr, "2026-06-20").await;

    let token = co_owner_token(building_id, &cfg);
    let co_owner_server = common::test_server_with_pool(cfg, pool).await;
    let res = co_owner_server
        .get("/api/v1/calendar/events?from=2026-06-01&to=2026-06-30")
        .add_header("Authorization", format!("Bearer {token}"))
        .await;
    assert_eq!(res.status_code(), StatusCode::OK);
    let events: Vec<Value> = res.json();

    assert_eq!(
        events.len(),
        1,
        "expected only the meeting event: {events:?}"
    );
    assert_eq!(events[0]["source"], "meeting");
    assert_eq!(events[0]["is_deadline"], false);
}

#[tokio::test]
async fn events_out_of_range_are_excluded() {
    let (server, pool, building_id) = calendar_test_server().await;
    let mgr = manager_id(&pool, building_id).await;

    seed_meeting(&pool, building_id, mgr, "2026-01-01T10:00:00Z").await;

    let res = server
        .get("/api/v1/calendar/events?from=2026-06-01&to=2026-06-30")
        .await;
    assert_eq!(res.status_code(), StatusCode::OK);
    let events: Vec<Value> = res.json();
    assert!(events.is_empty());
}

#[tokio::test]
async fn from_after_to_is_rejected() {
    let (server, _pool, _building_id) = calendar_test_server().await;

    let res = server
        .get("/api/v1/calendar/events?from=2026-06-30&to=2026-06-01")
        .await;
    assert_eq!(res.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
        res.json::<Value>()["message"],
        "errors.calendar.invalid_range"
    );
}

#[tokio::test]
async fn range_over_366_days_is_rejected() {
    let (server, _pool, _building_id) = calendar_test_server().await;

    let res = server
        .get("/api/v1/calendar/events?from=2025-01-01&to=2026-06-01")
        .await;
    assert_eq!(res.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
        res.json::<Value>()["message"],
        "errors.calendar.invalid_range"
    );
}

#[tokio::test]
async fn events_from_another_building_are_isolated() {
    let (server, pool, building_id) = calendar_test_server().await;
    let mgr = manager_id(&pool, building_id).await;

    // Another building entirely, with its own meeting in range.
    let other_building_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total)
         VALUES ($1, 'Other building', '2 rue Autre', '75002', 'Paris', 1000)",
    )
    .bind(other_building_id)
    .execute(&pool)
    .await
    .expect("insert other building");
    seed_meeting(&pool, other_building_id, mgr, "2026-06-10T10:00:00Z").await;

    // Nothing seeded for `building_id` in range.
    let res = server
        .get("/api/v1/calendar/events?from=2026-06-01&to=2026-06-30")
        .await;
    assert_eq!(res.status_code(), StatusCode::OK);
    let events: Vec<Value> = res.json();
    assert!(
        events.is_empty(),
        "expected no cross-building leakage: {events:?}"
    );
}
