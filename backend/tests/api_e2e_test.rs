// End-to-end API tests — nominal (happy-path) flows with result assertions.
//
// Exercises the full HTTP stack against the self-contained harness (embedded
// PostgreSQL + wiremock S3). The whole flow runs as a single sequential test
// because instance bootstrap is a once-per-database operation: splitting it
// across parallel tests would race on the first-run guard.

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{Value, json};

#[tokio::test]
async fn nominal_api_flow() {
    let mut server = common::isolated_test_server().await;

    // ── Onboarding: fresh instance needs onboarding ───────────────────
    let status = server.get("/api/v1/onboarding/status").await;
    assert_eq!(status.status_code(), StatusCode::OK);
    assert_eq!(status.json::<Value>()["needs_onboarding"], true);

    // ── Bootstrap: creates building + first manager, returns a session ─
    let bootstrap = server
        .post("/api/v1/onboarding/bootstrap")
        .json(&common::bootstrap_payload())
        .await;
    assert_eq!(bootstrap.status_code(), StatusCode::CREATED);
    let bootstrap_body: Value = bootstrap.json();
    assert!(bootstrap_body["user_id"].is_string());
    assert!(bootstrap_body["building_id"].is_string());
    assert_eq!(bootstrap_body["roles"], json!(["manager"]));
    // Explicitly carry session cookies forward; belt-and-suspenders over save_cookies().
    server.add_cookies(bootstrap.cookies());

    // ── Onboarding status flips to false once a user exists ───────────
    let status_after = server.get("/api/v1/onboarding/status").await;
    assert_eq!(status_after.status_code(), StatusCode::OK);
    assert_eq!(status_after.json::<Value>()["needs_onboarding"], false);

    // ── Authenticated session carries over via the saved cookie ───────
    let me = server.get("/api/v1/auth/me").await;
    assert_eq!(
        me.status_code(),
        StatusCode::OK,
        "auth/me failed — status {} body: {}",
        me.status_code(),
        me.text()
    );
    let me_body: Value = me.json();
    assert_eq!(me_body["email"], "syndic@test.fr");
    assert_eq!(me_body["roles"], json!(["manager"]));

    // ── Building reflects the bootstrap payload ───────────────────────
    let building = server.get("/api/v1/building").await;
    assert_eq!(building.status_code(), StatusCode::OK);
    let building_body: Value = building.json();
    assert_eq!(building_body["name"], "Résidence Les Tests");
    assert_eq!(building_body["shares_total"], 1000);
    assert_eq!(building_body["city"], "Paris");

    // ── Units: create → list → get ────────────────────────────────────
    let created_unit = server
        .post("/api/v1/units")
        .json(&json!({
            "unit_number": "A001",
            "unit_type": "apartment",
            "floor": "2"
        }))
        .await;
    assert_eq!(created_unit.status_code(), StatusCode::CREATED);
    let unit_body: Value = created_unit.json();
    let unit_id = unit_body["id"].as_str().expect("unit id").to_string();
    assert_eq!(unit_body["unit_number"], "A001");
    assert_eq!(unit_body["unit_type"], "apartment");

    let units = server.get("/api/v1/units").await;
    assert_eq!(units.status_code(), StatusCode::OK);
    let units_body: Value = units.json();
    let units_arr = units_body.as_array().expect("units array");
    assert_eq!(units_arr.len(), 1);
    assert_eq!(units_arr[0]["id"], unit_id);

    let unit = server.get(&format!("/api/v1/units/{unit_id}")).await;
    assert_eq!(unit.status_code(), StatusCode::OK);
    assert_eq!(unit.json::<Value>()["unit_number"], "A001");

    // ── Governance: create a mandate → list ───────────────────────────
    let mandate = server
        .post("/api/v1/governance/mandates")
        .json(&json!({
            "role": "council_member",
            "holder_name": "Mme Bernard",
            "start_date": "2026-01-01"
        }))
        .await;
    assert_eq!(mandate.status_code(), StatusCode::CREATED);
    assert_eq!(mandate.json::<Value>()["role"], "council_member");

    let mandates = server.get("/api/v1/governance/mandates").await;
    assert_eq!(mandates.status_code(), StatusCode::OK);
    assert_eq!(mandates.json::<Value>().as_array().expect("array").len(), 1);
}

// ── EP-018-S05: Pattern B vendor_contract amendment lifecycle ─────────────

