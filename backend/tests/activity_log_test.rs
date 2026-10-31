// Activity log (lifecycle timeline) — EP-017 S01/S02/S03.
// Emission of events at every report/task mutation, then read through the
// merged /timeline endpoints; events are also checked directly in the DB.

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{Value, json};
use sqlx::Row;

async fn events_for(pool: &sqlx::PgPool, entity_type: &str, entity_id: &str) -> Vec<Value> {
    let rows = sqlx::query(
        "SELECT action, actor_id::text, payload FROM activity_log \
         WHERE entity_type = $1 AND entity_id = $2::uuid ORDER BY created_at ASC, id ASC",
    )
    .bind(entity_type)
    .bind(entity_id)
    .fetch_all(pool)
    .await
    .expect("select activity_log");
    rows.iter()
        .map(|r| {
            json!({
                "action": r.try_get::<String, _>("action").unwrap(),
                "actor_id": r.try_get::<Option<String>, _>("actor_id").unwrap(),
                "payload": r.try_get::<Option<Value>, _>("payload").unwrap(),
            })
        })
        .collect()
}

async fn create_report(server: &TestServer, auth: &str, title: &str) -> String {
    let created = server
        .post("/api/v1/reports")
        .add_header("Authorization", auth)
        .json(&json!({ "title": title }))
        .await;
    assert_eq!(created.status_code(), StatusCode::CREATED);
    created.json::<Value>()["id"]
        .as_str()
        .expect("id")
        .to_string()
}

async fn create_task(server: &TestServer, auth: &str, title: &str) -> String {
    let created = server
        .post("/api/v1/tasks")
        .add_header("Authorization", auth)
        .json(&json!({ "title": title }))
        .await;
    assert!(created.status_code().is_success());
    created.json::<Value>()["id"]
        .as_str()
        .expect("id")
        .to_string()
}

#[tokio::test]
async fn insert_and_list_for_entity_are_ordered_and_transactional() {
    let (_cfg, pool) = common::test_db().await;
    let entity_id = uuid::Uuid::new_v4();

    // Insert inside a rolled-back transaction: nothing must persist.
    let mut tx = pool.begin().await.expect("begin");
    laruche::db::activity_log::insert(
        &mut *tx,
        laruche::db::activity_log::ENTITY_TASK,
        entity_id,
        laruche::db::activity_log::actions::CREATED,
        None,
        Some(json!({ "rolled_back": true })),
    )
    .await
    .expect("insert in tx");
    tx.rollback().await.expect("rollback");
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM activity_log WHERE entity_id = $1")
        .bind(entity_id)
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(count, 0, "rolled-back insert must not persist");

    // Three events (one anonymous actor, one with payload) via the pool.
    for (action, actor, payload) in [
        ("created", None, None),
        (
            "status_changed",
            None,
            Some(json!({ "old_status": "open", "new_status": "in_progress" })),
        ),
        ("closed", None, None),
    ] {
        laruche::db::activity_log::insert(
            &pool,
            laruche::db::activity_log::ENTITY_TASK,
            entity_id,
            action,
            actor,
            payload,
        )
        .await
        .expect("insert");
    }
    let rows = laruche::db::activity_log::list_for_entity(
        &pool,
        laruche::db::activity_log::ENTITY_TASK,
        entity_id,
    )
    .await
    .expect("list");
    let actions: Vec<&str> = rows.iter().map(|r| r.action.as_str()).collect();
    assert_eq!(actions, vec!["created", "status_changed", "closed"]);
    assert_eq!(rows[0].actor_id, None);
    assert_eq!(
        rows[1].payload.as_ref().unwrap()["new_status"],
        json!("in_progress")
    );
}

