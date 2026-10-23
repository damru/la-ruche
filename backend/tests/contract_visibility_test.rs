// Integration tests — EP-006-S07: document link visibility on contracts
// (Syndic + Conseil syndical read, Syndic-only write, manager-only reminder
// acknowledgment per AC-CF-002-E).

mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::{Value, json};
use uuid::Uuid;

async fn create_contract(server: &TestServer, jwt: &str, payload: &Value) -> String {
    let resp = server
        .post("/api/v1/contracts")
        .add_header("Authorization", format!("Bearer {jwt}"))
        .json(payload)
        .await;
    assert_eq!(resp.status_code(), StatusCode::CREATED, "{}", resp.text());
    resp.json::<Value>()["id"].as_str().unwrap().to_string()
}

struct Ctx {
    manager_jwt: String,
    board_member_jwt: String,
    contract_a: String,
    contract_b: String,
    document_id: String,
    reminder_id: String,
}

fn contract_payload(vendor_name: &str, contract_type: &str, subject: &str) -> Value {
    json!({
        "vendor_name": vendor_name,
        "subject": subject,
        "contract_type": contract_type,
        "start_date": "2026-01-01",
        "end_date": "2026-12-31",
        "amount": "1200.00",
        "payment_frequency": "monthly",
        "auto_renewal": true,
        "notice_days": 90
    })
}

async fn setup() -> Ctx {
    let (cfg, pool) = common::test_db().await;

    let building_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO building (id, name, address_line1, postal_code, city, shares_total) \
         VALUES ($1, 'FilterTestBuilding', '1 rue Filtre', '75001', 'Paris', 1000)",
    )
    .bind(building_id)
    .execute(&pool)
    .await
    .expect("building");

    let manager_id = insert_user(&pool, &cfg, building_id, "manager").await;
    let board_member_id = insert_user(&pool, &cfg, building_id, "board_member").await;

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

    let manager_jwt = jwt(manager_id, laruche::domain::Role::Manager);
    let board_member_jwt = jwt(board_member_id, laruche::domain::Role::BoardMember);

    let server = common::test_server().await;

    // Contract A: ACME / maintenance, activated (status active).
    let contract_a = create_contract(
        &server,
        &manager_jwt,
        &contract_payload("ACME Nettoyage", "maintenance", "Entretien"),
    )
    .await;
    let resp = server
        .post(&format!("/api/v1/contracts/{contract_a}/activate"))
        .add_header("Authorization", format!("Bearer {manager_jwt}"))
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK, "{}", resp.text());

    // Contract B: Beta / cleaning, left in draft.
    let contract_b = create_contract(
        &server,
        &manager_jwt,
        &contract_payload("Beta Jardins", "cleaning", "Espaces verts"),
    )
    .await;

    // A document attached to contract A (upload flow: presign -> confirm -> attach).
    let presign = server
        .post("/api/v1/documents/upload-url")
        .add_header("Authorization", format!("Bearer {manager_jwt}"))
        .json(&json!({"extension": "pdf", "content_type": "application/pdf", "size_bytes": 128}))
        .await;
    assert_eq!(presign.status_code(), StatusCode::OK, "{}", presign.text());
    let presign_body: Value = presign.json();

    let confirm = server
        .post("/api/v1/documents")
        .add_header("Authorization", format!("Bearer {manager_jwt}"))
        .json(&json!({
            "key": presign_body["key"],
            "bucket": presign_body["bucket"],
            "original_name": "contrat-acme.pdf",
            "mime_type": "application/pdf",
            "size_bytes": 128,
            "visible_roles": ["manager", "board_member"],
            "category": "contrat"
        }))
        .await;
    assert_eq!(
        confirm.status_code(),
        StatusCode::CREATED,
        "{}",
        confirm.text()
    );
    let document_id = confirm.json::<Value>()["id"].as_str().unwrap().to_string();

    let attach = server
        .post(&format!("/api/v1/contracts/{contract_a}/documents"))
        .add_header("Authorization", format!("Bearer {manager_jwt}"))
        .json(&json!({"document_id": document_id}))
        .await;
    assert_eq!(
        attach.status_code(),
        StatusCode::NO_CONTENT,
        "{}",
        attach.text()
    );

    // A pending reminder on contract A (acknowledge is manager-only per AC-CF-002-E).
    let reminder_id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO contract_reminder (id, contract_id, reminder_type, reminder_date, status) \
         VALUES ($1, $2, 'renewal', '2026-06-01', 'pending')",
    )
    .bind(Uuid::parse_str(&reminder_id).expect("uuid"))
    .bind(Uuid::parse_str(&contract_a).expect("uuid"))
    .execute(&pool)
    .await
    .expect("reminder");

    Ctx {
        manager_jwt,
        board_member_jwt,
        contract_a,
        contract_b,
        document_id,
        reminder_id,
    }
}

