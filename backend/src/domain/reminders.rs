/// Calculation of reminder dates for vendor contracts.
/// Pure logic -- no I/O.
use chrono::{Days, Months, NaiveDate};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReminderKind {
    Renewal,
    PaymentDue,
    Expiration,
    Termination,
}

#[derive(Debug, Clone)]
pub struct PendingReminder {
    pub kind: ReminderKind,
    pub reminder_date: NaiveDate,
}

#[derive(Debug, Clone)]
pub struct ContractReminderSpec {
    pub end_date: Option<NaiveDate>,
    pub auto_renewal: bool,
    pub notice_days: Option<u32>,
    pub payment_frequency_months: Option<u32>,
    pub start_date: NaiveDate,
}

/// Generates all reminders to create/recreate for a contract.
/// Called at contract creation and on every subsequent update.
pub fn compute_reminders(
    contract: &ContractReminderSpec,
    today: NaiveDate,
) -> Vec<PendingReminder> {
    let mut reminders = Vec::new();

    if let Some(end_date) = contract.end_date
        && end_date > today
    {
        // Expiration reminder -- 30 days before end_date
        if let Some(date) = end_date.checked_sub_days(Days::new(30)) {
            reminders.push(PendingReminder {
                kind: ReminderKind::Expiration,
                reminder_date: date.max(today),
            });
        }

        if contract.auto_renewal
            && let Some(notice) = contract.notice_days
        {
            // Termination reminder = end_date - notice - 15-day margin
            let margin = notice as u64 + 15;
            if let Some(date) = end_date.checked_sub_days(Days::new(margin))
                && date > today
            {
                reminders.push(PendingReminder {
                    kind: ReminderKind::Termination,
                    reminder_date: date,
                });
            }

            // Renewal reminder = end_date - notice
            if let Some(date) = end_date.checked_sub_days(Days::new(notice as u64))
                && date > today
            {
                reminders.push(PendingReminder {
                    kind: ReminderKind::Renewal,
                    reminder_date: date,
                });
            }
        }
    }

    // Payment due reminders
    if let Some(freq_months) = contract.payment_frequency_months {
        let effective_end = contract
            .end_date
            .unwrap_or_else(|| today.checked_add_months(Months::new(12)).unwrap_or(today));

        let mut next = contract.start_date;
        while next <= effective_end {
            if next > today
                && let Some(reminder_date) = next.checked_sub_days(Days::new(7))
                && reminder_date > today
            {
                reminders.push(PendingReminder {
                    kind: ReminderKind::PaymentDue,
                    reminder_date,
                });
            }
            match next.checked_add_months(Months::new(freq_months)) {
                Some(n) => next = n,
                None => break,
            }
        }
    }

    reminders
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_renewal_generates_reminders() {
        let today = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let spec = ContractReminderSpec {
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            end_date: Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
            auto_renewal: true,
            notice_days: Some(90),
            payment_frequency_months: Some(12),
        };
        let reminders = compute_reminders(&spec, today);
        assert!(reminders.iter().any(|r| r.kind == ReminderKind::Renewal));
        assert!(
            reminders
                .iter()
                .any(|r| r.kind == ReminderKind::Termination)
        );
        assert!(reminders.iter().any(|r| r.kind == ReminderKind::Expiration));
    }
}
