use crate::api::ApiResult;
use crate::config;
use crate::error::ErrorResponse;
use crate::models::{Device, validate_mac_address};
use crate::storage::{SharedStorage, StorageError};
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use tracing::{info, instrument};
use utoipa::ToSchema;

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
pub async fn get_devices(State(storage): State<SharedStorage>) -> ApiResult<Json<Vec<Device>>> {
    let devices = storage.get_all();
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
    State(storage): State<SharedStorage>,
) -> ApiResult<Json<Vec<ExportResponse>>> {
    let devices = storage.get_all();
    let count = devices.len();
    let result: Vec<ExportResponse> = devices
        .into_iter()
        .map(|device| ExportResponse {
            name: device.name,
            mac_address: device.mac_address,
            port: device.port,
            ip_address: device.ip_address,
            description: device.description,
        })
        .collect();
    info!(count = count, "Devices exported");
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
    State(storage): State<SharedStorage>,
    Json(req): Json<Vec<ImportRequest>>,
) -> ApiResult<(StatusCode, Json<Vec<Device>>)> {
    let mut devices = Vec::new();
    for device in req {
        let device = Device::new(
            device.name,
            device.mac_address,
            device.ip_address,
            device
                .port
                .unwrap_or_else(|| config::get().wol.default_port),
            device.description,
        )?;
        devices.push(device);
    }
    storage.add_all(devices.clone())?;
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
    State(storage): State<SharedStorage>,
    Json(req): Json<CreateDeviceRequest>,
) -> ApiResult<(StatusCode, Json<Device>)> {
    let device = Device::new(
        req.name,
        req.mac_address,
        req.ip_address,
        req.port.unwrap_or_else(|| config::get().wol.default_port),
        req.description,
    )?;

    storage.add(device.clone())?;

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
    State(storage): State<SharedStorage>,
    Path(id): Path<String>,
    Json(req): Json<UpdateDeviceRequest>,
) -> ApiResult<Json<Device>> {
    let existing = storage
        .get(&id)
        .ok_or_else(|| StorageError::NotFound(id.clone()))?;

    let mac_address = match req.mac_address {
        Some(mac) => {
            validate_mac_address(&mac)?;
            mac
        }
        None => existing.mac_address,
    };

    let updated = Device {
        id: existing.id,
        name: req.name.unwrap_or(existing.name),
        mac_address,
        ip_address: req.ip_address.or(existing.ip_address),
        port: req.port.unwrap_or(existing.port),
        description: req.description,
        created_at: existing.created_at,
    };

    storage.update(&id, updated.clone())?;
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
    State(storage): State<SharedStorage>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    storage.remove(&id)?;
    info!("Device deleted");
    Ok(StatusCode::NO_CONTENT)
}
