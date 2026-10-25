use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// `user_token.token_type` values (TEXT column, not a PG enum — see migration 0001).
pub mod token_type {
    pub const INVITATION: &str = "invitation";
    pub const REFRESH: &str = "refresh";
    pub const RESET_PASSWORD: &str = "reset_password";
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub email: Option<String>,
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
    Ok(sqlx::query_as::<_, UserRow>(
        "SELECT id, email, last_name, first_name, password_hash, active, created_at FROM user_account WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> crate::errors::AppResult<Option<UserRow>> {
    Ok(sqlx::query_as::<_, UserRow>(
        "SELECT id, email, last_name, first_name, password_hash, active, created_at FROM user_account WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn list(
    pool: &PgPool,
    building_id: Uuid,
) -> crate::errors::AppResult<Vec<serde_json::Value>> {
    let rows = sqlx::query(
        r#"SELECT u.id, u.email, u.last_name, u.first_name, u.active, u.phone,
                  ub.roles::text[] AS roles,
                  CASE
                    WHEN ub.end_date IS NOT NULL THEN 'archived'
                    WHEN u.password_hash IS NOT NULL OR u.passkey_credential IS NOT NULL THEN 'active'
                    ELSE 'no_access'
                  END AS status
           FROM user_account u
           JOIN user_building ub ON ub.user_id = u.id AND ub.building_id = $1
           WHERE u.active = true
             AND (
               ub.end_date IS NOT NULL
               OR NOT EXISTS (
                 SELECT 1 FROM user_token ut
                 WHERE ut.user_id = u.id
                   AND ut.token_type = $2
                   AND ut.used_at IS NULL
                   AND ut.expires_at > NOW()
               )
             )
           ORDER BY u.last_name NULLS LAST, u.first_name NULLS LAST"#,
    )
    .bind(building_id)
    .bind(token_type::INVITATION)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let roles: Vec<String> = r.get::<Option<Vec<String>>, _>("roles").unwrap_or_default();
            serde_json::json!({
                "id": r.get::<Uuid, _>("id"),
                "email": r.get::<Option<String>, _>("email"),
                "last_name": r.get::<String, _>("last_name"),
                "first_name": r.get::<String, _>("first_name"),
                "active": r.get::<bool, _>("active"),
                "phone": r.get::<Option<String>, _>("phone"),
                "roles": roles,
                "status": r.get::<String, _>("status"),
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
    Ok(sqlx::query_as::<_, UserRow>(
        r#"INSERT INTO user_account (email, last_name, first_name, password_hash)
           VALUES ($1, $2, $3, $4)
           RETURNING id, email, last_name, first_name, password_hash, active, created_at"#,
    )
    .bind(email)
    .bind(last_name)
    .bind(first_name)
    .bind(password_hash)
    .fetch_one(pool)
    .await?)
}

pub async fn get_phone(pool: &PgPool, id: Uuid) -> crate::errors::AppResult<Option<String>> {
    let row = sqlx::query("SELECT phone FROM user_account WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row.and_then(|r| {
        use sqlx::Row;
        r.try_get::<Option<String>, _>("phone").ok().flatten()
    }))
}

pub async fn update_phone(
    pool: &PgPool,
    id: Uuid,
    phone: Option<&str>,
) -> crate::errors::AppResult<()> {
    sqlx::query("UPDATE user_account SET phone = $1, updated_at = NOW() WHERE id = $2")
        .bind(phone)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
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
    let row = sqlx::query(
        r#"UPDATE user_token SET used_at = NOW()
           WHERE token_hash = $1 AND token_type = $2 AND used_at IS NULL AND expires_at > NOW()
           RETURNING id, email, building_id, invited_role::text as invited_role"#,
    )
    .bind(token_hash)
    .bind(token_type)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| {
        serde_json::json!({
            "id": r.get::<Uuid, _>("id"),
            "email": r.get::<String, _>("email"),
            "building_id": r.get::<Option<Uuid>, _>("building_id"),
            "invited_role": r.get::<Option<String>, _>("invited_role"),
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
    sqlx::query(
        r#"INSERT INTO user_token (user_id, email, token_type, token_hash, building_id, expires_at)
           VALUES ($1, $2, $3, $4, $5, $6)"#,
    )
    .bind(user_id)
    .bind(email)
    .bind(token_type::REFRESH)
    .bind(token_hash)
    .bind(building_id)
    .bind(expires_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Create an invitation token with optional unit association (EP-002-S03).
pub async fn create_invite_token(
    pool: &PgPool,
    email: &str,
    token_hash: &str,
    building_id: Uuid,
    invited_role: &str,
    unit_id: Option<Uuid>,
    expires_at: DateTime<Utc>,
) -> crate::errors::AppResult<Uuid> {
    let id: Uuid = sqlx::query_scalar(
        r#"INSERT INTO user_token (email, token_type, token_hash, building_id, invited_role, unit_id, expires_at)
           VALUES ($1, $2, $3, $4, $5::user_role, $6, $7)
           RETURNING id"#,
    )
    .bind(email)
    .bind(token_type::INVITATION)
    .bind(token_hash)
    .bind(building_id)
    .bind(invited_role)
    .bind(unit_id)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

/// Invalidate any previous unused invitation tokens for this email in this building.
/// Called before generating a new invite (AC-GP-002-G).
pub async fn revoke_pending_invites(
    pool: &PgPool,
    email: &str,
    building_id: Uuid,
) -> crate::errors::AppResult<()> {
    sqlx::query(
        r#"UPDATE user_token SET used_at = NOW()
           WHERE email = $1 AND building_id = $2 AND token_type = $3 AND used_at IS NULL"#,
    )
    .bind(email)
    .bind(building_id)
    .bind(token_type::INVITATION)
    .execute(pool)
    .await?;
    Ok(())
}

/// Peek at an invitation token without consuming it (for validate endpoint).
pub struct InviteTokenInfo {
    pub email: String,
    pub invited_role: Option<String>,
    pub unit_id: Option<Uuid>,
    pub expires_at: DateTime<Utc>,
    /// Set when the invite is for a pre-registered (no_access) user.
    pub user_id: Option<Uuid>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

pub async fn peek_invite_token(
    pool: &PgPool,
    token_hash: &str,
) -> crate::errors::AppResult<Option<InviteTokenInfo>> {
    let row = sqlx::query(
        r#"SELECT ut.email, ut.invited_role::text AS invited_role, ut.unit_id,
                  ut.expires_at, ut.used_at, ut.user_id,
                  ua.first_name, ua.last_name
           FROM user_token ut
           LEFT JOIN user_account ua ON ua.id = ut.user_id
           WHERE ut.token_hash = $1 AND ut.token_type = $2"#,
    )
    .bind(token_hash)
    .bind(token_type::INVITATION)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| InviteTokenInfo {
        email: r.get("email"),
        invited_role: r.get("invited_role"),
        unit_id: r.get("unit_id"),
        expires_at: r.get("expires_at"),
        user_id: r.get("user_id"),
        first_name: r.get("first_name"),
        last_name: r.get("last_name"),
    }))
}

/// Consume an invitation token, returning its payload.
pub struct ConsumedInviteToken {
    pub email: String,
    pub building_id: Option<Uuid>,
    pub invited_role: Option<String>,
    pub unit_id: Option<Uuid>,
    /// Set when the invite is for a pre-registered (no_access) user — skip account creation.
    pub user_id: Option<Uuid>,
}

pub async fn consume_invite_token(
    pool: &PgPool,
    token_hash: &str,
) -> crate::errors::AppResult<Option<ConsumedInviteToken>> {
    let row = sqlx::query(
        r#"UPDATE user_token SET used_at = NOW()
           WHERE token_hash = $1 AND token_type = $2 AND used_at IS NULL AND expires_at > NOW()
           RETURNING email, building_id, invited_role::text AS invited_role, unit_id, user_id"#,
    )
    .bind(token_hash)
    .bind(token_type::INVITATION)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| ConsumedInviteToken {
        email: r.get("email"),
        building_id: r.get("building_id"),
        invited_role: r.get("invited_role"),
        unit_id: r.get("unit_id"),
        user_id: r.get("user_id"),
    }))
}

/// Add a user to a building with specified roles.
pub async fn create_user_building(
    pool: &PgPool,
    user_id: Uuid,
    building_id: Uuid,
    role: &str,
) -> crate::errors::AppResult<()> {
    sqlx::query(
        r#"INSERT INTO user_building (user_id, building_id, roles)
           VALUES ($1, $2, ARRAY[$3::user_role])
           ON CONFLICT (user_id, building_id) DO NOTHING"#,
    )
    .bind(user_id)
    .bind(building_id)
    .bind(role)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get full user info including associated lots for this building.
pub struct UserDetail {
    pub id: Uuid,
    pub email: Option<String>,
    pub last_name: String,
    pub first_name: String,
    pub phone: Option<String>,
    pub active: bool,
    pub roles: Vec<String>,
    pub created_at: DateTime<Utc>,
    /// True if the user has a password or passkey credential (can log in).
    pub has_access: bool,
}

pub async fn get_user_detail(
    pool: &PgPool,
    user_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<Option<UserDetail>> {
    let row = sqlx::query(
        r#"SELECT ua.id, ua.email, ua.last_name, ua.first_name, ua.phone, ua.active,
                  ub.roles::text[] AS roles, ua.created_at,
                  (ua.password_hash IS NOT NULL OR ua.passkey_credential IS NOT NULL) AS has_access
           FROM user_account ua
           JOIN user_building ub ON ub.user_id = ua.id AND ub.building_id = $2
           WHERE ua.id = $1"#,
    )
    .bind(user_id)
    .bind(building_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| {
        let roles: Vec<String> = r.get::<Option<Vec<String>>, _>("roles").unwrap_or_default();
        UserDetail {
            id: r.get("id"),
            email: r.get("email"),
            last_name: r.get("last_name"),
            first_name: r.get("first_name"),
            phone: r.get("phone"),
            active: r.get("active"),
            roles,
            created_at: r.get("created_at"),
            has_access: r.get("has_access"),
        }
    }))
}

/// Get units currently owned by a user in this building.
pub struct UserUnit {
    pub unit_id: Uuid,
    pub unit_number: String,
    pub unit_type: String,
    pub start_date: NaiveDate,
}

pub async fn get_user_units(
    pool: &PgPool,
    user_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<Vec<UserUnit>> {
    let rows = sqlx::query(
        r#"SELECT u.id AS unit_id, u.unit_number, u.unit_type::text AS unit_type, uo.start_date
           FROM unit_owner uo
           JOIN unit u ON u.id = uo.unit_id AND u.building_id = $2
           WHERE uo.user_id = $1 AND uo.end_date IS NULL"#,
    )
    .bind(user_id)
    .bind(building_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| UserUnit {
            unit_id: r.get("unit_id"),
            unit_number: r.get("unit_number"),
            unit_type: r.get("unit_type"),
            start_date: r.get("start_date"),
        })
        .collect())
}

/// List pending (unused, non-expired) invitations for a building.
/// For invites linked to a pre-registered user (user_id IS NOT NULL), the name
/// is taken from the existing user_account record.
pub async fn list_pending_invites(
    pool: &PgPool,
    building_id: Uuid,
) -> crate::errors::AppResult<Vec<serde_json::Value>> {
    let rows = sqlx::query(
        r#"SELECT ut.id, ut.email, ut.invited_role::text AS invited_role, ut.expires_at,
                  COALESCE(ua.first_name, '') AS first_name,
                  COALESCE(ua.last_name, '')  AS last_name
           FROM user_token ut
           LEFT JOIN user_account ua ON ua.id = ut.user_id
           WHERE ut.building_id = $1
             AND ut.token_type = $2
             AND ut.used_at IS NULL
             AND ut.expires_at > NOW()
           ORDER BY ut.expires_at DESC"#,
    )
    .bind(building_id)
    .bind(token_type::INVITATION)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let role: Option<String> = r.get("invited_role");
            let roles = role.map(|ro| vec![ro]).unwrap_or_default();
            let token_id: Uuid = r.get("id");
            serde_json::json!({
                "id": token_id,
                "email": r.get::<String, _>("email"),
                "first_name": r.get::<String, _>("first_name"),
                "last_name": r.get::<String, _>("last_name"),
                "phone": serde_json::Value::Null,
                "active": false,
                "roles": roles,
                "status": "pending_invite",
                "expires_at": r.get::<DateTime<Utc>, _>("expires_at"),
                "invite_token_id": token_id,
            })
        })
        .collect())
}

/// Revoke a specific invitation token (manager only).
pub async fn revoke_invite_by_id(
    pool: &PgPool,
    token_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<bool> {
    let result = sqlx::query(
        r#"UPDATE user_token SET used_at = NOW()
           WHERE id = $1 AND building_id = $2 AND token_type = $3 AND used_at IS NULL"#,
    )
    .bind(token_id)
    .bind(building_id)
    .bind(token_type::INVITATION)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Archive a user (revoke building access without deleting data).
pub async fn archive_user(
    pool: &PgPool,
    user_id: Uuid,
    building_id: Uuid,
    departure_date: NaiveDate,
) -> crate::errors::AppResult<bool> {
    let result = sqlx::query(
        r#"UPDATE user_building SET end_date = $3, updated_at = NOW()
           WHERE user_id = $1 AND building_id = $2 AND end_date IS NULL"#,
    )
    .bind(user_id)
    .bind(building_id)
    .bind(departure_date)
    .execute(pool)
    .await?;

    // Also revoke all active non-refresh tokens for this user
    sqlx::query(
        r#"UPDATE user_token SET used_at = NOW()
           WHERE user_id = $1 AND used_at IS NULL AND token_type != $2"#,
    )
    .bind(user_id)
    .bind(token_type::REFRESH)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Reactivate an archived user (restore building access).
pub async fn reactivate_user(
    pool: &PgPool,
    user_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<bool> {
    let result = sqlx::query(
        r#"UPDATE user_building SET end_date = NULL, updated_at = NOW()
           WHERE user_id = $1 AND building_id = $2 AND end_date IS NOT NULL"#,
    )
    .bind(user_id)
    .bind(building_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Update a resident's profile fields (name, email, phone).
pub async fn update_resident(
    pool: &PgPool,
    user_id: Uuid,
    first_name: &str,
    last_name: &str,
    email: Option<&str>,
    phone: Option<&str>,
) -> crate::errors::AppResult<()> {
    sqlx::query(
        r#"UPDATE user_account
           SET first_name = $2, last_name = $3, email = $4, phone = $5, updated_at = NOW()
           WHERE id = $1"#,
    )
    .bind(user_id)
    .bind(first_name)
    .bind(last_name)
    .bind(email)
    .bind(phone)
    .execute(pool)
    .await?;
    Ok(())
}

/// Anonymize a user account (RGPD art. 17).
/// Replaces PII with `ANONYMISÉ-{uuid}`, deactivates account, revokes all tokens.
/// Does NOT delete levy_call, vote, resolution or audit_log data.
pub async fn anonymize_user(
    pool: &PgPool,
    user_id: Uuid,
    anon_value: &str,
) -> crate::errors::AppResult<()> {
    let mut tx = pool.begin().await?;

    // Replace PII in user_account
    sqlx::query(
        r#"UPDATE user_account
           SET email = $2, last_name = $2, first_name = $2, phone = NULL,
               password_hash = NULL, passkey_credential = NULL, active = FALSE,
               updated_at = NOW()
           WHERE id = $1"#,
    )
    .bind(user_id)
    .bind(anon_value)
    .execute(&mut *tx)
    .await?;

    // Revoke all tokens
    sqlx::query("UPDATE user_token SET used_at = NOW() WHERE user_id = $1 AND used_at IS NULL")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    // Close all unit_owner records and mark anonymized
    sqlx::query(
        r#"UPDATE unit_owner
           SET anonymized_at = NOW(), anonymized_name = $2, user_id = NULL
           WHERE user_id = $1"#,
    )
    .bind(user_id)
    .bind(anon_value)
    .execute(&mut *tx)
    .await?;

    // Revoke building access
    sqlx::query(
        r#"UPDATE user_building SET end_date = CURRENT_DATE, updated_at = NOW()
           WHERE user_id = $1 AND end_date IS NULL"#,
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

/// Create a resident account without a password (no_access state, US-GP-008).
/// The account can later be activated by accepting an invitation.
pub async fn create_resident(
    pool: &PgPool,
    first_name: &str,
    last_name: &str,
    email: Option<&str>,
) -> crate::errors::AppResult<Uuid> {
    let id: Uuid = sqlx::query_scalar(
        r#"INSERT INTO user_account (first_name, last_name, email)
           VALUES ($1, $2, $3)
           RETURNING id"#,
    )
    .bind(first_name)
    .bind(last_name)
    .bind(email)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

/// Update the email of a no_access resident.
pub async fn update_user_email(
    pool: &PgPool,
    user_id: Uuid,
    email: &str,
) -> crate::errors::AppResult<bool> {
    let result =
        sqlx::query("UPDATE user_account SET email = $2, updated_at = NOW() WHERE id = $1")
            .bind(user_id)
            .bind(email)
            .execute(pool)
            .await?;
    Ok(result.rows_affected() > 0)
}

/// Create an invitation token linked to an existing user_id (deferred invite flow, US-GP-008).
pub async fn create_invite_for_existing_user(
    pool: &PgPool,
    user_id: Uuid,
    email: &str,
    token_hash: &str,
    building_id: Uuid,
    invited_role: &str,
    unit_id: Option<Uuid>,
    expires_at: DateTime<Utc>,
) -> crate::errors::AppResult<Uuid> {
    let id: Uuid = sqlx::query_scalar(
        r#"INSERT INTO user_token
               (user_id, email, token_type, token_hash, building_id, invited_role, unit_id, expires_at)
           VALUES ($1, $2, $3, $4, $5, $6::user_role, $7, $8)
           RETURNING id"#,
    )
    .bind(user_id)
    .bind(email)
    .bind(token_type::INVITATION)
    .bind(token_hash)
    .bind(building_id)
    .bind(invited_role)
    .bind(unit_id)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

/// Revoke all pending invitation tokens for a specific user (used before re-invite).
pub async fn revoke_pending_invites_by_user_id(
    pool: &PgPool,
    user_id: Uuid,
    building_id: Uuid,
) -> crate::errors::AppResult<()> {
    sqlx::query(
        r#"UPDATE user_token SET used_at = NOW()
           WHERE user_id = $1 AND building_id = $2 AND token_type = $3 AND used_at IS NULL"#,
    )
    .bind(user_id)
    .bind(building_id)
    .bind(token_type::INVITATION)
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
    let row = sqlx::query(
        r#"UPDATE user_token SET used_at = NOW()
           WHERE token_hash = $1 AND token_type = $2 AND used_at IS NULL AND expires_at > NOW()
           RETURNING user_id, building_id, email"#,
    )
    .bind(old_hash)
    .bind(token_type::REFRESH)
    .fetch_optional(pool)
    .await?;

    if let Some(ref r) = row {
        sqlx::query(
            r#"INSERT INTO user_token (user_id, email, token_type, token_hash, building_id, expires_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
        )
        .bind(r.get::<Uuid, _>("user_id"))
        .bind(r.get::<String, _>("email"))
        .bind(token_type::REFRESH)
        .bind(new_hash)
        .bind(r.get::<Option<Uuid>, _>("building_id"))
        .bind(expires_at)
        .execute(pool)
        .await?;
    }

    Ok(row.map(|r| {
        serde_json::json!({
            "user_id": r.get::<Uuid, _>("user_id"),
            "building_id": r.get::<Option<Uuid>, _>("building_id"),
            "email": r.get::<String, _>("email"),
        })
    }))
}
