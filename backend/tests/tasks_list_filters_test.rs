// EP-007-S09 — GET /tasks query params: status/priority/assignee/contractor
// filters, q search, limit/offset pagination with total, invalid filter 400.

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{Value, json};

async fn create_task(server: &TestServer, auth: &str, body: Value) -> String {
    let created = server
        .post("/api/v1/tasks")
        .add_header("Authorization", auth)
        .json(&body)
        .await;
    assert!(created.status_code().is_success(), "setup create task");
    created.json::<Value>()["id"]
        .as_str()
        .expect("id")
        .to_string()
}

async fn list(server: &TestServer, auth: &str, query: &str) -> (StatusCode, Value) {
    let resp = server
        .get(&format!("/api/v1/tasks?{query}"))
        .add_header("Authorization", auth)
        .await;
    let status = resp.status_code();
    (status, resp.json::<Value>())
}

#[tokio::test]
async fn filters_by_status_priority_and_search_with_pagination() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);

    create_task(
        &server,
        &auth,
        json!({ "title": "Chaudière en panne", "priority": "urgent" }),
    )
    .await;
    create_task(
        &server,
        &auth,
        json!({ "title": "Remplacer ampoule", "priority": "low" }),
    )
    .await;
    let in_progress = create_task(
        &server,
        &auth,
        json!({ "title": "Réparer porte", "priority": "normal" }),
    )
    .await;
    let updated = server
        .put(&format!("/api/v1/tasks/{in_progress}"))
        .add_header("Authorization", &auth)
        .json(&json!({ "status": "in_progress" }))
        .await;
    assert_eq!(
        updated.status_code(),
        StatusCode::NO_CONTENT,
        "update: {:?}",
        updated.text()
    );

    // No params: envelope { items, total }, default limit applies.
    let (status, body) = list(&server, &auth, "").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 3);
    assert_eq!(body["items"].as_array().unwrap().len(), 3);

    // Filter by status.
    let (status, body) = list(&server, &auth, "status=in_progress").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["title"], "Réparer porte");

    // Filter by priority.
    let (_, body) = list(&server, &auth, "priority=urgent").await;
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["title"], "Chaudière en panne");

    // Search on title.
    let (_, body) = list(&server, &auth, "q=chaudière").await;
    assert_eq!(body["total"], 1);

    // Search on description.
    let divers = create_task(
        &server,
        &auth,
        json!({ "title": "Divers", "description": "nettoyage de la chaufferie" }),
    )
    .await;
    let (_, check) = list(&server, &auth, "q=chaufferie").await;
    assert_eq!(check["items"][0]["id"], divers, "description search hit");
    let (_, body) = list(&server, &auth, "q=chaufferie").await;
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["title"], "Divers");

    // ILIKE metacharacters are matched literally.
    let (_, body) = list(&server, &auth, "q=%25").await;
    assert_eq!(body["total"], 0);

    // Combined filters are AND-ed.
    let (_, body) = list(&server, &auth, "status=open&priority=low&q=ampoule").await;
    assert_eq!(body["total"], 1);
    let (_, body) = list(&server, &auth, "status=in_progress&priority=low").await;
    assert_eq!(body["total"], 0);

    // Pagination: limit=2&offset=2 returns the tail with the full total.
    let (status, body) = list(&server, &auth, "limit=2&offset=2").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 4);
    assert_eq!(body["items"].as_array().unwrap().len(), 2);

    // limit is clamped to 200, offset past total returns an empty page.
    let (_, body) = list(&server, &auth, "limit=9999").await;
    assert_eq!(body["total"], 4);
    assert_eq!(body["items"].as_array().unwrap().len(), 4);
    let (_, body) = list(&server, &auth, "limit=10&offset=99").await;
    assert_eq!(body["items"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn filter_by_assignee_and_contractor() {
    let (ctx, pool) = common::rbac_setup().await;
    let board_member_id = common::rbac_insert_user(
        &pool,
        &common::test_config().await,
        ctx.building_id,
        laruche::domain::Role::BoardMember,
    )
    .await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);

    let contractor_id = uuid::Uuid::new_v4();
    sqlx::query("INSERT INTO contractor (id, building_id, name) VALUES ($1, $2, 'Plombier Test')")
        .bind(contractor_id)
        .bind(ctx.building_id)
        .execute(&pool)
        .await
        .expect("contractor");

    create_task(
        &server,
        &auth,
        json!({ "title": "Fuite évier", "assigned_to": board_member_id, "contractor_id": contractor_id }),
    )
    .await;
    create_task(&server, &auth, json!({ "title": "Ampoule couloir" })).await;

    let (status, body) = list(&server, &auth, &format!("assigned_to={}", board_member_id)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["title"], "Fuite évier");

    let (_, body) = list(&server, &auth, &format!("contractor_id={contractor_id}")).await;
    assert_eq!(body["total"], 1);

    let (_, body) = list(
        &server,
        &auth,
        &format!("contractor_id={}", uuid::Uuid::new_v4()),
    )
    .await;
    assert_eq!(body["total"], 0);
}

#[tokio::test]
async fn invalid_status_or_priority_returns_400() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);

    let (status, body) = list(&server, &auth, "status=bogus").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "validation_error");
    assert_eq!(body["message"], "errors.tasks.invalid_filter");

    let (status, _) = list(&server, &auth, "priority=bogus").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
