use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::Utc;
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
        .map_err(|_| AppError::validation_code("errors.auth.invalid_login_payload"))?;

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
        user.email.as_deref().unwrap_or(""),
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
        .map_err(|_| AppError::validation_code("errors.auth.invalid_reset_request_payload"))?;

    // Always returns 204 — no account existence leak
    if let Some(user) = db::user::find_by_email(&state.pool, &input.email).await? {
        let (plain, hash, expire_at) = token::generate_token(chrono::Duration::seconds(
            state.cfg.auth.reset_token_ttl_secs as i64,
        ));
        db::user::create_token(
            &state.pool,
            user.email.as_deref().unwrap_or(""),
            "reset_password",
            &hash,
            None,
            None,
            expire_at,
        )
        .await?;
        // TODO post-MVP: send email with reset link /reset/{plain}
        let email_str = user.email.as_deref().unwrap_or("");
        tracing::info!(email = %email_str, "Password reset token generated (email not sent -- post-MVP)");
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
        .map_err(|_| AppError::validation_code("errors.auth.invalid_reset_confirm_payload"))?;

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

/// GET /auth/invite/validate?token=... — check invitation without consuming (EP-002-S03)
pub async fn validate_invite(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> AppResult<Json<serde_json::Value>> {
    let plain = params
        .get("token")
        .filter(|t| !t.is_empty())
        .ok_or_else(|| AppError::validation_code("errors.auth.missing_token"))?;

    let token_hash = token::hash_token(plain);
    let info = db::user::peek_invite_token(&state.pool, &token_hash)
        .await?
        .ok_or(AppError::NotFound)?;

    let now = Utc::now();
    if info.expires_at <= now {
        return Err(AppError::Conflict("errors.auth.invite_expired".into()));
    }

    Ok(Json(serde_json::json!({
        "email": info.email,
        "role": info.invited_role,
        "unit_id": info.unit_id,
        "expires_at": info.expires_at,
        "first_name": info.first_name,
        "last_name": info.last_name,
    })))
}

/// POST /auth/invite/accept — complete account creation from invitation (EP-002-S03)
pub async fn invite_accept(
    State(state): State<Arc<AppState>>,
    Json(input): Json<AcceptInviteInput>,
) -> AppResult<StatusCode> {
    input
        .validate()
        .map_err(|_| AppError::validation_code("errors.auth.invalid_invite_accept_payload"))?;

    let token_hash = token::hash_token(&input.token);

    // Peek first to differentiate expired vs already-used (AC-GP-002-C, AC-GP-002-D)
    let peek = db::user::peek_invite_token(&state.pool, &token_hash)
        .await?
        .ok_or(AppError::TokenInvalid)?;

    if peek.expires_at <= Utc::now() {
        // 410 Gone — expired
        return Err(AppError::Conflict("errors.auth.invite_expired".into()));
    }

    // Consume atomically — returns None if already used
    let info = db::user::consume_invite_token(&state.pool, &token_hash)
        .await?
        .ok_or_else(|| AppError::Conflict("errors.auth.invite_already_used".into()))?;

    let email = &info.email;
    let building_id = info.building_id.ok_or(AppError::TokenInvalid)?;
    let role = info.invited_role.as_deref().unwrap_or("co_owner");

    let hash = password::hash_password(&input.password, &state.cfg.auth)?;

    let user_id = if let Some(linked_id) = info.user_id {
        // Token linked to a pre-registered no_access user — activate their account.
        // Email and names are set; only password (and possibly email) need updating.
        sqlx::query(
            r#"UPDATE user_account
               SET email = $2, last_name = $3, first_name = $4, password_hash = $5,
                   active = TRUE, updated_at = NOW()
               WHERE id = $1"#,
        )
        .bind(linked_id)
        .bind(email)
        .bind(&input.last_name)
        .bind(&input.first_name)
        .bind(&hash)
        .execute(&state.pool)
        .await?;
        linked_id
    } else {
        // Standard flow: create or update user account by email
        match db::user::find_by_email(&state.pool, email).await? {
            Some(existing) => {
                sqlx::query(
                    r#"UPDATE user_account
                       SET last_name = $2, first_name = $3, password_hash = $4,
                           active = TRUE, updated_at = NOW()
                       WHERE id = $1"#,
                )
                .bind(existing.id)
                .bind(&input.last_name)
                .bind(&input.first_name)
                .bind(&hash)
                .execute(&state.pool)
                .await?;
                existing.id
            }
            None => {
                db::user::create(
                    &state.pool,
                    email,
                    &input.last_name,
                    &input.first_name,
                    &hash,
                )
                .await?
                .id
            }
        }
    };

    // Add to building (ON CONFLICT DO NOTHING handles pre-registered users)
    db::user::create_user_building(&state.pool, user_id, building_id, role).await?;

    // If tenant invite with unit_id, create unit_owner record
    if let Some(unit_id) = info.unit_id {
        let today = chrono::Utc::now().naive_utc().date();
        db::unit::transfer_ownership(&state.pool, unit_id, building_id, user_id, today).await?;
    }

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
        email: user.email,
        phone,
    }))
}

#[derive(Deserialize, Validate, ToSchema)]
pub struct UpdateMeInput {
    pub phone: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    /// Rendered `prefilled` in the frontend (AC-PP-002-B) — present here only
    /// so an attempt to modify it can be detected and rejected (AC-PP-002-D).
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

/// PATCH /auth/me — update the current user's own profile (email login + phone)
pub async fn update_me(
    State(state): State<Arc<AppState>>,
    auth: crate::domain::AuthUser,
    Json(input): Json<UpdateMeInput>,
) -> AppResult<impl axum::response::IntoResponse> {
    input
        .validate()
        .map_err(|_| AppError::validation_code("errors.users.invalid_email"))?;

    let current = db::user::find_by_id(&state.pool, auth.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // full_name is prefilled/locked — reject any attempt to change it (AC-PP-002-D)
    if let Some(ref first_name) = input.first_name
        && *first_name != current.first_name
    {
        return Err(AppError::Forbidden);
    }
    if let Some(ref last_name) = input.last_name
        && *last_name != current.last_name
    {
        return Err(AppError::Forbidden);
    }

    db::user::update_phone(&state.pool, auth.user_id, input.phone.as_deref()).await?;

    if let Some(new_email) = input.email.as_deref() {
        let normalized = new_email.to_lowercase();
        db::user::update_user_email(&state.pool, auth.user_id, &normalized)
            .await
            .map_err(|e| match &e {
                AppError::Database(sqlx::Error::Database(db_err))
                    if db_err.constraint() == Some("user_account_email_unique") =>
                {
                    AppError::Conflict("errors.users.email_already_exists".into())
                }
                _ => e,
            })?;
    }

    let user = db::user::find_by_id(&state.pool, auth.user_id)
        .await?
        .ok_or(AppError::NotFound)?;
    // ponytail: phone was just written above, so echo input back instead of re-querying.
    Ok(Json(LoginResponse {
        user_id: auth.user_id,
        building_id: auth.building_id,
        roles: auth.roles.iter().map(|r| r.to_string()).collect(),
        last_name: user.last_name,
        first_name: user.first_name,
        email: user.email,
        phone: input.phone,
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
