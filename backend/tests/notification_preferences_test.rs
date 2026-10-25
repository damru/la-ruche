// GET/PUT/DELETE /me/notification-preferences — server-side notification prefs.

mod common;

use axum::http::StatusCode;
use serde_json::{Value, json};

/// Rebuilds a cookie-free server on the same database as `pool` — the auth
/// extractor prefers the cookie over the Authorization header, so a second
/// user's JWT must be sent without the manager's session cookies.
async fn cookie_free_server(pool: &sqlx::PgPool) -> axum_test::TestServer {
    let cfg = common::test_config().await;
    let mut cfg2 = cfg.clone();
    cfg2.db.url = pool
        .connect_options()
        .get_database()
        .map(|db| {
            let mut base = url::Url::parse(&cfg.db.url).expect("parse base url");
            base.set_path(&format!("/{db}"));
            base.to_string()
        })
        .expect("db name");
    common::test_server_with_pool(cfg2, pool.clone()).await
}

/// Boots an isolated server (with a pool on the same database), bootstraps a
/// manager, returns the session-ready server and the pool.
async fn prefs_test_server() -> (axum_test::TestServer, sqlx::PgPool) {
    let (mut server, pool) = common::isolated_test_server_with_pool().await;
    let bootstrap = server
        .post("/api/v1/onboarding/bootstrap")
        .json(&common::bootstrap_payload())
        .await;
    assert_eq!(bootstrap.status_code(), StatusCode::CREATED);
    server.add_cookies(bootstrap.cookies());
    (server, pool)
}

#[tokio::test]
async fn get_returns_catalog_defaults_without_overrides() {
    let (server, _pool) = prefs_test_server().await;

    let resp = server.get("/api/v1/me/notification-preferences").await;
    assert_eq!(
        resp.status_code(),
        StatusCode::OK,
        "get failed: {}",
        resp.text()
    );
    let body: Value = resp.json();
    let events = body["events"].as_array().expect("events array");
    assert_eq!(events.len(), 16);

    // Mandatory events are always enabled.
    for key in ["ag_convocation", "levy_call_issued", "account_security"] {
        let event = events.iter().find(|e| e["key"] == key).expect("event");
        assert_eq!(event["mandatory"], true);
        assert_eq!(event["enabled"], true);
    }
    // Optional defaults: weekly_digest + claim_resolved off, the rest on.
    for event in events {
        if event["mandatory"] == true {
            continue;
        }
        let expected = event["key"] != "weekly_digest" && event["key"] != "claim_resolved";
        assert_eq!(event["enabled"], expected, "default for {}", event["key"]);
    }
}

#[tokio::test]
async fn put_persists_overrides_and_is_idempotent() {
    let (server, _pool) = prefs_test_server().await;

    let resp = server
        .put("/api/v1/me/notification-preferences")
        .json(&json!({ "prefs": { "weekly_digest": true, "ag_minutes": false } }))
        .await;
    assert_eq!(
        resp.status_code(),
        StatusCode::OK,
        "put failed: {}",
        resp.text()
    );
    let body: Value = resp.json();
    let events = body["events"].as_array().expect("events array");
    let find = |key: &str| events.iter().find(|e| e["key"] == key).expect("event");
    assert_eq!(find("weekly_digest")["enabled"], true);
    assert_eq!(find("ag_minutes")["enabled"], false);

    // GET reflects the persisted overrides.
    let resp = server.get("/api/v1/me/notification-preferences").await;
    let body: Value = resp.json();
    let events = body["events"].as_array().expect("events array");
    let find = |key: &str| events.iter().find(|e| e["key"] == key).expect("event");
    assert_eq!(find("weekly_digest")["enabled"], true);
    assert_eq!(find("ag_minutes")["enabled"], false);

    // Upsert is idempotent: a second identical PUT succeeds and keeps values.
    let resp = server
        .put("/api/v1/me/notification-preferences")
        .json(&json!({ "prefs": { "weekly_digest": true, "ag_minutes": false } }))
        .await;
    assert_eq!(
        resp.status_code(),
        StatusCode::OK,
        "second put: {}",
        resp.text()
    );
}

