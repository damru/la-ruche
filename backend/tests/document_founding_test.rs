// Integration tests for founding-document typing and versioning
// (EP-005-S06, LEGAL §1.2: règlement de copropriété / EDD are permanent legal
// acts — replaced via a chained new version, never hard-deleted).
//
// Run with: cargo test --test document_founding_test

mod common;

use axum::http::StatusCode;
use laruche::domain::Role;
use serde_json::{Value, json};
use uuid::Uuid;

struct Fixture {
    building_id: Uuid,
    manager_id: Uuid,
}

async fn setup(pool: &sqlx::PgPool) -> Fixture {
    let building_id = Uuid::new_v4();
    let manager_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total)
         VALUES ($1, 'TestBuilding', '1 rue Test', '75001', 'Paris', 1000)",
    )
    .bind(building_id)
    .execute(pool)
    .await
    .expect("building");

    let password_hash =
        laruche::auth::password::hash_password("TestPass123!", &common::test_config().await.auth)
            .expect("hash");
    sqlx::query(
        "INSERT INTO user_account (id, email, last_name, first_name, password_hash)
         VALUES ($1, $2, 'Test', 'Manager', $3)",
    )
    .bind(manager_id)
    .bind(format!("mgr-{}@test.local", Uuid::new_v4()))
    .bind(&password_hash)
    .execute(pool)
    .await
    .expect("user");

    sqlx::query(
        "INSERT INTO user_building (user_id, building_id, roles)
         VALUES ($1, $2, ARRAY[$3]::user_role[])",
    )
    .bind(manager_id)
    .bind(building_id)
    .bind(Role::Manager.to_string())
    .execute(pool)
    .await
    .expect("user_building");

    Fixture {
        building_id,
        manager_id,
    }
}

fn token_for(user_id: Uuid, building_id: Uuid, cfg: &laruche::config::AppConfig) -> String {
    laruche::auth::jwt::generate_access_token(
        user_id,
        building_id,
        vec![Role::Manager],
        &cfg.auth.jwt_secret,
        cfg.auth.access_token_ttl_secs,
    )
    .expect("token")
}

async fn confirm_upload(
    server: &axum_test::TestServer,
    auth: &str,
    building_id: Uuid,
    original_name: &str,
    category: &str,
    supersedes_id: Option<Uuid>,
    reason: Option<&str>,
) -> axum_test::TestResponse {
    let bucket = format!("laruche-{}", building_id.simple());
    let mut body = json!({
        "key": format!("uploads/{}.pdf", Uuid::new_v4()),
        "bucket": bucket,
        "original_name": original_name,
        "mime_type": "application/pdf",
        "size_bytes": 1024,
        "visible_roles": ["manager"],
        "category": category,
    });
    if let Some(id) = supersedes_id {
        body["supersedes_id"] = json!(id);
    }
    if let Some(r) = reason {
        body["reason"] = json!(r);
    }

    server
        .post("/api/v1/documents")
        .add_header("Authorization", auth)
        .json(&body)
        .await
}

#[tokio::test]
async fn founding_category_is_recorded_and_marked_current() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let resp = confirm_upload(
        &server,
        &auth,
        f.building_id,
        "reglement.pdf",
        "reglement_copropriete",
        None,
        None,
    )
    .await;
    assert_eq!(resp.status_code(), StatusCode::CREATED);
    let body: Value = resp.json();
    assert_eq!(body["category"].as_str().unwrap(), "reglement_copropriete");
    assert!(body["is_current"].as_bool().unwrap());
    assert!(body["supersedes_id"].is_null());
}

