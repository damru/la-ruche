//! Pattern-A immutability guards (ADR-009).
//!
//! Pure stateless functions — no DB access. Call `assert_mutable` before any
//! financial mutation in `domain/` or `db/` layers, never in handlers.

use crate::errors::{AppError, AppResult};

/// Pattern-A entities whose fields are frozen on a terminal state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entity {
    /// Frozen once `status != 'draft'` (i.e. issued / paid / overdue).
    LevyCall,
    /// Frozen once the parent assembly reaches `ongoing` or `closed`.
    Vote,
    /// Frozen once the parent assembly reaches `closed`.
    Resolution,
    /// `completion_cost` / `closed_at` frozen when `status = 'closed'`.
    Task,
    /// Financial fields frozen once `transmitted_at IS NOT NULL` (pass "transmitted").
    Claim,
    /// All entries frozen (EP-018-S07 Pattern C); corrections use append-only correction entries.
    MaintenanceLog,
    /// Financial/legal fields (`amount`, `start_date`, `end_date`, `notice_days`,
    /// `contract_type`, `payment_frequency`, `auto_renewal`) frozen once
    /// `status != 'draft'`. Amended via `POST /contracts/{id}/amendments`
    /// (Pattern B, ADR-009, PRD Contrats fournisseurs §4).
    VendorContract,
    /// Budget and its lines are frozen once approved in AG: the budget becomes a
    /// legal act. Amended via `POST /budgets/{id}/rectificatif`
    /// (Pattern B, ADR-009, PRD Budget et charges §Immuabilité).
    Budget,
}

/// Returns `true` when the entity may still be mutated.
///
/// `status` semantics per entity:
/// - `LevyCall`       → levy_status string ("draft" | "issued" | "paid" | "overdue")
/// - `Vote`           → parent assembly's meeting_status
/// - `Resolution`     → parent assembly's meeting_status
/// - `Task`           → task_status string
/// - `Claim`          → "transmitted" when transmitted_at IS NOT NULL, else any other value
/// - `MaintenanceLog` → ignored; always frozen (EP-018-S07 Pattern C)
pub fn is_mutable(entity: &Entity, status: &str) -> bool {
    match entity {
        Entity::LevyCall => status == "draft",
        Entity::Vote => !matches!(status, "ongoing" | "closed"),
        Entity::Resolution => status != "closed",
        Entity::Task => !matches!(status, "closed" | "cancelled"),
        Entity::Claim => status != "transmitted",
        Entity::MaintenanceLog => false,
        Entity::VendorContract => status == "draft",
        Entity::Budget => status == "draft",
    }
}

/// Returns `true` when the entity may be deleted.
///
/// For Pattern-A entities deletion follows the same rule as mutation.
pub fn is_deletable(entity: &Entity, status: &str) -> bool {
    is_mutable(entity, status)
}

/// Asserts the entity is still mutable; returns `errors.immutability.frozen` otherwise.
pub fn assert_mutable(entity: &Entity, status: &str) -> AppResult<()> {
    if is_mutable(entity, status) {
        Ok(())
    } else {
        Err(AppError::validation_code("errors.immutability.frozen"))
    }
}

/// Founding documents (règlement de copropriété, EDD) are permanent legal
/// acts (LEGAL §1.2): hard delete is forbidden, replacement only via a new
/// version (`supersedes_id` chain).
pub(crate) fn is_document_deletable(category: &str) -> bool {
    !matches!(category, "reglement_copropriete" | "edd")
}

