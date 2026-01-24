use parking_lot::RwLock;

use crate::error::AppError;
use crate::models::Device;
use std::fs;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct DeviceStorage {
    path: String,
    pub devices: Vec<Device>,
}

impl DeviceStorage {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            devices: Vec::new(),
        }
    }

    pub fn load(path: &str) -> Result<Self, AppError> {
        if !Path::new(path).exists() {
            return Ok(Self::new(path));
        }

        let content = fs::read_to_string(path).map_err(AppError::StorageIo)?;
        let devices: Vec<Device> =
            serde_json::from_str(&content).map_err(AppError::StorageParse)?;

        Ok(Self {
            path: path.to_string(),
            devices,
        })
    }

    pub fn save(&self) -> Result<(), AppError> {
        let content =
            serde_json::to_string_pretty(&self.devices).map_err(AppError::StorageParse)?;

        fs::write(&self.path, content).map_err(AppError::StorageIo)
    }

    pub fn add(&mut self, device: Device) -> Result<(), AppError> {
        self.devices.push(device);
        self.save()
    }

    pub fn add_all(&mut self, devices: Vec<Device>) -> Result<(), AppError> {
        self.devices.extend(devices);
        self.save()
    }

    pub fn remove(&mut self, id: &str) -> Result<Option<Device>, AppError> {
        let index = self.devices.iter().position(|d| d.id == id);
        match index {
            Some(i) => {
                let device = self.devices.remove(i);
                self.save()?;
                Ok(Some(device))
            }
            None => Ok(None),
        }
    }

    pub fn update(&mut self, id: &str, device: Device) -> Result<Option<Device>, AppError> {
        let index = self.devices.iter().position(|d| d.id == id);
        match index {
            Some(i) => {
                self.devices[i] = device;
                self.save()?;
                Ok(self.devices.get(i).cloned())
            }
            None => Ok(None),
        }
    }

    pub fn get(&self, id: &str) -> Option<Device> {
        self.devices.iter().find(|d| d.id == id).cloned()
    }

    pub fn get_all(&self) -> Vec<Device> {
        self.devices.clone()
    }
}

#[derive(Debug, Clone)]
pub struct SharedStorage(Arc<RwLock<DeviceStorage>>);

impl SharedStorage {
    pub fn load(path: &str) -> Result<Self, AppError> {
        Ok(Self(Arc::new(RwLock::new(DeviceStorage::load(path)?))))
    }

    pub fn add(&self, device: Device) -> Result<(), AppError> {
        self.0.write().add(device)
    }

    pub fn add_all(&self, devices: Vec<Device>) -> Result<(), AppError> {
        self.0.write().add_all(devices)
    }

    pub fn remove(&self, id: &str) -> Result<Option<Device>, AppError> {
        self.0.write().remove(id)
    }

    pub fn update(&self, id: &str, device: Device) -> Result<Option<Device>, AppError> {
        self.0.write().update(id, device)
    }

    pub fn get(&self, id: &str) -> Option<Device> {
        self.0.read().get(id)
    }

    pub fn get_all(&self) -> Vec<Device> {
        self.0.read().get_all()
    }
}
