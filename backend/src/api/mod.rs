pub mod assemblies;
pub mod auth;
pub mod budget;
pub mod building;
pub mod contracts;
pub mod documents;
pub mod contractor;
pub mod onboarding;
mod stubs;
pub mod tasks;

// Modules still implemented as stubs — re-exported from stubs.rs
use stubs::{logbook, consultations, units, reports, claims, users};

use std::sync::Arc;

use axum::{Router, routing::{delete, get, post, put}};
use utoipa::OpenApi;
use webauthn_rs::Webauthn;

use crate::db::AppState;

type State = axum::extract::State<Arc<AppState>>;

const NOTICE_TEXT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../NOTICE"));

pub fn router(state: Arc<AppState>, _webauthn: Webauthn) -> Router {
    let api = Router::new()
        // ── Auth ────────────────────────────────────────────────
        .route("/auth/register",                    post(auth::register))
        .route("/auth/login",                       post(auth::login))
        .route("/auth/logout",                      post(auth::logout))
        .route("/auth/refresh",                     post(auth::refresh))
        .route("/auth/me",                          get(auth::me))
        .route("/auth/passkey/register/start",      post(auth::passkey_register_start))
        .route("/auth/passkey/register/finish",     post(auth::passkey_register_finish))
        .route("/auth/passkey/login/start",         post(auth::passkey_login_start))
        .route("/auth/passkey/login/finish",        post(auth::passkey_login_finish))
        .route("/auth/reset-password/request",      post(auth::reset_password_request))
        .route("/auth/reset-password/confirm",      post(auth::reset_password_confirm))
        .route("/auth/invite/accept",               post(auth::invite_accept))

        // ── Building ────────────────────────────────────────────
        .route("/building",                         get(building::get).put(building::update))
        .route("/building/stats",                   get(building::stats))
        .route("/building/keys",                    get(building::list_keys).post(building::create_key))
        .route("/building/keys/{id}",                put(building::update_key).delete(building::delete_key))

        // ── Units ───────────────────────────────────────────────
        .route("/units",                            get(units::list).post(units::create))
        .route("/units/{id}",                        get(units::get).put(units::update).delete(units::archive))
        .route("/units/{id}/owners",                 get(units::owners))
        .route("/units/{id}/transfer",               post(units::transfer))

        // ── Users ───────────────────────────────────────────────
        .route("/users",                            get(users::list))
        .route("/users/invite",                     post(users::invite))
        .route("/users/{id}",                        get(users::get).delete(users::deactivate))
        .route("/users/{id}/roles",                  put(users::update_roles))

        // ── Meetings ────────────────────────────────────────────
        .route("/meetings",                         get(assemblies::list).post(assemblies::create))
        .route("/meetings/{id}",                     get(assemblies::get).put(assemblies::update))
        .route("/meetings/{id}/call",                post(assemblies::call))
        .route("/meetings/{id}/close",               post(assemblies::close))
        .route("/meetings/{id}/resolutions",         get(assemblies::list_resolutions).post(assemblies::create_resolution))
        .route("/meetings/{id}/resolutions/{rid}",    put(assemblies::update_resolution))
        .route("/meetings/{id}/resolutions/{rid}/vote",    post(assemblies::vote))
        .route("/meetings/{id}/resolutions/{rid}/results", get(assemblies::results))

        // ── Budget ──────────────────────────────────────────────
        .route("/budgets",                          get(budget::list).post(budget::create))
        .route("/budgets/{id}",                      get(budget::get).put(budget::update))
        .route("/budgets/{id}/lines",                get(budget::list_lines).post(budget::create_line))
        .route("/budgets/{id}/lines/{lid}",           put(budget::update_line).delete(budget::delete_line))
        .route("/budgets/{id}/levies/generate",      post(budget::generate_levies))
        .route("/budgets/{id}/levies",               get(budget::list_levies))
        .route("/levies/{id}/payment",               put(budget::mark_paid))
        .route("/works-fund",                       get(budget::list_works_fund))
        .route("/works-fund/{fiscal_year}",          get(budget::get_works_fund))

        // ── Documents ───────────────────────────────────────────
        .route("/documents",                        get(documents::list).post(documents::confirm_upload))
        .route("/documents/upload-url",             post(documents::get_upload_url))
        .route("/documents/{id}/download-url",       get(documents::get_download_url))
        .route("/documents/{id}",                    delete(documents::delete))

        // ── Contracts ───────────────────────────────────────────
        .route("/contracts",                        get(contracts::list).post(contracts::create))
        .route("/contracts/{id}",                    get(contracts::get).put(contracts::update).delete(contracts::terminate))
        .route("/contracts/{id}/reminders",          get(contracts::list_reminders))
        .route("/contracts/{id}/reminders/{rid}/acknowledge", put(contracts::acknowledge_reminder))

        // ── Tasks ───────────────────────────────────────────────
        .route("/tasks",                            get(tasks::list).post(tasks::create))
        .route("/tasks/{id}",                        get(tasks::get).put(tasks::update))
        .route("/tasks/{id}/vendor-token",           post(tasks::generate_token).delete(tasks::revoke_token))

        // ── Claims ──────────────────────────────────────────────
        .route("/claims",                           get(claims::list).post(claims::create))
        .route("/claims/{id}",                       get(claims::get).put(claims::update))

        // ── Maintenance log ─────────────────────────────────────
        .route("/logbook",                          get(logbook::list).post(logbook::create))
        .route("/logbook/{id}",                      get(logbook::get).put(logbook::update).delete(logbook::delete))

        // ── Reports ─────────────────────────────────────────────
        .route("/reports",                          get(reports::list).post(reports::create))
        .route("/reports/{id}",                      get(reports::get))
        .route("/reports/{id}/status",               put(reports::update_status))

        // ── Consultations ────────────────────────────────────────
        .route("/consultations",                    get(consultations::list).post(consultations::create))
        .route("/consultations/{id}",                get(consultations::get).put(consultations::update))
        .route("/consultations/{id}/respond",        post(consultations::respond))
        .route("/consultations/{id}/results",        get(consultations::results))
        .route("/consultations/{id}/close",          post(consultations::close))
        ;

    // ── Public routes (vendor token, no JWT) ─────────────────────
    let public = Router::new()
        .route("/vendor/{token}",                    get(contractor::get_task))
        .route("/vendor/{token}/submission",         post(contractor::submit))
        // ── Onboarding first-run (no JWT) ─────────────────────────
        .route("/api/v1/onboarding/status",         get(onboarding::status))
        .route("/api/v1/onboarding/bootstrap",      post(onboarding::bootstrap));

    Router::new()
        .nest("/api/v1", api)
        .merge(public)
        // ── AGPL notice (mandatory in UI) ──────────────────────
        .route("/notice", get(notice))
        .with_state(state)
}

async fn notice() -> &'static str {
    NOTICE_TEXT
}

// ── OpenAPI ─────────────────────────────────────────────────────

#[derive(OpenApi)]
#[openapi(
    info(
        title = "La Ruche API",
        version = "0.1.0",
        license(name = "AGPL-3.0", url = "https://www.gnu.org/licenses/agpl-3.0.html"),
    ),
    tags(
        (name = "auth", description = "Authentication"),
        (name = "building", description = "Building management"),
        (name = "units", description = "Units and owners"),
        (name = "assemblies", description = "General assemblies"),
        (name = "budget", description = "Budget and levy calls"),
        (name = "documents", description = "Document management"),
        (name = "contracts", description = "Vendor contracts"),
        (name = "tasks", description = "Task tracking"),
        (name = "claims", description = "Claims"),
        (name = "logbook", description = "Maintenance logbook"),
        (name = "reports", description = "Reports"),
        (name = "consultations", description = "Consultations / polls"),
        (name = "contractor", description = "Contractor access (token)"),
    )
)]
pub struct ApiDoc;
