// RBAC matrix — tasks: manager + board_member manage (C/U/D),
// co_owner / tenant read-only. Views are open to any authenticated user.

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{Value, json};

async fn create_task(server: &TestServer, auth: &str) -> String {
    let created = server
        .post("/api/v1/tasks")
        .add_header("Authorization", auth)
        .json(&json!({ "title": "Remplacer l'ampoule" }))
        .await;
    assert!(created.status_code().is_success(), "setup create task");
    created.json::<Value>()["id"]
        .as_str()
        .expect("id")
        .to_string()
}

#[tokio::test]
async fn board_member_can_manage_tasks() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.board_member_jwt);

    let task_id = create_task(&server, &auth).await;

    let updated = server
        .put(&format!("/api/v1/tasks/{task_id}"))
        .add_header("Authorization", &auth)
        .json(&json!({ "status": "in_progress" }))
        .await;
    assert_eq!(updated.status_code(), StatusCode::NO_CONTENT);

    let deleted = server
        .delete(&format!("/api/v1/tasks/{task_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(deleted.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn residents_cannot_manage_tasks() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let mgr_auth = format!("Bearer {}", ctx.manager_jwt);
    let task_id = create_task(&server, &mgr_auth).await;

    for jwt in [&ctx.coowner_jwt, &ctx.tenant_jwt] {
        let auth = format!("Bearer {jwt}");

        let listed = server
            .get("/api/v1/tasks")
            .add_header("Authorization", &auth)
            .await;
        assert_eq!(listed.status_code(), StatusCode::OK);

        let created = server
            .post("/api/v1/tasks")
            .add_header("Authorization", &auth)
            .json(&json!({ "title": "Interdit" }))
            .await;
        assert_eq!(created.status_code(), StatusCode::FORBIDDEN);

        let updated = server
            .put(&format!("/api/v1/tasks/{task_id}"))
            .add_header("Authorization", &auth)
            .json(&json!({ "status": "in_progress" }))
            .await;
        assert_eq!(updated.status_code(), StatusCode::FORBIDDEN);

        let deleted = server
            .delete(&format!("/api/v1/tasks/{task_id}"))
            .add_header("Authorization", &auth)
            .await;
        assert_eq!(deleted.status_code(), StatusCode::FORBIDDEN);
    }
}

#[tokio::test]
async fn vendor_token_endpoints_are_manager_only() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let mgr_auth = format!("Bearer {}", ctx.manager_jwt);
    let task_id = create_task(&server, &mgr_auth).await;

    let board_auth = format!("Bearer {}", ctx.board_member_jwt);
    let resp = server
        .post(&format!("/api/v1/tasks/{task_id}/vendor-token"))
        .add_header("Authorization", &board_auth)
        .await;
    assert_eq!(resp.status_code(), StatusCode::FORBIDDEN);
}
