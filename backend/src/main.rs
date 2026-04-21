// AGPL-3.0 — La Ruche

use std::{env, path::Path, sync::Arc};

use laruche::{
    api, auth,
    config::{AppConfig, config_debug_enabled},
    db::AppState,
    jobs,
    storage::client::StorageClient,
};

use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tower_http::{
    cors::CorsLayer, sensitive_headers::SetSensitiveHeadersLayer, timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

fn load_dotenv() {
    let candidates = [".env", "../.env", "../../.env", "../../../.env"];
    let debug = config_debug_enabled();

    if debug {
        let cwd = env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "<unknown>".to_owned());
        eprintln!("[config-debug] CWD: {cwd}");
    }

    for candidate in candidates {
        let exists = Path::new(candidate).exists();
        if debug {
            eprintln!("[config-debug] Trying dotenv file: {candidate} (exists={exists})");
        }

        match dotenvy::from_filename(candidate) {
            Ok(path) => {
                if debug {
                    eprintln!("[config-debug] Loaded dotenv from: {}", path.display());
                }
                return;
            }
            Err(err) => {
                if debug {
                    eprintln!("[config-debug] Failed loading {candidate}: {err}");
                }
            }
        }
    }

    if debug {
        eprintln!("[config-debug] No dotenv file loaded. Using process environment only.");
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ── Config ──────────────────────────────────────────────────
    load_dotenv();
    let cfg = AppConfig::load()?;

    // ── Logs ────────────────────────────────────────────────────
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "laruche=info".into()))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    info!(version = env!("CARGO_PKG_VERSION"), "Starting La Ruche");

    // ── Base de données ─────────────────────────────────────────
    let pool = PgPoolOptions::new()
        .max_connections(cfg.db.max_connections)
        .connect(&cfg.db.url)
        .await?;

    // Run migrations automatically on startup
    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("Migrations applied");

    // ── Stockage S3 ─────────────────────────────────────────────
    let storage = StorageClient::new(&cfg.s3).await?;

    // ── State partagé ───────────────────────────────────────────
    let state = Arc::new(AppState {
        pool,
        storage,
        cfg: cfg.clone(),
    });

    // ── Auth (WebAuthn) ─────────────────────────────────────────
    let webauthn = auth::passkey::build_webauthn(&cfg.app)?;

    // ── Router ──────────────────────────────────────────────────
    let app = Router::new()
        .merge(api::router(Arc::clone(&state), webauthn))
        .layer(SetSensitiveHeadersLayer::new([axum::http::header::AUTHORIZATION]))
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(axum::http::StatusCode::REQUEST_TIMEOUT, std::time::Duration::from_secs(30)))
        .layer(CorsLayer::new()) // configured from cfg in production
        ;

    // ── Jobs de fond ────────────────────────────────────────────
    let state_jobs = Arc::clone(&state);
    tokio::spawn(async move {
        jobs::reminders::run(state_jobs).await;
    });

    // ── Serveur ─────────────────────────────────────────────────
    let addr = cfg.app.listen_addr()?;
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(addr = %addr, "Server started");

    axum::serve(listener, app).await?;
    Ok(())
}
