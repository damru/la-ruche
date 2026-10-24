// RBAC matrix — reports (conformité): manager + board_member manage;
// all authenticated users may read.

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{Value, json};

async fn create_report(server: &TestServer, auth: &str) -> String {
    let created = server
        .post("/api/v1/reports")
        .add_header("Authorization", auth)
        .json(&json!({ "title": "Fiche RNC 2026" }))
        .await;
    assert!(created.status_code().is_success(), "setup create report");
    created.json::<Value>()["id"]
        .as_str()
        .expect("id")
        .to_string()
}

#[tokio::test]
async fn board_member_can_manage_reports() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.board_member_jwt);

    let report_id = create_report(&server, &auth).await;

    let updated = server
        .put(&format!("/api/v1/reports/{report_id}"))
        .add_header("Authorization", &auth)
        .json(&json!({ "title": "Fiche RNC 2026 (maj)" }))
        .await;
    assert_eq!(updated.status_code(), StatusCode::OK);

    let deleted = server
        .delete(&format!("/api/v1/reports/{report_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(deleted.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn residents_cannot_manage_reports() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let mgr_auth = format!("Bearer {}", ctx.manager_jwt);
    let report_id = create_report(&server, &mgr_auth).await;

    for jwt in [&ctx.coowner_jwt, &ctx.tenant_jwt] {
        let auth = format!("Bearer {jwt}");

        let listed = server
            .get("/api/v1/reports")
            .add_header("Authorization", &auth)
            .await;
        assert_eq!(listed.status_code(), StatusCode::OK);

        let created = server
            .post("/api/v1/reports")
            .add_header("Authorization", &auth)
            .json(&json!({ "title": "Interdit" }))
            .await;
        assert_eq!(created.status_code(), StatusCode::FORBIDDEN);

        let updated = server
            .put(&format!("/api/v1/reports/{report_id}"))
            .add_header("Authorization", &auth)
            .json(&json!({ "title": "Interdit" }))
            .await;
        assert_eq!(updated.status_code(), StatusCode::FORBIDDEN);

        let deleted = server
            .delete(&format!("/api/v1/reports/{report_id}"))
            .add_header("Authorization", &auth)
            .await;
        assert_eq!(deleted.status_code(), StatusCode::FORBIDDEN);
    }
}
