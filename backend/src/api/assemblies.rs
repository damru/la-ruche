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
    domain::{AuthUser, Role, vote::VoteRule},
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
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateResolutionInput {
    #[validate(length(min = 1, max = 500))]
    pub title: String,
    pub description: Option<String>,
    pub vote_type: String, // "simple_majority" | "absolute_majority" | "double_majority" | "unanimity"
    pub number: i32,
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
) -> AppResult<StatusCode> {
    auth.require_role(&Role::Manager)?;
    crate::db::assembly::call(&state.pool, id, auth.building_id).await?;
    Ok(StatusCode::NO_CONTENT)
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

pub async fn vote(
    auth: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((meeting_id, resolution_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<VoteInput>,
) -> AppResult<StatusCode> {
    auth.require_any_role(&[Role::Manager, Role::BoardMember, Role::CoOwner])?;

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
