// Integration tests — in-app PDF viewing: the download endpoint gains an
// `inline` disposition variant, restricted to PDFs and to roles allowed by
// `visible_to_roles` (same visibility verdict as the document list).

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{Value, json};
use uuid::Uuid;

async fn seed_building_and_users() -> (sqlx::PgPool, Uuid, String, String) {
    let (cfg, pool) = common::test_db().await;

    let building_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total) \
         VALUES ($1, 'InlineViewBuilding', '1 rue Test', '75001', 'Paris', 1000)",
    )
    .bind(building_id)
    .execute(&pool)
    .await
    .expect("building");

    let manager_id = insert_user(&pool, &cfg, building_id, laruche::domain::Role::Manager).await;
    let tenant_id = insert_user(&pool, &cfg, building_id, laruche::domain::Role::Tenant).await;

    let jwt_secret = cfg.auth.jwt_secret.clone();
    let ttl = cfg.auth.access_token_ttl_secs;
    let jwt = |user_id: Uuid, role: laruche::domain::Role| {
        laruche::auth::jwt::generate_access_token(
            user_id,
            building_id,
            vec![role],
            &jwt_secret,
            ttl,
        )
        .expect("token")
    };

    (
        pool,
        building_id,
        jwt(manager_id, laruche::domain::Role::Manager),
        jwt(tenant_id, laruche::domain::Role::Tenant),
    )
}

async fn insert_user(
    pool: &sqlx::PgPool,
    cfg: &laruche::config::AppConfig,
    building_id: Uuid,
    role: laruche::domain::Role,
) -> Uuid {
    common::rbac_insert_user(pool, cfg, building_id, role).await
}

/// Runs the upload flow (presign -> confirm) and returns the document id.
async fn upload_document(
    server: &TestServer,
    jwt: &str,
    name: &str,
    mime_type: &str,
    visible_roles: &[&str],
) -> String {
    let ext = name.rsplit('.').next().unwrap_or("bin");
    let presign = server
        .post("/api/v1/documents/upload-url")
        .add_header("Authorization", format!("Bearer {jwt}"))
        .json(&json!({"extension": ext, "content_type": mime_type, "size_bytes": 128}))
        .await;
    assert_eq!(presign.status_code(), StatusCode::OK, "{}", presign.text());
    let presign_body: Value = presign.json();

    let confirm = server
        .post("/api/v1/documents")
        .add_header("Authorization", format!("Bearer {jwt}"))
        .json(&json!({
            "key": presign_body["key"],
            "bucket": presign_body["bucket"],
            "original_name": name,
            "mime_type": mime_type,
            "size_bytes": 128,
            "visible_roles": visible_roles,
            "category": "autre"
        }))
        .await;
    assert_eq!(
        confirm.status_code(),
        StatusCode::CREATED,
        "{}",
        confirm.text()
    );
    confirm.json::<Value>()["id"].as_str().unwrap().to_string()
}

fn disposition_of(url: &str) -> String {
    let query = url.split('?').nth(1).unwrap_or("");
    for part in query.split('&') {
        if let Some(value) = part.strip_prefix("response-content-disposition=") {
            return urlencoding_decode(value);
        }
    }
    panic!("no response-content-disposition in presigned url: {url}");
}

/// Minimal percent-decoding for the query parameter (presigned URLs are
/// percent-encoded by the AWS SDK).
fn urlencoding_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap();
            out.push(u8::from_str_radix(hex, 16).unwrap());
            i += 3;
        } else if bytes[i] == b'+' {
            out.push(b' ');
            i += 1;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).unwrap()
}

