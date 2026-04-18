// Tests d'intégration La Ruche
// Requiert une base PostgreSQL de test (DATABASE_URL=...laruche_test)
// Exécution : cargo test --test integration_test

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use axum_test::TestServer;
use serde_json::{json, Value};

// ── Helpers ───────────────────────────────────────────────────────

/// Crée un serveur de test avec la stack complète
async fn test_server() -> TestServer {
    dotenvy::dotenv().ok();
    let cfg = laruche::config::AppConfig::load().expect("config");
    let pool = sqlx::PgPool::connect(&cfg.db.url).await.expect("db");
    sqlx::migrate!("./migrations").run(&pool).await.expect("migrations");

    let storage = laruche::storage::client::StorageClient::new(&cfg.s3)
        .await
        .expect("storage");

    let state = std::sync::Arc::new(laruche::db::AppState { pool, storage, cfg });
    let webauthn = laruche::auth::passkey::build_webauthn(&state.cfg.app).expect("webauthn");
    let app = laruche::api::router(state, webauthn);

    TestServer::new(app).expect("test server")
}

// ── Onboarding ────────────────────────────────────────────────────

/// Payload minimal valide pour le bootstrap
fn bootstrap_payload() -> serde_json::Value {
    json!({
        "email": "syndic@test.fr",
        "last_name": "Martin",
        "first_name": "Sophie",
        "password": "MotDePasse123!",
        "building_name": "Résidence Les Tests",
        "address_line1": "1 rue des Tests",
        "postal_code": "75001",
        "city": "Paris",
        "shares_total": "1000"
    })
}

#[tokio::test]
async fn test_onboarding_status_renvoie_needs_onboarding() {
    let server = test_server().await;
    // Note: la base de test peut déjà avoir des données ; ce test vérifie
    // seulement que la route répond avec le bon format JSON.
    let resp = server.get("/api/v1/onboarding/status").await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let body: Value = resp.json();
    assert!(body.get("needs_onboarding").is_some(), "Le champ needs_onboarding doit être présent");
}

