use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;
use tracing::{error, warn};

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Storage error: {0}")]
    StorageIo(#[source] std::io::Error),

    #[error("Storage data corruption: {0}")]
    StorageParse(#[source] serde_json::Error),

    #[error("Invalid MAC address format - {0}")]
    InvalidMac(String),

    #[error("Invalid IP address format: {0}")]
    InvalidIp(#[source] std::net::AddrParseError),

    #[error("Network error: {0}")]
    Network(#[source] std::io::Error),

    #[error("Failed to query ARP table: {0}")]
    ArpQuery(#[source] std::io::Error),

    #[error("Device {0} not found")]
    DeviceNotFound(String),
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::DeviceNotFound(_) => StatusCode::NOT_FOUND,
            AppError::InvalidMac(_) => StatusCode::BAD_REQUEST,
            AppError::InvalidIp(_) => StatusCode::BAD_REQUEST,
            AppError::StorageIo(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::StorageParse(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Network(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ArpQuery(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn log(&self, status: StatusCode) {
        let status_code = status.as_u16();

        match self {
            AppError::DeviceNotFound(id) => {
                warn!(
                    error_type = "device_not_found",
                    status_code = status_code,
                    device_id = %id,
                    "Request failed"
                );
            }
            AppError::InvalidMac(mac) => {
                warn!(
                    error_type = "invalid_mac",
                    status_code = status_code,
                    mac = %mac,
                    "Request failed"
                );
            }
            AppError::InvalidIp(e) => {
                warn!(
                    error_type = "invalid_ip",
                    status_code = status_code,
                    details = %e,
                    "Request failed"
                );
            }
            AppError::StorageIo(e) => {
                error!(
                    error_type = "storage_io",
                    status_code = status_code,
                    details = %e,
                    "Request failed"
                );
            }
            AppError::StorageParse(e) => {
                error!(
                    error_type = "storage_parse",
                    status_code = status_code,
                    details = %e,
                    "Request failed"
                );
            }
            AppError::Network(e) => {
                error!(
                    error_type = "network",
                    status_code = status_code,
                    details = %e,
                    "Request failed"
                );
            }
            AppError::ArpQuery(e) => {
                error!(
                    error_type = "arp_query",
                    status_code = status_code,
                    details = %e,
                    "Request failed"
                );
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status_code = self.status_code();
        self.log(status_code);

        let body = serde_json::json!({
            "status": "error",
            "message": self.to_string()
        });

        (status_code, axum::Json(body)).into_response()
    }
}
