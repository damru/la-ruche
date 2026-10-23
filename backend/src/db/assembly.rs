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
        CallMeetingInput, ConvocationChannelDto, CreateMeetingInput, CreateResolutionInput,
        MeetingAttendanceEntryResponse, MeetingAttendanceInputEntry, MeetingAttendanceModeDto,
        MeetingAttendanceResponse, MeetingDocumentKindDto, MeetingDocumentResponse,
        MeetingResponse, MeetingTypeDto, ScrutineerEntry, ScrutineerSignature, SignMinutesInput,
        StartMeetingInput, VoteInput,
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
    president_name: Option<String>,
    president_user_id: Option<Uuid>,
    secretary_name: Option<String>,
    secretary_user_id: Option<Uuid>,
    scrutineers: serde_json::Value,
    minutes_notified_at: Option<DateTime<Utc>>,
    convocation_sent_at: Option<DateTime<Utc>>,
    convocation_channel: Option<String>,
    minutes_president_signed_at: Option<DateTime<Utc>>,
    minutes_secretary_signed_at: Option<DateTime<Utc>>,
    minutes_scrutineers_signed: serde_json::Value,
}

impl From<MeetingRow> for MeetingResponse {
    fn from(row: MeetingRow) -> Self {
        let scrutineers: Vec<ScrutineerEntry> =
            serde_json::from_value(row.scrutineers).unwrap_or_default();
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
            president_name: row.president_name,
            president_user_id: row.president_user_id,
            secretary_name: row.secretary_name,
            secretary_user_id: row.secretary_user_id,
            scrutineers,
            minutes_notified_at: row.minutes_notified_at,
            convocation_sent_at: row.convocation_sent_at,
            convocation_channel: row.convocation_channel,
            minutes_president_signed_at: row.minutes_president_signed_at,
            minutes_secretary_signed_at: row.minutes_secretary_signed_at,
            minutes_scrutineers_signed: serde_json::from_value(row.minutes_scrutineers_signed)
                .unwrap_or_default(),
        }
    }
}

fn convocation_channel_to_db(c: &ConvocationChannelDto) -> &'static str {
    match c {
        ConvocationChannelDto::Lrar => "lrar",
        ConvocationChannelDto::HandDelivery => "hand_delivery",
        ConvocationChannelDto::Email => "email",
    }
}

fn meeting_document_kind_to_db(kind: &MeetingDocumentKindDto) -> &'static str {
    match kind {
        MeetingDocumentKindDto::Notice => "notice",
        MeetingDocumentKindDto::Attachment => "attachment",
        MeetingDocumentKindDto::VoteByMailForm => "vote_by_mail_form",
    }
}

