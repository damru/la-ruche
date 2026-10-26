// RBAC matrix — claims: any authenticated user may create and read;// update/delete are manager-only. Document endpoints require// Manager | BoardMember. Notes require Manager | BoardMember.

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{Value, json};

fn claim_payload() -> Value {
    json!({
        "title": "Fuite hall B",
        "claim_type": "current"
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

async fn declare_claim(server: &TestServer, auth: &str, claim_id: &str) {
    let declared = server
        .post(&format!("/api/v1/claims/{claim_id}/declare"))
        .add_header("Authorization", auth)
        .await;
    assert_eq!(
        declared.status_code(),
        StatusCode::OK,
        "setup declare claim"
    );
}

// ── Draft lifecycle ────────────────────────────────────────────────

#[tokio::test]
async fn new_claim_is_created_as_draft() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);

    let created = server
        .post("/api/v1/claims")
        .add_header("Authorization", &auth)
        .json(&claim_payload())
        .await;
    assert_eq!(created.status_code(), StatusCode::CREATED);
    let body = created.json::<Value>();
    assert_eq!(body["status"], "draft");
    assert!(body["declared_at"].is_null());
}

#[tokio::test]
async fn drafts_are_invisible_to_non_managers() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let mgr_auth = format!("Bearer {}", ctx.manager_jwt);
    let claim_id = create_claim(&server, &mgr_auth).await;

    for jwt in [&ctx.coowner_jwt, &ctx.tenant_jwt, &ctx.board_member_jwt] {
        let auth = format!("Bearer {jwt}");
        let listed = server
            .get("/api/v1/claims")
            .add_header("Authorization", &auth)
            .await;
        assert_eq!(listed.status_code(), StatusCode::OK);
        let claims = listed.json::<Vec<Value>>();
        assert!(
            !claims.iter().any(|c| c["id"].as_str() == Some(&claim_id)),
            "draft should not appear in list for non-manager"
        );

        let got = server
            .get(&format!("/api/v1/claims/{claim_id}"))
            .add_header("Authorization", &auth)
            .await;
        assert_eq!(got.status_code(), StatusCode::NOT_FOUND);
    }

    let listed = server
        .get("/api/v1/claims")
        .add_header("Authorization", &mgr_auth)
        .await;
    let claims = listed.json::<Vec<Value>>();
    assert!(claims.iter().any(|c| c["id"].as_str() == Some(&claim_id)));
}

#[tokio::test]
async fn declare_transitions_draft_to_open() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let mgr_auth = format!("Bearer {}", ctx.manager_jwt);
    let claim_id = create_claim(&server, &mgr_auth).await;

    let declared = server
        .post(&format!("/api/v1/claims/{claim_id}/declare"))
        .add_header("Authorization", &mgr_auth)
        .await;
    assert_eq!(declared.status_code(), StatusCode::OK);
    let body = declared.json::<Value>();
    assert_eq!(body["status"], "open");
    assert!(!body["declared_at"].is_null());

    let redeclared = server
        .post(&format!("/api/v1/claims/{claim_id}/declare"))
        .add_header("Authorization", &mgr_auth)
        .await;
    assert_eq!(redeclared.status_code(), StatusCode::BAD_REQUEST);

    let co_auth = format!("Bearer {}", ctx.coowner_jwt);
    let got = server
        .get(&format!("/api/v1/claims/{claim_id}"))
        .add_header("Authorization", &co_auth)
        .await;
    assert_eq!(got.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn declare_rejects_future_date() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let mgr_auth = format!("Bearer {}", ctx.manager_jwt);
    let claim_id = create_claim(&server, &mgr_auth).await;

    let declared = server
        .post(&format!("/api/v1/claims/{claim_id}/declare"))
        .add_header("Authorization", &mgr_auth)
        .json(&json!({ "declared_at": "2099-01-01" }))
        .await;
    assert_eq!(declared.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn declare_requires_manager_role() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let mgr_auth = format!("Bearer {}", ctx.manager_jwt);
    let claim_id = create_claim(&server, &mgr_auth).await;

    for jwt in [&ctx.coowner_jwt, &ctx.tenant_jwt, &ctx.board_member_jwt] {
        let auth = format!("Bearer {jwt}");
        let declared = server
            .post(&format!("/api/v1/claims/{claim_id}/declare"))
            .add_header("Authorization", &auth)
            .await;
        assert_eq!(declared.status_code(), StatusCode::FORBIDDEN);
    }
}

// ── Claim notes ───────────────────────────────────────────────────

#[tokio::test]
async fn manager_and_board_can_create_and_list_notes() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let mgr_auth = format!("Bearer {}", ctx.manager_jwt);
    let claim_id = create_claim(&server, &mgr_auth).await;

    let created = server
        .post(&format!("/api/v1/claims/{claim_id}/notes"))
        .add_header("Authorization", &mgr_auth)
        .json(&json!({ "body": "Note syndic" }))
        .await;
    assert_eq!(created.status_code(), StatusCode::CREATED);

    let bm_auth = format!("Bearer {}", ctx.board_member_jwt);
    let created = server
        .post(&format!("/api/v1/claims/{claim_id}/notes"))
        .add_header("Authorization", &bm_auth)
        .json(&json!({ "body": "Note conseil" }))
        .await;
    assert_eq!(created.status_code(), StatusCode::CREATED);

    let listed = server
        .get(&format!("/api/v1/claims/{claim_id}/notes"))
        .add_header("Authorization", &mgr_auth)
        .await;
    assert_eq!(listed.status_code(), StatusCode::OK);
    let notes = listed.json::<Vec<Value>>();
    assert_eq!(notes.len(), 2);
    assert!(notes[0]["author_name"].is_string());
}

