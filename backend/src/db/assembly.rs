/// Repository for general meetings -- reference pattern for all other db/ modules.
///
/// Rules:
///   - No business logic here (quorums, calculations -> domain/)
///   - All queries verify building_id (tenant isolation)
///   - SQLx errors surface via ? -> AppError::Database via From

use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::{
    api::assemblies::{MeetingResponse, CreateMeetingInput, CreateResolutionInput, MeetingTypeDto, VoteInput},
    domain::vote::VoteTally,
    errors::{AppError, AppResult},
};

#[derive(Debug)]
struct MeetingRow {
    id: Uuid,
    meeting_type: String,
    status: String,
    notice_date: Option<chrono::NaiveDate>,
    meeting_date: DateTime<Utc>,
    location: Option<String>,
    created_at: DateTime<Utc>,
}

impl From<MeetingRow> for MeetingResponse {
    fn from(row: MeetingRow) -> Self {
        MeetingResponse {
            id: row.id,
            meeting_type: match row.meeting_type.as_str() {
                "extraordinary" => MeetingTypeDto::Extraordinary,
                _ => MeetingTypeDto::Annual,
            },
            status: row.status,
            notice_date: row.notice_date,
            meeting_date: row.meeting_date,
            location: row.location,
            created_at: row.created_at,
        }
    }
}

pub async fn list(pool: &PgPool, building_id: Uuid) -> AppResult<Vec<MeetingResponse>> {
    let rows = sqlx::query_as!(
        MeetingRow,
        r#"
        SELECT id, meeting_type::text as "meeting_type!", status::text as "status!",
               notice_date, meeting_date, location, created_at
        FROM general_meeting
        WHERE building_id = $1
        ORDER BY meeting_date DESC
        "#,
        building_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn get(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
) -> AppResult<Option<MeetingResponse>> {
    let row = sqlx::query_as!(
        MeetingRow,
        r#"
        SELECT id, meeting_type::text as "meeting_type!", status::text as "status!",
               notice_date, meeting_date, location, created_at
        FROM general_meeting
        WHERE id = $1 AND building_id = $2
        "#,
        id, building_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(Into::into))
}

pub async fn create(
    pool: &PgPool,
    building_id: Uuid,
    created_by: Uuid,
    input: CreateMeetingInput,
) -> AppResult<MeetingResponse> {
    let meeting_type = match input.meeting_type {
        MeetingTypeDto::Annual        => "annual",
        MeetingTypeDto::Extraordinary => "extraordinary",
    };

    let row = sqlx::query_as!(
        MeetingRow,
        r#"
        INSERT INTO general_meeting
            (building_id, meeting_type, meeting_date, location, agenda, created_by)
        VALUES ($1, $2::meeting_type, $3, $4, $5, $6)
        RETURNING id, meeting_type::text as "meeting_type!", status::text as "status!",
                  notice_date, meeting_date, location, created_at
        "#,
        building_id,
        meeting_type as _,
        input.meeting_date,
        input.location,
        input.agenda,
        created_by,
    )
    .fetch_one(pool)
    .await?;

    Ok(row.into())
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    input: CreateMeetingInput,
) -> AppResult<Option<MeetingResponse>> {
    let row = sqlx::query_as!(
        MeetingRow,
        r#"
        UPDATE general_meeting
        SET meeting_date = $3,
            location = $4,
            agenda = $5,
            updated_at = NOW()
        WHERE id = $1
          AND building_id = $2
          AND status = 'draft'
        RETURNING id, meeting_type::text as "meeting_type!", status::text as "status!",
                  notice_date, meeting_date, location, created_at
        "#,
        id, building_id,
        input.meeting_date, input.location, input.agenda,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(Into::into))
}

pub async fn call(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<()> {
    let affected = sqlx::query!(
        r#"
        UPDATE general_meeting
        SET status = 'called', notice_date = CURRENT_DATE, updated_at = NOW()
        WHERE id = $1 AND building_id = $2 AND status = 'draft'
        "#,
        id, building_id
    )
    .execute(pool)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::Conflict(
            "Meeting not found or no longer in draft status".into()
        ));
    }
    Ok(())
}

pub async fn close(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<()> {
    sqlx::query!(
        r#"
        UPDATE general_meeting
        SET status = 'closed', updated_at = NOW()
        WHERE id = $1 AND building_id = $2 AND status = 'held'
        "#,
        id, building_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_resolutions(
    pool: &PgPool,
    meeting_id: Uuid,
    building_id: Uuid,
) -> AppResult<serde_json::Value> {
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM general_meeting WHERE id = $1 AND building_id = $2)",
        meeting_id, building_id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);

    if !exists {
        return Err(AppError::NotFound);
    }

    let resolutions = sqlx::query!(
        r#"
        SELECT id, number, title, description,
               vote_type::text as vote_type, status::text as status
        FROM resolution
        WHERE meeting_id = $1
        ORDER BY number
        "#,
        meeting_id
    )
    .fetch_all(pool)
    .await?;

    Ok(serde_json::json!(resolutions.iter().map(|r| serde_json::json!({
        "id": r.id,
        "number": r.number,
        "title": r.title,
        "description": r.description,
        "vote_type": r.vote_type,
        "status": r.status,
    })).collect::<Vec<_>>()))
}

pub async fn create_resolution(
    pool: &PgPool,
    meeting_id: Uuid,
    building_id: Uuid,
    input: CreateResolutionInput,
) -> AppResult<serde_json::Value> {
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM general_meeting WHERE id = $1 AND building_id = $2 AND status IN ('draft','called'))",
        meeting_id, building_id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);

    if !exists {
        return Err(AppError::Conflict("Meeting not found or already closed".into()));
    }

    let row = sqlx::query!(
        r#"
        INSERT INTO resolution (meeting_id, number, title, description, vote_type)
        VALUES ($1, $2, $3, $4, $5::vote_type)
        RETURNING id, number, title, vote_type::text as vote_type, status::text as status
        "#,
        meeting_id,
        input.number,
        input.title,
        input.description,
        input.vote_type as _,
    )
    .fetch_one(pool)
    .await?;

    Ok(serde_json::json!({
        "id": row.id,
        "number": row.number,
        "title": row.title,
        "vote_type": row.vote_type,
        "status": row.status,
    }))
}

