mod arp;
mod devices;
mod wol;

pub use arp::*;
pub use devices::*;
pub use wol::wake_device;

use crate::storage::SharedStorage;
use axum::{
    Router,
    routing::{get, post, put},
};

/// Creates and configures the API router with all API routes
pub fn router() -> Router<SharedStorage> {
    Router::new()
        .route("/api/devices", get(get_devices).post(create_device))
        .route("/api/devices/export", get(export_devices))
        .route("/api/devices/import", post(import_devices))
        .route(
            "/api/devices/{id}",
            put(update_device).delete(delete_device),
        )
        .route("/api/devices/{id}/wake", post(wake_device))
        .route("/api/arp-lookup", post(arp_lookup))
}
