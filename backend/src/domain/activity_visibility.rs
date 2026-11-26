use crate::db::activity_log::actions;
use crate::domain::Role;

/// Sensitive lifecycle actions are reserved for manager / board_member
/// ("conseil syndical"). Anything else is visible to every role that can read
/// the underlying entity; new vocabulary is visible by default.
/// Allow-list: an unknown or future role is treated as unprivileged.
pub(crate) fn can_view_action(roles: &[Role], action: &str) -> bool {
    if actions::is_sensitive(action) {
        roles
            .iter()
            .any(|r| matches!(r, Role::Manager | Role::BoardMember))
    } else {
        true
    }
}

/// Drops the events the viewer may not see. Applied after serialization and
/// before `report_privacy::mask_timeline`: the anonymity mask must still see
/// every event, and the two rules stay in this order at a single place.
pub(crate) fn filter_events(events: &mut serde_json::Value, roles: &[Role]) {
    let Some(list) = events.as_array_mut() else {
        return;
    };
    list.retain(|event| {
        let action = event
            .get("action")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        can_view_action(roles, action)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_catalogue_is_sane() {
        let mut names: Vec<&str> = actions::ALL.iter().map(|def| def.name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), actions::ALL.len(), "duplicate action name");
        let sensitive: Vec<&str> = actions::ALL
            .iter()
            .filter(|def| def.sensitive)
            .map(|def| def.name)
            .collect();
        assert_eq!(
            sensitive,
            vec![actions::ASSIGNEE_CHANGED, actions::VENDOR_LINK_GENERATED]
        );
        assert!(!actions::is_sensitive("some_future_action"));
    }

    #[test]
    fn filter_events_keeps_only_visible_actions() {
        let mut events = serde_json::json!([
            { "action": actions::STATUS_CHANGED },
            { "action": actions::ASSIGNEE_CHANGED },
            { "action": actions::VENDOR_LINK_GENERATED },
            { "action": "future_action" }
        ]);
        filter_events(&mut events, &[Role::Tenant]);
        let kept: Vec<&str> = events
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["action"].as_str().unwrap())
            .collect();
        assert_eq!(kept, vec![actions::STATUS_CHANGED, "future_action"]);
    }

    #[test]
    fn sensitive_actions_restricted_to_managers() {
        for roles in [vec![Role::CoOwner], vec![Role::Tenant]] {
            assert!(!can_view_action(&roles, actions::ASSIGNEE_CHANGED));
            assert!(!can_view_action(&roles, actions::VENDOR_LINK_GENERATED));
            assert!(can_view_action(&roles, actions::STATUS_CHANGED));
            assert!(can_view_action(&roles, actions::CONTRACTOR_ASSIGNED));
        }
        for roles in [vec![Role::Manager], vec![Role::BoardMember]] {
            assert!(can_view_action(&roles, actions::ASSIGNEE_CHANGED));
            assert!(can_view_action(&roles, actions::VENDOR_LINK_GENERATED));
        }
    }

    #[test]
    fn unknown_action_is_visible_by_default() {
        assert!(can_view_action(&[Role::Tenant], "some_future_action"));
    }
}
