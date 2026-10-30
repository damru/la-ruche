//! Read-visibility rule for general meetings, per resident role.
//!
//! Pure stateless function — no DB access. Tenants only see `closed` meetings
//! (results + published minutes); every other role sees every meeting of
//! their building.

use crate::domain::{Role, immutability::meeting_status};

/// Whether a user holding `role` may read a meeting in `status`.
/// `status` is the `general_meeting.status` text value.
pub(crate) fn meeting_visible_to(role: &Role, status: &str) -> bool {
    match role {
        Role::Tenant => status == meeting_status::CLOSED,
        _ => true,
    }
}

/// Whether a tenant-only user may read a meeting in `status`.
pub(crate) fn tenant_may_read(status: &str) -> bool {
    meeting_visible_to(&Role::Tenant, status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tenant_sees_only_closed() {
        for status in [
            meeting_status::DRAFT,
            meeting_status::CALLED,
            meeting_status::ONGOING,
        ] {
            assert!(!meeting_visible_to(&Role::Tenant, status));
        }
        assert!(meeting_visible_to(&Role::Tenant, meeting_status::CLOSED));
    }

    #[test]
    fn other_roles_see_every_status() {
        for role in [Role::Manager, Role::BoardMember, Role::CoOwner] {
            for status in [
                meeting_status::DRAFT,
                meeting_status::CALLED,
                meeting_status::ONGOING,
                meeting_status::CLOSED,
            ] {
                assert!(meeting_visible_to(&role, status));
            }
        }
    }
}
