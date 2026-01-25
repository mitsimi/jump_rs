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
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub log_level: String,
}

#[derive(Debug, Deserialize)]
pub struct StorageConfig {
    pub file_path: String,
}

#[derive(Debug, Deserialize)]
pub struct WolConfig {
    pub default_port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            log_level: "info,tower_http=debug".to_string(),
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
        .set_default("server.port", 3000)?
        .set_default("server.log_level", "info,tower_http=debug")?
        .set_default("storage.file_path", "devices.json")?
        .set_default("wol.default_port", 9)?
        .add_source(File::with_name(&config_path).required(false))
        .add_source(
            Environment::with_prefix(ENV_PREFIX)
                .separator("_")
                .try_parsing(true),
        )
        .build()?;

    config.try_deserialize()
}
