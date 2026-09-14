/// Repository for the aggregated calendar feed.
///
/// Rules:
///   - No business logic here (deadline calculations -> domain/calendar.rs)
///   - Every query is bounded by building_id + [from, to] (tenant isolation)
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::Role,
    domain::calendar::{
        CalendarEvent, CalendarSource, can_view_meeting_deadlines, contestation_deadline,
        minutes_deadline,
    },
    errors::AppResult,
};

/// Aggregates events from every requested source into a single sorted feed.
pub async fn events(
    pool: &PgPool,
    building_id: Uuid,
    from: NaiveDate,
    to: NaiveDate,
    sources: &[CalendarSource],
    roles: &[Role],
) -> AppResult<Vec<CalendarEvent>> {
    let mut events = Vec::new();

    if sources.contains(&CalendarSource::Meeting) {
        events.extend(meeting_events(pool, building_id, from, to, roles).await?);
    }
    if sources.contains(&CalendarSource::Contract) {
        events.extend(contract_reminder_events(pool, building_id, from, to).await?);
    }
    if sources.contains(&CalendarSource::Task) {
        events.extend(task_events(pool, building_id, from, to).await?);
    }
    if sources.contains(&CalendarSource::Claim) {
        events.extend(claim_events(pool, building_id, from, to).await?);
    }
    if sources.contains(&CalendarSource::Maintenance) {
        events.extend(maintenance_events(pool, building_id, from, to).await?);
    }
    // Manual events (calendar_event table) are introduced by EP-014-S03;
    // until that table exists this branch is a tolerant no-op.

    events.sort_by_key(|e| (e.date, e.time));
    Ok(events)
}

async fn meeting_events(
    pool: &PgPool,
    building_id: Uuid,
    from: NaiveDate,
    to: NaiveDate,
    roles: &[Role],
) -> AppResult<Vec<CalendarEvent>> {
    struct Row {
        id: Uuid,
        meeting_type: String,
        status: String,
        meeting_date: chrono::DateTime<chrono::Utc>,
    }

    let rows = sqlx::query_as!(
        Row,
        r#"SELECT id, meeting_type::text as "meeting_type!", status::text as "status!", meeting_date
           FROM general_meeting
           WHERE building_id = $1 AND meeting_date::date BETWEEN $2 AND $3"#,
        building_id,
        from,
        to,
    )
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    let deadlines_visible = can_view_meeting_deadlines(roles);

    for row in rows {
        let local = row.meeting_date.naive_local();
        result.push(CalendarEvent {
            id: format!("meeting:{}", row.id),
            source: CalendarSource::Meeting,
            title: meeting_title(&row.meeting_type),
            date: local.date(),
            time: Some(local.time()),
            link: format!("/meetings/{}", row.id),
            is_deadline: false,
        });

        if deadlines_visible && row.status == "closed" {
            let meeting_day = local.date();
            if let Some(deadline) = minutes_deadline(meeting_day)
                && deadline >= from
                && deadline <= to
            {
                result.push(CalendarEvent {
                    id: format!("meeting:{}:minutes_deadline", row.id),
                    source: CalendarSource::Meeting,
                    title: "PV — délai de notification".into(),
                    date: deadline,
                    time: None,
                    link: format!("/meetings/{}", row.id),
                    is_deadline: true,
                });
            }
            if let Some(deadline) = contestation_deadline(meeting_day)
                && deadline >= from
                && deadline <= to
            {
                result.push(CalendarEvent {
                    id: format!("meeting:{}:contestation_deadline", row.id),
                    source: CalendarSource::Meeting,
                    title: "AG — délai de contestation".into(),
                    date: deadline,
                    time: None,
                    link: format!("/meetings/{}", row.id),
                    is_deadline: true,
                });
            }
        }
    }

    Ok(result)
}

fn meeting_title(meeting_type: &str) -> String {
    match meeting_type {
        "annual" => "Assemblée générale ordinaire".into(),
        "extraordinary" => "Assemblée générale extraordinaire".into(),
        other => format!("Assemblée générale ({other})"),
    }
}

async fn contract_reminder_events(
    pool: &PgPool,
    building_id: Uuid,
    from: NaiveDate,
    to: NaiveDate,
) -> AppResult<Vec<CalendarEvent>> {
    struct Row {
        id: Uuid,
        reminder_date: NaiveDate,
        subject: String,
    }

    let rows = sqlx::query_as!(
        Row,
        r#"SELECT cr.id, cr.reminder_date, vc.subject
           FROM contract_reminder cr
           JOIN vendor_contract vc ON vc.id = cr.contract_id
           WHERE vc.building_id = $1 AND cr.status = 'pending'
             AND cr.reminder_date BETWEEN $2 AND $3"#,
        building_id,
        from,
        to,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| CalendarEvent {
            id: format!("contract:{}", row.id),
            source: CalendarSource::Contract,
            title: format!("Contrat — {}", row.subject),
            date: row.reminder_date,
            time: None,
            link: "/contracts".into(),
            is_deadline: true,
        })
        .collect())
}

async fn task_events(
    pool: &PgPool,
    building_id: Uuid,
    from: NaiveDate,
    to: NaiveDate,
) -> AppResult<Vec<CalendarEvent>> {
    struct Row {
        id: Uuid,
        title: String,
        closed_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    let rows = sqlx::query_as!(
        Row,
        r#"SELECT id, title, closed_at
           FROM task
           WHERE building_id = $1 AND closed_at IS NOT NULL
             AND closed_at::date BETWEEN $2 AND $3"#,
        building_id,
        from,
        to,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let closed_at = row.closed_at?;
            let local = closed_at.naive_local();
            Some(CalendarEvent {
                id: format!("task:{}", row.id),
                source: CalendarSource::Task,
                title: row.title,
                date: local.date(),
                time: Some(local.time()),
                link: format!("/tasks/{}", row.id),
                is_deadline: false,
            })
        })
        .collect())
}

async fn claim_events(
    pool: &PgPool,
    building_id: Uuid,
    from: NaiveDate,
    to: NaiveDate,
) -> AppResult<Vec<CalendarEvent>> {
    struct Row {
        id: Uuid,
        title: String,
        declared_at: NaiveDate,
    }

    let rows = sqlx::query_as!(
        Row,
        r#"SELECT id, title, declared_at
           FROM claim
           WHERE building_id = $1 AND declared_at BETWEEN $2 AND $3"#,
        building_id,
        from,
        to,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| CalendarEvent {
            id: format!("claim:{}", row.id),
            source: CalendarSource::Claim,
            title: format!("Sinistre — {}", row.title),
            date: row.declared_at,
            time: None,
            link: "/claims".into(),
            is_deadline: false,
        })
        .collect())
}

async fn maintenance_events(
    pool: &PgPool,
    building_id: Uuid,
    from: NaiveDate,
    to: NaiveDate,
) -> AppResult<Vec<CalendarEvent>> {
    struct Row {
        id: Uuid,
        title: String,
        intervention_date: NaiveDate,
    }

    let rows = sqlx::query_as!(
        Row,
        r#"SELECT id, title, intervention_date
           FROM maintenance_log
           WHERE building_id = $1 AND intervention_date BETWEEN $2 AND $3"#,
        building_id,
        from,
        to,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| CalendarEvent {
            id: format!("maintenance:{}", row.id),
            source: CalendarSource::Maintenance,
            title: row.title,
            date: row.intervention_date,
            time: None,
            link: "/logbook".into(),
            is_deadline: false,
        })
        .collect())
}