/// Boots an isolated server and bootstraps a manager session, ready for
/// authenticated contract requests.
async fn amendment_test_server() -> TestServer {
    let mut server = common::isolated_test_server().await;
    let bootstrap = server
        .post("/api/v1/onboarding/bootstrap")
        .json(&common::bootstrap_payload())
        .await;
    assert_eq!(bootstrap.status_code(), StatusCode::CREATED);
    server.add_cookies(bootstrap.cookies());
    server
}

/// Minimal valid contract payload (financial + legal fields populated so
/// activation computes reminders).
fn contract_payload() -> Value {
    json!({
        "vendor_name": "ACME Nettoyage",
        "vendor_email": "contact@acme.fr",
        "vendor_phone": "0102030405",
        "subject": "Entretien parties communes",
        "contract_type": "maintenance",
        "start_date": "2026-01-01",
        "end_date": "2026-12-31",
        "amount": "1200.00",
        "payment_frequency": "monthly",
        "auto_renewal": true,
        "notice_days": 90
    })
}

async fn create_contract(server: &TestServer) -> String {
    let created = server
        .post("/api/v1/contracts")
        .json(&contract_payload())
        .await;
    assert_eq!(
        created.status_code(),
        StatusCode::CREATED,
        "create contract failed: {}",
        created.text()
    );
    let body: Value = created.json();
    assert_eq!(body["status"], "draft");
    body["id"].as_str().expect("contract id").to_string()
}

async fn activate(server: &TestServer, id: &str) {
    let resp = server
        .post(&format!("/api/v1/contracts/{id}/activate"))
        .await;
    assert_eq!(
        resp.status_code(),
        StatusCode::OK,
        "activate failed: {}",
        resp.text()
    );
    assert_eq!(resp.json::<Value>()["status"], "active");
}

// Test 1: creation → 201, status draft, no reminders.
#[tokio::test]
async fn contract_created_as_draft_without_reminders() {
    let server = amendment_test_server().await;
    let id = create_contract(&server).await;

    let reminders = server
        .get(&format!("/api/v1/contracts/{id}/reminders"))
        .await;
    assert_eq!(reminders.status_code(), StatusCode::OK);
    assert_eq!(
        reminders.json::<Value>().as_array().expect("array").len(),
        0,
        "a draft contract must not have reminders"
    );
}

// Test 2: activation → 200, status active, reminders computed.
#[tokio::test]
async fn activation_sets_active_and_computes_reminders() {
    let server = amendment_test_server().await;
    let id = create_contract(&server).await;
    activate(&server, &id).await;

    let reminders = server
        .get(&format!("/api/v1/contracts/{id}/reminders"))
        .await;
    assert_eq!(reminders.status_code(), StatusCode::OK);
    assert!(
        !reminders
            .json::<Value>()
            .as_array()
            .expect("array")
            .is_empty(),
        "an active contract with notice_days/end_date must have reminders"
    );
}

// Test 3: re-activation of an already-active contract → 400 not_activatable.
#[tokio::test]
async fn reactivation_is_rejected() {
    let server = amendment_test_server().await;
    let id = create_contract(&server).await;
    activate(&server, &id).await;

    let resp = server
        .post(&format!("/api/v1/contracts/{id}/activate"))
        .await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
        resp.json::<Value>()["message"],
        "errors.contracts.not_activatable"
    );
}

// Test 4: editing a descriptive field on an active contract → 200.
#[tokio::test]
async fn descriptive_field_editable_on_active_contract() {
    let server = amendment_test_server().await;
    let id = create_contract(&server).await;
    activate(&server, &id).await;

    let mut payload = contract_payload();
    payload["vendor_email"] = json!("nouveau@acme.fr");
    let resp = server
        .put(&format!("/api/v1/contracts/{id}"))
        .json(&payload)
        .await;
    assert_eq!(
        resp.status_code(),
        StatusCode::OK,
        "descriptive edit failed: {}",
        resp.text()
    );
    assert_eq!(resp.json::<Value>()["vendor_email"], "nouveau@acme.fr");
}

// Test 5: editing a frozen field on an active contract → 400 frozen.
#[tokio::test]
async fn frozen_field_rejected_on_active_contract() {
    let server = amendment_test_server().await;
    let id = create_contract(&server).await;
    activate(&server, &id).await;

    let mut payload = contract_payload();
    payload["amount"] = json!("9999.00");
    let resp = server
        .put(&format!("/api/v1/contracts/{id}"))
        .json(&payload)
        .await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
        resp.json::<Value>()["message"],
        "errors.immutability.frozen"
    );
}

