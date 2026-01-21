use crate::arp;
use crate::error::AppError;
use crate::models::{Device, validate_mac_address};
use crate::storage::SharedStorage;
use crate::wol;
use axum::{
    extract::{Json, Path, State},
    response::IntoResponse,
};

pub async fn get_devices(State(storage): State<SharedStorage>) -> Json<Vec<Device>> {
    Json(storage.lock().unwrap().get_all())
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
) -> Result<Json<Device>, AppError> {
    let device = Device::new(
        req.name,
        req.mac_address,
        req.ip_address,
        req.port,
        req.description,
    )?;

    {
        let mut storage = storage.lock().unwrap();
        storage.add(device.clone())?;
    }

    Ok(Json(device))
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
        .lock()
        .unwrap()
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

    storage.lock().unwrap().update(&id, updated.clone())?;
    Ok(Json(updated))
}

pub async fn delete_device(
    State(storage): State<SharedStorage>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    storage.lock().unwrap().remove(&id)?;
    Ok(Json(serde_json::json!({ "success": true })))
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WakeResponse {
    pub success: bool,
    pub message: String,
    pub device_name: String,
}

pub async fn wake_device(
    State(storage): State<SharedStorage>,
    Path(id): Path<String>,
) -> Result<Json<WakeResponse>, AppError> {
    let device = storage
        .lock()
        .unwrap()
        .get(&id)
        .ok_or(AppError::DeviceNotFound(id))?;

    wol::send_wol_packet(&device)?;

    Ok(Json(WakeResponse {
        success: true,
        message: format!("Wake-on-LAN packet sent to {}", device.name),
        device_name: device.name,
    }))
}

#[derive(Debug, serde::Deserialize)]
pub struct ArpLookupRequest {
    ip: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ArpLookupResponse {
    pub mac: Option<String>,
    pub found: bool,
    pub error: Option<String>,
}

pub async fn arp_lookup(
    State(_storage): State<SharedStorage>,
    Json(req): Json<ArpLookupRequest>,
) -> Json<ArpLookupResponse> {
    match arp::lookup_mac(&req.ip) {
        Ok(mac) => Json(ArpLookupResponse {
            mac: mac.clone(),
            found: mac.is_some(),
            error: None,
        }),
        Err(e) => Json(ArpLookupResponse {
            mac: None,
            found: false,
            error: Some(e.to_string()),
        }),
    }
}
