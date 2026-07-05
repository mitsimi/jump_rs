use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::{self, LogFormat};

pub fn init() {
    let config = config::get();
    let server_config = &config.server;

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| server_config.log_level.as_filter().into());

    let fmt_layer = match server_config.log_format {
        LogFormat::Json => fmt::layer().json().boxed(),
        LogFormat::Pretty => fmt::layer().pretty().boxed(),
        LogFormat::Compact => fmt::layer().compact().boxed(),
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}
