// Tenant read-visibility on general meetings (PRD US-AG-014):
// a tenant only sees `closed` meetings (results + published documents),
// attendance / votes / convocation checklist stay 403, and other roles
// are unaffected.
//
// AC1: tenant GET /meetings -> only `closed` meetings.
// AC2: tenant GET /meetings/{id} on `called` -> 404, on `closed` -> 200.
// AC3: tenant GET resolutions / results / documents on `closed` -> 200.
// AC4: tenant GET attendance / votes / convocation-checklist -> 403.
// AC5: co_owner still sees a `called` meeting (non-regression).

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use laruche::domain::Role;
use serde_json::{Value, json};
use uuid::Uuid;

struct Fixture {
    building_id: Uuid,
    manager_id: Uuid,
    tenant_id: Uuid,
    co_owner_id: Uuid,
}

async fn setup(pool: &sqlx::PgPool) -> Fixture {
    let building_id = Uuid::new_v4();
    let manager_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();
    let co_owner_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total)
         VALUES ($1, 'TenantBuilding', '1 rue Loc', '75001', 'Paris', 1000)",
    )
    .bind(building_id)
    .execute(pool)
    .await
    .expect("building");

    let password_hash =
        laruche::auth::password::hash_password("TestPass123!", &common::test_config().await.auth)
            .expect("hash");
    for (uid, email) in [
        (manager_id, "tenant-mgr"),
        (tenant_id, "tenant-loc"),
        (co_owner_id, "tenant-copro"),
    ] {
        sqlx::query(
            "INSERT INTO user_account (id, email, last_name, first_name, password_hash)
             VALUES ($1, $2, 'Test', 'User', $3)",
        )
        .bind(uid)
        .bind(format!("{email}-{}@test.local", Uuid::new_v4()))
        .bind(&password_hash)
        .execute(pool)
        .await
        .expect("user");
    }

    for (uid, role) in [
        (manager_id, Role::Manager),
        (tenant_id, Role::Tenant),
        (co_owner_id, Role::CoOwner),
    ] {
        sqlx::query(
            "INSERT INTO user_building (user_id, building_id, roles)
             VALUES ($1, $2, ARRAY[$3]::user_role[])",
        )
        .bind(uid)
        .bind(building_id)
        .bind(role.to_string())
        .execute(pool)
        .await
        .expect("user_building");
    }

    Fixture {
        building_id,
        manager_id,
        tenant_id,
        co_owner_id,
    }
}

fn token_for(
    user_id: Uuid,
    building_id: Uuid,
    role: Role,
    cfg: &laruche::config::AppConfig,
) -> String {
    laruche::auth::jwt::generate_access_token(
        user_id,
        building_id,
        vec![role],
        &cfg.auth.jwt_secret,
        cfg.auth.access_token_ttl_secs,
    )
    .expect("token")
}

