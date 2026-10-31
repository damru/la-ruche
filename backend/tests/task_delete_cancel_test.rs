// Task delete/cancel lifecycle — hard delete is refused when evidentiary
// records (vendor submission, maintenance-log entry) reference the task;
// cancellation is the non-destructive alternative and freezes the task.

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

async fn add_vendor_submission(pool: &sqlx::PgPool, task_id: &str) {
    sqlx::query("INSERT INTO vendor_submission (task_id, comment) VALUES ($1::uuid, 'devis')")
        .bind(task_id)
        .execute(pool)
        .await
        .expect("submission");
}

async fn add_logbook_entry(
    pool: &sqlx::PgPool,
    building_id: uuid::Uuid,
    manager_id: uuid::Uuid,
    task_id: &str,
) {
    sqlx::query(
        "INSERT INTO maintenance_log (building_id, title, intervention_date, task_id, created_by)
         VALUES ($1, 'Intervention', CURRENT_DATE, $2::uuid, $3)",
    )
    .bind(building_id)
    .bind(task_id)
    .bind(manager_id)
    .execute(pool)
    .await
    .expect("logbook entry");
}

#[tokio::test]
async fn delete_refused_when_vendor_submission_exists() {
    let (ctx, pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);
    let task_id = create_task(&server, &auth, "Avec soumission").await;
    add_vendor_submission(&pool, &task_id).await;

    let resp = server
        .delete(&format!("/api/v1/tasks/{task_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
        resp.json::<Value>()["message"],
        "errors.tasks.has_submissions"
    );

    let still_there = server
        .get(&format!("/api/v1/tasks/{task_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(still_there.status_code(), StatusCode::OK);
    let submissions: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM vendor_submission WHERE task_id = $1::uuid")
            .bind(&task_id)
            .fetch_one(&pool)
            .await
            .expect("count");
    assert_eq!(submissions, 1, "submission must never be cascade-deleted");
}

#[tokio::test]
async fn delete_refused_when_logbook_entry_exists() {
    let (ctx, pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);
    let task_id = create_task(&server, &auth, "Dans le carnet").await;
    add_logbook_entry(&pool, ctx.building_id, ctx.manager_id, &task_id).await;

    let resp = server
        .delete(&format!("/api/v1/tasks/{task_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
        resp.json::<Value>()["message"],
        "errors.tasks.has_logbook_entry"
    );
}

#[tokio::test]
async fn delete_blank_task_succeeds() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);
    let task_id = create_task(&server, &auth, "Vierge").await;

    let resp = server
        .delete(&format!("/api/v1/tasks/{task_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(resp.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn cancel_sets_status_and_freezes_task() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);
    let task_id = create_task(&server, &auth, "A annuler").await;

    let cancelled = server
        .post(&format!("/api/v1/tasks/{task_id}/cancel"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(cancelled.status_code(), StatusCode::NO_CONTENT);

    let task = server
        .get(&format!("/api/v1/tasks/{task_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(task.json::<Value>()["status"], "cancelled");

    let updated = server
        .put(&format!("/api/v1/tasks/{task_id}"))
        .add_header("Authorization", &auth)
        .json(&json!({ "title": "Nope" }))
        .await;
    assert_eq!(updated.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
        updated.json::<Value>()["message"],
        "errors.immutability.frozen"
    );

    let deleted = server
        .delete(&format!("/api/v1/tasks/{task_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(deleted.status_code(), StatusCode::BAD_REQUEST);

    let cancelled_again = server
        .post(&format!("/api/v1/tasks/{task_id}/cancel"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(cancelled_again.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
        cancelled_again.json::<Value>()["message"],
        "errors.immutability.frozen"
    );
}

#[tokio::test]
async fn cancel_refused_on_closed_task() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);
    let task_id = create_task(&server, &auth, "A cloturer").await;

    let closed = server
        .put(&format!("/api/v1/tasks/{task_id}"))
        .add_header("Authorization", &auth)
        .json(&json!({ "status": "closed" }))
        .await;
    assert_eq!(closed.status_code(), StatusCode::NO_CONTENT);

    let resp = server
        .post(&format!("/api/v1/tasks/{task_id}/cancel"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
        resp.json::<Value>()["message"],
        "errors.immutability.frozen"
    );
}

#[tokio::test]
async fn cancel_is_manager_only() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let mgr_auth = format!("Bearer {}", ctx.manager_jwt);
    let task_id = create_task(&server, &mgr_auth, "Annulation CS").await;

    let board_auth = format!("Bearer {}", ctx.board_member_jwt);
    let resp = server
        .post(&format!("/api/v1/tasks/{task_id}/cancel"))
        .add_header("Authorization", &board_auth)
        .await;
    assert_eq!(resp.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn vendor_token_rejected_on_cancelled_task() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);
    let task_id = create_task(&server, &auth, "Token puis annulation").await;

    let token_resp = server
        .post(&format!("/api/v1/tasks/{task_id}/vendor-token"))
        .add_header("Authorization", &auth)
        .await;
    assert!(token_resp.status_code().is_success());
    let token = token_resp.json::<Value>()["token"]
        .as_str()
        .expect("token")
        .to_string();

    let cancelled = server
        .post(&format!("/api/v1/tasks/{task_id}/cancel"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(cancelled.status_code(), StatusCode::NO_CONTENT);

    let vendor_view = server.get(&format!("/api/v1/vendor/{token}")).await;
    assert_eq!(vendor_view.status_code(), StatusCode::NOT_FOUND);
}
