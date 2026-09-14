pub mod admin;
pub mod assemblies;
pub mod auth;
pub mod budget;
pub mod building;
pub mod claims;
pub mod consultations;
pub mod contractor;
pub mod contractors;
pub mod contracts;
pub mod documents;
pub mod etat_date;
pub mod governance;
pub mod judicial_proceedings;
pub mod logbook;
pub mod onboarding;
pub mod reports;
pub mod runtime;
mod stubs;
pub mod tasks;
pub mod units;
pub mod users;

// Modules still implemented as stubs — re-exported from stubs.rs
use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, post, put},
};
use utoipa::OpenApi;
use webauthn_rs::Webauthn;

use crate::db::AppState;

type State = axum::extract::State<Arc<AppState>>;

const NOTICE_TEXT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../NOTICE"));

pub fn router(state: Arc<AppState>, _webauthn: Webauthn) -> Router {
    let api = Router::new()
        // ── Admin ────────────────────────────────────────────────
        .route("/admin/reset-demo", post(admin::reset_demo))
        .route("/admin/reset-data", post(admin::reset_data))
        // ── Auth ────────────────────────────────────────────────
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        .route("/auth/refresh", post(auth::refresh))
        .route("/auth/me", get(auth::me).patch(auth::update_me))
        .route(
            "/auth/passkey/register/start",
            post(auth::passkey_register_start),
        )
        .route(
            "/auth/passkey/register/finish",
            post(auth::passkey_register_finish),
        )
        .route("/auth/passkey/login/start", post(auth::passkey_login_start))
        .route(
            "/auth/passkey/login/finish",
            post(auth::passkey_login_finish),
        )
        .route(
            "/auth/reset-password/request",
            post(auth::reset_password_request),
        )
        .route(
            "/auth/reset-password/confirm",
            post(auth::reset_password_confirm),
        )
        .route("/auth/invite/accept", post(auth::invite_accept))
        .route("/auth/invite/validate", get(auth::validate_invite))
        // ── Building ────────────────────────────────────────────
        .route("/building", get(building::get).put(building::update))
        .route("/building/stats", get(building::stats))
        .route("/building/registry", get(building::registry))
        .route(
            "/building/keys",
            get(building::list_keys).post(building::create_key),
        )
        .route(
            "/building/keys/{id}",
            put(building::update_key).delete(building::delete_key),
        )
        .route("/building/keys/{id}/balance", get(building::key_balance))
        .route("/building/keys/{id}/audit", get(building::key_audit))
        .route(
            "/building/keys/{id}/shares",
            get(building::list_key_shares).put(building::set_key_shares),
        )
        .route(
            "/building/edd-references",
            get(building::list_edd_references).post(building::create_edd_reference),
        )
        .route("/building/edd-check", get(building::edd_check))
        // ── Units ───────────────────────────────────────────────
        .route("/units", get(units::list).post(units::create))
        .route(
            "/units/{id}",
            get(units::get).put(units::update).delete(units::archive),
        )
        .route("/units/{id}/owners", get(units::owners))
        .route("/units/{id}/transfer", post(units::transfer))
        .route("/units/{id}/charges", get(units::charges))
        .route("/units/{id}/etat-date-requests", post(etat_date::create))
        // ── État daté ───────────────────────────────────────────
        .route("/etat-date-requests", get(etat_date::list))
        .route("/etat-date-requests/{id}", get(etat_date::get))
        .route(
            "/etat-date-requests/{id}/generate",
            post(etat_date::generate),
        )
        // ── Users ───────────────────────────────────────────────
        .route("/users", get(users::list).post(users::create))
        .route("/users/invite", post(users::invite))
        .route(
            "/users/{id}",
            get(users::get)
                .put(users::update_resident)
                .delete(users::deactivate),
        )
        .route("/users/{id}/roles", put(users::update_roles))
        .route("/users/{id}/anonymize", post(users::anonymize))
        .route("/users/{id}/invite", post(users::send_invite))
        .route("/users/{id}/reactivate", post(users::reactivate))
        .route("/users/invites/{id}", delete(users::revoke_invite))
        // ── Meetings ────────────────────────────────────────────
        .route("/meetings", get(assemblies::list).post(assemblies::create))
        .route(
            "/meetings/{id}",
            get(assemblies::get).put(assemblies::update),
        )
        .route("/meetings/{id}/call", post(assemblies::call))
        .route(
            "/meetings/{id}/convocation-checklist",
            get(assemblies::get_convocation_checklist),
        )
        .route("/meetings/{id}/start", post(assemblies::start))
        .route("/meetings/{id}/close", post(assemblies::close))
        .route(
            "/meetings/{id}/documents",
            get(assemblies::list_documents).post(assemblies::attach_document),
        )
        .route(
            "/meetings/{id}/documents/{document_id}",
            delete(assemblies::detach_document),
        )
        .route(
            "/meetings/{id}/attendance",
            get(assemblies::list_attendance).put(assemblies::save_attendance),
        )
        .route(
            "/meetings/{id}/resolutions",
            get(assemblies::list_resolutions).post(assemblies::create_resolution),
        )
        .route(
            "/meetings/{id}/resolutions/{rid}",
            put(assemblies::update_resolution).patch(assemblies::patch_resolution),
        )
        .route(
            "/meetings/{id}/resolutions/{rid}/vote",
            post(assemblies::vote),
        )
        .route(
            "/meetings/{id}/resolutions/{rid}/votes",
            get(assemblies::list_votes),
        )
        .route(
            "/meetings/{id}/resolutions/{rid}/results",
            get(assemblies::results),
        )
        .route(
            "/meetings/{id}/resolutions/{rid}/finalize",
            post(assemblies::finalize_resolution),
        )
        .route(
            "/meetings/{id}/minutes/draft",
            get(assemblies::get_minutes_draft),
        )
        .route("/meetings/{id}/minutes", post(assemblies::attach_minutes))
        .route(
            "/meetings/{id}/minutes/signatures",
            post(assemblies::sign_minutes),
        )
        .route(
            "/meetings/{id}/notify-minutes",
            post(assemblies::notify_minutes),
        )
        // ── Budget ──────────────────────────────────────────────
        .route("/budgets", get(budget::list).post(budget::create))
        .route("/budgets/{id}", get(budget::get).put(budget::update))
        .route("/budgets/{id}/close", post(budget::close))
        .route("/budgets/{id}/approve", post(budget::approve))
        .route(
            "/budgets/{id}/rectificatif",
            post(budget::create_rectificatif),
        )
        .route(
            "/budgets/{id}/rectificatifs",
            get(budget::rectificatif_chain),
        )
        .route(
            "/budgets/{id}/lines",
            get(budget::list_lines).post(budget::create_line),
        )
        .route(
            "/budgets/{id}/lines/{lid}",
            put(budget::update_line).delete(budget::delete_line),
        )
        .route(
            "/budgets/{id}/levies/generate",
            post(budget::generate_levies),
        )
        .route("/budgets/{id}/levies", get(budget::list_levies))
        .route("/levy-calls/{id}/issue", post(budget::issue_levy_call))
        .route("/levy-calls/{id}/credit-note", post(budget::credit_note))
        .route(
            "/levies/{levy_call_id}/units/{unit_id}/payment",
            put(budget::mark_paid),
        )
        .route("/works-fund", get(budget::list_works_fund))
        .route(
            "/works-fund/{fiscal_year}",
            get(budget::get_works_fund).put(budget::update_works_fund),
        )
        .route("/accounting-codes", get(budget::list_accounting_codes))
        // ── Documents ───────────────────────────────────────────
        .route(
            "/documents",
            get(documents::list).post(documents::confirm_upload),
        )
        .route("/documents/upload-url", post(documents::get_upload_url))
        .route(
            "/documents/{id}/download-url",
            get(documents::get_download_url),
        )
        .route("/documents/{id}", delete(documents::delete))
        .route("/documents/{id}/versions", get(documents::versions))
        // ── Contracts ───────────────────────────────────────────
        .route("/contracts", get(contracts::list).post(contracts::create))
        .route("/contracts/reminders", get(contracts::list_all_reminders))
        .route(
            "/contracts/{id}",
            get(contracts::get)
                .put(contracts::update)
                .delete(contracts::delete),
        )
        .route("/contracts/{id}/terminate", put(contracts::terminate))
        .route("/contracts/{id}/activate", post(contracts::activate))
        .route(
            "/contracts/{id}/amendments",
            post(contracts::create_amendment).get(contracts::amendment_chain),
        )
        .route("/contracts/{id}/reminders", get(contracts::list_reminders))
        .route(
            "/contracts/{id}/reminders/{rid}/acknowledge",
            put(contracts::acknowledge_reminder),
        )
        .route(
            "/contracts/{id}/documents",
            get(contracts::list_documents).post(contracts::attach_document),
        )
        .route(
            "/contracts/{id}/documents/{document_id}",
            delete(contracts::detach_document),
        )
        // ── Tasks ───────────────────────────────────────────────
        .route("/tasks", get(tasks::list).post(tasks::create))
        .route(
            "/tasks/{id}",
            get(tasks::get)
                .put(tasks::update)
                .delete(tasks::delete_task),
        )
        .route(
            "/tasks/{id}/documents",
            get(tasks::list_documents).post(tasks::attach_document),
        )
        .route(
            "/tasks/{id}/documents/{document_id}",
            delete(tasks::detach_document),
        )
        .route(
            "/tasks/{id}/vendor-token",
            get(tasks::get_vendor_token_url)
                .post(tasks::generate_token)
                .delete(tasks::revoke_token),
        )
        // ── Contractors referential ─────────────────────────────
        .route(
            "/contractors",
            get(contractors::list).post(contractors::create),
        )
        .route(
            "/contractors/{id}",
            get(contractors::get)
                .put(contractors::update)
                .delete(contractors::delete),
        )
        // ── Claims ──────────────────────────────────────────────
        .route("/claims", get(claims::list).post(claims::create))
        .route(
            "/claims/{id}",
            get(claims::get).put(claims::update).delete(claims::delete),
        )
        .route(
            "/claims/{id}/documents",
            get(claims::list_documents).post(claims::attach_document),
        )
        .route(
            "/claims/{id}/documents/{doc_id}",
            delete(claims::detach_document),
        )
        // ── Maintenance log ─────────────────────────────────────
        .route("/logbook", get(logbook::list).post(logbook::create))
        .route("/logbook/summary", get(logbook::summary))
        .route(
            "/logbook/{id}",
            get(logbook::get)
                .put(logbook::update)
                .delete(logbook::delete),
        )
        .route("/logbook/{id}/correction", post(logbook::correction))
        // ── Reports ─────────────────────────────────────────────
        .route("/reports", get(reports::list).post(reports::create))
        .route(
            "/reports/{id}",
            get(reports::get)
                .put(reports::update)
                .delete(reports::delete),
        )
        .route("/reports/{id}/status", put(reports::update_status))
        .route("/reports/{id}/task", post(reports::convert_to_task))
        .route(
            "/reports/{id}/documents",
            get(reports::list_documents).post(reports::attach_document),
        )
        .route(
            "/reports/{id}/documents/{doc_id}",
            delete(reports::detach_document),
        )
        // ── Consultations ────────────────────────────────────────
        .route(
            "/consultations",
            get(consultations::list).post(consultations::create),
        )
        .route(
            "/consultations/{id}",
            get(consultations::get)
                .put(consultations::update)
                .delete(consultations::delete),
        )
        .route("/consultations/{id}/respond", post(consultations::respond))
        .route("/consultations/{id}/results", get(consultations::results))
        .route("/consultations/{id}/close", post(consultations::close))
        // ── Judicial proceedings register ───────────────────────
        .route(
            "/judicial-proceedings",
            get(judicial_proceedings::list).post(judicial_proceedings::create),
        )
        .route(
            "/judicial-proceedings/{id}",
            get(judicial_proceedings::get)
                .put(judicial_proceedings::update)
                .delete(judicial_proceedings::delete),
        )
        // ── Governance register ─────────────────────────────────
        .route(
            "/governance/mandates",
            get(governance::list).post(governance::create),
        )
        .route(
            "/governance/mandates/{id}",
            get(governance::get)
                .put(governance::update)
                .delete(governance::delete),
        );

    // ── Public routes (vendor token, no JWT) ─────────────────────
    // Vendor routes live under /api/v1 (without auth middleware) so that the
    // SPA can serve /vendor/:token as a proper page and API calls go to
    // /api/v1/vendor/:token — avoiding the JSON-page issue.
    let public = Router::new()
        // ── Onboarding first-run (no JWT) ─────────────────────────
        .route("/api/v1/onboarding/status", get(onboarding::status))
        .route("/api/v1/onboarding/bootstrap", post(onboarding::bootstrap))
        .route("/api/v1/runtime", get(runtime::status))
        // ── Contractor access (tokenized, no JWT) ─────────────────
        .route("/api/v1/vendor/{token}", get(contractor::get_task))
        .route(
            "/api/v1/vendor/{token}/upload-url",
            post(contractor::upload_url),
        )
        .route(
            "/api/v1/vendor/{token}/submission",
            post(contractor::submit),
        );

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
