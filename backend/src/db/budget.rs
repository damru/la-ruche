use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::PgPool;
use sqlx::types::Decimal;
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
    pub unit_id: Uuid,
    pub amount: Decimal,
    pub due_date: NaiveDate,
    pub status: String,
    pub issued_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub key_id: Option<Uuid>,
    pub shares_snapshot: Option<i32>,
    pub shares_total_snapshot: Option<i32>,
    pub call_label: String,
    pub charge_amount: Decimal,
    pub works_fund_amount: Decimal,
}

/// One entry per unit to insert as levy_call.
pub struct LevyInsert {
    pub unit_id: Uuid,
    pub key_id: Uuid,
    pub amount: Decimal,
    pub charge_amount: Decimal,
    pub works_fund_amount: Decimal,
    pub shares_snapshot: i64,
    pub shares_total_snapshot: i64,
}

pub struct LevyAmountTotals {
    pub total_amount: Decimal,
    pub charge_amount: Decimal,
    pub works_fund_amount: Decimal,
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
    Ok(sqlx::query_as!(
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
    .fetch_one(pool)
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

pub async fn generate_levies(
    pool: &PgPool,
    budget_id: Uuid,
    _building_id: Uuid,
    due_date: NaiveDate,
    call_label: &str,
    levies: &[LevyInsert],
) -> crate::errors::AppResult<Vec<LevyCallRow>> {
    let mut tx = pool.begin().await?;
    let mut result = Vec::new();

    for l in levies {
        let row = sqlx::query_as!(
            LevyCallRow,
            r#"INSERT INTO levy_call
                   (budget_id, unit_id, amount, charge_amount, works_fund_amount, due_date, call_label,
                    key_id, shares_snapshot, shares_total_snapshot)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                ON CONFLICT DO NOTHING
                RETURNING id, budget_id, unit_id, amount, due_date,
                 status::text as "status!", issued_at, paid_at,
                 key_id, shares_snapshot, shares_total_snapshot, call_label, charge_amount, works_fund_amount"#,
            budget_id,
            l.unit_id,
            l.amount,
            l.charge_amount,
            l.works_fund_amount,
            due_date,
            call_label,
            l.key_id,
            l.shares_snapshot as i32,
            l.shares_total_snapshot as i32,
        )
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(row) = row {
            result.push(row);
        }
    }

    tx.commit().await?;
    Ok(result)
}

pub async fn list_levies(
    pool: &PgPool,
    budget_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<Vec<serde_json::Value>> {
    let rows = sqlx::query!(
        r#"SELECT lc.id, lc.unit_id, u.unit_number, lc.amount, lc.charge_amount, lc.works_fund_amount, lc.due_date, lc.call_label,
           lc.status::text as status, lc.issued_at, lc.paid_at,
            ua.first_name || ' ' || ua.last_name as owner_name
           FROM levy_call lc
           JOIN budget b ON b.id = lc.budget_id AND b.building_id = $2
           JOIN unit u ON u.id = lc.unit_id
           LEFT JOIN unit_owner uo ON uo.unit_id = u.id AND uo.end_date IS NULL
           LEFT JOIN user_account ua ON ua.id = uo.user_id
           WHERE lc.budget_id = $1
           ORDER BY lc.due_date DESC, u.unit_number"#,
        budget_id,
        building_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "unit_id": r.unit_id,
                "unit_number": r.unit_number,
                "owner_name": r.owner_name,
                "amount": r.amount,
                "charge_amount": r.charge_amount,
                "works_fund_amount": r.works_fund_amount,
                "due_date": r.due_date,
                "call_label": r.call_label,
                "status": r.status,
                "issued_at": r.issued_at,
                "paid_at": r.paid_at,
            })
        })
        .collect())
}

pub async fn levy_amount_totals(
    pool: &PgPool,
    budget_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<LevyAmountTotals> {
    let row = sqlx::query!(
        r#"SELECT
               COALESCE(SUM(lc.amount), 0)::numeric as "total!",
               COALESCE(SUM(lc.charge_amount), 0)::numeric as "charge_total!",
               COALESCE(SUM(lc.works_fund_amount), 0)::numeric as "works_total!"
           FROM levy_call lc
           JOIN budget b ON b.id = lc.budget_id
           WHERE lc.budget_id = $1 AND b.building_id = $2"#,
        budget_id,
        building_id
    )
    .fetch_one(pool)
    .await?;
    Ok(LevyAmountTotals {
        total_amount: row.total,
        charge_amount: row.charge_total,
        works_fund_amount: row.works_total,
    })
}

pub async fn mark_paid(
    pool: &PgPool,
    levy_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<()> {
    sqlx::query!(
        r#"UPDATE levy_call lc SET status = 'paid', paid_at = NOW()
           FROM budget b
           WHERE lc.id = $1 AND lc.budget_id = b.id AND b.building_id = $2
           AND lc.status != 'paid'"#,
        levy_id,
        building_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_works_fund(
    pool: &PgPool,
    building_id: Uuid,
    fiscal_year: i32,
) -> crate::errors::AppResult<Option<serde_json::Value>> {
    let row = sqlx::query!(
        "SELECT id, fiscal_year, levied_amount, available_amount FROM works_fund WHERE building_id = $1 AND fiscal_year = $2",
        building_id, fiscal_year
    ).fetch_optional(pool).await?;

    Ok(row.map(|r| {
        serde_json::json!({
            "id": r.id,
            "fiscal_year": r.fiscal_year,
            "levied_amount": r.levied_amount,
            "available_amount": r.available_amount,
        })
    }))
}
