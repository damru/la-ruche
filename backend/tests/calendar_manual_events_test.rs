// Integration tests — EP-014-S03: manual calendar events (US-CAL-005).
// AC-CAL-005-A..E: create/edit/delete guarded to Manager/BoardMember,
// read open to all roles, document attachment, building isolation.

mod common;

use axum::http::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

struct Ctx {
    building_id: Uuid,
    manager_jwt: String,
    board_member_jwt: String,
    coowner_jwt: String,
    tenant_jwt: String,
}

async fn setup() -> (Ctx, sqlx::PgPool) {
    let (cfg, pool) = common::test_db().await;

    let building_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total) \
         VALUES ($1, 'CalEventTestBuilding', '1 rue Cal', '75001', 'Paris', 1000)",
    )
    .bind(building_id)
    .execute(&pool)
    .await
    .expect("building");

    let manager_id = insert_user(&pool, &cfg, building_id, "manager").await;
    let board_member_id = insert_user(&pool, &cfg, building_id, "board_member").await;
    let coowner_id = insert_user(&pool, &cfg, building_id, "co_owner").await;
    let tenant_id = insert_user(&pool, &cfg, building_id, "tenant").await;

    let jwt = |user_id: Uuid, role: laruche::domain::Role| {
        laruche::auth::jwt::generate_access_token(
            user_id,
            building_id,
            vec![role],
            &cfg.auth.jwt_secret,
            cfg.auth.access_token_ttl_secs,
        )
        .expect("token")
    };

    let ctx = Ctx {
        building_id,
        manager_jwt: jwt(manager_id, laruche::domain::Role::Manager),
        board_member_jwt: jwt(board_member_id, laruche::domain::Role::BoardMember),
        coowner_jwt: jwt(coowner_id, laruche::domain::Role::CoOwner),
        tenant_jwt: jwt(tenant_id, laruche::domain::Role::Tenant),
    };
    (ctx, pool)
}

async fn insert_user(
    pool: &sqlx::PgPool,
    cfg: &laruche::config::AppConfig,
    building_id: Uuid,
    role: &str,
) -> Uuid {
    let user_id = Uuid::new_v4();
    let email = format!("cal-{role}-{}@test.local", Uuid::new_v4());
    let password_hash =
        laruche::auth::password::hash_password("MotDePasse123!", &cfg.auth).expect("hash");
    sqlx::query(
        "INSERT INTO user_account (id, email, last_name, first_name, password_hash) \
         VALUES ($1, $2, 'Cal', 'User', $3)",
    )
    .bind(user_id)
    .bind(&email)
    .bind(&password_hash)
    .execute(pool)
    .await
    .expect("user");
    sqlx::query(
        "INSERT INTO user_building (user_id, building_id, roles) \
         VALUES ($1, $2, ARRAY[$3]::user_role[])",
    )
    .bind(user_id)
    .bind(building_id)
    .bind(role)
    .execute(pool)
    .await
    .expect("role");
    user_id
}

#[tokio::test]
async fn manager_creates_event_with_title_and_date_only() {
    let (ctx, _pool) = setup().await;
    let server = common::test_server().await;

    let created = server
        .post("/api/v1/calendar/manual-events")
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .json(&json!({"title": "Réunion informelle", "event_date": "2026-07-01"}))
        .await;
    assert_eq!(created.status_code(), StatusCode::CREATED);
    let body: Value = created.json();
    assert_eq!(body["title"], "Réunion informelle");
    assert!(body["category"].is_null());
}

#[tokio::test]
async fn create_without_title_is_rejected() {
    let (ctx, _pool) = setup().await;
    let server = common::test_server().await;

    let created = server
        .post("/api/v1/calendar/manual-events")
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .json(&json!({"title": "   ", "event_date": "2026-07-01"}))
        .await;
    assert_eq!(created.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
        created.json::<Value>()["message"],
        "errors.calendar_event.title_required"
    );
}

