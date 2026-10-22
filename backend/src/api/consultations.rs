// ── api/consultations.rs — Informal consultations (EP-010) ────
//
// LEGAL: These consultations are informal and have no decision-making
// legal value under the law of 10 July 1965 (arts. 17 and 24).
// They do not replace votes in the general assembly.
// A mandatory notice is included in every response.
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    db::{self, AppState},
    domain::{AuthUser, Role},
    errors::{AppError, AppResult, json_value},
};

/// Legal disclaimer added to every consultation response.
const INFORMAL_DISCLAIMER: &str =
    "Consultation informelle — aucune valeur décisionnelle légale (loi 10 juillet 1965).";

/// Valid role strings accepted in visible_to_roles.
const VALID_ROLES: &[&str] = &["manager", "board_member", "co_owner", "tenant"];

#[derive(Deserialize)]
pub struct CreateConsultationInput {
    pub title: String,
    pub description: Option<String>,
    /// Roles that can see and respond to this consultation.
    pub visible_to_roles: Vec<String>,
    /// Optional automatic close date.
    pub closed_at: Option<DateTime<Utc>>,
    /// Choices presented to respondents (min 2).
    pub choices: Vec<String>,
}

#[derive(Deserialize)]
pub struct RespondInput {
    /// One or more choice IDs selected by the user (multi-choice supported).
    pub choice_ids: Vec<Uuid>,
    pub comment: Option<String>,
}

/// All authenticated users can list consultations.
pub async fn list(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let rows = db::consultation::list(&state.pool, auth.building_id, auth.user_id).await?;
    Ok(Json(serde_json::json!({
        "disclaimer": INFORMAL_DISCLAIMER,
        "consultations": rows,
    })))
}

/// All authenticated users can view a consultation and its choices.
pub async fn get(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let c = db::consultation::get_with_choices(&state.pool, id, auth.building_id, auth.user_id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(serde_json::json!({
        "disclaimer": INFORMAL_DISCLAIMER,
        "consultation": c,
    })))
}

/// Shared payload validation for create and update (EP-010-S04).
fn validate_input(input: &CreateConsultationInput) -> AppResult<()> {
    if input.title.trim().is_empty() {
        return Err(AppError::validation_code(
            "errors.consultations.title_required",
        ));
    }
    if input.choices.len() < 2 {
        return Err(AppError::validation_code(
            "errors.consultations.min_choices",
        ));
    }
    for choice in &input.choices {
        if choice.trim().is_empty() {
            return Err(AppError::validation_code(
                "errors.consultations.choice_label_required",
            ));
        }
    }
    if input.visible_to_roles.is_empty() {
        return Err(AppError::validation_code(
            "errors.consultations.visible_roles_required",
        ));
    }
    for role in &input.visible_to_roles {
        if !VALID_ROLES.contains(&role.as_str()) {
            return Err(AppError::validation_code(
                "errors.consultations.invalid_visible_role",
            ));
        }
    }
    Ok(())
}

/// Any authenticated user can create a consultation.
/// It is created in 'draft' state (EP-010-S04) and opened explicitly.
pub async fn create(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateConsultationInput>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    validate_input(&input)?;

    let result = db::consultation::create(
        &state.pool,
        auth.building_id,
        auth.user_id,
        &input.title,
        input.description.as_deref(),
        &input.visible_to_roles,
        input.closed_at,
        &input.choices,
    )
    .await?;

    Ok((StatusCode::CREATED, json_value(result)?))
}

