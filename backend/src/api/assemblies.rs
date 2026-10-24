/// Meeting handlers -- reference pattern for all other modules.
///
/// Rule: handler = validate input + check role + call db/ or domain/ + return.
/// Zero business logic here.
use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::{
    api::documents::DocumentListItem,
    db::AppState,
    domain::{AuthUser, Role, immutability::resolution_status, permissions, vote::VoteRule},
    errors::{AppError, AppResult},
};

// ── DTOs ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateMeetingInput {
    #[validate(length(min = 1))]
    pub location: Option<String>,
    pub meeting_date: DateTime<Utc>,
    pub meeting_type: MeetingTypeDto,
    pub agenda: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MeetingTypeDto {
    Annual,
    Extraordinary,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MeetingResponse {
    pub id: Uuid,
    pub meeting_type: MeetingTypeDto,
    pub status: String,
    pub notice_date: Option<NaiveDate>,
    pub meeting_date: DateTime<Utc>,
    pub location: Option<String>,
    pub agenda: Option<String>,
    pub minutes_document_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub resolution_count: i64,
    pub passed_count: i64,
    // EP-013-S02 — session roles (set on /start, frozen at /close)
    pub president_name: Option<String>,
    pub president_user_id: Option<Uuid>,
    pub secretary_name: Option<String>,
    pub secretary_user_id: Option<Uuid>,
    pub scrutineers: Vec<ScrutineerEntry>,
    pub minutes_notified_at: Option<DateTime<Utc>>,
    pub convocation_sent_at: Option<DateTime<Utc>>,
    pub convocation_channel: Option<String>,
    pub minutes_president_signed_at: Option<DateTime<Utc>>,
    pub minutes_secretary_signed_at: Option<DateTime<Utc>>,
    pub minutes_scrutineers_signed: Vec<ScrutineerSignature>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ScrutineerSignature {
    pub name: String,
    pub signed_at: DateTime<Utc>,
}

/// `president_signed` / `secretary_signed`: if `true`, a server-side timestamp is set.
/// `scrutineers_signed`: free-text names of scrutineers who signed.
#[derive(Debug, Deserialize, ToSchema)]
pub struct SignMinutesInput {
    pub president_signed: bool,
    pub secretary_signed: bool,
    #[serde(default)]
    pub scrutineers_signed: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConvocationChannelDto {
    Lrar,
    HandDelivery,
    Email,
}

/// `sent_at` is optional: if absent, the server records `NOW()`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CallMeetingInput {
    pub channel: ConvocationChannelDto,
    pub sent_at: Option<DateTime<Utc>>,
}

/// if the user is known in the system. Stored as JSONB on `general_meeting.scrutineers`.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ScrutineerEntry {
    pub name: String,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct StartMeetingInput {
    #[validate(length(min = 1))]
    pub president_name: String,
    pub president_user_id: Option<Uuid>,
    #[validate(length(min = 1))]
    pub secretary_name: String,
    pub secretary_user_id: Option<Uuid>,
    #[serde(default)]
    pub scrutineers: Vec<ScrutineerEntry>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateResolutionInput {
    #[validate(length(min = 1, max = 500))]
    pub title: String,
    pub description: Option<String>,
    pub vote_type: String, // "simple_majority" | "absolute_majority" | "double_majority" | "unanimity"
    pub number: i32,
}

/// Ordered agenda ids; must be an exact permutation of the current draft resolutions.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ReorderResolutionsInput {
    pub ordered_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct VoteInput {
    pub unit_id: Uuid,
    pub position: String, // "in_favor" | "against" | "abstain"
    pub proxy_for: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MeetingDocumentKindDto {
    Notice,
    Attachment,
    VoteByMailForm,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MeetingDocumentResponse {
    #[serde(flatten)]
    pub document: DocumentListItem,
    pub kind: MeetingDocumentKindDto,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AttachMeetingDocumentInput {
    pub document_id: Uuid,
    pub kind: MeetingDocumentKindDto,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MeetingAttendanceModeDto {
    Present,
    Proxy,
    Correspondence,
    Absent,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MeetingAttendanceEntryResponse {
    pub unit_id: Uuid,
    pub unit_number: String,
    pub owner_name: Option<String>,
    pub mode: MeetingAttendanceModeDto,
    pub representative_name: Option<String>,
    pub shares_represented: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MeetingAttendanceResponse {
    pub entries: Vec<MeetingAttendanceEntryResponse>,
    pub shares_present: i32,
    pub total_shares: i32,
    pub present_units: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MeetingAttendanceInputEntry {
    pub unit_id: Uuid,
    pub mode: MeetingAttendanceModeDto,
    pub representative_name: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SaveMeetingAttendanceInput {
    pub entries: Vec<MeetingAttendanceInputEntry>,
}

// ── Handlers ─────────────────────────────────────────────────────

pub async fn list(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<MeetingResponse>>> {
    let meetings = crate::db::assembly::list(&state.pool, auth.building_id).await?;
    Ok(Json(meetings))
}

pub async fn create(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateMeetingInput>,
) -> AppResult<(StatusCode, Json<MeetingResponse>)> {
    auth.require_role(&Role::Manager)?;
    input
        .validate()
        .map_err(|_| AppError::validation_code("errors.meetings.invalid_meeting_payload"))?;

    let meeting =
        crate::db::assembly::create(&state.pool, auth.building_id, auth.user_id, input).await?;
    Ok((StatusCode::CREATED, Json(meeting)))
}

pub async fn get(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<MeetingResponse>> {
    let meeting = crate::db::assembly::get(&state.pool, id, auth.building_id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(meeting))
}

pub async fn update(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateMeetingInput>,
) -> AppResult<Json<MeetingResponse>> {
    auth.require_role(&Role::Manager)?;
    input
        .validate()
        .map_err(|_| AppError::validation_code("errors.meetings.invalid_meeting_payload"))?;

    let meeting = crate::db::assembly::update(&state.pool, id, auth.building_id, input)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(meeting))
}

pub async fn call(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<CallMeetingInput>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    crate::db::assembly::call(&state.pool, id, auth.building_id, input).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Deletes a draft meeting; non-draft / foreign / unknown -> 404.
pub async fn delete(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    let deleted = crate::db::assembly::delete_meeting(&state.pool, id, auth.building_id).await?;
    if !deleted {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Evaluates regulatory completeness per art. 7–13 décret 1967.
pub async fn get_convocation_checklist(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<crate::domain::convocation_checklist::ConvocationChecklist>> {
    let checklist =
        crate::db::assembly::load_convocation_checklist(&state.pool, id, auth.building_id).await?;
    Ok(Json(checklist))
}

pub async fn close(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    crate::db::assembly::close(&state.pool, id, auth.building_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_resolutions(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let resolutions =
        crate::db::assembly::list_resolutions(&state.pool, id, auth.building_id).await?;
    Ok(Json(resolutions))
}

pub async fn create_resolution(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateResolutionInput>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    auth.require_role(&Role::Manager)?;
    input
        .validate()
        .map_err(|_| AppError::validation_code("errors.meetings.invalid_resolution_payload"))?;

    let resolution =
        crate::db::assembly::create_resolution(&state.pool, id, auth.building_id, input).await?;
    Ok((StatusCode::CREATED, Json(resolution)))
}

pub async fn update_resolution(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((meeting_id, resolution_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<CreateResolutionInput>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_role(&Role::Manager)?;
    let meeting_status =
        crate::db::assembly::get_meeting_status(&state.pool, meeting_id, auth.building_id)
            .await?
            .ok_or(AppError::NotFound)?;
    crate::domain::immutability::assert_mutable(
        &crate::domain::immutability::Entity::Resolution,
        &meeting_status,
    )?;
    let resolution = crate::db::assembly::update_resolution(
        &state.pool,
        meeting_id,
        resolution_id,
        auth.building_id,
        input,
    )
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(resolution))
}

/// Deletes a draft agenda resolution (404 once convoked).
pub async fn delete_resolution(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((meeting_id, resolution_id)): Path<(Uuid, Uuid)>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    let deleted = crate::db::assembly::delete_resolution(
        &state.pool,
        meeting_id,
        resolution_id,
        auth.building_id,
    )
    .await?;
    if !deleted {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Reorders the draft agenda and renumbers it 1..N.
pub async fn reorder_resolutions(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<ReorderResolutionsInput>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_role(&Role::Manager)?;
    let resolutions = crate::db::assembly::reorder_resolutions(
        &state.pool,
        id,
        auth.building_id,
        input.ordered_ids,
    )
    .await?;
    Ok(Json(resolutions))
}

pub async fn vote(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((meeting_id, resolution_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<VoteInput>,
) -> AppResult<StatusCode> {
    if !permissions::can_vote_gm(&auth) {
        return Err(AppError::Forbidden);
    }

    crate::db::assembly::cast_vote(
        &state.pool,
        meeting_id,
        resolution_id,
        auth.building_id,
        auth.user_id,
        input,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn results(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((meeting_id, resolution_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    let (resolution, tally) =
        crate::db::assembly::get_tally(&state.pool, meeting_id, resolution_id, auth.building_id)
            .await?;

    let rule = parse_vote_rule(&resolution.vote_type)?;
    let result = tally.compute(&rule);

    crate::errors::json_value(result)
}

pub async fn finalize_resolution(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((meeting_id, resolution_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_role(&Role::Manager)?;

    let (resolution, tally) =
        crate::db::assembly::get_tally(&state.pool, meeting_id, resolution_id, auth.building_id)
            .await?;

    let rule = parse_vote_rule(&resolution.vote_type)?;
    let result = tally.compute(&rule);

    let new_status = if result.passed {
        resolution_status::PASSED
    } else {
        resolution_status::REJECTED
    };

    let finalized = crate::db::assembly::finalize_resolution(
        &state.pool,
        meeting_id,
        resolution_id,
        auth.building_id,
        new_status,
    )
    .await?;

    let result_json = if finalized {
        Some(serde_json::to_value(&result).map_err(|_| {
            AppError::bad_request_code("errors.meetings.invalid_resolution_payload")
        })?)
    } else {
        None
    };

    Ok(Json(serde_json::json!({
        "finalized": finalized,
        "result": result_json,
        "eligible_for_second_vote": result.eligible_for_second_vote,
    })))
}

pub async fn list_documents(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<MeetingDocumentResponse>>> {
    let roles: Vec<String> = auth.roles.iter().map(ToString::to_string).collect();
    let documents =
        crate::db::assembly::list_documents(&state.pool, id, auth.building_id, &roles).await?;
    Ok(Json(documents))
}

pub async fn attach_document(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<AttachMeetingDocumentInput>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    let attached = crate::db::assembly::attach_document(
        &state.pool,
        id,
        input.document_id,
        input.kind,
        auth.building_id,
    )
    .await?;
    if !attached {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn detach_document(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((id, document_id)): Path<(Uuid, Uuid)>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    let detached =
        crate::db::assembly::detach_document(&state.pool, id, document_id, auth.building_id)
            .await?;
    if !detached {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_attendance(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<MeetingAttendanceResponse>> {
    let attendance =
        crate::db::assembly::list_attendance(&state.pool, id, auth.building_id).await?;
    Ok(Json(attendance))
}

pub async fn save_attendance(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<SaveMeetingAttendanceInput>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    crate::db::assembly::save_attendance(&state.pool, id, auth.building_id, input.entries).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UnitVoteResponse {
    pub unit_id: Uuid,
    pub position: String,
}

pub async fn list_votes(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((meeting_id, resolution_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Vec<UnitVoteResponse>>> {
    let votes =
        crate::db::assembly::list_votes(&state.pool, meeting_id, resolution_id, auth.building_id)
            .await?;

    Ok(Json(
        votes
            .into_iter()
            .map(|v| UnitVoteResponse {
                unit_id: v.unit_id,
                position: v.position,
            })
            .collect(),
    ))
}

pub async fn start(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<StartMeetingInput>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    input
        .validate()
        .map_err(|_| AppError::validation_code("errors.meetings.session_roles_required"))?;
    crate::db::assembly::start(&state.pool, id, auth.building_id, input).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PatchResolutionInput {
    pub description: Option<String>,
}

pub async fn patch_resolution(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((meeting_id, resolution_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<PatchResolutionInput>,
) -> AppResult<Json<serde_json::Value>> {
    auth.require_role(&Role::Manager)?;
    let meeting_status =
        crate::db::assembly::get_meeting_status(&state.pool, meeting_id, auth.building_id)
            .await?
            .ok_or(AppError::NotFound)?;
    crate::domain::immutability::assert_mutable(
        &crate::domain::immutability::Entity::ResolutionVote,
        &meeting_status,
    )?;
    let resolution = crate::db::assembly::patch_resolution_description(
        &state.pool,
        meeting_id,
        resolution_id,
        auth.building_id,
        input.description,
    )
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(resolution))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MinutesDraftResponse {
    pub markdown: String,
}

/// Generates a structured Markdown draft PV for manual signature. Requires `status = 'closed'`.
pub async fn get_minutes_draft(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<MinutesDraftResponse>> {
    auth.require_role(&Role::Manager)?;
    let ctx = crate::db::assembly::load_minutes_context(&state.pool, id, auth.building_id).await?;
    let markdown = crate::domain::minutes_draft::render(&ctx);
    Ok(Json(MinutesDraftResponse { markdown }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AttachMinutesInput {
    pub document_id: Uuid,
}

/// Attaches a signed PV (already uploaded via pre-signed URL EP-005) to a closed AG.
pub async fn attach_minutes(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<AttachMinutesInput>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    crate::db::assembly::attach_signed_minutes(
        &state.pool,
        id,
        input.document_id,
        auth.building_id,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Records bureau de séance signatures. Requires a signed PV (`minutes_document_id IS NOT NULL`).
pub async fn sign_minutes(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(input): Json<SignMinutesInput>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    crate::db::assembly::sign_minutes(&state.pool, id, auth.building_id, input).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Records the PV notification date for absent co-owners.
/// Contestation deadline: art. 42 loi 1965 = 2 months from this date.
pub async fn notify_minutes(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    crate::db::assembly::notify_minutes(&state.pool, id, auth.building_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

fn parse_vote_rule(s: &str) -> AppResult<VoteRule> {
    match s {
        "simple_majority" => Ok(VoteRule::SimpleMajority),
        "absolute_majority" => Ok(VoteRule::AbsoluteMajority),
        "double_majority" => Ok(VoteRule::DoubleMajority),
        "unanimity" => Ok(VoteRule::Unanimity),
        _ => Err(AppError::bad_request_code(
            "errors.meetings.invalid_vote_type",
        )),
    }
}
