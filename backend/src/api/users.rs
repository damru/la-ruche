use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::token,
    db::{self, AppState},
    domain::{AuthUser, Role, owners},
    errors::{AppError, AppResult},
};

// ── DTOs ──────────────────────────────────────────────────────────

#[derive(Deserialize, Validate)]
pub struct InviteInput {
    #[validate(email)]
    pub email: String,
    pub role: String,
    pub unit_id: Option<Uuid>,
    /// Expiry in hours — defaults to 72 (AC-GP-002-A)
    pub expires_in_hours: Option<u32>,
}

#[derive(Serialize)]
pub struct InviteResponse {
    pub token: String,
    pub email: String,
    pub role: String,
    pub expires_at: DateTime<Utc>,
    pub invite_link: String,
}

#[derive(Deserialize, Validate)]
pub struct UpdateRolesInput {
    pub roles: Vec<String>,
}

#[derive(Deserialize, Validate)]
pub struct AnonymizeInput {
    #[validate(length(min = 1))]
    pub confirm_name: String,
}

#[derive(Serialize)]
pub struct UserUnitSummary {
    pub unit_id: Uuid,
    pub unit_number: String,
    pub unit_type: String,
    pub start_date: NaiveDate,
}

#[derive(Serialize)]
pub struct UserDetailResponse {
    pub id: Uuid,
    pub email: String,
    pub last_name: String,
    pub first_name: String,
    pub phone: Option<String>,
    pub active: bool,
    pub roles: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub units: Vec<UserUnitSummary>,
}

fn require_manager(auth: &AuthUser) -> AppResult<()> {
    auth.require_role(&Role::Manager)
}

fn require_manager_or_board(auth: &AuthUser) -> AppResult<()> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])
}

// ── Handlers ──────────────────────────────────────────────────────

/// GET /users — list active users + pending invitations for the current building
pub async fn list(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
) -> AppResult<Json<Vec<serde_json::Value>>> {
    require_manager_or_board(&auth)?;
    let mut users = db::user::list(&state.pool, auth.building_id).await?;
    // Append pending invites so the manager can see and revoke them
    let mut pending = db::user::list_pending_invites(&state.pool, auth.building_id).await?;
    users.append(&mut pending);
    Ok(Json(users))
}

/// POST /users/invite — generate an invitation link (EP-002-S01, S03)
pub async fn invite(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(input): Json<InviteInput>,
) -> AppResult<(StatusCode, Json<InviteResponse>)> {
    require_manager(&auth)?;
    input
        .validate()
        .map_err(|_| AppError::validation_code("errors.users.invalid_invite_payload"))?;

    // Validate role
    let role = input.role.as_str();
    if !["manager", "board_member", "co_owner", "tenant"].contains(&role) {
        return Err(AppError::validation_code("errors.users.invalid_role"));
    }

    // Validate unit_id is required for tenant role (AC-GP-002-H)
    if role == "tenant" && input.unit_id.is_none() {
        return Err(AppError::validation_code("errors.users.tenant_requires_unit"));
    }

    // Invalidate any prior pending invites for this email/building (AC-GP-002-G)
    db::user::revoke_pending_invites(&state.pool, &input.email, auth.building_id).await?;

    let expiry_hours = input.expires_in_hours.unwrap_or(72) as i64;
    let expires_at = Utc::now() + Duration::hours(expiry_hours);

    let (plain, hash, _) = token::generate_token(Duration::hours(expiry_hours));

    db::user::create_invite_token(
        &state.pool,
        &input.email,
        &hash,
        auth.building_id,
        role,
        input.unit_id,
        expires_at,
    )
    .await?;

    let invite_link = format!(
        "{}/invite/{}",
        state.cfg.app.public_url.trim_end_matches('/'),
        plain
    );

    Ok((
        StatusCode::CREATED,
        Json(InviteResponse {
            token: plain,
            email: input.email,
            role: role.to_string(),
            expires_at,
            invite_link,
        }),
    ))
}

/// GET /users/{id} — get user detail with lots and roles (EP-002-S04)
pub async fn get(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<UserDetailResponse>> {
    require_manager_or_board(&auth)?;

    let detail = db::user::get_user_detail(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let units = db::user::get_user_units(&state.pool, id, auth.building_id)
        .await?
        .into_iter()
        .map(|u| UserUnitSummary {
            unit_id: u.unit_id,
            unit_number: u.unit_number,
            unit_type: u.unit_type,
            start_date: u.start_date,
        })
        .collect();

    Ok(Json(UserDetailResponse {
        id: detail.id,
        email: detail.email,
        last_name: detail.last_name,
        first_name: detail.first_name,
        phone: detail.phone,
        active: detail.active,
        roles: detail.roles,
        created_at: detail.created_at,
        units,
    }))
}

/// DELETE /users/{id} — archive user (revoke access, keep data) (EP-002-S04, US-GP-004)
pub async fn deactivate(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    require_manager(&auth)?;

    // Prevent self-deactivation
    if id == auth.user_id {
        return Err(AppError::validation_code("errors.users.cannot_deactivate_self"));
    }

    let found = db::user::archive_user(&state.pool, id, auth.building_id).await?;
    if !found {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

/// PUT /users/{id}/roles — update user roles (EP-002-S06, US-GP-006)
pub async fn update_roles(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateRolesInput>,
) -> AppResult<StatusCode> {
    require_manager(&auth)?;

    // Validate roles
    for role in &input.roles {
        if !["manager", "board_member", "co_owner", "tenant"].contains(&role.as_str()) {
            return Err(AppError::validation_code("errors.users.invalid_role"));
        }
    }

    // Prevent removing manager role from self without replacement (AC-GP-006-D)
    if id == auth.user_id && !input.roles.contains(&"manager".to_string()) {
        return Err(AppError::validation_code("errors.users.cannot_remove_own_manager_role"));
    }

    db::user::update_roles(&state.pool, id, auth.building_id, &input.roles).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /users/{id}/anonymize — RGPD anonymization (EP-002-S05, US-GP-005)
pub async fn anonymize(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<AnonymizeInput>,
) -> AppResult<StatusCode> {
    require_manager(&auth)?;
    input
        .validate()
        .map_err(|_| AppError::validation_code("errors.users.invalid_anonymize_payload"))?;

    let detail = db::user::get_user_detail(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // Require confirmation with exact full name (AC-GP-005-C)
    let full_name = format!("{} {}", detail.first_name, detail.last_name);
    if !owners::validate_anonymize_confirmation(&full_name, &input.confirm_name) {
        return Err(AppError::validation_code("errors.users.anonymize_confirmation_mismatch"));
    }

    let anon_id = Uuid::new_v4();
    let anon_value = owners::anonymized_value(anon_id);

    db::user::anonymize_user(&state.pool, id, &anon_value).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /users/invites/{id} — revoke a pending invitation token (EP-002-S03)
pub async fn revoke_invite(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(token_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    require_manager(&auth)?;
    let found = db::user::revoke_invite_by_id(&state.pool, token_id, auth.building_id).await?;
    if !found {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
