use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Params, Algorithm, Version,
};

use crate::{config::AuthSettings, errors::{AppError, AppResult}};

pub fn hash_password(password: &str, cfg: &AuthSettings) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let params = Params::new(
        cfg.argon2_memory_kb,
        cfg.argon2_iterations,
        1, // parallelism
        None,
    )
    .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid Argon2 params")))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Password hashing failed")))
}

pub fn verify_password(password: &str, hash: &str) -> AppResult<()> {
    let parsed = PasswordHash::new(hash)
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid hash")))?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| AppError::Unauthorized)
}
