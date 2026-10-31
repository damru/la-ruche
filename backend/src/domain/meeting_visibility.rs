//! Read-visibility rule for general meetings, per resident roles.
//!
//! Pure stateless function — no DB access. A tenant-only user only sees
//! `closed` meetings (results + published minutes); any other role
//! combination sees every meeting of their building.

use crate::domain::{Role, immutability::meeting_status};

/// Whether a user holding `roles` may read a meeting in `status`.
/// `status` is the `general_meeting.status` text value.
///
/// A user who is both tenant and co-owner reads as a co-owner.
pub(crate) fn meeting_visible_to(roles: &[Role], status: &str) -> bool {
    if roles.contains(&Role::CoOwner) {
        return true;
    }
    match roles {
        [Role::Tenant] => status == meeting_status::CLOSED,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tenant_only_sees_closed() {
        for status in [
            meeting_status::DRAFT,
            meeting_status::CALLED,
            meeting_status::ONGOING,
        ] {
            assert!(!meeting_visible_to(&[Role::Tenant], status));
        }
        assert!(meeting_visible_to(&[Role::Tenant], meeting_status::CLOSED));
    }

    #[test]
    fn tenant_plus_co_owner_reads_everything() {
        for status in [
            meeting_status::DRAFT,
            meeting_status::CALLED,
            meeting_status::ONGOING,
            meeting_status::CLOSED,
        ] {
            assert!(meeting_visible_to(&[Role::Tenant, Role::CoOwner], status));
        }
    }

    #[test]
    fn other_roles_see_every_status() {
        for roles in [
            &[Role::Manager][..],
            &[Role::BoardMember][..],
            &[Role::CoOwner][..],
        ] {
            for status in [
                meeting_status::DRAFT,
                meeting_status::CALLED,
                meeting_status::ONGOING,
                meeting_status::CLOSED,
            ] {
                assert!(meeting_visible_to(roles, status));
            }
        }
    }
}
