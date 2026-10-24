/// Background job: polls pending contract reminders.
///
/// Principle:
///   - Loops every hour (tokio::time::interval)
///   - Fetches reminders with status='pending' AND reminder_date <= today
///   - Marks them 'sent' (real notifications will come post-MVP)
///   - Structured logging for external monitoring
use std::sync::Arc;
use tokio::time::{Duration, interval};
use tracing::{error, info};

use crate::db::AppState;

/// Maximum number of reminders processed per run (safety bound).
const MAX_REMINDERS_PER_RUN: i64 = 100;

pub async fn run(state: Arc<AppState>) {
    let mut ticker = interval(Duration::from_secs(3600));
    loop {
        ticker.tick().await;
        if let Err(e) = process_reminders(&state).await {
            error!(error = %e, "Contract reminder job error");
        }
    }
}

async fn process_reminders(state: &Arc<AppState>) -> anyhow::Result<()> {
    let reminders = sqlx::query!(
        r#"
        SELECT cr.id, cr.reminder_type::text as reminder_type,
               cr.reminder_date, vc.vendor_name, vc.subject,
               vc.building_id
        FROM contract_reminder cr
        JOIN vendor_contract vc ON vc.id = cr.contract_id
        WHERE cr.status = $2::text::reminder_status
          AND cr.reminder_date <= CURRENT_DATE
        ORDER BY cr.reminder_date
        LIMIT $3
        "#
    )
    .bind(crate::db::contract::reminder_status::PENDING)
    .bind(MAX_REMINDERS_PER_RUN)
    .fetch_all(&state.pool)
    .await?;

    if reminders.is_empty() {
        return Ok(());
    }

    info!(count = reminders.len(), "Processing contract reminders");

    for reminder in &reminders {
        // TODO post-MVP: trigger email notification
        info!(
            reminder_id = %reminder.id,
            reminder_type = %reminder.reminder_type.as_deref().unwrap_or("?"),
            vendor = %reminder.vendor_name,
            subject = %reminder.subject,
            "Contract reminder triggered"
        );

        sqlx::query!(
            "UPDATE contract_reminder SET status = $2::text::reminder_status, updated_at = NOW() WHERE id = $1",
            reminder.id,
            crate::db::contract::reminder_status::SENT
        )
        .execute(&state.pool)
        .await?;
    }

    Ok(())
}
