use crate::api::ApiResult;
use crate::config;
use crate::models::{Device, validate_mac_address};
use crate::storage::{SharedStorage, StorageError};
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use tracing::{info, instrument};

#[instrument(skip_all)]
pub async fn get_devices(State(storage): State<SharedStorage>) -> ApiResult<Json<Vec<Device>>> {
    let devices = storage.get_all();
    info!(count = devices.len(), "Devices retrieved");
    Ok(Json(devices))
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExportResponse {
    pub name: String,
    pub mac_address: String,
    pub port: u16,
    pub ip_address: Option<String>,
    pub description: Option<String>,
}

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

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ImportRequest {
    pub name: String,
    pub mac_address: String,
    pub port: Option<u16>,
    pub ip_address: Option<String>,
    pub description: Option<String>,
}

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
            device.port.unwrap_or(config::get().wol.default_port),
            device.description,
        )?;
        devices.push(device);
    }
    storage.add_all(devices.clone())?;
    info!(count = devices.len(), "Devices imported");
    Ok((StatusCode::CREATED, Json(devices)))
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateDeviceRequest {
    pub name: String,
    pub mac_address: String,
    pub ip_address: Option<String>,
    pub port: Option<u16>,
    pub description: Option<String>,
}

#[instrument(skip_all, fields(device_name = %req.name))]
pub async fn create_device(
    State(storage): State<SharedStorage>,
    Json(req): Json<CreateDeviceRequest>,
) -> ApiResult<(StatusCode, Json<Device>)> {
    let device = Device::new(
        req.name,
        req.mac_address,
        req.ip_address,
        req.port.unwrap_or(config::get().wol.default_port),
        req.description,
    )?;

    storage.add(device.clone())?;

    info!(device_id = %device.id, "Device created");
    Ok((StatusCode::CREATED, Json(device)))
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct UpdateDeviceRequest {
    pub name: Option<String>,
    pub mac_address: Option<String>,
    pub ip_address: Option<String>,
    pub port: Option<u16>,
    pub description: Option<String>,
}

#[instrument(skip_all, fields(device_id = %id))]
pub async fn update_device(
    State(storage): State<SharedStorage>,
    Path(id): Path<String>,
    Json(req): Json<UpdateDeviceRequest>,
) -> ApiResult<Json<Device>> {
    let existing = storage.get(&id).ok_or(StorageError::NotFound(id.clone()))?;

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

#[instrument(skip_all, fields(device_id = %id))]
pub async fn delete_device(
    State(storage): State<SharedStorage>,
    Path(id): Path<String>,
) -> ApiResult<StatusCode> {
    storage.remove(&id)?;
    info!("Device deleted");
    Ok(StatusCode::NO_CONTENT)
}
