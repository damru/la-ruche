// ── Handler stubs ─────────────────────────────────────────────────
// Each module follows the same pattern as api/assemblies.rs.
// These stubs enable compilation — to be implemented iteratively.

use crate::errors::AppResult;
use axum::http::StatusCode;

macro_rules! stub {
    ($name:ident) => {
        pub async fn $name() -> AppResult<StatusCode> {
            Ok(StatusCode::NOT_IMPLEMENTED)
        }
    };
}

// ── auth ─────────────────────────────────────────────────────────
pub mod auth {
    use super::*;
    stub!(register);
    stub!(login);
    stub!(logout);
    stub!(refresh);
    stub!(passkey_register_start);
    stub!(passkey_register_finish);
    stub!(passkey_login_start);
    stub!(passkey_login_finish);
    stub!(reset_password_request);
    stub!(reset_password_confirm);
    stub!(invite_accept);
}

// ── copropriete ───────────────────────────────────────────────────
pub mod building {
    use super::*;
    stub!(get);
    stub!(update);
    stub!(stats);
    stub!(list_keys);
    stub!(create_key);
    stub!(update_key);
    stub!(delete_key);
}

// ── lots ──────────────────────────────────────────────────────────
pub mod units {
    use super::*;
    stub!(list);
    stub!(create);
    stub!(get);
    stub!(update);
    stub!(archive);
    stub!(owners);
    stub!(transfer);
}

// ── users ───────────────────────────────────────────────────────────
pub mod users {
    use super::*;
    stub!(list);
    stub!(invite);
    stub!(get);
    stub!(deactivate);
    stub!(update_roles);
}

// ── budget ────────────────────────────────────────────────────────
pub mod budget {
    use super::*;
    stub!(list);
    stub!(create);
    stub!(get);
    stub!(update);
    stub!(list_lines);
    stub!(create_line);
    stub!(update_line);
    stub!(delete_line);
    stub!(generate_levies);
    stub!(list_levies);
    stub!(mark_paid);
    stub!(list_works_fund);
    stub!(get_works_fund);
}

// ── documents ─────────────────────────────────────────────────────
pub mod documents {
    use super::*;
    stub!(list);
    stub!(confirm_upload);
    stub!(get_upload_url);
    stub!(get_download_url);
    stub!(delete);
}

// ── contracts ────────────────────────────────────────────────────
pub mod contracts {
    use super::*;
    stub!(list);
    stub!(create);
    stub!(get);
    stub!(update);
    stub!(terminate);
    stub!(list_reminders);
    stub!(acknowledge_reminder);
}

// ── tasks ─────────────────────────────────────────────────────────
pub mod tasks {
    use super::*;
    stub!(list);
    stub!(create);
    stub!(get);
    stub!(update);
    stub!(generate_token);
    stub!(revoke_token);
}

// ── claims ───────────────────────────────────────────────────────────
pub mod claims {
    use super::*;
    stub!(list);
    stub!(create);
    stub!(get);
    stub!(update);
}

// ── logbook ──────────────────────────────────────────────────────────
// (wired in api/logbook.rs)

// ── reports ──────────────────────────────────────────────────────────
pub mod reports {
    use super::*;
    stub!(list);
    stub!(create);
    stub!(get);
    stub!(update_status);
}

// ── consultations ─────────────────────────────────────────────────
// (wired in api/consultations.rs)

// ── contractor (tokenized public access) ─────────────────────────
// (wired in api/contractor.rs)
