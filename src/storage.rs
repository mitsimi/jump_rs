use crate::error::AppError;
use crate::models::Device;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

const STORAGE_FILE: &str = "devices.json";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceStorage {
    pub devices: Vec<Device>,
}

impl DeviceStorage {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
        }
    }

    pub fn load() -> Result<Self, AppError> {
        if !Path::new(STORAGE_FILE).exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(STORAGE_FILE).map_err(AppError::StorageIo)?;

        serde_json::from_str(&content).map_err(AppError::StorageParse)
    }

    pub fn save(&self) -> Result<(), AppError> {
        let content = serde_json::to_string_pretty(self).map_err(AppError::StorageParse)?;

        fs::write(STORAGE_FILE, content).map_err(AppError::StorageIo)
    }

    pub fn add(&mut self, device: Device) -> Result<(), AppError> {
        self.devices.push(device);
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

    pub fn close(&self) -> Result<(), AppError> {
        self.save()
    }
}

pub type SharedStorage = Arc<Mutex<DeviceStorage>>;

pub fn load_storage() -> SharedStorage {
    Arc::new(Mutex::new(
        DeviceStorage::load().unwrap_or_else(|_| DeviceStorage::new()),
    ))
}
