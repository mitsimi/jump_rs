use crate::api::ApiResult;
use crate::api::devices::{
    CreateDeviceRequest, ExportResponse, ImportRequest, UpdateDeviceRequest,
};
use crate::config;
use crate::models::{Device, validate_mac_address};
use crate::storage::{SharedStorage, StorageError};

pub fn list_devices(storage: &SharedStorage) -> Vec<Device> {
    storage.get_all()
}

pub fn export_devices(storage: &SharedStorage) -> Vec<ExportResponse> {
    storage
        .get_all()
        .into_iter()
        .map(|device| ExportResponse {
            name: device.name,
            mac_address: device.mac_address,
            port: device.port,
            ip_address: device.ip_address,
            description: device.description,
        })
        .collect()
}

pub fn import_devices(storage: &SharedStorage, req: Vec<ImportRequest>) -> ApiResult<Vec<Device>> {
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
    Ok(devices)
}

pub fn create_device(storage: &SharedStorage, req: CreateDeviceRequest) -> ApiResult<Device> {
    let device = Device::new(
        req.name,
        req.mac_address,
        req.ip_address,
        req.port.unwrap_or_else(|| config::get().wol.default_port),
        req.description,
    )?;

    storage.add(device.clone())?;
    Ok(device)
}

pub fn update_device(
    storage: &SharedStorage,
    id: &str,
    req: UpdateDeviceRequest,
) -> ApiResult<Device> {
    let existing = storage
        .get(id)
        .ok_or_else(|| StorageError::NotFound(id.to_string()))?;

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

    Ok(storage.update(id, updated)?)
}

pub fn delete_device(storage: &SharedStorage, id: &str) -> ApiResult<()> {
    storage.remove(id)?;
    Ok(())
}

pub fn wake_device(storage: &SharedStorage, id: &str) -> ApiResult<()> {
    let device = storage
        .get(id)
        .ok_or_else(|| StorageError::NotFound(id.to_string()))?;

    crate::wol::send_wol_packet(&device)?;
    Ok(())
}

pub fn arp_lookup(ip: &str) -> ApiResult<String> {
    Ok(crate::arp::lookup_mac(ip)?)
}
