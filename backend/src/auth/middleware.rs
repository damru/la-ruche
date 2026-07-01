use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use std::sync::Arc;

use crate::{auth::jwt::validate_access_token, db::AppState, domain::AuthUser, errors::AppError};

/// Axum extractor — injects AuthUser into all protected handlers.
/// Reads JWT from the `access_token` cookie (httpOnly).
/// Falls back to the Authorization: Bearer <token> header for API clients.
impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_token(parts).ok_or(AppError::Unauthorized)?;
        let claims = validate_access_token(&token, &state.cfg.auth.jwt_secret)?;

        Ok(AuthUser {
            user_id: claims.sub,
            building_id: claims.building_id,
            roles: claims.roles,
        })
    }
}

fn extract_token(parts: &Parts) -> Option<String> {
    // 1. Cookie httpOnly (prioritaire) — iterate all Cookie headers because
    //    HTTP allows multiple Cookie headers and some clients (e.g. axum-test)
    //    send each cookie as a separate header instead of combining them.
    for cookie_header in parts.headers.get_all(header::COOKIE) {
        if let Ok(cookies) = cookie_header.to_str() {
            for cookie in cookies.split(';') {
                let cookie = cookie.trim();
                if let Some(val) = cookie.strip_prefix("access_token=") {
                    return Some(val.to_string());
                }
            }
        }
    }

    // 2. Fallback Authorization: Bearer (clients API)
    if let Some(auth_header) = parts.headers.get(header::AUTHORIZATION) {
        let val = auth_header.to_str().ok()?;
        if let Some(token) = val.strip_prefix("Bearer ") {
            return Some(token.to_string());
        }
    }

    None
}

/// Optional extractor — for mixed routes (auth or public)
pub struct MaybeAuthUser(pub Option<AuthUser>);

impl FromRequestParts<Arc<AppState>> for MaybeAuthUser {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let user = AuthUser::from_request_parts(parts, state).await.ok();
        Ok(MaybeAuthUser(user))
    }
}

/// Extractor for vendor routes (URL token, no JWT)
pub struct VendorToken(pub String);

impl<S: Send + Sync> FromRequestParts<S> for VendorToken {
    type Rejection = AppError;

    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // The token is in the path param — retrieved in the handler directly
        // This extractor only checks for custom header presence if needed in the future
        Err(AppError::Unauthorized) // placeholder
    }
}
