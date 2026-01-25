mod api;
mod arp;
mod config;
mod error;
mod models;
mod storage;
mod wol;

use crate::storage::SharedStorage;
use axum::{
    Router,
    routing::{get, post, put},
};
use std::net::SocketAddr;
use tokio::signal;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    let config = match config::init() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Failed to load configuration: {}", err);
            std::process::exit(1);
        }
    };

    tracing_subscriber::fmt()
        .with_level(true)
        .with_target(false)
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.server.log_level.clone().into()),
        )
        .init();

    info!("Configuration loaded successfully");

    let storage = match SharedStorage::load(&config.storage.file_path) {
        Ok(storage) => storage,
        Err(err) => {
            error!("Failed to load storage: {}", err);
            std::process::exit(1);
        }
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .fallback_service(ServeDir::new("static/dist"))
        .route(
            "/api/devices",
            get(api::get_devices).post(api::create_device),
        )
        .route("/api/devices/export", get(api::export_devices))
        .route("/api/devices/import", post(api::import_devices))
        .route(
            "/api/devices/{id}",
            put(api::update_device).delete(api::delete_device),
        )
        .route("/api/devices/{id}/wake", post(api::wake_device))
        .route("/api/arp-lookup", post(api::arp_lookup))
        .with_state(storage.clone())
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Server running on http://{}", addr);

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
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
