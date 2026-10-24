use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, Postgres};
use uuid::Uuid;

use crate::db::audit::{self, NewAuditEntry, entity_type};

/// `allocation_key_type` values (PG enum `allocation_key_type`).
pub mod key_type {
    pub const GENERAL: &str = "general";
    pub const SPECIAL: &str = "special";
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AllocationKeyRow {
    pub id: Uuid,
    pub building_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub key_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UnitShareRow {
    pub unit_id: Uuid,
    pub unit_number: String,
    pub key_id: Uuid,
    pub share_value: i32,
}

#[derive(Debug, sqlx::FromRow)]
pub struct GeneralShareRow {
    pub unit_id: Uuid,
    pub key_id: Uuid,
    pub shares: i32,
}

pub async fn list(
    pool: &PgPool,
    building_id: Uuid,
) -> crate::errors::AppResult<Vec<AllocationKeyRow>> {
    Ok(sqlx::query_as::<_, AllocationKeyRow>(
        "SELECT id, building_id, name, description, key_type::text, created_at \
         FROM allocation_key WHERE building_id = $1 AND deleted_at IS NULL \
         ORDER BY CASE key_type WHEN 'general' THEN 0 ELSE 1 END, name",
    )
    .bind(building_id)
    .fetch_all(pool)
    .await?)
}

pub async fn get(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<Option<AllocationKeyRow>> {
    Ok(sqlx::query_as::<_, AllocationKeyRow>(
        "SELECT id, building_id, name, description, key_type::text, created_at \
         FROM allocation_key WHERE id = $1 AND building_id = $2 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(building_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn create(
    pool: &PgPool,
    building_id: Uuid,
    name: &str,
    description: Option<&str>,
    user_id: Uuid,
    justification: Option<&str>,
) -> crate::errors::AppResult<AllocationKeyRow> {
    let mut tx = pool.begin().await?;

    let row = sqlx::query_as::<_, AllocationKeyRow>(
        "INSERT INTO allocation_key (building_id, name, description) \
         VALUES ($1, $2, $3) \
         RETURNING id, building_id, name, description, key_type::text, created_at",
    )
    .bind(building_id)
    .bind(name)
    .bind(description)
    .fetch_one(&mut *tx)
    .await?;

    audit::insert(
        &mut *tx,
        NewAuditEntry {
            building_id,
            entity_type: entity_type::ALLOCATION_KEY,
            entity_id: row.id,
            operation: "allocation_key.created",
            changed_by: Some(user_id),
            old_values: None,
            new_values: Some(serde_json::json!({
                "name": row.name,
                "description": row.description,
                "justification": justification,
            })),
        },
    )
    .await?;

    tx.commit().await?;
    Ok(row)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    name: &str,
    description: Option<&str>,
    user_id: Uuid,
    justification: Option<&str>,
) -> crate::errors::AppResult<Option<AllocationKeyRow>> {
    let mut tx = pool.begin().await?;

    let before = sqlx::query_as::<_, AllocationKeyRow>(
        "SELECT id, building_id, name, description, key_type::text, created_at \
         FROM allocation_key WHERE id = $1 AND building_id = $2 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(building_id)
    .fetch_optional(&mut *tx)
    .await?;

    let before = match before {
        Some(row) => row,
        None => return Ok(None),
    };

    let row = sqlx::query_as::<_, AllocationKeyRow>(
        "UPDATE allocation_key SET name = $3, description = $4 \
         WHERE id = $1 AND building_id = $2 AND deleted_at IS NULL \
         RETURNING id, building_id, name, description, key_type::text, created_at",
    )
    .bind(id)
    .bind(building_id)
    .bind(name)
    .bind(description)
    .fetch_optional(&mut *tx)
    .await?;

    let row = match row {
        Some(row) => row,
        None => return Ok(None),
    };

    audit::insert(
        &mut *tx,
        NewAuditEntry {
            building_id,
            entity_type: entity_type::ALLOCATION_KEY,
            entity_id: row.id,
            operation: "allocation_key.updated",
            changed_by: Some(user_id),
            old_values: Some(serde_json::json!({
                "name": before.name,
                "description": before.description,
            })),
            new_values: Some(serde_json::json!({
                "name": row.name,
                "description": row.description,
                "justification": justification,
            })),
        },
    )
    .await?;

    tx.commit().await?;
    Ok(Some(row))
}

pub async fn delete(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    user_id: Uuid,
    justification: Option<&str>,
) -> crate::errors::AppResult<bool> {
    let mut tx = pool.begin().await?;

    // Serialize concurrent delete calls on the same key. Without this lock,
    // N racers can all read the same "before" row (fetched outside any
    // lock), all pass the not-yet-deleted check, and all write their own
    // `allocation_key.deleted` audit entry even though only the first
    // UPDATE actually flips deleted_at — corrupting the audit trail with
    // duplicate entries for a single deletion.
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1::text, 0))")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    // Protect the general key — it is the legal accounting foundation
    let key: Option<AllocationKeyRow> = sqlx::query_as::<_, AllocationKeyRow>(
        "SELECT id, building_id, name, description, key_type::text, created_at \
         FROM allocation_key WHERE id = $1 AND building_id = $2 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(building_id)
    .fetch_optional(&mut *tx)
    .await?;

    let key = match key {
        None => {
            tx.commit().await?;
            return Ok(false);
        }
        Some(key) => key,
    };

    if key.key_type == key_type::GENERAL {
        return Err(crate::errors::AppError::conflict_code(
            "errors.allocation_keys.general_key_protected",
        ));
    }

    // Close active shares for this key (ADR-006 versioning invariant)
    sqlx::query(
        "UPDATE unit_allocation_key SET valid_to = CURRENT_DATE \
         WHERE key_id = $1 AND valid_to IS NULL",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;

    // Soft delete the key (preserves levy_call.key_id references)
    let result = sqlx::query(
        "UPDATE allocation_key SET deleted_at = NOW() \
         WHERE id = $1 AND building_id = $2 AND deleted_at IS NULL",
    )
    .bind(id)
    .bind(building_id)
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() == 0 {
        // Another racer already deleted it between our lock acquisition and
        // now — should be unreachable given the advisory lock above, but
        // guard against writing a stray audit entry if it ever happens.
        tx.commit().await?;
        return Ok(false);
    }

    audit::insert(
        &mut *tx,
        NewAuditEntry {
            building_id,
            entity_type: entity_type::ALLOCATION_KEY,
            entity_id: id,
            operation: "allocation_key.deleted",
            changed_by: Some(user_id),
            old_values: Some(serde_json::json!({
                "name": key.name,
                "description": key.description,
                "justification": justification,
            })),
            new_values: None,
        },
    )
    .await?;

    tx.commit().await?;
    Ok(true)
}

pub async fn list_shares(
    pool: &PgPool,
    key_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<Vec<UnitShareRow>> {
    Ok(sqlx::query_as::<_, UnitShareRow>(
        r#"SELECT u.id AS unit_id, u.unit_number,
                  COALESCE(uak.key_id, $1) AS key_id,
                  COALESCE(uak.share_value, 0) AS share_value
           FROM unit u
           LEFT JOIN unit_allocation_key uak
                  ON uak.unit_id = u.id AND uak.key_id = $1 AND uak.valid_to IS NULL
           WHERE u.building_id = $2
             AND u.active = true
           ORDER BY u.unit_number"#,
    )
    .bind(key_id)
    .bind(building_id)
    .fetch_all(pool)
    .await?)
}

pub async fn list_general_shares(
    pool: &PgPool,
    building_id: Uuid,
) -> crate::errors::AppResult<Vec<GeneralShareRow>> {
    Ok(sqlx::query_as::<_, GeneralShareRow>(
        r#"SELECT u.id AS unit_id, ak.id AS key_id, uak.share_value AS shares
                     FROM unit u
                     JOIN unit_allocation_key uak
                         ON uak.unit_id = u.id AND uak.valid_to IS NULL
                     JOIN allocation_key ak
                         ON ak.id = uak.key_id
                        AND ak.key_type = 'general'
                        AND ak.deleted_at IS NULL
                     WHERE u.building_id = $1 AND u.active = true"#,
    )
    .bind(building_id)
    .fetch_all(pool)
    .await?)
}

pub async fn get_unit_general_share(
    pool: &PgPool,
    unit_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<Option<i32>> {
    Ok(sqlx::query_scalar(
        r#"SELECT uak.share_value
                     FROM unit u
                     JOIN unit_allocation_key uak
                         ON uak.unit_id = u.id AND uak.valid_to IS NULL
                     JOIN allocation_key ak
                         ON ak.id = uak.key_id
                        AND ak.key_type = 'general'
                        AND ak.deleted_at IS NULL
                     WHERE u.id = $1 AND u.building_id = $2 AND u.active = true"#,
    )
    .bind(unit_id)
    .bind(building_id)
    .fetch_optional(pool)
    .await?)
}

/// Same lookup, executed on the caller's transaction: the share read
/// participates in the `cast_vote` session-guard lock.
pub(crate) async fn get_unit_general_share_in_tx(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    unit_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<Option<i32>> {
    Ok(sqlx::query_scalar(
        r#"SELECT uak.share_value
                     FROM unit u
                     JOIN unit_allocation_key uak
                         ON uak.unit_id = u.id AND uak.valid_to IS NULL
                     JOIN allocation_key ak
                         ON ak.id = uak.key_id
                        AND ak.key_type = 'general'
                        AND ak.deleted_at IS NULL
                     WHERE u.id = $1 AND u.building_id = $2 AND u.active = true"#,
    )
    .bind(unit_id)
    .bind(building_id)
    .fetch_optional(&mut **tx)
    .await?)
}

/// Returns the current active sum of shares for a key and whether it balances
/// against the building's `shares_total`. Used for EP-001-S07 imbalance guard.
pub async fn shares_balance(
    pool: &PgPool,
    key_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<(i64, i32)> {
    let sum: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(uak.share_value), 0) \
         FROM unit_allocation_key uak \
         JOIN unit u ON u.id = uak.unit_id \
         WHERE uak.key_id = $1 AND u.building_id = $2 AND u.active = true AND uak.valid_to IS NULL",
    )
    .bind(key_id)
    .bind(building_id)
    .fetch_one(pool)
    .await?;

    let total: i32 = sqlx::query_scalar("SELECT shares_total FROM building WHERE id = $1")
        .bind(building_id)
        .fetch_one(pool)
        .await?;

    Ok((sum, total))
}

pub async fn list_unbalanced_active_keys(
    pool: &PgPool,
    building_id: Uuid,
) -> crate::errors::AppResult<Vec<String>> {
    Ok(sqlx::query_scalar::<_, String>(
        r#"SELECT ak.name
                     FROM allocation_key ak
                     JOIN building b ON b.id = ak.building_id
                     INNER JOIN unit_allocation_key uak
                         ON uak.key_id = ak.id
                        AND uak.valid_to IS NULL
                     INNER JOIN unit u
                         ON u.id = uak.unit_id
                        AND u.building_id = ak.building_id
                        AND u.active = true
                     WHERE ak.building_id = $1
                       AND ak.deleted_at IS NULL
                     GROUP BY ak.id, ak.name, b.shares_total
                     HAVING COALESCE(SUM(uak.share_value), 0) != b.shares_total
                     ORDER BY ak.name"#,
    )
    .bind(building_id)
    .fetch_all(pool)
    .await?)
}

/// If the value is unchanged for the active record, this is a no-op.
/// Otherwise: close the current record (valid_to = today) and insert a new one.
pub async fn set_share(
    pool: &PgPool,
    unit_id: Uuid,
    key_id: Uuid,
    share_value: i32,
    building_id: Uuid,
    user_id: Uuid,
    justification: Option<&str>,
) -> crate::errors::AppResult<()> {
    let mut tx = pool.begin().await?;

    // Serialize concurrent set_share calls on the same (unit, key) pair.
    // Without this lock, two concurrent callers can both read the same
    // "current" active row (or both see none), both close it, and both
    // insert a new active row — racing into `uak_unique_active`
    // (unit_allocation_key unique partial index). The advisory lock is
    // released automatically at transaction end (commit or rollback).
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1::text || ':' || $2::text, 0))")
        .bind(unit_id)
        .bind(key_id)
        .execute(&mut *tx)
        .await?;

    // Check current active value to avoid spurious history entries
    let current: Option<i32> = sqlx::query_scalar(
        "SELECT share_value FROM unit_allocation_key \
         WHERE unit_id = $1 AND key_id = $2 AND valid_to IS NULL",
    )
    .bind(unit_id)
    .bind(key_id)
    .fetch_optional(&mut *tx)
    .await?;

    if current == Some(share_value) {
        tx.commit().await?;
        return Ok(());
    }

    // Close the current record if it exists
    sqlx::query(
        "UPDATE unit_allocation_key SET valid_to = CURRENT_DATE \
         WHERE unit_id = $1 AND key_id = $2 AND valid_to IS NULL",
    )
    .bind(unit_id)
    .bind(key_id)
    .execute(&mut *tx)
    .await?;

    // Insert the new active record
    sqlx::query(
        "INSERT INTO unit_allocation_key (unit_id, key_id, share_value) \
         VALUES ($1, $2, $3)",
    )
    .bind(unit_id)
    .bind(key_id)
    .bind(share_value)
    .execute(&mut *tx)
    .await?;

    audit::insert(
        &mut *tx,
        NewAuditEntry {
            building_id,
            entity_type: entity_type::ALLOCATION_KEY,
            entity_id: key_id,
            operation: "allocation_key.share_set",
            changed_by: Some(user_id),
            old_values: Some(serde_json::json!({ "unit_id": unit_id, "share_value": current })),
            new_values: Some(serde_json::json!({
                "unit_id": unit_id,
                "share_value": share_value,
                "justification": justification,
            })),
        },
    )
    .await?;

    tx.commit().await?;
    Ok(())
}
