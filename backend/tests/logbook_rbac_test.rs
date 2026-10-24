// RBAC matrix — maintenance logbook. Since EP-018-S07 (Pattern C, append-only),
// manual entries are immutable: PUT/DELETE return 400 errors.immutability.frozen
// for every role — the only amendment path is POST /correction (append-only).
// Permission layer: create + correction are manager + board_member; residents
// read-only.

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{Value, json};

fn entry_payload() -> Value {
    json!({
        "title": "Entretien annuel chaudière",
        "intervention_date": "2026-01-15"
    })
}

async fn create_entry(server: &TestServer, auth: &str) -> String {
    let created = server
        .post("/api/v1/logbook")
        .add_header("Authorization", auth)
        .json(&entry_payload())
        .await;
    assert_eq!(
        created.status_code(),
        StatusCode::CREATED,
        "setup create entry"
    );
    created.json::<Value>()["id"]
        .as_str()
        .expect("id")
        .to_string()
}

#[tokio::test]
async fn board_member_can_create_but_not_mutate_entries() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.board_member_jwt);

    let entry_id = create_entry(&server, &auth).await;

    // Entries are frozen (EP-018-S07 Pattern C) — update/delete are 400 for
    // everyone; the amendment path is the append-only correction.
    let updated = server
        .put(&format!("/api/v1/logbook/{entry_id}"))
        .add_header("Authorization", &auth)
        .json(&entry_payload())
        .await;
    assert_eq!(updated.status_code(), StatusCode::BAD_REQUEST);

    let deleted = server
        .delete(&format!("/api/v1/logbook/{entry_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(deleted.status_code(), StatusCode::BAD_REQUEST);

    let corrected = server
        .post(&format!("/api/v1/logbook/{entry_id}/correction"))
        .add_header("Authorization", &auth)
        .json(&json!({
            "title": "Entretien annuel chaudière (corrigé)",
            "intervention_date": "2026-01-16"
        }))
        .await;
    assert_eq!(corrected.status_code(), StatusCode::CREATED);
}

#[tokio::test]
async fn residents_can_read_but_not_write_logbook() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let mgr_auth = format!("Bearer {}", ctx.manager_jwt);
    let entry_id = create_entry(&server, &mgr_auth).await;

    for jwt in [&ctx.coowner_jwt, &ctx.tenant_jwt] {
        let auth = format!("Bearer {jwt}");

        let listed = server
            .get("/api/v1/logbook")
            .add_header("Authorization", &auth)
            .await;
        assert_eq!(listed.status_code(), StatusCode::OK);

        let created = server
            .post("/api/v1/logbook")
            .add_header("Authorization", &auth)
            .json(&entry_payload())
            .await;
        assert_eq!(created.status_code(), StatusCode::FORBIDDEN);

        let corrected = server
            .post(&format!("/api/v1/logbook/{entry_id}/correction"))
            .add_header("Authorization", &auth)
            .json(&json!({
                "title": "Interdit",
                "intervention_date": "2026-01-16"
            }))
            .await;
        assert_eq!(corrected.status_code(), StatusCode::FORBIDDEN);
    }
}

#[tokio::test]
async fn manager_delete_is_frozen_for_everyone() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);

    let entry_id = create_entry(&server, &auth).await;

    // Even the manager cannot hard-delete: entries are append-only
    // (EP-018-S07 Pattern C). The permission layer would allow it, the
    // immutability domain rule does not.
    let deleted = server
        .delete(&format!("/api/v1/logbook/{entry_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(deleted.status_code(), StatusCode::BAD_REQUEST);
}
