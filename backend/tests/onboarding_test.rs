// HTTP integration tests — onboarding, authentication, protected routes, vendor flow, AGPL notice.
// Requires a PostgreSQL test database (DATABASE_URL).
// Run with: cargo test --test onboarding_test

mod common;

use axum::http::StatusCode;
use serde_json::{Value, json};

// ── Onboarding ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_onboarding_status_returns_needs_onboarding() {
    let server = common::test_server().await;
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
    let server = common::test_server().await;
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
    let server = common::test_server().await;
    let mut payload = common::bootstrap_payload();
    payload["shares_total"] = json!(0);
    let resp = server
        .post("/api/v1/onboarding/bootstrap")
        .json(&payload)
        .await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
}

// ── Authentication ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_login_wrong_password_returns_401() {
    let server = common::test_server().await;
    let resp = server
        .post("/api/v1/auth/login")
        .json(&json!({ "email": "inconnu@test.fr", "password": "wrongpass" }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_invalid_input_returns_400() {
    let server = common::test_server().await;
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
    let server = common::test_server().await;
    let resp = server.post("/api/v1/auth/refresh").await;
    assert_eq!(resp.status_code(), StatusCode::UNAUTHORIZED);
}

// ── Protected routes without auth ────────────────────────────────────────────

#[tokio::test]
async fn test_protected_implemented_routes_without_jwt_return_401() {
    let server = common::test_server().await;
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

// ── Vendor flow (public route) ────────────────────────────────────────────────

#[tokio::test]
async fn test_vendor_token_not_found_returns_404() {
    let server = common::test_server().await;
    let fake_token = uuid::Uuid::new_v4();
    let resp = server.get(&format!("/vendor/{}", fake_token)).await;
    assert_eq!(resp.status_code(), StatusCode::NOT_FOUND);
}

// ── AGPL notice ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_notice_agpl_is_accessible() {
    let server = common::test_server().await;
    let resp = server.get("/notice").await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let body = resp.text();
    assert!(
        body.contains("AGPL"),
        "The NOTICE endpoint must mention the AGPL license"
    );
}
