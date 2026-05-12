// La Ruche integration tests
// Requires a PostgreSQL test database (DATABASE_URL=...laruche_test)
// Run with: cargo test --test integration_test

use axum::http::{StatusCode, header::AUTHORIZATION};
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
        favorable_owners: 4,
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
        favorable_owners: 4,
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
        favorable_owners: 6,
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

// ── Integration: Assembly legal cases (real DB) ────────────────────────────
//
// These tests require a live PostgreSQL database. They run through the full HTTP
// stack and cover all majority rules defined in the French condominium act:
//   Art. 24 — simple majority of votes expressed
//   Art. 25 — absolute majority of all co-owners
//   Art. 25-1 — second-vote passerelle when Art. 25 fails above 1/3
//   Art. 26 — double majority (2/3 shares + 2/3 owners)
//   Unanimity
//
// Share distribution provisioned by `setup_voting_environment`:
//   A = 500 shares, B = 300 shares, C = 200 shares → total = 1000
//
// All cases are grouped in a single `#[tokio::test]` to avoid parallel
// mutations on the shared building state.

/// Shared test environment for assembly integration tests.
struct VotingEnv {
    server: TestServer,
    bearer: String,
    uid_a: uuid::Uuid, // 500 shares
    uid_b: uuid::Uuid, // 300 shares
    uid_c: uuid::Uuid, // 200 shares
}

/// Bootstraps the building (if empty) and logs in with standard credentials.
/// Generates a short-lived JWT for Bearer auth (avoids cookie persistence).
async fn setup_voting_environment() -> VotingEnv {
    let server = test_server().await;

    // Try bootstrap; safe to ignore 409 (already initialized)
    server
        .post("/api/v1/onboarding/bootstrap")
        .json(&bootstrap_payload())
        .await;

    let login: Value = server
        .post("/api/v1/auth/login")
        .json(&json!({ "email": "syndic@test.fr", "password": "MotDePasse123!" }))
        .await
        .json();

    let user_id =
        uuid::Uuid::parse_str(login["user_id"].as_str().expect("user_id in login response"))
            .expect("valid UUID");
    let building_id = uuid::Uuid::parse_str(
        login["building_id"]
            .as_str()
            .expect("building_id in login response"),
    )
    .expect("valid UUID");

    dotenvy::dotenv().ok();
    let cfg = laruche::config::AppConfig::load().expect("AppConfig");
    let token = laruche::auth::jwt::generate_access_token(
        user_id,
        building_id,
        vec![laruche::domain::Role::Manager],
        &cfg.auth.jwt_secret,
        cfg.auth.access_token_ttl_secs,
    )
    .expect("JWT generation");
    let bearer = format!("Bearer {}", token);

    // Archive ALL currently active units to get a clean slate.
    // Archived units are excluded from compute_active_shares_total().
    let units: Value = server
        .get("/api/v1/units")
        .add_header(AUTHORIZATION, bearer.as_str())
        .await
        .json();
    for unit in units.as_array().expect("units list") {
        if unit["active"].as_bool().unwrap_or(false) {
            let uid = unit["id"].as_str().expect("unit id");
            server
                .delete(&format!("/api/v1/units/{}", uid))
                .add_header(AUTHORIZATION, bearer.as_str())
                .await;
        }
    }

    // Create 3 fresh test units
    let a: Value = server
        .post("/api/v1/units")
        .add_header(AUTHORIZATION, bearer.as_str())
        .json(&json!({ "unit_number": "INT-A", "unit_type": "apartment" }))
        .await
        .json();
    let b: Value = server
        .post("/api/v1/units")
        .add_header(AUTHORIZATION, bearer.as_str())
        .json(&json!({ "unit_number": "INT-B", "unit_type": "apartment" }))
        .await
        .json();
    let c: Value = server
        .post("/api/v1/units")
        .add_header(AUTHORIZATION, bearer.as_str())
        .json(&json!({ "unit_number": "INT-C", "unit_type": "apartment" }))
        .await
        .json();

    let uid_a = uuid::Uuid::parse_str(a["id"].as_str().expect("unit A id")).expect("UUID");
    let uid_b = uuid::Uuid::parse_str(b["id"].as_str().expect("unit B id")).expect("UUID");
    let uid_c = uuid::Uuid::parse_str(c["id"].as_str().expect("unit C id")).expect("UUID");

    // Locate the general allocation key created by bootstrap
    let keys: Value = server
        .get("/api/v1/building/keys")
        .add_header(AUTHORIZATION, bearer.as_str())
        .await
        .json();
    let key_id = keys
        .as_array()
        .expect("keys list")
        .iter()
        .find(|k| k["key_type"] == "general")
        .expect("general allocation key")["id"]
        .as_str()
        .expect("key id")
        .to_string();

    // Set general-key shares: A=500, B=300, C=200 — total 1000 = building.shares_total
    let shares_resp = server
        .put(&format!("/api/v1/building/keys/{}/shares", key_id))
        .add_header(AUTHORIZATION, bearer.as_str())
        .json(&json!({
            "shares": [
                { "unit_id": uid_a, "share_value": 500 },
                { "unit_id": uid_b, "share_value": 300 },
                { "unit_id": uid_c, "share_value": 200 },
            ]
        }))
        .await;
    assert_eq!(
        shares_resp.status_code(),
        StatusCode::NO_CONTENT,
        "shares must be set without error"
    );

    VotingEnv {
        server,
        bearer,
        uid_a,
        uid_b,
        uid_c,
    }
}

