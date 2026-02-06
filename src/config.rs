use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;
use std::sync::OnceLock;

const DEFAULT_CONFIG_FILE: &str = "config.toml";
const ENV_PREFIX: &str = "JUMPERS";
const CONFIG_PATH_ENV: &str = "JUMPERS_CONFIG";

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

#[derive(Debug, Default, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub wol: WolConfig,
    pub auth: AuthConfig,
    #[cfg(feature = "otlp")]
    pub otel: OtelConfig,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub port: u16,
    pub log_level: LogLevel,
    pub log_format: LogFormat,
    /// Allow cookies over insecure HTTP (useful for local development).
    pub allow_insecure_cookies: bool,
    /// Explicit origins allowed for auth actions.
    pub allow_origins: Vec<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub const fn as_filter(self) -> &'static str {
        match self {
            Self::Trace => "trace,tower_http=trace",
            Self::Debug => "debug,tower_http=debug",
            Self::Info => "info,tower_http=debug",
            Self::Warn => "warn,tower_http=warn",
            Self::Error => "error,tower_http=error",
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    #[default]
    Compact,
    Json,
    Pretty,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct StorageConfig {
    pub file_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct WolConfig {
    pub default_port: u16,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct AuthConfig {
    /// Whether authentication is disabled.
    pub disabled: bool,
    /// String of comma-separated usernames and passwords.
    /// Format: username:password,username:password,...
    /// Passwords are hashed using bcrypt.
    pub users: String,
    /// Session timeout in seconds.
    pub session_timeout: u64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            disabled: false,
            users: String::new(),
            session_timeout: 86400,
        }
    }
}

#[cfg(feature = "otlp")]
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct OtelConfig {
    /// OTLP endpoint URL (e.g., <http://localhost:4317>). If empty, OTEL is disabled.
    pub endpoint: Option<String>,
    /// Service name for traces. Defaults to `jump_rs`.
    pub service_name: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            log_level: LogLevel::default(),
            log_format: LogFormat::default(),
            allow_insecure_cookies: false,
            allow_origins: Vec::new(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            file_path: "devices.json".to_string(),
        }
    }
}

impl Default for WolConfig {
    fn default() -> Self {
        Self { default_port: 9 }
    }
}

#[cfg(feature = "otlp")]
impl Default for OtelConfig {
    fn default() -> Self {
        Self {
            endpoint: None,
            service_name: "jump_rs".to_string(),
        }
    }
}

/// Initialize the global configuration. Must be called once at startup.
pub fn init() -> Result<&'static AppConfig, ConfigError> {
    let config = load()?;
    Ok(CONFIG.get_or_init(|| config))
}

/// Get the global configuration. Panics if `init()` was not called.
pub fn get() -> &'static AppConfig {
    CONFIG
        .get()
        .expect("Config not initialized. Call config::init() first.")
}

fn load() -> Result<AppConfig, ConfigError> {
    let config_path = env::var(CONFIG_PATH_ENV).unwrap_or_else(|_| DEFAULT_CONFIG_FILE.to_string());

    let config = Config::builder()
        .add_source(File::with_name(&config_path).required(false))
        .add_source(
            Environment::with_prefix(ENV_PREFIX)
                .separator("_")
                .try_parsing(true),
        )
        .build()?;

    config.try_deserialize()
}