#[tokio::test]
async fn only_one_current_founding_document_per_category_per_building() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let first = confirm_upload(
        &server,
        &auth,
        f.building_id,
        "edd-v1.pdf",
        "edd",
        None,
        None,
    )
    .await;
    assert_eq!(first.status_code(), StatusCode::CREATED);

    // A second independent upload (not a version replacement) of the same
    // founding category must be rejected: there can be only one current EDD.
    let second = confirm_upload(
        &server,
        &auth,
        f.building_id,
        "edd-v1-bis.pdf",
        "edd",
        None,
        None,
    )
    .await;
    assert_eq!(second.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn publishing_a_new_version_supersedes_the_previous_one() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let v1 = confirm_upload(
        &server,
        &auth,
        f.building_id,
        "reglement-v1.pdf",
        "reglement_copropriete",
        None,
        None,
    )
    .await;
    let v1_id: Uuid = v1.json::<Value>()["id"].as_str().unwrap().parse().unwrap();

    let v2 = confirm_upload(
        &server,
        &auth,
        f.building_id,
        "reglement-v2.pdf",
        "reglement_copropriete",
        Some(v1_id),
        Some("Modification votée en AG du 12/03"),
    )
    .await;
    assert_eq!(v2.status_code(), StatusCode::CREATED);
    let v2_body: Value = v2.json();
    let v2_id: Uuid = v2_body["id"].as_str().unwrap().parse().unwrap();
    assert!(v2_body["is_current"].as_bool().unwrap());
    assert_eq!(
        v2_body["supersedes_id"].as_str().unwrap(),
        v1_id.to_string()
    );

    // The predecessor is no longer current, but still present in the chain.
    let is_current: bool = sqlx::query_scalar("SELECT is_current FROM document WHERE id = $1")
        .bind(v1_id)
        .fetch_one(&pool)
        .await
        .expect("v1 row");
    assert!(!is_current);

    let chain = server
        .get(&format!("/api/v1/documents/{v2_id}/versions"))
        .add_header("Authorization", &auth)
        .await
        .json::<Vec<Value>>();
    assert_eq!(chain.len(), 2);
    assert_eq!(chain[0]["id"].as_str().unwrap(), v1_id.to_string());
    assert_eq!(chain[1]["id"].as_str().unwrap(), v2_id.to_string());
    assert!(!chain[0]["is_current"].as_bool().unwrap());
    assert!(chain[1]["is_current"].as_bool().unwrap());

    // AC-DOC-007-D: replacement is traced in audit_log with both document
    // references and the justification.
    let (operation, new_values): (String, Value) = sqlx::query_as(
        "SELECT operation, new_values FROM audit_log
         WHERE entity_type = 'document' AND entity_id = $1",
    )
    .bind(v2_id)
    .fetch_one(&pool)
    .await
    .expect("audit_log row");
    assert_eq!(operation, "UPDATE");
    assert_eq!(
        new_values["reason"].as_str().unwrap(),
        "Modification votée en AG du 12/03"
    );
}