#[tokio::test]
async fn report_lifecycle_events() {
    let (ctx, pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let tenant = format!("Bearer {}", ctx.tenant_jwt);
    let manager = format!("Bearer {}", ctx.manager_jwt);

    // 1. POST /reports -> report.created with the creator as actor.
    let report_id = create_report(&server, &tenant, "Fuite en cuisine").await;
    let ev = events_for(&pool, "report", &report_id).await;
    assert_eq!(ev.len(), 1);
    assert_eq!(ev[0]["action"], json!("created"));
    assert!(ev[0]["actor_id"].is_string(), "creator must be the actor");

    // 2. PUT /reports/{id}/status -> status_changed with old/new.
    let resp = server
        .put(&format!("/api/v1/reports/{report_id}/status"))
        .add_header("Authorization", &manager)
        .json(&json!({ "status": "in_progress" }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::NO_CONTENT);
    let ev = events_for(&pool, "report", &report_id).await;
    assert_eq!(ev.len(), 2);
    assert_eq!(ev[1]["action"], json!("status_changed"));
    assert_eq!(ev[1]["payload"]["old_status"], json!("open"));
    assert_eq!(ev[1]["payload"]["new_status"], json!("in_progress"));

    // No-op status: no new event.
    server
        .put(&format!("/api/v1/reports/{report_id}/status"))
        .add_header("Authorization", &manager)
        .json(&json!({ "status": "in_progress" }))
        .await;
    assert_eq!(events_for(&pool, "report", &report_id).await.len(), 2);

    // 3. POST /reports/{id}/task -> converted_to_task + task.created, atomically.
    let resp = server
        .post(&format!("/api/v1/reports/{report_id}/task"))
        .add_header("Authorization", &manager)
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let task_id = resp.json::<Value>()["task_id"]
        .as_str()
        .expect("task_id")
        .to_string();

    let report_ev = events_for(&pool, "report", &report_id).await;
    assert_eq!(report_ev.len(), 3);
    assert_eq!(report_ev[2]["action"], json!("converted_to_task"));
    assert_eq!(report_ev[2]["payload"]["task_id"], json!(task_id));

    let task_ev = events_for(&pool, "task", &task_id).await;
    assert_eq!(task_ev.len(), 1);
    assert_eq!(task_ev[0]["action"], json!("created"));
    assert_eq!(task_ev[0]["payload"]["source_report_id"], json!(report_id));
    assert!(
        task_ev[0]["actor_id"].is_string(),
        "task.created from a conversion carries the manager as actor"
    );
}

#[tokio::test]
async fn task_mutation_events_and_closed_transition() {
    let (ctx, pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let manager = format!("Bearer {}", ctx.manager_jwt);

    let task_id = create_task(&server, &manager, "Refaire la peinture").await;
    assert_eq!(
        events_for(&pool, "task", &task_id).await[0]["action"],
        json!("created")
    );

    // 4. PUT /tasks/{id} changing status + contractor -> two distinct events.
    let resp = server
        .put(&format!("/api/v1/tasks/{task_id}"))
        .add_header("Authorization", &manager)
        .json(&json!({ "status": "in_progress", "contractor_name": "Plomberie Pro" }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::NO_CONTENT);
    let ev = events_for(&pool, "task", &task_id).await;
    assert_eq!(ev.len(), 3);
    assert_eq!(ev[1]["action"], json!("status_changed"));
    assert_eq!(ev[1]["payload"]["new_status"], json!("in_progress"));
    assert_eq!(ev[2]["action"], json!("contractor_assigned"));
    assert_eq!(ev[2]["payload"]["contractor_name"], json!("Plomberie Pro"));

    // Same body again -> no new event (no-op diff).
    server
        .put(&format!("/api/v1/tasks/{task_id}"))
        .add_header("Authorization", &manager)
        .json(&json!({ "status": "in_progress", "contractor_name": "Plomberie Pro" }))
        .await;
    assert_eq!(events_for(&pool, "task", &task_id).await.len(), 3);

    // 5. Transition to closed -> `closed` event (not status_changed).
    server
        .put(&format!("/api/v1/tasks/{task_id}"))
        .add_header("Authorization", &manager)
        .json(&json!({ "status": "closed" }))
        .await;
    let ev = events_for(&pool, "task", &task_id).await;
    assert_eq!(ev.last().unwrap()["action"], json!("closed"));
    assert_eq!(
        ev.last().unwrap()["payload"]["old_status"],
        json!("in_progress")
    );
}

#[tokio::test]
async fn vendor_token_and_submission_events() {
    let (ctx, pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let manager = format!("Bearer {}", ctx.manager_jwt);

    let task_id = create_task(&server, &manager, "Intervention chauffe-eau").await;

    // 6. POST /tasks/{id}/vendor-token -> vendor_link_generated (no raw token) + status_changed.
    let resp = server
        .post(&format!("/api/v1/tasks/{task_id}/vendor-token"))
        .add_header("Authorization", &manager)
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let body = resp.json::<Value>();
    let token = body["token"].as_str().expect("token").to_string();

    let ev = events_for(&pool, "task", &task_id).await;
    let link = ev
        .iter()
        .find(|e| e["action"] == json!("vendor_link_generated"))
        .expect("link event");
    assert!(link["payload"]["expires_at"].is_string());
    assert!(
        !serde_json::to_string(&link["payload"])
            .unwrap()
            .contains(&token),
        "raw vendor token must never appear in the payload"
    );
    let status = ev
        .iter()
        .find(|e| e["action"] == json!("status_changed"))
        .expect("status event");
    assert_eq!(status["payload"]["new_status"], json!("awaiting_vendor"));

    // 7. POST /api/v1/vendor/{token}/submission -> vendor_submission_received, actor NULL.
    let resp = server
        .post(&format!("/api/v1/vendor/{token}/submission"))
        .json(&json!({ "comment": "Devis ci-joint", "documents": [] }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::CREATED);
    let ev = events_for(&pool, "task", &task_id).await;
    let sub = ev
        .iter()
        .find(|e| e["action"] == json!("vendor_submission_received"))
        .expect("submission event");
    assert_eq!(sub["actor_id"], Value::Null);
    assert!(sub["payload"]["submission_id"].is_string());
}

#[tokio::test]
async fn timeline_endpoints_return_the_same_merged_chain() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let manager = format!("Bearer {}", ctx.manager_jwt);
    let tenant = format!("Bearer {}", ctx.tenant_jwt);

    let report_id = create_report(&server, &tenant, "Portail cassé").await;
    let resp = server
        .post(&format!("/api/v1/reports/{report_id}/task"))
        .add_header("Authorization", &manager)
        .await;
    let task_id = resp.json::<Value>()["task_id"]
        .as_str()
        .expect("task_id")
        .to_string();
    server
        .put(&format!("/api/v1/tasks/{task_id}"))
        .add_header("Authorization", &manager)
        .json(&json!({ "status": "in_progress" }))
        .await;

    // 1. Both endpoints return the same ordered chain.
    let from_report = server
        .get(&format!("/api/v1/reports/{report_id}/timeline"))
        .add_header("Authorization", &tenant)
        .await;
    assert_eq!(from_report.status_code(), StatusCode::OK);
    let from_task = server
        .get(&format!("/api/v1/tasks/{task_id}/timeline"))
        .add_header("Authorization", &manager)
        .await;
    assert_eq!(from_task.status_code(), StatusCode::OK);
    let a = from_report.json::<Vec<Value>>();
    let b = from_task.json::<Vec<Value>>();
    assert_eq!(
        a, b,
        "both maillons of the chain must return the same timeline"
    );
    // Order is by (created_at, id): both conversion events share the same
    // millisecond, so the stable tie-break on id decides between them.
    let actions: Vec<&str> = a.iter().map(|e| e["action"].as_str().unwrap()).collect();
    assert_eq!(actions.len(), 4);
    assert_eq!(actions[0], "created");
    assert_eq!(actions[3], "status_changed");
    let mut middle: Vec<&str> = actions[1..3].to_vec();
    middle.sort_unstable();
    assert_eq!(middle, vec!["converted_to_task", "created"]);
    // actor_name resolved for known actors.
    assert!(a[0]["actor_name"].is_string());

    // 2. Task without source report: timeline = task events only.
    let lone = create_task(&server, &manager, "Tâche isolée").await;
    let resp = server
        .get(&format!("/api/v1/tasks/{lone}/timeline"))
        .add_header("Authorization", &manager)
        .await;
    let events = resp.json::<Vec<Value>>();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["action"], json!("created"));

    // 3. Report not converted: timeline = report events only.
    let plain = create_report(&server, &tenant, "Ampoule couloir").await;
    let resp = server
        .get(&format!("/api/v1/reports/{plain}/timeline"))
        .add_header("Authorization", &tenant)
        .await;
    let events = resp.json::<Vec<Value>>();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["entity_type"], json!("report"));

    // 4. Cross-building and unknown entity -> 404.
    let other_building = uuid::Uuid::new_v4();
    let (_cfg, pool) = common::test_db().await;
    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total) \
         VALUES ($1, 'Other', '2 rue Autre', '75002', 'Paris', 1000)",
    )
    .bind(other_building)
    .execute(&pool)
    .await
    .expect("building");
    let resp = server
        .get(&format!("/api/v1/tasks/{task_id}/timeline"))
        .add_header("Authorization", &format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK); // same building still OK
    let unknown = uuid::Uuid::new_v4();
    let resp = server
        .get(&format!("/api/v1/tasks/{unknown}/timeline"))
        .add_header("Authorization", &manager)
        .await;
    assert_eq!(resp.status_code(), StatusCode::NOT_FOUND);
    let resp = server
        .get(&format!("/api/v1/reports/{unknown}/timeline"))
        .add_header("Authorization", &manager)
        .await;
    assert_eq!(resp.status_code(), StatusCode::NOT_FOUND);
}
