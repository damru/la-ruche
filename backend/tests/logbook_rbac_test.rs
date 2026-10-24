// RBAC matrix — maintenance logbook: manager + board_member create/update;
// delete is manager-only; all authenticated users may read.

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
    assert!(created.status_code().is_success(), "setup create entry");
    created.json::<Value>()["id"]
        .as_str()
        .expect("id")
        .to_string()
}

#[tokio::test]
async fn board_member_can_create_update_but_not_delete_entries() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.board_member_jwt);

    let entry_id = create_entry(&server, &auth).await;

    let updated = server
        .put(&format!("/api/v1/logbook/{entry_id}"))
        .add_header("Authorization", &auth)
        .json(&entry_payload())
        .await;
    assert_eq!(updated.status_code(), StatusCode::OK);

    let deleted = server
        .delete(&format!("/api/v1/logbook/{entry_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(deleted.status_code(), StatusCode::FORBIDDEN);
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

        let updated = server
            .put(&format!("/api/v1/logbook/{entry_id}"))
            .add_header("Authorization", &auth)
            .json(&entry_payload())
            .await;
        assert_eq!(updated.status_code(), StatusCode::FORBIDDEN);
    }
}

#[tokio::test]
async fn manager_can_delete_entries() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);

    let entry_id = create_entry(&server, &auth).await;

    let deleted = server
        .delete(&format!("/api/v1/logbook/{entry_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(deleted.status_code(), StatusCode::NO_CONTENT);
}
