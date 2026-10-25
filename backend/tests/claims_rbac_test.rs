// RBAC matrix — claims (signalements): any authenticated user may create and
// read; update/delete are manager + board_member.

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{Value, json};

fn claim_payload() -> Value {
    json!({
        "title": "Fuite hall B",
        "claim_type": "current",
        "declared_at": "2026-01-15"
    })
}

async fn create_claim(server: &TestServer, auth: &str) -> String {
    let created = server
        .post("/api/v1/claims")
        .add_header("Authorization", auth)
        .json(&claim_payload())
        .await;
    assert!(created.status_code().is_success(), "setup create claim");
    created.json::<Value>()["id"]
        .as_str()
        .expect("id")
        .to_string()
}

#[tokio::test]
async fn residents_can_create_but_not_update_claims() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let mgr_auth = format!("Bearer {}", ctx.manager_jwt);
    let claim_id = create_claim(&server, &mgr_auth).await;

    for jwt in [&ctx.coowner_jwt, &ctx.tenant_jwt] {
        let auth = format!("Bearer {jwt}");

        let created = server
            .post("/api/v1/claims")
            .add_header("Authorization", &auth)
            .json(&claim_payload())
            .await;
        assert_eq!(created.status_code(), StatusCode::CREATED);

        let listed = server
            .get("/api/v1/claims")
            .add_header("Authorization", &auth)
            .await;
        assert_eq!(listed.status_code(), StatusCode::OK);

        let updated = server
            .put(&format!("/api/v1/claims/{claim_id}"))
            .add_header("Authorization", &auth)
            .json(&json!({
                "title": "Fuite hall B",
                "claim_type": "current",
                "declared_at": "2026-01-15",
                "status": "in_progress"
            }))
            .await;
        assert_eq!(updated.status_code(), StatusCode::FORBIDDEN);

        let deleted = server
            .delete(&format!("/api/v1/claims/{claim_id}"))
            .add_header("Authorization", &auth)
            .await;
        assert_eq!(deleted.status_code(), StatusCode::FORBIDDEN);
    }
}

#[tokio::test]
async fn board_member_can_update_and_delete_claims() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let mgr_auth = format!("Bearer {}", ctx.manager_jwt);
    let claim_id = create_claim(&server, &mgr_auth).await;

    let auth = format!("Bearer {}", ctx.board_member_jwt);
    let updated = server
        .put(&format!("/api/v1/claims/{claim_id}"))
        .add_header("Authorization", &auth)
        .json(&json!({
            "title": "Fuite hall B",
            "claim_type": "current",
            "declared_at": "2026-01-15",
            "status": "in_progress"
        }))
        .await;
    assert_eq!(updated.status_code(), StatusCode::OK);

    // NOTE: PRD Sinistres gives Conseil `R` only, but the handler
    // (pre-existing) grants update/delete to Manager + BoardMember.
    // This test pins the current implementation.
    let deleted = server
        .delete(&format!("/api/v1/claims/{claim_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(deleted.status_code(), StatusCode::NO_CONTENT);
}
