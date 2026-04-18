/// Background job: polls pending contract reminders.
///
/// Principle:
///   - Loops every hour (tokio::time::interval)
///   - Fetches reminders with status='pending' AND reminder_date <= today
///   - Marks them 'sent' (real notifications will come post-MVP)
///   - Structured logging for external monitoring

use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info};

use crate::db::AppState;

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
        WHERE cr.status = 'pending'
          AND cr.reminder_date <= CURRENT_DATE
        ORDER BY cr.reminder_date
        LIMIT 100
        "#
    )
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
            "UPDATE contract_reminder SET status = 'sent', updated_at = NOW() WHERE id = $1",
            reminder.id
        )
        .execute(&state.pool)
        .await?;
    }

    Ok(())
}
