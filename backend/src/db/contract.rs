use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::types::Decimal;
use uuid::Uuid;

use crate::db::document::DocumentListRow;
use crate::domain::reminders::{ContractReminderSpec, ReminderKind, compute_reminders};
use crate::errors::AppResult;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct VendorContractRow {
    pub id: Uuid,
    pub building_id: Uuid,
    pub vendor_name: String,
    pub vendor_email: Option<String>,
    pub vendor_phone: Option<String>,
    pub subject: String,
    pub contract_type: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub amount: Option<Decimal>,
    pub payment_frequency: Option<String>,
    pub auto_renewal: bool,
    pub notice_days: Option<i32>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVendorContractInput {
    pub vendor_name: String,
    pub vendor_email: Option<String>,
    pub vendor_phone: Option<String>,
    pub subject: String,
    pub contract_type: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub amount: Option<Decimal>,
    pub payment_frequency: Option<String>,
    pub auto_renewal: bool,
    pub notice_days: Option<i32>,
}

pub async fn list(pool: &PgPool, building_id: Uuid) -> AppResult<Vec<VendorContractRow>> {
    Ok(sqlx::query_as!(
        VendorContractRow,
        r#"SELECT id, building_id, vendor_name, vendor_email, vendor_phone,
           subject, contract_type::text as "contract_type!", start_date, end_date, amount,
           payment_frequency::text as payment_frequency,
           auto_renewal, notice_days, status::text as "status!", created_at
           FROM vendor_contract WHERE building_id = $1 ORDER BY vendor_name"#,
        building_id
    )
    .fetch_all(pool)
    .await?)
}

pub async fn list_all_reminders(
    pool: &PgPool,
    building_id: Uuid,
) -> AppResult<Vec<serde_json::Value>> {
    let rows = sqlx::query!(
        r#"
        SELECT cr.id,
               cr.contract_id,
               cr.reminder_type::text as reminder_type,
               cr.reminder_date,
               cr.status::text as status
        FROM contract_reminder cr
        JOIN vendor_contract vc ON vc.id = cr.contract_id
        WHERE vc.building_id = $1
        ORDER BY cr.reminder_date
        "#,
        building_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.id,
                "contract_id": row.contract_id,
                "reminder_type": row.reminder_type,
                "reminder_date": row.reminder_date,
                "status": row.status,
            })
        })
        .collect())
}

pub async fn get(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
) -> AppResult<Option<VendorContractRow>> {
    Ok(sqlx::query_as!(
        VendorContractRow,
        r#"SELECT id, building_id, vendor_name, vendor_email, vendor_phone,
           subject, contract_type::text as "contract_type!", start_date, end_date, amount,
           payment_frequency::text as payment_frequency,
           auto_renewal, notice_days, status::text as "status!", created_at
           FROM vendor_contract WHERE id = $1 AND building_id = $2"#,
        id,
        building_id
    )
    .fetch_optional(pool)
    .await?)
}

