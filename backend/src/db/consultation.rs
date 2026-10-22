use crate::errors::AppResult;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// List all consultations for a building, embedding choices (with live vote counts),
/// total response count, and the current user's selected choice IDs.
pub async fn list(
    pool: &PgPool,
    building_id: Uuid,
    user_id: Uuid,
) -> AppResult<Vec<serde_json::Value>> {
    // Each consultation row includes:
    //  - choices: [{id, label, order_index, votes}]
    //  - total_responses: number of distinct voters
    //  - my_choice_ids: UUIDs (as text) of choices the current user has selected
    let rows = sqlx::query(
        r#"SELECT
           c.id, c.title, c.description, c.status::text AS status,
           c.visible_to_roles::text[] AS visible_to_roles, c.closed_at, c.created_at, c.updated_at,
           c.created_by,
           total_r.cnt AS total_responses,
           lr.last_reminder_at,
           COALESCE(my_c.choice_ids, '[]'::json) AS my_choice_ids,
           COALESCE(choices_agg.data, '[]'::json) AS choices
           FROM consultation c
           JOIN LATERAL (
               SELECT MAX(created_at) AS last_reminder_at
               FROM consultation_reminder cr0
               WHERE cr0.consultation_id = c.id
           ) lr ON true
           JOIN LATERAL (
               SELECT COUNT(*) AS cnt
               FROM consultation_response cr
               WHERE cr.consultation_id = c.id
           ) total_r ON true
           JOIN LATERAL (
               SELECT COALESCE(JSON_AGG(crc.choice_id::text), '[]'::json) AS choice_ids
               FROM consultation_response cr2
               JOIN consultation_response_choice crc ON crc.response_id = cr2.id
               WHERE cr2.consultation_id = c.id AND cr2.user_id = $2
           ) my_c ON true
           JOIN LATERAL (
               SELECT COALESCE(JSON_AGG(JSON_BUILD_OBJECT(
                   'id',          cc.id,
                   'label',       cc.label,
                   'order_index', cc.order_index,
                   'votes',       COALESCE(vc.cnt, 0)::int
               ) ORDER BY cc.order_index), '[]'::json) AS data
               FROM consultation_choice cc
               LEFT JOIN (
                   SELECT crc2.choice_id, COUNT(*) AS cnt
                   FROM consultation_response cr3
                   JOIN consultation_response_choice crc2 ON crc2.response_id = cr3.id
                   WHERE cr3.consultation_id = c.id
                   GROUP BY crc2.choice_id
               ) vc ON vc.choice_id = cc.id
               WHERE cc.consultation_id = c.id
           ) choices_agg ON true
           WHERE c.building_id = $1
             AND (
                 c.status <> 'draft'
                 OR c.created_by = $2
                 OR EXISTS (
                     SELECT 1 FROM user_building ub
                     WHERE ub.user_id = $2 AND ub.building_id = c.building_id
                       AND ub.roles && ARRAY['manager'::user_role, 'board_member'::user_role]
                       AND (ub.end_date IS NULL OR ub.end_date >= CURRENT_DATE)
                 )
             )
           ORDER BY c.updated_at DESC"#,
    )
    .bind(building_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    rows.iter()
        .map(|r| -> Result<_, sqlx::Error> {
            Ok(serde_json::json!({
                "id": r.try_get::<Uuid, _>("id")?,
                "title": r.try_get::<String, _>("title")?,
                "description": r.try_get::<Option<String>, _>("description")?,
                "status": r.try_get::<String, _>("status")?,
                 "visible_to_roles": r.try_get::<Vec<String>, _>("visible_to_roles")?,
                 "created_by": r.try_get::<Uuid, _>("created_by")?,
                 "closed_at": r.try_get::<Option<DateTime<Utc>>, _>("closed_at")?,
                 "created_at": r.try_get::<DateTime<Utc>, _>("created_at")?,
                 "updated_at": r.try_get::<DateTime<Utc>, _>("updated_at")?,
                 "total_responses": r.try_get::<i64, _>("total_responses")?,
                 "last_reminder_at": r.try_get::<Option<DateTime<Utc>>, _>("last_reminder_at")?,
                 "my_choice_ids": r.try_get::<serde_json::Value, _>("my_choice_ids")?,
                 "choices": r.try_get::<serde_json::Value, _>("choices")?,
            }))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

/// Returns a single consultation with the same full payload as `list`:
/// choices include live vote counts + percentages; my_choice_ids and
/// total_responses are also included so callers don't need extra round-trips.
pub async fn get_with_choices(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    user_id: Uuid,
) -> AppResult<Option<serde_json::Value>> {
    let row = sqlx::query(
        r#"SELECT
           c.id, c.title, c.description, c.status::text AS status,
           c.visible_to_roles::text[] AS visible_to_roles, c.closed_at, c.created_at, c.updated_at,
           c.created_by,
           total_r.cnt AS total_responses,
           lr.last_reminder_at,
           COALESCE(my_c.choice_ids, '[]'::json) AS my_choice_ids,
           COALESCE(choices_agg.data, '[]'::json) AS choices
           FROM consultation c
           JOIN LATERAL (
               SELECT MAX(created_at) AS last_reminder_at
               FROM consultation_reminder cr0
               WHERE cr0.consultation_id = c.id
           ) lr ON true
           JOIN LATERAL (
               SELECT COUNT(*) AS cnt
               FROM consultation_response cr
               WHERE cr.consultation_id = c.id
           ) total_r ON true
           JOIN LATERAL (
               SELECT COALESCE(JSON_AGG(crc.choice_id::text), '[]'::json) AS choice_ids
               FROM consultation_response cr2
               JOIN consultation_response_choice crc ON crc.response_id = cr2.id
               WHERE cr2.consultation_id = c.id AND cr2.user_id = $3
           ) my_c ON true
           JOIN LATERAL (
               SELECT COALESCE(JSON_AGG(JSON_BUILD_OBJECT(
                   'id',          cc.id,
                   'label',       cc.label,
                   'order_index', cc.order_index,
                   'votes',       COALESCE(vc.cnt, 0)::int
               ) ORDER BY cc.order_index), '[]'::json) AS data
               FROM consultation_choice cc
               LEFT JOIN (
                   SELECT crc2.choice_id, COUNT(*) AS cnt
                   FROM consultation_response cr3
                   JOIN consultation_response_choice crc2 ON crc2.response_id = cr3.id
                   WHERE cr3.consultation_id = c.id
                   GROUP BY crc2.choice_id
               ) vc ON vc.choice_id = cc.id
               WHERE cc.consultation_id = c.id
           ) choices_agg ON true
           WHERE c.id = $1 AND c.building_id = $2"#,
    )
    .bind(id)
    .bind(building_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let Some(r) = row else { return Ok(None) };

    Ok(Some(serde_json::json!({
        "id": r.try_get::<Uuid, _>("id")?,
        "title": r.try_get::<String, _>("title")?,
        "description": r.try_get::<Option<String>, _>("description")?,
        "status": r.try_get::<String, _>("status")?,
                "visible_to_roles": r.try_get::<Vec<String>, _>("visible_to_roles")?,
                "created_by": r.try_get::<Uuid, _>("created_by")?,
                "closed_at": r.try_get::<Option<DateTime<Utc>>, _>("closed_at")?,
                "created_at": r.try_get::<DateTime<Utc>, _>("created_at")?,
                "updated_at": r.try_get::<DateTime<Utc>, _>("updated_at")?,
                "total_responses": r.try_get::<i64, _>("total_responses")?,
        "last_reminder_at": r.try_get::<Option<DateTime<Utc>>, _>("last_reminder_at")?,
        "my_choice_ids": r.try_get::<serde_json::Value, _>("my_choice_ids")?,
        "choices": r.try_get::<serde_json::Value, _>("choices")?,
    })))
}

/// Creates a consultation in 'draft' status (EP-010-S04) with its choices.
#[allow(clippy::too_many_arguments)]
pub async fn create(
    pool: &PgPool,
    building_id: Uuid,
    user_id: Uuid,
    title: &str,
    description: Option<&str>,
    visible_to_roles: &[String],
    closed_at: Option<DateTime<Utc>>,
    choices: &[String],
) -> AppResult<serde_json::Value> {
    let mut tx = pool.begin().await?;

    let id: Uuid = sqlx::query(
        r#"INSERT INTO consultation (building_id, created_by, title, description,
           visible_to_roles, status, closed_at)
           VALUES ($1, $2, $3, $4, $5::user_role[], 'draft'::poll_status, $6) RETURNING id"#,
    )
    .bind(building_id)
    .bind(user_id)
    .bind(title)
    .bind(description)
    .bind(visible_to_roles)
    .bind(closed_at)
    .fetch_one(&mut *tx)
    .await?
    .try_get("id")?;

    for (i, label) in choices.iter().enumerate() {
        sqlx::query(
            "INSERT INTO consultation_choice (consultation_id, label, order_index) VALUES ($1, $2, $3)",
        )
        .bind(id)
        .bind(label)
        .bind(i as i32)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(serde_json::json!({ "id": id, "title": title }))
}

/// Updates a draft consultation (title, description, visible roles,
/// closed_at) and replaces its choices. 404 if not found or not draft.
/// Safe to replace choices: no response can reference a draft.
#[allow(clippy::too_many_arguments)]
pub async fn update_draft(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    title: &str,
    description: Option<&str>,
    visible_to_roles: &[String],
    closed_at: Option<DateTime<Utc>>,
    choices: &[String],
) -> AppResult<()> {
    let mut tx = pool.begin().await?;

    let result = sqlx::query(
        r#"UPDATE consultation
           SET title=$3, description=$4, visible_to_roles=$5::user_role[], closed_at=$6
           WHERE id=$1 AND building_id=$2 AND status='draft'"#,
    )
    .bind(id)
    .bind(building_id)
    .bind(title)
    .bind(description)
    .bind(visible_to_roles)
    .bind(closed_at)
    .execute(&mut *tx)
    .await?;
    if result.rows_affected() == 0 {
        return Err(crate::errors::AppError::NotFound);
    }

    sqlx::query("DELETE FROM consultation_choice WHERE consultation_id=$1")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    for (i, label) in choices.iter().enumerate() {
        sqlx::query(
            "INSERT INTO consultation_choice (consultation_id, label, order_index) VALUES ($1, $2, $3)",
        )
        .bind(id)
        .bind(label)
        .bind(i as i32)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Submit or update a multi-choice response.
/// Each user may select one or more choices; calling again replaces the previous selection.
pub async fn respond(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    user_id: Uuid,
    choice_ids: &[Uuid],
    comment: Option<&str>,
) -> AppResult<()> {
    // A response is only possible on an open consultation (EP-010-S04 guard).
    let status: Option<String> =
        sqlx::query_scalar("SELECT status::text FROM consultation WHERE id=$1 AND building_id=$2")
            .bind(id)
            .bind(building_id)
            .fetch_optional(pool)
            .await?;
    match status.as_deref() {
        None => return Err(crate::errors::AppError::NotFound),
        Some("open") => {}
        _ => {
            return Err(crate::errors::AppError::bad_request_code(
                "errors.consultations.not_open",
            ));
        }
    }

    // Validate that every requested choice belongs to this consultation
    // in a single query rather than one round-trip per choice.
    let distinct_ids: std::collections::HashSet<Uuid> = choice_ids.iter().copied().collect();
    let valid_count: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM consultation_choice cc
           JOIN consultation c ON c.id = cc.consultation_id
           WHERE cc.id = ANY($1) AND c.id = $2 AND c.building_id = $3"#,
    )
    .bind(choice_ids)
    .bind(id)
    .bind(building_id)
    .fetch_one(pool)
    .await?;

    if valid_count as usize != distinct_ids.len() {
        return Err(crate::errors::AppError::conflict_code(
            "errors.consultations.not_found_or_closed",
        ));
    }

    let mut tx = pool.begin().await?;

    // Upsert response header (comment); get the response ID
    let response_id: Uuid = sqlx::query(
        r#"INSERT INTO consultation_response (consultation_id, user_id, comment)
           VALUES ($1, $2, $3)
           ON CONFLICT (consultation_id, user_id)
           DO UPDATE SET comment = EXCLUDED.comment
           RETURNING id"#,
    )
    .bind(id)
    .bind(user_id)
    .bind(comment)
    .fetch_one(&mut *tx)
    .await?
    .try_get("id")?;

    // Replace selected choices: delete existing, insert new
    sqlx::query("DELETE FROM consultation_response_choice WHERE response_id = $1")
        .bind(response_id)
        .execute(&mut *tx)
        .await?;

    for &choice_id in choice_ids {
        sqlx::query(
            "INSERT INTO consultation_response_choice (response_id, choice_id) VALUES ($1, $2)",
        )
        .bind(response_id)
        .bind(choice_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn results(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<serde_json::Value> {
    let rows = sqlx::query(
        r#"SELECT cc.id, cc.label, COUNT(crc.response_id) AS votes
           FROM consultation_choice cc
           LEFT JOIN consultation_response_choice crc ON crc.choice_id = cc.id
           JOIN consultation c ON c.id = cc.consultation_id AND c.building_id = $2
           WHERE cc.consultation_id = $1
           GROUP BY cc.id, cc.label, cc.order_index ORDER BY cc.order_index"#,
    )
    .bind(id)
    .bind(building_id)
    .fetch_all(pool)
    .await?;

    // Count total voters (distinct response records)
    let total_row = sqlx::query(
        r#"SELECT COUNT(DISTINCT cr.id) AS total
           FROM consultation_response cr
           JOIN consultation c ON c.id = cr.consultation_id AND c.building_id = $2
           WHERE cr.consultation_id = $1"#,
    )
    .bind(id)
    .bind(building_id)
    .fetch_one(pool)
    .await?;
    let total: i64 = total_row.try_get("total")?;

    let parsed: Vec<(Uuid, String, i64)> = rows
        .iter()
        .map(|r| {
            Ok((
                r.try_get::<Uuid, _>("id")?,
                r.try_get::<String, _>("label")?,
                r.try_get::<Option<i64>, _>("votes")?.unwrap_or(0),
            ))
        })
        .collect::<Result<_, sqlx::Error>>()?;

    Ok(serde_json::json!({
        "total": total,
        "choices": parsed.iter().map(|(choice_id, label, votes)| serde_json::json!({
            "id": choice_id,
            "label": label,
            "votes": votes
        })).collect::<Vec<_>>()
    }))
}

/// Closes an open consultation. Returns 404 if not found or already closed.
pub async fn close(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE consultation SET status='closed', closed_at=NOW(), updated_at=NOW()
         WHERE id=$1 AND building_id=$2 AND status='open'",
    )
    .bind(id)
    .bind(building_id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(crate::errors::AppError::NotFound);
    }
    Ok(())
}

/// Opens a draft consultation. Returns 404 if not found or not in draft state.
pub async fn open(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE consultation SET status='open', updated_at=NOW()
         WHERE id=$1 AND building_id=$2 AND status='draft'",
    )
    .bind(id)
    .bind(building_id)
    .execute(pool)
    .await?;
    if result.rows_affected() == 0 {
        return Err(crate::errors::AppError::NotFound);
    }
    Ok(())
}

/// Deletes a closed consultation (or a draft, EP-010-S04) and all its related
/// data (choices, responses). Returns 404 if not found or not deletable.
pub async fn delete(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<()> {
    // Delete in dependency order: response_choices → responses → choices → consultation
    let mut tx = pool.begin().await?;

    // Verify the consultation exists and is closed or draft
    let row = sqlx::query(
        "SELECT id FROM consultation WHERE id=$1 AND building_id=$2 AND status IN ('closed','draft')",
    )
    .bind(id)
    .bind(building_id)
    .fetch_optional(&mut *tx)
    .await?;

    if row.is_none() {
        return Err(crate::errors::AppError::NotFound);
    }

    // Delete response_choices (cascade not set, delete explicitly via responses)
    sqlx::query(
        r#"DELETE FROM consultation_response_choice
           WHERE response_id IN (
               SELECT id FROM consultation_response WHERE consultation_id=$1
           )"#,
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;

    // Delete responses
    sqlx::query("DELETE FROM consultation_response WHERE consultation_id=$1")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    // Delete choices
    sqlx::query("DELETE FROM consultation_choice WHERE consultation_id=$1")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    // Delete the consultation itself
    sqlx::query("DELETE FROM consultation WHERE id=$1 AND building_id=$2")
        .bind(id)
        .bind(building_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

/// Users expected to respond (active building members whose roles intersect
/// visible_to_roles) who have no response yet. Returns (user_id, first_name,
/// last_name) rows. 404 upstream if the consultation doesn't exist.
pub async fn non_respondents(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
) -> AppResult<Vec<serde_json::Value>> {
    let rows = sqlx::query(
        r#"SELECT u.id, u.first_name, u.last_name
           FROM consultation c
           JOIN user_building ub ON ub.building_id = c.building_id
           JOIN user_account u ON u.id = ub.user_id AND u.active
           WHERE c.id = $1 AND c.building_id = $2
             AND ub.roles && c.visible_to_roles
             AND (ub.end_date IS NULL OR ub.end_date >= CURRENT_DATE)
             AND NOT EXISTS (
                 SELECT 1 FROM consultation_response r
                 WHERE r.consultation_id = c.id AND r.user_id = u.id
             )
           ORDER BY u.last_name, u.first_name"#,
    )
    .bind(id)
    .bind(building_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|r| -> Result<_, sqlx::Error> {
            Ok(serde_json::json!({
                "id": r.try_get::<Uuid, _>("id")?,
                "first_name": r.try_get::<String, _>("first_name")?,
                "last_name": r.try_get::<String, _>("last_name")?,
            }))
        })
        .collect::<Result<Vec<_>, _>>()?)
}

/// Records a reminder for an OPEN consultation. Guards status='open'
/// (errors.consultations.not_open otherwise). Returns recipient_count.
/// Zero non-respondents yields errors.consultations.no_non_respondents.
pub async fn remind_non_respondents(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    sent_by: Uuid,
) -> AppResult<i64> {
    let mut tx = pool.begin().await?;

    let status: Option<String> = sqlx::query_scalar(
        "SELECT status::text FROM consultation WHERE id=$1 AND building_id=$2 FOR UPDATE",
    )
    .bind(id)
    .bind(building_id)
    .fetch_optional(&mut *tx)
    .await?;
    match status.as_deref() {
        None => return Err(crate::errors::AppError::NotFound),
        Some("open") => {}
        _ => {
            return Err(crate::errors::AppError::bad_request_code(
                "errors.consultations.not_open",
            ));
        }
    }

    let recipients: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM user_building ub
           JOIN user_account u ON u.id = ub.user_id AND u.active
           JOIN consultation c ON c.building_id = ub.building_id
           WHERE c.id = $1 AND c.building_id = $2
             AND ub.roles && c.visible_to_roles
             AND (ub.end_date IS NULL OR ub.end_date >= CURRENT_DATE)
             AND NOT EXISTS (
                 SELECT 1 FROM consultation_response r
                 WHERE r.consultation_id = c.id AND r.user_id = u.id
             )"#,
    )
    .bind(id)
    .bind(building_id)
    .fetch_one(&mut *tx)
    .await?;

    if recipients == 0 {
        return Err(crate::errors::AppError::bad_request_code(
            "errors.consultations.no_non_respondents",
        ));
    }

    sqlx::query(
        r#"INSERT INTO consultation_reminder (consultation_id, sent_by, recipient_count)
           VALUES ($1, $2, $3)"#,
    )
    .bind(id)
    .bind(sent_by)
    .bind(recipients as i32)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(recipients)
}