/// PUT /consultations/{id} — edits a draft consultation (EP-010-S04).
/// Managers and board members can edit any draft; the creator can edit their own.
pub async fn update(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateConsultationInput>,
) -> AppResult<StatusCode> {
    validate_input(&input)?;

    let can_manage = auth.has_role(&Role::Manager) || auth.has_role(&Role::BoardMember);
    if !can_manage {
        let c = db::consultation::get_with_choices(&state.pool, id, auth.building_id, auth.user_id)
            .await?
            .ok_or(AppError::NotFound)?;
        let created_by = c
            .get("created_by")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or(AppError::NotFound)?;
        if created_by != auth.user_id {
            return Err(AppError::Forbidden);
        }
    }

    db::consultation::update_draft(
        &state.pool,
        id,
        auth.building_id,
        &input.title,
        input.description.as_deref(),
        &input.visible_to_roles,
        input.closed_at,
        &input.choices,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /consultations/{id}/open — opens a draft consultation (draft → open).
/// Managers and board members can open any draft; the creator can open their own.
pub async fn open(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let can_manage = auth.has_role(&Role::Manager) || auth.has_role(&Role::BoardMember);
    if !can_manage {
        let c = db::consultation::get_with_choices(&state.pool, id, auth.building_id, auth.user_id)
            .await?
            .ok_or(AppError::NotFound)?;
        let created_by = c
            .get("created_by")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or(AppError::NotFound)?;
        if created_by != auth.user_id {
            return Err(AppError::Forbidden);
        }
    }

    db::consultation::open(&state.pool, id, auth.building_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /consultations/{id}/non-respondents — manager | board_member (EP-010-S05).
pub async fn non_respondents(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    let users = db::consultation::non_respondents(&state.pool, id, auth.building_id).await?;
    let count = users.len();
    Ok(Json(serde_json::json!({
        "count": count,
        "users": users,
    })))
}

/// POST /consultations/{id}/remind — records a reminder for non-respondents
/// of an open consultation (EP-010-S05).
pub async fn remind(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    let recipient_count =
        db::consultation::remind_non_respondents(&state.pool, id, auth.building_id, auth.user_id)
            .await?;
    Ok(Json(serde_json::json!({
        "recipient_count": recipient_count,
    })))
}

/// POST /consultations/{id}/respond — submit or update a response.
/// The user must be in `visible_to_roles`.
pub async fn respond(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<RespondInput>,
) -> AppResult<StatusCode> {
    if input.choice_ids.is_empty() {
        return Err(AppError::validation_code(
            "errors.consultations.min_choices",
        ));
    }

    // visible_to_roles check: fetch consultation and verify caller role is allowed
    let c = db::consultation::get_with_choices(&state.pool, id, auth.building_id, auth.user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let visible_roles: Vec<String> = c
        .get("visible_to_roles")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_owned()))
                .collect()
        })
        .unwrap_or_default();

    let caller_roles: Vec<String> = auth.roles.iter().map(|r| r.to_string()).collect();
    let authorized = caller_roles.iter().any(|r| visible_roles.contains(r));
    if !authorized {
        return Err(AppError::Forbidden);
    }

    db::consultation::respond(
        &state.pool,
        id,
        auth.building_id,
        auth.user_id,
        &input.choice_ids,
        input.comment.as_deref(),
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /consultations/{id}/results — any authenticated user can view results.
pub async fn results(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let data = db::consultation::results(&state.pool, id, auth.building_id).await?;
    Ok(Json(serde_json::json!({
        "disclaimer": INFORMAL_DISCLAIMER,
        "results": data,
    })))
}

/// DELETE /consultations/{id} — managers only; only closed consultations can be deleted.
/// Deletes the consultation and all related choices and responses.
pub async fn delete(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    db::consultation::delete(&state.pool, id, auth.building_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /consultations/{id}/close — managers and board members can close any consultation.
pub async fn close(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember])?;
    db::consultation::close(&state.pool, id, auth.building_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(title: &str, choices: &[&str], roles: &[&str]) -> CreateConsultationInput {
        CreateConsultationInput {
            title: title.to_string(),
            description: None,
            visible_to_roles: roles.iter().map(|r| r.to_string()).collect(),
            closed_at: None,
            choices: choices.iter().map(|c| c.to_string()).collect(),
        }
    }

    #[test]
    fn validate_input_accepts_valid_payload() {
        assert!(validate_input(&input("Titre", &["A", "B"], &["co_owner"])).is_ok());
    }

    #[test]
    fn validate_input_rejects_empty_title() {
        let err = validate_input(&input("  ", &["A", "B"], &["co_owner"])).unwrap_err();
        assert!(
            matches!(err, AppError::Validation(m) if m == "errors.consultations.title_required")
        );
    }

    #[test]
    fn validate_input_rejects_less_than_two_choices() {
        let err = validate_input(&input("Titre", &["A"], &["co_owner"])).unwrap_err();
        assert!(matches!(err, AppError::Validation(m) if m == "errors.consultations.min_choices"));
    }

    #[test]
    fn validate_input_rejects_blank_choice_label() {
        let err = validate_input(&input("Titre", &["A", " "], &["co_owner"])).unwrap_err();
        assert!(
            matches!(err, AppError::Validation(m) if m == "errors.consultations.choice_label_required")
        );
    }

    #[test]
    fn validate_input_rejects_empty_visible_roles() {
        let err = validate_input(&input("Titre", &["A", "B"], &[])).unwrap_err();
        assert!(
            matches!(err, AppError::Validation(m) if m == "errors.consultations.visible_roles_required")
        );
    }

    #[test]
    fn validate_input_rejects_invalid_role() {
        let err = validate_input(&input("Titre", &["A", "B"], &["superadmin"])).unwrap_err();
        assert!(
            matches!(err, AppError::Validation(m) if m == "errors.consultations.invalid_visible_role")
        );
    }
}
