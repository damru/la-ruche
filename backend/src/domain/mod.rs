pub mod calendar;
pub mod charges;
pub mod convocation_checklist;
pub mod edd_check;
pub mod etat_date;
pub mod ics;
pub mod immutability;
pub mod legal_documents;
pub mod minutes_draft;
pub mod owners;
pub mod processor;
pub mod reminders;
pub mod retention;
pub mod vote;

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

/// User role — mirrors the PostgreSQL `user_role` enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
pub enum Role {
    Manager,
    BoardMember,
    CoOwner,
    Tenant,
}

impl Role {
    /// Every role variant, in declaration order (PG enum order).
    pub const ALL: &'static [Role] = &[
        Role::Manager,
        Role::BoardMember,
        Role::CoOwner,
        Role::Tenant,
    ];

    /// Wire/DB representation (snake_case, matches the PG enum labels).
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Manager => "manager",
            Role::BoardMember => "board_member",
            Role::CoOwner => "co_owner",
            Role::Tenant => "tenant",
        }
    }
}

impl FromStr for Role {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Role::ALL
            .iter()
            .find(|r| r.as_str() == s)
            .copied()
            .ok_or(())
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Role {
    pub fn can_view_accounting(&self) -> bool {
        matches!(self, Role::Manager | Role::BoardMember)
    }

    pub fn can_administer(&self) -> bool {
        matches!(self, Role::Manager)
    }

    pub fn can_vote_gm(&self) -> bool {
        matches!(self, Role::Manager | Role::BoardMember | Role::CoOwner)
    }

    pub fn can_view_gm(&self) -> bool {
        true
    }

    pub fn can_initiate_consultation(&self) -> bool {
        true
    }
}

/// Authenticated user extracted from the JWT — injected into every protected handler
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub building_id: Uuid,
    pub roles: Vec<Role>,
}

impl AuthUser {
    pub fn has_role(&self, role: &Role) -> bool {
        self.roles.contains(role)
    }

    pub fn require_role(&self, role: &Role) -> crate::errors::AppResult<()> {
        if self.has_role(role) {
            Ok(())
        } else {
            Err(crate::errors::AppError::Forbidden)
        }
    }

    pub fn require_any_role(&self, roles: &[Role]) -> crate::errors::AppResult<()> {
        if roles.iter().any(|r| self.has_role(r)) {
            Ok(())
        } else {
            Err(crate::errors::AppError::Forbidden)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AppError;

    fn user_with(roles: Vec<Role>) -> AuthUser {
        AuthUser {
            user_id: Uuid::new_v4(),
            building_id: Uuid::new_v4(),
            roles,
        }
    }

    #[test]
    fn role_from_str_parses_every_known_variant() {
        assert_eq!(Role::from_str("manager"), Ok(Role::Manager));
        assert_eq!(Role::from_str("board_member"), Ok(Role::BoardMember));
        assert_eq!(Role::from_str("co_owner"), Ok(Role::CoOwner));
        assert_eq!(Role::from_str("tenant"), Ok(Role::Tenant));
    }

    #[test]
    fn role_from_str_rejects_unknown_or_miscased_input() {
        assert_eq!(Role::from_str("admin"), Err(()));
        assert_eq!(Role::from_str(""), Err(()));
        assert_eq!(Role::from_str("Manager"), Err(()));
        assert_eq!(Role::from_str("co-owner"), Err(()));
    }

    #[test]
    fn role_display_renders_snake_case() {
        assert_eq!(Role::Manager.to_string(), "manager");
        assert_eq!(Role::BoardMember.to_string(), "board_member");
        assert_eq!(Role::CoOwner.to_string(), "co_owner");
        assert_eq!(Role::Tenant.to_string(), "tenant");
    }

    #[test]
    fn role_display_and_from_str_round_trip() {
        for role in [
            Role::Manager,
            Role::BoardMember,
            Role::CoOwner,
            Role::Tenant,
        ] {
            assert_eq!(Role::from_str(&role.to_string()), Ok(role));
        }
    }

    #[test]
    fn can_view_accounting_only_for_manager_and_board_member() {
        assert!(Role::Manager.can_view_accounting());
        assert!(Role::BoardMember.can_view_accounting());
        assert!(!Role::CoOwner.can_view_accounting());
        assert!(!Role::Tenant.can_view_accounting());
    }

    #[test]
    fn can_administer_only_for_manager() {
        assert!(Role::Manager.can_administer());
        assert!(!Role::BoardMember.can_administer());
        assert!(!Role::CoOwner.can_administer());
        assert!(!Role::Tenant.can_administer());
    }

    #[test]
    fn can_vote_gm_excludes_tenant() {
        assert!(Role::Manager.can_vote_gm());
        assert!(Role::BoardMember.can_vote_gm());
        assert!(Role::CoOwner.can_vote_gm());
        assert!(!Role::Tenant.can_vote_gm());
    }

    #[test]
    fn can_view_gm_allowed_for_every_role() {
        assert!(Role::Manager.can_view_gm());
        assert!(Role::BoardMember.can_view_gm());
        assert!(Role::CoOwner.can_view_gm());
        assert!(Role::Tenant.can_view_gm());
    }

    #[test]
    fn can_initiate_consultation_allowed_for_every_role() {
        assert!(Role::Manager.can_initiate_consultation());
        assert!(Role::BoardMember.can_initiate_consultation());
        assert!(Role::CoOwner.can_initiate_consultation());
        assert!(Role::Tenant.can_initiate_consultation());
    }

    #[test]
    fn has_role_detects_presence_and_absence() {
        let user = user_with(vec![Role::CoOwner, Role::Tenant]);
        assert!(user.has_role(&Role::CoOwner));
        assert!(user.has_role(&Role::Tenant));
        assert!(!user.has_role(&Role::Manager));
    }

    #[test]
    fn has_role_is_false_for_user_without_roles() {
        let user = user_with(vec![]);
        assert!(!user.has_role(&Role::Manager));
    }

    #[test]
    fn require_role_ok_when_role_present() {
        let user = user_with(vec![Role::Manager]);
        assert!(user.require_role(&Role::Manager).is_ok());
    }

    #[test]
    fn require_role_forbidden_when_role_absent() {
        let user = user_with(vec![Role::CoOwner]);
        let err = user.require_role(&Role::Manager).unwrap_err();
        assert!(matches!(err, AppError::Forbidden));
    }

    #[test]
    fn require_role_forbidden_when_user_has_no_roles() {
        let user = user_with(vec![]);
        let err = user.require_role(&Role::CoOwner).unwrap_err();
        assert!(matches!(err, AppError::Forbidden));
    }

    #[test]
    fn require_any_role_ok_when_one_matches() {
        let user = user_with(vec![Role::Tenant, Role::CoOwner]);
        assert!(
            user.require_any_role(&[Role::Manager, Role::CoOwner])
                .is_ok()
        );
    }

    #[test]
    fn require_any_role_forbidden_when_none_match() {
        let user = user_with(vec![Role::Tenant]);
        let err = user
            .require_any_role(&[Role::Manager, Role::BoardMember])
            .unwrap_err();
        assert!(matches!(err, AppError::Forbidden));
    }

    #[test]
    fn require_any_role_forbidden_on_empty_allowed_slice() {
        let user = user_with(vec![Role::Manager]);
        let err = user.require_any_role(&[]).unwrap_err();
        assert!(matches!(err, AppError::Forbidden));
    }
}
