// RBAC matrix — contractors referential: manager + board_member manage;
// read is manager + board_member too.

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{Value, json};

fn contractor_payload() -> Value {
    json!({
        "name": "Plombier Pro",
        "domains": ["plumbing"],
        "phone": "0102030405"
    })
}

async fn create_contractor(server: &TestServer, auth: &str) -> String {
    let created = server
        .post("/api/v1/contractors")
        .add_header("Authorization", auth)
        .json(&contractor_payload())
        .await;
    assert!(
        created.status_code().is_success(),
        "setup create contractor"
    );
    created.json::<Value>()["id"]
        .as_str()
        .expect("id")
        .to_string()
}

#[tokio::test]
async fn board_member_can_manage_contractors() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.board_member_jwt);

    let contractor_id = create_contractor(&server, &auth).await;

    let updated = server
        .put(&format!("/api/v1/contractors/{contractor_id}"))
        .add_header("Authorization", &auth)
        .json(&json!({ "name": "Plombier Pro", "domains": ["plumbing"], "phone": "0102030406" }))
        .await;
    assert_eq!(updated.status_code(), StatusCode::OK);

    let deleted = server
        .delete(&format!("/api/v1/contractors/{contractor_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(deleted.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn residents_cannot_access_contractors() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let mgr_auth = format!("Bearer {}", ctx.manager_jwt);
    let contractor_id = create_contractor(&server, &mgr_auth).await;

    for jwt in [&ctx.coowner_jwt, &ctx.tenant_jwt] {
        let auth = format!("Bearer {jwt}");

        let listed = server
            .get("/api/v1/contractors")
            .add_header("Authorization", &auth)
            .await;
        assert_eq!(listed.status_code(), StatusCode::FORBIDDEN);

        let created = server
            .post("/api/v1/contractors")
            .add_header("Authorization", &auth)
            .json(&contractor_payload())
            .await;
        assert_eq!(created.status_code(), StatusCode::FORBIDDEN);

        let deleted = server
            .delete(&format!("/api/v1/contractors/{contractor_id}"))
            .add_header("Authorization", &auth)
            .await;
        assert_eq!(deleted.status_code(), StatusCode::FORBIDDEN);
    }
}
