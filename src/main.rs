mod api;
mod app;
mod arp;
mod auth;
mod cli;
mod config;
mod error;
mod models;
mod storage;
mod telemetry;
mod wol;

use crate::app::{AppState, build_service};
use crate::auth::{
    AuthState, SessionManager, UserStore, has_global_wildcard, validate_auth_config,
};
use crate::cli::Cli;
use crate::storage::SharedStorage;
use clap::Parser;
use std::net::SocketAddr;
use tokio::signal;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Handle CLI commands that exit before running the server
    if cli.handle_commands() {
        return;
    }

    let config = match config::init() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Failed to load configuration: {err}");
            std::process::exit(1);
        }
    };

    if let Err(err) = validate_auth_config(&config.auth) {
        eprintln!("Failed to load configuration: {err}");
        std::process::exit(1);
    }

    telemetry::init_tracing();
    info!(version = env!("CARGO_PKG_VERSION"), "Starting jump.rs");

    if config.auth.disabled
        && has_global_wildcard(&config.auth.allow_origins)
        && config
            .auth
            .allow_origins
            .iter()
            .any(|origin| origin.trim() != "*")
    {
        warn!(
            "auth.allow_origins contains '*' along with specific origins; specific origins are ignored"
        );
    }

    let storage: SharedStorage = {
        let file_path = &config.storage.file_path;
        match SharedStorage::load(file_path) {
            Ok(storage) => {
                info!(file = file_path, "Storage initialized");
                storage
            }
            Err(err) => {
                error!(error = %err, file = file_path, "Failed to load storage");
                std::process::exit(1);
            }
        }
    };

    let auth_state = {
        let user_store = UserStore::from_config(&config.auth.users);
        let session_manager = SessionManager::new(config.auth.session_timeout);

        if config.auth.disabled {
            info!("Authentication disabled");
        } else if user_store.is_empty() {
            warn!("Authentication enabled but no users configured");
        } else {
            info!(
                user_count = user_store.user_count(),
                "Authentication enabled"
            );
        }

        AuthState::new(user_store, session_manager)
    };

    let app_state = AppState::new(storage, auth_state, config);
    let app = build_service(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!(addr = %addr, "Server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("failed to start server");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    info!("Shutdown signal received");

    // Flush any pending traces before shutdown
    #[cfg(feature = "otlp")]
    {
        opentelemetry::global::shutdown_tracer_provider();
    }
}
