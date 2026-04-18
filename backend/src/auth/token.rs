use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Generates a URL-safe token and returns (token_plain, token_hash, expire_at).
/// The plain token is sent by email/URL.
/// The SHA-256 hash is stored in DB.
pub fn generate_token(ttl: Duration) -> (String, String, chrono::DateTime<Utc>) {
    let token_plain = Uuid::new_v4().to_string().replace('-', "")
        + &Uuid::new_v4().to_string().replace('-', ""); // 64 chars hex
    let token_hash = hash_token(&token_plain);
    let expire_at = Utc::now() + ttl;
    (token_plain, token_hash, expire_at)
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Generates a UUID v4 for the vendor token (stored as-is in DB,
/// not hashed because it's routing data, not an authentication secret).
/// The token is in the URL, TTL controls validity.
pub fn generate_vendor_token() -> Uuid {
    Uuid::new_v4()
}