/// Creates a meeting + resolution with the given `vote_type`, casts `votes`,
/// and returns `(meeting_id, resolution_id)`.
async fn create_resolution_with_votes(
    env: &VotingEnv,
    vote_type: &str,
    votes: &[(uuid::Uuid, &str)], // (unit_id, "in_favor"|"against"|"abstain")
) -> (uuid::Uuid, uuid::Uuid) {
    let meeting: Value = env
        .server
        .post("/api/v1/meetings")
        .add_header(AUTHORIZATION, env.bearer.as_str())
        .json(&json!({
            "meeting_date": "2099-01-01T10:00:00Z",
            "meeting_type": "annual",
        }))
        .await
        .json();
    let mid = uuid::Uuid::parse_str(meeting["id"].as_str().expect("meeting id")).expect("UUID");

    let res: Value = env
        .server
        .post(&format!("/api/v1/meetings/{}/resolutions", mid))
        .add_header(AUTHORIZATION, env.bearer.as_str())
        .json(&json!({
            "number": 1,
            "title": format!("Test {}", vote_type),
            "vote_type": vote_type,
        }))
        .await
        .json();
    let rid =
        uuid::Uuid::parse_str(res["id"].as_str().expect("resolution id")).expect("UUID");

    for (unit_id, position) in votes {
        let vote_resp = env
            .server
            .post(&format!(
                "/api/v1/meetings/{}/resolutions/{}/vote",
                mid, rid
            ))
            .add_header(AUTHORIZATION, env.bearer.as_str())
            .json(&json!({ "unit_id": unit_id, "position": position }))
            .await;
        assert_eq!(
            vote_resp.status_code(),
            StatusCode::NO_CONTENT,
            "vote for unit {} must be accepted",
            unit_id
        );
    }

    (mid, rid)
}

/// Fetches the vote result for `(mid, rid)` and asserts HTTP 200.
async fn get_vote_result(env: &VotingEnv, mid: uuid::Uuid, rid: uuid::Uuid) -> Value {
    let resp = env
        .server
        .get(&format!(
            "/api/v1/meetings/{}/resolutions/{}/results",
            mid, rid
        ))
        .add_header(AUTHORIZATION, env.bearer.as_str())
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    resp.json()
}

