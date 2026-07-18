// PATCH /auth/me — personal info update (email + phone only), PRD US-PP-002.

mod common;

use axum::http::StatusCode;
use serde_json::{Value, json};

/// Boots an isolated server, bootstraps a manager, returns the session-ready server.
async fn profile_test_server() -> axum_test::TestServer {
    let mut server = common::isolated_test_server().await;
    let bootstrap = server
        .post("/api/v1/onboarding/bootstrap")
        .json(&common::bootstrap_payload())
        .await;
    assert_eq!(bootstrap.status_code(), StatusCode::CREATED);
    server.add_cookies(bootstrap.cookies());
    server
}

#[tokio::test]
async fn update_me_valid_email_and_phone() {
    let server = profile_test_server().await;

    let resp = server
        .patch("/api/v1/auth/me")
        .json(&json!({
            "email": "new-email@test.fr",
            "phone": "0611223344"
        }))
        .await;
    assert_eq!(
        resp.status_code(),
        StatusCode::OK,
        "update failed: {}",
        resp.text()
    );
    let body: Value = resp.json();
    assert_eq!(body["email"], "new-email@test.fr");
    assert_eq!(body["phone"], "0611223344");
    // full_name stays prefilled/unchanged
    assert_eq!(body["first_name"], "Sophie");
    assert_eq!(body["last_name"], "Martin");

    // Persisted: /auth/me reflects the change on a fresh read.
    let me = server.get("/api/v1/auth/me").await;
    assert_eq!(me.status_code(), StatusCode::OK);
    let me_body: Value = me.json();
    assert_eq!(me_body["email"], "new-email@test.fr");
    assert_eq!(me_body["phone"], "0611223344");
}

#[tokio::test]
async fn update_me_rejects_duplicate_email() {
    let server = profile_test_server().await;

    // Create a second resident with a distinct email via the manager-only endpoint.
    let created = server
        .post("/api/v1/users")
        .json(&json!({
            "first_name": "Jean",
            "last_name": "Dupont",
            "email": "jean.dupont@test.fr",
            "role": "co_owner"
        }))
        .await;
    assert_eq!(
        created.status_code(),
        StatusCode::CREATED,
        "create resident failed: {}",
        created.text()
    );

    // The bootstrapped manager (currently syndic@test.fr) tries to take that email.
    let resp = server
        .patch("/api/v1/auth/me")
        .json(&json!({ "email": "jean.dupont@test.fr" }))
        .await;
    assert_eq!(
        resp.status_code(),
        StatusCode::CONFLICT,
        "expected conflict, got {}: {}",
        resp.status_code(),
        resp.text()
    );
}

#[tokio::test]
async fn update_me_rejects_full_name_modification() {
    let server = profile_test_server().await;

    let resp = server
        .patch("/api/v1/auth/me")
        .json(&json!({
            "phone": "0611223344",
            "first_name": "Autre",
        }))
        .await;
    assert_eq!(
        resp.status_code(),
        StatusCode::FORBIDDEN,
        "expected forbidden, got {}: {}",
        resp.status_code(),
        resp.text()
    );

    let resp = server
        .patch("/api/v1/auth/me")
        .json(&json!({
            "phone": "0611223344",
            "last_name": "Autre",
        }))
        .await;
    assert_eq!(
        resp.status_code(),
        StatusCode::FORBIDDEN,
        "expected forbidden, got {}: {}",
        resp.status_code(),
        resp.text()
    );

    // Sending back the current (unchanged) full name must succeed (AC-PP-002-B).
    let resp = server
        .patch("/api/v1/auth/me")
        .json(&json!({
            "phone": "0699887766",
            "first_name": "Sophie",
            "last_name": "Martin",
        }))
        .await;
    assert_eq!(
        resp.status_code(),
        StatusCode::OK,
        "unchanged full_name should be accepted: {}",
        resp.text()
    );
}
