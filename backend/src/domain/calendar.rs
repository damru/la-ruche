/// Calendar feed — pure types and role-based visibility rules.
/// No SQL here; see `db::calendar` for the aggregating queries.
use chrono::{Days, NaiveDate, NaiveTime};
use serde::Serialize;

use crate::domain::Role;

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
    pub id: String,
    pub source: CalendarSource,
    pub title: String,
    pub date: NaiveDate,
    pub time: Option<NaiveTime>,
    pub link: String,
    pub is_deadline: bool,
}

/// AC-CAL-001-C — sources visible for a given set of roles.
/// Manager/BoardMember see everything; CoOwner/Tenant only see meeting dates
/// (the séance itself, not the PV/contestation deadlines) plus manual events.
pub fn visible_sources(roles: &[Role]) -> Vec<CalendarSource> {
    if roles
        .iter()
        .any(|r| matches!(r, Role::Manager | Role::BoardMember))
    {
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

/// True when the given roles may see the PV / contestation deadlines derived
/// from a closed general meeting (Manager/BoardMember only).
pub fn can_view_meeting_deadlines(roles: &[Role]) -> bool {
    roles
        .iter()
        .any(|r| matches!(r, Role::Manager | Role::BoardMember))
}

/// PV notification deadline — J+30 after a closed meeting's date.
pub fn minutes_deadline(meeting_date: NaiveDate) -> Option<NaiveDate> {
    meeting_date.checked_add_days(Days::new(30))
}

/// Contestation deadline — J+60 after a closed meeting's date.
pub fn contestation_deadline(meeting_date: NaiveDate) -> Option<NaiveDate> {
    meeting_date.checked_add_days(Days::new(60))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manager_sees_all_sources() {
        let sources = visible_sources(&[Role::Manager]);
        assert!(sources.contains(&CalendarSource::Contract));
        assert!(sources.contains(&CalendarSource::Task));
        assert!(sources.contains(&CalendarSource::Claim));
        assert!(sources.contains(&CalendarSource::Maintenance));
        assert!(sources.contains(&CalendarSource::Meeting));
        assert!(sources.contains(&CalendarSource::Manual));
    }

    #[test]
    fn board_member_sees_all_sources() {
        let sources = visible_sources(&[Role::BoardMember]);
        assert!(sources.contains(&CalendarSource::Contract));
    }

    #[test]
    fn co_owner_sees_only_meeting_and_manual() {
        let sources = visible_sources(&[Role::CoOwner]);
        assert_eq!(
            sources,
            vec![CalendarSource::Meeting, CalendarSource::Manual]
        );
    }

    #[test]
    fn tenant_sees_only_meeting_and_manual() {
        let sources = visible_sources(&[Role::Tenant]);
        assert_eq!(
            sources,
            vec![CalendarSource::Meeting, CalendarSource::Manual]
        );
    }

    #[test]
    fn deadlines_visible_only_for_manager_and_board_member() {
        assert!(can_view_meeting_deadlines(&[Role::Manager]));
        assert!(can_view_meeting_deadlines(&[Role::BoardMember]));
        assert!(!can_view_meeting_deadlines(&[Role::CoOwner]));
        assert!(!can_view_meeting_deadlines(&[Role::Tenant]));
    }

    #[test]
    fn minutes_deadline_is_thirty_days_after_meeting() {
        let meeting_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        assert_eq!(
            minutes_deadline(meeting_date),
            NaiveDate::from_ymd_opt(2026, 1, 31)
        );
    }

    #[test]
    fn contestation_deadline_is_sixty_days_after_meeting() {
        let meeting_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        assert_eq!(
            contestation_deadline(meeting_date),
            NaiveDate::from_ymd_opt(2026, 3, 2)
        );
    }
}
