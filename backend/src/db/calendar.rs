// ── db/calendar.rs — Calendrier copropriété (EP-014-S01) ─────────────────
//
// Read-only aggregation of existing tables into `CalendarEvent`s. One query
// per source, filtered by `building_id` + `[from, to]`. Final sort by
// `(date, time)` happens in Rust once all sources are merged.

use chrono::NaiveDate;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    domain::calendar::{CalendarEvent, CalendarSource, includes_legal_deadlines},
    errors::AppResult,
};

async fn meeting_events(
    pool: &PgPool,
    building_id: Uuid,
    from: NaiveDate,
    to: NaiveDate,
) -> AppResult<Vec<CalendarEvent>> {
    let rows = sqlx::query(
        r#"
        SELECT id, meeting_type::text as meeting_type, status::text as status, meeting_date
        FROM general_meeting
        WHERE building_id = $1
          AND meeting_date::date BETWEEN $2 AND $3
        "#,
    )
    .bind(building_id)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|r| {
            let id: Uuid = r.try_get("id")?;
            let meeting_type: String = r.try_get("meeting_type")?;
            let status: String = r.try_get("status")?;
            let meeting_date: chrono::DateTime<chrono::Utc> = r.try_get("meeting_date")?;
            let title = format!(
                "Assemblée générale {}{}",
                if meeting_type == "extraordinary" {
                    "extraordinaire"
                } else {
                    "annuelle"
                },
                if status == "draft" {
                    " (brouillon)"
                } else {
                    ""
                }
            );
            Ok(CalendarEvent {
                id: format!("meeting:{id}"),
                source: CalendarSource::Meeting,
                title,
                date: meeting_date.date_naive(),
                time: Some(meeting_date.time()),
                link: format!("/meetings/{id}"),
                is_deadline: false,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(Into::into)
}

/// J+30 (notification PV) and J+60 (contestation) deadlines derived from
/// closed meetings — reserved to Manager/BoardMember.
async fn meeting_legal_deadline_events(
    pool: &PgPool,
    building_id: Uuid,
    from: NaiveDate,
    to: NaiveDate,
) -> AppResult<Vec<CalendarEvent>> {
    let rows = sqlx::query(
        r#"
        SELECT id, meeting_date
        FROM general_meeting
        WHERE building_id = $1
          AND status = 'closed'
        "#,
    )
    .bind(building_id)
    .fetch_all(pool)
    .await?;

    let mut events = Vec::new();
    for r in rows {
        let id: Uuid = r.try_get("id")?;
        let meeting_date: chrono::DateTime<chrono::Utc> = r.try_get("meeting_date")?;
        let meeting_date = meeting_date.date_naive();

        let notification_deadline =
            crate::domain::calendar::minutes_notification_deadline(meeting_date);
        if notification_deadline >= from && notification_deadline <= to {
            events.push(CalendarEvent {
                id: format!("meeting-deadline-notification:{id}"),
                source: CalendarSource::Meeting,
                title: "Délai de notification du PV (J+30)".to_string(),
                date: notification_deadline,
                time: None,
                link: format!("/meetings/{id}"),
                is_deadline: true,
            });
        }

        let contestation_deadline = crate::domain::calendar::contestation_deadline(meeting_date);
        if contestation_deadline >= from && contestation_deadline <= to {
            events.push(CalendarEvent {
                id: format!("meeting-deadline-contestation:{id}"),
                source: CalendarSource::Meeting,
                title: "Délai de contestation des résolutions (J+60)".to_string(),
                date: contestation_deadline,
                time: None,
                link: format!("/meetings/{id}"),
                is_deadline: true,
            });
        }
    }
    Ok(events)
}

async fn contract_reminder_events(
    pool: &PgPool,
    building_id: Uuid,
    from: NaiveDate,
    to: NaiveDate,
) -> AppResult<Vec<CalendarEvent>> {
    let rows = sqlx::query(
        r#"
        SELECT cr.id, cr.reminder_date, vc.subject, vc.vendor_name
        FROM contract_reminder cr
        JOIN vendor_contract vc ON vc.id = cr.contract_id
        WHERE vc.building_id = $1
          AND cr.status = 'pending'
          AND cr.reminder_date BETWEEN $2 AND $3
        "#,
    )
    .bind(building_id)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|r| {
            let id: Uuid = r.try_get("id")?;
            let reminder_date: NaiveDate = r.try_get("reminder_date")?;
            let subject: String = r.try_get("subject")?;
            let vendor_name: String = r.try_get("vendor_name")?;
            Ok(CalendarEvent {
                id: format!("contract:{id}"),
                source: CalendarSource::Contract,
                title: format!("Rappel contrat — {subject} ({vendor_name})"),
                date: reminder_date,
                time: None,
                link: "/contracts".to_string(),
                is_deadline: true,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(Into::into)
}

async fn task_events(
    pool: &PgPool,
    building_id: Uuid,
    from: NaiveDate,
    to: NaiveDate,
) -> AppResult<Vec<CalendarEvent>> {
    let rows = sqlx::query(
        r#"
        SELECT id, title, closed_at
        FROM task
        WHERE building_id = $1
          AND closed_at IS NOT NULL
          AND closed_at::date BETWEEN $2 AND $3
        "#,
    )
    .bind(building_id)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|r| {
            let id: Uuid = r.try_get("id")?;
            let title: String = r.try_get("title")?;
            let closed_at: chrono::DateTime<chrono::Utc> = r.try_get("closed_at")?;
            Ok(CalendarEvent {
                id: format!("task:{id}"),
                source: CalendarSource::Task,
                title: format!("Tâche clôturée — {title}"),
                date: closed_at.date_naive(),
                time: Some(closed_at.time()),
                link: format!("/tasks/{id}"),
                is_deadline: false,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(Into::into)
}

async fn claim_events(
    pool: &PgPool,
    building_id: Uuid,
    from: NaiveDate,
    to: NaiveDate,
) -> AppResult<Vec<CalendarEvent>> {
    let rows = sqlx::query(
        r#"
        SELECT id, title, declared_at, transmitted_at
        FROM claim
        WHERE building_id = $1
          AND (
            declared_at BETWEEN $2 AND $3
            OR transmitted_at BETWEEN $2 AND $3
          )
        "#,
    )
    .bind(building_id)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;

    let mut events = Vec::new();
    for r in rows {
        let id: Uuid = r.try_get("id")?;
        let title: String = r.try_get("title")?;
        let declared_at: NaiveDate = r.try_get("declared_at")?;
        let transmitted_at: Option<NaiveDate> = r.try_get("transmitted_at")?;

        if declared_at >= from && declared_at <= to {
            events.push(CalendarEvent {
                id: format!("claim-declared:{id}"),
                source: CalendarSource::Claim,
                title: format!("Sinistre déclaré — {title}"),
                date: declared_at,
                time: None,
                link: "/claims".to_string(),
                is_deadline: false,
            });
        }
        if let Some(transmitted_at) = transmitted_at
            && transmitted_at >= from
            && transmitted_at <= to
        {
            events.push(CalendarEvent {
                id: format!("claim-transmitted:{id}"),
                source: CalendarSource::Claim,
                title: format!("Sinistre transmis — {title}"),
                date: transmitted_at,
                time: None,
                link: "/claims".to_string(),
                is_deadline: false,
            });
        }
    }
    Ok(events)
}

async fn maintenance_log_events(
    pool: &PgPool,
    building_id: Uuid,
    from: NaiveDate,
    to: NaiveDate,
) -> AppResult<Vec<CalendarEvent>> {
    let rows = sqlx::query(
        r#"
        SELECT id, title, intervention_date
        FROM maintenance_log
        WHERE building_id = $1
          AND intervention_date BETWEEN $2 AND $3
        "#,
    )
    .bind(building_id)
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(|r| {
            let id: Uuid = r.try_get("id")?;
            let title: String = r.try_get("title")?;
            let intervention_date: NaiveDate = r.try_get("intervention_date")?;
            Ok(CalendarEvent {
                id: format!("maintenance:{id}"),
                source: CalendarSource::Maintenance,
                title: format!("Carnet d'entretien — {title}"),
                date: intervention_date,
                time: None,
                link: "/logbook".to_string(),
                is_deadline: false,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(Into::into)
}

/// Aggregates calendar events across every visible source for the given
/// building and date range, sorted by `(date, time)`.
///
/// The `Manual` source (table `calendar_event`, EP-014-S03) doesn't exist
/// yet — that branch is a no-op until the table is created.
pub async fn events(
    pool: &PgPool,
    building_id: Uuid,
    from: NaiveDate,
    to: NaiveDate,
    sources: &[CalendarSource],
    roles: &[crate::domain::Role],
) -> AppResult<Vec<CalendarEvent>> {
    let mut events = Vec::new();

    for source in sources {
        match source {
            CalendarSource::Meeting => {
                events.extend(meeting_events(pool, building_id, from, to).await?);
                if includes_legal_deadlines(roles) {
                    events
                        .extend(meeting_legal_deadline_events(pool, building_id, from, to).await?);
                }
            }
            CalendarSource::Contract => {
                events.extend(contract_reminder_events(pool, building_id, from, to).await?);
            }
            CalendarSource::Task => {
                events.extend(task_events(pool, building_id, from, to).await?);
            }
            CalendarSource::Claim => {
                events.extend(claim_events(pool, building_id, from, to).await?);
            }
            CalendarSource::Maintenance => {
                events.extend(maintenance_log_events(pool, building_id, from, to).await?);
            }
            // Manual events (calendar_event table) land in EP-014-S03.
            CalendarSource::Manual => {}
        }
    }

    events.sort_by(|a, b| a.date.cmp(&b.date).then(a.time.cmp(&b.time)));
    Ok(events)
}
