use axum::Extension;
use axum::{Router, http::Request, response::Response};
use std::time::Duration;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::request_id::RequestId;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::Span;

use crate::api;
use crate::auth;
use crate::storage::SharedStorage;
use crate::web;

pub fn build_app(storage: SharedStorage, auth_state: Option<auth::AuthState>) -> Router {
    let mut protected = Router::new().merge(web::router()).merge(api::router());

    if crate::config::get().server.api_docs {
        protected = protected.merge(api::docs_router());
    }

    let router = if let Some(auth_state) = auth_state {
        Router::new()
            .merge(auth::public_router(auth_state.clone()))
            .merge(protected.route_layer(axum::middleware::from_fn_with_state(
                auth_state,
                auth::require_authentication,
            )))
    } else {
        protected
    }
    .nest_service("/static", ServeDir::new("static"));

    router
        .layer(Extension(storage))
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
                .on_response(|response: &Response, latency: Duration, _span: &Span| {
                    tracing::info!(
                        status = response.status().as_u16(),
                        latency_us = latency.as_micros(),
                        "finished processing request"
                    );
                })
                .on_failure(
                    |error: ServerErrorsFailureClass, latency: Duration, _span: &Span| {
                        tracing::error!(
                            error = %error,
                            latency_us = latency.as_micros(),
                            "request failed"
                        );
                    },
                ),
        )
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
}
