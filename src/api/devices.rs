use crate::error::AppError;
use crate::models::{Device, validate_mac_address};
use crate::storage::SharedStorage;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;

pub async fn get_devices(
    State(storage): State<SharedStorage>,
) -> Result<Json<Vec<Device>>, AppError> {
    let devices = storage.get_all();
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

pub async fn export_devices(
    State(storage): State<SharedStorage>,
) -> Result<Json<Vec<ExportResponse>>, AppError> {
    let devices = storage.get_all();
    Ok(Json(
        devices
            .into_iter()
            .map(|device| ExportResponse {
                name: device.name,
                mac_address: device.mac_address,
                port: device.port,
                ip_address: device.ip_address,
                description: device.description,
            })
            .collect(),
    ))
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ImportRequest {
    pub name: String,
    pub mac_address: String,
    pub port: Option<u16>,
    pub ip_address: Option<String>,
    pub description: Option<String>,
}

pub async fn import_devices(
    State(storage): State<SharedStorage>,
    Json(req): Json<Vec<ImportRequest>>,
) -> Result<(StatusCode, Json<Vec<Device>>), AppError> {
    let mut devices = Vec::new();
    for device in req {
        let device = Device::new(
            device.name,
            device.mac_address,
            device.ip_address,
            device.port,
            device.description,
        )?;
        devices.push(device);
    }
    storage.add_all(devices.clone())?;
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

pub async fn create_device(
    State(storage): State<SharedStorage>,
    Json(req): Json<CreateDeviceRequest>,
) -> Result<(StatusCode, Json<Device>), AppError> {
    let device = Device::new(
        req.name,
        req.mac_address,
        req.ip_address,
        req.port,
        req.description,
    )?;

    {
        storage.add(device.clone())?;
    }

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

pub async fn update_device(
    State(storage): State<SharedStorage>,
    Path(id): Path<String>,
    Json(req): Json<UpdateDeviceRequest>,
) -> Result<Json<Device>, AppError> {
    let existing = storage
        .get(&id)
        .ok_or(AppError::DeviceNotFound(id.clone()))?;

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
        description: req.description.or(existing.description),
        created_at: existing.created_at,
    };

    storage.update(&id, updated.clone())?;
    Ok(Json(updated))
}

pub async fn delete_device(
    State(storage): State<SharedStorage>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    storage.remove(&id)?;
    Ok(StatusCode::NO_CONTENT)
}
