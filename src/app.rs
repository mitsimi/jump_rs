use axum::Extension;
use axum::{Router, http::Request};
use tower_http::cors::{Any, CorsLayer};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::services::ServeDir;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tower_http::{LatencyUnit, request_id::RequestId};
use tracing::Span;

use crate::api;
use crate::storage::SharedStorage;

pub fn build_app(storage: SharedStorage) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .fallback_service(ServeDir::new("static/dist"))
        .merge(api::router())
        .layer(Extension(storage))
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<axum::body::Body>| {
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
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
}

pub fn build_service(storage: SharedStorage) -> Router {
    build_app(storage)
}
