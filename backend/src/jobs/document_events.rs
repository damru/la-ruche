/// Background job: polls pending document events (outbox-lite pattern).
///
/// Principle:
///   - Loops every 5 minutes (tokio::time::interval)
///   - Fetches document events with processed_at IS NULL
///   - Marks them processed with processed_at = NOW()
///   - Structured logging for external monitoring
///   - MVP consumer only logs; real notification delivery is out of scope
///     (will come with the notifications epic)
///
/// Note: This is a single-instance job (no distributed locking).
///       Multiple backend instances would cause duplicate processing.
use std::sync::Arc;
use tokio::time::{Duration, interval};
use tracing::{error, info};

use crate::db::AppState;

pub async fn run(state: Arc<AppState>) {
    let mut ticker = interval(Duration::from_secs(300));
    loop {
        ticker.tick().await;
        if let Err(e) = process_events(&state).await {
            error!(error = %e, "Document events job error");
        }
    }
}

/// Process pending document events in a single batch.
/// Exported for testing.
pub async fn process_events(state: &Arc<AppState>) -> anyhow::Result<()> {
    let events = sqlx::query!(
        r#"SELECT id, building_id, document_id, event_type, payload
           FROM document_event
           WHERE processed_at IS NULL
           ORDER BY created_at
           LIMIT 100"#,
    )
    .fetch_all(&state.pool)
    .await?;

    if events.is_empty() {
        return Ok(());
    }

    info!(count = events.len(), "Processing document events");

    for ev in &events {
        // MVP consumer: structured log only.
        // Real notification delivery (email, in-app) will come with the notifications epic.
        info!(
            event_id = %ev.id,
            building_id = %ev.building_id,
            document_id = %ev.document_id,
            event_type = %ev.event_type,
            "document event"
        );

        sqlx::query!(
            "UPDATE document_event SET processed_at = NOW() WHERE id = $1",
            ev.id
        )
        .execute(&state.pool)
        .await?;
    }

    Ok(())
}
