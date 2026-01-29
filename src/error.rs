use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;
use tracing::{error, warn};

use crate::arp::ArpError;
use crate::models::ValidationError;
use crate::storage::StorageError;
use crate::wol::WolError;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error(transparent)]
    Validation(#[from] ValidationError),

    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error(transparent)]
    Wol(#[from] WolError),

    #[error(transparent)]
    Arp(#[from] ArpError),
}

impl ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Validation(_) => StatusCode::BAD_REQUEST,

            ApiError::Storage(e) => match e {
                StorageError::NotFound(_) => StatusCode::NOT_FOUND,
                StorageError::Io(_) | StorageError::Parse(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },

            ApiError::Wol(e) => match e {
                WolError::InvalidMac(_) => StatusCode::BAD_REQUEST,
                WolError::Network(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },

            ApiError::Arp(e) => match e {
                ArpError::InvalidIp(_) => StatusCode::BAD_REQUEST,
                ArpError::NotFound(_) => StatusCode::NOT_FOUND,
                ArpError::Query(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
        }
    }

    fn log(&self, status: StatusCode) {
        let status_code = status.as_u16();

        match self {
            ApiError::Validation(e) => {
                warn!(
                    error_type = "validation",
                    status_code = status_code,
                    details = %e,
                    "Request failed"
                );
            }
            ApiError::Storage(e) => match e {
                StorageError::NotFound(id) => {
                    warn!(
                        error_type = "storage_not_found",
                        status_code = status_code,
                        device_id = %id,
                        "Request failed"
                    );
                }
                StorageError::Io(err) => {
                    error!(
                        error_type = "storage_io",
                        status_code = status_code,
                        details = %err,
                        "Request failed"
                    );
                }
                StorageError::Parse(err) => {
                    error!(
                        error_type = "storage_parse",
                        status_code = status_code,
                        details = %err,
                        "Request failed"
                    );
                }
            },
            ApiError::Wol(e) => match e {
                WolError::InvalidMac(mac) => {
                    warn!(
                        error_type = "wol_invalid_mac",
                        status_code = status_code,
                        mac = %mac,
                        "Request failed"
                    );
                }
                WolError::Network(err) => {
                    error!(
                        error_type = "wol_network",
                        status_code = status_code,
                        details = %err,
                        "Request failed"
                    );
                }
            },
            ApiError::Arp(e) => match e {
                ArpError::InvalidIp(err) => {
                    warn!(
                        error_type = "arp_invalid_ip",
                        status_code = status_code,
                        details = %err,
                        "Request failed"
                    );
                }
                ArpError::NotFound(ip) => {
                    warn!(
                        error_type = "arp_not_found",
                        status_code = status_code,
                        ip = %ip,
                        "Request failed"
                    );
                }
                ArpError::Query(err) => {
                    error!(
                        error_type = "arp_query",
                        status_code = status_code,
                        details = %err,
                        "Request failed"
                    );
                }
            },
        }
    }
}

impl IntoResponse for ApiError {
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
