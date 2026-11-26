use crate::db::activity_log::actions;
use crate::domain::Role;

/// Sensitive lifecycle actions are reserved for manager / board_member
/// ("conseil syndical"). Anything else is visible to every role that can read
/// the underlying entity; new vocabulary is visible by default.
const SENSITIVE_ACTIONS: &[&str] = &[actions::ASSIGNEE_CHANGED, actions::VENDOR_LINK_GENERATED];

/// Allow-list: an unknown or future role is treated as unprivileged.
pub(crate) fn can_view_action(roles: &[Role], action: &str) -> bool {
    if SENSITIVE_ACTIONS.contains(&action) {
        roles
            .iter()
            .any(|r| matches!(r, Role::Manager | Role::BoardMember))
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
