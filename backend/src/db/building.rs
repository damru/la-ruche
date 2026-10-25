use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BuildingRow {
    pub id: Uuid,
    pub name: String,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub postal_code: String,
    pub city: String,
    pub manager_siret: Option<String>,
    pub shares_total: i32,
    pub bylaws_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBuildingInput {
    pub name: String,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub postal_code: String,
    pub city: String,
    pub manager_siret: Option<String>,
    pub shares_total: i32,
    pub bylaws_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize)]
pub struct BuildingStats {
    pub nb_units: i64,
    pub nb_users: i64,
    pub nb_tenants: i64,
    pub shares_total: i32,
    pub works_fund_available: sqlx::types::Decimal,
    pub works_fund_pct: Option<i32>,
}

pub async fn get(pool: &PgPool, id: Uuid) -> crate::errors::AppResult<Option<BuildingRow>> {
    Ok(sqlx::query_as!(
        BuildingRow,
        r#"SELECT id, name, address_line1, address_line2, postal_code, city, manager_siret,
           shares_total, bylaws_date, created_at, updated_at
           FROM building WHERE id = $1 AND active = true"#,
        id
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn update(
    pool: &PgPool,
    id: Uuid,
    input: UpdateBuildingInput,
) -> crate::errors::AppResult<Option<BuildingRow>> {
    Ok(sqlx::query_as::<_, BuildingRow>(
        "UPDATE building SET \
            name = $2, address_line1 = $3, address_line2 = $4, \
            postal_code = $5, city = $6, manager_siret = $7, \
            shares_total = $8, bylaws_date = $9, updated_at = NOW() \
         WHERE id = $1 AND active = true \
         RETURNING id, name, address_line1, address_line2, postal_code, city, manager_siret, \
         shares_total, bylaws_date, created_at, updated_at",
    )
    .bind(id)
    .bind(input.name)
    .bind(input.address_line1)
    .bind(input.address_line2)
    .bind(input.postal_code)
    .bind(input.city)
    .bind(input.manager_siret)
    .bind(input.shares_total)
    .bind(input.bylaws_date)
    .fetch_optional(pool)
    .await?)
}

pub async fn get_stats(pool: &PgPool, id: Uuid) -> crate::errors::AppResult<BuildingStats> {
    let row = sqlx::query(
        r#"
        SELECT
            (SELECT COUNT(*) FROM unit WHERE building_id = $1 AND active = true) as nb_units,
            (SELECT COUNT(DISTINCT user_id) FROM user_building WHERE building_id = $1) as nb_users,
            (SELECT COUNT(DISTINCT ub.user_id) FROM user_building ub
             WHERE ub.building_id = $1 AND $2::user_role = ANY(ub.roles)) as nb_tenants,
            b.shares_total,
            COALESCE(wf.available_amount, 0) as works_fund_available,
            CASE WHEN wf.levied_amount > 0
                THEN ROUND(((wf.levied_amount - wf.available_amount) / wf.levied_amount * 100))::int
                ELSE NULL END as works_fund_pct
        FROM building b
        LEFT JOIN works_fund wf ON wf.building_id = b.id
            AND wf.fiscal_year = EXTRACT(YEAR FROM NOW())::int
        WHERE b.id = $1
        "#,
    )
    .bind(id)
    .bind(crate::domain::Role::Tenant.as_str())
    .fetch_one(pool)
    .await?;

    Ok(BuildingStats {
        nb_units: row
            .try_get::<Option<i64>, _>("nb_units")
            .ok()
            .flatten()
            .unwrap_or(0),
        nb_users: row
            .try_get::<Option<i64>, _>("nb_users")
            .ok()
            .flatten()
            .unwrap_or(0),
        nb_tenants: row
            .try_get::<Option<i64>, _>("nb_tenants")
            .ok()
            .flatten()
            .unwrap_or(0),
        shares_total: row.try_get("shares_total")?,
        works_fund_available: row
            .try_get::<sqlx::types::Decimal, _>("works_fund_available")
            .unwrap_or_default(),
        works_fund_pct: row
            .try_get::<Option<i32>, _>("works_fund_pct")
            .ok()
            .flatten(),
    })
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AllocationKeyRow {
    pub id: Uuid,
    pub building_id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

pub async fn list_keys(
    pool: &PgPool,
    building_id: Uuid,
) -> crate::errors::AppResult<Vec<AllocationKeyRow>> {
    Ok(sqlx::query_as!(AllocationKeyRow,
        "SELECT id, building_id, name, description FROM allocation_key WHERE building_id = $1 ORDER BY name",
        building_id
    ).fetch_all(pool).await?)
}

pub async fn create_key(
    pool: &PgPool,
    building_id: Uuid,
    name: String,
    description: Option<String>,
) -> crate::errors::AppResult<AllocationKeyRow> {
    Ok(sqlx::query_as!(AllocationKeyRow,
        "INSERT INTO allocation_key (building_id, name, description) VALUES ($1, $2, $3) RETURNING id, building_id, name, description",
        building_id, name, description
    ).fetch_one(pool).await?)
}

pub async fn delete_key(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<()> {
    sqlx::query!(
        "DELETE FROM allocation_key WHERE id = $1 AND building_id = $2",
        id,
        building_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_key(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
    name: String,
    description: Option<String>,
) -> crate::errors::AppResult<Option<AllocationKeyRow>> {
    Ok(sqlx::query_as!(
        AllocationKeyRow,
        r#"UPDATE allocation_key SET name = $3, description = $4
           WHERE id = $1 AND building_id = $2
           RETURNING id, building_id, name, description"#,
        id,
        building_id,
        name,
        description
    )
    .fetch_optional(pool)
    .await?)
}
