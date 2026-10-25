//! Canonical notification event catalog — single source of truth.
//! Mirrored by the frontend, which keeps only display order + i18n.
//!
//! No notification delivery infrastructure exists yet: these preferences are
//! stored for the future dispatcher and have no sending effect today.

pub(crate) struct NotifEvent {
    pub key: &'static str,
    pub mandatory: bool,
    pub default_enabled: bool,
}

/// Canonical catalog — 16 events, 3 mandatory (legal / security, always on).
pub(crate) const NOTIF_EVENTS: &[NotifEvent] = &[
    NotifEvent {
        key: "ag_convocation",
        mandatory: true,
        default_enabled: true,
    },
    NotifEvent {
        key: "levy_call_issued",
        mandatory: true,
        default_enabled: true,
    },
    NotifEvent {
        key: "account_security",
        mandatory: true,
        default_enabled: true,
    },
    NotifEvent {
        key: "ag_minutes",
        mandatory: false,
        default_enabled: true,
    },
    NotifEvent {
        key: "ag_vote_reminder",
        mandatory: false,
        default_enabled: true,
    },
    NotifEvent {
        key: "levy_overdue",
        mandatory: false,
        default_enabled: true,
    },
    NotifEvent {
        key: "consultation_opened",
        mandatory: false,
        default_enabled: true,
    },
    NotifEvent {
        key: "report_published",
        mandatory: false,
        default_enabled: true,
    },
    NotifEvent {
        key: "document_published",
        mandatory: false,
        default_enabled: true,
    },
    NotifEvent {
        key: "contract_expiring",
        mandatory: false,
        default_enabled: true,
    },
    NotifEvent {
        key: "claim_created",
        mandatory: false,
        default_enabled: true,
    },
    NotifEvent {
        key: "claim_resolved",
        mandatory: false,
        default_enabled: false,
    },
    NotifEvent {
        key: "task_assigned",
        mandatory: false,
        default_enabled: true,
    },
    NotifEvent {
        key: "task_due",
        mandatory: false,
        default_enabled: true,
    },
    NotifEvent {
        key: "maintenance_due",
        mandatory: false,
        default_enabled: true,
    },
    NotifEvent {
        key: "weekly_digest",
        mandatory: false,
        default_enabled: false,
    },
];

pub(crate) fn find(key: &str) -> Option<&'static NotifEvent> {
    NOTIF_EVENTS.iter().find(|e| e.key == key)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Guards against catalog drift — breaks if the catalog no longer
    /// matches the agreed 16 events / 3 mandatory contract.
    #[test]
    fn catalog_has_16_events_and_3_mandatory() {
        assert_eq!(NOTIF_EVENTS.len(), 16);
        assert_eq!(NOTIF_EVENTS.iter().filter(|e| e.mandatory).count(), 3);
        // Mandatory events are always enabled.
        assert!(
            NOTIF_EVENTS
                .iter()
                .filter(|e| e.mandatory)
                .all(|e| e.default_enabled)
        );
    }

    #[test]
    fn find_returns_none_on_unknown_key() {
        assert!(find("ag_convocation").is_some());
        assert!(find("no_such_event").is_none());
    }
}
