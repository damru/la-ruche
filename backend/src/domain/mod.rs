pub mod charges;
pub mod processor;
pub mod reminders;
pub mod vote;

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

/// User role — mirrors the PostgreSQL `user_role` enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
pub enum Role {
    Manager,
    BoardMember,
    CoOwner,
    Tenant,
}

impl FromStr for Role {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "manager" => Ok(Role::Manager),
            "board_member" => Ok(Role::BoardMember),
            "co_owner" => Ok(Role::CoOwner),
            "tenant" => Ok(Role::Tenant),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Role::Manager => "manager",
            Role::BoardMember => "board_member",
            Role::CoOwner => "co_owner",
            Role::Tenant => "tenant",
        };
        f.write_str(s)
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
