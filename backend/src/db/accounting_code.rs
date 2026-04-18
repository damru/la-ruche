use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AccountingCodeRow {
    pub code: String,
    pub label: String,
    pub category: String,
}

pub async fn list(pool: &PgPool) -> crate::errors::AppResult<Vec<AccountingCodeRow>> {
    Ok(sqlx::query_as::<_, AccountingCodeRow>(
        "SELECT code, label, category FROM accounting_code ORDER BY code",
    )
    .fetch_all(pool)
    .await?)
}