#[tokio::test]
async fn replacing_a_founding_document_requires_a_reason() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let v1 = confirm_upload(
        &server,
        &auth,
        f.building_id,
        "edd-v1.pdf",
        "edd",
        None,
        None,
    )
    .await;
    let v1_id: Uuid = v1.json::<Value>()["id"].as_str().unwrap().parse().unwrap();

    let v2 = confirm_upload(
        &server,
        &auth,
        f.building_id,
        "edd-v2.pdf",
        "edd",
        Some(v1_id),
        None,
    )
    .await;
    assert_eq!(v2.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn version_must_match_predecessor_category() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let v1 = confirm_upload(
        &server,
        &auth,
        f.building_id,
        "edd-v1.pdf",
        "edd",
        None,
        None,
    )
    .await;
    let v1_id: Uuid = v1.json::<Value>()["id"].as_str().unwrap().parse().unwrap();

    let mismatched = confirm_upload(
        &server,
        &auth,
        f.building_id,
        "reglement.pdf",
        "reglement_copropriete",
        Some(v1_id),
        Some("reason"),
    )
    .await;
    assert_eq!(mismatched.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn a_founding_document_with_a_successor_cannot_be_deleted() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let v1 = confirm_upload(
        &server,
        &auth,
        f.building_id,
        "edd-v1.pdf",
        "edd",
        None,
        None,
    )
    .await;
    let v1_id: Uuid = v1.json::<Value>()["id"].as_str().unwrap().parse().unwrap();

    let delete = server
        .delete(&format!("/api/v1/documents/{v1_id}"))
        .add_header("Authorization", &auth)
        .json(&json!({}))
        .await;
    assert_eq!(delete.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn a_current_founding_document_cannot_be_hard_deleted_either() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let v1 = confirm_upload(
        &server,
        &auth,
        f.building_id,
        "reglement.pdf",
        "reglement_copropriete",
        None,
        None,
    )
    .await;
    let v1_id: Uuid = v1.json::<Value>()["id"].as_str().unwrap().parse().unwrap();

    let delete = server
        .delete(&format!("/api/v1/documents/{v1_id}"))
        .add_header("Authorization", &auth)
        .json(&json!({}))
        .await;
    assert_eq!(delete.status_code(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn soft_deleted_document_is_excluded_from_claim_listing() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let upload = confirm_upload(
        &server,
        &auth,
        f.building_id,
        "facture.pdf",
        "autre",
        None,
        None,
    )
    .await;
    let doc_id: Uuid = upload.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    let claim_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO claim (id, building_id, title, declared_at, status, created_by)
         VALUES ($1, $2, 'Test claim', CURRENT_DATE, 'open', $3)",
    )
    .bind(claim_id)
    .bind(f.building_id)
    .bind(f.manager_id)
    .execute(&pool)
    .await
    .expect("claim");

    let attach = server
        .post(&format!("/api/v1/claims/{claim_id}/documents"))
        .add_header("Authorization", &auth)
        .json(&json!({ "document_id": doc_id }))
        .await;
    assert_eq!(attach.status_code(), StatusCode::NO_CONTENT);

    let before = server
        .get(&format!("/api/v1/claims/{claim_id}/documents"))
        .add_header("Authorization", &auth)
        .await
        .json::<Vec<Value>>();
    assert_eq!(before.len(), 1);

    sqlx::query("UPDATE document SET deleted_at = now() WHERE id = $1")
        .bind(doc_id)
        .execute(&pool)
        .await
        .expect("soft delete");

    let after = server
        .get(&format!("/api/v1/claims/{claim_id}/documents"))
        .add_header("Authorization", &auth)
        .await
        .json::<Vec<Value>>();
    assert!(after.is_empty());
}

#[tokio::test]
async fn deleting_the_owning_entity_preserves_a_document_with_a_successor() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let v1 = confirm_upload(
        &server,
        &auth,
        f.building_id,
        "devis-v1.pdf",
        "autre",
        None,
        None,
    )
    .await;
    let v1_id: Uuid = v1.json::<Value>()["id"].as_str().unwrap().parse().unwrap();

    let v2 = confirm_upload(
        &server,
        &auth,
        f.building_id,
        "devis-v2.pdf",
        "autre",
        Some(v1_id),
        None,
    )
    .await;
    assert_eq!(v2.status_code(), StatusCode::CREATED);

    let claim_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO claim (id, building_id, title, declared_at, status, created_by)
         VALUES ($1, $2, 'Test claim', CURRENT_DATE, 'open', $3)",
    )
    .bind(claim_id)
    .bind(f.building_id)
    .bind(f.manager_id)
    .execute(&pool)
    .await
    .expect("claim");

    let attach = server
        .post(&format!("/api/v1/claims/{claim_id}/documents"))
        .add_header("Authorization", &auth)
        .json(&json!({ "document_id": v1_id }))
        .await;
    assert_eq!(attach.status_code(), StatusCode::NO_CONTENT);

    let delete = server
        .delete(&format!("/api/v1/claims/{claim_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(delete.status_code(), StatusCode::NO_CONTENT);

    // The claim (and its claim_document link) is gone, but v1 must survive
    // because v2 supersedes it: the version chain must stay intact.
    let deleted_at: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT deleted_at FROM document WHERE id = $1")
            .bind(v1_id)
            .fetch_one(&pool)
            .await
            .expect("v1 row");
    assert!(deleted_at.is_none());
}

#[tokio::test]
async fn non_founding_documents_are_not_subject_to_the_uniqueness_constraint() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let first = confirm_upload(
        &server,
        &auth,
        f.building_id,
        "facture-1.pdf",
        "autre",
        None,
        None,
    )
    .await;
    assert_eq!(first.status_code(), StatusCode::CREATED);
    let second = confirm_upload(
        &server,
        &auth,
        f.building_id,
        "facture-2.pdf",
        "autre",
        None,
        None,
    )
    .await;
    assert_eq!(second.status_code(), StatusCode::CREATED);
}

/// EP-005-S07 (LEGAL §3.4, §6.2, §6.3): technical regulatory documents
/// (DTG / DPE collectif → `diagnostic`, PV → `pv_ag`) are accepted as
/// non-founding categories — not subject to the founding uniqueness
/// constraint nor the replacement-reason workflow.
#[tokio::test]
async fn technical_regulatory_categories_are_accepted_and_not_founding() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let dtg = confirm_upload(
        &server,
        &auth,
        f.building_id,
        "dtg-2026.pdf",
        "diagnostic",
        None,
        None,
    )
    .await;
    assert_eq!(dtg.status_code(), StatusCode::CREATED);
    let dtg_body: Value = dtg.json();
    assert_eq!(dtg_body["category"].as_str().unwrap(), "diagnostic");

    // A second `diagnostic` document (e.g. DPE collectif) for the same
    // building must not collide with the founding uniqueness index, which
    // is scoped to `reglement_copropriete`/`edd` only.
    let dpe = confirm_upload(
        &server,
        &auth,
        f.building_id,
        "dpe-collectif.pdf",
        "diagnostic",
        None,
        None,
    )
    .await;
    assert_eq!(dpe.status_code(), StatusCode::CREATED);

    let pv = confirm_upload(
        &server,
        &auth,
        f.building_id,
        "pv-ag-2026.pdf",
        "pv_ag",
        None,
        None,
    )
    .await;
    assert_eq!(pv.status_code(), StatusCode::CREATED);
    let pv_body: Value = pv.json();
    assert_eq!(pv_body["category"].as_str().unwrap(), "pv_ag");
}

/// US-DOC-004: DELETE /api/v1/documents/:id without a JSON body (no
/// Content-Type header) must not be rejected with 415 — the reason is optional.
#[tokio::test]
async fn deleting_a_document_without_a_json_body_is_not_rejected() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let upload = confirm_upload(
        &server,
        &auth,
        f.building_id,
        "facture-a-supprimer.pdf",
        "autre",
        None,
        None,
    )
    .await;
    assert_eq!(upload.status_code(), StatusCode::CREATED);
    let doc_id: Uuid = upload.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    let delete = server
        .delete(&format!("/api/v1/documents/{doc_id}"))
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(delete.status_code(), StatusCode::NO_CONTENT);

    let still = server
        .get("/api/v1/documents")
        .add_header("Authorization", &auth)
        .await;
    let docs = still.json::<Vec<Value>>();
    assert!(
        docs.iter()
            .all(|d| d["id"].as_str().unwrap() != doc_id.to_string())
    );
}