// Test 6: creating an amendment on an active contract → 201, draft, predecessor set.
#[tokio::test]
async fn amendment_created_as_draft_linked_to_predecessor() {
    let server = amendment_test_server().await;
    let id = create_contract(&server).await;
    activate(&server, &id).await;

    let mut payload = contract_payload();
    payload["amount"] = json!("1500.00");
    let resp = server
        .post(&format!("/api/v1/contracts/{id}/amendments"))
        .json(&payload)
        .await;
    assert_eq!(
        resp.status_code(),
        StatusCode::CREATED,
        "amendment creation failed: {}",
        resp.text()
    );
    let body: Value = resp.json();
    assert_eq!(body["status"], "draft");
    assert_eq!(body["predecessor_id"], id);
}

// Test 7: a second pending amendment → 400 amendment_already_pending.
#[tokio::test]
async fn second_pending_amendment_is_rejected() {
    let server = amendment_test_server().await;
    let id = create_contract(&server).await;
    activate(&server, &id).await;

    let first = server
        .post(&format!("/api/v1/contracts/{id}/amendments"))
        .json(&contract_payload())
        .await;
    assert_eq!(first.status_code(), StatusCode::CREATED);

    let second = server
        .post(&format!("/api/v1/contracts/{id}/amendments"))
        .json(&contract_payload())
        .await;
    assert_eq!(second.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
        second.json::<Value>()["message"],
        "errors.contracts.amendment_already_pending"
    );
}

// Test 8: activating an amendment closes its predecessor atomically.
#[tokio::test]
async fn activating_amendment_closes_predecessor() {
    let server = amendment_test_server().await;
    let id = create_contract(&server).await;
    activate(&server, &id).await;

    let amendment = server
        .post(&format!("/api/v1/contracts/{id}/amendments"))
        .json(&contract_payload())
        .await;
    assert_eq!(amendment.status_code(), StatusCode::CREATED);
    let amendment_id = amendment.json::<Value>()["id"]
        .as_str()
        .expect("amendment id")
        .to_string();

    activate(&server, &amendment_id).await;

    let predecessor = server.get(&format!("/api/v1/contracts/{id}")).await;
    assert_eq!(predecessor.status_code(), StatusCode::OK);
    assert_eq!(predecessor.json::<Value>()["status"], "terminated");
}

// Test 11: activation and amendment creation write audit_log entries
// (NFR-Audit-001) — every mutation, financial or contractual, must be traced.
#[tokio::test]
async fn amendment_lifecycle_writes_audit_trail() {
    let (mut server, pool) = common::isolated_test_server_with_pool().await;
    let bootstrap = server
        .post("/api/v1/onboarding/bootstrap")
        .json(&common::bootstrap_payload())
        .await;
    assert_eq!(bootstrap.status_code(), StatusCode::CREATED);
    server.add_cookies(bootstrap.cookies());

    let id = create_contract(&server).await;
    activate(&server, &id).await;

    let amendment = server
        .post(&format!("/api/v1/contracts/{id}/amendments"))
        .json(&contract_payload())
        .await;
    assert_eq!(amendment.status_code(), StatusCode::CREATED);
    let amendment_id = amendment.json::<Value>()["id"]
        .as_str()
        .expect("amendment id")
        .to_string();
    activate(&server, &amendment_id).await;

    let entries: Vec<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT entity_id::text, new_values->>'status', old_values->>'status'
         FROM audit_log
         WHERE entity_type = 'vendor_contract'
         ORDER BY changed_at",
    )
    .fetch_all(&pool)
    .await
    .expect("query audit_log");

    // Expect 4 rows across the lifecycle:
    //   1. original contract created draft -> activated to 'active'
    //   2. amendment created in 'draft' (predecessor still active)
    //   3. amendment activated to 'active'
    //   4. original contract closed to 'terminated' as a side effect of (3)
    assert_eq!(
        entries.len(),
        4,
        "expected one audit_log row per traced transition, got {entries:?}"
    );

    let amendment_created = entries
        .iter()
        .find(|(entity_id, new_status, _)| entity_id == &amendment_id && new_status == "draft");
    assert!(
        amendment_created.is_some(),
        "amendment creation must be audited: {entries:?}"
    );

    let predecessor_closed = entries.iter().find(|(entity_id, new_status, old_status)| {
        entity_id == &id && new_status == "terminated" && old_status.as_deref() == Some("active")
    });
    assert!(
        predecessor_closed.is_some(),
        "predecessor closure on activation must be audited: {entries:?}"
    );
}

