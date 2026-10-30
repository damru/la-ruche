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

// ── Follow-up comments (EP-011-S04, US-SIG-006) ──────────────────────────────
// Write is manager + board_member (201); read is manager/board_member or the
// report's author (200); any other resident gets 403. Comments are append-only
// and cascade on report deletion.

#[tokio::test]
async fn manager_can_post_and_list_comments_chronologically() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);
    let report_id = create_report(&server, &auth).await;

    let first = server
        .post(&format!("/api/v1/reports/{report_id}/comments"))
        .add_header("Authorization", &auth)
        .json(&json!({ "body": "Premier commentaire" }))
        .await;
    assert_eq!(first.status_code(), StatusCode::CREATED);
    let first_body = first.json::<Value>();
    assert_eq!(first_body["body"], "Premier commentaire");
    assert!(first_body["author_first_name"].is_string());
    assert!(first_body["created_at"].is_string());

    let second = server
        .post(&format!("/api/v1/reports/{report_id}/comments"))
        .add_header("Authorization", &auth)
        .json(&json!({ "body": "Second commentaire" }))
        .await;
    assert_eq!(second.status_code(), StatusCode::CREATED);

    let listed = server
        .get(&format!("/api/v1/reports/{report_id}/comments"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(listed.status_code(), StatusCode::OK);
    let comments = listed.json::<Value>();
    let bodies: Vec<&str> = comments
        .as_array()
        .expect("array")
        .iter()
        .map(|c| c["body"].as_str().unwrap())
        .collect();
    assert_eq!(bodies, ["Premier commentaire", "Second commentaire"]);
}

#[tokio::test]
async fn comment_body_must_not_be_empty() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);
    let report_id = create_report(&server, &auth).await;

    for body in ["", "   "] {
        let res = server
            .post(&format!("/api/v1/reports/{report_id}/comments"))
            .add_header("Authorization", &auth)
            .json(&json!({ "body": body }))
            .await;
        assert_eq!(res.status_code(), StatusCode::BAD_REQUEST, "body={body:?}");
        assert_eq!(res.json::<Value>()["error"], "validation_error");
        assert_eq!(
            res.json::<Value>()["message"],
            "errors.reports.comment_body_required"
        );
    }
}

#[tokio::test]
async fn reporter_can_read_but_not_post_comments() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let mgr_auth = format!("Bearer {}", ctx.manager_jwt);
    let coowner_auth = format!("Bearer {}", ctx.coowner_jwt);
    let report_id = create_report(&server, &coowner_auth).await;

    let posted = server
        .post(&format!("/api/v1/reports/{report_id}/comments"))
        .add_header("Authorization", &mgr_auth)
        .json(&json!({ "body": "Avancement" }))
        .await;
    assert_eq!(posted.status_code(), StatusCode::CREATED);

    // The reporter reads the comments of their own report.
    let listed = server
        .get(&format!("/api/v1/reports/{report_id}/comments"))
        .add_header("Authorization", &coowner_auth)
        .await;
    assert_eq!(listed.status_code(), StatusCode::OK);
    assert_eq!(listed.json::<Value>().as_array().unwrap().len(), 1);

    // ...but cannot post one (write is reserved to manager/board_member).
    let forbidden = server
        .post(&format!("/api/v1/reports/{report_id}/comments"))
        .add_header("Authorization", &coowner_auth)
        .json(&json!({ "body": "Moi aussi" }))
        .await;
    assert_eq!(forbidden.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn third_party_resident_cannot_read_comments() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let mgr_auth = format!("Bearer {}", ctx.manager_jwt);
    let tenant_auth = format!("Bearer {}", ctx.tenant_jwt);
    // Manager creates the report; a tenant is a third party here.
    let report_id = create_report(&server, &mgr_auth).await;

    let listed = server
        .get(&format!("/api/v1/reports/{report_id}/comments"))
        .add_header("Authorization", &tenant_auth)
        .await;
    assert_eq!(listed.status_code(), StatusCode::FORBIDDEN);

    let posted = server
        .post(&format!("/api/v1/reports/{report_id}/comments"))
        .add_header("Authorization", &tenant_auth)
        .json(&json!({ "body": "Piqué" }))
        .await;
    assert_eq!(posted.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn comments_of_a_report_from_another_building_return_404() {
    let (ctx, pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);
    let report_id = create_report(&server, &auth).await;

    let other_building_id = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total) \
         VALUES ($1, 'Other building', '2 rue Autre', '75002', 'Paris', 1000)",
    )
    .bind(other_building_id)
    .execute(&pool)
    .await
    .expect("insert other building");
    let other_report_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO report (building_id, author_id, title) \
         VALUES ($1, $2, 'Autre immeuble') RETURNING id",
    )
    .bind(other_building_id)
    .bind(ctx.manager_id)
    .fetch_one(&pool)
    .await
    .expect("insert other report");

    // The manager of building A cannot read/post on building B's report.
    let listed = server
        .get(&format!("/api/v1/reports/{other_report_id}/comments"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(listed.status_code(), StatusCode::NOT_FOUND, "GET");

    let posted = server
        .post(&format!("/api/v1/reports/{other_report_id}/comments"))
        .add_header("Authorization", &auth)
        .json(&json!({ "body": "Intrusion" }))
        .await;
    assert_eq!(posted.status_code(), StatusCode::NOT_FOUND, "POST");

    // Sanity: the report of our own building still answers 200.
    let listed = server
        .get(&format!("/api/v1/reports/{report_id}/comments"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(listed.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn deleting_a_report_cascades_its_comments() {
    let (ctx, pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);
    let report_id = create_report(&server, &auth).await;

    let posted = server
        .post(&format!("/api/v1/reports/{report_id}/comments"))
        .add_header("Authorization", &auth)
        .json(&json!({ "body": "Avant suppression" }))
        .await;
    assert_eq!(posted.status_code(), StatusCode::CREATED);

    let deleted = server
        .delete(&format!("/api/v1/reports/{report_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(deleted.status_code(), StatusCode::NO_CONTENT);

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM report_comment WHERE report_id = $1")
        .bind(uuid::Uuid::parse_str(&report_id).unwrap())
        .fetch_one(&pool)
        .await
        .expect("count comments");
    assert_eq!(count, 0, "comments must be cascade-deleted with the report");

    let listed = server
        .get(&format!("/api/v1/reports/{report_id}/comments"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(listed.status_code(), StatusCode::NOT_FOUND);
}