/// Creates the contract and computes + inserts all reminders in a transaction.
pub async fn create(
    pool: &PgPool,
    building_id: Uuid,
    user_id: Uuid,
    input: CreateVendorContractInput,
) -> AppResult<VendorContractRow> {
    let mut tx = pool.begin().await?;

    let row = sqlx::query_as!(VendorContractRow,
        r#"INSERT INTO vendor_contract
           (building_id, vendor_name, vendor_email, vendor_phone,
            subject, contract_type, start_date, end_date, amount, payment_frequency,
            auto_renewal, notice_days, created_by)
           VALUES ($1,$2,$3,$4,$5,$6::text::contract_type,$7,$8,$9,$10::text::payment_frequency,$11,$12,$13)
           RETURNING id, building_id, vendor_name, vendor_email, vendor_phone,
           subject, contract_type::text as "contract_type!", start_date, end_date, amount,
           payment_frequency::text as payment_frequency,
           auto_renewal, notice_days, status::text as "status!", created_at"#,
        building_id, input.vendor_name, input.vendor_email, input.vendor_phone,
        input.subject, input.contract_type, input.start_date, input.end_date, input.amount,
        input.payment_frequency, input.auto_renewal, input.notice_days, user_id
    ).fetch_one(&mut *tx).await?;

    let spec = ContractReminderSpec {
        start_date: row.start_date,
        end_date: row.end_date,
        auto_renewal: row.auto_renewal,
        notice_days: row.notice_days.map(|d| d as u32),
        payment_frequency_months: frequency_to_months(row.payment_frequency.as_deref()),
    };

    let today = chrono::Local::now().date_naive();
    for reminder in compute_reminders(&spec, today) {
        let kind_str = reminder_kind_to_str(&reminder.kind);
        sqlx::query!(
            "INSERT INTO contract_reminder (contract_id, reminder_type, reminder_date) VALUES ($1, $2::text::reminder_type, $3)",
            row.id, kind_str, reminder.reminder_date
        ).execute(&mut *tx).await?;
    }

    tx.commit().await?;
    Ok(row)
}

/// Recalculates reminders on every contract update.
pub async fn update(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    input: CreateVendorContractInput,
) -> AppResult<Option<VendorContractRow>> {
    let mut tx = pool.begin().await?;

    let row = sqlx::query_as!(
        VendorContractRow,
        r#"UPDATE vendor_contract SET
           vendor_name=$3, vendor_email=$4, vendor_phone=$5,
           subject=$6, contract_type=$7::text::contract_type, start_date=$8, end_date=$9,
           amount=$10, payment_frequency=$11::text::payment_frequency,
           auto_renewal=$12, notice_days=$13, updated_at=NOW()
           WHERE id=$1 AND building_id=$2
           RETURNING id, building_id, vendor_name, vendor_email, vendor_phone,
           subject, contract_type::text as "contract_type!", start_date, end_date, amount,
           payment_frequency::text as payment_frequency,
           auto_renewal, notice_days, status::text as "status!", created_at"#,
        id,
        building_id,
        input.vendor_name,
        input.vendor_email,
        input.vendor_phone,
        input.subject,
        input.contract_type,
        input.start_date,
        input.end_date,
        input.amount,
        input.payment_frequency,
        input.auto_renewal,
        input.notice_days
    )
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(ref row) = row {
        sqlx::query!(
            "DELETE FROM contract_reminder WHERE contract_id = $1 AND status = 'pending'",
            id
        )
        .execute(&mut *tx)
        .await?;

        let spec = ContractReminderSpec {
            start_date: row.start_date,
            end_date: row.end_date,
            auto_renewal: row.auto_renewal,
            notice_days: row.notice_days.map(|d| d as u32),
            payment_frequency_months: frequency_to_months(row.payment_frequency.as_deref()),
        };

        let today = chrono::Local::now().date_naive();
        for reminder in compute_reminders(&spec, today) {
            let kind_str = reminder_kind_to_str(&reminder.kind);
            sqlx::query!(
                "INSERT INTO contract_reminder (contract_id, reminder_type, reminder_date) VALUES ($1, $2::text::reminder_type, $3)",
                id, kind_str, reminder.reminder_date
            ).execute(&mut *tx).await?;
        }
    }

    tx.commit().await?;
    Ok(row)
}

pub async fn terminate(pool: &PgPool, id: Uuid, building_id: Uuid) -> AppResult<()> {
    sqlx::query!(
        "UPDATE vendor_contract SET status = 'terminated', updated_at = NOW() WHERE id=$1 AND building_id=$2",
        id, building_id
    ).execute(pool).await?;
    Ok(())
}

