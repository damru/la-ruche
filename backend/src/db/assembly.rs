use chrono::{DateTime, Utc};
/// Repository for general meetings -- reference pattern for all other db/ modules.
///
/// Rules:
///   - No business logic here (quorums, calculations -> domain/)
///   - All queries verify building_id (tenant isolation)
///   - SQLx errors surface via ? -> AppError::Database via From
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    api::assemblies::{
        CreateMeetingInput, CreateResolutionInput, MeetingAttendanceEntryResponse,
        MeetingAttendanceInputEntry, MeetingAttendanceModeDto, MeetingAttendanceResponse,
        MeetingDocumentKindDto, MeetingDocumentResponse, MeetingResponse, MeetingTypeDto,
        VoteInput,
    },
    api::documents::DocumentListItem,
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
    agenda: Option<String>,
    minutes_document_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    resolution_count: i64,
    passed_count: i64,
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
            agenda: row.agenda,
            minutes_document_id: row.minutes_document_id,
            created_at: row.created_at,
            resolution_count: row.resolution_count,
            passed_count: row.passed_count,
        }
    }
}

fn meeting_document_kind_to_db(kind: &MeetingDocumentKindDto) -> &'static str {
    match kind {
        MeetingDocumentKindDto::Notice => "notice",
        MeetingDocumentKindDto::Attachment => "attachment",
    }
}

fn meeting_document_kind_from_db(kind: &str) -> MeetingDocumentKindDto {
    match kind {
        "attachment" => MeetingDocumentKindDto::Attachment,
        _ => MeetingDocumentKindDto::Notice,
    }
}

fn meeting_attendance_mode_to_db(mode: &MeetingAttendanceModeDto) -> &'static str {
    match mode {
        MeetingAttendanceModeDto::Present => "present",
        MeetingAttendanceModeDto::Proxy => "proxy",
        MeetingAttendanceModeDto::Correspondence => "correspondence",
        MeetingAttendanceModeDto::Absent => "absent",
    }
}

fn meeting_attendance_mode_from_db(mode: &str) -> MeetingAttendanceModeDto {
    match mode {
        "proxy" => MeetingAttendanceModeDto::Proxy,
        "correspondence" => MeetingAttendanceModeDto::Correspondence,
        "absent" => MeetingAttendanceModeDto::Absent,
        _ => MeetingAttendanceModeDto::Present,
    }
}

