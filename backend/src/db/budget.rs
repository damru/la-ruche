use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::PgPool;
use sqlx::types::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

use crate::db::audit::{self, NewAuditEntry};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BudgetRow {
    pub id: Uuid,
    pub building_id: Uuid,
    pub fiscal_year: i32,
    pub status: String,
    pub approval_meeting_id: Option<Uuid>,
    pub parent_budget_id: Option<Uuid>,
    pub total_forecast: Decimal,
    pub total_actual: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BudgetLineRow {
    pub id: Uuid,
    pub budget_id: Uuid,
    pub label: String,
    pub key_id: Uuid,
    pub budgeted_amount: Decimal,
    pub actual_amount: Option<Decimal>,
    pub account_code: Option<String>,
    pub valid_from: NaiveDate,
    pub valid_to: Option<NaiveDate>,
    pub rectificatif_version: Option<i32>,
    pub version: i32,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LevyCallRow {
    pub id: Uuid,
    pub budget_id: Uuid,
    pub call_label: String,
    pub due_date: NaiveDate,
    pub charge_amount: Decimal,
    pub works_fund_amount: Decimal,
    pub issued_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct WorksFundRow {
    pub id: Uuid,
    pub fiscal_year: i32,
    pub levied_amount: Decimal,
    pub available_amount: Decimal,
    pub contribution_rate_pct: Decimal,
}

#[derive(Debug, sqlx::FromRow)]
struct LevyChargeTotalByLineRow {
    budget_line_id: Uuid,
    charge_total: Decimal,
}

#[derive(Debug, sqlx::FromRow)]
struct LevyLineAllocationDetailRow {
    levy_call_id: Uuid,
    budget_line_id: Uuid,
    budget_line_label: String,
    amount: Decimal,
}

pub struct LevyInsertLine {
    pub budget_line_id: Uuid,
    pub amount: Decimal,
}

/// One entry per unit and allocation key to insert as levy_call.
pub struct LevyInsert {
    pub unit_id: Uuid,
    pub key_id: Uuid,
    pub amount: Decimal,
    pub charge_amount: Decimal,
    pub works_fund_amount: Decimal,
    pub shares_snapshot: i64,
    pub shares_total_snapshot: i64,
    pub line_allocations: Vec<LevyInsertLine>,
}

pub struct LevyAmountTotals {
    pub total_amount: Decimal,
    pub charge_amount: Decimal,
    pub works_fund_amount: Decimal,
}

pub async fn charge_amount_totals_by_line(
    pool: &PgPool,
    budget_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<HashMap<Uuid, Decimal>> {
    let rows = sqlx::query_as::<_, LevyChargeTotalByLineRow>(
        r#"SELECT
               lcl.budget_line_id,
               COALESCE(SUM(lcl.amount), 0)::numeric AS charge_total
           FROM levy_call_line lcl
           JOIN levy_call lc ON lc.id = lcl.levy_call_id
           JOIN budget b ON b.id = lc.budget_id
           WHERE lc.budget_id = $1
             AND b.building_id = $2
           GROUP BY lcl.budget_line_id"#,
    )
    .bind(budget_id)
    .bind(building_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| (row.budget_line_id, row.charge_total))
        .collect())
}

pub async fn levy_line_allocations_by_levy(
    pool: &PgPool,
    budget_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<HashMap<Uuid, Vec<serde_json::Value>>> {
    let rows = sqlx::query_as::<_, LevyLineAllocationDetailRow>(
        r#"SELECT
               lcl.levy_call_id,
               lcl.budget_line_id,
               bl.label AS budget_line_label,
               lcl.amount
           FROM levy_call_line lcl
           JOIN levy_call lc ON lc.id = lcl.levy_call_id
           JOIN budget b ON b.id = lc.budget_id AND b.building_id = $2
           JOIN budget_line bl ON bl.id = lcl.budget_line_id
           WHERE lc.budget_id = $1
           ORDER BY lc.due_date DESC, bl.label"#,
    )
    .bind(budget_id)
    .bind(building_id)
    .fetch_all(pool)
    .await?;

    let mut grouped: HashMap<Uuid, Vec<serde_json::Value>> = HashMap::new();
    for row in rows {
        grouped
            .entry(row.levy_call_id)
            .or_default()
            .push(serde_json::json!({
                "budget_line_id": row.budget_line_id,
                "budget_line_label": row.budget_line_label,
                "amount": row.amount,
            }));
    }

    Ok(grouped)
}

pub async fn list(pool: &PgPool, building_id: Uuid) -> crate::errors::AppResult<Vec<BudgetRow>> {
    Ok(sqlx::query_as!(
        BudgetRow,
        r#"SELECT
            b.id,
            b.building_id,
            b.fiscal_year,
            b.status::text as "status!",
            b.approval_meeting_id,
            b.parent_budget_id,
            COALESCE((SELECT SUM(bl.budgeted_amount) FROM budget_line bl WHERE bl.budget_id = b.id AND bl.valid_to IS NULL), 0)::numeric as "total_forecast!",
            COALESCE((SELECT SUM(bl.actual_amount) FROM budget_line bl WHERE bl.budget_id = b.id AND bl.valid_to IS NULL), 0)::numeric as "total_actual!",
            b.created_at,
            b.updated_at
           FROM budget b
           WHERE b.building_id = $1
           ORDER BY b.fiscal_year DESC"#,
        building_id
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<Option<BudgetRow>> {
    Ok(sqlx::query_as!(
        BudgetRow,
        r#"SELECT
            b.id,
            b.building_id,
            b.fiscal_year,
            b.status::text as "status!",
            b.approval_meeting_id,
            b.parent_budget_id,
            COALESCE((SELECT SUM(bl.budgeted_amount) FROM budget_line bl WHERE bl.budget_id = b.id AND bl.valid_to IS NULL), 0)::numeric as "total_forecast!",
            COALESCE((SELECT SUM(bl.actual_amount) FROM budget_line bl WHERE bl.budget_id = b.id AND bl.valid_to IS NULL), 0)::numeric as "total_actual!",
            b.created_at,
            b.updated_at
           FROM budget b
           WHERE b.id = $1 AND b.building_id = $2"#,
        id,
        building_id
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn create(
    pool: &PgPool,
    building_id: Uuid,
    fiscal_year: i32,
) -> crate::errors::AppResult<BudgetRow> {
    let mut tx = pool.begin().await?;

    let budget = sqlx::query_as!(
        BudgetRow,
        r#"INSERT INTO budget (building_id, fiscal_year)
           VALUES ($1, $2)
           RETURNING
           id,
           building_id,
           fiscal_year,
           status::text as "status!",
           approval_meeting_id,
           parent_budget_id,
           0::numeric as "total_forecast!",
           0::numeric as "total_actual!",
           created_at,
           updated_at"#,
        building_id,
        fiscal_year
    )
    .fetch_one(&mut *tx)
    .await?;

    ensure_works_fund(&mut *tx, building_id, fiscal_year).await?;

    tx.commit().await?;
    Ok(budget)
}

pub async fn get_by_fiscal_year(
    pool: &PgPool,
    building_id: Uuid,
    fiscal_year: i32,
) -> crate::errors::AppResult<Option<BudgetRow>> {
    Ok(sqlx::query_as::<_, BudgetRow>(
        r#"SELECT
            b.id,
            b.building_id,
            b.fiscal_year,
            b.status::text as status,
            b.approval_meeting_id,
            b.parent_budget_id,
            COALESCE((SELECT SUM(bl.budgeted_amount) FROM budget_line bl WHERE bl.budget_id = b.id AND bl.valid_to IS NULL), 0)::numeric as total_forecast,
            COALESCE((SELECT SUM(bl.actual_amount) FROM budget_line bl WHERE bl.budget_id = b.id AND bl.valid_to IS NULL), 0)::numeric as total_actual,
            b.created_at,
            b.updated_at
           FROM budget b
           WHERE b.building_id = $1 AND b.fiscal_year = $2"#,
    )
    .bind(building_id)
    .bind(fiscal_year)
    .fetch_optional(pool)
    .await?)
}

pub async fn ensure_works_fund<'e, E>(
    executor: E,
    building_id: Uuid,
    fiscal_year: i32,
) -> crate::errors::AppResult<()>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    sqlx::query(
        r#"INSERT INTO works_fund (building_id, fiscal_year)
           VALUES ($1, $2)
           ON CONFLICT (building_id, fiscal_year) DO NOTHING"#,
    )
    .bind(building_id)
    .bind(fiscal_year)
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn get_works_fund(
    pool: &PgPool,
    building_id: Uuid,
    fiscal_year: i32,
) -> crate::errors::AppResult<Option<WorksFundRow>> {
    Ok(sqlx::query_as::<_, WorksFundRow>(
        r#"SELECT id, fiscal_year, levied_amount, available_amount, contribution_rate_pct
           FROM works_fund
           WHERE building_id = $1 AND fiscal_year = $2"#,
    )
    .bind(building_id)
    .bind(fiscal_year)
    .fetch_optional(pool)
    .await?)
}

pub async fn get_or_create_works_fund(
    pool: &PgPool,
    building_id: Uuid,
    fiscal_year: i32,
) -> crate::errors::AppResult<WorksFundRow> {
    ensure_works_fund(pool, building_id, fiscal_year).await?;
    get_works_fund(pool, building_id, fiscal_year)
        .await?
        .ok_or(crate::errors::AppError::NotFound)
}

pub async fn update_works_fund_rate(
    pool: &PgPool,
    building_id: Uuid,
    fiscal_year: i32,
    contribution_rate_pct: Decimal,
) -> crate::errors::AppResult<Option<WorksFundRow>> {
    Ok(sqlx::query_as::<_, WorksFundRow>(
        r#"UPDATE works_fund
           SET contribution_rate_pct = $3,
               updated_at = NOW()
           WHERE building_id = $1 AND fiscal_year = $2
           RETURNING id, fiscal_year, levied_amount, available_amount, contribution_rate_pct"#,
    )
    .bind(building_id)
    .bind(fiscal_year)
    .bind(contribution_rate_pct)
    .fetch_optional(pool)
    .await?)
}

pub async fn ensure_budget_is_mutable(
    pool: &PgPool,
    budget_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<()> {
    let status = sqlx::query_scalar!(
        r#"SELECT status::text as "status!" FROM budget WHERE id = $1 AND building_id = $2"#,
        budget_id,
        building_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(crate::errors::AppError::NotFound)?;

    if status == "closed" {
        return Err(crate::errors::AppError::conflict_code(
            "errors.budget.exercise_closed",
        ));
    }

    Ok(())
}

/// Pattern B guard (ADR-009): budget lines may only be created, edited or
/// deleted while the budget is still a `draft`. Once approved in AG the budget
/// is a legal act — amendments go through `POST /budgets/{id}/rectificatif`.
pub async fn ensure_budget_lines_mutable(
    pool: &PgPool,
    budget_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<()> {
    let status = sqlx::query_scalar!(
        r#"SELECT status::text as "status!" FROM budget WHERE id = $1 AND building_id = $2"#,
        budget_id,
        building_id
    )
    .fetch_optional(pool)
    .await?
    .ok_or(crate::errors::AppError::NotFound)?;

    crate::domain::immutability::assert_mutable(
        &crate::domain::immutability::Entity::Budget,
        &status,
    )
}

/// Maps the budget/budget_line unique and check constraints to i18n error codes.
///
/// The constraints are the only race-free guard on these invariants: an
/// application-side `SELECT COUNT(*)` before the write is a check-then-act race
/// that two concurrent requests both pass.
fn map_budget_constraint(err: sqlx::Error) -> crate::errors::AppError {
    use crate::errors::AppError;

    if let sqlx::Error::Database(ref db_err) = err {
        match db_err.constraint() {
            Some("budget_line_unique_active") => {
                return AppError::conflict_code("errors.budget_line.duplicate_active");
            }
            Some("budget_unique_root_per_year") => {
                return AppError::conflict_code("errors.budget.fiscal_year_already_exists");
            }
            Some("budget_unique_pending_rectificatif") => {
                return AppError::conflict_code("errors.budget.rectificatif_already_pending");
            }
            Some("budget_unique_approved_per_year") => {
                return AppError::conflict_code("errors.budget.approved_budget_already_exists");
            }
            Some("budget_line_valid_period") => {
                return AppError::validation_code("errors.budget_line.invalid_validity_period");
            }
            Some("budget_line_rectificatif_version_positive") => {
                return AppError::validation_code(
                    "errors.budget_line.invalid_rectificatif_version",
                );
            }
            _ => {}
        }
    }

    err.into()
}

pub async fn close(
    pool: &PgPool,
    budget_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<bool> {
    let result = sqlx::query!(
        r#"UPDATE budget
           SET status = 'closed', updated_at = NOW()
           WHERE id = $1 AND building_id = $2 AND status != 'closed'"#,
        budget_id,
        building_id
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Creates a rectificatif (amendment) budget for an existing approved budget.
///
/// Pattern B (ADR-009): a new `draft` budget is created with `parent_budget_id`
/// referencing the amended budget, and the amended budget's lines are *copied*
/// into it. The amended budget is left completely untouched: it stays
/// `approved`, keeps its lines open (`valid_to IS NULL`) and remains fully
/// consultable, because until the rectificatif is voted in AG it is still the
/// budget in force (AC-BC-009-B, LEGAL.md §11). The parent is closed only in
/// `approve`.
///
/// At most one rectificatif may be in preparation per amended budget — enforced
/// by `budget_unique_pending_rectificatif`, not by a pre-check.
pub async fn create_rectificatif(
    pool: &PgPool,
    original_budget_id: Uuid,
    building_id: Uuid,
    user_id: Uuid,
) -> crate::errors::AppResult<BudgetRow> {
    let mut tx = pool.begin().await?;

    // Lock the amended budget for the duration of the transaction so its status
    // cannot change under us while we derive the rectificatif number.
    let original = sqlx::query_as::<_, BudgetRow>(
        r#"SELECT id, building_id, fiscal_year, status::text as status, approval_meeting_id, parent_budget_id,
              0::numeric as total_forecast, 0::numeric as total_actual, created_at, updated_at
           FROM budget
           WHERE id = $1 AND building_id = $2
           FOR UPDATE"#,
    )
    .bind(original_budget_id)
    .bind(building_id)
    .fetch_optional(&mut *tx)
    .await?;

    let original = match original {
        Some(b) => b,
        None => return Err(crate::errors::AppError::NotFound),
    };

    // Only a budget voted in AG can be amended: a draft is simply edited.
    if original.status != "approved" {
        return Err(crate::errors::AppError::validation_code(
            "errors.budget.rectificatif_requires_approved",
        ));
    }

    // Rectificatif n is created from rectificatif n-1, so the amended budget's
    // own number + 1 gives the next one. Root budget lines carry NULL → 1.
    let next_rectificatif_version: i32 = sqlx::query_scalar!(
        r#"SELECT COALESCE(MAX(rectificatif_version), 0) + 1 as "next!"
           FROM budget_line
           WHERE budget_id = $1"#,
        original_budget_id
    )
    .fetch_one(&mut *tx)
    .await?;

    let rectificatif = sqlx::query_as!(
        BudgetRow,
        r#"INSERT INTO budget (building_id, fiscal_year, status, parent_budget_id)
           VALUES ($1, $2, 'draft', $3)
           RETURNING
           id,
           building_id,
           fiscal_year,
           status::text as "status!",
           approval_meeting_id,
           parent_budget_id,
           0::numeric as "total_forecast!",
           0::numeric as "total_actual!",
           created_at,
           updated_at"#,
        building_id,
        original.fiscal_year,
        original_budget_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(map_budget_constraint)?;

    // Copy the amended budget's active lines as the rectificatif's starting
    // point. `actual_amount` is deliberately NOT copied: realised amounts belong
    // to the exercise, not to the newly voted forecast, and copying them would
    // double-count them across the chain.
    sqlx::query!(
        r#"INSERT INTO budget_line
               (budget_id, label, key_id, budgeted_amount,
                account_code, valid_from, valid_to, rectificatif_version)
           SELECT $1, label, key_id, budgeted_amount,
                  account_code, CURRENT_DATE, NULL, $2
           FROM budget_line
           WHERE budget_id = $3 AND valid_to IS NULL"#,
        rectificatif.id,
        next_rectificatif_version,
        original_budget_id
    )
    .execute(&mut *tx)
    .await
    .map_err(map_budget_constraint)?;

    audit::insert(
        &mut *tx,
        NewAuditEntry {
            building_id,
            entity_type: "budget",
            entity_id: rectificatif.id,
            operation: "CREATE",
            changed_by: Some(user_id),
            old_values: None,
            new_values: Some(serde_json::json!({
                "status": "draft",
                "parent_budget_id": original_budget_id,
                "fiscal_year": original.fiscal_year,
                "rectificatif_version": next_rectificatif_version,
            })),
        },
    )
    .await?;

    tx.commit().await?;
    Ok(rectificatif)
}

/// Approves a draft budget, transitioning it to `approved`.
///
/// `approval_meeting_id` is the legal basis for the approval (art. 24 loi 1965):
/// it must reference a `closed` general meeting of the same building. For a
/// rectificatif this is the moment the amendment takes effect — the amended
/// budget is closed and its lines are dated out, in that order, so that the two
/// never both hold `approved` (`budget_unique_approved_per_year`).
pub async fn approve(
    pool: &PgPool,
    budget_id: Uuid,
    building_id: Uuid,
    approval_meeting_id: Uuid,
    user_id: Uuid,
) -> crate::errors::AppResult<BudgetRow> {
    let mut tx = pool.begin().await?;

    // The AG must exist, belong to this building, and be closed — a budget
    // cannot be approved by a meeting that has not been held.
    let meeting_status = sqlx::query_scalar!(
        r#"SELECT status::text as "status!"
           FROM general_meeting
           WHERE id = $1 AND building_id = $2"#,
        approval_meeting_id,
        building_id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| {
        crate::errors::AppError::validation_code("errors.budget.approval_meeting_not_found")
    })?;

    if meeting_status != "closed" {
        return Err(crate::errors::AppError::validation_code(
            "errors.budget.approval_meeting_not_closed",
        ));
    }

    let target = sqlx::query_as::<_, BudgetRow>(
        r#"SELECT id, building_id, fiscal_year, status::text as status, approval_meeting_id, parent_budget_id,
              0::numeric as total_forecast, 0::numeric as total_actual, created_at, updated_at
           FROM budget
           WHERE id = $1 AND building_id = $2
           FOR UPDATE"#,
    )
    .bind(budget_id)
    .bind(building_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(crate::errors::AppError::NotFound)?;

    if target.status != "draft" {
        return Err(crate::errors::AppError::conflict_code(
            "errors.budget.not_draft",
        ));
    }

    // Close the amended budget FIRST: `budget_unique_approved_per_year` is a
    // non-deferrable unique index, so the two budgets must never be `approved`
    // at the same point in the transaction.
    if let Some(parent_id) = target.parent_budget_id {
        sqlx::query!(
            r#"UPDATE budget
               SET status = 'closed', updated_at = NOW()
               WHERE id = $1 AND building_id = $2"#,
            parent_id,
            building_id
        )
        .execute(&mut *tx)
        .await?;

        // Date out the amended budget's lines. `valid_to = CURRENT_DATE` is
        // exclusive and matches the convention used by allocation_key/unit
        // versioning (ADR-006); it also cannot invert the validity period when
        // the rectificatif is approved the same day the lines were created.
        sqlx::query!(
            r#"UPDATE budget_line
               SET valid_to = CURRENT_DATE
               WHERE budget_id = $1 AND valid_to IS NULL"#,
            parent_id
        )
        .execute(&mut *tx)
        .await
        .map_err(map_budget_constraint)?;
    }

    let budget = sqlx::query_as!(
        BudgetRow,
        r#"UPDATE budget
           SET status = 'approved',
               approval_meeting_id = $1,
               updated_at = NOW()
           WHERE id = $2 AND building_id = $3 AND status = 'draft'
           RETURNING
           id, building_id, fiscal_year,
           status::text as "status!",
           approval_meeting_id,
           parent_budget_id,
           0::numeric as "total_forecast!",
           0::numeric as "total_actual!",
           created_at,
           updated_at"#,
        approval_meeting_id,
        budget_id,
        building_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(map_budget_constraint)?;

    audit::insert(
        &mut *tx,
        NewAuditEntry {
            building_id,
            entity_type: "budget",
            entity_id: budget_id,
            operation: "UPDATE",
            changed_by: Some(user_id),
            old_values: Some(serde_json::json!({ "status": "draft" })),
            new_values: Some(serde_json::json!({
                "status": "approved",
                "approval_meeting_id": approval_meeting_id,
            })),
        },
    )
    .await?;

    tx.commit().await?;
    Ok(budget)
}

/// Returns the full rectificatif chain of a budget, root first, so the UI can
/// expose the amendment history (Pattern B, ADR-009).
pub async fn rectificatif_chain(
    pool: &PgPool,
    budget_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<Vec<BudgetRow>> {
    Ok(sqlx::query_as::<_, BudgetRow>(
        r#"WITH RECURSIVE up AS (
               SELECT id, parent_budget_id
               FROM budget
               WHERE id = $1 AND building_id = $2
               UNION ALL
               SELECT b.id, b.parent_budget_id
               FROM budget b
               JOIN up ON up.parent_budget_id = b.id
           ),
           root AS (
               SELECT id FROM up WHERE parent_budget_id IS NULL
           ),
           down AS (
               SELECT id, parent_budget_id, 0 AS depth
               FROM budget
               WHERE id = (SELECT id FROM root)
               UNION ALL
               SELECT b.id, b.parent_budget_id, down.depth + 1
               FROM budget b
               JOIN down ON b.parent_budget_id = down.id
           )
           SELECT
               b.id,
               b.building_id,
               b.fiscal_year,
               b.status::text as status,
               b.approval_meeting_id,
               b.parent_budget_id,
               COALESCE((SELECT SUM(bl.budgeted_amount) FROM budget_line bl WHERE bl.budget_id = b.id AND bl.valid_to IS NULL), 0)::numeric as total_forecast,
               COALESCE((SELECT SUM(bl.actual_amount) FROM budget_line bl WHERE bl.budget_id = b.id AND bl.valid_to IS NULL), 0)::numeric as total_actual,
               b.created_at,
               b.updated_at
           FROM down
           JOIN budget b ON b.id = down.id
           ORDER BY down.depth"#,
    )
    .bind(budget_id)
    .bind(building_id)
    .fetch_all(pool)
    .await?)
}

/// Lists only the currently active version of each budget_line
/// (valid_to IS NULL). Historical (closed) versions are excluded — see
/// ADR-009 §Versioning temporel.
pub async fn list_lines(
    pool: &PgPool,
    budget_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<Vec<BudgetLineRow>> {
    Ok(sqlx::query_as::<_, BudgetLineRow>(
        r#"SELECT bl.id, bl.budget_id, bl.label, bl.key_id,
           bl.budgeted_amount, bl.actual_amount, bl.account_code,
           bl.valid_from, bl.valid_to, bl.rectificatif_version, bl.version
           FROM budget_line bl
           JOIN budget b ON b.id = bl.budget_id AND b.building_id = $2
           WHERE bl.budget_id = $1 AND bl.valid_to IS NULL
           ORDER BY bl.label"#,
    )
    .bind(budget_id)
    .bind(building_id)
    .fetch_all(pool)
    .await?)
}

/// `rectificatif_version` is chain metadata: it is set by `create_rectificatif`
/// when it copies the lines, never by the client. A line created directly on a
/// rectificatif inherits the rectificatif's number.
pub async fn create_line(
    pool: &PgPool,
    budget_id: Uuid,
    label: &str,
    key_id: Uuid,
    budgeted_amount: Decimal,
    account_code: Option<&str>,
) -> crate::errors::AppResult<BudgetLineRow> {
    sqlx::query_as::<_, BudgetLineRow>(
        r#"INSERT INTO budget_line
               (budget_id, label, key_id, budgeted_amount, account_code, rectificatif_version)
           VALUES ($1, $2, $3, $4, $5,
                   (SELECT MAX(rectificatif_version) FROM budget_line WHERE budget_id = $1))
           RETURNING id, budget_id, label, key_id, budgeted_amount, actual_amount, account_code,
           valid_from, valid_to, rectificatif_version, version"#,
    )
    .bind(budget_id)
    .bind(label)
    .bind(key_id)
    .bind(budgeted_amount)
    .bind(account_code)
    .fetch_one(pool)
    .await
    .map_err(map_budget_constraint)
}

/// Updates the active version of a budget line, guarded by its optimistic-locking
/// `version`.
///
/// `valid_to IS NULL` in the WHERE clause is what keeps accounting history
/// immutable: a closed historical version can never be rewritten, only read.
/// The whole write is journalled with a field-scoped diff (ADR-009).
pub async fn update_line(
    pool: &PgPool,
    line_id: Uuid,
    budget_id: Uuid,
    building_id: Uuid,
    label: &str,
    key_id: Uuid,
    budgeted_amount: Decimal,
    account_code: Option<&str>,
    version: i32,
    user_id: Uuid,
) -> crate::errors::AppResult<BudgetLineRow> {
    let mut tx = pool.begin().await?;

    let before = sqlx::query_as::<_, BudgetLineRow>(
        r#"SELECT id, budget_id, label, key_id, budgeted_amount, actual_amount, account_code,
           valid_from, valid_to, rectificatif_version, version
           FROM budget_line
           WHERE id = $1 AND budget_id = $2 AND valid_to IS NULL"#,
    )
    .bind(line_id)
    .bind(budget_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(crate::errors::AppError::NotFound)?;

    if before.version != version {
        return Err(crate::errors::AppError::conflict_code(
            "errors.budget_line.version_mismatch",
        ));
    }

    let after = sqlx::query_as::<_, BudgetLineRow>(
        r#"UPDATE budget_line
           SET label = $1, key_id = $2, budgeted_amount = $3, account_code = $4
           WHERE id = $5 AND budget_id = $6 AND version = $7 AND valid_to IS NULL
           RETURNING id, budget_id, label, key_id, budgeted_amount, actual_amount, account_code,
           valid_from, valid_to, rectificatif_version, version"#,
    )
    .bind(label)
    .bind(key_id)
    .bind(budgeted_amount)
    .bind(account_code)
    .bind(line_id)
    .bind(budget_id)
    .bind(version)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_budget_constraint)?
    .ok_or_else(|| crate::errors::AppError::conflict_code("errors.budget_line.version_mismatch"))?;

    audit::insert(
        &mut *tx,
        NewAuditEntry {
            building_id,
            entity_type: "budget_line",
            entity_id: line_id,
            operation: "UPDATE",
            changed_by: Some(user_id),
            old_values: Some(serde_json::json!({
                "label": before.label,
                "key_id": before.key_id,
                "budgeted_amount": before.budgeted_amount.to_string(),
                "account_code": before.account_code,
            })),
            new_values: Some(serde_json::json!({
                "label": after.label,
                "key_id": after.key_id,
                "budgeted_amount": after.budgeted_amount.to_string(),
                "account_code": after.account_code,
            })),
        },
    )
    .await?;

    tx.commit().await?;
    Ok(after)
}

/// Deletes a budget line. Only the active version of a line on a draft budget
/// may be deleted — a closed historical version is accounting history and is
/// never erased (ADR-009). The deletion is journalled.
pub async fn delete_line(
    pool: &PgPool,
    line_id: Uuid,
    budget_id: Uuid,
    building_id: Uuid,
    user_id: Uuid,
) -> crate::errors::AppResult<()> {
    let mut tx = pool.begin().await?;

    let before = sqlx::query_as::<_, BudgetLineRow>(
        r#"SELECT id, budget_id, label, key_id, budgeted_amount, actual_amount, account_code,
           valid_from, valid_to, rectificatif_version, version
           FROM budget_line
           WHERE id = $1 AND budget_id = $2 AND valid_to IS NULL"#,
    )
    .bind(line_id)
    .bind(budget_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(crate::errors::AppError::NotFound)?;

    sqlx::query!(
        "DELETE FROM budget_line WHERE id = $1 AND budget_id = $2",
        line_id,
        budget_id
    )
    .execute(&mut *tx)
    .await?;

    audit::insert(
        &mut *tx,
        NewAuditEntry {
            building_id,
            entity_type: "budget_line",
            entity_id: line_id,
            operation: "DELETE",
            changed_by: Some(user_id),
            old_values: Some(serde_json::json!({
                "label": before.label,
                "key_id": before.key_id,
                "budgeted_amount": before.budgeted_amount.to_string(),
                "account_code": before.account_code,
            })),
            new_values: None,
        },
    )
    .await?;

    tx.commit().await?;
    Ok(())
}

/// Generates one levy_call header, per-unit/per-budget-line entries in levy_call_unit,
/// and aggregate totals in levy_call_line. Returns the count of distinct units levied.
pub async fn generate_levies(
    pool: &PgPool,
    budget_id: Uuid,
    building_id: Uuid,
    due_date: NaiveDate,
    call_label: &str,
    levies: &[LevyInsert],
) -> crate::errors::AppResult<usize> {
    let mut tx = pool.begin().await?;

    // Build header-level totals from all unit levies.
    let total_charge: Decimal = levies.iter().map(|l| l.charge_amount).sum();
    let total_works_fund: Decimal = levies.iter().map(|l| l.works_fund_amount).sum();

    // Insert one levy_call header for the whole generation batch.
    let levy_call_id: Uuid = sqlx::query_scalar(
        r#"INSERT INTO levy_call (budget_id, call_label, due_date, charge_amount, works_fund_amount)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id"#,
    )
    .bind(budget_id)
    .bind(call_label)
    .bind(due_date)
    .bind(total_charge)
    .bind(total_works_fund)
    .fetch_one(&mut *tx)
    .await?;

    // Collect every levy_call_unit row (per budget-line allocations + the
    // works_fund portion) and the per-line totals in one pass, then bulk-insert.
    let mut unit_rows: Vec<(Uuid, Option<Uuid>, Uuid, Decimal, i32, i32)> = Vec::new();
    let mut line_totals: HashMap<Uuid, Decimal> = HashMap::new();

    for l in levies {
        for line_alloc in &l.line_allocations {
            unit_rows.push((
                l.unit_id,
                Some(line_alloc.budget_line_id),
                l.key_id,
                line_alloc.amount,
                l.shares_snapshot as i32,
                l.shares_total_snapshot as i32,
            ));
            *line_totals.entry(line_alloc.budget_line_id).or_default() += line_alloc.amount;
        }

        // Works_fund portion carries a NULL budget_line_id.
        if l.works_fund_amount > Decimal::ZERO {
            unit_rows.push((
                l.unit_id,
                None,
                l.key_id,
                l.works_fund_amount,
                l.shares_snapshot as i32,
                l.shares_total_snapshot as i32,
            ));
        }
    }

    // Bulk-insert levy_call_unit rows (7 binds/row → chunk well under the 65k cap).
    for chunk in unit_rows.chunks(1000) {
        let mut qb = sqlx::QueryBuilder::new(
            "INSERT INTO levy_call_unit \
             (levy_call_id, unit_id, budget_line_id, key_id, amount, \
              shares_snapshot, shares_total_snapshot) ",
        );
        qb.push_values(
            chunk,
            |mut b, &(unit_id, line_id, key_id, amount, shares, total)| {
                b.push_bind(levy_call_id)
                    .push_bind(unit_id)
                    .push_bind(line_id)
                    .push_bind(key_id)
                    .push_bind(amount)
                    .push_bind(shares)
                    .push_bind(total);
            },
        );
        qb.build().execute(&mut *tx).await?;
    }

    // Bulk-insert aggregate totals per budget_line into levy_call_line.
    if !line_totals.is_empty() {
        let line_rows: Vec<(Uuid, Decimal)> = line_totals.into_iter().collect();
        for chunk in line_rows.chunks(1000) {
            let mut qb = sqlx::QueryBuilder::new(
                "INSERT INTO levy_call_line (levy_call_id, budget_line_id, amount) ",
            );
            qb.push_values(chunk, |mut b, &(budget_line_id, amount)| {
                b.push_bind(levy_call_id)
                    .push_bind(budget_line_id)
                    .push_bind(amount);
            });
            qb.build().execute(&mut *tx).await?;
        }
    }

    if total_works_fund > Decimal::ZERO {
        sqlx::query(
            r#"UPDATE works_fund wf
               SET levied_amount = wf.levied_amount + $1,
                   available_amount = wf.available_amount + $1,
                   updated_at = NOW()
               FROM budget b
               WHERE b.id = $2
                 AND b.building_id = $3
                 AND wf.building_id = b.building_id
                 AND wf.fiscal_year = b.fiscal_year"#,
        )
        .bind(total_works_fund)
        .bind(budget_id)
        .bind(building_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    // Number of distinct units that received a levy in this batch.
    let nb_units = levies
        .iter()
        .map(|l| l.unit_id)
        .collect::<std::collections::HashSet<_>>()
        .len();
    Ok(nb_units)
}

#[derive(Debug, sqlx::FromRow)]
struct LevyListRow {
    id: Uuid,
    call_label: String,
    due_date: NaiveDate,
    issued_at: Option<DateTime<Utc>>,
    unit_id: Uuid,
    unit_number: String,
    amount: Decimal,
    charge_amount: Decimal,
    works_fund_amount: Decimal,
    call_status: String,
    status: String,
    paid_at: Option<DateTime<Utc>>,
    owner_id: Option<Uuid>,
    owner_name: Option<String>,
}

pub async fn list_levies(
    pool: &PgPool,
    budget_id: Uuid,
    building_id: Uuid,
    owner_id: Option<Uuid>,
) -> crate::errors::AppResult<Vec<serde_json::Value>> {
    let rows = sqlx::query_as::<_, LevyListRow>(
        r#"SELECT
               lc.id,
               lc.call_label,
               lc.due_date,
               lc.issued_at,
               u.id    AS unit_id,
               u.unit_number,
               SUM(lcu.amount)::numeric                                                              AS amount,
               COALESCE(SUM(lcu.amount) FILTER (WHERE lcu.budget_line_id IS NOT NULL), 0)::numeric AS charge_amount,
               COALESCE(SUM(lcu.amount) FILTER (WHERE lcu.budget_line_id IS NULL), 0)::numeric     AS works_fund_amount,
               lc.status::text                 AS call_status,
               CASE
                   WHEN bool_or(lcu.status::text = 'overdue') THEN 'overdue'
                   WHEN bool_and(lcu.status::text = 'paid')   THEN 'paid'
                   ELSE 'issued'
               END                              AS status,
               MAX(lcu.paid_at)                AS paid_at,
               uo.user_id                      AS owner_id,
               ua.first_name || ' ' || ua.last_name AS owner_name
           FROM levy_call lc
           JOIN budget b              ON b.id           = lc.budget_id AND b.building_id = $2
           JOIN levy_call_unit lcu    ON lcu.levy_call_id = lc.id
           JOIN unit u                ON u.id           = lcu.unit_id
           -- When owner_id filter is set: join to that owner's period covering the levy due_date.
           -- When no filter: get the current owner (end_date IS NULL), LIMIT 1 for safety.
           LEFT JOIN LATERAL (
               SELECT uo2.user_id
               FROM unit_owner uo2
               WHERE uo2.unit_id = u.id
                 AND (
                     ($3::uuid IS NULL AND uo2.end_date IS NULL)
                     OR
                     ($3::uuid IS NOT NULL
                      AND uo2.user_id = $3::uuid
                      AND lc.due_date >= uo2.start_date
                      AND (uo2.end_date IS NULL OR lc.due_date < uo2.end_date))
                 )
               ORDER BY uo2.created_at DESC
               LIMIT 1
           ) uo ON true
           LEFT JOIN user_account ua ON ua.id = uo.user_id
           WHERE lc.budget_id = $1
             AND ($3::uuid IS NULL OR uo.user_id IS NOT NULL)
           GROUP BY lc.id, lc.call_label, lc.due_date, lc.issued_at, lc.status,
                    u.id, u.unit_number, uo.user_id, ua.first_name, ua.last_name
           ORDER BY lc.due_date DESC, u.unit_number"#,
    )
    .bind(budget_id)
    .bind(building_id)
    .bind(owner_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id":                r.id,
                "unit_id":           r.unit_id,
                "unit_number":       r.unit_number,
                "owner_id":          r.owner_id,
                "owner_name":        r.owner_name,
                "amount":            r.amount,
                "charge_amount":     r.charge_amount,
                "works_fund_amount": r.works_fund_amount,
                "due_date":          r.due_date,
                "call_label":        r.call_label,
                "call_status":       r.call_status,
                "status":            r.status,
                "issued_at":         r.issued_at,
                "paid_at":           r.paid_at,
            })
        })
        .collect())
}

#[derive(sqlx::FromRow)]
struct LevyAmountTotalsRow {
    total: Decimal,
    charge_total: Decimal,
    works_total: Decimal,
}

pub async fn levy_amount_totals(
    pool: &PgPool,
    budget_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<LevyAmountTotals> {
    let row = sqlx::query_as::<_, LevyAmountTotalsRow>(
        r#"SELECT
               COALESCE(SUM(lc.charge_amount + lc.works_fund_amount), 0)::numeric AS total,
               COALESCE(SUM(lc.charge_amount), 0)::numeric                        AS charge_total,
               COALESCE(SUM(lc.works_fund_amount), 0)::numeric                    AS works_total
           FROM levy_call lc
           JOIN budget b ON b.id = lc.budget_id
           WHERE lc.budget_id = $1 AND b.building_id = $2"#,
    )
    .bind(budget_id)
    .bind(building_id)
    .fetch_one(pool)
    .await?;
    Ok(LevyAmountTotals {
        total_amount: row.total,
        charge_amount: row.charge_total,
        works_fund_amount: row.works_total,
    })
}

/// Transitions a levy_call from 'draft' to 'issued'. Returns AppError::NotFound if the
/// levy_call doesn't exist in this building, or the immutability error if already issued.
pub async fn issue_levy_call(
    pool: &PgPool,
    levy_call_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<()> {
    let status: Option<String> = sqlx::query_scalar(
        r#"SELECT lc.status::text
           FROM levy_call lc
           JOIN budget b ON b.id = lc.budget_id
           WHERE lc.id = $1 AND b.building_id = $2"#,
    )
    .bind(levy_call_id)
    .bind(building_id)
    .fetch_optional(pool)
    .await?;

    let status = status.ok_or(crate::errors::AppError::NotFound)?;
    crate::domain::immutability::assert_mutable(
        &crate::domain::immutability::Entity::LevyCall,
        &status,
    )?;

    sqlx::query("UPDATE levy_call SET status = 'issued', issued_at = NOW() WHERE id = $1")
        .bind(levy_call_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Mark all levy_call_unit rows for a given (levy_call, unit) pair as paid.
pub async fn mark_paid(
    pool: &PgPool,
    levy_call_id: Uuid,
    unit_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<()> {
    sqlx::query(
        r#"UPDATE levy_call_unit lcu
           SET status = 'paid', paid_at = NOW()
           FROM levy_call lc
           JOIN budget b ON b.id = lc.budget_id
           WHERE lcu.levy_call_id = $1
             AND lcu.unit_id = $2
             AND lcu.levy_call_id = lc.id
             AND b.building_id = $3
             AND lcu.status != 'paid'"#,
    )
    .bind(levy_call_id)
    .bind(unit_id)
    .bind(building_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Creates a credit note (avoir) for a non-draft levy_call.
/// Returns the new levy_call id.
pub async fn create_credit_note(
    pool: &PgPool,
    levy_call_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<Uuid> {
    #[derive(sqlx::FromRow)]
    struct LevyCallSourceRow {
        budget_id: Uuid,
        call_label: String,
        charge_amount: Decimal,
        works_fund_amount: Decimal,
        status: String,
    }

    let source = sqlx::query_as::<_, LevyCallSourceRow>(
        r#"SELECT lc.budget_id, lc.call_label, lc.charge_amount, lc.works_fund_amount,
                  lc.status::text AS status
           FROM levy_call lc
           JOIN budget b ON b.id = lc.budget_id
           WHERE lc.id = $1 AND b.building_id = $2"#,
    )
    .bind(levy_call_id)
    .bind(building_id)
    .fetch_optional(pool)
    .await?
    .ok_or(crate::errors::AppError::NotFound)?;

    if source.status == "draft" {
        return Err(crate::errors::AppError::validation_code(
            "errors.levy_call.credit_note_requires_issued",
        ));
    }

    // Header + per-unit copy must be atomic: a failure on the second insert would
    // otherwise leave an orphan credit-note header without its unit breakdown.
    let mut tx = pool.begin().await?;

    let new_id: Uuid = sqlx::query_scalar(
        r#"INSERT INTO levy_call
               (budget_id, call_label, charge_amount, works_fund_amount, corrects_levy_call_id, status)
           VALUES ($1, $2, $3, $4, $5, 'draft'::levy_status)
           RETURNING id"#,
    )
    .bind(source.budget_id)
    .bind(format!("AVOIR — {}", source.call_label))
    .bind(-source.charge_amount)
    .bind(-source.works_fund_amount)
    .bind(levy_call_id)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"INSERT INTO levy_call_unit
               (levy_call_id, unit_id, budget_line_id, key_id, amount,
                shares_snapshot, shares_total_snapshot)
           SELECT $1, unit_id, budget_line_id, key_id, -amount,
                  shares_snapshot, shares_total_snapshot
           FROM levy_call_unit
           WHERE levy_call_id = $2"#,
    )
    .bind(new_id)
    .bind(levy_call_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(new_id)
}
