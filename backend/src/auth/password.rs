use argon2::{
    Algorithm, Argon2, Params, Version,
    password_hash::{PasswordHasher, PasswordVerifier, generate_salt},
};

use crate::{
    config::AuthSettings,
    errors::{AppError, AppResult},
};

pub fn hash_password(password: &str, cfg: &AuthSettings) -> AppResult<String> {
    // argon2 0.6: salt is raw bytes (SaltString was removed).
    let salt = generate_salt();
    let params = Params::new(
        cfg.argon2_memory_kb,
        cfg.argon2_iterations,
        1, // parallelism
        None,
    )
    .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid Argon2 params")))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    argon2
        .hash_password_with_salt(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Password hashing failed")))
}

pub fn verify_password(password: &str, hash: &str) -> AppResult<()> {
    // argon2 0.6: PasswordVerifier<str> parses the PHC string itself.
    Argon2::default()
        .verify_password(password.as_bytes(), hash)
        .map_err(|_| AppError::Unauthorized)
}
