mod api;
mod arp;
mod config;
mod error;
mod models;
mod storage;
mod wol;

use crate::config::LogFormat;
use crate::storage::SharedStorage;
use axum::{
    Json, Router,
    http::Request,
    routing::{get, post, put},
};
use std::net::SocketAddr;
use tokio::signal;
use tower_http::cors::{Any, CorsLayer};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::services::ServeDir;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tower_http::{LatencyUnit, request_id::RequestId};
use tracing::{Span, error, info};
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    let config = match config::init() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Failed to load configuration: {err}");
            std::process::exit(1);
        }
    };

    init_tracing();
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
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<axum::body::Body>| {
                    // let request_id = request
                    //     .headers()
                    //     .get("x-request-id")
                    //     .and_then(|v| v.to_str().ok())
                    //     .unwrap_or("unknown");

                    let request_id = request
                        .extensions()
                        .get::<RequestId>()
                        .and_then(|id| id.header_value().to_str().ok())
                        .unwrap_or("unknown");

                    tracing::info_span!(
                        "request",
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                        request_id = %request_id,
                    )
                })
                .on_request(|_request: &Request<axum::body::Body>, _span: &Span| {
                    tracing::debug!("started processing request");
                })
                .on_response(DefaultOnResponse::new().latency_unit(LatencyUnit::Micros)),
        )
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid));

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

fn init_tracing() {
    let config = config::get();
    let server_config = &config.server;

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| server_config.log_level.as_filter().into());

    let fmt_layer = match server_config.log_format {
        LogFormat::Json => fmt::layer().json().boxed(),
        LogFormat::Pretty => fmt::layer().pretty().boxed(),
        LogFormat::Compact => fmt::layer().compact().boxed(),
    };

    #[cfg(feature = "otlp")]
    {
        use opentelemetry::KeyValue;
        use opentelemetry::trace::TracerProvider;
        use opentelemetry_otlp::WithExportConfig;
        use opentelemetry_sdk::Resource;

        if let Some(endpoint) = &config.otel.endpoint {
            let service_name = &config.otel.service_name;

            let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(endpoint)
                .build()
                .expect("Failed to create OTLP exporter");

            let tracer_provider = opentelemetry_sdk::trace::TracerProvider::builder()
                .with_batch_exporter(otlp_exporter, opentelemetry_sdk::runtime::Tokio)
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", service_name.clone()),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ]))
                .build();

            let tracer = tracer_provider.tracer(service_name.clone());

            opentelemetry::global::set_tracer_provider(tracer_provider);

            let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer)
                .with(otel_layer)
                .init();

            return;
        }
    }

    // Default: logging only (no OTLP export)
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}