fn meeting_document_kind_from_db(kind: &str) -> MeetingDocumentKindDto {
    match kind {
        "attachment" => MeetingDocumentKindDto::Attachment,
        "vote_by_mail_form" => MeetingDocumentKindDto::VoteByMailForm,
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
               gm.president_name, gm.president_user_id,
               gm.secretary_name, gm.secretary_user_id,
               gm.scrutineers as "scrutineers!: serde_json::Value",
               gm.minutes_notified_at,
               gm.convocation_sent_at,
               gm.convocation_channel::text as convocation_channel,
               gm.minutes_president_signed_at,
               gm.minutes_secretary_signed_at,
               gm.minutes_scrutineers_signed as "minutes_scrutineers_signed!: serde_json::Value",
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
               president_name, president_user_id, secretary_name, secretary_user_id,
               scrutineers as "scrutineers!: serde_json::Value",
               minutes_notified_at,
               convocation_sent_at,
               convocation_channel::text as convocation_channel,
               minutes_president_signed_at,
               minutes_secretary_signed_at,
               minutes_scrutineers_signed as "minutes_scrutineers_signed!: serde_json::Value",
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
                  president_name, president_user_id, secretary_name, secretary_user_id,
                  scrutineers as "scrutineers!: serde_json::Value",
                  minutes_notified_at,
                  convocation_sent_at,
                  convocation_channel::text as convocation_channel,
                  minutes_president_signed_at,
                  minutes_secretary_signed_at,
                  minutes_scrutineers_signed as "minutes_scrutineers_signed!: serde_json::Value",
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
                  president_name, president_user_id, secretary_name, secretary_user_id,
                  scrutineers as "scrutineers!: serde_json::Value",
                  minutes_notified_at,
                  convocation_sent_at,
                  convocation_channel::text as convocation_channel,
                  minutes_president_signed_at,
                  minutes_secretary_signed_at,
                  minutes_scrutineers_signed as "minutes_scrutineers_signed!: serde_json::Value",
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

/// Deletes a draft meeting and its children (votes, resolutions, attendance,
/// documents) atomically. The `status = 'draft'` SQL filter yields no row for
/// a non-draft, foreign or unknown meeting -> `Ok(false)` -> 404.
pub async fn delete_meeting(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<bool> {
    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"
        DELETE FROM vote v
        USING resolution r, general_meeting gm
        WHERE v.resolution_id = r.id
          AND r.meeting_id = $1
          AND gm.id = r.meeting_id
          AND gm.building_id = $2
          AND gm.status = 'draft'
        "#,
        id,
        building_id,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM resolution r
        USING general_meeting gm
        WHERE r.meeting_id = $1
          AND gm.id = r.meeting_id
          AND gm.building_id = $2
          AND gm.status = 'draft'
        "#,
        id,
        building_id,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM general_meeting_attendance a
        USING general_meeting gm
        WHERE a.meeting_id = $1
          AND gm.id = a.meeting_id
          AND gm.building_id = $2
          AND gm.status = 'draft'
        "#,
        id,
        building_id,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM general_meeting_document d
        USING general_meeting gm
        WHERE d.meeting_id = $1
          AND gm.id = d.meeting_id
          AND gm.building_id = $2
          AND gm.status = 'draft'
        "#,
        id,
        building_id,
    )
    .execute(&mut *tx)
    .await?;

    let deleted = sqlx::query!(
        r#"
        DELETE FROM general_meeting
        WHERE id = $1 AND building_id = $2 AND status = 'draft'
        "#,
        id,
        building_id,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();

    tx.commit().await?;
    Ok(deleted > 0)
}

pub async fn call(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    input: CallMeetingInput,
) -> AppResult<()> {
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
        return Err(AppError::conflict_code(
            "errors.meetings.notice_document_required",
        ));
    }

    let channel = convocation_channel_to_db(&input.channel);
    let sent_at = input.sent_at.unwrap_or_else(chrono::Utc::now);

    let affected = sqlx::query!(
        r#"
        UPDATE general_meeting
        SET status = 'called',
            notice_date = CURRENT_DATE,
            convocation_sent_at = $3,
            convocation_channel = $4::meeting_convocation_channel,
            updated_at = NOW()
        WHERE id = $1 AND building_id = $2 AND status = 'draft'
        "#,
        id,
        building_id,
        sent_at,
        channel as _,
    )
    .execute(pool)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::conflict_code(
            "errors.meetings.not_in_draft_status",
        ));
    }
    Ok(())
}

///Evaluates the convocation completeness checklist for an AG.
pub async fn load_convocation_checklist(
    pool: &PgPool,
    meeting_id: Uuid,
    building_id: Uuid,
) -> AppResult<crate::domain::convocation_checklist::ConvocationChecklist> {
    use crate::domain::convocation_checklist::{ConvocationContext, evaluate};

    let row = sqlx::query!(
        r#"
        SELECT meeting_date,
               agenda,
               convocation_sent_at,
               convocation_channel::text as convocation_channel,
               (SELECT COUNT(*)::int FROM general_meeting_document
                WHERE meeting_id = $1 AND kind = 'notice')             as "notice_count!",
               (SELECT COUNT(*)::int FROM resolution
                WHERE meeting_id = $1)                                  as "resolution_count!",
               (SELECT COUNT(*)::int FROM general_meeting_document
                WHERE meeting_id = $1 AND kind = 'vote_by_mail_form')  as "vote_by_mail_form_count!"
        FROM general_meeting
        WHERE id = $1 AND building_id = $2
        "#,
        meeting_id,
        building_id,
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(evaluate(&ConvocationContext {
        meeting_date: row.meeting_date,
        now: chrono::Utc::now(),
        agenda: row.agenda,
        notice_documents_count: row.notice_count as usize,
        resolutions_count: row.resolution_count as usize,
        convocation_sent_at: row.convocation_sent_at,
        convocation_channel: row.convocation_channel,
        has_vote_by_mail_form: row.vote_by_mail_form_count > 0,
    }))
}

pub async fn close(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<()> {
    let affected = sqlx::query!(
        r#"
        UPDATE general_meeting
        SET status = 'closed', updated_at = NOW()
        WHERE id = $1 AND building_id = $2 AND status = 'ongoing'
        "#,
        id,
        building_id
    )
    .execute(pool)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::conflict_code(
            "errors.meetings.not_in_ongoing_status",
        ));
    }

    Ok(())
}

/// Guard : meeting `status = 'closed'` ET `minutes_document_id` NULL.
pub async fn attach_signed_minutes(
    pool: &PgPool,
    meeting_id: Uuid,
    document_id: Uuid,
    building_id: Uuid,
) -> AppResult<()> {
    let document_exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM document WHERE id = $1 AND building_id = $2)",
        document_id,
        building_id,
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(false);

    if !document_exists {
        return Err(AppError::NotFound);
    }

    let affected = sqlx::query!(
        r#"
        UPDATE general_meeting
        SET minutes_document_id = $3, updated_at = NOW()
        WHERE id = $1
          AND building_id = $2
          AND status = 'closed'
          AND minutes_document_id IS NULL
        "#,
        meeting_id,
        building_id,
        document_id,
    )
    .execute(pool)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::conflict_code(
            "errors.meetings.minutes_already_attached",
        ));
    }

    Ok(())
}