#[tokio::test]
async fn put_rejects_unknown_key_and_mandatory_event() {
    let (server, _pool) = prefs_test_server().await;

    // Unknown key → 400 unknown_event, nothing written.
    let resp = server
        .put("/api/v1/me/notification-preferences")
        .json(&json!({ "prefs": { "no_such_event": true } }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
    assert!(
        resp.text().contains("unknown_event"),
        "body: {}",
        resp.text()
    );

    // Mandatory event cannot be disabled → 400 mandatory_event.
    let resp = server
        .put("/api/v1/me/notification-preferences")
        .json(&json!({ "prefs": { "ag_convocation": false } }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
    assert!(
        resp.text().contains("mandatory_event"),
        "body: {}",
        resp.text()
    );

    // Nothing was persisted by the rejected calls.
    let resp = server.get("/api/v1/me/notification-preferences").await;
    let body: Value = resp.json();
    let events = body["events"].as_array().expect("events array");
    let mandatory = events
        .iter()
        .find(|e| e["key"] == "ag_convocation")
        .expect("event");
    assert_eq!(mandatory["enabled"], true);
}

#[tokio::test]
async fn overrides_are_isolated_between_users() {
    let (server, pool) = prefs_test_server().await;

    // Manager sets an override.
    let resp = server
        .put("/api/v1/me/notification-preferences")
        .json(&json!({ "prefs": { "weekly_digest": true } }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);

    // Seed a second user (co_owner) in the same building, with a JWT.
    let building_id = {
        let me = server.get("/api/v1/auth/me").await;
        assert_eq!(me.status_code(), StatusCode::OK);
        uuid::Uuid::parse_str(
            me.json::<Value>()["building_id"]
                .as_str()
                .expect("building_id"),
        )
        .expect("uuid")
    };
    let cfg = common::test_config().await;
    let second_user_id = uuid::Uuid::new_v4();
    let second_email = format!("second-{}@test.local", second_user_id.simple());
    let password_hash =
        laruche::auth::password::hash_password("MotDePasse123!", &cfg.auth).expect("hash");
    sqlx::query(
        "INSERT INTO user_account (id, email, last_name, first_name, password_hash) \
         VALUES ($1, $2, 'Doe', 'Jane', $3)",
    )
    .bind(second_user_id)
    .bind(&second_email)
    .bind(&password_hash)
    .execute(&pool)
    .await
    .expect("user");
    sqlx::query(
        "INSERT INTO user_building (user_id, building_id, roles) \
         VALUES ($1, $2, ARRAY[$3]::user_role[])",
    )
    .bind(second_user_id)
    .bind(building_id)
    .bind(laruche::domain::Role::CoOwner.to_string())
    .execute(&pool)
    .await
    .expect("role");
    let second_jwt = laruche::auth::jwt::generate_access_token(
        second_user_id,
        building_id,
        vec![laruche::domain::Role::CoOwner],
        &cfg.auth.jwt_secret,
        cfg.auth.access_token_ttl_secs,
    )
    .expect("jwt");

    // The second user reads the endpoint with their own JWT: catalog defaults,
    // not the manager's override.
    let second_server = cookie_free_server(&pool).await;
    let resp = second_server
        .get("/api/v1/me/notification-preferences")
        .add_header("Authorization", format!("Bearer {second_jwt}"))
        .await;
    assert_eq!(
        resp.status_code(),
        StatusCode::OK,
        "second user get failed: {}",
        resp.text()
    );
    let body: Value = resp.json();
    let events = body["events"].as_array().expect("events array");
    let weekly = events
        .iter()
        .find(|e| e["key"] == "weekly_digest")
        .expect("event");
    assert_eq!(weekly["enabled"], false, "second user must see the default");

    // Overrides are stored per user: the manager's row is the only one.
    let rows: Vec<(uuid::Uuid, String, bool)> =
        sqlx::query_as("SELECT user_id, event_key, enabled FROM user_notification_preference")
            .fetch_all(&pool)
            .await
            .expect("query overrides");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].1, "weekly_digest");
    assert!(rows[0].2);
}

#[tokio::test]
async fn delete_drops_overrides_back_to_defaults_and_is_idempotent() {
    let (server, pool) = prefs_test_server().await;

    // Set two overrides.
    let resp = server
        .put("/api/v1/me/notification-preferences")
        .json(&json!({ "prefs": { "weekly_digest": true, "ag_minutes": false } }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);

    // DELETE drops all overrides; response is the catalog defaults.
    let resp = server.delete("/api/v1/me/notification-preferences").await;
    assert_eq!(
        resp.status_code(),
        StatusCode::OK,
        "delete failed: {}",
        resp.text()
    );
    let body: Value = resp.json();
    let events = body["events"].as_array().expect("events array");
    assert_eq!(events.len(), 16);
    let find = |key: &str| events.iter().find(|e| e["key"] == key).expect("event");
    assert_eq!(find("weekly_digest")["enabled"], false);
    assert_eq!(find("ag_minutes")["enabled"], true);

    // Overrides are gone from the table.
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_notification_preference")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(count, 0);

    // Idempotent: a second DELETE on an empty set succeeds.
    let resp = server.delete("/api/v1/me/notification-preferences").await;
    assert_eq!(
        resp.status_code(),
        StatusCode::OK,
        "second delete: {}",
        resp.text()
    );

    // GET confirms the defaults after the reset.
    let resp = server.get("/api/v1/me/notification-preferences").await;
    let body: Value = resp.json();
    let events = body["events"].as_array().expect("events array");
    let weekly = events
        .iter()
        .find(|e| e["key"] == "weekly_digest")
        .expect("event");
    assert_eq!(weekly["enabled"], false);
}

#[tokio::test]
async fn unauthenticated_requests_are_rejected() {
    let server = common::isolated_test_server().await;

    let resp = server.get("/api/v1/me/notification-preferences").await;
    assert_eq!(resp.status_code(), StatusCode::UNAUTHORIZED);

    let resp = server
        .put("/api/v1/me/notification-preferences")
        .json(&json!({ "prefs": { "weekly_digest": true } }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::UNAUTHORIZED);

    let resp = server.delete("/api/v1/me/notification-preferences").await;
    assert_eq!(resp.status_code(), StatusCode::UNAUTHORIZED);
}