async fn create_meeting(server: &TestServer, auth: &str, meeting_date: &str) -> Uuid {
    let resp = server
        .post("/api/v1/meetings")
        .add_header("Authorization", auth)
        .json(&json!({
            "meeting_type": "annual",
            "meeting_date": meeting_date,
            "location": "Salle commune",
            "agenda": "Ordre du jour",
        }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::CREATED);
    resp.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap()
}

async fn create_resolution(server: &TestServer, auth: &str, meeting_id: Uuid) -> Uuid {
    let resp = server
        .post(&format!("/api/v1/meetings/{meeting_id}/resolutions"))
        .add_header("Authorization", auth)
        .json(&json!({
            "number": 1,
            "title": "Résolution 1",
            "vote_type": "simple_majority",
        }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::CREATED);
    resp.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap()
}

async fn set_status(pool: &sqlx::PgPool, meeting_id: Uuid, status: &str) {
    sqlx::query("UPDATE general_meeting SET status = $2::meeting_status WHERE id = $1")
        .bind(meeting_id)
        .bind(status)
        .execute(pool)
        .await
        .expect("set status");
}

#[tokio::test]
async fn tenant_read_visibility_on_meetings() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let server = common::test_server_with_pool(cfg.clone(), pool.clone()).await;

    let manager_auth = format!(
        "Bearer {}",
        token_for(f.manager_id, f.building_id, Role::Manager, &cfg)
    );
    let tenant_auth = format!(
        "Bearer {}",
        token_for(f.tenant_id, f.building_id, Role::Tenant, &cfg)
    );
    let co_owner_auth = format!(
        "Bearer {}",
        token_for(f.co_owner_id, f.building_id, Role::CoOwner, &cfg)
    );

    let closed_meeting = create_meeting(&server, &manager_auth, "2026-03-10T18:00:00Z").await;
    let resolution_id = create_resolution(&server, &manager_auth, closed_meeting).await;
    let called_meeting = create_meeting(&server, &manager_auth, "2027-03-10T18:00:00Z").await;
    set_status(&pool, called_meeting, "called").await;
    set_status(&pool, closed_meeting, "closed").await;

    // AC1: tenant list -> only the closed meeting.
    let list = server
        .get("/api/v1/meetings")
        .add_header("Authorization", &tenant_auth)
        .await;
    assert_eq!(list.status_code(), StatusCode::OK);
    let meetings = list.json::<Vec<Value>>();
    let ids: Vec<&str> = meetings.iter().filter_map(|m| m["id"].as_str()).collect();
    assert!(ids.contains(&closed_meeting.to_string().as_str()));
    assert!(!ids.contains(&called_meeting.to_string().as_str()));

    // AC2: tenant get on called -> 404, on closed -> 200.
    let resp = server
        .get(&format!("/api/v1/meetings/{called_meeting}"))
        .add_header("Authorization", &tenant_auth)
        .await;
    assert_eq!(resp.status_code(), StatusCode::NOT_FOUND);
    let resp = server
        .get(&format!("/api/v1/meetings/{closed_meeting}"))
        .add_header("Authorization", &tenant_auth)
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);

    // AC3: tenant reads resolutions / results / documents on a closed meeting.
    for path in [
        format!("/api/v1/meetings/{closed_meeting}/resolutions"),
        format!("/api/v1/meetings/{closed_meeting}/resolutions/{resolution_id}/results"),
        format!("/api/v1/meetings/{closed_meeting}/documents"),
    ] {
        let resp = server
            .get(&path)
            .add_header("Authorization", &tenant_auth)
            .await;
        assert_eq!(resp.status_code(), StatusCode::OK, "GET {path}");
    }

    // tenant cannot read resolutions of a non-closed meeting (404).
    let resp = server
        .get(&format!("/api/v1/meetings/{called_meeting}/resolutions"))
        .add_header("Authorization", &tenant_auth)
        .await;
    assert_eq!(resp.status_code(), StatusCode::NOT_FOUND);

    // AC4: tenant -> 403 on attendance, votes, convocation checklist.
    for path in [
        format!("/api/v1/meetings/{closed_meeting}/attendance"),
        format!("/api/v1/meetings/{closed_meeting}/resolutions/{resolution_id}/votes"),
        format!("/api/v1/meetings/{closed_meeting}/convocation-checklist"),
        format!("/api/v1/meetings/{closed_meeting}/minutes/draft"),
    ] {
        let resp = server
            .get(&path)
            .add_header("Authorization", &tenant_auth)
            .await;
        assert_eq!(resp.status_code(), StatusCode::FORBIDDEN, "GET {path}");
    }

    // AC5: co_owner still sees the called meeting (non-regression).
    let resp = server
        .get(&format!("/api/v1/meetings/{called_meeting}"))
        .add_header("Authorization", &co_owner_auth)
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let list = server
        .get("/api/v1/meetings")
        .add_header("Authorization", &co_owner_auth)
        .await;
    let meetings = list.json::<Vec<Value>>();
    assert_eq!(meetings.len(), 2);
}
