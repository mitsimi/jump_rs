use crate::error::AppError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_mac_with_colons() {
        assert!(validate_mac_address("AA:BB:CC:DD:EE:FF").is_ok());
    }

    #[test]
    fn validate_mac_with_dashes() {
        assert!(validate_mac_address("AA-BB-CC-DD-EE-FF").is_ok());
    }

    #[test]
    fn validate_mac_with_dots() {
        assert!(validate_mac_address("AABB.CCDD.EEFF").is_ok());
    }

    #[test]
    fn validate_mac_without_separators() {
        assert!(validate_mac_address("AABBCCDDEEFF").is_ok());
    }

    #[test]
    fn validate_mac_lowercase() {
        assert!(validate_mac_address("aa:bb:cc:dd:ee:ff").is_ok());
    }

    #[test]
    fn validate_mac_mixed_case() {
        assert!(validate_mac_address("Aa:Bb:Cc:Dd:Ee:Ff").is_ok());
    }

    #[test]
    fn validate_mac_with_spaces() {
        assert!(validate_mac_address("AA BB CC DD EE FF").is_ok());
    }

    #[test]
    fn validate_mac_mixed_separators() {
        assert!(validate_mac_address("AA:BB-CC.DD EE:FF").is_ok());
    }

    #[test]
    fn reject_mac_too_short() {
        let result = validate_mac_address("AA:BB:CC:DD:EE");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::InvalidMac(_)));
    }

    #[test]
    fn reject_mac_too_long() {
        let result = validate_mac_address("AA:BB:CC:DD:EE:FF:00");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::InvalidMac(_)));
    }

    #[test]
    fn reject_mac_empty() {
        let result = validate_mac_address("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::InvalidMac(_)));
    }

    #[test]
    fn reject_mac_invalid_hex_chars() {
        let result = validate_mac_address("GG:HH:II:JJ:KK:LL");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::InvalidMac(_)));
    }

    #[test]
    fn reject_mac_special_chars() {
        let result = validate_mac_address("AA:BB:CC:DD:EE:F!");
        assert!(result.is_err());
    }

    #[test]
    fn reject_mac_partial_invalid() {
        let result = validate_mac_address("AA:BB:CC:DD:EE:ZZ");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::InvalidMac(_)));
    }

    #[test]
    fn create_device_with_valid_mac() {
        let device = Device::new(
            "Test Device".to_string(),
            "AA:BB:CC:DD:EE:FF".to_string(),
            Some("192.168.1.100".to_string()),
            9,
            Some("Test description".to_string()),
        );
        assert!(device.is_ok());
        let device = device.unwrap();
        assert_eq!(device.name, "Test Device");
        assert_eq!(device.mac_address, "AA:BB:CC:DD:EE:FF");
        assert_eq!(device.port, 9);
    }

    #[test]
    fn create_device_with_invalid_mac_fails() {
        let result = Device::new("Test".to_string(), "invalid-mac".to_string(), None, 9, None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::InvalidMac(_)));
    }

    #[test]
    fn create_device_with_optional_fields_none() {
        let device = Device::new(
            "Minimal Device".to_string(),
            "11:22:33:44:55:66".to_string(),
            None,
            9,
            None,
        )
        .unwrap();

        assert!(device.ip_address.is_none());
        assert!(device.description.is_none());
    }
}