/// Requires an attached signed PV (`minutes_document_id IS NOT NULL`).
/// Idempotent: re-applies NOW() timestamp on each call for checked roles.
pub async fn sign_minutes(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    input: SignMinutesInput,
) -> AppResult<()> {
    let now = chrono::Utc::now();
    let scrutineers_signed: Vec<ScrutineerSignature> = input
        .scrutineers_signed
        .into_iter()
        .map(|name| ScrutineerSignature {
            name,
            signed_at: now,
        })
        .collect();
    let scrutineers_json = serde_json::to_value(&scrutineers_signed)
        .map_err(|_| AppError::validation_code("errors.meetings.signatures_invalid"))?;

    let president_at = if input.president_signed {
        Some(now)
    } else {
        None
    };
    let secretary_at = if input.secretary_signed {
        Some(now)
    } else {
        None
    };

    let affected = sqlx::query!(
        r#"
        UPDATE general_meeting
        SET minutes_president_signed_at = COALESCE($3, minutes_president_signed_at),
            minutes_secretary_signed_at = COALESCE($4, minutes_secretary_signed_at),
            minutes_scrutineers_signed = $5::jsonb,
            updated_at = NOW()
        WHERE id = $1
          AND building_id = $2
          AND status = 'closed'
          AND minutes_document_id IS NOT NULL
        "#,
        id,
        building_id,
        president_at,
        secretary_at,
        scrutineers_json,
    )
    .execute(pool)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::conflict_code(
            "errors.meetings.minutes_not_signed",
        ));
    }
    Ok(())
}

pub async fn notify_minutes(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<()> {
    let affected = sqlx::query!(
        r#"
        UPDATE general_meeting
        SET minutes_notified_at = NOW(), updated_at = NOW()
        WHERE id = $1
          AND building_id = $2
          AND status = 'closed'
          AND minutes_document_id IS NOT NULL
        "#,
        id,
        building_id,
    )
    .execute(pool)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::conflict_code(
            "errors.meetings.minutes_not_signed",
        ));
    }
    Ok(())
}