/// Critical categories require the justified-replacement workflow
/// (US-DOC-007): plain deletion is refused, replacement demands a reason.
pub(crate) fn document_requires_replacement_reason(category: &str) -> bool {
    matches!(category, "reglement_copropriete" | "edd")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levy_call_draft_is_mutable() {
        assert!(is_mutable(&Entity::LevyCall, "draft"));
    }

    #[test]
    fn levy_call_issued_is_frozen() {
        assert!(!is_mutable(&Entity::LevyCall, "issued"));
        assert!(!is_mutable(&Entity::LevyCall, "paid"));
        assert!(!is_mutable(&Entity::LevyCall, "overdue"));
    }

    #[test]
    fn vote_frozen_on_ongoing_and_closed() {
        assert!(!is_mutable(&Entity::Vote, "ongoing"));
        assert!(!is_mutable(&Entity::Vote, "closed"));
        assert!(is_mutable(&Entity::Vote, "draft"));
        assert!(is_mutable(&Entity::Vote, "called"));
    }

    #[test]
    fn resolution_frozen_only_on_closed() {
        assert!(!is_mutable(&Entity::Resolution, "closed"));
        assert!(is_mutable(&Entity::Resolution, "ongoing"));
        assert!(is_mutable(&Entity::Resolution, "draft"));
    }

    #[test]
    fn task_frozen_on_closed() {
        assert!(!is_mutable(&Entity::Task, "closed"));
        assert!(!is_mutable(&Entity::Task, "cancelled"));
        assert!(is_mutable(&Entity::Task, "open"));
        assert!(is_mutable(&Entity::Task, "in_progress"));
    }

    #[test]
    fn claim_frozen_when_transmitted() {
        assert!(!is_mutable(&Entity::Claim, "transmitted"));
        assert!(is_mutable(&Entity::Claim, "open"));
        assert!(is_mutable(&Entity::Claim, "in_progress"));
        assert!(is_mutable(&Entity::Claim, "closed"));
    }

    #[test]
    fn maintenance_log_always_frozen() {
        assert!(!is_mutable(&Entity::MaintenanceLog, "automatic"));
        assert!(!is_mutable(&Entity::MaintenanceLog, "manual"));
    }

    #[test]
    fn vendor_contract_draft_is_mutable() {
        assert!(is_mutable(&Entity::VendorContract, "draft"));
        assert!(!is_mutable(&Entity::VendorContract, "active"));
        assert!(!is_mutable(&Entity::VendorContract, "terminated"));
        assert!(!is_mutable(&Entity::VendorContract, "expired"));
    }

    #[test]
    fn budget_frozen_once_approved() {
        assert!(is_mutable(&Entity::Budget, "draft"));
        assert!(!is_mutable(&Entity::Budget, "approved"));
        assert!(!is_mutable(&Entity::Budget, "closed"));
    }

    #[test]
    fn is_deletable_mirrors_is_mutable() {
        for (entity, status, expected) in [
            (&Entity::LevyCall, "issued", false),
            (&Entity::LevyCall, "draft", true),
            (&Entity::Task, "closed", false),
            (&Entity::MaintenanceLog, "automatic", false),
            (&Entity::MaintenanceLog, "manual", false),
        ] {
            assert_eq!(is_deletable(entity, status), expected);
        }
    }

    #[test]
    fn assert_mutable_returns_error_when_frozen() {
        let err = assert_mutable(&Entity::LevyCall, "issued").unwrap_err();
        assert!(
            matches!(err, AppError::Validation(ref msg) if msg == "errors.immutability.frozen")
        );
    }

    #[test]
    fn assert_mutable_ok_when_mutable() {
        assert!(assert_mutable(&Entity::Task, "open").is_ok());
    }

    #[test]
    fn founding_categories_are_not_deletable() {
        assert!(!is_document_deletable("reglement_copropriete"));
        assert!(!is_document_deletable("edd"));
    }

    #[test]
    fn non_founding_category_is_deletable() {
        assert!(is_document_deletable("autre"));
    }

    #[test]
    fn founding_categories_require_replacement() {
        assert!(document_requires_replacement_reason(
            "reglement_copropriete"
        ));
        assert!(document_requires_replacement_reason("edd"));
    }

    #[test]
    fn non_founding_category_does_not_require_replacement() {
        assert!(!document_requires_replacement_reason("autre"));
    }
}