pub async fn update_resolution(
    pool: &PgPool,
    meeting_id: Uuid,
    resolution_id: Uuid,
    building_id: Uuid,
    input: CreateResolutionInput,
) -> AppResult<Option<serde_json::Value>> {
    let row = sqlx::query!(
        r#"
        UPDATE resolution
        SET title = $3, description = $4, vote_type = $5::vote_type
        FROM general_meeting gm
        WHERE resolution.id = $1
          AND resolution.meeting_id = $2
          AND gm.id = resolution.meeting_id
          AND gm.building_id = $6
        RETURNING resolution.id, resolution.number, resolution.title,
                  resolution.vote_type::text as vote_type, resolution.status::text as status
        "#,
        resolution_id, meeting_id,
        input.title, input.description, input.vote_type as _,
        building_id,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| serde_json::json!({
        "id": r.id,
        "number": r.number,
        "title": r.title,
        "vote_type": r.vote_type,
        "status": r.status,
    })))
}

pub async fn cast_vote(
    pool: &PgPool,
    meeting_id: Uuid,
    resolution_id: Uuid,
    building_id: Uuid,
    user_id: Uuid,
    input: VoteInput,
) -> AppResult<()> {
    let shares = sqlx::query_scalar!(
        r#"
        SELECT uak.share_value
        FROM unit u
        JOIN unit_allocation_key uak ON uak.unit_id = u.id
        JOIN allocation_key ak ON ak.id = uak.key_id AND ak.name = 'General'
        WHERE u.id = $1 AND u.building_id = $2
        "#,
        input.unit_id, building_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::BadRequest("Unit not found or has no general shares".into()))?;

    sqlx::query!(
        r#"
        INSERT INTO vote
            (resolution_id, user_id, unit_id, position, shares_represented, proxy_for)
        VALUES ($1, $2, $3, $4::vote_position, $5, $6)
        ON CONFLICT (resolution_id, unit_id) DO UPDATE
            SET position = EXCLUDED.position,
                user_id = EXCLUDED.user_id,
                proxy_for = EXCLUDED.proxy_for
        "#,
        resolution_id,
        user_id,
        input.unit_id,
        input.position as _,
        shares,
        input.proxy_for,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub struct ResolutionInfo {
    pub vote_type: String,
}

pub async fn get_tally(
    pool: &PgPool,
    meeting_id: Uuid,
    resolution_id: Uuid,
    building_id: Uuid,
) -> AppResult<(ResolutionInfo, VoteTally)> {
    let resolution = sqlx::query!(
        r#"
        SELECT r.vote_type::text as "vote_type!"
        FROM resolution r
        JOIN general_meeting gm ON gm.id = r.meeting_id
        WHERE r.id = $1 AND r.meeting_id = $2 AND gm.building_id = $3
        "#,
        resolution_id, meeting_id, building_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let votes = sqlx::query!(
        r#"
        SELECT position::text as "position!", shares_represented
        FROM vote WHERE resolution_id = $1
        "#,
        resolution_id
    )
    .fetch_all(pool)
    .await?;

    let building = sqlx::query!(
        r#"
        SELECT b.shares_total,
               COUNT(DISTINCT ub.user_id)::int as total_owners,
               COUNT(DISTINCT v.user_id)::int as present_owners
        FROM building b
        JOIN unit u ON u.building_id = b.id
        LEFT JOIN user_building ub ON ub.building_id = b.id
            AND 'co_owner' = ANY(ub.roles::text[])
        LEFT JOIN vote v ON v.resolution_id = $1
        WHERE b.id = $2
        GROUP BY b.shares_total
        "#,
        resolution_id, building_id
    )
    .fetch_one(pool)
    .await?;

    use rust_decimal::Decimal;

    let mut for_votes   = Decimal::ZERO;
    let mut against_votes = Decimal::ZERO;
    let mut abstentions = Decimal::ZERO;
    let mut shares_present = Decimal::ZERO;

    for v in &votes {
        shares_present += v.shares_represented;
        match v.position.as_str() {
            "in_favor" => for_votes     += v.shares_represented,
            "against"  => against_votes += v.shares_represented,
            _          => abstentions   += v.shares_represented,
        }
    }

    let tally = VoteTally {
        shares_total:   building.shares_total,
        shares_present,
        total_owners:   building.total_owners.unwrap_or(0) as u32,
        present_owners: building.present_owners.unwrap_or(0) as u32,
        for_votes,
        against_votes,
        abstentions,
    };

    Ok((ResolutionInfo { vote_type: resolution.vote_type }, tally))
}