async fn insert_user(
    pool: &sqlx::PgPool,
    cfg: &laruche::config::AppConfig,
    building_id: Uuid,
    role: &str,
) -> Uuid {
    let user_id = Uuid::new_v4();
    let email = format!("filter-{role}-{}@test.local", Uuid::new_v4());
    let password_hash =
        laruche::auth::password::hash_password("MotDePasse123!", &cfg.auth).expect("hash");
    sqlx::query(
        "INSERT INTO user_account (id, email, last_name, first_name, password_hash) \
         VALUES ($1, $2, 'Filtre', 'User', $3)",
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

// EP-006-S07 (option A): the board member can READ linked documents but
// cannot attach/detach them — writes stay manager-only.
#[tokio::test]
async fn board_member_reads_linked_documents_but_cannot_write() {
    let ctx = setup().await;
    let server = common::test_server().await;

    // Read as board member -> 200 with the attached document.
    let resp = server
        .get(&format!("/api/v1/contracts/{}/documents", ctx.contract_a))
        .add_header("Authorization", format!("Bearer {}", ctx.board_member_jwt))
        .await;
    assert_eq!(resp.status_code(), StatusCode::OK, "{}", resp.text());
    let docs = resp.json::<Value>();
    let docs = docs.as_array().expect("array");
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0]["id"], json!(ctx.document_id));

    // Attach as board member -> 403.
    let resp = server
        .post(&format!("/api/v1/contracts/{}/documents", ctx.contract_b))
        .add_header("Authorization", format!("Bearer {}", ctx.board_member_jwt))
        .json(&json!({"document_id": ctx.document_id}))
        .await;
    assert_eq!(resp.status_code(), StatusCode::FORBIDDEN);

    // Detach as board member -> 403.
    let resp = server
        .delete(&format!(
            "/api/v1/contracts/{}/documents/{}",
            ctx.contract_a, ctx.document_id
        ))
        .add_header("Authorization", format!("Bearer {}", ctx.board_member_jwt))
        .await;
    assert_eq!(resp.status_code(), StatusCode::FORBIDDEN);

    // Acknowledge a reminder as board member -> 403 (manager-only, AC-CF-002-E).
    let resp = server
        .put(&format!(
            "/api/v1/contracts/{}/reminders/{}/acknowledge",
            ctx.contract_a, ctx.reminder_id
        ))
        .add_header("Authorization", format!("Bearer {}", ctx.board_member_jwt))
        .await;
    assert_eq!(resp.status_code(), StatusCode::FORBIDDEN);

    // Acknowledge as manager -> 204.
    let resp = server
        .put(&format!(
            "/api/v1/contracts/{}/reminders/{}/acknowledge",
            ctx.contract_a, ctx.reminder_id
        ))
        .add_header("Authorization", format!("Bearer {}", ctx.manager_jwt))
        .await;
    assert_eq!(
        resp.status_code(),
        StatusCode::NO_CONTENT,
        "{}",
        resp.text()
    );
}
