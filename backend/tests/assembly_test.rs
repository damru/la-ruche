// HTTP + DB integration tests — General Assembly (AG) legal cases.
// Tests cover all majority rules of the French condominium act (loi du 10 juillet 1965):
//   Art. 24 — simple majority of votes expressed
//   Art. 25 — absolute majority of all co-owners
//   Art. 25-1 — second-vote passerelle when Art. 25 fails above 1/3
//   Art. 26 — double majority (2/3 shares + 2/3 owners)
//   Unanimity
//
// Requires a live PostgreSQL database (DATABASE_URL).
// Run with: cargo test --test assembly_test
//
// Share distribution: A = 500 shares, B = 300 shares, C = 200 shares → total = 1000.
// All cases are grouped in one `#[tokio::test]` to avoid parallel mutations on the
// shared building state.

mod common;

use axum::http::{StatusCode, header::AUTHORIZATION};
use axum_test::TestServer;
use serde_json::{Value, json};

// ── Shared test environment ───────────────────────────────────────────────────

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
    let server = common::test_server().await;

    // Try bootstrap; safe to ignore 409 (already initialized)
    server
        .post("/api/v1/onboarding/bootstrap")
        .json(&common::bootstrap_payload())
        .await;

    let login: Value = server
        .post("/api/v1/auth/login")
        .json(&json!({ "email": "syndic@test.fr", "password": "MotDePasse123!" }))
        .await
        .json();

    let user_id = uuid::Uuid::parse_str(
        login["user_id"]
            .as_str()
            .expect("user_id in login response"),
    )
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
    let rid = uuid::Uuid::parse_str(res["id"].as_str().expect("resolution id")).expect("UUID");

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

// ── Tests ─────────────────────────────────────────────────────────────────────

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
        assert_eq!(
            result["shares_counted"],
            json!(800),
            "Art.24: shares_counted"
        );
        assert_eq!(
            result["threshold"],
            json!(401),
            "Art.24: threshold = 800/2+1"
        );
        assert_eq!(
            result["passed"],
            json!(true),
            "Art.24: 500 >= 401 must pass"
        );
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
        assert_eq!(
            result["threshold"],
            json!(501),
            "Art.25: threshold = 1000/2+1"
        );
        assert_eq!(
            result["passed"],
            json!(true),
            "Art.25: 800 >= 501 must pass"
        );
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
        assert_eq!(
            result["threshold"],
            json!(667),
            "Art.26: (1000*2+2)/3 = 667"
        );
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
        assert_eq!(
            result["passed"],
            json!(true),
            "1000/1000 in favour → unanimity"
        );
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

        assert_eq!(
            result["passed"],
            json!(false),
            "abstention breaks unanimity"
        );
        assert_eq!(result["eligible_for_second_vote"], Value::Null);
    }

    // ── Protected route — results returns 401 without auth ───────────────────
    {
        let (mid, rid) =
            create_resolution_with_votes(&env, "simple_majority", &[(env.uid_a, "in_favor")]).await;
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
