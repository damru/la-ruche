use chrono::{DateTime, Days, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::types::Decimal;
use uuid::Uuid;

use crate::db::document::DocumentListRow;
use crate::domain::reminders::{ContractReminderSpec, ReminderKind, compute_reminders};
use crate::errors::AppResult;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
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
    pub document_count: i64,
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

#[derive(Debug, Clone, sqlx::FromRow)]
struct ContractReminderCandidateRow {
    id: Uuid,
    contract_id: Uuid,
    reminder_type: String,
    reminder_date: NaiveDate,
    status: String,
    end_date: Option<NaiveDate>,
    contract_status: String,
}

pub async fn list(pool: &PgPool, building_id: Uuid) -> AppResult<Vec<VendorContractRow>> {
    Ok(sqlx::query_as::<_, VendorContractRow>(
        r#"SELECT vc.id,
                  vc.building_id,
                  vc.vendor_name,
                  vc.vendor_email,
                  vc.vendor_phone,
                  vc.subject,
                  vc.contract_type::text AS contract_type,
                  vc.start_date,
                  vc.end_date,
                  vc.amount,
                  vc.payment_frequency::text AS payment_frequency,
                  vc.auto_renewal,
                  vc.notice_days,
                  vc.status::text AS status,
                  vc.created_at,
                  COUNT(cd.document_id)::bigint AS document_count
           FROM vendor_contract vc
           LEFT JOIN contract_document cd ON cd.contract_id = vc.id
           WHERE vc.building_id = $1
           GROUP BY vc.id
           ORDER BY vc.vendor_name"#,
    )
    .bind(building_id)
    .fetch_all(pool)
    .await?)
}

pub async fn list_all_reminders(
    pool: &PgPool,
    building_id: Uuid,
) -> AppResult<Vec<serde_json::Value>> {
    let rows = sqlx::query_as::<_, ContractReminderCandidateRow>(
        r#"
        SELECT cr.id,
               cr.contract_id,
               cr.reminder_type::text as reminder_type,
               cr.reminder_date,
               cr.status::text as status,
               vc.end_date,
               vc.status::text as contract_status
        FROM contract_reminder cr
        JOIN vendor_contract vc ON vc.id = cr.contract_id
        WHERE vc.building_id = $1
        ORDER BY cr.reminder_date
        "#,
    )
    .bind(building_id)
    .fetch_all(pool)
    .await?;

    let today = Utc::now().date_naive();

    Ok(select_visible_contract_reminders(rows, today)
        .into_iter()
        .map(|row| {
            serde_json::json!({
                "id": row.id,
                "contract_id": row.contract_id,
                "reminder_type": row.reminder_type,
                "reminder_date": row.reminder_date,
                "target_date": reminder_target_date(&row),
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
    let mut contract = sqlx::query_as::<_, VendorContractRow>(
        r#"SELECT vc.id,
                  vc.building_id,
                  vc.vendor_name,
                  vc.vendor_email,
                  vc.vendor_phone,
                  vc.subject,
                  vc.contract_type::text AS contract_type,
                  vc.start_date,
                  vc.end_date,
                  vc.amount,
                  vc.payment_frequency::text AS payment_frequency,
                  vc.auto_renewal,
                  vc.notice_days,
                  vc.status::text AS status,
                  vc.created_at,
                  0::bigint AS document_count
           FROM vendor_contract vc
           WHERE vc.id = $1 AND vc.building_id = $2"#,
    )
    .bind(id)
    .bind(building_id)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = contract.as_mut() {
        row.document_count = count_documents(pool, row.id).await?;
    }

    Ok(contract)
}

/// Creates the contract and computes + inserts all reminders in a transaction.
pub async fn create(
    pool: &PgPool,
    building_id: Uuid,
    user_id: Uuid,
    input: CreateVendorContractInput,
) -> AppResult<VendorContractRow> {
    let mut tx = pool.begin().await?;

    let row = sqlx::query_as::<_, VendorContractRow>(
        r#"INSERT INTO vendor_contract
           (building_id, vendor_name, vendor_email, vendor_phone,
             subject, contract_type, start_date, end_date, amount, payment_frequency,
             auto_renewal, notice_days, created_by)
           VALUES ($1,$2,$3,$4,$5,$6::text::contract_type,$7,$8,$9,$10::text::payment_frequency,$11,$12,$13)
           RETURNING id, building_id, vendor_name, vendor_email, vendor_phone,
           subject, contract_type::text as contract_type, start_date, end_date, amount,
           payment_frequency::text as payment_frequency,
           auto_renewal, notice_days, status::text as status, created_at,
           0::bigint as document_count"#,
    )
    .bind(building_id)
    .bind(input.vendor_name)
    .bind(input.vendor_email)
    .bind(input.vendor_phone)
    .bind(input.subject)
    .bind(input.contract_type)
    .bind(input.start_date)
    .bind(input.end_date)
    .bind(input.amount)
    .bind(input.payment_frequency)
    .bind(input.auto_renewal)
    .bind(input.notice_days)
    .bind(user_id)
    .fetch_one(&mut *tx)
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

    let row = sqlx::query_as::<_, VendorContractRow>(
        r#"UPDATE vendor_contract SET
           vendor_name=$3, vendor_email=$4, vendor_phone=$5,
           subject=$6, contract_type=$7::text::contract_type, start_date=$8, end_date=$9,
           amount=$10, payment_frequency=$11::text::payment_frequency,
           auto_renewal=$12, notice_days=$13, updated_at=NOW()
           WHERE id=$1 AND building_id=$2
           RETURNING id, building_id, vendor_name, vendor_email, vendor_phone,
           subject, contract_type::text as contract_type, start_date, end_date, amount,
           payment_frequency::text as payment_frequency,
           auto_renewal, notice_days, status::text as status, created_at,
           (
               SELECT COUNT(*)::bigint
               FROM contract_document cd
               WHERE cd.contract_id = vendor_contract.id
           ) AS document_count"#,
    )
    .bind(id)
    .bind(building_id)
    .bind(input.vendor_name)
    .bind(input.vendor_email)
    .bind(input.vendor_phone)
    .bind(input.subject)
    .bind(input.contract_type)
    .bind(input.start_date)
    .bind(input.end_date)
    .bind(input.amount)
    .bind(input.payment_frequency)
    .bind(input.auto_renewal)
    .bind(input.notice_days)
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

    let row = if let Some(mut row) = row {
        row.document_count = count_documents(&mut *tx, row.id).await?;
        Some(row)
    } else {
        None
    };

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

pub async fn delete(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
) -> AppResult<Vec<(String, String)>> {
    let mut tx = pool.begin().await?;
    let linked_document_ids = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT cd.document_id
        FROM contract_document cd
        JOIN vendor_contract vc ON vc.id = cd.contract_id
        WHERE cd.contract_id = $1 AND vc.building_id = $2
        "#,
    )
    .bind(id)
    .bind(building_id)
    .fetch_all(&mut *tx)
    .await?;

    sqlx::query!("DELETE FROM contract_reminder WHERE contract_id = $1", id)
        .execute(&mut *tx)
        .await?;
    sqlx::query!("DELETE FROM contract_document WHERE contract_id = $1", id)
        .execute(&mut *tx)
        .await?;

    let result = sqlx::query("DELETE FROM vendor_contract WHERE id = $1 AND building_id = $2")
        .bind(id)
        .bind(building_id)
        .execute(&mut *tx)
        .await?;

    if result.rows_affected() == 0 {
        return Err(crate::errors::AppError::NotFound);
    }

    let deleted_documents =
        crate::db::document::delete_orphaned_tx(&mut tx, &linked_document_ids, building_id).await?;
    tx.commit().await?;
    Ok(deleted_documents)
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
               COALESCE(
                   array_remove(
                       array_agg(DISTINCT vc_link.vendor_name ORDER BY vc_link.vendor_name),
                       NULL
                    ),
                    ARRAY[]::text[]
                ) AS linked_contract_names,
                COALESCE(
                    array_remove(
                        array_agg(DISTINCT r.title ORDER BY r.title),
                        NULL
                    ),
                    ARRAY[]::text[]
                ) AS linked_report_titles,
                ARRAY[]::text[] AS linked_claim_titles,
                COUNT(DISTINCT rd.report_id) AS linked_report_count,
                d.created_at
        FROM contract_document cd_current
        JOIN vendor_contract vc ON vc.id = cd_current.contract_id
        JOIN document d ON d.id = cd_current.document_id
        JOIN contract_document cd_link ON cd_link.document_id = d.id
        LEFT JOIN vendor_contract vc_link ON vc_link.id = cd_link.contract_id AND vc_link.building_id = $2
        LEFT JOIN report_document rd ON rd.document_id = d.id
        LEFT JOIN report r ON r.id = rd.report_id AND r.building_id = $2
        WHERE cd_current.contract_id = $1
          AND vc.building_id = $2
          AND d.building_id = $2
          AND d.visible_to_roles::text[] && $3
        GROUP BY d.id, d.original_name, d.mime_type, d.size_bytes, d.visible_to_roles, d.created_at
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

async fn count_documents<'e, E>(executor: E, contract_id: Uuid) -> AppResult<i64>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    Ok(sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)::bigint
        FROM contract_document
        WHERE contract_id = $1
        "#,
    )
    .bind(contract_id)
    .fetch_one(executor)
    .await?)
}

#[derive(Default)]
struct SelectedReminderRows {
    next_payment: Option<ContractReminderCandidateRow>,
    next_contract_end: Option<ContractReminderCandidateRow>,
}

fn select_visible_contract_reminders(
    rows: Vec<ContractReminderCandidateRow>,
    today: NaiveDate,
) -> Vec<ContractReminderCandidateRow> {
    let payment_threshold = today.checked_add_days(Days::new(30)).unwrap_or(today);
    let contract_end_threshold = today.checked_add_days(Days::new(90)).unwrap_or(today);
    let mut selected_by_contract = std::collections::BTreeMap::<Uuid, SelectedReminderRows>::new();

    for row in rows {
        if row.status != "pending" || !is_contract_reminder_visible(&row, today) {
            continue;
        }

        let target_date = reminder_target_date(&row);
        let selected = selected_by_contract.entry(row.contract_id).or_default();

        if row.reminder_type == "payment_due" {
            if target_date <= payment_threshold && selected.next_payment.is_none() {
                selected.next_payment = Some(row);
            }
            continue;
        }

        if is_contract_end_reminder(&row.reminder_type)
            && target_date <= contract_end_threshold
            && selected.next_contract_end.is_none()
        {
            selected.next_contract_end = Some(row);
        }
    }

    let mut reminders = Vec::new();

    for selected in selected_by_contract.into_values() {
        if let Some(reminder) = selected.next_payment {
            reminders.push(reminder);
        }
        if let Some(reminder) = selected.next_contract_end {
            reminders.push(reminder);
        }
    }

    reminders.sort_by_key(reminder_target_date);
    reminders
}

fn is_contract_reminder_visible(row: &ContractReminderCandidateRow, today: NaiveDate) -> bool {
    if row.contract_status != "active" {
        return false;
    }

    if let Some(end_date) = row.end_date
        && end_date < today
    {
        return false;
    }

    reminder_target_date(row) >= today
}

fn is_contract_end_reminder(reminder_type: &str) -> bool {
    matches!(reminder_type, "renewal" | "expiration" | "termination")
}

fn reminder_target_date(row: &ContractReminderCandidateRow) -> NaiveDate {
    if row.reminder_type == "payment_due" {
        row.reminder_date
    } else {
        row.end_date.unwrap_or(row.reminder_date)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reminder_row(
        contract_id: Uuid,
        reminder_type: &str,
        reminder_date: (i32, u32, u32),
        end_date: Option<(i32, u32, u32)>,
    ) -> ContractReminderCandidateRow {
        ContractReminderCandidateRow {
            id: Uuid::new_v4(),
            contract_id,
            reminder_type: reminder_type.to_string(),
            reminder_date: NaiveDate::from_ymd_opt(
                reminder_date.0,
                reminder_date.1,
                reminder_date.2,
            )
            .unwrap(),
            status: "pending".to_string(),
            end_date: end_date
                .map(|(year, month, day)| NaiveDate::from_ymd_opt(year, month, day).unwrap()),
            contract_status: "active".to_string(),
        }
    }

    #[test]
    fn keeps_only_next_payment_and_next_contract_end_per_contract() {
        let today = NaiveDate::from_ymd_opt(2026, 4, 26).unwrap();
        let contract_id = Uuid::new_v4();
        let reminders = vec![
            reminder_row(
                contract_id,
                "payment_due",
                (2026, 4, 28),
                Some((2026, 6, 20)),
            ),
            reminder_row(
                contract_id,
                "payment_due",
                (2026, 5, 18),
                Some((2026, 6, 20)),
            ),
            reminder_row(
                contract_id,
                "termination",
                (2026, 4, 20),
                Some((2026, 6, 20)),
            ),
            reminder_row(contract_id, "renewal", (2026, 5, 1), Some((2026, 6, 20))),
            reminder_row(
                contract_id,
                "expiration",
                (2026, 5, 21),
                Some((2026, 6, 20)),
            ),
        ];

        let visible = select_visible_contract_reminders(reminders, today);

        assert_eq!(visible.len(), 2);
        assert_eq!(visible[0].reminder_type, "payment_due");
        assert_eq!(
            visible[0].reminder_date,
            NaiveDate::from_ymd_opt(2026, 4, 28).unwrap()
        );
        assert_eq!(visible[1].reminder_type, "termination");
        assert_eq!(
            visible[1].reminder_date,
            NaiveDate::from_ymd_opt(2026, 4, 20).unwrap()
        );
    }

    #[test]
    fn drops_contracts_outside_visibility_windows_or_not_active() {
        let today = NaiveDate::from_ymd_opt(2026, 4, 26).unwrap();
        let active_contract = Uuid::new_v4();
        let terminated_contract = Uuid::new_v4();
        let mut terminated = reminder_row(
            terminated_contract,
            "payment_due",
            (2026, 4, 29),
            Some((2026, 7, 10)),
        );
        terminated.contract_status = "terminated".to_string();

        let visible = select_visible_contract_reminders(
            vec![
                reminder_row(
                    active_contract,
                    "payment_due",
                    (2026, 6, 1),
                    Some((2026, 8, 20)),
                ),
                reminder_row(
                    active_contract,
                    "renewal",
                    (2026, 5, 15),
                    Some((2026, 8, 5)),
                ),
                terminated,
            ],
            today,
        );

        assert!(visible.is_empty());
    }
}
