use serde::Deserialize;

use crate::api::devices::{CreateDeviceRequest, ImportRequest, UpdateDeviceRequest};

#[derive(Debug, Deserialize)]
pub struct DeviceForm {
    pub name: String,
    pub mac_address: String,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub ip_address: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none_u16")]
    pub port: Option<u16>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImportDevicesForm {
    pub payload: String,
}

#[derive(Debug, Deserialize)]
pub struct ArpLookupForm {
    pub ip_address: String,
    pub mac_address: Option<String>,
}

impl DeviceForm {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Device name is required".to_string());
        }

        if self.mac_address.trim().is_empty() {
            return Err("MAC address is required".to_string());
        }

        Ok(())
    }

    pub fn into_create_request(self) -> Result<CreateDeviceRequest, String> {
        self.validate()?;
        Ok(CreateDeviceRequest {
            name: self.name.trim().to_string(),
            mac_address: self.mac_address.trim().to_string(),
            ip_address: self
                .ip_address
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            port: self.port,
            description: self
                .description
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
        })
    }

    pub fn into_update_request(self) -> Result<UpdateDeviceRequest, String> {
        self.validate()?;
        Ok(UpdateDeviceRequest {
            name: Some(self.name.trim().to_string()),
            mac_address: Some(self.mac_address.trim().to_string()),
            ip_address: self
                .ip_address
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            port: self
                .port
                .or_else(|| Some(crate::config::get().wol.default_port)),
            description: self
                .description
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
        })
    }
}

impl ImportDevicesForm {
    pub fn into_import_requests(self) -> Result<Vec<ImportRequest>, String> {
        let payload = self.payload.trim();
        if payload.is_empty() {
            return Err("Paste or drop a JSON device list first".to_string());
        }

        serde_json::from_str(payload).map_err(|err| format!("Invalid JSON: {err}"))
    }
}

fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    Ok(value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }))
}

fn empty_string_as_none_u16<'de, D>(deserializer: D) -> Result<Option<u16>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    value
        .and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.parse::<u16>())
            }
        })
        .transpose()
        .map_err(serde::de::Error::custom)
}