pub async fn list(pool: &PgPool, building_id: Uuid) -> AppResult<Vec<MeetingResponse>> {
    let rows: Vec<MeetingRow> = sqlx::query_as!(
        MeetingRow,
        r#"
        SELECT gm.id,
               gm.meeting_type::text as "meeting_type!",
               gm.status::text as "status!",
               gm.notice_date, gm.meeting_date, gm.location, gm.agenda,
               gm.minutes_document_id, gm.created_at,
               COUNT(r.id)                                       AS "resolution_count!",
               COUNT(r.id) FILTER (WHERE r.status = 'passed')    AS "passed_count!"
        FROM general_meeting gm
        LEFT JOIN resolution r ON r.meeting_id = gm.id
        WHERE gm.building_id = $1
        GROUP BY gm.id
        ORDER BY gm.meeting_date DESC
        "#,
        building_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn get(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<Option<MeetingResponse>> {
    let row = sqlx::query_as!(
        MeetingRow,
        r#"
        SELECT id, meeting_type::text as "meeting_type!", status::text as "status!",
               notice_date, meeting_date, location, agenda, minutes_document_id, created_at,
               0::bigint AS "resolution_count!", 0::bigint AS "passed_count!"
        FROM general_meeting
        WHERE id = $1 AND building_id = $2
        "#,
        id,
        building_id
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
        MeetingTypeDto::Annual => "annual",
        MeetingTypeDto::Extraordinary => "extraordinary",
    };

    let row = sqlx::query_as!(
        MeetingRow,
        r#"
        INSERT INTO general_meeting
            (building_id, meeting_type, meeting_date, location, agenda, created_by)
        VALUES ($1, $2::meeting_type, $3, $4, $5, $6)
        RETURNING id, meeting_type::text as "meeting_type!", status::text as "status!",
                  notice_date, meeting_date, location, agenda, minutes_document_id, created_at,
                  0::bigint AS "resolution_count!", 0::bigint AS "passed_count!"
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
    let meeting_type = match input.meeting_type {
        MeetingTypeDto::Annual => "annual",
        MeetingTypeDto::Extraordinary => "extraordinary",
    };

    let row = sqlx::query_as!(
        MeetingRow,
        r#"
        UPDATE general_meeting
        SET meeting_type = $3::meeting_type,
            meeting_date = $4,
            location = $5,
            agenda = $6,
            updated_at = NOW()
        WHERE id = $1
          AND building_id = $2
          AND status = 'draft'
        RETURNING id, meeting_type::text as "meeting_type!", status::text as "status!",
                  notice_date, meeting_date, location, agenda, minutes_document_id, created_at,
                  0::bigint AS "resolution_count!", 0::bigint AS "passed_count!"
        "#,
        id,
        building_id,
        meeting_type as _,
        input.meeting_date,
        input.location,
        input.agenda,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(Into::into))
}

pub async fn call(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<()> {
    let has_notice_document = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM general_meeting_document gmd
            JOIN general_meeting gm ON gm.id = gmd.meeting_id
            WHERE gm.id = $1
              AND gm.building_id = $2
              AND gmd.kind = 'notice'
        )
        "#,
        id,
        building_id,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);

    if !has_notice_document {
        return Err(AppError::Conflict(
            "Meeting notice cannot be sent without at least one notice document".into(),
        ));
    }

    let affected = sqlx::query!(
        r#"
        UPDATE general_meeting
        SET status = 'called', notice_date = CURRENT_DATE, updated_at = NOW()
        WHERE id = $1 AND building_id = $2 AND status = 'draft'
        "#,
        id,
        building_id
    )
    .execute(pool)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::Conflict(
            "Meeting not found or no longer in draft status".into(),
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
        id,
        building_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_documents(
    pool: &PgPool,
    meeting_id: Uuid,
    building_id: Uuid,
    roles: &[String],
) -> AppResult<Vec<MeetingDocumentResponse>> {
    let rows = sqlx::query!(
        r#"
        SELECT d.id,
               d.original_name,
               d.mime_type,
               d.size_bytes,
               d.visible_to_roles::text[] as visible_roles,
               d.created_at,
               gmd.kind::text as "kind!"
        FROM general_meeting_document gmd
        JOIN general_meeting gm ON gm.id = gmd.meeting_id
        JOIN document d ON d.id = gmd.document_id
        WHERE gmd.meeting_id = $1
          AND gm.building_id = $2
          AND d.building_id = $2
          AND d.visible_to_roles::text[] && $3
        ORDER BY CASE WHEN gmd.kind = 'notice' THEN 0 ELSE 1 END, d.created_at DESC
        "#,
        meeting_id,
        building_id,
        roles,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| MeetingDocumentResponse {
            document: DocumentListItem {
                id: row.id,
                original_name: row.original_name,
                mime_type: row.mime_type,
                size_bytes: row.size_bytes,
                visible_roles: row.visible_roles.unwrap_or_default(),
                linked_contract_names: Vec::new(),
                linked_report_titles: Vec::new(),
                linked_claim_titles: Vec::new(),
                linked_report_count: 0,
                created_at: row.created_at,
            },
            kind: meeting_document_kind_from_db(&row.kind),
        })
        .collect())
}

pub async fn attach_document(
    pool: &PgPool,
    meeting_id: Uuid,
    document_id: Uuid,
    kind: MeetingDocumentKindDto,
    building_id: Uuid,
) -> AppResult<bool> {
    let exists = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM general_meeting gm
            JOIN document d ON d.id = $2
            WHERE gm.id = $1
              AND gm.building_id = $3
              AND d.building_id = $3
        )
        "#,
        meeting_id,
        document_id,
        building_id,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);

    if !exists {
        return Ok(false);
    }

    sqlx::query!(
        r#"
        INSERT INTO general_meeting_document (meeting_id, document_id, kind)
        VALUES ($1, $2, $3::meeting_document_kind)
        ON CONFLICT (meeting_id, document_id) DO UPDATE
            SET kind = EXCLUDED.kind
        "#,
        meeting_id,
        document_id,
        meeting_document_kind_to_db(&kind) as _,
    )
    .execute(pool)
    .await?;

    Ok(true)
}

pub async fn detach_document(
    pool: &PgPool,
    meeting_id: Uuid,
    document_id: Uuid,
    building_id: Uuid,
) -> AppResult<bool> {
    let deleted = sqlx::query!(
        r#"
        DELETE FROM general_meeting_document gmd
        USING general_meeting gm
        WHERE gmd.meeting_id = $1
          AND gmd.document_id = $2
          AND gm.id = gmd.meeting_id
          AND gm.building_id = $3
        "#,
        meeting_id,
        document_id,
        building_id,
    )
    .execute(pool)
    .await?
    .rows_affected();

    Ok(deleted > 0)
}

pub async fn list_attendance(
    pool: &PgPool,
    meeting_id: Uuid,
    building_id: Uuid,
) -> AppResult<MeetingAttendanceResponse> {
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM general_meeting WHERE id = $1 AND building_id = $2)",
        meeting_id,
        building_id,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);

    if !exists {
        return Err(AppError::NotFound);
    }

    let rows = sqlx::query!(
        r#"
        SELECT u.id as unit_id,
               u.unit_number,
               CASE
                   WHEN ua.id IS NULL THEN NULL
                   ELSE ua.first_name || ' ' || ua.last_name
               END as owner_name,
               COALESCE(gma.mode::text, 'absent') as "mode!",
               gma.representative_name,
               COALESCE(gma.shares_represented, 0) as "shares_represented!"
        FROM unit u
        LEFT JOIN unit_owner uo
            ON uo.unit_id = u.id
           AND uo.end_date IS NULL
        LEFT JOIN user_account ua ON ua.id = uo.user_id
        LEFT JOIN general_meeting_attendance gma
            ON gma.unit_id = u.id
           AND gma.meeting_id = $1
        WHERE u.building_id = $2
          AND u.active = true
        ORDER BY u.unit_number
        "#,
        meeting_id,
        building_id,
    )
    .fetch_all(pool)
    .await?;

    let summary = sqlx::query!(
        r#"
        SELECT COALESCE(SUM(gma.shares_represented) FILTER (WHERE gma.mode != 'absent'), 0)::int as "shares_present!",
               COUNT(*) FILTER (WHERE gma.mode != 'absent')::int as "present_units!",
               b.shares_total as "total_shares!"
        FROM building b
        LEFT JOIN general_meeting_attendance gma ON gma.meeting_id = $1
        WHERE b.id = $2
        GROUP BY b.shares_total
        "#,
        meeting_id,
        building_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(MeetingAttendanceResponse {
        entries: rows
            .into_iter()
            .map(|row| MeetingAttendanceEntryResponse {
                unit_id: row.unit_id,
                unit_number: row.unit_number,
                owner_name: row.owner_name,
                mode: meeting_attendance_mode_from_db(&row.mode),
                representative_name: row.representative_name,
                shares_represented: row.shares_represented,
            })
            .collect(),
        shares_present: summary.shares_present,
        total_shares: summary.total_shares,
        present_units: summary.present_units,
    })
}

pub async fn save_attendance(
    pool: &PgPool,
    meeting_id: Uuid,
    building_id: Uuid,
    entries: Vec<MeetingAttendanceInputEntry>,
) -> AppResult<()> {
    let mut tx = pool.begin().await?;

    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM general_meeting WHERE id = $1 AND building_id = $2)",
        meeting_id,
        building_id,
    )
    .fetch_one(&mut *tx)
    .await?
    .unwrap_or(false);

    if !exists {
        return Err(AppError::NotFound);
    }

    sqlx::query!(
        "DELETE FROM general_meeting_attendance WHERE meeting_id = $1",
        meeting_id
    )
    .execute(&mut *tx)
    .await?;

    for entry in entries {
        let shares_represented = sqlx::query_scalar!(
            r#"
            SELECT uak.share_value
            FROM unit u
            JOIN unit_allocation_key uak
                ON uak.unit_id = u.id
               AND uak.valid_to IS NULL
            JOIN allocation_key ak
                ON ak.id = uak.key_id
               AND ak.key_type = 'general'
               AND ak.deleted_at IS NULL
            WHERE u.id = $1
              AND u.building_id = $2
              AND u.active = true
            "#,
            entry.unit_id,
            building_id,
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::bad_request_code("errors.meetings.unit_general_shares_missing"))?;

        sqlx::query!(
            r#"
            INSERT INTO general_meeting_attendance
                (meeting_id, unit_id, mode, representative_name, shares_represented)
            VALUES ($1, $2, $3::meeting_attendance_mode, $4, $5)
            "#,
            meeting_id,
            entry.unit_id,
            meeting_attendance_mode_to_db(&entry.mode) as _,
            entry.representative_name,
            shares_represented,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn list_resolutions(
    pool: &PgPool,
    meeting_id: Uuid,
    building_id: Uuid,
) -> AppResult<serde_json::Value> {
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM general_meeting WHERE id = $1 AND building_id = $2)",
        meeting_id,
        building_id
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

    Ok(serde_json::json!(
        resolutions
            .iter()
            .map(|r| serde_json::json!({
                "id": r.id,
                "number": r.number,
                "title": r.title,
                "description": r.description,
                "vote_type": r.vote_type,
                "status": r.status,
            }))
            .collect::<Vec<_>>()
    ))
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
        return Err(AppError::conflict_code(
            "errors.meetings.meeting_not_mutable",
        ));
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
        resolution_id,
        meeting_id,
        input.title,
        input.description,
        input.vote_type as _,
        building_id,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| {
        serde_json::json!({
            "id": r.id,
            "number": r.number,
            "title": r.title,
            "vote_type": r.vote_type,
            "status": r.status,
        })
    }))
}

