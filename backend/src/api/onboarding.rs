use std::{str::FromStr, sync::Arc};

use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use utoipa::ToSchema;
use validator::Validate;

use crate::{
    auth::{jwt, password, token},
    db::{self, AppState},
    errors::{AppError, AppResult},
};

// ── DTOs ──────────────────────────────────────────────────────────

#[derive(Deserialize, Validate, ToSchema)]
pub struct BootstrapInput {
    // Manager
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 100))]
    pub last_name: String,
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,
    #[validate(length(min = 8))]
    pub password: String,

    // Building
    #[validate(length(min = 1, max = 200))]
    pub building_name: String,
    #[validate(length(min = 1, max = 200))]
    pub address_line1: String,
    #[validate(length(min = 1, max = 10))]
    pub postal_code: String,
    #[validate(length(min = 1, max = 100))]
    pub city: String,
    /// Total shares — transmitted as a decimal string (e.g. "1000")
    pub shares_total: String,
}

#[derive(Serialize, ToSchema)]
pub struct OnboardingStatusResponse {
    pub needs_onboarding: bool,
}

#[derive(Serialize, ToSchema)]
pub struct BootstrapResponse {
    pub user_id: String,
    pub building_id: String,
    pub roles: Vec<String>,
    pub last_name: String,
    pub first_name: String,
}

// ── Helpers ───────────────────────────────────────────────────────

fn set_cookie(name: &str, value: &str, max_age_secs: u64, secure: bool) -> String {
    let secure_flag = if secure { "; Secure" } else { "" };
    format!(
        "{}={}; HttpOnly{}; SameSite=Strict; Path=/; Max-Age={}",
        name, value, secure_flag, max_age_secs
    )
}

// ── Handlers ──────────────────────────────────────────────────────

/// GET /onboarding/status
///
/// Renvoie si l'instance nécessite un onboarding (aucun utilisateur en base).
/// Route publique — aucune authentification requise.
pub async fn status(
    State(state): State<Arc<AppState>>,
) -> AppResult<Response> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_account")
        .fetch_one(&state.pool)
        .await?;

    let body = Json(OnboardingStatusResponse { needs_onboarding: count == 0 });
    let response = (
        [(header::CACHE_CONTROL, "no-store, no-cache, must-revalidate")],
        body,
    )
        .into_response();
    Ok(response)
}

/// POST /onboarding/bootstrap
///
/// Initialise l'instance au premier lancement : crée la copropriété et le premier
/// compte manager dans une transaction atomique, puis retourne une session
/// authentifiée (cookies access_token + refresh_token).
///
/// Retourne 409 si l'instance est déjà initialisée (au moins un utilisateur existe).
pub async fn bootstrap(
    State(state): State<Arc<AppState>>,
    Json(input): Json<BootstrapInput>,
) -> AppResult<Response> {
    input.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // Validation métier : tantièmes > 0
    let shares_total = Decimal::from_str(input.shares_total.trim())
        .map_err(|_| AppError::Validation("Tantièmes totaux invalides".into()))?;
    if shares_total <= Decimal::ZERO {
        return Err(AppError::Validation("Les tantièmes totaux doivent être supérieurs à 0".into()));
    }

    let password_hash = password::hash_password(&input.password, &state.cfg.auth)?;

    // Transaction atomique — guard first-run inclus
    let mut tx = state.pool.begin().await?;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_account")
        .fetch_one(&mut *tx)
        .await?;

    if count > 0 {
        return Err(AppError::Conflict("Instance déjà initialisée".into()));
    }

    // Création de la copropriété
    let building_row = sqlx::query(
        "INSERT INTO building (name, address_line1, postal_code, city, shares_total) \
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(&input.building_name)
    .bind(&input.address_line1)
    .bind(&input.postal_code)
    .bind(&input.city)
    .bind(shares_total)
    .fetch_one(&mut *tx)
    .await?;
    let building_id: uuid::Uuid = building_row.get("id");

    // Création du compte manager
    let user_row = sqlx::query(
        "INSERT INTO user_account (email, last_name, first_name, password_hash) \
         VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(&input.email)
    .bind(&input.last_name)
    .bind(&input.first_name)
    .bind(&password_hash)
    .fetch_one(&mut *tx)
    .await?;
    let user_id: uuid::Uuid = user_row.get("id");

    // Liaison utilisateur ↔ copropriété avec rôle manager
    sqlx::query(
        "INSERT INTO user_building (user_id, building_id, roles) \
         VALUES ($1, $2, ARRAY['manager']::user_role[])",
    )
    .bind(user_id)
    .bind(building_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    tracing::info!(
        user_id = %user_id,
        building_id = %building_id,
        email = %input.email,
        "Onboarding first-run : instance initialisée"
    );

    // Génération des tokens d'authentification
    let access_token = jwt::generate_access_token(
        user_id,
        building_id,
        vec![crate::domain::Role::Manager],
        &state.cfg.auth.jwt_secret,
        state.cfg.auth.access_token_ttl_secs,
    )?;

    let (refresh_plain, refresh_hash, expire_at) = token::generate_token(
        chrono::Duration::days(state.cfg.auth.refresh_token_ttl_days as i64),
    );

    db::user::save_refresh_token(
        &state.pool,
        user_id,
        building_id,
        &input.email,
        &refresh_hash,
        expire_at,
    )
    .await?;

    let secure = state.cfg.app.public_url.starts_with("https");
    let body = Json(BootstrapResponse {
        user_id:     user_id.to_string(),
        building_id: building_id.to_string(),
        roles:       vec!["manager".to_string()],
        last_name:   input.last_name.clone(),
        first_name:  input.first_name.clone(),
    });

    let mut headers = HeaderMap::new();
    headers.append(header::SET_COOKIE, HeaderValue::from_str(&set_cookie("access_token",  &access_token,  state.cfg.auth.access_token_ttl_secs, secure)).unwrap());
    headers.append(header::SET_COOKIE, HeaderValue::from_str(&set_cookie("refresh_token", &refresh_plain, state.cfg.auth.refresh_token_ttl_days * 86400, secure)).unwrap());
    Ok((StatusCode::CREATED, headers, body).into_response())
}
