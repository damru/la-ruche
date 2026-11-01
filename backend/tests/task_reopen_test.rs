// Task reopen lifecycle (PRD US-TP-013) — a `closed` or `cancelled` task can be
// reopened to `in_progress` through a dedicated endpoint that is the sanctioned
// exit from the ADR-009 frozen states. Completion fields are reset, the original
// maintenance-log entry stays intact, and a re-closing creates a new entry.

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

async fn close_task(server: &TestServer, auth: &str, task_id: &str, cost: Option<i64>) {
    let closed = server
        .put(&format!("/api/v1/tasks/{task_id}"))
        .add_header("Authorization", auth)
        .json(&json!({ "status": "closed", "completion_cost": cost }))
        .await;
    assert_eq!(closed.status_code(), StatusCode::NO_CONTENT);
}

async fn logbook_entries(pool: &sqlx::PgPool, task_id: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM maintenance_log WHERE task_id = $1::uuid")
        .bind(task_id)
        .fetch_one(pool)
        .await
        .expect("count")
}

#[tokio::test]
async fn reopen_closed_task_resets_completion_and_keeps_logbook_entry() {
    let (ctx, pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);
    let task_id = create_task(&server, &auth, "A rouvrir").await;
    close_task(&server, &auth, &task_id, Some(1500)).await;
    assert_eq!(logbook_entries(&pool, &task_id).await, 1);

    let reopened = server
        .post(&format!("/api/v1/tasks/{task_id}/reopen"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(reopened.status_code(), StatusCode::OK);
    let body = reopened.json::<Value>();
    assert_eq!(body["status"], "in_progress");
    assert!(body["closed_at"].is_null());
    assert!(body["completion_cost"].is_null());

    assert_eq!(
        logbook_entries(&pool, &task_id).await,
        1,
        "original logbook entry must stay intact after reopen"
    );

    let task = server
        .get(&format!("/api/v1/tasks/{task_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(task.json::<Value>()["status"], "in_progress");
}

#[tokio::test]
async fn reclose_after_reopen_creates_a_second_logbook_entry() {
    let (ctx, pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);
    let task_id = create_task(&server, &auth, "Double cloture").await;
    close_task(&server, &auth, &task_id, Some(100)).await;

    let reopened = server
        .post(&format!("/api/v1/tasks/{task_id}/reopen"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(reopened.status_code(), StatusCode::OK);

    close_task(&server, &auth, &task_id, Some(200)).await;
    assert_eq!(
        logbook_entries(&pool, &task_id).await,
        2,
        "two interventions = two logbook lines"
    );
}

#[tokio::test]
async fn reopen_rejected_on_non_frozen_task() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);
    let task_id = create_task(&server, &auth, "Deja ouverte").await;

    let resp = server
        .post(&format!("/api/v1/tasks/{task_id}/reopen"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
        resp.json::<Value>()["message"],
        "errors.tasks.invalid_transition"
    );
}

#[tokio::test]
async fn reopen_cancelled_task_succeeds() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);
    let task_id = create_task(&server, &auth, "Annulee puis rouverte").await;

    let cancelled = server
        .post(&format!("/api/v1/tasks/{task_id}/cancel"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(cancelled.status_code(), StatusCode::NO_CONTENT);

    let reopened = server
        .post(&format!("/api/v1/tasks/{task_id}/reopen"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(reopened.status_code(), StatusCode::OK);
    assert_eq!(reopened.json::<Value>()["status"], "in_progress");
}

#[tokio::test]
async fn reopen_is_manager_only() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let mgr_auth = format!("Bearer {}", ctx.manager_jwt);
    let task_id = create_task(&server, &mgr_auth, "Reopen RBAC").await;
    close_task(&server, &mgr_auth, &task_id, None).await;

    for jwt in [&ctx.board_member_jwt, &ctx.coowner_jwt, &ctx.tenant_jwt] {
        let resp = server
            .post(&format!("/api/v1/tasks/{task_id}/reopen"))
            .add_header("Authorization", format!("Bearer {jwt}"))
            .await;
        assert_eq!(resp.status_code(), StatusCode::FORBIDDEN);
    }
}

#[tokio::test]
async fn reopen_unknown_task_is_404() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);

    let resp = server
        .post(&format!("/api/v1/tasks/{}/reopen", uuid::Uuid::new_v4()))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(resp.status_code(), StatusCode::NOT_FOUND);
}
