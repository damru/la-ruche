use crate::errors::AppResult;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list(pool: &PgPool, building_id: Uuid) -> AppResult<Vec<serde_json::Value>> {
    let rows = sqlx::query!(
        r#"SELECT id, title, description, status::text as status,
           visible_to_roles::text[] as visible_to_roles, closed_at, created_at
           FROM consultation WHERE building_id = $1 ORDER BY created_at DESC"#,
        building_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "title": r.title,
                "description": r.description,
                "status": r.status,
                "visible_to_roles": r.visible_to_roles,
                "closed_at": r.closed_at,
                "created_at": r.created_at,
            })
        })
        .collect())
}

pub async fn get_with_choices(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
) -> AppResult<Option<serde_json::Value>> {
    let row = sqlx::query!(
        r#"SELECT id, title, description, status::text as status,
           visible_to_roles::text[] as visible_to_roles, closed_at, created_at
         FROM consultation WHERE id = $1 AND building_id = $2"#,
        id,
        building_id
    )
    .fetch_optional(pool)
    .await?;

    let Some(r) = row else { return Ok(None) };

    let choices = sqlx::query!(
        "SELECT id, label, order_index FROM consultation_choice WHERE consultation_id = $1 ORDER BY order_index",
        id
    ).fetch_all(pool).await?;

    Ok(Some(serde_json::json!({
        "id": r.id,
        "title": r.title,
        "description": r.description,
        "status": r.status,
        "visible_to_roles": r.visible_to_roles,
        "closed_at": r.closed_at,
        "created_at": r.created_at,
        "choices": choices.iter().map(|c| serde_json::json!({
            "id": c.id, "label": c.label, "order_index": c.order_index
        })).collect::<Vec<_>>(),
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

    let id = sqlx::query_scalar!(
        r#"INSERT INTO consultation (building_id, created_by, title, description, visible_to_roles, closed_at)
           VALUES ($1, $2, $3, $4, $5::user_role[], $6) RETURNING id"#,
        building_id, user_id, title, description, visible_to_roles as _, closed_at
    ).fetch_one(&mut *tx).await?;

    for (i, label) in choices.iter().enumerate() {
        sqlx::query!(
            "INSERT INTO consultation_choice (consultation_id, label, order_index) VALUES ($1, $2, $3)",
            id, label, i as i32
        ).execute(&mut *tx).await?;
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
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM consultation WHERE id=$1 AND building_id=$2 AND status='open')",
        id, building_id
    ).fetch_one(pool).await?.unwrap_or(false);

    if !exists {
        return Err(crate::errors::AppError::Conflict(
            "Consultation not found or closed".into(),
        ));
    }

    sqlx::query!(
        r#"INSERT INTO consultation_response (consultation_id, choice_id, user_id, comment)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (consultation_id, user_id)
           DO UPDATE SET choice_id = EXCLUDED.choice_id, comment = EXCLUDED.comment"#,
        id,
        choice_id,
        user_id,
        comment
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn results(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<serde_json::Value> {
    let rows = sqlx::query!(
        r#"SELECT cc.id, cc.label, COUNT(cr.id) as votes
           FROM consultation_choice cc
           LEFT JOIN consultation_response cr ON cr.choice_id = cc.id
           JOIN consultation c ON c.id = cc.consultation_id AND c.building_id = $2
           WHERE cc.consultation_id = $1
           GROUP BY cc.id, cc.label, cc.order_index ORDER BY cc.order_index"#,
        id,
        building_id
    )
    .fetch_all(pool)
    .await?;

    let total: i64 = rows.iter().map(|r| r.votes.unwrap_or(0)).sum();
    Ok(
        serde_json::json!({ "total": total, "choices": rows.iter().map(|r| serde_json::json!({
        "id": r.id, "label": r.label, "votes": r.votes.unwrap_or(0),
        "pct": if total > 0 { r.votes.unwrap_or(0) * 100 / total } else { 0 }
    })).collect::<Vec<_>>() }),
    )
}

pub async fn close(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<()> {
    sqlx::query!(
        "UPDATE consultation SET status='closed', closed_at=NOW(), updated_at=NOW() WHERE id=$1 AND building_id=$2 AND status='open'",
        id, building_id
    ).execute(pool).await?;
    Ok(())
}