/// US-DOC-004: the shared frontend axios client sets
/// `Content-Type: application/json` on every request, including DELETEs with
/// no body. That exact combination must not be rejected with 415 — an empty
/// body simply means "no deletion reason".
#[tokio::test]
async fn deleting_a_document_with_json_content_type_and_empty_body_is_not_rejected() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let upload = confirm_upload(
        &server,
        &auth,
        f.building_id,
        "facture-ct-vide.pdf",
        "autre",
        None,
        None,
    )
    .await;
    assert_eq!(upload.status_code(), StatusCode::CREATED);
    let doc_id: Uuid = upload.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    let delete = server
        .delete(&format!("/api/v1/documents/{doc_id}"))
        .add_header("Authorization", &auth)
        .add_header("Content-Type", "application/json")
        .await;
    assert_eq!(delete.status_code(), StatusCode::NO_CONTENT);

    let row: Option<(Option<chrono::DateTime<chrono::Utc>>, Option<String>)> =
        sqlx::query_as("SELECT deleted_at, deletion_reason FROM document WHERE id = $1")
            .bind(doc_id)
            .fetch_optional(&pool)
            .await
            .expect("query");
    let (deleted_at, reason) = row.expect("row kept (soft delete)");
    assert!(deleted_at.is_some());
    assert_eq!(reason, None);
}

/// US-DOC-004: a soft-deleted document is excluded from the listing and the
/// S3 object row is preserved (soft delete, not hard delete).
#[tokio::test]
async fn soft_deleting_a_document_with_a_json_body_records_the_reason() {
    let (cfg, pool) = common::test_db().await;
    let f = setup(&pool).await;
    let token = token_for(f.manager_id, f.building_id, &cfg);
    let server = common::test_server_with_pool(cfg, pool.clone()).await;
    let auth = format!("Bearer {token}");

    let upload = confirm_upload(
        &server,
        &auth,
        f.building_id,
        "facture-raison.pdf",
        "autre",
        None,
        None,
    )
    .await;
    assert_eq!(upload.status_code(), StatusCode::CREATED);
    let doc_id: Uuid = upload.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    let delete = server
        .delete(&format!("/api/v1/documents/{doc_id}"))
        .add_header("Authorization", &auth)
        .json(&json!({ "reason": "obsolète" }))
        .await;
    assert_eq!(delete.status_code(), StatusCode::NO_CONTENT);

    let row: Option<(Option<chrono::DateTime<chrono::Utc>>, Option<String>)> =
        sqlx::query_as("SELECT deleted_at, deletion_reason FROM document WHERE id = $1")
            .bind(doc_id)
            .fetch_optional(&pool)
            .await
            .expect("query");
    let (deleted_at, reason) = row.expect("row kept (soft delete)");
    assert!(deleted_at.is_some());
    assert_eq!(reason.as_deref(), Some("obsolète"));
}
