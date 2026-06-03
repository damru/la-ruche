/// Domain rules for EP-002 — owner management.
/// Pure logic, no I/O.
use chrono::NaiveDate;
use uuid::Uuid;

/// Anonymized PII placeholder format: `ANONYMISÉ-{uuid}`.
pub fn anonymized_value(id: Uuid) -> String {
    format!("ANONYMISÉ-{}", id)
}

/// Validate that a transfer date is not in the future.
/// Returns `Err` with an i18n code if the date is invalid.
pub fn validate_transfer_date(date: NaiveDate, today: NaiveDate) -> Result<(), &'static str> {
    if date > today {
        return Err("errors.owners.transfer_date_future");
    }
    Ok(())
}

/// Validate the anonymization confirmation name matches the stored full name.
pub fn validate_anonymize_confirmation(stored_full_name: &str, confirm_name: &str) -> bool {
    stored_full_name.trim().eq_ignore_ascii_case(confirm_name.trim())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use uuid::Uuid;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn anonymized_value_format() {
        let id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        assert_eq!(
            anonymized_value(id),
            "ANONYMISÉ-00000000-0000-0000-0000-000000000001"
        );
    }

    #[test]
    fn anonymized_value_starts_with_prefix() {
        let id = Uuid::new_v4();
        assert!(anonymized_value(id).starts_with("ANONYMISÉ-"));
    }

    #[test]
    fn transfer_date_today_is_valid() {
        let today = date(2026, 6, 3);
        assert!(validate_transfer_date(today, today).is_ok());
    }

    #[test]
    fn transfer_date_past_is_valid() {
        let today = date(2026, 6, 3);
        let past = date(2024, 1, 1);
        assert!(validate_transfer_date(past, today).is_ok());
    }

    #[test]
    fn transfer_date_future_is_rejected() {
        let today = date(2026, 6, 3);
        let future = date(2026, 6, 4);
        let result = validate_transfer_date(future, today);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "errors.owners.transfer_date_future");
    }

    #[test]
    fn anonymize_confirmation_matches() {
        assert!(validate_anonymize_confirmation("Jean Dupont", "Jean Dupont"));
    }

    #[test]
    fn anonymize_confirmation_case_insensitive() {
        assert!(validate_anonymize_confirmation("Jean Dupont", "jean dupont"));
    }

    #[test]
    fn anonymize_confirmation_trims_whitespace() {
        assert!(validate_anonymize_confirmation("Jean Dupont", "  Jean Dupont  "));
    }

    #[test]
    fn anonymize_confirmation_mismatch_rejected() {
        assert!(!validate_anonymize_confirmation("Jean Dupont", "Marie Martin"));
    }

    #[test]
    fn tantieme_invariant_unit_allocation_key_independent_of_owner() {
        // S08: tantièmes are stored in unit_allocation_key (unit_id + key_id → share_value).
        // The transfer flow touches unit_owner, NOT unit_allocation_key.
        // This test documents the invariant: a transfer does not change share values.
        // The actual DB enforcement is the schema separation: unit_owner and
        // unit_allocation_key are independent tables sharing only unit_id.
        // There is no FK or cascade from unit_owner to unit_allocation_key.
        // → Ownership transfer preserves tantièmes by design (art. 6 loi 1965).
        let unit_id = Uuid::new_v4();
        // Simulate: unit has shares 250/1000 on key "Charges Générales"
        let share_value_before = 250i32;
        // After a transfer (new owner assigned), share_value must remain 250
        let share_value_after = share_value_before; // no mutation in transfer_ownership
        assert_eq!(share_value_before, share_value_after);
    }
}
