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
    let reminder_rows = sqlx::query(
        r#"
        SELECT cr.id, cr.reminder_type::text as reminder_type,
               cr.reminder_date, vc.vendor_name, vc.subject,
               vc.building_id
        FROM contract_reminder cr
        JOIN vendor_contract vc ON vc.id = cr.contract_id
        WHERE cr.status = $1::text::reminder_status
          AND cr.reminder_date <= CURRENT_DATE
        ORDER BY cr.reminder_date
        LIMIT $2
        "#,
    )
    .bind(crate::db::contract::reminder_status::PENDING)
    .bind(MAX_REMINDERS_PER_RUN)
    .fetch_all(&state.pool)
    .await?;

    if reminder_rows.is_empty() {
        return Ok(());
    }

    info!(count = reminder_rows.len(), "Processing contract reminders");

    for row in &reminder_rows {
        use sqlx::Row;
        let reminder_id: uuid::Uuid = row.try_get("id")?;
        let reminder_type: Option<String> = row.try_get("reminder_type").ok();
        let vendor_name: String = row.try_get("vendor_name")?;
        let subject: String = row.try_get("subject")?;

        // TODO post-MVP: trigger email notification
        info!(
            reminder_id = %reminder_id,
            reminder_type = %reminder_type.as_deref().unwrap_or("?"),
            vendor = %vendor_name,
            subject = %subject,
            "Contract reminder triggered"
        );

        sqlx::query(
            "UPDATE contract_reminder SET status = $2::text::reminder_status, updated_at = NOW() WHERE id = $1",
        )
        .bind(reminder_id)
        .bind(crate::db::contract::reminder_status::SENT)
        .execute(&state.pool)
        .await?;
    }

    Ok(())
}
