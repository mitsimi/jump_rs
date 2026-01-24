use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use tracing::instrument;

use crate::{error::AppError, storage::SharedStorage, wol};

#[instrument(skip(storage), fields(id = id))]
pub async fn wake_device(
    State(storage): State<SharedStorage>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let device = storage.get(&id).ok_or(AppError::DeviceNotFound(id))?;

    wol::send_wol_packet(&device)?;

    Ok(StatusCode::NO_CONTENT)
}
