use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use tracing::{info, instrument};

use crate::{error::AppError, storage::SharedStorage, wol};

#[instrument(skip_all, fields(device_id = %id))]
pub async fn wake_device(
    State(storage): State<SharedStorage>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let device = storage.get(&id).ok_or(AppError::DeviceNotFound(id))?;

    wol::send_wol_packet(&device)?;

    info!(device_name = %device.name, mac = %device.mac_address, "WoL packet sent");
    Ok(StatusCode::NO_CONTENT)
}
