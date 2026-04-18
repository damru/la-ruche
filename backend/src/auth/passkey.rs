use url::Url;
use webauthn_rs::{Webauthn, WebauthnBuilder};

use crate::{config::AppSettings, errors::{AppError, AppResult}};

/// Builds the shared Webauthn instance (can be stored in AppState
/// or rebuilt on demand — it is stateless).
pub fn build_webauthn(cfg: &AppSettings) -> AppResult<Webauthn> {
    let rp_id = Url::parse(&cfg.public_url)
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid public_url")))?
        .host_str()
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("Cannot extract host from public_url")))?
        .to_string();

    let rp_origin = Url::parse(&cfg.public_url)
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Invalid public_url")))?;

    WebauthnBuilder::new(&rp_id, &rp_origin)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("WebAuthn init: {}", e)))?
        .rp_name(&cfg.name)
        .build()
        .map_err(|e| AppError::Internal(anyhow::anyhow!("WebAuthn build: {}", e)))
}

// ── Persistence types ────────────────────────────────────────
// The WebAuthn ceremony state (in-progress challenge) must be
// persisted between start and finish. We use a signed httpOnly
// cookie on the client side (serialised as JSON) — no server session.
//
// The final credential (PasskeyAuthentication) is stored as JSONB
// in user.passkey_credential.
