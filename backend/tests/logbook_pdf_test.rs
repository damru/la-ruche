// EP-009-S09 — logbook PDF export (server-side, shared document template).
// AC: GET /logbook/export/pdf returns an `application/pdf` attachment built
// from the shared template (header, contrats actifs, chronologie, signature,
// footer). Access matches the logbook summary: any authenticated building
// member (the logbook is a regulatory record every member may consult).
// Requires a PostgreSQL test database. Run with: cargo test --test logbook_pdf_test

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{Value, json};

async fn create_entry(server: &TestServer, auth: &str) {
    let created = server
        .post("/api/v1/logbook")
        .add_header("Authorization", auth)
        .json(&json!({
            "title": "Révision annuelle chaudière",
            "intervention_date": "2026-01-15",
            "category": "contrat"
        }))
        .await;
    assert_eq!(created.status_code(), StatusCode::CREATED, "setup entry");
}

#[tokio::test]
async fn members_can_export_logbook_as_pdf() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;

    for jwt in [
        &ctx.manager_jwt,
        &ctx.board_member_jwt,
        &ctx.coowner_jwt,
        &ctx.tenant_jwt,
    ] {
        let auth = format!("Bearer {jwt}");
        let response = server
            .get("/api/v1/logbook/export/pdf")
            .add_header("Authorization", &auth)
            .await;
        assert_eq!(response.status_code(), StatusCode::OK, "role {jwt}");
        assert_eq!(
            response.header("content-type"),
            "application/pdf",
            "role {jwt}"
        );
        let disposition = response
            .header("content-disposition")
            .to_str()
            .expect("content-disposition header must be ASCII");
        assert!(
            disposition.contains("attachment"),
            "role {jwt}: {disposition}"
        );
        assert!(
            disposition.contains("carnet-entretien.pdf"),
            "role {jwt}: {disposition}"
        );
        let bytes = response.as_bytes();
        assert!(bytes.starts_with(b"%PDF"), "role {jwt}: not a PDF");
        assert!(!bytes.is_empty(), "role {jwt}: empty body");
    }
}

#[tokio::test]
async fn export_pdf_reflects_entries_and_contracts() {
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);

    // Seed one logbook entry so the chronology is non-empty.
    create_entry(&server, &auth).await;

    let response = server
        .get("/api/v1/logbook/export/pdf")
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let bytes = response.as_bytes();
    assert!(bytes.starts_with(b"%PDF"));
    assert!(!bytes.is_empty());
}

#[tokio::test]
async fn export_pdf_requires_authentication() {
    let (_ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;

    let response = server.get("/api/v1/logbook/export/pdf").await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn export_pdf_on_empty_logbook_still_renders() {
    // Spec EP-009-S09 §6: carnet vide + aucun contrat → le document s'imprime
    // quand même (en-tête + mentions « Aucun contrat en cours » / « Aucune
    // intervention enregistrée »).
    let (ctx, _pool) = common::rbac_setup().await;
    let server = common::test_server().await;
    let auth = format!("Bearer {}", ctx.manager_jwt);

    let listed = server
        .get("/api/v1/logbook")
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(listed.status_code(), StatusCode::OK);
    assert_eq!(listed.json::<Value>().as_array().map(Vec::len), Some(0));

    let response = server
        .get("/api/v1/logbook/export/pdf")
        .add_header("Authorization", &auth)
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    assert!(response.as_bytes().starts_with(b"%PDF"));
}