#[tokio::test]
async fn pdf_inline_disposition_for_authorized_role() {
    let (_pool, _building_id, manager_jwt, _tenant_jwt) = seed_building_and_users().await;
    let server = common::test_server().await;

    let doc_id = upload_document(
        &server,
        &manager_jwt,
        "PV AG 2026 été.pdf",
        "application/pdf",
        &["manager", "co_owner"],
    )
    .await;

    let resp = server
        .get(&format!(
            "/api/v1/documents/{doc_id}/download-url?disposition=inline"
        ))
        .add_header("Authorization", format!("Bearer {manager_jwt}"))
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK, "{}", resp.text());
    let url = resp.json::<Value>()["url"].as_str().unwrap().to_string();

    let disposition = disposition_of(&url);
    assert!(
        disposition.starts_with("inline; filename=\""),
        "expected inline disposition, got: {disposition}"
    );
    assert!(
        disposition.contains("filename*=UTF-8''PV%20AG%202026%20%C3%A9t%C3%A9.pdf"),
        "expected RFC 5987-encoded filename, got: {disposition}"
    );
}

#[tokio::test]
async fn non_pdf_falls_back_to_attachment_even_with_inline() {
    let (_pool, _building_id, manager_jwt, _tenant_jwt) = seed_building_and_users().await;
    let server = common::test_server().await;

    let doc_id = upload_document(
        &server,
        &manager_jwt,
        "budget.xlsx",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        &["manager"],
    )
    .await;

    let resp = server
        .get(&format!(
            "/api/v1/documents/{doc_id}/download-url?disposition=inline"
        ))
        .add_header("Authorization", format!("Bearer {manager_jwt}"))
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK, "{}", resp.text());
    let url = resp.json::<Value>()["url"].as_str().unwrap().to_string();
    let disposition = disposition_of(&url);
    assert!(
        disposition.starts_with("attachment;"),
        "non-PDF must stay attachment, got: {disposition}"
    );
}

#[tokio::test]
async fn default_disposition_stays_attachment_for_pdf() {
    let (_pool, _building_id, manager_jwt, _tenant_jwt) = seed_building_and_users().await;
    let server = common::test_server().await;

    let doc_id = upload_document(
        &server,
        &manager_jwt,
        "pv.pdf",
        "application/pdf",
        &["manager"],
    )
    .await;

    let resp = server
        .get(&format!("/api/v1/documents/{doc_id}/download-url"))
        .add_header("Authorization", format!("Bearer {manager_jwt}"))
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK, "{}", resp.text());
    let url = resp.json::<Value>()["url"].as_str().unwrap().to_string();
    assert!(disposition_of(&url).starts_with("attachment;"));
}

#[tokio::test]
async fn role_without_visibility_gets_404_for_inline_and_download() {
    let (_pool, _building_id, manager_jwt, tenant_jwt) = seed_building_and_users().await;
    let server = common::test_server().await;

    // Visible to managers only — the tenant must be refused on both variants.
    let doc_id = upload_document(
        &server,
        &manager_jwt,
        "contrat.pdf",
        "application/pdf",
        &["manager"],
    )
    .await;

    for suffix in ["", "?disposition=inline"] {
        let resp = server
            .get(&format!("/api/v1/documents/{doc_id}/download-url{suffix}"))
            .add_header("Authorization", format!("Bearer {tenant_jwt}"))
            .await;
        assert_eq!(
            resp.status_code(),
            StatusCode::NOT_FOUND,
            "tenant must get the same refusal for download and inline ({suffix})"
        );
    }

    // The manager (allowed role) still gets both.
    for suffix in ["", "?disposition=inline"] {
        let resp = server
            .get(&format!("/api/v1/documents/{doc_id}/download-url{suffix}"))
            .add_header("Authorization", format!("Bearer {manager_jwt}"))
            .await;
        assert_eq!(resp.status_code(), StatusCode::OK, "{}", resp.text());
    }
}

#[tokio::test]
async fn unknown_document_returns_404() {
    let (_pool, _building_id, manager_jwt, _tenant_jwt) = seed_building_and_users().await;
    let server = common::test_server().await;

    let resp = server
        .get(&format!(
            "/api/v1/documents/{}/download-url?disposition=inline",
            Uuid::new_v4()
        ))
        .add_header("Authorization", format!("Bearer {manager_jwt}"))
        .await;
    assert_eq!(resp.status_code(), StatusCode::NOT_FOUND);
}
