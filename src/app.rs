use axum::http::HeaderValue;
use axum::{Router, http::Request, middleware};
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::services::ServeDir;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tower_http::{LatencyUnit, request_id::RequestId};
use tracing::Span;

use crate::api;
use crate::auth::{AuthState, auth_middleware};
use crate::config::AppConfig;
use crate::storage::SharedStorage;

#[derive(Clone)]
pub struct AppState {
    pub storage: SharedStorage,
    pub auth_state: AuthState,
    pub config: &'static AppConfig,
}

impl AppState {
    pub const fn new(
        storage: SharedStorage,
        auth_state: AuthState,
        config: &'static AppConfig,
    ) -> Self {
        Self {
            storage,
            auth_state,
            config,
        }
    }
}

pub fn build_app(state: AppState) -> Router {
    let cors = build_cors_layer(&state);

    Router::new()
        .fallback_service(ServeDir::new("static/dist"))
        .merge(api::router())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state)
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

fn build_cors_layer(state: &AppState) -> CorsLayer {
    if state.config.auth.disabled {
        return CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);
    }

    let has_wildcard = state
        .config
        .auth
        .allow_origins
        .iter()
        .any(|origin| origin.trim() == "*");

    if has_wildcard {
        let has_specific_origins = state
            .config
            .auth
            .allow_origins
            .iter()
            .any(|origin| origin.trim() != "*");

        if has_specific_origins {
            tracing::warn!(
                "CORS allow_origins contains '*' along with specific origins; specific origins are ignored"
            );
        }

        return CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);
    }

    let allowed_origins: Vec<HeaderValue> = state
        .config
        .auth
        .allow_origins
        .iter()
        .filter_map(|origin| match origin.parse() {
            Ok(origin) => Some(origin),
            Err(err) => {
                tracing::warn!(
                    origin = %origin,
                    error = %err,
                    "Ignoring invalid CORS origin in auth.allow_origins"
                );
                None
            }
        })
        .collect();

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(true);

    if allowed_origins.is_empty() {
        cors
    } else {
        cors.allow_origin(AllowOrigin::list(allowed_origins))
    }
}

pub fn build_service(state: AppState) -> Router {
    build_app(state)
}
