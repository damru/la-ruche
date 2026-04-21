// La Ruche integration tests
// Requires a PostgreSQL test database (DATABASE_URL=...laruche_test)
// Run with: cargo test --test integration_test

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{Value, json};

// ── Helpers ───────────────────────────────────────────────────────

/// Creates a test server with the full application stack.
async fn test_server() -> TestServer {
    dotenvy::dotenv().ok();
    let cfg = laruche::config::AppConfig::load().expect("config");
    let pool = sqlx::PgPool::connect(&cfg.db.url).await.expect("db");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations");

    let storage = laruche::storage::client::StorageClient::new(&cfg.s3)
        .await
        .expect("storage");

    let state = std::sync::Arc::new(laruche::db::AppState { pool, storage, cfg });
    let webauthn = laruche::auth::passkey::build_webauthn(&state.cfg.app).expect("webauthn");
    let app = laruche::api::router(state, webauthn);

    TestServer::new(app)
}

// ── Onboarding ────────────────────────────────────────────────────

/// Minimal valid bootstrap payload.
fn bootstrap_payload() -> serde_json::Value {
    json!({
        "email": "syndic@test.fr",
        "last_name": "Martin",
        "first_name": "Sophie",
        "password": "MotDePasse123!",
        "building_name": "Résidence Les Tests",
        "address_line1": "1 rue des Tests",
        "postal_code": "75001",
        "city": "Paris",
        "shares_total": 1000
    })
}

#[tokio::test]
async fn test_onboarding_status_returns_needs_onboarding() {
    let server = test_server().await;
    // The test database may already contain data; this only checks the JSON shape.
    let resp = server.get("/api/v1/onboarding/status").await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let body: Value = resp.json();
    assert!(
        body.get("needs_onboarding").is_some(),
        "needs_onboarding must be present"
    );
}

