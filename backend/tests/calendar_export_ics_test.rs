// EP-014-S04: `GET /calendar/export.ics` — role-filtered ICS export
// (AC-CAL-003-A/B/C).

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
async fn manager_export_contains_meeting_and_contract_with_correct_headers() {
    let (server, pool, building_id) = calendar_test_server().await;
    let mgr = manager_id(&pool, building_id).await;

    seed_meeting(&pool, building_id, mgr, "2026-06-10T10:00:00Z").await;
    seed_contract_reminder(&pool, building_id, mgr, "2026-06-05").await;

    let res = server
        .get("/api/v1/calendar/export.ics?from=2026-06-01&to=2026-06-30")
        .await;
    assert_eq!(res.status_code(), StatusCode::OK);
    assert!(
        res.headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("text/calendar")
    );
    assert_eq!(
        res.headers().get("content-disposition").unwrap(),
        "attachment; filename=\"laruche-calendar.ics\""
    );

    let body = res.text();
    assert!(body.starts_with("BEGIN:VCALENDAR"));
    assert!(body.trim_end().ends_with("END:VCALENDAR"));
    assert!(body.contains("BEGIN:VEVENT"));
    // Contract reminder is a deadline (is_deadline=true) → VALARM present.
    assert!(body.contains("BEGIN:VALARM"));
    assert!(body.contains("TRIGGER:-PT24H"));
}

#[tokio::test]
async fn co_owner_export_excludes_contract_events() {
    let (_server, pool, building_id) = calendar_test_server().await;
    let mgr = manager_id(&pool, building_id).await;
    let cfg = common::test_config().await;

    seed_meeting(&pool, building_id, mgr, "2026-06-10T10:00:00Z").await;
    seed_contract_reminder(&pool, building_id, mgr, "2026-06-05").await;

    let token = co_owner_token(building_id, &cfg);
    let co_owner_server = common::test_server_with_pool(cfg, pool).await;
    let res = co_owner_server
        .get("/api/v1/calendar/export.ics?from=2026-06-01&to=2026-06-30")
        .add_header("Authorization", format!("Bearer {token}"))
        .await;
    assert_eq!(res.status_code(), StatusCode::OK);
    let body = res.text();

    assert!(body.contains("CATEGORIES:Meeting"));
    assert!(!body.contains("CATEGORIES:Contract"));
}

#[tokio::test]
async fn from_after_to_is_rejected() {
    let (server, _pool, _building_id) = calendar_test_server().await;

    let res = server
        .get("/api/v1/calendar/export.ics?from=2026-06-30&to=2026-06-01")
        .await;
    assert_eq!(res.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
        res.json::<Value>()["message"],
        "errors.calendar.invalid_range"
    );
}

#[tokio::test]
async fn missing_range_defaults_to_valid_empty_calendar() {
    let (server, _pool, _building_id) = calendar_test_server().await;

    let res = server.get("/api/v1/calendar/export.ics").await;
    assert_eq!(res.status_code(), StatusCode::OK);
    let body = res.text();
    assert!(body.starts_with("BEGIN:VCALENDAR"));
    assert!(body.trim_end().ends_with("END:VCALENDAR"));
}
