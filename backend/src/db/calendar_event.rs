// ── db/calendar_event.rs — Événements manuels du calendrier (EP-014-S03) ──

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::errors::AppResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "calendar_event_category", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CalendarEventCategory {
    Reunion,
    Visite,
    Permanence,
    Autre,
}

#[derive(Debug, Serialize)]
pub struct CalendarEventRow {
    pub id: Uuid,
    pub building_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub event_date: NaiveDate,
    pub event_time: Option<NaiveTime>,
    pub category: Option<CalendarEventCategory>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn row_from_pg(r: &sqlx::postgres::PgRow) -> Result<CalendarEventRow, sqlx::Error> {
    Ok(CalendarEventRow {
        id: r.try_get("id")?,
        building_id: r.try_get("building_id")?,
        title: r.try_get("title")?,
        description: r.try_get("description")?,
        event_date: r.try_get("event_date")?,
        event_time: r.try_get("event_time")?,
        category: r.try_get("category")?,
        created_by: r.try_get("created_by")?,
        created_at: r.try_get("created_at")?,
        updated_at: r.try_get("updated_at")?,
    })
}

pub async fn list(pool: &PgPool, building_id: Uuid) -> AppResult<Vec<CalendarEventRow>> {
    let rows = sqlx::query(
        r#"SELECT id, building_id, title, description, event_date, event_time,
           category, created_by, created_at, updated_at
           FROM calendar_event WHERE building_id = $1
           ORDER BY event_date DESC, event_time DESC"#,
    )
    .bind(building_id)
    .fetch_all(pool)
    .await?;
    rows.iter()
        .map(row_from_pg)
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

pub async fn get(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
) -> AppResult<Option<CalendarEventRow>> {
    let row = sqlx::query(
        r#"SELECT id, building_id, title, description, event_date, event_time,
           category, created_by, created_at, updated_at
           FROM calendar_event WHERE id = $1 AND building_id = $2"#,
    )
    .bind(id)
    .bind(building_id)
    .fetch_optional(pool)
    .await?;
    row.as_ref()
        .map(row_from_pg)
        .transpose()
        .map_err(Into::into)
}

#[allow(clippy::too_many_arguments)]
pub async fn create(
    pool: &PgPool,
    building_id: Uuid,
    user_id: Uuid,
    title: &str,
    description: Option<&str>,
    event_date: NaiveDate,
    event_time: Option<NaiveTime>,
    category: Option<CalendarEventCategory>,
) -> AppResult<CalendarEventRow> {
    let row = sqlx::query(
        r#"INSERT INTO calendar_event
           (building_id, created_by, title, description, event_date, event_time, category)
           VALUES ($1,$2,$3,$4,$5,$6,$7)
           RETURNING id, building_id, title, description, event_date, event_time,
           category, created_by, created_at, updated_at"#,
    )
    .bind(building_id)
    .bind(user_id)
    .bind(title)
    .bind(description)
    .bind(event_date)
    .bind(event_time)
    .bind(category)
    .fetch_one(pool)
    .await?;
    row_from_pg(&row).map_err(Into::into)
}

#[allow(clippy::too_many_arguments)]
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    title: &str,
    description: Option<&str>,
    event_date: NaiveDate,
    event_time: Option<NaiveTime>,
    category: Option<CalendarEventCategory>,
) -> AppResult<Option<CalendarEventRow>> {
    let row = sqlx::query(
        r#"UPDATE calendar_event
           SET title=$3, description=$4, event_date=$5, event_time=$6,
               category=$7, updated_at=NOW()
           WHERE id=$1 AND building_id=$2
           RETURNING id, building_id, title, description, event_date, event_time,
           category, created_by, created_at, updated_at"#,
    )
    .bind(id)
    .bind(building_id)
    .bind(title)
    .bind(description)
    .bind(event_date)
    .bind(event_time)
    .bind(category)
    .fetch_optional(pool)
    .await?;
    row.as_ref()
        .map(row_from_pg)
        .transpose()
        .map_err(Into::into)
}

pub async fn delete(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<Option<()>> {
    let result = sqlx::query("DELETE FROM calendar_event WHERE id=$1 AND building_id=$2")
        .bind(id)
        .bind(building_id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Ok(None);
    }
    Ok(Some(()))
}

pub async fn list_documents(
    pool: &PgPool,
    event_id: Uuid,
    building_id: Uuid,
) -> AppResult<Vec<serde_json::Value>> {
    let rows = sqlx::query(
        r#"SELECT d.id, d.original_name, d.mime_type, d.size_bytes, d.created_at
           FROM calendar_event_document ced
           JOIN document d ON d.id = ced.document_id AND d.building_id = $2
           WHERE ced.event_id = $1 AND d.deleted_at IS NULL
           ORDER BY d.created_at ASC"#,
    )
    .bind(event_id)
    .bind(building_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.try_get::<Uuid, _>("id").unwrap_or_default(),
                "original_name": row.try_get::<String, _>("original_name").unwrap_or_default(),
                "mime_type": row.try_get::<String, _>("mime_type").unwrap_or_default(),
                "size_bytes": row.try_get::<i64, _>("size_bytes").unwrap_or_default(),
                "created_at": row.try_get::<DateTime<Utc>, _>("created_at").unwrap_or_default(),
            })
        })
        .collect())
}

pub async fn attach_document(
    pool: &PgPool,
    event_id: Uuid,
    document_id: Uuid,
    building_id: Uuid,
) -> AppResult<bool> {
    let pair_exists: bool = sqlx::query_scalar(
        r#"SELECT EXISTS(
               SELECT 1 FROM calendar_event e, document d
               WHERE e.id = $1 AND e.building_id = $3
                 AND d.id = $2 AND d.building_id = $3
           )"#,
    )
    .bind(event_id)
    .bind(document_id)
    .bind(building_id)
    .fetch_one(pool)
    .await?;

    if !pair_exists {
        return Ok(false);
    }

    sqlx::query(
        r#"INSERT INTO calendar_event_document (event_id, document_id)
           VALUES ($1, $2)
           ON CONFLICT (event_id, document_id) DO NOTHING"#,
    )
    .bind(event_id)
    .bind(document_id)
    .execute(pool)
    .await?;

    Ok(true)
}

pub async fn detach_document(
    pool: &PgPool,
    event_id: Uuid,
    document_id: Uuid,
    building_id: Uuid,
) -> AppResult<bool> {
    let result = sqlx::query(
        r#"DELETE FROM calendar_event_document ced
           USING calendar_event e
           WHERE ced.event_id = $1
             AND ced.document_id = $2
             AND e.id = ced.event_id
             AND e.building_id = $3"#,
    )
    .bind(event_id)
    .bind(document_id)
    .bind(building_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
