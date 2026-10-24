use crate::domain::{AuthUser, Role};

//! Centralised permission checks — every `can<Action><Scope>` method lives here
//! so that role-based access rules are defined in exactly one place.
//!
//! Naming convention: `can_<action>_<scope>` where:
//! - action  = view | manage | vote | initiate | administer
//! - scope   = gm (general meeting) | consultation | budget | task | contract |
//!   document | accounting | logbook | report | claim | unit |
//!   user | building | calendar | etat_date | judicial | governance
//!
//! A user may hold several roles simultaneously (e.g. `[Manager, CoOwner]`);
//! every check inspects **all** of the user's roles.

// ── General Meeting (AG) ──────────────────────────────────────────────────────

/// Anyone authenticated may view AG data (list, get, results, attendance).
pub fn can_view_gm(user: &AuthUser) -> bool {
    !user.roles.is_empty()
}

/// Only co-owners (residents) may vote in a general meeting.
/// Managers and board members who are not also co-owners cannot vote.
pub fn can_vote_gm(user: &AuthUser) -> bool {
    user.has_role(&Role::CoOwner)
}

/// Managers, board members and co-owners may record GM vote entries.
/// During the session the manager (syndic) manually enters each unit's vote
/// in the VoteModal; co-owners record their own vote directly.
pub fn can_record_gm_votes(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager)
        || user.has_role(&Role::BoardMember)
        || user.has_role(&Role::CoOwner)
}

/// Only the manager may administer AG lifecycle (create, update, call, start,
/// close, resolutions, attendance, minutes, documents).
pub fn can_manage_gm(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager)
}

// ── Consultation ──────────────────────────────────────────────────────────────

/// Anyone authenticated may view consultations (list, get, results).
pub fn can_view_consultation(user: &AuthUser) -> bool {
    !user.roles.is_empty()
}

/// Only co-owners and tenants may vote in a consultation.
/// Managers and board members who are not also residents cannot vote.
pub fn can_vote_consultation(user: &AuthUser) -> bool {
    user.has_role(&Role::CoOwner) || user.has_role(&Role::Tenant)
}

/// Managers and board members manage the consultation lifecycle
/// (create, update, open, close, remind, delete).
pub fn can_manage_consultation(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager) || user.has_role(&Role::BoardMember)
}

/// Anyone authenticated may initiate (create) a consultation.
pub fn can_initiate_consultation(user: &AuthUser) -> bool {
    !user.roles.is_empty()
}

// ── Budget ────────────────────────────────────────────────────────────────────

/// Co-owners and tenants may view budgets (approved ones only for non-managers).
pub fn can_view_budget(user: &AuthUser) -> bool {
    !user.roles.is_empty()
}

/// Managers and board members may see draft budgets and levy details.
pub fn can_manage_budget(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager) || user.has_role(&Role::BoardMember)
}

/// Only the manager may create / update / close / approve budgets and lines.
pub fn can_administer_budget(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager)
}

// ── Task ──────────────────────────────────────────────────────────────────────

/// Co-owners and tenants may view tasks; managers and board members may manage them.
pub fn can_view_task(user: &AuthUser) -> bool {
    !user.roles.is_empty()
}

pub fn can_manage_task(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager) || user.has_role(&Role::BoardMember)
}

// ── Contract ──────────────────────────────────────────────────────────────────

pub fn can_view_contract(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager) || user.has_role(&Role::BoardMember)
}

pub fn can_manage_contract(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager) || user.has_role(&Role::BoardMember)
}

pub fn can_administer_contract(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager)
}

// ── Document ──────────────────────────────────────────────────────────────────

/// Anyone authenticated may view documents they are allowed to see
/// (filtered by `visible_to_roles` at the DB layer).
pub fn can_view_document(user: &AuthUser) -> bool {
    !user.roles.is_empty()
}

pub fn can_manage_document(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager) || user.has_role(&Role::BoardMember)
}

// ── Logbook ───────────────────────────────────────────────────────────────────

/// Anyone authenticated may view the logbook (list, summary, get).
pub fn can_view_logbook(user: &AuthUser) -> bool {
    !user.roles.is_empty()
}

pub fn can_manage_logbook(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager) || user.has_role(&Role::BoardMember)
}

// ── Report ────────────────────────────────────────────────────────────────────

pub fn can_view_report(user: &AuthUser) -> bool {
    !user.roles.is_empty()
}

pub fn can_manage_report(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager) || user.has_role(&Role::BoardMember)
}

// ── Claim ─────────────────────────────────────────────────────────────────────

pub fn can_view_claim(user: &AuthUser) -> bool {
    !user.roles.is_empty()
}

pub fn can_manage_claim(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager) || user.has_role(&Role::BoardMember)
}

// ── Unit ──────────────────────────────────────────────────────────────────────

pub fn can_view_unit(user: &AuthUser) -> bool {
    !user.roles.is_empty()
}

pub fn can_manage_unit(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager)
}

// ── User ──────────────────────────────────────────────────────────────────────

