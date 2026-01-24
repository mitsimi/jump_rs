use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

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

    fn log_level(&self) -> &str {
        match self {
            AppError::DeviceNotFound(_) => "warn",
            AppError::InvalidMac(_) => "warn",
            AppError::InvalidIp(_) => "warn",
            AppError::StorageIo(_) => "error",
            AppError::StorageParse(_) => "error",
            AppError::Network(_) => "error",
            AppError::ArpQuery(_) => "error",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status_code = self.status_code();
        let log_level = self.log_level();

        match log_level {
            "warn" => tracing::warn!("{} - {}", status_code, self),
            "error" => tracing::error!("{} - {}", status_code, self),
            _ => {}
        }

        let body = serde_json::json!({
            "status": "error",
            "message": self.to_string()
        });

        (status_code, axum::Json(body)).into_response()
    }
}