#[tokio::test]
async fn test_onboarding_bootstrap_invalid_input_returns_400() {
    let server = test_server().await;
    // Invalid email + too-short password
    let resp = server
        .post("/api/v1/onboarding/bootstrap")
        .json(&json!({
            "email": "pas-un-email",
            "last_name": "X",
            "first_name": "Y",
            "password": "court",
            "building_name": "Test",
            "address_line1": "1 rue",
            "postal_code": "75001",
            "city": "Paris",
            "shares_total": 1000
        }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
    let body: Value = resp.json();
    assert_eq!(body["error"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_onboarding_bootstrap_invalid_shares_returns_400() {
    let server = test_server().await;
    let mut payload = bootstrap_payload();
    payload["shares_total"] = json!(0);
    let resp = server
        .post("/api/v1/onboarding/bootstrap")
        .json(&payload)
        .await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_login_wrong_password_returns_401() {
    let server = test_server().await;
    let resp = server
        .post("/api/v1/auth/login")
        .json(&json!({ "email": "inconnu@test.fr", "password": "wrongpass" }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_invalid_input_returns_400() {
    let server = test_server().await;
    let resp = server
        .post("/api/v1/auth/login")
        .json(&json!({ "email": "pas-un-email", "password": "x" }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
    let body: Value = resp.json();
    assert_eq!(body["error"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_refresh_without_cookie_returns_401() {
    let server = test_server().await;
    let resp = server.post("/api/v1/auth/refresh").await;
    assert_eq!(resp.status_code(), StatusCode::UNAUTHORIZED);
}

// ── Protected routes without auth ─────────────────────────────────

#[tokio::test]
async fn test_protected_implemented_routes_without_jwt_return_401() {
    let server = test_server().await;
    let routes = [
        "/api/v1/building",
        "/api/v1/units",
        "/api/v1/meetings",
        "/api/v1/budgets",
        "/api/v1/documents",
        "/api/v1/contracts",
        "/api/v1/tasks",
    ];
    for route in routes {
        let resp = server.get(route).await;
        assert!(
            resp.status_code() == StatusCode::UNAUTHORIZED,
            "Route {} should return 401, got {}",
            route,
            resp.status_code()
        );
    }
}

// ── Vendor flow (public route) ───────────────────────────────────

#[tokio::test]
async fn test_vendor_token_not_found_returns_404() {
    let server = test_server().await;
    let fake_token = uuid::Uuid::new_v4();
    let resp = server.get(&format!("/vendor/{}", fake_token)).await;
    assert_eq!(resp.status_code(), StatusCode::NOT_FOUND);
}

// ── NOTICE AGPL ───────────────────────────────────────────────────

#[tokio::test]
async fn test_notice_agpl_is_accessible() {
    let server = test_server().await;
    let resp = server.get("/notice").await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let body = resp.text();
    assert!(
        body.contains("AGPL"),
        "The NOTICE endpoint must mention the AGPL license"
    );
}

// ── Domain: assembly voting ──────────────────────────────────────

#[test]
fn test_simple_majority_vote_passes() {
    use laruche::domain::vote::{VoteRule, VoteTally};

    let tally = VoteTally {
        shares_total: 1000,
        shares_present: 600,
        total_owners: 10,
        present_owners: 7,
        for_votes: 400,
        against_votes: 200,
        abstentions: 0,
    };
    let result = tally.compute(&VoteRule::SimpleMajority);
    assert!(
        result.passed,
        "400 > 300 should pass article 24 style majority"
    );
}

#[test]
fn test_absolute_majority_vote_fails_when_below_threshold() {
    use laruche::domain::vote::{VoteRule, VoteTally};

    let tally = VoteTally {
        shares_total: 1000,
        shares_present: 800,
        total_owners: 10,
        present_owners: 8,
        for_votes: 490,
        against_votes: 310,
        abstentions: 0,
    };
    let result = tally.compute(&VoteRule::AbsoluteMajority);
    assert!(!result.passed, "490 < 500 should fail absolute majority");
}

#[test]
fn test_double_majority_vote_requires_two_thirds() {
    use laruche::domain::vote::{VoteRule, VoteTally};

    let tally = VoteTally {
        shares_total: 1000,
        shares_present: 900,
        total_owners: 10,
        present_owners: 9,
        for_votes: 650,
        against_votes: 250,
        abstentions: 0,
    };
    let result = tally.compute(&VoteRule::DoubleMajority);
    assert!(
        !result.passed,
        "650 < 667 should fail the two-thirds threshold"
    );
}

// ── Domain: levies ───────────────────────────────────────────────

#[test]
fn test_levies_sum_matches_total_amount() {
    use laruche::domain::charges::{UnitWithShares, compute_levies};
    use rust_decimal::Decimal;

    let key_id = uuid::Uuid::new_v4();
    let units = vec![
        UnitWithShares {
            unit_id: uuid::Uuid::new_v4(),
            key_id,
            shares: 333,
        },
        UnitWithShares {
            unit_id: uuid::Uuid::new_v4(),
            key_id,
            shares: 333,
        },
        UnitWithShares {
            unit_id: uuid::Uuid::new_v4(),
            key_id,
            shares: 334,
        },
    ];
    let amount = Decimal::new(120000, 2);
    let total_shares = 1000;

    let levies = compute_levies(amount, &units, total_shares);
    let sum: Decimal = levies.iter().map(|levy| levy.amount).sum();
    assert_eq!(
        sum, amount,
        "The sum of computed levies must match the total amount exactly"
    );
}

#[test]
fn test_levies_with_empty_units_returns_empty() {
    use laruche::domain::charges::compute_levies;
    use rust_decimal::Decimal;

    let levies = compute_levies(Decimal::new(1000, 0), &[], 1000);
    assert!(levies.is_empty());
}

// ── Domain: contract reminders ───────────────────────────────────

#[test]
fn test_annual_contract_generates_renewal_and_expiration_reminders() {
    use chrono::NaiveDate;
    use laruche::domain::reminders::{ContractReminderSpec, ReminderKind, compute_reminders};

    let today = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let contract = ContractReminderSpec {
        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        end_date: Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
        auto_renewal: true,
        notice_days: Some(90),
        payment_frequency_months: Some(12),
    };

    let reminders = compute_reminders(&contract, today);
    assert!(
        reminders
            .iter()
            .any(|reminder| reminder.kind == ReminderKind::Renewal),
        "Expected a renewal reminder"
    );
    assert!(
        reminders
            .iter()
            .any(|reminder| reminder.kind == ReminderKind::Expiration),
        "Expected an expiration reminder"
    );
}

#[test]
fn test_reminders_never_return_past_dates() {
    use chrono::NaiveDate;
    use laruche::domain::reminders::{ContractReminderSpec, compute_reminders};

    let today = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let contract = ContractReminderSpec {
        start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        end_date: Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
        auto_renewal: true,
        notice_days: Some(30),
        payment_frequency_months: Some(3),
    };

    let reminders = compute_reminders(&contract, today);
    for reminder in &reminders {
        assert!(
            reminder.reminder_date >= today,
            "Reminder {:?} must not be in the past",
            reminder.reminder_date
        );
    }
}