/// Loads all data needed to render the draft PV of a closed AG.
/// Returns a 409 error if the meeting is not in `closed` status.
pub async fn load_minutes_context(
    pool: &PgPool,
    meeting_id: Uuid,
    building_id: Uuid,
) -> AppResult<crate::domain::minutes_draft::MinutesContext> {
    use crate::domain::minutes_draft::{
        MinutesAttendanceLine, MinutesContext, MinutesResolution, MinutesSignature,
        MinutesVoteDetail,
    };

    let meeting = sqlx::query!(
        r#"
        SELECT gm.meeting_type::text as "meeting_type!",
               gm.status::text as "status!",
               gm.meeting_date,
               gm.location,
               gm.agenda,
               gm.president_name,
               gm.secretary_name,
               gm.scrutineers as "scrutineers!: serde_json::Value",
               gm.minutes_president_signed_at,
               gm.minutes_secretary_signed_at,
               gm.minutes_scrutineers_signed as "minutes_scrutineers_signed!: serde_json::Value",
               b.name as building_name,
               b.address_line1,
               b.postal_code,
               b.city,
               b.shares_total
        FROM general_meeting gm
        JOIN building b ON b.id = gm.building_id
        WHERE gm.id = $1 AND gm.building_id = $2
        "#,
        meeting_id,
        building_id,
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    if meeting.status != "closed" {
        return Err(AppError::conflict_code(
            "errors.meetings.not_in_closed_status",
        ));
    }

    let resolutions = sqlx::query!(
        r#"
        SELECT r.id, r.number, r.title, r.description, r.vote_type::text as "vote_type!",
               r.status::text as "status!"
        FROM resolution r
        WHERE r.meeting_id = $1
        ORDER BY r.number
        "#,
        meeting_id,
    )
    .fetch_all(pool)
    .await?;

    let mut minutes_resolutions: Vec<MinutesResolution> = Vec::with_capacity(resolutions.len());
    for r in resolutions {
        let resolution_id = r.id;

        let (_info, tally) = get_tally(pool, meeting_id, resolution_id, building_id).await?;

        let vote_detail_rows = sqlx::query!(
            r#"
            SELECT u.unit_number,
                   COALESCE(ua.first_name || ' ' || ua.last_name, '—') as "owner_name!",
                   v.position::text as "position!",
                   v.shares_represented as "shares!"
            FROM vote v
            JOIN unit u ON u.id = v.unit_id
            LEFT JOIN unit_owner uo ON uo.unit_id = u.id AND uo.end_date IS NULL
            LEFT JOIN user_account ua ON ua.id = uo.user_id
            WHERE v.resolution_id = $1
              AND v.position::text != 'in_favor'
            ORDER BY u.unit_number
            "#,
            resolution_id,
        )
        .fetch_all(pool)
        .await?;

        let rule = match r.vote_type.as_str() {
            "simple_majority" => crate::domain::vote::VoteRule::SimpleMajority,
            "absolute_majority" => crate::domain::vote::VoteRule::AbsoluteMajority,
            "double_majority" => crate::domain::vote::VoteRule::DoubleMajority,
            _ => crate::domain::vote::VoteRule::Unanimity,
        };
        let result = tally.compute(&rule);

        minutes_resolutions.push(MinutesResolution {
            number: r.number,
            title: r.title,
            description: r.description,
            vote_type: r.vote_type,
            status: r.status,
            for_votes: result.for_votes,
            against_votes: result.against_votes,
            abstentions: result.abstentions,
            threshold: result.threshold,
            eligible_for_second_vote: result.eligible_for_second_vote.as_ref().map(|s| match s {
                crate::domain::vote::SecondVoteType::Art25_1 => "art_25_1".to_string(),
                crate::domain::vote::SecondVoteType::Art26_1 => "art_26_1".to_string(),
            }),
            reservations: vec![],
            vote_details: vote_detail_rows
                .into_iter()
                .map(|v| MinutesVoteDetail {
                    owner_name: v.owner_name,
                    unit_number: v.unit_number,
                    shares: v.shares,
                    position: v.position,
                })
                .collect(),
        });
    }

    let attendance_rows = sqlx::query!(
        r#"
        SELECT u.unit_number,
               CASE WHEN ua.id IS NULL THEN NULL
                    ELSE ua.first_name || ' ' || ua.last_name
               END as owner_name,
               gma.representative_name,
               gma.mode::text as "mode!",
               gma.shares_represented as "shares!"
        FROM general_meeting_attendance gma
        JOIN unit u ON u.id = gma.unit_id
        LEFT JOIN unit_owner uo ON uo.unit_id = u.id AND uo.end_date IS NULL
        LEFT JOIN user_account ua ON ua.id = uo.user_id
        WHERE gma.meeting_id = $1
        ORDER BY u.unit_number
        "#,
        meeting_id,
    )
    .fetch_all(pool)
    .await?;

    let scrutineers: Vec<String> =
        serde_json::from_value::<Vec<serde_json::Value>>(meeting.scrutineers)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|s| {
                s.get("name")
                    .and_then(|n| n.as_str())
                    .map(|s| s.to_string())
            })
            .collect();

    let scrutineers_signed: Vec<MinutesSignature> =
        serde_json::from_value::<Vec<serde_json::Value>>(meeting.minutes_scrutineers_signed)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|s| {
                let name = s.get("name").and_then(|n| n.as_str())?.to_string();
                let signed_at = s
                    .get("signed_at")
                    .and_then(|t| t.as_str())
                    .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                    .map(|t| t.with_timezone(&chrono::Utc))?;
                Some(MinutesSignature { name, signed_at })
            })
            .collect();

    Ok(MinutesContext {
        building_name: meeting.building_name,
        building_address: format!(
            "{}, {} {}",
            meeting.address_line1, meeting.postal_code, meeting.city
        ),
        meeting_type: meeting.meeting_type,
        meeting_date: meeting.meeting_date,
        closing_time: None,
        location: meeting.location,
        agenda: meeting.agenda,
        total_building_shares: Some(meeting.shares_total),
        proxy_distribution: None,
        technical_incidents: vec![],
        president_name: meeting.president_name,
        secretary_name: meeting.secretary_name,
        scrutineers,
        resolutions: minutes_resolutions,
        attendance: attendance_rows
            .into_iter()
            .map(|a| MinutesAttendanceLine {
                unit_number: a.unit_number,
                owner_name: a.owner_name,
                mode: a.mode,
                shares: a.shares,
                representative_name: a.representative_name,
            })
            .collect(),
        president_signed_at: meeting.minutes_president_signed_at,
        secretary_signed_at: meeting.minutes_secretary_signed_at,
        scrutineers_signed,
    })
}

