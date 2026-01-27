use parking_lot::RwLock;
use tracing::{debug, info, instrument};

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

    #[instrument(skip_all, fields(path = %path))]
    pub fn load(path: &str) -> Result<Self, AppError> {
        if !Path::new(path).exists() {
            info!("Storage file not found, starting fresh");
            return Ok(Self::new(path));
        }

        let content = fs::read_to_string(path).map_err(AppError::StorageIo)?;
        let devices: Vec<Device> = if content.trim().is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&content).map_err(AppError::StorageParse)?
        };

        info!(device_count = devices.len(), "Storage loaded");
        Ok(Self {
            path: path.to_string(),
            devices,
        })
    }

    #[instrument(skip_all, fields(path = %self.path))]
    pub fn save(&self) -> Result<(), AppError> {
        let content =
            serde_json::to_string_pretty(&self.devices).map_err(AppError::StorageParse)?;

        fs::write(&self.path, content).map_err(AppError::StorageIo)?;
        debug!(device_count = self.devices.len(), "Storage saved");
        Ok(())
    }

    #[instrument(skip_all)]
    pub fn add(&mut self, device: Device) -> Result<(), AppError> {
        self.devices.push(device);
        self.save()
    }

    #[instrument(skip_all)]
    pub fn add_all(&mut self, devices: Vec<Device>) -> Result<(), AppError> {
        self.devices.extend(devices);
        self.save()
    }

    #[instrument(skip_all)]
    pub fn remove(&mut self, id: &str) -> Result<Option<Device>, AppError> {
        let index = self.devices.iter().position(|d| d.id == id);
        if let Some(i) = index {
            let device = self.devices.remove(i);
            self.save()?;
            debug!("Device removed from storage");
            Ok(Some(device))
        } else {
            debug!("Device not found in storage");
            Ok(None)
        }
    }

    #[instrument(skip_all)]
    pub fn update(&mut self, id: &str, device: Device) -> Result<Option<Device>, AppError> {
        let index = self.devices.iter().position(|d| d.id == id);
        if let Some(i) = index {
            self.devices[i] = Device {
                id: self.devices[i].id.clone(),
                ..device
            };
            self.save()?;
            debug!("Device updated in storage");
            Ok(self.devices.get(i).cloned())
        } else {
            debug!("Device not found in storage");
            Ok(None)
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper to create a test device with a given name
    fn create_test_device(name: &str) -> Device {
        Device {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            mac_address: "AA:BB:CC:DD:EE:FF".to_string(),
            ip_address: Some("192.168.1.100".to_string()),
            port: 9,
            description: Some("Test device".to_string()),
            created_at: chrono::Utc::now(),
        }
    }

    /// Helper to create a temp file path
    fn temp_storage_path(dir: &TempDir) -> String {
        dir.path()
            .join("devices.json")
            .to_string_lossy()
            .to_string()
    }

    #[test]
    fn new_storage_is_empty() {
        let storage = DeviceStorage::new("/tmp/test.json");
        assert!(storage.devices.is_empty());
        assert_eq!(storage.get_all().len(), 0);
    }

    #[test]
    fn add_device_increases_count() {
        let dir = TempDir::new().unwrap();
        let path = temp_storage_path(&dir);
        let mut storage = DeviceStorage::new(&path);

        let device = create_test_device("Device 1");
        storage.add(device).unwrap();

        assert_eq!(storage.devices.len(), 1);
    }

    #[test]
    fn get_returns_added_device() {
        let dir = TempDir::new().unwrap();
        let path = temp_storage_path(&dir);
        let mut storage = DeviceStorage::new(&path);

        let device = create_test_device("Device 1");
        storage.add(device.clone()).unwrap();

        let retrieved = storage.get(&device.id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), device);
    }

    #[test]
    fn get_returns_none_for_unknown_id() {
        let storage = DeviceStorage::new("/tmp/test.json");
        let result = storage.get("nonexistent-id");
        assert!(result.is_none());
    }

    #[test]
    fn get_all_returns_all_devices() {
        let dir = TempDir::new().unwrap();
        let path = temp_storage_path(&dir);
        let mut storage = DeviceStorage::new(&path);

        storage.add(create_test_device("Device 1")).unwrap();
        storage.add(create_test_device("Device 2")).unwrap();
        storage.add(create_test_device("Device 3")).unwrap();

        let all = storage.get_all();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn remove_existing_device_returns_it() {
        let dir = TempDir::new().unwrap();
        let path = temp_storage_path(&dir);
        let mut storage = DeviceStorage::new(&path);

        let device = create_test_device("Device 1");
        storage.add(device.clone()).unwrap();

        let removed = storage.remove(&device.id).unwrap();
        assert!(removed.is_some());
        assert_eq!(removed.unwrap(), device);
        assert_eq!(storage.devices.len(), 0);
    }

    #[test]
    fn remove_nonexistent_device_returns_none() {
        let dir = TempDir::new().unwrap();
        let path = temp_storage_path(&dir);
        let mut storage = DeviceStorage::new(&path);

        let removed = storage.remove("nonexistent-id").unwrap();
        assert!(removed.is_none());
    }

    #[test]
    fn remove_nonexistent_device_maintains_others() {
        let dir = TempDir::new().unwrap();
        let path = temp_storage_path(&dir);
        let mut storage = DeviceStorage::new(&path);

        let device = create_test_device("Device 1");
        storage.add(device).unwrap();

        let _ = storage.remove("nonexistent-id").unwrap();
        assert!(storage.devices.len() == 1);
    }

    #[test]
    fn update_maintains_old_id() {
        let dir = TempDir::new().unwrap();
        let path = temp_storage_path(&dir);
        let mut storage = DeviceStorage::new(&path);

        let device = create_test_device("Device 1");
        storage.add(device.clone()).unwrap();

        let mut updated_device = create_test_device("Updated Name");

        let result = storage.update(&device.id, updated_device.clone()).unwrap();

        updated_device.id = device.id.clone();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), updated_device);
    }

    #[test]
    fn update_existing_device_succeeds() {
        let dir = TempDir::new().unwrap();
        let path = temp_storage_path(&dir);
        let mut storage = DeviceStorage::new(&path);

        let device = create_test_device("Original Name");
        storage.add(device.clone()).unwrap();

        let mut updated_device = create_test_device("Updated Name");
        updated_device.id = device.id.clone();

        let result = storage.update(&device.id, updated_device.clone()).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), updated_device);
    }

    #[test]
    fn update_nonexistent_device_returns_none() {
        let dir = TempDir::new().unwrap();
        let path = temp_storage_path(&dir);
        let mut storage = DeviceStorage::new(&path);

        let device = create_test_device("Test");
        let result = storage.update("nonexistent-id", device).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn add_all_adds_multiple_devices() {
        let dir = TempDir::new().unwrap();
        let path = temp_storage_path(&dir);
        let mut storage = DeviceStorage::new(&path);

        let devices = vec![
            create_test_device("Device 1"),
            create_test_device("Device 2"),
            create_test_device("Device 3"),
        ];

        storage.add_all(devices).unwrap();
        assert_eq!(storage.devices.len(), 3);
    }

    #[test]
    fn save_creates_file() {
        let dir = TempDir::new().unwrap();
        let path = temp_storage_path(&dir);
        let mut storage = DeviceStorage::new(&path);

        storage.add(create_test_device("Test")).unwrap();

        assert!(std::path::Path::new(&path).exists());
    }

    #[test]
    fn load_restores_saved_devices() {
        let dir = TempDir::new().unwrap();
        let path = temp_storage_path(&dir);

        // Save some devices
        {
            let mut storage = DeviceStorage::new(&path);
            storage.add(create_test_device("Device 1")).unwrap();
            storage.add(create_test_device("Device 2")).unwrap();
        }

        // Load them back
        let loaded = DeviceStorage::load(&path).unwrap();
        assert_eq!(loaded.devices.len(), 2);
    }

    #[test]
    fn load_missing_file_returns_empty_storage() {
        let dir = TempDir::new().unwrap();
        let path = dir
            .path()
            .join("nonexistent.json")
            .to_string_lossy()
            .to_string();

        let storage = DeviceStorage::load(&path).unwrap();
        assert!(storage.devices.is_empty());
    }

    #[test]
    fn load_empty_file_returns_empty_storage() {
        let dir = TempDir::new().unwrap();
        let path = temp_storage_path(&dir);

        // Create empty file
        std::fs::write(&path, "").unwrap();

        let storage = DeviceStorage::load(&path).unwrap();
        assert!(storage.devices.is_empty());
    }

    #[test]
    fn load_whitespace_only_file_returns_empty_storage() {
        let dir = TempDir::new().unwrap();
        let path = temp_storage_path(&dir);

        // Create file with only whitespace
        std::fs::write(&path, "   \n\t  ").unwrap();

        let storage = DeviceStorage::load(&path).unwrap();
        assert!(storage.devices.is_empty());
    }

    #[test]
    fn save_and_load_preserves_device_data() {
        let dir = TempDir::new().unwrap();
        let path = temp_storage_path(&dir);

        let original_device = Device {
            id: "test-id-123".to_string(),
            name: "My Server".to_string(),
            mac_address: "11:22:33:44:55:66".to_string(),
            ip_address: Some("10.0.0.50".to_string()),
            port: 7,
            description: Some("Production server".to_string()),
            created_at: chrono::Utc::now(),
        };

        // Save
        {
            let mut storage = DeviceStorage::new(&path);
            storage.add(original_device.clone()).unwrap();
        }

        // Load and verify
        let loaded = DeviceStorage::load(&path).unwrap();
        let device = loaded.get("test-id-123").unwrap();

        assert_eq!(device.name, "My Server");
        assert_eq!(device.mac_address, "11:22:33:44:55:66");
        assert_eq!(device.ip_address, Some("10.0.0.50".to_string()));
        assert_eq!(device.port, 7);
        assert_eq!(device.description, Some("Production server".to_string()));
    }

    #[test]
    fn shared_storage_load_creates_instance() {
        let dir = TempDir::new().unwrap();
        let path = temp_storage_path(&dir);

        let storage = SharedStorage::load(&path);
        assert!(storage.is_ok());
    }

    #[test]
    fn shared_storage_add_and_get() {
        let dir = TempDir::new().unwrap();
        let path = temp_storage_path(&dir);

        let storage = SharedStorage::load(&path).unwrap();
        let device = create_test_device("Shared Test");

        storage.add(device.clone()).unwrap();

        let retrieved = storage.get(&device.id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), device);
    }

    #[test]
    fn shared_storage_concurrent_reads() {
        let dir = TempDir::new().unwrap();
        let path = temp_storage_path(&dir);

        let storage = SharedStorage::load(&path).unwrap();
        let device = create_test_device("Concurrent Test");
        storage.add(device.clone()).unwrap();

        // Simulate concurrent reads by getting multiple references
        let all1 = storage.get_all();
        let all2 = storage.get_all();
        let single = storage.get(&device.id);

        assert_eq!(all1.len(), 1);
        assert_eq!(all2.len(), 1);
        assert!(single.is_some());
    }

    #[test]
    fn shared_storage_remove() {
        let dir = TempDir::new().unwrap();
        let path = temp_storage_path(&dir);

        let storage = SharedStorage::load(&path).unwrap();
        let device = create_test_device("To Remove");
        storage.add(device.clone()).unwrap();

        let removed = storage.remove(&device.id).unwrap();
        assert!(removed.is_some());
        assert!(storage.get(&device.id).is_none());
    }

    #[test]
    fn shared_storage_update() {
        let dir = TempDir::new().unwrap();
        let path = temp_storage_path(&dir);

        let storage = SharedStorage::load(&path).unwrap();
        let device = create_test_device("Original");
        storage.add(device.clone()).unwrap();

        let mut updated = create_test_device("Updated");

        storage.update(&device.id, updated.clone()).unwrap();

        updated.id = device.id.clone();
        let retrieved = storage.get(&device.id).unwrap();
        assert_eq!(retrieved, updated);
    }

    #[test]
    fn shared_storage_add_all() {
        let dir = TempDir::new().unwrap();
        let path = temp_storage_path(&dir);

        let storage = SharedStorage::load(&path).unwrap();
        let devices = vec![
            create_test_device("Device 1"),
            create_test_device("Device 2"),
        ];

        storage.add_all(devices).unwrap();
        assert_eq!(storage.get_all().len(), 2);
    }
}
