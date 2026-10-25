// RBAC matrix — reports (signalements).
// Create is open to any authenticated user (resident-initiative pattern);
// update / delete are manager + board_member (204).
// Status change and convert-to-task are Syndic-only (PRD Signalements
// matrice CRUD: S² réservé au Syndic, Q-SIG-003).

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
    assert_eq!(
        created.status_code(),
        StatusCode::CREATED,
        "setup create report"
    );
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
    assert_eq!(updated.status_code(), StatusCode::NO_CONTENT);

    let status = server
        .put(&format!("/api/v1/reports/{report_id}/status"))
        .add_header("Authorization", &auth)
        .json(&json!({ "status": "in_progress" }))
        .await;
    assert_eq!(
        status.status_code(),
        StatusCode::FORBIDDEN,
        "board_member change status"
    );

    let converted = server
        .post(&format!("/api/v1/reports/{report_id}/task"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(
        converted.status_code(),
        StatusCode::FORBIDDEN,
        "board_member convert"
    );

    let deleted = server
        .delete(&format!("/api/v1/reports/{report_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(deleted.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn residents_can_create_but_not_manage_reports() {
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
            .json(&json!({ "title": "Fiche résident" }))
            .await;
        assert_eq!(created.status_code(), StatusCode::CREATED);

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
