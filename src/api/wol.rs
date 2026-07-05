use axum::{Extension, Router, extract::Path, http::StatusCode, routing::post};
use tracing::{info, instrument};
use utoipa::OpenApi;

use crate::{api::ApiResult, error::ErrorResponse, storage::SharedStorage};

#[derive(OpenApi)]
#[openapi(
    paths(
        wake_device,
    ),
    components(
        schemas(
            crate::error::ErrorResponse,
        )
    ),
    tags(
        (name = "wol", description = "Wake-on-LAN operations")
    )
)]
pub struct WolApiDoc;

pub fn router() -> Router {
    Router::new().route("/api/devices/{id}/wake", post(wake_device))
}

#[utoipa::path(
    post,
    path = "/api/devices/{id}/wake",
    operation_id = "wakeDevice",
    tag = "wol",
    summary = "Wake a device",
    description = "Sends a Wake-on-LAN magic packet to the device. The target device must have WoL enabled in BIOS and be connected via ethernet.",
    params(
        ("id" = String, Path, description = "Device ID", example = "V1StGXR8_Z5jdHi6B")
    ),
    responses(
        (status = 204, description = "WoL packet sent successfully"),
        (status = 404, description = "Device not found", body = ErrorResponse),
        (status = 500, description = "Network error while sending packet", body = ErrorResponse)
    )
)]
#[instrument(skip_all, fields(device_id = %id))]
pub async fn wake_device(
    Extension(storage): Extension<SharedStorage>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    crate::devices::wake_device(&storage, &id)?;
    info!(device_id = %id, "WoL packet sent");
    Ok(StatusCode::NO_CONTENT)
}
