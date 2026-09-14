// ── domain/calendar.rs — Calendrier copropriété (EP-014-S01) ─────────────
//
// Pure types and role-visibility logic. Zero SQL — the DB layer
// (`db::calendar`) is responsible for fetching rows per source.

use chrono::{Days, NaiveDate, NaiveTime};
use serde::Serialize;

use crate::domain::Role;

/// Legal delay after a closed general meeting for the minutes (PV) to be
/// notified — art. 42 loi 1965.
const MINUTES_NOTIFICATION_DEADLINE_DAYS: u64 = 30;
/// Legal delay after a closed general meeting for a resolution to be
/// contested — art. 42 loi 1965.
const CONTESTATION_DEADLINE_DAYS: u64 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CalendarSource {
    Meeting,
    Contract,
    Task,
    Claim,
    Maintenance,
    Manual,
}

#[derive(Debug, Clone, Serialize)]
pub struct CalendarEvent {
    /// Composite id, e.g. "meeting:{uuid}", "contract:{uuid}", ...
    pub id: String,
    pub source: CalendarSource,
    pub title: String,
    pub date: NaiveDate,
    /// Populated for events backed by a TIMESTAMPTZ (meeting_date).
    pub time: Option<NaiveTime>,
    /// SPA path to the source entity, e.g. "/meetings/{id}".
    pub link: String,
    /// True for contract reminders and legal deadlines (consumed by S04's ICS export).
    pub is_deadline: bool,
}

/// AC-CAL-001-C — role-based source visibility, pure and testable.
///
/// Manager/BoardMember see every source, including the legal deadlines
/// (J+30 notification, J+60 contestation) derived from closed meetings.
/// CoOwner/Tenant only see meeting dates and manual events.
pub fn visible_sources(roles: &[Role]) -> Vec<CalendarSource> {
    let privileged = roles
        .iter()
        .any(|r| matches!(r, Role::Manager | Role::BoardMember));

    if privileged {
        vec![
            CalendarSource::Meeting,
            CalendarSource::Contract,
            CalendarSource::Task,
            CalendarSource::Claim,
            CalendarSource::Maintenance,
            CalendarSource::Manual,
        ]
    } else {
        vec![CalendarSource::Meeting, CalendarSource::Manual]
    }
}

/// Whether legal deadlines derived from closed meetings (J+30 / J+60) should
/// be included for the given roles — reserved to Manager/BoardMember.
pub fn includes_legal_deadlines(roles: &[Role]) -> bool {
    roles
        .iter()
        .any(|r| matches!(r, Role::Manager | Role::BoardMember))
}

/// PV notification deadline (J+30) derived from the meeting closing date.
pub fn minutes_notification_deadline(meeting_date: NaiveDate) -> NaiveDate {
    meeting_date
        .checked_add_days(Days::new(MINUTES_NOTIFICATION_DEADLINE_DAYS))
        .unwrap_or(meeting_date)
}

/// Resolution contestation deadline (J+60) derived from the meeting closing date.
pub fn contestation_deadline(meeting_date: NaiveDate) -> NaiveDate {
    meeting_date
        .checked_add_days(Days::new(CONTESTATION_DEADLINE_DAYS))
        .unwrap_or(meeting_date)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    #[test]
    fn manager_sees_all_sources() {
        let sources = visible_sources(&[Role::Manager]);
        assert_eq!(
            sources,
            vec![
                CalendarSource::Meeting,
                CalendarSource::Contract,
                CalendarSource::Task,
                CalendarSource::Claim,
                CalendarSource::Maintenance,
                CalendarSource::Manual,
            ]
        );
    }

    #[test]
    fn board_member_sees_all_sources() {
        let sources = visible_sources(&[Role::BoardMember]);
        assert_eq!(sources.len(), 6);
        assert!(sources.contains(&CalendarSource::Contract));
    }

    #[test]
    fn co_owner_only_sees_meeting_and_manual() {
        let sources = visible_sources(&[Role::CoOwner]);
        assert_eq!(
            sources,
            vec![CalendarSource::Meeting, CalendarSource::Manual]
        );
    }

    #[test]
    fn tenant_only_sees_meeting_and_manual() {
        let sources = visible_sources(&[Role::Tenant]);
        assert_eq!(
            sources,
            vec![CalendarSource::Meeting, CalendarSource::Manual]
        );
    }

    #[test]
    fn multiple_roles_grant_the_most_permissive_visibility() {
        let sources = visible_sources(&[Role::CoOwner, Role::BoardMember]);
        assert_eq!(sources.len(), 6);
    }

    #[test]
    fn legal_deadlines_reserved_to_manager_and_board_member() {
        assert!(includes_legal_deadlines(&[Role::Manager]));
        assert!(includes_legal_deadlines(&[Role::BoardMember]));
        assert!(!includes_legal_deadlines(&[Role::CoOwner]));
        assert!(!includes_legal_deadlines(&[Role::Tenant]));
    }

    #[test]
    fn minutes_notification_deadline_is_thirty_days_after_meeting() {
        assert_eq!(
            minutes_notification_deadline(date(2026, 1, 1)),
            date(2026, 1, 31)
        );
    }

    #[test]
    fn contestation_deadline_is_sixty_days_after_meeting() {
        assert_eq!(contestation_deadline(date(2026, 1, 1)), date(2026, 3, 2));
    }
}
