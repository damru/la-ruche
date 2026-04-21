use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    domain::Role,
    errors::{AppError, AppResult},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // user_id
    pub building_id: Uuid,
    pub roles: Vec<Role>,
    pub exp: i64, // expiration UNIX timestamp
    pub iat: i64, // issued at
}

pub fn generate_access_token(
    user_id: Uuid,
    building_id: Uuid,
    roles: Vec<Role>,
    secret: &str,
    ttl_secs: u64,
) -> AppResult<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id,
        building_id,
        roles,
        iat: now.timestamp(),
        exp: (now + Duration::seconds(ttl_secs as i64)).timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| AppError::Internal(anyhow::anyhow!("JWT generation error")))
}

pub fn validate_access_token(token: &str, secret: &str) -> AppResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenInvalid,
        _ => AppError::TokenInvalid,
    })
}
