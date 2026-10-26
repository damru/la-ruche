// EP-009-S10 — logbook category (PRD US-CE-005).
// 1. POST with an explicit category is echoed back on GET.
// 2. POST without a category defaults to `autre`.
// 3. An invalid category is rejected with 400 (no raw SQL cast error leak).
// 4. Closing a task generates a logbook entry with category `travaux`.

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{Value, json};

async fn create_entry(server: &TestServer, auth: &str, payload: Value) -> axum_test::TestResponse {
    server
        .post("/api/v1/logbook")
        .add_header("Authorization", auth)
        .json(&payload)
        .await
}

#[tokio::test]
async fn category_is_echoed_and_defaults_to_autre() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);

    // Explicit category is persisted and returned.
    let created = create_entry(
        &server,
        &auth,
        json!({
            "title": "Remplacement chaudière",
            "intervention_date": "2026-02-10",
            "category": "contrat"
        }),
    )
    .await;
    assert_eq!(created.status_code(), StatusCode::CREATED);
    let body = created.json::<Value>();
    assert_eq!(body["category"], "contrat");
    let entry_id = body["id"].as_str().expect("id").to_string();

    let fetched = server
        .get(&format!("/api/v1/logbook/{entry_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(fetched.status_code(), StatusCode::OK);
    assert_eq!(fetched.json::<Value>()["category"], "contrat");

    // No category → defaults to `autre`.
    let defaulted = create_entry(
        &server,
        &auth,
        json!({
            "title": "Contrôle extincteurs",
            "intervention_date": "2026-02-11"
        }),
    )
    .await;
    assert_eq!(defaulted.status_code(), StatusCode::CREATED);
    assert_eq!(defaulted.json::<Value>()["category"], "autre");
}

#[tokio::test]
async fn invalid_category_is_rejected_with_400() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);

    let rejected = create_entry(
        &server,
        &auth,
        json!({
            "title": "Entrée invalide",
            "intervention_date": "2026-02-10",
            "category": "n'importe_quoi"
        }),
    )
    .await;
    assert_eq!(rejected.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
        rejected.json::<Value>()["code"],
        "errors.maintenance.invalid_category"
    );
}

#[tokio::test]
async fn task_closure_generates_travaux_category_entry() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);

    let created = server
        .post("/api/v1/tasks")
        .add_header("Authorization", &auth)
        .json(&json!({ "title": "Réparer la porte d'entrée" }))
        .await;
    assert!(created.status_code().is_success(), "setup create task");
    let task_id = created.json::<Value>()["id"]
        .as_str()
        .expect("id")
        .to_string();

    let closed = server
        .put(&format!("/api/v1/tasks/{task_id}"))
        .add_header("Authorization", &auth)
        .json(&json!({ "status": "closed" }))
        .await;
    assert_eq!(closed.status_code(), StatusCode::NO_CONTENT);

    let listed = server
        .get("/api/v1/logbook")
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(listed.status_code(), StatusCode::OK);
    let entries = listed.json::<Value>();
    let entry = entries
        .as_array()
        .expect("array")
        .iter()
        .find(|entry| entry["task_id"] == task_id)
        .expect("logbook entry generated from closed task");
    assert_eq!(entry["source"], "automatic");
    assert_eq!(entry["category"], "travaux");
}
