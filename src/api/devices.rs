use crate::api::ApiResult;
use crate::error::ErrorResponse;
use crate::models::Device;
use crate::storage::SharedStorage;
use axum::{
    Router,
    extract::{Extension, Json, Path},
    http::StatusCode,
    routing::{get, post, put},
};
use tracing::{info, instrument};
use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    paths(
        get_devices,
        export_devices,
        import_devices,
        create_device,
        update_device,
        delete_device,
    ),
    components(
        schemas(
            crate::models::Device,
            crate::error::ErrorResponse,
            ExportResponse,
            ImportRequest,
            CreateDeviceRequest,
            UpdateDeviceRequest,
        )
    ),
    tags(
        (name = "devices", description = "Device management endpoints")
    )
)]
pub struct DeviceApiDoc;

pub fn router() -> Router {
    Router::new()
        .route("/api/devices", get(get_devices).post(create_device))
        .route("/api/devices/export", get(export_devices))
        .route("/api/devices/import", post(import_devices))
        .route(
            "/api/devices/{id}",
            put(update_device).delete(delete_device),
        )
}

#[utoipa::path(
    get,
    path = "/api/devices",
    operation_id = "getDevices",
    tag = "devices",
    summary = "List all devices",
    description = "Returns a list of all registered devices that can receive Wake-on-LAN packets.",
    responses(
        (status = 200, description = "List of all devices", body = Vec<Device>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[instrument(skip_all)]
pub async fn get_devices(
    Extension(storage): Extension<SharedStorage>,
) -> ApiResult<Json<Vec<Device>>> {
    let devices = crate::devices::list_devices(&storage);
    info!(count = devices.len(), "Devices retrieved");
    Ok(Json(devices))
}

#[derive(Debug, Clone, serde::Serialize, ToSchema)]
pub struct ExportResponse {
    #[schema(example = "Gaming PC")]
    pub name: String,
    #[schema(example = "00:11:22:33:44:55")]
    pub mac_address: String,
    #[schema(example = 9)]
    pub port: u16,
    #[schema(example = "192.168.1.100")]
    pub ip_address: Option<String>,
    #[schema(example = "My main gaming rig")]
    pub description: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/devices/export",
    operation_id = "exportDevices",
    tag = "devices",
    summary = "Export all devices",
    description = "Exports all devices in a portable format suitable for backup or migration. Does not include internal fields like id and created_at.",
    responses(
        (status = 200, description = "Exported device list", body = Vec<ExportResponse>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[instrument(skip_all)]
pub async fn export_devices(
    Extension(storage): Extension<SharedStorage>,
) -> ApiResult<Json<Vec<ExportResponse>>> {
    let result = crate::devices::export_devices(&storage);
    info!(count = result.len(), "Devices exported");
    Ok(Json(result))
}

#[derive(Debug, Clone, serde::Deserialize, ToSchema)]
pub struct ImportRequest {
    #[schema(example = "Gaming PC")]
    pub name: String,
    #[schema(example = "00:11:22:33:44:55")]
    pub mac_address: String,
    #[schema(example = 9)]
    pub port: Option<u16>,
    #[schema(example = "192.168.1.100")]
    pub ip_address: Option<String>,
    #[schema(example = "My main gaming rig")]
    pub description: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/devices/import",
    operation_id = "importDevices",
    tag = "devices",
    summary = "Import devices",
    description = "Imports multiple devices from a portable format. Useful for restoring backups or migrating from another system.",
    request_body(content = Vec<ImportRequest>, description = "List of devices to import"),
    responses(
        (status = 201, description = "Devices imported successfully", body = Vec<Device>),
        (status = 400, description = "Validation error (e.g., invalid MAC address)", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[instrument(skip_all, fields(count = req.len()))]
pub async fn import_devices(
    Extension(storage): Extension<SharedStorage>,
    Json(req): Json<Vec<ImportRequest>>,
) -> ApiResult<(StatusCode, Json<Vec<Device>>)> {
    let devices = crate::devices::import_devices(&storage, req)?;
    info!(count = devices.len(), "Devices imported");
    Ok((StatusCode::CREATED, Json(devices)))
}

#[derive(Debug, Clone, serde::Deserialize, ToSchema)]
pub struct CreateDeviceRequest {
    #[schema(example = "Gaming PC")]
    pub name: String,
    #[schema(example = "00:11:22:33:44:55")]
    pub mac_address: String,
    #[schema(example = "192.168.1.100")]
    pub ip_address: Option<String>,
    #[schema(example = 9)]
    pub port: Option<u16>,
    #[schema(example = "My main gaming rig")]
    pub description: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/devices",
    operation_id = "createDevice",
    tag = "devices",
    summary = "Create a new device",
    description = "Creates a new device that can receive Wake-on-LAN packets. The MAC address must be valid.",
    request_body(content = CreateDeviceRequest, description = "Device to create"),
    responses(
        (status = 201, description = "Device created successfully", body = Device),
        (status = 400, description = "Validation error (e.g., invalid MAC address)", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[instrument(skip_all, fields(device_name = %req.name))]
pub async fn create_device(
    Extension(storage): Extension<SharedStorage>,
    Json(req): Json<CreateDeviceRequest>,
) -> ApiResult<(StatusCode, Json<Device>)> {
    let device = crate::devices::create_device(&storage, req)?;
    info!(device_id = %device.id, "Device created");
    Ok((StatusCode::CREATED, Json(device)))
}

#[derive(Debug, Clone, serde::Deserialize, ToSchema)]
pub struct UpdateDeviceRequest {
    #[schema(example = "Gaming PC")]
    pub name: Option<String>,
    #[schema(example = "00:11:22:33:44:55")]
    pub mac_address: Option<String>,
    #[schema(example = "192.168.1.100")]
    pub ip_address: Option<String>,
    #[schema(example = 9)]
    pub port: Option<u16>,
    #[schema(example = "Updated description")]
    pub description: Option<String>,
}

#[utoipa::path(
    put,
    path = "/api/devices/{id}",
    operation_id = "updateDevice",
    tag = "devices",
    summary = "Update a device",
    description = "Updates an existing device. Only provided fields will be updated (partial update).",
    params(
        ("id" = String, Path, description = "Device ID", example = "V1StGXR8_Z5jdHi6B")
    ),
    request_body(content = UpdateDeviceRequest, description = "Fields to update"),
    responses(
        (status = 200, description = "Device updated successfully", body = Device),
        (status = 400, description = "Validation error (e.g., invalid MAC address)", body = ErrorResponse),
        (status = 404, description = "Device not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[instrument(skip_all, fields(device_id = %id))]
pub async fn update_device(
    Extension(storage): Extension<SharedStorage>,
    Path(id): Path<String>,
    Json(req): Json<UpdateDeviceRequest>,
) -> ApiResult<Json<Device>> {
    let updated = crate::devices::update_device(&storage, &id, req)?;
    info!("Device updated");
    Ok(Json(updated))
}

#[utoipa::path(
    delete,
    path = "/api/devices/{id}",
    operation_id = "deleteDevice",
    tag = "devices",
    summary = "Delete a device",
    description = "Permanently deletes a device from the system.",
    params(
        ("id" = String, Path, description = "Device ID", example = "V1StGXR8_Z5jdHi6B")
    ),
    responses(
        (status = 204, description = "Device deleted successfully"),
        (status = 404, description = "Device not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[instrument(skip_all, fields(device_id = %id))]
pub async fn delete_device(
    Extension(storage): Extension<SharedStorage>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    crate::devices::delete_device(&storage, &id)?;
    info!("Device deleted");
    Ok(StatusCode::NO_CONTENT)
}