pub async fn cast_vote(
    pool: &PgPool,
    _meeting_id: Uuid,
    resolution_id: Uuid,
    building_id: Uuid,
    user_id: Uuid,
    input: VoteInput,
) -> AppResult<()> {
    let shares =
        crate::db::allocation_key::get_unit_general_share(pool, input.unit_id, building_id)
            .await?
            .ok_or_else(|| {
                AppError::bad_request_code("errors.meetings.unit_general_shares_missing")
            })?;

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
        resolution_id,
        meeting_id,
        building_id
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
             COUNT(DISTINCT v.user_id)::int as present_owners,
             COUNT(DISTINCT v.user_id) FILTER (WHERE v.position = 'in_favor')::int as "favorable_owners!"
        FROM building b
        JOIN unit u ON u.building_id = b.id
        LEFT JOIN user_building ub ON ub.building_id = b.id
            AND 'co_owner' = ANY(ub.roles::text[])
        LEFT JOIN vote v ON v.resolution_id = $1
        WHERE b.id = $2
        GROUP BY b.shares_total
        "#,
        resolution_id,
        building_id
    )
    .fetch_one(pool)
    .await?;

    let attendance = sqlx::query!(
        r#"
        SELECT COALESCE(SUM(shares_represented) FILTER (WHERE mode != 'absent'), 0)::int as "shares_present!",
               COUNT(*) FILTER (WHERE mode != 'absent')::int as "present_units!"
        FROM general_meeting_attendance
        WHERE meeting_id = $1
        "#,
        meeting_id,
    )
    .fetch_one(pool)
    .await?;

    let mut for_votes = 0i32;
    let mut against_votes = 0i32;
    let mut abstentions = 0i32;
    let mut shares_present = 0i32;

    for v in &votes {
        shares_present += v.shares_represented;
        match v.position.as_str() {
            "in_favor" => for_votes += v.shares_represented,
            "against" => against_votes += v.shares_represented,
            _ => abstentions += v.shares_represented,
        }
    }

    let tally = VoteTally {
        shares_total: building.shares_total,
        shares_present: if attendance.shares_present > 0 {
            attendance.shares_present
        } else {
            shares_present
        },
        total_owners: building.total_owners.unwrap_or(0) as u32,
        present_owners: if attendance.present_units > 0 {
            attendance.present_units as u32
        } else {
            building.present_owners.unwrap_or(0) as u32
        },
        favorable_owners: building.favorable_owners as u32,
        for_votes,
        against_votes,
        abstentions,
    };

    Ok((
        ResolutionInfo {
            vote_type: resolution.vote_type,
        },
        tally,
    ))
}
