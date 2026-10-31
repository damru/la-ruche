// EP-007-S07 (PRD US-TP-012): a manager can take a task back from the quote
// phase (`awaiting_quote` / `quote_received`) to `in_progress` without
// revoking the vendor token or touching attached documents.

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{Value, json};

async fn create_task(server: &TestServer, auth: &str, title: &str) -> String {
    let created = server
        .post("/api/v1/tasks")
        .add_header("Authorization", auth)
        .json(&json!({ "title": title }))
        .await;
    assert!(created.status_code().is_success(), "setup create task");
    created.json::<Value>()["id"]
        .as_str()
        .expect("id")
        .to_string()
}

async fn set_status(pool: &sqlx::PgPool, task_id: &str, status: &str) {
    sqlx::query("UPDATE task SET status = $2::text::task_status WHERE id = $1::uuid")
        .bind(task_id)
        .bind(status)
        .execute(pool)
        .await
        .expect("set status");
}

#[tokio::test]
async fn take_back_from_awaiting_quote_returns_in_progress_and_keeps_token() {
    let (ctx, pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);
    let task_id = create_task(&server, &auth, "Devis demandé").await;

    // A vendor token is active: take-back must NOT revoke it.
    let token_resp = server
        .post(&format!("/api/v1/tasks/{task_id}/vendor-token"))
        .add_header("Authorization", &auth)
        .await;
    assert!(token_resp.status_code().is_success());
    let token = token_resp.json::<Value>()["token"]
        .as_str()
        .expect("token")
        .to_string();
    set_status(&pool, &task_id, "awaiting_quote").await;

    let resp = server
        .post(&format!("/api/v1/tasks/{task_id}/take-back"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let body = resp.json::<Value>();
    assert_eq!(body["status"], "in_progress");
    assert_eq!(body["has_vendor_token"], true);

    let stored_token: Option<uuid::Uuid> =
        sqlx::query_scalar("SELECT vendor_token FROM task WHERE id = $1::uuid")
            .bind(&task_id)
            .fetch_one(&pool)
            .await
            .expect("token");
    assert_eq!(
        stored_token.map(|t| t.to_string()).as_deref(),
        Some(token.as_str()),
        "vendor token must be preserved by take-back"
    );
}

#[tokio::test]
async fn take_back_from_quote_received_keeps_documents() {
    let (ctx, pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);
    let task_id = create_task(&server, &auth, "Devis reçu").await;
    set_status(&pool, &task_id, "quote_received").await;

    // Attach a document as manager via the API, then check it survives take-back.
    let doc_id: String = sqlx::query_scalar(
        "INSERT INTO document (id, building_id, category, original_name, storage_path, bucket, mime_type, size_bytes, visible_to_roles, uploaded_by)
         VALUES (gen_random_uuid(), $1, 'autre', 'devis.pdf', 'k/devis.pdf', 'bucket', 'application/pdf', 10, ARRAY['manager','board_member','co_owner','tenant']::user_role[], $2)
         RETURNING id::text",
    )
    .bind(ctx.building_id)
    .bind(ctx.manager_id)
    .fetch_one(&pool)
    .await
    .expect("document");
    let attached = server
        .post(&format!("/api/v1/tasks/{task_id}/documents"))
        .add_header("Authorization", &auth)
        .json(&json!({ "document_id": doc_id }))
        .await;
    assert_eq!(attached.status_code(), StatusCode::NO_CONTENT);

    let resp = server
        .post(&format!("/api/v1/tasks/{task_id}/take-back"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    assert_eq!(resp.json::<Value>()["status"], "in_progress");

    let docs = server
        .get(&format!("/api/v1/tasks/{task_id}/documents"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(docs.status_code(), StatusCode::OK);
    let listed = docs.json::<Vec<Value>>();
    assert_eq!(listed.len(), 1, "attached document must survive take-back");
}

#[tokio::test]
async fn take_back_refused_outside_quote_phase() {
    let (ctx, pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);

    for status in ["open", "in_progress", "awaiting_vendor", "awaiting_validation"] {
        let task_id = create_task(&server, &auth, &format!("Statut {status}")).await;
        set_status(&pool, &task_id, status).await;
        let resp = server
            .post(&format!("/api/v1/tasks/{task_id}/take-back"))
            .add_header("Authorization", &auth)
            .await;
        assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST, "status {status}");
        assert_eq!(
            resp.json::<Value>()["message"],
            "errors.tasks.not_in_quote_phase"
        );
    }

    // Terminal statuses: frozen guard fires first with the same 400 contract.
    let closed_id = create_task(&server, &auth, "Clôturée").await;
    set_status(&pool, &closed_id, "closed").await;
    let resp = server
        .post(&format!("/api/v1/tasks/{closed_id}/take-back"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn take_back_forbidden_for_non_manager_roles() {
    let (ctx, pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let manager_auth = format!("Bearer {}", ctx.manager_jwt);
    let task_id = create_task(&server, &manager_auth, "Devis en cours").await;
    set_status(&pool, &task_id, "awaiting_quote").await;

    for jwt in [&ctx.board_member_jwt, &ctx.coowner_jwt, &ctx.tenant_jwt] {
        let resp = server
            .post(&format!("/api/v1/tasks/{task_id}/take-back"))
            .add_header("Authorization", format!("Bearer {jwt}"))
            .await;
        assert_eq!(resp.status_code(), StatusCode::FORBIDDEN);
    }
}

#[tokio::test]
async fn take_back_unknown_task_returns_404() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);
    let resp = server
        .post(&format!("/api/v1/tasks/{}/take-back", uuid::Uuid::new_v4()))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(resp.status_code(), StatusCode::NOT_FOUND);
}