// Test 9: amendment chain identical from any link.
#[tokio::test]
async fn amendment_chain_identical_from_any_link() {
    let server = amendment_test_server().await;
    let id = create_contract(&server).await;
    activate(&server, &id).await;

    let amendment = server
        .post(&format!("/api/v1/contracts/{id}/amendments"))
        .json(&contract_payload())
        .await;
    let amendment_id = amendment.json::<Value>()["id"]
        .as_str()
        .expect("amendment id")
        .to_string();
    activate(&server, &amendment_id).await;

    let chain_from_head = server
        .get(&format!("/api/v1/contracts/{id}/amendments"))
        .await;
    let chain_from_tail = server
        .get(&format!("/api/v1/contracts/{amendment_id}/amendments"))
        .await;
    assert_eq!(chain_from_head.status_code(), StatusCode::OK);
    assert_eq!(chain_from_tail.status_code(), StatusCode::OK);

    let head: Value = chain_from_head.json();
    let tail: Value = chain_from_tail.json();
    assert_eq!(head.as_array().expect("array").len(), 2);
    assert_eq!(head, tail, "chain must be identical from any link");
}

// Test 10: multi-level chain (3 links) identical from link 1 and link 3.
#[tokio::test]
async fn multi_level_amendment_chain() {
    let server = amendment_test_server().await;
    let id1 = create_contract(&server).await;
    activate(&server, &id1).await;

    let a2 = server
        .post(&format!("/api/v1/contracts/{id1}/amendments"))
        .json(&contract_payload())
        .await;
    let id2 = a2.json::<Value>()["id"].as_str().expect("id2").to_string();
    activate(&server, &id2).await;

    let a3 = server
        .post(&format!("/api/v1/contracts/{id2}/amendments"))
        .json(&contract_payload())
        .await;
    let id3 = a3.json::<Value>()["id"].as_str().expect("id3").to_string();
    activate(&server, &id3).await;

    let from1 = server
        .get(&format!("/api/v1/contracts/{id1}/amendments"))
        .await;
    let from3 = server
        .get(&format!("/api/v1/contracts/{id3}/amendments"))
        .await;
    let c1: Value = from1.json();
    let c3: Value = from3.json();
    assert_eq!(c1.as_array().expect("array").len(), 3);
    assert_eq!(c1, c3, "multi-level chain must match from link 1 and 3");
}

// Test 11: amending a terminated contract → 400 not_amendable.
#[tokio::test]
async fn terminated_contract_not_amendable() {
    let server = amendment_test_server().await;
    let id = create_contract(&server).await;
    activate(&server, &id).await;

    let terminate = server
        .put(&format!("/api/v1/contracts/{id}/terminate"))
        .await;
    assert_eq!(terminate.status_code(), StatusCode::NO_CONTENT);

    let resp = server
        .post(&format!("/api/v1/contracts/{id}/amendments"))
        .json(&contract_payload())
        .await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
        resp.json::<Value>()["message"],
        "errors.contracts.not_amendable"
    );
}

// Test 12: deleting a predecessor with a successor → 400 has_successor.
#[tokio::test]
async fn deleting_predecessor_with_successor_is_blocked() {
    let server = amendment_test_server().await;
    let id = create_contract(&server).await;
    activate(&server, &id).await;

    let amendment = server
        .post(&format!("/api/v1/contracts/{id}/amendments"))
        .json(&contract_payload())
        .await;
    assert_eq!(amendment.status_code(), StatusCode::CREATED);

    let resp = server.delete(&format!("/api/v1/contracts/{id}")).await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
        resp.json::<Value>()["message"],
        "errors.contracts.has_successor"
    );
}

#[tokio::test]
async fn login_with_invalid_credentials_is_unauthorized() {
    let server = common::test_server().await;
    let resp = server
        .post("/api/v1/auth/login")
        .json(&json!({ "email": "nobody@test.fr", "password": "WrongPassword1!" }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::UNAUTHORIZED);
}
