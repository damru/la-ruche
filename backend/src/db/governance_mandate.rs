use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct GovernanceMandateRow {
    pub id: Uuid,
    pub building_id: Uuid,
    pub role: String,
    pub holder_name: String,
    pub user_id: Option<Uuid>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub notes: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMandateInput {
    pub role: String,
    pub holder_name: String,
    pub user_id: Option<Uuid>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMandateInput {
    pub role: String,
    pub holder_name: String,
    pub user_id: Option<Uuid>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub notes: Option<String>,
}

pub const ROLE_SYNDIC: &str = "syndic";
const VALID_ROLES: &[&str] = &[ROLE_SYNDIC, "council_president", "council_member"];

const LIST_QUERY: &str = concat!(
    "SELECT id, building_id, role::text AS role, holder_name, user_id,",
    " start_date, end_date, notes, created_by, created_at, updated_at",
    " FROM governance_mandate WHERE building_id = $1",
    " ORDER BY start_date DESC, created_at DESC",
);

const GET_QUERY: &str = concat!(
    "SELECT id, building_id, role::text AS role, holder_name, user_id,",
    " start_date, end_date, notes, created_by, created_at, updated_at",
    " FROM governance_mandate WHERE id = $1 AND building_id = $2",
);

fn validate_payload(
    role: &str,
    holder_name: &str,
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
) -> AppResult<()> {
    if holder_name.trim().is_empty() {
        return Err(AppError::validation_code(
            "errors.governance.holder_name_required",
        ));
    }
    if !VALID_ROLES.contains(&role) {
        return Err(AppError::validation_code("errors.governance.invalid_role"));
    }
    if let Some(end_date) = end_date
        && end_date < start_date
    {
        return Err(AppError::validation_code(
            "errors.governance.end_before_start",
        ));
    }
    Ok(())
}

/// Enforce the "single active syndic" invariant: no two syndic mandates of the
/// same building may overlap in time (open-ended mandates run to infinity).
async fn assert_no_syndic_overlap(
    pool: &PgPool,
    building_id: Uuid,
    exclude_id: Uuid,
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
) -> AppResult<()> {
    let conflicts: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM governance_mandate
           WHERE building_id = $1
             AND role = 'syndic'
             AND id <> $2
             AND start_date <= COALESCE($4, 'infinity'::date)
             AND $3 <= COALESCE(end_date, 'infinity'::date)"#,
    )
    .bind(building_id)
    .bind(exclude_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_one(pool)
    .await?;

    if conflicts > 0 {
        return Err(AppError::validation_code(
            "errors.governance.syndic_overlap",
        ));
    }
    Ok(())
}

pub async fn list(pool: &PgPool, building_id: Uuid) -> AppResult<Vec<GovernanceMandateRow>> {
    sqlx::query_as::<_, GovernanceMandateRow>(LIST_QUERY)
        .bind(building_id)
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

pub async fn get(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
) -> AppResult<Option<GovernanceMandateRow>> {
    sqlx::query_as::<_, GovernanceMandateRow>(GET_QUERY)
        .bind(id)
        .bind(building_id)
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
}

pub async fn create(
    pool: &PgPool,
    building_id: Uuid,
    created_by: Uuid,
    input: CreateMandateInput,
) -> AppResult<GovernanceMandateRow> {
    validate_payload(
        &input.role,
        &input.holder_name,
        input.start_date,
        input.end_date,
    )?;

    if input.role == ROLE_SYNDIC {
        assert_no_syndic_overlap(
            pool,
            building_id,
            Uuid::nil(),
            input.start_date,
            input.end_date,
        )
        .await?;
    }

    let new_id: Uuid = sqlx::query_scalar(
        r#"INSERT INTO governance_mandate
               (building_id, role, holder_name, user_id, start_date, end_date, notes, created_by)
           VALUES ($1, $2::text::governance_role, $3, $4, $5, $6, $7, $8)
           RETURNING id"#,
    )
    .bind(building_id)
    .bind(&input.role)
    .bind(input.holder_name.trim())
    .bind(input.user_id)
    .bind(input.start_date)
    .bind(input.end_date)
    .bind(input.notes.as_deref().map(str::trim))
    .bind(created_by)
    .fetch_one(pool)
    .await?;

    get(pool, new_id, building_id)
        .await?
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("created mandate not found")))
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    input: UpdateMandateInput,
) -> AppResult<Option<GovernanceMandateRow>> {
    validate_payload(
        &input.role,
        &input.holder_name,
        input.start_date,
        input.end_date,
    )?;

    if get(pool, id, building_id).await?.is_none() {
        return Ok(None);
    }

    if input.role == ROLE_SYNDIC {
        assert_no_syndic_overlap(pool, building_id, id, input.start_date, input.end_date).await?;
    }

    sqlx::query(
        r#"UPDATE governance_mandate SET
               role = $3::text::governance_role,
               holder_name = $4,
               user_id = $5,
               start_date = $6,
               end_date = $7,
               notes = $8,
               updated_at = NOW()
           WHERE id = $1 AND building_id = $2"#,
    )
    .bind(id)
    .bind(building_id)
    .bind(&input.role)
    .bind(input.holder_name.trim())
    .bind(input.user_id)
    .bind(input.start_date)
    .bind(input.end_date)
    .bind(input.notes.as_deref().map(str::trim))
    .execute(pool)
    .await?;

    get(pool, id, building_id).await
}

pub async fn delete(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<bool> {
    let result = sqlx::query("DELETE FROM governance_mandate WHERE id = $1 AND building_id = $2")
        .bind(id)
        .bind(building_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn rejects_blank_holder() {
        assert!(validate_payload("syndic", "  ", date(2026, 1, 1), None).is_err());
    }

    #[test]
    fn rejects_invalid_role() {
        assert!(validate_payload("president", "Conseil", date(2026, 1, 1), None).is_err());
    }

    #[test]
    fn rejects_end_before_start() {
        assert!(
            validate_payload(
                "council_member",
                "X",
                date(2026, 2, 1),
                Some(date(2026, 1, 1))
            )
            .is_err()
        );
    }

    #[test]
    fn accepts_open_ended_mandate() {
        assert!(validate_payload("syndic", "Cabinet X", date(2026, 1, 1), None).is_ok());
    }
}
