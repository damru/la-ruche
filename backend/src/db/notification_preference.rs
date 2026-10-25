// ── db/notification_preference.rs — per-user notification overrides ──

use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::errors::AppResult;

/// Returns the user's stored overrides as (event_key, enabled).
pub async fn list(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<(String, bool)>> {
    let rows = sqlx::query(
        "SELECT event_key, enabled FROM user_notification_preference WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let key: String = r.get("event_key");
            let enabled: bool = r.get("enabled");
            (key, enabled)
        })
        .collect())
}

/// Upserts the given optional-event values in one transaction
/// (INSERT ... ON CONFLICT (user_id, event_key) DO UPDATE).
pub async fn upsert(pool: &PgPool, user_id: Uuid, prefs: &[(String, bool)]) -> AppResult<()> {
    let mut tx = pool.begin().await?;
    for (key, enabled) in prefs {
        sqlx::query(
            r#"INSERT INTO user_notification_preference (user_id, event_key, enabled)
               VALUES ($1, $2, $3)
               ON CONFLICT (user_id, event_key)
               DO UPDATE SET enabled = EXCLUDED.enabled, updated_at = NOW()"#,
        )
        .bind(user_id)
        .bind(key)
        .bind(*enabled)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}