#[tokio::test]
async fn board_member_edits_and_deletes_event_created_by_manager() {
    let (ctx, _pool) = setup().await;
    let server = common::test_server().await;

    let created = server
        .post("/api/v1/calendar/manual-events")
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .json(&json!({
            "title": "Visite technique",
            "event_date": "2026-07-05",
            "category": "visite"
        }))
        .await;
    assert_eq!(created.status_code(), StatusCode::CREATED);
    let id = created.json::<Value>()["id"].as_str().unwrap().to_string();

    let updated = server
        .put(&format!("/api/v1/calendar/manual-events/{id}"))
        .add_header("Authorization", format!("Bearer {}", ctx.board_member_jwt))
        .json(&json!({
            "title": "Visite technique — reportée",
            "event_date": "2026-07-06",
            "category": "visite"
        }))
        .await;
    assert_eq!(updated.status_code(), StatusCode::OK);
    assert_eq!(
        updated.json::<Value>()["title"],
        "Visite technique — reportée"
    );

    let deleted = server
        .delete(&format!("/api/v1/calendar/manual-events/{id}"))
        .add_header("Authorization", format!("Bearer {}", ctx.board_member_jwt))
        .await;
    assert_eq!(deleted.status_code(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn co_owner_can_list_but_not_create() {
    let (ctx, _pool) = setup().await;
    let server = common::test_server().await;

    let listed = server
        .get("/api/v1/calendar/manual-events")
        .add_header("Authorization", format!("Bearer {}", ctx.coowner_jwt))
        .await;
    assert_eq!(listed.status_code(), StatusCode::OK);

    let created = server
        .post("/api/v1/calendar/manual-events")
        .add_header("Authorization", format!("Bearer {}", ctx.coowner_jwt))
        .json(&json!({"title": "Interdit", "event_date": "2026-07-01"}))
        .await;
    assert_eq!(created.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn tenant_sees_manual_event_in_calendar_feed() {
    let (ctx, _pool) = setup().await;
    let server = common::test_server().await;

    let created = server
        .post("/api/v1/calendar/manual-events")
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .json(&json!({"title": "Permanence syndic", "event_date": "2026-07-10"}))
        .await;
    assert_eq!(created.status_code(), StatusCode::CREATED);

    let feed = server
        .get("/api/v1/calendar/events?from=2026-07-01&to=2026-07-31")
        .add_header("Authorization", format!("Bearer {}", ctx.tenant_jwt))
        .await;
    assert_eq!(feed.status_code(), StatusCode::OK);
    let events: Vec<Value> = feed.json();
    assert!(events.iter().any(|e| e["source"] == "manual"));
    let _ = ctx.building_id;
}

#[tokio::test]
async fn attaching_document_lists_it_and_survives_after_event_deletion_removed_document_stays() {
    let (ctx, pool) = setup().await;
    let server = common::test_server().await;

    let created = server
        .post("/api/v1/calendar/manual-events")
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .json(&json!({"title": "Réunion avec PJ", "event_date": "2026-07-15"}))
        .await;
    let event_id = created.json::<Value>()["id"].as_str().unwrap().to_string();

    let manager_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id FROM user_building WHERE building_id = $1 AND $2::user_role = ANY(roles)",
    )
    .bind(ctx.building_id)
    .bind("manager")
    .fetch_one(&pool)
    .await
    .expect("manager id");

    let document_id = sqlx::query_scalar::<_, Uuid>(
        r#"INSERT INTO document (building_id, uploaded_by, original_name, storage_path,
           bucket, mime_type, size_bytes, visible_to_roles, category)
           VALUES ($1, $2, 'ordre-du-jour.pdf', 'k', 'b', 'application/pdf', 10,
                   ARRAY['manager','board_member']::user_role[], 'libre')
           RETURNING id"#,
    )
    .bind(ctx.building_id)
    .bind(manager_id)
    .fetch_one(&pool)
    .await
    .expect("insert document");

    let attached = server
        .post(&format!(
            "/api/v1/calendar/manual-events/{event_id}/documents"
        ))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .json(&json!({"document_id": document_id}))
        .await;
    assert_eq!(attached.status_code(), StatusCode::NO_CONTENT);

    let listed = server
        .get(&format!(
            "/api/v1/calendar/manual-events/{event_id}/documents"
        ))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(listed.status_code(), StatusCode::OK);
    let docs: Value = listed.json();
    assert_eq!(docs.as_array().unwrap().len(), 1);

    let deleted = server
        .delete(&format!("/api/v1/calendar/manual-events/{event_id}"))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(deleted.status_code(), StatusCode::NO_CONTENT);

    // The document itself survives in the GED (only the link is CASCADE-deleted).
    let still_present: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM document WHERE id = $1 AND deleted_at IS NULL)",
    )
    .bind(document_id)
    .fetch_one(&pool)
    .await
    .expect("check document");
    assert!(still_present, "document should survive event deletion");
}

#[tokio::test]
async fn event_from_another_building_returns_404() {
    let (ctx, pool) = setup().await;
    let server = common::test_server().await;

    let other_building_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total) \
         VALUES ($1, 'Other building', '2 rue Autre', '75002', 'Paris', 1000)",
    )
    .bind(other_building_id)
    .execute(&pool)
    .await
    .expect("insert other building");

    let other_event_id = sqlx::query_scalar::<_, Uuid>(
        r#"INSERT INTO calendar_event (building_id, title, event_date, created_by)
           VALUES ($1, 'Autre immeuble', '2026-07-20',
               (SELECT user_id FROM user_building WHERE building_id = $2 LIMIT 1))
           RETURNING id"#,
    )
    .bind(other_building_id)
    .bind(ctx.building_id)
    .fetch_one(&pool)
    .await
    .expect("insert event for other building");

    let resp = server
        .get(&format!("/api/v1/calendar/manual-events/{other_event_id}"))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(resp.status_code(), StatusCode::NOT_FOUND);
}