pub async fn list_documents(
    pool: &PgPool,
    meeting_id: Uuid,
    building_id: Uuid,
    roles: &[String],
) -> AppResult<Vec<MeetingDocumentResponse>> {
    #[derive(sqlx::FromRow)]
    struct Row {
        id: Uuid,
        original_name: String,
        mime_type: String,
        size_bytes: i64,
        visible_roles: Option<Vec<String>>,
        category: String,
        supersedes_id: Option<Uuid>,
        is_current: bool,
        created_at: chrono::DateTime<chrono::Utc>,
        kind: String,
    }

    let rows = sqlx::query_as::<_, Row>(
        r#"
        SELECT d.id,
               d.original_name,
               d.mime_type,
               d.size_bytes,
               d.visible_to_roles::text[] as visible_roles,
               d.category::text as category,
               d.supersedes_id,
               d.is_current,
               d.created_at,
               gmd.kind::text as kind
        FROM general_meeting_document gmd
        JOIN general_meeting gm ON gm.id = gmd.meeting_id
        JOIN document d ON d.id = gmd.document_id
        WHERE gmd.meeting_id = $1
          AND gm.building_id = $2
          AND d.building_id = $2
          AND d.deleted_at IS NULL
          AND d.visible_to_roles::text[] && $3
        ORDER BY CASE WHEN gmd.kind = 'notice' THEN 0 ELSE 1 END, d.created_at DESC
        "#,
    )
    .bind(meeting_id)
    .bind(building_id)
    .bind(roles)
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
                category: row.category,
                supersedes_id: row.supersedes_id,
                is_current: row.is_current,
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

pub async fn get_meeting_status(
    pool: &PgPool,
    meeting_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<Option<String>> {
    Ok(sqlx::query_scalar!(
        "SELECT status::text FROM general_meeting WHERE id = $1 AND building_id = $2",
        meeting_id,
        building_id
    )
    .fetch_optional(pool)
    .await?
    .flatten())
}

pub async fn list_attendance(
    pool: &PgPool,
    meeting_id: Uuid,
    building_id: Uuid,
) -> AppResult<MeetingAttendanceResponse> {
    let status = sqlx::query_scalar!(
        "SELECT status::text FROM general_meeting WHERE id = $1 AND building_id = $2",
        meeting_id,
        building_id,
    )
    .fetch_optional(pool)
    .await?
    .flatten()
    .ok_or(AppError::NotFound)?;

    // ongoing/closed: drive from attendance table (list frozen, archived units included as-is).
    // others: drive from unit table (active only), and always resolve shares from
    // the current active general allocation key at query time.
    let entries: Vec<MeetingAttendanceEntryResponse> = if status == "ongoing" || status == "closed"
    {
        sqlx::query!(
            r#"
            SELECT u.id as unit_id,
                   u.unit_number,
                   CASE WHEN ua.id IS NULL THEN NULL
                        ELSE ua.first_name || ' ' || ua.last_name
                   END as owner_name,
                   gma.mode::text as "mode!",
                   gma.representative_name,
                   gma.shares_represented as "shares_represented!"
            FROM general_meeting_attendance gma
            JOIN unit u ON u.id = gma.unit_id
            LEFT JOIN unit_owner uo ON uo.unit_id = u.id AND uo.end_date IS NULL
            LEFT JOIN user_account ua ON ua.id = uo.user_id
            WHERE gma.meeting_id = $1
            ORDER BY u.unit_number
            "#,
            meeting_id,
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| MeetingAttendanceEntryResponse {
            unit_id: row.unit_id,
            unit_number: row.unit_number,
            owner_name: row.owner_name,
            mode: meeting_attendance_mode_from_db(&row.mode),
            representative_name: row.representative_name,
            shares_represented: row.shares_represented,
        })
        .collect()
    } else {
        sqlx::query!(
            r#"
            SELECT u.id as unit_id,
                   u.unit_number,
                   CASE WHEN ua.id IS NULL THEN NULL
                        ELSE ua.first_name || ' ' || ua.last_name
                   END as owner_name,
                   COALESCE(gma.mode::text, 'absent') as "mode!",
                   gma.representative_name,
                   COALESCE(uak.share_value, 0) as "shares_represented!"
            FROM unit u
            LEFT JOIN unit_owner uo ON uo.unit_id = u.id AND uo.end_date IS NULL
            LEFT JOIN user_account ua ON ua.id = uo.user_id
            LEFT JOIN general_meeting_attendance gma
                ON gma.unit_id = u.id AND gma.meeting_id = $1
            LEFT JOIN unit_allocation_key uak
                ON uak.unit_id = u.id
               AND uak.valid_to IS NULL
               AND EXISTS (
                   SELECT 1
                   FROM allocation_key ak
                   WHERE ak.id = uak.key_id
                     AND ak.key_type = 'general'
                     AND ak.deleted_at IS NULL
               )
            WHERE u.building_id = $2
              AND u.active = true
            ORDER BY u.unit_number
            "#,
            meeting_id,
            building_id,
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| MeetingAttendanceEntryResponse {
            unit_id: row.unit_id,
            unit_number: row.unit_number,
            owner_name: row.owner_name,
            mode: meeting_attendance_mode_from_db(&row.mode),
            representative_name: row.representative_name,
            shares_represented: row.shares_represented,
        })
        .collect()
    };

    let (shares_present, present_units): (i32, i32) = if status == "closed" {
        let summary = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(gma.shares_represented) FILTER (WHERE gma.mode != 'absent'), 0)::int as "shares_present!",
                   COUNT(*) FILTER (WHERE gma.mode != 'absent')::int as "present_units!"
            FROM general_meeting_attendance gma
            WHERE gma.meeting_id = $1
            "#,
            meeting_id,
        )
        .fetch_one(pool)
        .await?;

        (summary.shares_present, summary.present_units)
    } else {
        let summary = sqlx::query!(
            r#"
            SELECT
                COALESCE(
                    SUM(COALESCE(uak.share_value, 0))
                    FILTER (WHERE COALESCE(gma.mode::text, 'absent') != 'absent'),
                    0
                )::int as "shares_present!",
                COUNT(*)
                FILTER (WHERE COALESCE(gma.mode::text, 'absent') != 'absent')::int as "present_units!"
            FROM unit u
            LEFT JOIN general_meeting_attendance gma
                ON gma.unit_id = u.id AND gma.meeting_id = $1
            LEFT JOIN unit_allocation_key uak
                ON uak.unit_id = u.id
               AND uak.valid_to IS NULL
               AND EXISTS (
                   SELECT 1
                   FROM allocation_key ak
                   WHERE ak.id = uak.key_id
                     AND ak.key_type = 'general'
                     AND ak.deleted_at IS NULL
               )
            WHERE u.building_id = $2
              AND u.active = true
            "#,
            meeting_id,
            building_id,
        )
        .fetch_one(pool)
        .await?;

        (summary.shares_present, summary.present_units)
    };

    let total_shares = get_meeting_total_shares(pool, meeting_id, building_id).await?;

    Ok(MeetingAttendanceResponse {
        entries,
        shares_present,
        total_shares,
        present_units,
    })
}

