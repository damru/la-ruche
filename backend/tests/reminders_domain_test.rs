// Domain unit tests — contract reminder computation.
// These tests exercise pure domain logic and do NOT require a database.
// Run with: cargo test --test reminders_domain_test

use chrono::NaiveDate;
use laruche::domain::reminders::{ContractReminderSpec, ReminderKind, compute_reminders};

#[test]
fn test_annual_contract_generates_renewal_and_expiration_reminders() {
    let today = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let contract = ContractReminderSpec {
        start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        end_date: Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
        auto_renewal: true,
        notice_days: Some(90),
        payment_frequency_months: Some(12),
    };

    let reminders = compute_reminders(&contract, today);
    assert!(
        reminders
            .iter()
            .any(|reminder| reminder.kind == ReminderKind::Renewal),
        "Expected a renewal reminder"
    );
    assert!(
        reminders
            .iter()
            .any(|reminder| reminder.kind == ReminderKind::Expiration),
        "Expected an expiration reminder"
    );
}

#[test]
fn test_reminders_never_return_past_dates() {
    let today = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let contract = ContractReminderSpec {
        start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        end_date: Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
        auto_renewal: true,
        notice_days: Some(30),
        payment_frequency_months: Some(3),
    };

    let reminders = compute_reminders(&contract, today);
    for reminder in &reminders {
        assert!(
            reminder.reminder_date >= today,
            "Reminder {:?} must not be in the past",
            reminder.reminder_date
        );
    }
}
