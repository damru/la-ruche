use crate::errors::AppResult;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub async fn list(pool: &PgPool, building_id: Uuid) -> AppResult<Vec<serde_json::Value>> {
    let rows = sqlx::query(
        r#"SELECT id, title, description, status::text AS status,
           visible_to_roles::text[] AS visible_to_roles, closed_at, created_at
           FROM consultation WHERE building_id = $1 ORDER BY created_at DESC"#,
    )
    .bind(building_id)
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
                "closed_at": r.try_get::<Option<DateTime<Utc>>, _>("closed_at")?,
                "created_at": r.try_get::<DateTime<Utc>, _>("created_at")?,
            }))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

pub async fn get_with_choices(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
) -> AppResult<Option<serde_json::Value>> {
    let row = sqlx::query(
        r#"SELECT id, title, description, status::text AS status,
           visible_to_roles::text[] AS visible_to_roles, closed_at, created_at
           FROM consultation WHERE id = $1 AND building_id = $2"#,
    )
    .bind(id)
    .bind(building_id)
    .fetch_optional(pool)
    .await?;

    let Some(r) = row else { return Ok(None) };

    let choices = sqlx::query(
        "SELECT id, label, order_index FROM consultation_choice WHERE consultation_id = $1 ORDER BY order_index",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    Ok(Some(serde_json::json!({
        "id": r.try_get::<Uuid, _>("id")?,
        "title": r.try_get::<String, _>("title")?,
        "description": r.try_get::<Option<String>, _>("description")?,
        "status": r.try_get::<String, _>("status")?,
        "visible_to_roles": r.try_get::<Vec<String>, _>("visible_to_roles")?,
        "closed_at": r.try_get::<Option<DateTime<Utc>>, _>("closed_at")?,
        "created_at": r.try_get::<DateTime<Utc>, _>("created_at")?,
        "choices": choices.iter().map(|c| -> Result<_, sqlx::Error> {
            Ok(serde_json::json!({
                "id": c.try_get::<Uuid, _>("id")?,
                "label": c.try_get::<String, _>("label")?,
                "order_index": c.try_get::<i32, _>("order_index")?,
            }))
        }).collect::<Result<Vec<_>, _>>()?,
    })))
}

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

    // Create as 'open' so it is immediately visible and respondable
    let id: Uuid = sqlx::query(
        r#"INSERT INTO consultation (building_id, created_by, title, description,
           visible_to_roles, status, closed_at)
           VALUES ($1, $2, $3, $4, $5::user_role[], 'open'::poll_status, $6) RETURNING id"#,
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

pub async fn respond(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    user_id: Uuid,
    choice_id: Uuid,
    comment: Option<&str>,
) -> AppResult<()> {
    // Verify consultation is open and choice belongs to this consultation
    let valid_choice = sqlx::query(
        r#"SELECT cc.id FROM consultation_choice cc
           JOIN consultation c ON c.id = cc.consultation_id
           WHERE cc.id = $1 AND cc.consultation_id = $2
             AND c.building_id = $3 AND c.status = 'open'"#,
    )
    .bind(choice_id)
    .bind(id)
    .bind(building_id)
    .fetch_optional(pool)
    .await?;

    if valid_choice.is_none() {
        return Err(crate::errors::AppError::Conflict(
            "Consultation not found, closed, or choice does not belong to it".into(),
        ));
    }

    sqlx::query(
        r#"INSERT INTO consultation_response (consultation_id, choice_id, user_id, comment)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (consultation_id, user_id)
           DO UPDATE SET choice_id = EXCLUDED.choice_id, comment = EXCLUDED.comment"#,
    )
    .bind(id)
    .bind(choice_id)
    .bind(user_id)
    .bind(comment)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn results(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<serde_json::Value> {
    let rows = sqlx::query(
        r#"SELECT cc.id, cc.label, COUNT(cr.id) AS votes
           FROM consultation_choice cc
           LEFT JOIN consultation_response cr ON cr.choice_id = cc.id
           JOIN consultation c ON c.id = cc.consultation_id AND c.building_id = $2
           WHERE cc.consultation_id = $1
           GROUP BY cc.id, cc.label, cc.order_index ORDER BY cc.order_index"#,
    )
    .bind(id)
    .bind(building_id)
    .fetch_all(pool)
    .await?;

    use sqlx::Row;
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

    let total: i64 = parsed.iter().map(|(_, _, v)| v).sum();
    Ok(serde_json::json!({
        "total": total,
        "choices": parsed.iter().map(|(choice_id, label, votes)| serde_json::json!({
            "id": choice_id,
            "label": label,
            "votes": votes,
            "pct": if total > 0 { votes * 100 / total } else { 0 }
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