pub async fn save_attendance(
    pool: &PgPool,
    meeting_id: Uuid,
    building_id: Uuid,
    entries: Vec<MeetingAttendanceInputEntry>,
) -> AppResult<()> {
    let mut tx = pool.begin().await?;

    let status = sqlx::query_scalar!(
        "SELECT status::text FROM general_meeting WHERE id = $1 AND building_id = $2",
        meeting_id,
        building_id,
    )
    .fetch_optional(&mut *tx)
    .await?
    .flatten()
    .ok_or(AppError::NotFound)?;

    if status == "closed" {
        return Err(AppError::conflict_code("errors.meetings.attendance_locked"));
    }

    if status == "ongoing" {
        // List frozen: only update mode/representative_name for existing records.
        for entry in entries {
            let updated = sqlx::query!(
                r#"
                UPDATE general_meeting_attendance
                SET mode = $3::meeting_attendance_mode,
                    representative_name = $4,
                    updated_at = NOW()
                WHERE meeting_id = $1 AND unit_id = $2
                "#,
                meeting_id,
                entry.unit_id,
                meeting_attendance_mode_to_db(&entry.mode) as _,
                entry.representative_name,
            )
            .execute(&mut *tx)
            .await?
            .rows_affected();

            if updated == 0 {
                return Err(AppError::bad_request_code(
                    "errors.meetings.attendance_unit_not_in_meeting",
                ));
            }
        }
    } else {
        // draft/called: full replace.
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
            .ok_or_else(|| {
                AppError::bad_request_code("errors.meetings.unit_general_shares_missing")
            })?;

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
    // Agenda frozen at convocation, except the second-vote passerelle
    // (art. 25-1/26-1) which may still inscribe a resolution during the séance.
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM general_meeting WHERE id = $1 AND building_id = $2 AND status IN ('draft','ongoing'))",
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

/// Deletes a draft resolution and its votes; the draft-only
/// SQL filter yields no row outside `draft` → 404.
pub async fn delete_resolution(
    pool: &PgPool,
    meeting_id: Uuid,
    resolution_id: Uuid,
    building_id: Uuid,
) -> AppResult<bool> {
    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"
        DELETE FROM vote v
        USING resolution r, general_meeting gm
        WHERE v.resolution_id = r.id
          AND r.id = $2
          AND r.meeting_id = $1
          AND gm.id = r.meeting_id
          AND gm.building_id = $3
          AND gm.status = 'draft'
        "#,
        meeting_id,
        resolution_id,
        building_id,
    )
    .execute(&mut *tx)
    .await?;

    let deleted = sqlx::query!(
        r#"
        DELETE FROM resolution r
        WHERE r.id = $2
          AND r.meeting_id = $1
          AND EXISTS (
              SELECT 1
              FROM general_meeting gm
              WHERE gm.id = r.meeting_id
                AND gm.building_id = $3
                AND gm.status = 'draft'
          )
        "#,
        meeting_id,
        resolution_id,
        building_id,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();

    tx.commit().await?;
    Ok(deleted > 0)
}

/// Renumbers the draft agenda from an ordered id list; refuses non-draft
/// meetings (409) and inexact permutations (400).
pub async fn reorder_resolutions(
    pool: &PgPool,
    meeting_id: Uuid,
    building_id: Uuid,
    ordered_ids: Vec<Uuid>,
) -> AppResult<serde_json::Value> {
    let mut tx = pool.begin().await?;

    let status = sqlx::query_scalar!(
        "SELECT status::text FROM general_meeting WHERE id = $1 AND building_id = $2",
        meeting_id,
        building_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .flatten()
    .ok_or(AppError::NotFound)?;

    if status != "draft" {
        return Err(AppError::conflict_code(
            "errors.meetings.not_in_draft_status",
        ));
    }

    let existing: Vec<Uuid> = sqlx::query_scalar!(
        "SELECT id FROM resolution WHERE meeting_id = $1",
        meeting_id
    )
    .fetch_all(&mut *tx)
    .await?;

    let mut expected = existing.clone();
    expected.sort_unstable();
    let mut provided = ordered_ids.clone();
    provided.sort_unstable();
    if expected != provided {
        return Err(AppError::bad_request_code(
            "errors.resolutions.reorder_mismatch",
        ));
    }

    // Two-phase renumber: first negate every number (UNIQUE (meeting_id,
    // number) can never collide), then assign the final 1..N positions.
    sqlx::query!(
        "UPDATE resolution SET number = -number WHERE meeting_id = $1",
        meeting_id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        UPDATE resolution r
        SET number = t.ord::int
        FROM unnest($2::uuid[]) WITH ORDINALITY AS t(id, ord)
        WHERE r.meeting_id = $1
          AND r.id = t.id
        "#,
        meeting_id,
        &ordered_ids,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    list_resolutions(pool, meeting_id, building_id).await
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
        SELECT COUNT(DISTINCT ub.user_id)::int as total_owners,
               COUNT(DISTINCT v.user_id)::int as present_owners,
               COUNT(DISTINCT v.user_id) FILTER (WHERE v.position = 'in_favor')::int as "favorable_owners!"
        FROM building b
        JOIN unit u ON u.building_id = b.id AND u.active = true
        LEFT JOIN user_building ub ON ub.building_id = b.id
            AND 'co_owner' = ANY(ub.roles::text[])
        LEFT JOIN vote v ON v.resolution_id = $1
        WHERE b.id = $2
        "#,
        resolution_id,
        building_id
    )
    .fetch_one(pool)
    .await?;

    let shares_total = get_meeting_total_shares(pool, meeting_id, building_id).await?;

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
        shares_total,
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

pub struct UnitVoteRow {
    pub unit_id: Uuid,
    pub position: String,
}

pub async fn list_votes(
    pool: &PgPool,
    meeting_id: Uuid,
    resolution_id: Uuid,
    building_id: Uuid,
) -> AppResult<Vec<UnitVoteRow>> {
    let rows = sqlx::query!(
        r#"
        SELECT v.unit_id, v.position::text as "position!"
        FROM vote v
        JOIN resolution r ON r.id = v.resolution_id
        JOIN general_meeting gm ON gm.id = r.meeting_id
        WHERE v.resolution_id = $1
          AND r.meeting_id = $2
          AND gm.building_id = $3
        ORDER BY v.unit_id
        "#,
        resolution_id,
        meeting_id,
        building_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| UnitVoteRow {
            unit_id: r.unit_id,
            position: r.position,
        })
        .collect())
}

pub async fn start(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    input: StartMeetingInput,
) -> AppResult<()> {
    let mut tx = pool.begin().await?;

    let scrutineers_json = serde_json::to_value(&input.scrutineers)
        .map_err(|_| AppError::validation_code("errors.meetings.session_roles_required"))?;

    let affected = sqlx::query!(
        r#"
        UPDATE general_meeting
        SET status = 'ongoing',
            president_name = $3,
            president_user_id = $4,
            secretary_name = $5,
            secretary_user_id = $6,
            scrutineers = $7::jsonb,
            updated_at = NOW()
        WHERE id = $1 AND building_id = $2 AND status = 'called'
        "#,
        id,
        building_id,
        input.president_name,
        input.president_user_id,
        input.secretary_name,
        input.secretary_user_id,
        scrutineers_json,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AppError::conflict_code(
            "errors.meetings.not_in_called_status",
        ));
    }

    // Complete attendance: insert absent records for any active unit not yet submitted.
    // Ensures SUM(shares_represented) over all attendance rows = total active tantièmes
    // at lock time, used as the frozen denominator for ongoing/closed tallies (EP-003-S07).
    sqlx::query!(
        r#"
        INSERT INTO general_meeting_attendance
            (meeting_id, unit_id, mode, shares_represented)
        SELECT $1, u.id, 'absent'::meeting_attendance_mode, uak.share_value
        FROM unit u
        JOIN unit_allocation_key uak ON uak.unit_id = u.id AND uak.valid_to IS NULL
        JOIN allocation_key ak ON ak.id = uak.key_id
            AND ak.key_type = 'general'
            AND ak.deleted_at IS NULL
        WHERE u.building_id = $2
          AND u.active = true
          AND NOT EXISTS (
              SELECT 1 FROM general_meeting_attendance
              WHERE meeting_id = $1 AND unit_id = u.id
          )
        "#,
        id,
        building_id,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

/// Total tantièmes of active lots on the general key.
/// Used for draft/called meetings (not yet locked).
async fn compute_active_shares_total(pool: &PgPool, building_id: Uuid) -> AppResult<i32> {
    sqlx::query_scalar!(
        r#"
        SELECT COALESCE(SUM(uak.share_value), 0)::int as "total!"
        FROM unit_allocation_key uak
        JOIN unit u ON u.id = uak.unit_id
        JOIN allocation_key ak ON ak.id = uak.key_id
        WHERE u.building_id = $1
          AND u.active = true
          AND uak.valid_to IS NULL
          AND ak.key_type = 'general'
          AND ak.deleted_at IS NULL
        "#,
        building_id,
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

/// ongoing/closed: SUM(shares_represented) from frozen attendance (all units incl. absent).
/// draft/called: SUM from active units on general key.
async fn get_meeting_total_shares(
    pool: &PgPool,
    meeting_id: Uuid,
    building_id: Uuid,
) -> AppResult<i32> {
    let status = sqlx::query_scalar!(
        "SELECT status::text FROM general_meeting WHERE id = $1 AND building_id = $2",
        meeting_id,
        building_id,
    )
    .fetch_optional(pool)
    .await?
    .flatten()
    .unwrap_or_default();

    if status == "ongoing" || status == "closed" {
        sqlx::query_scalar!(
            r#"
            SELECT COALESCE(SUM(shares_represented), 0)::int as "total!"
            FROM general_meeting_attendance
            WHERE meeting_id = $1
            "#,
            meeting_id,
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::from)
    } else {
        compute_active_shares_total(pool, building_id).await
    }
}

/// Finalizes a resolution by computing the majority result and persisting the status.
///
/// Returns `Ok(true)` if all attending units have voted and the status was updated.
/// Returns `Ok(false)` if some votes are still missing (no-op).
/// The meeting must be `ongoing` for the update to apply.
pub async fn finalize_resolution(
    pool: &PgPool,
    meeting_id: Uuid,
    resolution_id: Uuid,
    building_id: Uuid,
    new_status: &str,
) -> AppResult<bool> {
    let attending: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!"
        FROM general_meeting_attendance gma
        JOIN general_meeting gm ON gm.id = gma.meeting_id
        WHERE gma.meeting_id = $1
          AND gm.building_id = $2
          AND gma.mode != 'absent'
        "#,
        meeting_id,
        building_id,
    )
    .fetch_one(pool)
    .await?;

    let voted: i64 = sqlx::query_scalar!(
        r#"
        SELECT COUNT(DISTINCT v.unit_id) as "count!"
        FROM vote v
        JOIN resolution r ON r.id = v.resolution_id
        JOIN general_meeting gm ON gm.id = r.meeting_id
        WHERE v.resolution_id = $1
          AND r.meeting_id = $2
          AND gm.building_id = $3
        "#,
        resolution_id,
        meeting_id,
        building_id,
    )
    .fetch_one(pool)
    .await?;

    if attending > 0 && voted < attending {
        return Ok(false);
    }

    sqlx::query!(
        r#"
        UPDATE resolution
        SET status = $1::resolution_status
        FROM general_meeting gm
        WHERE resolution.id = $2
          AND resolution.meeting_id = $3
          AND gm.id = resolution.meeting_id
          AND gm.building_id = $4
          AND gm.status = 'ongoing'
        "#,
        new_status as _,
        resolution_id,
        meeting_id,
        building_id,
    )
    .execute(pool)
    .await?;

    Ok(true)
}

pub async fn patch_resolution_description(
    pool: &PgPool,
    meeting_id: Uuid,
    resolution_id: Uuid,
    building_id: Uuid,
    description: Option<String>,
) -> AppResult<Option<serde_json::Value>> {
    let row = sqlx::query!(
        r#"
        UPDATE resolution
        SET description = $3
        FROM general_meeting gm
        WHERE resolution.id = $1
          AND resolution.meeting_id = $2
          AND gm.id = resolution.meeting_id
          AND gm.building_id = $4
          AND gm.status = 'ongoing'
        RETURNING resolution.id, resolution.number, resolution.title,
                  resolution.description,
                  resolution.vote_type::text as vote_type,
                  resolution.status::text as status
        "#,
        resolution_id,
        meeting_id,
        description,
        building_id,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| {
        serde_json::json!({
            "id": r.id,
            "number": r.number,
            "title": r.title,
            "description": r.description,
            "vote_type": r.vote_type,
            "status": r.status,
        })
    }))
}
