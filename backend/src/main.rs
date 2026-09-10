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

const DEMO_SQL: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/seeds/demo.sql"));

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

async fn seed_demo_database(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    sqlx::raw_sql(DEMO_SQL).execute(pool).await?;
    Ok(())
}

async fn database_is_empty(pool: &sqlx::PgPool) -> anyhow::Result<bool> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM user_account)")
        .fetch_one(pool)
        .await?;
    Ok(!exists)
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

    // ── Database ───────────────────────────────────────────
    let pool = PgPoolOptions::new()
        .max_connections(cfg.db.max_connections)
        .connect(&cfg.db.url)
        .await?;

    // Run migrations automatically on startup
    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("Migrations applied");

    if cfg.app.demo_mode {
        if database_is_empty(&pool).await? {
            seed_demo_database(&pool).await?;
            info!("Demo database initialized from demo.sql");
        } else {
            info!(
                "Demo mode on, but data already present — seed skipped (use POST /admin/reset-demo to re-seed)"
            );
        }
    }

    // ── Stockage S3 ─────────────────────────────────────────────
    let storage = StorageClient::new(&cfg.s3).await?;

    // ── Shared state ───────────────────────────────────────────
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

    let state_s3_cleanup = Arc::clone(&state);
    tokio::spawn(async move {
        jobs::s3_cleanup::run(state_s3_cleanup).await;
    });

    // ── Serveur ─────────────────────────────────────────────────
    let addr = cfg.app.listen_addr()?;
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!(addr = %addr, "Server started");

    axum::serve(listener, app).await?;
    Ok(())
}
