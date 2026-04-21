use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub last_name: String,
    pub first_name: String,
    pub password_hash: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

pub async fn find_by_email(
    pool: &PgPool,
    email: &str,
) -> crate::errors::AppResult<Option<UserRow>> {
    Ok(sqlx::query_as!(UserRow,
        "SELECT id, email, last_name, first_name, password_hash, active, created_at FROM user_account WHERE email = $1",
        email
    ).fetch_optional(pool).await?)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> crate::errors::AppResult<Option<UserRow>> {
    Ok(sqlx::query_as!(UserRow,
        "SELECT id, email, last_name, first_name, password_hash, active, created_at FROM user_account WHERE id = $1",
        id
    ).fetch_optional(pool).await?)
}

pub async fn list(
    pool: &PgPool,
    building_id: Uuid,
) -> crate::errors::AppResult<Vec<serde_json::Value>> {
    let rows = sqlx::query!(
        r#"
        SELECT u.id, u.email, u.last_name, u.first_name, u.active,
               ub.roles::text[] as roles
        FROM user_account u
        JOIN user_building ub ON ub.user_id = u.id
        WHERE ub.building_id = $1 AND u.active = true
        ORDER BY u.last_name, u.first_name
        "#,
        building_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "email": r.email,
                "last_name": r.last_name,
                "first_name": r.first_name,
                "active": r.active,
                "roles": r.roles,
            })
        })
        .collect())
}

pub async fn create(
    pool: &PgPool,
    email: &str,
    last_name: &str,
    first_name: &str,
    password_hash: &str,
) -> crate::errors::AppResult<UserRow> {
    Ok(sqlx::query_as!(
        UserRow,
        r#"INSERT INTO user_account (email, last_name, first_name, password_hash)
           VALUES ($1, $2, $3, $4)
           RETURNING id, email, last_name, first_name, password_hash, active, created_at"#,
        email,
        last_name,
        first_name,
        password_hash
    )
    .fetch_one(pool)
    .await?)
}

pub async fn update_password(pool: &PgPool, id: Uuid, hash: &str) -> crate::errors::AppResult<()> {
    sqlx::query!(
        "UPDATE user_account SET password_hash = $1, updated_at = NOW() WHERE id = $2",
        hash,
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_passkey(
    pool: &PgPool,
    id: Uuid,
    credential: serde_json::Value,
) -> crate::errors::AppResult<()> {
    sqlx::query!(
        "UPDATE user_account SET passkey_credential = $1, updated_at = NOW() WHERE id = $2",
        credential,
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn deactivate(
    pool: &PgPool,
    id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<()> {
    sqlx::query!(
        "UPDATE user_building SET end_date = CURRENT_DATE WHERE user_id = $1 AND building_id = $2",
        id,
        building_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_roles(
    pool: &PgPool,
    user_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<Vec<String>> {
    let row = sqlx::query!(
        "SELECT roles::text[] as roles FROM user_building WHERE user_id = $1 AND building_id = $2",
        user_id,
        building_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.and_then(|r| r.roles).unwrap_or_default())
}

pub async fn update_roles(
    pool: &PgPool,
    user_id: Uuid,
    building_id: Uuid,
    roles: &[String],
) -> crate::errors::AppResult<()> {
    sqlx::query!(
        r#"UPDATE user_building SET roles = $3::user_role[], updated_at = NOW()
           WHERE user_id = $1 AND building_id = $2"#,
        user_id,
        building_id,
        roles as _
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn create_token(
    pool: &PgPool,
    email: &str,
    token_type: &str,
    token_hash: &str,
    building_id: Option<Uuid>,
    invited_role: Option<&str>,
    expires_at: DateTime<Utc>,
) -> crate::errors::AppResult<Uuid> {
    let id = sqlx::query_scalar!(
        r#"INSERT INTO user_token (email, token_type, token_hash, building_id, invited_role, expires_at)
           VALUES ($1, $2, $3, $4, $5::user_role, $6)
           RETURNING id"#,
        email, token_type, token_hash, building_id, invited_role as _, expires_at
    ).fetch_one(pool).await?;
    Ok(id)
}

pub async fn consume_token(
    pool: &PgPool,
    token_hash: &str,
    token_type: &str,
) -> crate::errors::AppResult<Option<serde_json::Value>> {
    let row = sqlx::query!(
        r#"UPDATE user_token SET used_at = NOW()
           WHERE token_hash = $1 AND token_type = $2 AND used_at IS NULL AND expires_at > NOW()
           RETURNING id, email, building_id, invited_role::text as invited_role"#,
        token_hash,
        token_type
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| {
        serde_json::json!({
            "id": r.id,
            "email": r.email,
            "building_id": r.building_id,
            "invited_role": r.invited_role,
        })
    }))
}

pub async fn save_refresh_token(
    pool: &PgPool,
    user_id: Uuid,
    building_id: Uuid,
    email: &str,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> crate::errors::AppResult<()> {
    sqlx::query!(
        r#"INSERT INTO user_token (user_id, email, token_type, token_hash, building_id, expires_at)
           VALUES ($1, $2, 'refresh', $3, $4, $5)"#,
        user_id,
        email,
        token_hash,
        building_id,
        expires_at
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn rotate_refresh_token(
    pool: &PgPool,
    old_hash: &str,
    new_hash: &str,
    expires_at: DateTime<Utc>,
) -> crate::errors::AppResult<Option<serde_json::Value>> {
    let row = sqlx::query!(
        r#"UPDATE user_token SET used_at = NOW()
           WHERE token_hash = $1 AND token_type = 'refresh' AND used_at IS NULL AND expires_at > NOW()
           RETURNING user_id, building_id, email"#,
        old_hash
    ).fetch_optional(pool).await?;

    if let Some(ref r) = row {
        sqlx::query!(
            r#"INSERT INTO user_token (user_id, email, token_type, token_hash, building_id, expires_at)
               VALUES ($1, $2, 'refresh', $3, $4, $5)"#,
            r.user_id, r.email, new_hash, r.building_id, expires_at
        ).execute(pool).await?;
    }

    Ok(row.map(|r| {
        serde_json::json!({
            "user_id": r.user_id,
            "building_id": r.building_id,
            "email": r.email,
        })
    }))
}