#[tokio::test]
async fn co_owner_and_tenant_cannot_access_notes() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let mgr_auth = format!("Bearer {}", ctx.manager_jwt);
    let claim_id = create_claim(&server, &mgr_auth).await;

    for jwt in [&ctx.coowner_jwt, &ctx.tenant_jwt] {
        let auth = format!("Bearer {jwt}");

        let created = server
            .post(&format!("/api/v1/claims/{claim_id}/notes"))
            .add_header("Authorization", &auth)
            .json(&json!({ "body": "Test" }))
            .await;
        assert_eq!(created.status_code(), StatusCode::FORBIDDEN);

        let listed = server
            .get(&format!("/api/v1/claims/{claim_id}/notes"))
            .add_header("Authorization", &auth)
            .await;
        assert_eq!(listed.status_code(), StatusCode::FORBIDDEN);
    }
}

#[tokio::test]
async fn note_body_cannot_be_empty() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let mgr_auth = format!("Bearer {}", ctx.manager_jwt);
    let claim_id = create_claim(&server, &mgr_auth).await;

    let created = server
        .post(&format!("/api/v1/claims/{claim_id}/notes"))
        .add_header("Authorization", &mgr_auth)
        .json(&json!({ "body": "   " }))
        .await;
    assert_eq!(created.status_code(), StatusCode::BAD_REQUEST);
}

// ── Document access control ───────────────────────────────────────

#[tokio::test]
async fn residents_cannot_list_or_attach_claim_documents() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let mgr_auth = format!("Bearer {}", ctx.manager_jwt);
    let claim_id = create_claim(&server, &mgr_auth).await;

    for jwt in [&ctx.coowner_jwt, &ctx.tenant_jwt] {
        let auth = format!("Bearer {jwt}");

        let listed = server
            .get(&format!("/api/v1/claims/{claim_id}/documents"))
            .add_header("Authorization", &auth)
            .await;
        assert_eq!(listed.status_code(), StatusCode::FORBIDDEN);

        let attached = server
            .post(&format!("/api/v1/claims/{claim_id}/documents"))
            .add_header("Authorization", &auth)
            .json(&json!({ "document_id": uuid::Uuid::new_v4() }))
            .await;
        assert_eq!(attached.status_code(), StatusCode::FORBIDDEN);
    }
}

#[tokio::test]
async fn board_member_can_access_claim_documents() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let mgr_auth = format!("Bearer {}", ctx.manager_jwt);
    let claim_id = create_claim(&server, &mgr_auth).await;
    declare_claim(&server, &mgr_auth, &claim_id).await;

    let bm_auth = format!("Bearer {}", ctx.board_member_jwt);

    let listed = server
        .get(&format!("/api/v1/claims/{claim_id}/documents"))
        .add_header("Authorization", &bm_auth)
        .await;
    assert_eq!(listed.status_code(), StatusCode::OK);

    let attached = server
        .post(&format!("/api/v1/claims/{claim_id}/documents"))
        .add_header("Authorization", &bm_auth)
        .json(&json!({ "document_id": uuid::Uuid::new_v4() }))
        .await;
    assert_eq!(attached.status_code(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn board_member_cannot_access_draft_claim_documents() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let mgr_auth = format!("Bearer {}", ctx.manager_jwt);
    let claim_id = create_claim(&server, &mgr_auth).await;

    let bm_auth = format!("Bearer {}", ctx.board_member_jwt);

    let listed = server
        .get(&format!("/api/v1/claims/{claim_id}/documents"))
        .add_header("Authorization", &bm_auth)
        .await;
    assert_eq!(listed.status_code(), StatusCode::NOT_FOUND);

    let attached = server
        .post(&format!("/api/v1/claims/{claim_id}/documents"))
        .add_header("Authorization", &bm_auth)
        .json(&json!({ "document_id": uuid::Uuid::new_v4() }))
        .await;
    assert_eq!(attached.status_code(), StatusCode::NOT_FOUND);

    let detached = server
        .delete(&format!(
            "/api/v1/claims/{claim_id}/documents/{}",
            uuid::Uuid::new_v4()
        ))
        .add_header("Authorization", &bm_auth)
        .await;
    assert_eq!(detached.status_code(), StatusCode::NOT_FOUND);
}

// ── Existing RBAC: update/delete are manager-only ──────────────────

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

        let updated = server
            .put(&format!("/api/v1/claims/{claim_id}"))
            .add_header("Authorization", &auth)
            .json(&json!({
                "title": "Fuite hall B",
                "claim_type": "current",
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
async fn board_member_cannot_update_or_delete_claims() {
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
