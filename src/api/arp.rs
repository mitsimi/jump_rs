use axum::Json;
use tracing::{info, instrument};

use crate::{arp, error::AppError};

#[derive(Debug, serde::Deserialize)]
pub struct ArpLookupRequest {
    ip: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ArpLookupResponse {
    pub mac: String,
}

#[instrument(skip_all, fields(target_ip = %req.ip))]
pub async fn arp_lookup(
    Json(req): Json<ArpLookupRequest>,
) -> Result<Json<ArpLookupResponse>, AppError> {
    let mac = arp::lookup_mac(&req.ip)?;
    info!(mac = %mac, "ARP lookup successful");
    Ok(Json(ArpLookupResponse { mac }))
}
