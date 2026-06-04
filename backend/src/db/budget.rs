use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::PgPool;
use sqlx::types::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BudgetRow {
    pub id: Uuid,
    pub building_id: Uuid,
    pub fiscal_year: i32,
    pub status: String,
    pub approval_meeting_id: Option<Uuid>,
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
            COALESCE((SELECT SUM(bl.budgeted_amount) FROM budget_line bl WHERE bl.budget_id = b.id), 0)::numeric as "total_forecast!",
            COALESCE((SELECT SUM(bl.actual_amount) FROM budget_line bl WHERE bl.budget_id = b.id), 0)::numeric as "total_actual!",
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
            COALESCE((SELECT SUM(bl.budgeted_amount) FROM budget_line bl WHERE bl.budget_id = b.id), 0)::numeric as "total_forecast!",
            COALESCE((SELECT SUM(bl.actual_amount) FROM budget_line bl WHERE bl.budget_id = b.id), 0)::numeric as "total_actual!",
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
            COALESCE((SELECT SUM(bl.budgeted_amount) FROM budget_line bl WHERE bl.budget_id = b.id), 0)::numeric as total_forecast,
            COALESCE((SELECT SUM(bl.actual_amount) FROM budget_line bl WHERE bl.budget_id = b.id), 0)::numeric as total_actual,
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

pub async fn list_lines(
    pool: &PgPool,
    budget_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<Vec<BudgetLineRow>> {
    Ok(sqlx::query_as!(
        BudgetLineRow,
        r#"SELECT bl.id, bl.budget_id, bl.label, bl.key_id,
           bl.budgeted_amount, bl.actual_amount
           FROM budget_line bl
           JOIN budget b ON b.id = bl.budget_id AND b.building_id = $2
           WHERE bl.budget_id = $1"#,
        budget_id,
        building_id
    )
    .fetch_all(pool)
    .await?)
}

pub async fn create_line(
    pool: &PgPool,
    budget_id: Uuid,
    label: &str,
    key_id: Uuid,
    budgeted_amount: Decimal,
) -> crate::errors::AppResult<BudgetLineRow> {
    Ok(sqlx::query_as!(
        BudgetLineRow,
        r#"INSERT INTO budget_line (budget_id, label, key_id, budgeted_amount)
           VALUES ($1, $2, $3, $4)
           RETURNING id, budget_id, label, key_id, budgeted_amount, actual_amount"#,
        budget_id,
        label,
        key_id,
        budgeted_amount
    )
    .fetch_one(pool)
    .await?)
}

pub async fn delete_line(
    pool: &PgPool,
    line_id: Uuid,
    budget_id: Uuid,
) -> crate::errors::AppResult<()> {
    sqlx::query!(
        "DELETE FROM budget_line WHERE id = $1 AND budget_id = $2",
        line_id,
        budget_id
    )
    .execute(pool)
    .await?;
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

    // Build aggregate totals per budget_line while inserting per-unit rows.
    let mut line_totals: HashMap<Uuid, Decimal> = HashMap::new();

    for l in levies {
        // One levy_call_unit row per (unit, budget_line).
        for line_alloc in &l.line_allocations {
            sqlx::query(
                r#"INSERT INTO levy_call_unit
                       (levy_call_id, unit_id, budget_line_id, key_id, amount,
                        shares_snapshot, shares_total_snapshot)
                   VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            )
            .bind(levy_call_id)
            .bind(l.unit_id)
            .bind(line_alloc.budget_line_id)
            .bind(l.key_id)
            .bind(line_alloc.amount)
            .bind(l.shares_snapshot as i32)
            .bind(l.shares_total_snapshot as i32)
            .execute(&mut *tx)
            .await?;

            *line_totals.entry(line_alloc.budget_line_id).or_default() += line_alloc.amount;
        }

        // One levy_call_unit row for the works_fund portion (budget_line_id = NULL).
        if l.works_fund_amount > Decimal::ZERO {
            sqlx::query(
                r#"INSERT INTO levy_call_unit
                       (levy_call_id, unit_id, budget_line_id, key_id, amount,
                        shares_snapshot, shares_total_snapshot)
                   VALUES ($1, $2, NULL, $3, $4, $5, $6)"#,
            )
            .bind(levy_call_id)
            .bind(l.unit_id)
            .bind(l.key_id)
            .bind(l.works_fund_amount)
            .bind(l.shares_snapshot as i32)
            .bind(l.shares_total_snapshot as i32)
            .execute(&mut *tx)
            .await?;
        }
    }

    // Insert aggregate totals per budget_line into levy_call_line.
    for (budget_line_id, amount) in &line_totals {
        sqlx::query(
            r#"INSERT INTO levy_call_line (levy_call_id, budget_line_id, amount)
               VALUES ($1, $2, $3)"#,
        )
        .bind(levy_call_id)
        .bind(budget_line_id)
        .bind(amount)
        .execute(&mut *tx)
        .await?;
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
    issued_at: DateTime<Utc>,
    unit_id: Uuid,
    unit_number: String,
    amount: Decimal,
    charge_amount: Decimal,
    works_fund_amount: Decimal,
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
               SUM(lcu.amount)::numeric                                               AS amount,
               SUM(lcu.amount) FILTER (WHERE lcu.budget_line_id IS NOT NULL)::numeric AS charge_amount,
               SUM(lcu.amount) FILTER (WHERE lcu.budget_line_id IS NULL)::numeric     AS works_fund_amount,
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
           GROUP BY lc.id, lc.call_label, lc.due_date, lc.issued_at,
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
