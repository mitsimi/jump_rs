use crate::error::AppError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub mac_address: String,
    pub ip_address: Option<String>,
    pub port: u16,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Device {
    pub fn new(
        name: String,
        mac_address: String,
        ip_address: Option<String>,
        port: u16,
        description: Option<String>,
    ) -> Result<Self, AppError> {
        validate_mac_address(&mac_address)?;

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            name,
            mac_address,
            ip_address,
            port,
            description,
            created_at: chrono::Utc::now(),
        })
    }
}

pub fn validate_mac_address(mac_str: &str) -> Result<(), AppError> {
    let cleaned: String = mac_str.replace([':', '-', '.', ' '], "").to_lowercase();

    if cleaned.len() != 12 {
        return Err(AppError::InvalidMac(mac_str.to_string()));
    }

    for (i, chunk) in cleaned.as_bytes().chunks(2).enumerate() {
        if i >= 6 {
            return Err(AppError::InvalidMac(mac_str.to_string()));
        }
        let hex_str = std::str::from_utf8(chunk).unwrap();
        if u8::from_str_radix(hex_str, 16).is_err() {
            return Err(AppError::InvalidMac(mac_str.to_string()));
        }
    }

    Ok(())
}
