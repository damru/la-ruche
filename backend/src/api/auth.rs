use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::{jwt, password, token},
    db::{self, AppState},
    domain::Role,
    errors::{AppError, AppResult},
};

// ── DTOs ──────────────────────────────────────────────────────────

#[derive(Deserialize, Validate, ToSchema)]
pub struct LoginInput {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct ResetRequestInput {
    #[validate(email)]
    pub email: String,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct ResetConfirmInput {
    pub token: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct AcceptInviteInput {
    pub token: String,
    #[validate(length(min = 1))]
    pub last_name: String,
    #[validate(length(min = 1))]
    pub first_name: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub user_id: Uuid,
    pub building_id: Uuid,
    pub roles: Vec<String>,
    pub last_name: String,
    pub first_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
}

// ── Helpers cookies ───────────────────────────────────────────────

fn set_cookie(name: &str, value: &str, max_age_secs: u64, secure: bool) -> String {
    let secure_flag = if secure { "; Secure" } else { "" };
    format!(
        "{}={}; HttpOnly{}; SameSite=Strict; Path=/; Max-Age={}",
        name, value, secure_flag, max_age_secs
    )
}

fn clear_cookie(name: &str) -> String {
    format!("{}=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0", name)
}

fn append_set_cookie(headers: &mut HeaderMap, cookie: String) -> AppResult<()> {
    let value = HeaderValue::from_str(&cookie).map_err(|e| AppError::Internal(e.into()))?;
    headers.append(header::SET_COOKIE, value);
    Ok(())
}

fn build_auth_cookie_headers(
    access_token: &str,
    refresh_token: &str,
    access_ttl_secs: u64,
    refresh_ttl_days: u64,
    secure: bool,
) -> AppResult<HeaderMap> {
    let mut headers = HeaderMap::new();
    append_set_cookie(
        &mut headers,
        set_cookie("access_token", access_token, access_ttl_secs, secure),
    )?;
    append_set_cookie(
        &mut headers,
        set_cookie(
            "refresh_token",
            refresh_token,
            refresh_ttl_days * 86400,
            secure,
        ),
    )?;
    Ok(headers)
}

fn build_logout_cookie_headers() -> AppResult<HeaderMap> {
    let mut headers = HeaderMap::new();
    append_set_cookie(&mut headers, clear_cookie("access_token"))?;
    append_set_cookie(&mut headers, clear_cookie("refresh_token"))?;
    Ok(headers)
}

fn parse_roles(raw_roles: &[String]) -> AppResult<Vec<Role>> {
    raw_roles
        .iter()
        .map(|role| {
            role.parse().map_err(|_| {
                AppError::Internal(anyhow::anyhow!("Invalid role in database: {role}"))
            })
        })
        .collect()
}

// ── Handlers ──────────────────────────────────────────────────────

/// POST /auth/login
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(input): Json<LoginInput>,
) -> AppResult<Response> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user = db::user::find_by_email(&state.pool, &input.email)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if !user.active {
        return Err(AppError::Unauthorized);
    }

    let hash = user
        .password_hash
        .as_deref()
        .ok_or(AppError::Unauthorized)?;
    password::verify_password(&input.password, hash)?;

    // Fetch building_id + roles (first active building of the user)
    let row = sqlx::query!(
        "SELECT building_id, roles::text[] as roles FROM user_building WHERE user_id = $1 LIMIT 1",
        user.id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::Forbidden)?;

    let building_id = row.building_id;
    let roles = row.roles.unwrap_or_default();

    // Access token
    let parsed_roles = parse_roles(&roles)?;

    let access_token = jwt::generate_access_token(
        user.id,
        building_id,
        parsed_roles,
        &state.cfg.auth.jwt_secret,
        state.cfg.auth.access_token_ttl_secs,
    )?;

    // Refresh token
    let (refresh_plain, refresh_hash, expire_at) = token::generate_token(chrono::Duration::days(
        state.cfg.auth.refresh_token_ttl_days as i64,
    ));

    db::user::save_refresh_token(
        &state.pool,
        user.id,
        building_id,
        &user.email,
        &refresh_hash,
        expire_at,
    )
    .await?;

    let secure = state.cfg.app.public_url.starts_with("https");
    let body = Json(LoginResponse {
        user_id: user.id,
        building_id,
        roles: roles.clone(),
        last_name: user.last_name.clone(),
        first_name: user.first_name.clone(),
        email: None,
        phone: None,
    });