pub async fn list_reminders(
    pool: &PgPool,
    contract_id: Uuid,
    building_id: Uuid,
) -> AppResult<Vec<serde_json::Value>> {
    let rows = sqlx::query!(
        r#"SELECT cr.id, cr.reminder_type::text as reminder_type, cr.reminder_date, cr.status::text as status
           FROM contract_reminder cr
           JOIN vendor_contract vc ON vc.id = cr.contract_id AND vc.building_id = $2
           WHERE cr.contract_id = $1 ORDER BY cr.reminder_date"#,
        contract_id, building_id
    ).fetch_all(pool).await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "contract_id": contract_id,
                "reminder_type": r.reminder_type,
                "reminder_date": r.reminder_date,
                "status": r.status,
            })
        })
        .collect())
}

pub async fn acknowledge_reminder(
    pool: &PgPool,
    reminder_id: Uuid,
    contract_id: Uuid,
    building_id: Uuid,
) -> AppResult<()> {
    sqlx::query!(
        r#"UPDATE contract_reminder cr SET status = 'acknowledged', updated_at = NOW()
           FROM vendor_contract vc
           WHERE cr.id = $1 AND cr.contract_id = $2 AND vc.id = cr.contract_id AND vc.building_id = $3"#,
        reminder_id, contract_id, building_id
    ).execute(pool).await?;
    Ok(())
}

pub async fn list_documents(
    pool: &PgPool,
    contract_id: Uuid,
    building_id: Uuid,
    roles: &[String],
) -> AppResult<Vec<DocumentListRow>> {
    Ok(sqlx::query_as::<_, DocumentListRow>(
        r#"
        SELECT d.id,
               d.original_name,
               d.mime_type,
               d.size_bytes,
               d.visible_to_roles::text[] AS visible_roles,
               d.created_at
        FROM contract_document cd
        JOIN vendor_contract vc ON vc.id = cd.contract_id
        JOIN document d ON d.id = cd.document_id
        WHERE cd.contract_id = $1
          AND vc.building_id = $2
          AND d.building_id = $2
          AND d.visible_to_roles::text[] && $3
        ORDER BY d.created_at DESC
        "#,
    )
    .bind(contract_id)
    .bind(building_id)
    .bind(roles)
    .fetch_all(pool)
    .await?)
}

pub async fn attach_document(
    pool: &PgPool,
    contract_id: Uuid,
    document_id: Uuid,
    building_id: Uuid,
) -> AppResult<bool> {
    let pair_exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM vendor_contract vc
            JOIN document d ON d.id = $2
            WHERE vc.id = $1
              AND vc.building_id = $3
              AND d.building_id = $3
        )
        "#,
    )
    .bind(contract_id)
    .bind(document_id)
    .bind(building_id)
    .fetch_one(pool)
    .await?;

    if !pair_exists {
        return Ok(false);
    }

    sqlx::query(
        r#"
        INSERT INTO contract_document (contract_id, document_id)
        VALUES ($1, $2)
        ON CONFLICT (contract_id, document_id) DO NOTHING
        "#,
    )
    .bind(contract_id)
    .bind(document_id)
    .execute(pool)
    .await?;

    Ok(true)
}

pub async fn detach_document(
    pool: &PgPool,
    contract_id: Uuid,
    document_id: Uuid,
    building_id: Uuid,
) -> AppResult<bool> {
    let result = sqlx::query(
        r#"
        DELETE FROM contract_document cd
        USING vendor_contract vc
        WHERE cd.contract_id = $1
          AND cd.document_id = $2
          AND vc.id = cd.contract_id
          AND vc.building_id = $3
        "#,
    )
    .bind(contract_id)
    .bind(document_id)
    .bind(building_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

fn frequency_to_months(p: Option<&str>) -> Option<u32> {
    match p {
        Some("monthly") => Some(1),
        Some("quarterly") => Some(3),
        Some("biannual") => Some(6),
        Some("annual") => Some(12),
        _ => None,
    }
}

fn reminder_kind_to_str(kind: &ReminderKind) -> &'static str {
    match kind {
        ReminderKind::Renewal => "renewal",
        ReminderKind::PaymentDue => "payment_due",
        ReminderKind::Expiration => "expiration",
        ReminderKind::Termination => "termination",
    }
}
