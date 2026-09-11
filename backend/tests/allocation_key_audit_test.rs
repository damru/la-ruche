// EP-001-S10: Audit trail for allocation-key mutations (LEGAL.md §1.2, §2.1).

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{Value, json};

async fn audit_test_server() -> TestServer {
    let mut server = common::isolated_test_server().await;
    let bootstrap = server
        .post("/api/v1/onboarding/bootstrap")
        .json(&common::bootstrap_payload())
        .await;
    assert_eq!(bootstrap.status_code(), StatusCode::CREATED);
    server.add_cookies(bootstrap.cookies());
    server
}

async fn create_unit(server: &TestServer, unit_number: &str) -> String {
    let created = server
        .post("/api/v1/units")
        .json(&json!({ "unit_number": unit_number, "unit_type": "apartment" }))
        .await;
    assert_eq!(created.status_code(), StatusCode::CREATED);
    created.json::<Value>()["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn create_update_share_delete_writes_audit_trail() {
    let server = audit_test_server().await;
    let unit_id = create_unit(&server, "A001").await;

    let created = server
        .post("/api/v1/building/keys")
        .json(&json!({
            "name": "Ascenseur",
            "description": "Clé spéciale",
            "justification": "Nouvelle clé pour l'ascenseur",
        }))
        .await;
    assert_eq!(created.status_code(), StatusCode::CREATED);
    let key_id = created.json::<Value>()["id"].as_str().unwrap().to_string();

    let updated = server
        .put(&format!("/api/v1/building/keys/{key_id}"))
        .json(&json!({
            "name": "Ascenseur principal",
            "description": "Clé spéciale",
            "justification": "Renommage suite à AG",
        }))
        .await;
    assert_eq!(updated.status_code(), StatusCode::OK);

    let shares_set = server
        .put(&format!("/api/v1/building/keys/{key_id}/shares"))
        .json(&json!({
            "shares": [{ "unit_id": unit_id, "share_value": 100 }],
            "justification": "Répartition initiale",
        }))
        .await;
    assert_eq!(shares_set.status_code(), StatusCode::NO_CONTENT);

    let shares_set_again = server
        .put(&format!("/api/v1/building/keys/{key_id}/shares"))
        .json(&json!({
            "shares": [{ "unit_id": unit_id, "share_value": 100 }],
            "justification": "Aucun changement",
        }))
        .await;
    assert_eq!(shares_set_again.status_code(), StatusCode::NO_CONTENT);

    let deleted = server
        .delete(&format!("/api/v1/building/keys/{key_id}"))
        .json(&json!({ "justification": "Clé obsolète" }))
        .await;
    assert_eq!(deleted.status_code(), StatusCode::NO_CONTENT);

    let audit = server
        .get(&format!("/api/v1/building/keys/{key_id}/audit"))
        .await;
    assert_eq!(audit.status_code(), StatusCode::OK);
    let entries: Value = audit.json();
    let entries = entries.as_array().expect("array");
    assert_eq!(
        entries.len(),
        4,
        "expected create + update + share_set + delete, got {entries:?}"
    );

    let operations: Vec<&str> = entries
        .iter()
        .map(|e| e["operation"].as_str().unwrap())
        .collect();
    assert!(operations.contains(&"allocation_key.created"));
    assert!(operations.contains(&"allocation_key.updated"));
    assert!(operations.contains(&"allocation_key.share_set"));
    assert!(operations.contains(&"allocation_key.deleted"));

    let created_entry = entries
        .iter()
        .find(|e| e["operation"] == "allocation_key.created")
        .expect("created entry");
    assert_eq!(
        created_entry["new_values"]["justification"],
        "Nouvelle clé pour l'ascenseur"
    );
}

#[tokio::test]
async fn justification_over_limit_is_rejected() {
    let server = audit_test_server().await;
    let long_justification = "x".repeat(501);

    let created = server
        .post("/api/v1/building/keys")
        .json(&json!({
            "name": "Clé test",
            "justification": long_justification,
        }))
        .await;
    assert_eq!(created.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn manager_can_read_audit_endpoint() {
    let server = audit_test_server().await;
    let created = server
        .post("/api/v1/building/keys")
        .json(&json!({ "name": "Clé test" }))
        .await;
    assert_eq!(created.status_code(), StatusCode::CREATED);
    let key_id = created.json::<Value>()["id"].as_str().unwrap().to_string();

    let audit = server
        .get(&format!("/api/v1/building/keys/{key_id}/audit"))
        .await;
    assert_eq!(audit.status_code(), StatusCode::OK);
}