#[tokio::test]
async fn test_onboarding_bootstrap_input_invalide() {
    let server = test_server().await;
    // Email invalide + mot de passe trop court
    let resp = server.post("/api/v1/onboarding/bootstrap")
        .json(&json!({
            "email": "pas-un-email",
            "last_name": "X",
            "first_name": "Y",
            "password": "court",
            "building_name": "Test",
            "address_line1": "1 rue",
            "postal_code": "75001",
            "city": "Paris",
            "shares_total": "1000"
        }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
    let body: Value = resp.json();
    assert_eq!(body["error"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_onboarding_bootstrap_tantièmes_invalides() {
    let server = test_server().await;
    let mut payload = bootstrap_payload();
    payload["shares_total"] = json!("pas-un-nombre");
    let resp = server.post("/api/v1/onboarding/bootstrap")
        .json(&payload)
        .await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
}



#[tokio::test]
async fn test_login_mauvais_mot_de_passe() {
    let server = test_server().await;
    let resp = server.post("/api/v1/auth/login")
        .json(&json!({ "email": "inconnu@test.fr", "password": "mauvais" }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_input_invalide() {
    let server = test_server().await;
    let resp = server.post("/api/v1/auth/login")
        .json(&json!({ "email": "pas-un-email", "password": "x" }))
        .await;
    assert_eq!(resp.status_code(), StatusCode::BAD_REQUEST);
    let body: Value = resp.json();
    assert_eq!(body["error"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_refresh_sans_cookie() {
    let server = test_server().await;
    let resp = server.post("/api/v1/auth/refresh").await;
    assert_eq!(resp.status_code(), StatusCode::UNAUTHORIZED);
}

// ── Routes protégées sans auth ────────────────────────────────────

#[tokio::test]
async fn test_route_protegee_sans_jwt_retourne_401() {
    let server = test_server().await;
    let routes = [
        "/api/v1/copropriete",
        "/api/v1/lots",
        "/api/v1/ag",
        "/api/v1/budgets",
        "/api/v1/documents",
        "/api/v1/contrats",
        "/api/v1/taches",
        "/api/v1/signalements",
        "/api/v1/consultations",
    ];
    for route in routes {
        let resp = server.get(route).await;
        assert!(
            resp.status_code() == StatusCode::UNAUTHORIZED,
            "Route {} devrait retourner 401, reçu {}",
            route, resp.status_code()
        );
    }
}

// ── Vendor flow (public route) ───────────────────────────────────

#[tokio::test]
async fn test_vendor_token_not_found_returns_404() {
    let server = test_server().await;
    let fake_token = uuid::Uuid::new_v4();
    let resp = server.get(&format!("/vendor/{}", fake_token)).await;
    assert_eq!(resp.status_code(), StatusCode::NOT_FOUND);
}

// ── NOTICE AGPL ───────────────────────────────────────────────────

#[tokio::test]
async fn test_notice_agpl_accessible() {
    let server = test_server().await;
    let resp = server.get("/notice").await;
    assert_eq!(resp.status_code(), StatusCode::OK);
    let body = resp.text();
    assert!(body.contains("AGPL"), "Le NOTICE doit mentionner la licence AGPL");
}

// ── Domain : votes AG ─────────────────────────────────────────────

#[test]
fn test_vote_art24_adopte() {
    use laruche::domain::vote::{Depouillement, TypeVote};
    use rust_decimal::Decimal;

    let d = Depouillement {
        tantièmes_total: Decimal::new(1000, 0),
        tantièmes_presents: Decimal::new(600, 0),
        nb_coproprietaires_total: 10,
        nb_coproprietaires_presents: 7,
        pour: Decimal::new(400, 0),
        contre: Decimal::new(200, 0),
        abstention: Decimal::ZERO,
    };
    let r = d.calculer(&TypeVote::Article24);
    assert!(r.adopte, "400 > 300 (seuil 50% des présents) → adopté");
}

#[test]
fn test_vote_art25_rejete_si_insuffisant() {
    use laruche::domain::vote::{Depouillement, TypeVote};
    use rust_decimal::Decimal;

    let d = Depouillement {
        tantièmes_total: Decimal::new(1000, 0),
        tantièmes_presents: Decimal::new(800, 0),
        nb_coproprietaires_total: 10,
        nb_coproprietaires_presents: 8,
        pour: Decimal::new(490, 0), // < 500 = majorité absolue
        contre: Decimal::new(310, 0),
        abstention: Decimal::ZERO,
    };
    let r = d.calculer(&TypeVote::Article25);
    assert!(!r.adopte, "490 < 500 → rejeté art. 25");
}

#[test]
fn test_vote_art26_necessite_deux_tiers() {
    use laruche::domain::vote::{Depouillement, TypeVote};
    use rust_decimal::Decimal;

    let d = Depouillement {
        tantièmes_total: Decimal::new(1000, 0),
        tantièmes_presents: Decimal::new(900, 0),
        nb_coproprietaires_total: 10,
        nb_coproprietaires_presents: 9,
        pour: Decimal::new(650, 0), // < 667 = 2/3 de 1000
        contre: Decimal::new(250, 0),
        abstention: Decimal::ZERO,
    };
    let r = d.calculer(&TypeVote::Article26);
    assert!(!r.adopte, "650 < 667 (2/3) → rejeté art. 26");
}

// ── Domain : charges ──────────────────────────────────────────────

#[test]
fn test_appels_somme_exacte() {
    use laruche::domain::charges::{calculer_appels, LotAvecTantièmes};
    use rust_decimal::Decimal;

    let lots = vec![
        LotAvecTantièmes { lot_id: uuid::Uuid::new_v4(), tantièmes: Decimal::new(333, 0) },
        LotAvecTantièmes { lot_id: uuid::Uuid::new_v4(), tantièmes: Decimal::new(333, 0) },
        LotAvecTantièmes { lot_id: uuid::Uuid::new_v4(), tantièmes: Decimal::new(334, 0) },
    ];
    let montant = Decimal::new(120000, 2); // 1200.00€
    let total = Decimal::new(1000, 0);

    let appels = calculer_appels(montant, &lots, total);
    let somme: Decimal = appels.iter().map(|a| a.montant).sum();
    assert_eq!(somme, montant, "La somme des appels doit être exactement le montant total");
}

#[test]
fn test_appels_lots_vides() {
    use laruche::domain::charges::{calculer_appels, LotAvecTantièmes};
    use rust_decimal::Decimal;

    let appels = calculer_appels(Decimal::new(1000, 0), &[], Decimal::new(1000, 0));
    assert!(appels.is_empty());
}

// ── Domain : rappels contrats ─────────────────────────────────────

#[test]
fn test_rappels_contrat_annuel_reconduction() {
    use laruche::domain::rappels::{calculer_rappels, ContratPourRappels, TypeRappel};
    use chrono::NaiveDate;

    let aujourd_hui = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let contrat = ContratPourRappels {
        date_debut: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        date_fin: Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
        reconduction_tacite: true,
        preavis_jours: Some(90),
        periodicite_mois: Some(12),
    };

    let rappels = calculer_rappels(&contrat, aujourd_hui);
    assert!(rappels.iter().any(|r| r.type_rappel == TypeRappel::Reconduction), "Doit avoir un rappel reconduction");
    assert!(rappels.iter().any(|r| r.type_rappel == TypeRappel::Expiration), "Doit avoir un rappel expiration");
}

#[test]
fn test_rappels_dates_coherentes() {
    use laruche::domain::rappels::{calculer_rappels, ContratPourRappels};
    use chrono::NaiveDate;

    let aujourd_hui = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let contrat = ContratPourRappels {
        date_debut: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        date_fin: Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
        reconduction_tacite: true,
        preavis_jours: Some(30),
        periodicite_mois: Some(3),
    };

    let rappels = calculer_rappels(&contrat, aujourd_hui);
    // Tous les rappels doivent être dans le futur
    for r in &rappels {
        assert!(r.date_rappel >= aujourd_hui, "Rappel {:?} ne doit pas être dans le passé", r.date_rappel);
    }
}