/// All assembly legal-case integration tests in a single sequential test.
/// Share distribution: A=500, B=300, C=200 → shares_total = 1000.
#[tokio::test]
async fn test_assembly_all_legal_cases_with_real_db() {
    let env = setup_voting_environment().await;

    // ── Art. 24 — simple_majority passes ─────────────────────────────────────
    // A(500) in_favor vs B(300) against: shares_counted=800, threshold=401 → passes
    {
        let (mid, rid) = create_resolution_with_votes(
            &env,
            "simple_majority",
            &[
                (env.uid_a, "in_favor"),
                (env.uid_b, "against"),
                (env.uid_c, "abstain"),
            ],
        )
        .await;
        let result = get_vote_result(&env, mid, rid).await;

        assert_eq!(result["for_votes"], json!(500), "Art.24: for_votes");
        assert_eq!(result["against_votes"], json!(300), "Art.24: against_votes");
        assert_eq!(result["abstentions"], json!(200), "Art.24: abstentions");
        assert_eq!(result["shares_counted"], json!(800), "Art.24: shares_counted");
        assert_eq!(result["threshold"], json!(401), "Art.24: threshold = 800/2+1");
        assert_eq!(result["passed"], json!(true), "Art.24: 500 >= 401 must pass");
        assert_eq!(
            result["eligible_for_second_vote"],
            Value::Null,
            "Art.24: no second-vote passerelle"
        );
    }

    // ── Art. 24 — simple_majority fails (minority) ────────────────────────────
    // B(300) in_favor vs A(500) against: threshold=401 → fails
    {
        let (mid, rid) = create_resolution_with_votes(
            &env,
            "simple_majority",
            &[
                (env.uid_a, "against"),
                (env.uid_b, "in_favor"),
                (env.uid_c, "abstain"),
            ],
        )
        .await;
        let result = get_vote_result(&env, mid, rid).await;

        assert_eq!(result["passed"], json!(false), "Art.24: minority must fail");
        assert_eq!(
            result["eligible_for_second_vote"],
            Value::Null,
            "Art.24: never eligible for second vote"
        );
    }

    // ── Art. 25 — absolute_majority passes ───────────────────────────────────
    // A+B(800) in_favor / 1000 total → threshold=501 → passes
    {
        let (mid, rid) = create_resolution_with_votes(
            &env,
            "absolute_majority",
            &[
                (env.uid_a, "in_favor"),
                (env.uid_b, "in_favor"),
                (env.uid_c, "against"),
            ],
        )
        .await;
        let result = get_vote_result(&env, mid, rid).await;

        assert_eq!(result["for_votes"], json!(800), "Art.25: for_votes");
        assert_eq!(result["shares_total"], json!(1000), "Art.25: shares_total");
        assert_eq!(result["threshold"], json!(501), "Art.25: threshold = 1000/2+1");
        assert_eq!(result["passed"], json!(true), "Art.25: 800 >= 501 must pass");
        assert_eq!(result["eligible_for_second_vote"], Value::Null);
    }

    // ── Art. 25 rejected — NOT eligible for Art. 25-1 (for ≤ 1/3) ────────────
    // C(200) in_favor / 1000: 200*3=600 ≤ 1000 → no passerelle
    {
        let (mid, rid) = create_resolution_with_votes(
            &env,
            "absolute_majority",
            &[
                (env.uid_a, "against"),
                (env.uid_b, "against"),
                (env.uid_c, "in_favor"),
            ],
        )
        .await;
        let result = get_vote_result(&env, mid, rid).await;

        assert_eq!(result["for_votes"], json!(200));
        assert_eq!(result["passed"], json!(false));
        assert_eq!(
            result["eligible_for_second_vote"],
            Value::Null,
            "200*3=600 ≤ 1000 → not eligible for Art.25-1"
        );
    }

    // ── Art. 25-1 — eligible when Art. 25 fails but > 1/3 in favour ──────────
    // A(500) in_favor / 1000: 500 < 501 (fails), but 500*3=1500 > 1000 → passerelle
    {
        let (mid, rid) = create_resolution_with_votes(
            &env,
            "absolute_majority",
            &[
                (env.uid_a, "in_favor"),
                (env.uid_b, "against"),
                (env.uid_c, "against"),
            ],
        )
        .await;
        let result = get_vote_result(&env, mid, rid).await;

        assert_eq!(result["for_votes"], json!(500));
        assert_eq!(result["passed"], json!(false), "500 < 501 → fails Art.25");
        assert_eq!(
            result["eligible_for_second_vote"],
            json!("art_25_1"),
            "500 > 333 (1/3 of 1000) → Art.25-1 eligible"
        );
    }

    // ── Art. 26 — double_majority passes ─────────────────────────────────────
    // A+B+C(1000) in_favor: shares_threshold(667) met; with 0 co_owners in the
    // test building the owners_threshold (0*3 ≥ 0*2) is trivially met → passes.
    {
        let (mid, rid) = create_resolution_with_votes(
            &env,
            "double_majority",
            &[
                (env.uid_a, "in_favor"),
                (env.uid_b, "in_favor"),
                (env.uid_c, "in_favor"),
            ],
        )
        .await;
        let result = get_vote_result(&env, mid, rid).await;

        assert_eq!(result["for_votes"], json!(1000));
        assert_eq!(result["threshold"], json!(667), "Art.26: (1000*2+2)/3 = 667");
        assert_eq!(result["passed"], json!(true), "1000 ≥ 667 → Art.26 passes");
        assert_eq!(result["eligible_for_second_vote"], Value::Null);
    }

    // ── Art. 26 rejected — for ≤ 1/2 total → not eligible for Art. 26-1 ──────
    // C(200) in_favor / 1000: 200*2=400 ≤ 1000 → no Art.26-1
    {
        let (mid, rid) = create_resolution_with_votes(
            &env,
            "double_majority",
            &[
                (env.uid_a, "against"),
                (env.uid_b, "against"),
                (env.uid_c, "in_favor"),
            ],
        )
        .await;
        let result = get_vote_result(&env, mid, rid).await;

        assert_eq!(result["for_votes"], json!(200));
        assert_eq!(result["passed"], json!(false));
        assert_eq!(
            result["eligible_for_second_vote"],
            Value::Null,
            "200*2=400 ≤ 1000 → not eligible for Art.26-1"
        );
    }

    // ── Unanimity — passes when all shares vote in favour ────────────────────
    {
        let (mid, rid) = create_resolution_with_votes(
            &env,
            "unanimity",
            &[
                (env.uid_a, "in_favor"),
                (env.uid_b, "in_favor"),
                (env.uid_c, "in_favor"),
            ],
        )
        .await;
        let result = get_vote_result(&env, mid, rid).await;

        assert_eq!(result["for_votes"], json!(1000));
        assert_eq!(result["against_votes"], json!(0));
        assert_eq!(result["abstentions"], json!(0));
        assert_eq!(result["passed"], json!(true), "1000/1000 in favour → unanimity");
        assert_eq!(result["eligible_for_second_vote"], Value::Null);
    }

    // ── Unanimity — fails when any abstention exists ──────────────────────────
    {
        let (mid, rid) = create_resolution_with_votes(
            &env,
            "unanimity",
            &[
                (env.uid_a, "in_favor"),
                (env.uid_b, "in_favor"),
                (env.uid_c, "abstain"),
            ],
        )
        .await;
        let result = get_vote_result(&env, mid, rid).await;

        assert_eq!(result["passed"], json!(false), "abstention breaks unanimity");
        assert_eq!(result["eligible_for_second_vote"], Value::Null);
    }

    // ── Protected route — results returns 401 without auth ───────────────────
    {
        let (mid, rid) = create_resolution_with_votes(
            &env,
            "simple_majority",
            &[(env.uid_a, "in_favor")],
        )
        .await;
        let resp = env
            .server
            .get(&format!(
                "/api/v1/meetings/{}/resolutions/{}/results",
                mid, rid
            ))
            .await; // no Authorization header
        assert_eq!(
            resp.status_code(),
            StatusCode::UNAUTHORIZED,
            "results endpoint must require authentication"
        );
    }
}
