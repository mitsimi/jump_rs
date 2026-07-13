#![allow(clippy::needless_for_each)]

mod api;
mod app;
mod auth;
mod cli;
mod config;
mod devices;
mod error;
mod logging;
mod models;
mod storage;
mod web;

use crate::app::build_app;
use crate::cli::Cli;
use crate::storage::SharedStorage;
use clap::Parser;
use std::net::SocketAddr;
use tokio::signal;
use tracing::{error, info};

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

    let auth_state = match auth::AuthState::from_config(&config.auth) {
        Ok(state) => state,
        Err(err) => {
            eprintln!("Invalid authentication configuration: {err}");
            std::process::exit(1);
        }
    };

    logging::init();
    info!(version = env!("CARGO_PKG_VERSION"), "Starting jump.rs");

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

    let app = build_app(storage, auth_state);

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
}
