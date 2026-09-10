//! Legal retention rules per document category (LEGAL §3.4, §6.1-6.3) and
//! the pure purge decision used by the S3 cleanup job (EP-005-S09).

use chrono::{DateTime, Months, Utc};

/// Legal retention duration in months for a document category, `None` when
/// the category must be conserved indefinitely (founding documents, PV d'AG,
/// carnet d'entretien — LEGAL §1.2, §3.4, §6.1).
pub fn retention_months(category: &str) -> Option<u32> {
    match category {
        // Founding acts and AG minutes: unlimited retention.
        "reglement_copropriete" | "edd" | "pv_ag" | "carnet_entretien" => None,
        // Everything else (technical diagnostics, accounting, contracts...):
        // accounting-grade default of 10 years (LEGAL §6.3, art. L123-22 Code
        // de commerce).
        _ => Some(120),
    }
}

/// Returns `true` when a soft-deleted document has passed its category's
/// retention period and may be physically purged (DB row + S3 object).
/// Categories with unlimited retention (`retention_months` returns `None`)
/// are never purgeable.
pub fn is_purgeable(category: &str, deleted_at: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    match retention_months(category) {
        None => false,
        Some(months) => match deleted_at.checked_add_months(Months::new(months)) {
            Some(purge_at) => now >= purge_at,
            None => false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn founding_and_indefinite_categories_have_no_retention_limit() {
        for category in ["reglement_copropriete", "edd", "pv_ag", "carnet_entretien"] {
            assert_eq!(retention_months(category), None);
        }
    }

    #[test]
    fn technical_diagnostics_have_ten_year_retention() {
        assert_eq!(retention_months("diagnostic"), Some(120));
    }

    #[test]
    fn generic_categories_default_to_ten_year_retention() {
        for category in ["comptabilite", "appel_fonds", "contrat", "autre"] {
            assert_eq!(retention_months(category), Some(120));
        }
    }

    #[test]
    fn indefinite_category_is_never_purgeable() {
        let now = Utc::now();
        let long_ago = now - Duration::days(365 * 50);
        assert!(!is_purgeable("pv_ag", long_ago, now));
    }

    #[test]
    fn finite_retention_not_yet_elapsed_is_not_purgeable() {
        let now = Utc::now();
        let deleted_at = now - Duration::days(30);
        assert!(!is_purgeable("diagnostic", deleted_at, now));
    }

    #[test]
    fn finite_retention_elapsed_is_purgeable() {
        let now = Utc::now();
        let deleted_at = now - Duration::days(11 * 365);
        assert!(is_purgeable("diagnostic", deleted_at, now));
    }

    #[test]
    fn finite_retention_exactly_at_boundary_is_purgeable() {
        let now = Utc::now();
        let deleted_at = now.checked_sub_months(Months::new(120)).unwrap();
        assert!(is_purgeable("autre", deleted_at, now));
    }
}