    let headers = build_auth_cookie_headers(
        &access_token,
        &refresh_plain,
        state.cfg.auth.access_token_ttl_secs,
        state.cfg.auth.refresh_token_ttl_days,
        secure,
    )?;
    Ok((StatusCode::OK, headers, body).into_response())
}

/// POST /auth/logout
pub async fn logout() -> AppResult<impl IntoResponse> {
    Ok((StatusCode::NO_CONTENT, build_logout_cookie_headers()?))
}

/// POST /auth/refresh
pub async fn refresh(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> AppResult<Response> {
    let cookie_str = headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let refresh_plain = cookie_str
        .split(';')
        .filter_map(|part| {
            let mut kv = part.trim().splitn(2, '=');
            let k = kv.next()?.trim();
            let v = kv.next()?.trim();
            (k == "refresh_token").then_some(v)
        })
        .next()
        .ok_or(AppError::TokenInvalid)?;
    let old_hash = token::hash_token(refresh_plain);

    let (new_plain, new_hash, expires_at) = token::generate_token(chrono::Duration::days(
        state.cfg.auth.refresh_token_ttl_days as i64,
    ));

    let info = db::user::rotate_refresh_token(&state.pool, &old_hash, &new_hash, expires_at)
        .await?
        .ok_or(AppError::TokenInvalid)?;

    let user_id: uuid::Uuid = info["user_id"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .ok_or(AppError::TokenInvalid)?;
    let building_id: uuid::Uuid = info["building_id"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .ok_or(AppError::TokenInvalid)?;

    let roles = db::user::get_roles(&state.pool, user_id, building_id).await?;

    let parsed_roles = parse_roles(&roles)?;

    let access_token = jwt::generate_access_token(
        user_id,
        building_id,
        parsed_roles,
        &state.cfg.auth.jwt_secret,
        state.cfg.auth.access_token_ttl_secs,
    )?;

    let secure = state.cfg.app.public_url.starts_with("https");
    let headers = build_auth_cookie_headers(
        &access_token,
        &new_plain,
        state.cfg.auth.access_token_ttl_secs,
        state.cfg.auth.refresh_token_ttl_days,
        secure,
    )?;
    Ok((StatusCode::NO_CONTENT, headers).into_response())
}

/// POST /auth/reset-password/request
pub async fn reset_password_request(
    State(state): State<Arc<AppState>>,
    Json(input): Json<ResetRequestInput>,
) -> AppResult<StatusCode> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Always returns 204 — no account existence leak
    if let Some(user) = db::user::find_by_email(&state.pool, &input.email).await? {
        let (plain, hash, expire_at) = token::generate_token(chrono::Duration::seconds(
            state.cfg.auth.reset_token_ttl_secs as i64,
        ));
        db::user::create_token(
            &state.pool,
            &user.email,
            "reset_password",
            &hash,
            None,
            None,
            expire_at,
        )
        .await?;
        // TODO post-MVP: send email with reset link /reset/{plain}
        tracing::info!(email = %user.email, "Password reset token generated (email not sent -- post-MVP)");
        let _ = plain;
    }

    Ok(StatusCode::NO_CONTENT)
}

/// POST /auth/reset-password/confirm
pub async fn reset_password_confirm(
    State(state): State<Arc<AppState>>,
    Json(input): Json<ResetConfirmInput>,
) -> AppResult<StatusCode> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let token_hash = token::hash_token(&input.token);
    let info = db::user::consume_token(&state.pool, &token_hash, "reset_password")
        .await?
        .ok_or(AppError::TokenInvalid)?;

    let email = info["email"].as_str().ok_or(AppError::TokenInvalid)?;
    let user = db::user::find_by_email(&state.pool, email)
        .await?
        .ok_or(AppError::NotFound)?;

    let hash = password::hash_password(&input.password, &state.cfg.auth)?;
    db::user::update_password(&state.pool, user.id, &hash).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /auth/invite/accept
pub async fn invite_accept(
    State(state): State<Arc<AppState>>,
    Json(input): Json<AcceptInviteInput>,
) -> AppResult<StatusCode> {
    input
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let token_hash = token::hash_token(&input.token);
    let info = db::user::consume_token(&state.pool, &token_hash, "invitation")
        .await?
        .ok_or(AppError::TokenInvalid)?;

    let email = info["email"].as_str().ok_or(AppError::TokenInvalid)?;
    let building_id: uuid::Uuid = info["building_id"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .ok_or(AppError::TokenInvalid)?;
    let role = info["invited_role"]
        .as_str()
        .unwrap_or("co_owner")
        .to_string();

    let hash = password::hash_password(&input.password, &state.cfg.auth)?;
    let user = db::user::create(
        &state.pool,
        email,
        &input.last_name,
        &input.first_name,
        &hash,
    )
    .await?;

    sqlx::query!(
        "INSERT INTO user_building (user_id, building_id, roles) VALUES ($1, $2, ARRAY[$3::user_role])",
        user.id, building_id, role as _
    ).execute(&state.pool).await?;

    Ok(StatusCode::CREATED)
}

/// POST /auth/register — stub (invite flow is the primary registration path)
pub async fn register(
    State(_state): State<Arc<AppState>>,
    Json(_input): Json<serde_json::Value>,
) -> AppResult<StatusCode> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

/// GET /auth/me — returns the current user's profile (hydrates the frontend after refresh)
pub async fn me(
    State(state): State<Arc<AppState>>,
    auth: crate::domain::AuthUser,
) -> AppResult<impl axum::response::IntoResponse> {
    let user = db::user::find_by_id(&state.pool, auth.user_id)
        .await?
        .ok_or(AppError::NotFound)?;
    let phone = db::user::get_phone(&state.pool, auth.user_id).await?;

    Ok(Json(LoginResponse {
        user_id: auth.user_id,
        building_id: auth.building_id,
        roles: auth.roles.iter().map(|r| r.to_string()).collect(),
        last_name: user.last_name,
        first_name: user.first_name,
        email: Some(user.email),
        phone,
    }))
}

// ── Passkey stubs (full implementation requires ceremony state) ──

pub async fn passkey_register_start(
    State(_state): State<Arc<AppState>>,
    _auth: crate::domain::AuthUser,
) -> AppResult<StatusCode> {
    tracing::warn!("passkey_register_start not implemented");
    Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn passkey_register_finish(State(_state): State<Arc<AppState>>) -> AppResult<StatusCode> {
    tracing::warn!("passkey_register_finish not implemented");
    Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn passkey_login_start(State(_state): State<Arc<AppState>>) -> AppResult<StatusCode> {
    tracing::warn!("passkey_login_start not implemented");
    Ok(StatusCode::NOT_IMPLEMENTED)
}

pub async fn passkey_login_finish(State(_state): State<Arc<AppState>>) -> AppResult<StatusCode> {
    tracing::warn!("passkey_login_finish not implemented");
    Ok(StatusCode::NOT_IMPLEMENTED)
}