pub fn can_view_users(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager) || user.has_role(&Role::BoardMember)
}

pub fn can_manage_users(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager)
}

// ── Accounting / Works fund ───────────────────────────────────────────────────

pub fn can_view_accounting(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager) || user.has_role(&Role::BoardMember)
}

pub fn can_manage_accounting(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager)
}

// ── Calendar ──────────────────────────────────────────────────────────────────

pub fn can_view_calendar(user: &AuthUser) -> bool {
    !user.roles.is_empty()
}

pub fn can_manage_calendar(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager) || user.has_role(&Role::BoardMember)
}

// ── État daté ─────────────────────────────────────────────────────────────────

pub fn can_view_etat_date(user: &AuthUser) -> bool {
    !user.roles.is_empty()
}

pub fn can_manage_etat_date(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager)
}

// ── Judicial proceedings ──────────────────────────────────────────────────────

pub fn can_view_judicial(user: &AuthUser) -> bool {
    !user.roles.is_empty()
}

pub fn can_manage_judicial(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager)
}

// ── Governance ────────────────────────────────────────────────────────────────

pub fn can_view_governance(user: &AuthUser) -> bool {
    !user.roles.is_empty()
}

pub fn can_manage_governance(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager)
}

// ── Contractor (vendor-facing) ────────────────────────────────────────────────

pub fn can_manage_contractor(user: &AuthUser) -> bool {
    user.has_role(&Role::Manager) || user.has_role(&Role::BoardMember)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn user_with(roles: &[Role]) -> AuthUser {
        AuthUser {
            user_id: Uuid::new_v4(),
            building_id: Uuid::new_v4(),
            roles: roles.to_vec(),
        }
    }

    // ── GM ──

    #[test]
    fn can_vote_gm_only_co_owner() {
        assert!(can_vote_gm(&user_with(&[Role::CoOwner])));
        assert!(!can_vote_gm(&user_with(&[Role::Manager])));
        assert!(!can_vote_gm(&user_with(&[Role::BoardMember])));
        assert!(!can_vote_gm(&user_with(&[Role::Tenant])));
    }

    #[test]
    fn can_vote_gm_multi_role_manager_plus_co_owner() {
        assert!(can_vote_gm(&user_with(&[Role::Manager, Role::CoOwner])));
    }

    #[test]
    fn can_view_gm_all_roles() {
        assert!(can_view_gm(&user_with(&[Role::Manager])));
        assert!(can_view_gm(&user_with(&[Role::BoardMember])));
        assert!(can_view_gm(&user_with(&[Role::CoOwner])));
        assert!(can_view_gm(&user_with(&[Role::Tenant])));
    }

    #[test]
    fn can_manage_gm_only_manager() {
        assert!(can_manage_gm(&user_with(&[Role::Manager])));
        assert!(!can_manage_gm(&user_with(&[Role::BoardMember])));
        assert!(!can_manage_gm(&user_with(&[Role::CoOwner])));
        assert!(!can_manage_gm(&user_with(&[Role::Tenant])));
    }

    // ── Consultation ──

    #[test]
    fn can_vote_consultation_co_owner_and_tenant() {
        assert!(can_vote_consultation(&user_with(&[Role::CoOwner])));
        assert!(can_vote_consultation(&user_with(&[Role::Tenant])));
        assert!(!can_vote_consultation(&user_with(&[Role::Manager])));
        assert!(!can_vote_consultation(&user_with(&[Role::BoardMember])));
    }

    #[test]
    fn can_vote_consultation_multi_role_manager_plus_co_owner() {
        assert!(can_vote_consultation(&user_with(&[
            Role::Manager,
            Role::CoOwner
        ])));
    }

    #[test]
    fn can_manage_consultation_manager_and_board_member() {
        assert!(can_manage_consultation(&user_with(&[Role::Manager])));
        assert!(can_manage_consultation(&user_with(&[Role::BoardMember])));
        assert!(!can_manage_consultation(&user_with(&[Role::CoOwner])));
        assert!(!can_manage_consultation(&user_with(&[Role::Tenant])));
    }

    // ── Budget ──

    #[test]
    fn can_manage_budget_manager_and_board_member() {
        assert!(can_manage_budget(&user_with(&[Role::Manager])));
        assert!(can_manage_budget(&user_with(&[Role::BoardMember])));
        assert!(!can_manage_budget(&user_with(&[Role::CoOwner])));
        assert!(!can_manage_budget(&user_with(&[Role::Tenant])));
    }

    #[test]
    fn can_administer_budget_only_manager() {
        assert!(can_administer_budget(&user_with(&[Role::Manager])));
        assert!(!can_administer_budget(&user_with(&[Role::BoardMember])));
    }

    // ── Document ──

    #[test]
    fn can_manage_document_manager_and_board_member() {
        assert!(can_manage_document(&user_with(&[Role::Manager])));
        assert!(can_manage_document(&user_with(&[Role::BoardMember])));
        assert!(!can_manage_document(&user_with(&[Role::CoOwner])));
    }
}
