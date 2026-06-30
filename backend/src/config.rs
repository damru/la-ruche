use serde::Deserialize;
use std::env;
use std::net::SocketAddr;

const CONFIG_DEBUG_ENV: &str = "LA_RUCHE_CONFIG_DEBUG";

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub app: AppSettings,
    pub db: DbSettings,
    pub s3: S3Settings,
    pub auth: AuthSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppSettings {
    pub host: String,
    pub port: u16,
    /// Public URL of the application (used by WebAuthn)
    pub public_url: String,
    /// Display name shown in passkey prompts
    pub name: String,
    #[serde(default)]
    pub demo_mode: bool,
}

impl AppSettings {
    pub fn listen_addr(&self) -> anyhow::Result<SocketAddr> {
        format!("{}:{}", self.host, self.port)
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid listen address {}:{}: {e}", self.host, self.port))
    }
}

pub fn config_debug_enabled() -> bool {
    matches!(
        env::var(CONFIG_DEBUG_ENV),
        Ok(v) if matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on")
    )
}

fn format_env_value(key: &str, raw: Option<String>) -> String {
    let is_secret = key.contains("SECRET") || key.contains("ACCESS_KEY") || key.contains("JWT");
    match raw {
        Some(value) if is_secret => format!("<set:{} chars>", value.chars().count()),
        Some(value) => value,
        None => "<missing>".to_owned(),
    }
}

fn format_config_value(cfg: &config::Config, key: &str) -> String {
    let is_secret = key.contains("secret") || key.contains("access_key");

    if let Ok(v) = cfg.get_string(key) {
        if is_secret {
            return format!("<set:{} chars>", v.chars().count());
        }
        return v;
    }
    if let Ok(v) = cfg.get_int(key) {
        return v.to_string();
    }
    if let Ok(v) = cfg.get_float(key) {
        return v.to_string();
    }
    if let Ok(v) = cfg.get_bool(key) {
        return v.to_string();
    }
    "<missing>".to_owned()
}

fn print_config_diagnostics(cfg: &config::Config) {
    eprintln!("[config-debug] Environment snapshot (LA_RUCHE__*):");
    let env_keys = [
        "LA_RUCHE__APP__HOST",
        "LA_RUCHE__APP__PORT",
        "LA_RUCHE__APP__PUBLIC_URL",
        "LA_RUCHE__APP__NAME",
        "LA_RUCHE__APP__DEMO_MODE",
        "LA_RUCHE__DB__URL",
        "LA_RUCHE__DB__MAX_CONNECTIONS",
        "LA_RUCHE__S3__ENDPOINT_URL",
        "LA_RUCHE__S3__REGION",
        "LA_RUCHE__S3__ACCESS_KEY_ID",
        "LA_RUCHE__S3__SECRET_ACCESS_KEY",
        "LA_RUCHE__AUTH__JWT_SECRET",
    ];
    for key in env_keys {
        let value = format_env_value(key, env::var(key).ok());
        eprintln!("[config-debug]   {key}={value}");
    }

    eprintln!("[config-debug] Merged config snapshot:");
    let cfg_keys = [
        "app.host",
        "app.port",
        "app.public_url",
        "app.name",
        "app.demo_mode",
        "db.url",
        "db.max_connections",
        "s3.endpoint_url",
        "s3.region",
        "s3.access_key_id",
        "s3.secret_access_key",
        "auth.jwt_secret",
    ];
    for key in cfg_keys {
        eprintln!("[config-debug]   {key}={}", format_config_value(cfg, key));
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DbSettings {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct S3Settings {
    pub endpoint_url: String, // e.g. http://minio:9000 — internal Docker endpoint
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    /// Bucket prefix, suffixed by building id
    pub bucket_prefix: String,
    /// Staging bucket for vendor uploads before validation
    pub staging_bucket: String,
    /// Pre-signed URL TTL in seconds
    pub presigned_ttl_secs: u64,
    /// Public-facing base URL rewriting presigned URLs for browser use.
    /// Required when endpoint_url is an internal Docker hostname (e.g. http://minio:9000)
    /// and the browser must reach MinIO on a different address (e.g. http://localhost:9000).
    pub public_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthSettings {
    pub jwt_secret: String, // min 256 bits
    pub access_token_ttl_secs: u64,
    pub refresh_token_ttl_days: u64,
    pub invitation_ttl_days: u64,
    pub reset_token_ttl_secs: u64,
    pub vendor_token_ttl_days: u64,
    /// Argon2 parameters (memory in KB)
    pub argon2_memory_kb: u32,
    pub argon2_iterations: u32,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let cfg = config::Config::builder()
            .add_source(config::File::with_name("config").required(false))
            .add_source(
                config::Environment::with_prefix("LA_RUCHE")
                    .prefix_separator("__")
                    .separator("__"),
            )
            .build()?;

        if config_debug_enabled() {
            print_config_diagnostics(&cfg);
        }

        match cfg.try_deserialize() {
            Ok(value) => Ok(value),
            Err(err) => {
                if config_debug_enabled() {
                    eprintln!("[config-debug] Deserialization failed: {err}");
                }
                Err(err.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_settings(host: &str, port: u16) -> AppSettings {
        AppSettings {
            host: host.to_string(),
            port,
            public_url: "http://localhost".to_string(),
            name: "La Ruche".to_string(),
            demo_mode: false,
        }
    }

    #[test]
    fn listen_addr_parses_valid_host_and_port() {
        let addr = app_settings("127.0.0.1", 8080).listen_addr().unwrap();
        assert_eq!(addr.to_string(), "127.0.0.1:8080");
    }

    #[test]
    fn listen_addr_rejects_non_ip_host() {
        let err = app_settings("not a host", 8080).listen_addr().unwrap_err();
        assert!(err.to_string().contains("Invalid listen address"));
    }
}
