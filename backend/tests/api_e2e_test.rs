// End-to-end API tests — nominal (happy-path) flows with result assertions.
//
// Exercises the full HTTP stack against the self-contained harness (embedded
// PostgreSQL + wiremock S3). The whole flow runs as a single sequential test
// because instance bootstrap is a once-per-database operation: splitting it
// across parallel tests would race on the first-run guard.

mod common;

use axum::http::StatusCode;
use serde_json::{Value, json};

#[tokio::test]
async fn nominal_api_flow() {
    let mut server = common::isolated_test_server().await;

    // ── Onboarding: fresh instance needs onboarding ───────────────────
    let status = server.get("/api/v1/onboarding/status").await;
    assert_eq!(status.status_code(), StatusCode::OK);
    assert_eq!(status.json::<Value>()["needs_onboarding"], true);

    // ── Bootstrap: creates building + first manager, returns a session ─
    let bootstrap = server
        .post("/api/v1/onboarding/bootstrap")
        .json(&common::bootstrap_payload())
        .await;
    assert_eq!(bootstrap.status_code(), StatusCode::CREATED);
    let bootstrap_body: Value = bootstrap.json();
    assert!(bootstrap_body["user_id"].is_string());
    assert!(bootstrap_body["building_id"].is_string());
    assert_eq!(bootstrap_body["roles"], json!(["manager"]));
    // Explicitly carry session cookies forward; belt-and-suspenders over save_cookies().
    server.add_cookies(bootstrap.cookies());

    // ── Onboarding status flips to false once a user exists ───────────
    let status_after = server.get("/api/v1/onboarding/status").await;
    assert_eq!(status_after.status_code(), StatusCode::OK);
    assert_eq!(status_after.json::<Value>()["needs_onboarding"], false);

    // ── Authenticated session carries over via the saved cookie ───────
    let me = server.get("/api/v1/auth/me").await;
    assert_eq!(
        me.status_code(),
        StatusCode::OK,
        "auth/me failed — status {} body: {}",
        me.status_code(),
        me.text()
    );
    let me_body: Value = me.json();
    assert_eq!(me_body["email"], "syndic@test.fr");
    assert_eq!(me_body["roles"], json!(["manager"]));

    // ── Building reflects the bootstrap payload ───────────────────────
    let building = server.get("/api/v1/building").await;
    assert_eq!(building.status_code(), StatusCode::OK);
    let building_body: Value = building.json();
    assert_eq!(building_body["name"], "Résidence Les Tests");
    assert_eq!(building_body["shares_total"], 1000);
    assert_eq!(building_body["city"], "Paris");

    // ── Units: create → list → get ────────────────────────────────────
    let created_unit = server
        .post("/api/v1/units")
        .json(&json!({
            "unit_number": "A001",
            "unit_type": "apartment",
            "floor": "2"
        }))
        .await;
    assert_eq!(created_unit.status_code(), StatusCode::CREATED);
    let unit_body: Value = created_unit.json();
    let unit_id = unit_body["id"].as_str().expect("unit id").to_string();
    assert_eq!(unit_body["unit_number"], "A001");
    assert_eq!(unit_body["unit_type"], "apartment");

    let units = server.get("/api/v1/units").await;
    assert_eq!(units.status_code(), StatusCode::OK);
    let units_body: Value = units.json();
    let units_arr = units_body.as_array().expect("units array");
    assert_eq!(units_arr.len(), 1);
    assert_eq!(units_arr[0]["id"], unit_id);

    let unit = server.get(&format!("/api/v1/units/{unit_id}")).await;
    assert_eq!(unit.status_code(), StatusCode::OK);
    assert_eq!(unit.json::<Value>()["unit_number"], "A001");

    // ── Governance: create a mandate → list ───────────────────────────
    let mandate = server
        .post("/api/v1/governance/mandates")
        .json(&json!({
            "role": "council_member",
            "holder_name": "Mme Bernard",
            "start_date": "2026-01-01"
        }))
        .await;
    assert_eq!(mandate.status_code(), StatusCode::CREATED);
    assert_eq!(mandate.json::<Value>()["role"], "council_member");

    let mandates = server.get("/api/v1/governance/mandates").await;
    assert_eq!(mandates.status_code(), StatusCode::OK);
    assert_eq!(mandates.json::<Value>().as_array().expect("array").len(), 1);
}

#[tokio::test]
async fn login_with_invalid_credentials_is_unauthorized() {
    let server = common::test_server().await;
    let resp = server
        .post("/api/v1/auth/login")
        .json(&json!({ "email": "nobody@test.fr", "password": "WrongPassword1!" }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::UNAUTHORIZED);
}
