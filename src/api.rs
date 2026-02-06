mod arp;
mod devices;
mod wol;

use crate::error::ApiError;
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub type ApiResult<T> = std::result::Result<T, ApiError>;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "jump.rs",
        description = "Wake-on-LAN API for managing and waking network devices",
        version = env!("CARGO_PKG_VERSION"),
        license(name = "MIT"),
    )
)]
pub struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut doc = ApiDoc::openapi();
    doc.merge(devices::DeviceApiDoc::openapi());
    doc.merge(wol::WolApiDoc::openapi());
    doc.merge(arp::NetworkApiDoc::openapi());
    doc
}

/// Creates and configures the API router with all API routes
pub fn router() -> Router {
    Router::new()
        .merge(SwaggerUi::new("/api/swagger").url("/api/docs/openapi.json", openapi()))
        .merge(devices::router())
        .merge(wol::router())
        .merge(arp::router())
}
