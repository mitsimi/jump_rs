mod arp;
mod devices;
mod wol;

pub use arp::*;
pub use devices::*;
pub use wol::*;

use crate::{error::ApiError, storage::SharedStorage};
use axum::{
    Router,
    routing::{get, post, put},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

type ApiResult<T> = std::result::Result<T, ApiError>;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "jump.rs",
        description = "Wake-on-LAN API for managing and waking network devices",
        version = env!("CARGO_PKG_VERSION"),
        license(name = "MIT"),
    ),
    paths(
        get_devices,
        export_devices,
        import_devices,
        create_device,
        update_device,
        delete_device,
        wake_device,
        arp_lookup,
    ),
    components(
        schemas(
            crate::models::Device,
            crate::error::ErrorResponse,
            ExportResponse,
            ImportRequest,
            CreateDeviceRequest,
            UpdateDeviceRequest,
            ArpLookupRequest,
            ArpLookupResponse
        )
    ),
    tags(
        (name = "devices", description = "Device management endpoints"),
        (name = "wol", description = "Wake-on-LAN operations"),
        (name = "network", description = "Network utility endpoints")
    )
)]
pub struct ApiDoc;

/// Creates and configures the API router with all API routes
pub fn router() -> Router<SharedStorage> {
    Router::new()
        .merge(SwaggerUi::new("/api/swagger").url("/api/docs/openapi.json", ApiDoc::openapi()))
        .route("/api/devices", get(get_devices).post(create_device))
        .route("/api/devices/export", get(export_devices))
        .route("/api/devices/import", post(import_devices))
        .route(
            "/api/devices/{id}",
            put(update_device).delete(delete_device),
        )
        .route("/api/devices/{id}/wake", post(wake_device))
        .route("/api/arp-lookup", post(arp_lookup))
}
