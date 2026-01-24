use axum::Json;

use crate::{arp, error::AppError};

#[derive(Debug, serde::Deserialize)]
pub struct ArpLookupRequest {
    ip: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ArpLookupResponse {
    pub mac: String,
}

pub async fn arp_lookup(
    Json(req): Json<ArpLookupRequest>,
) -> Result<Json<ArpLookupResponse>, AppError> {
    let mac = arp::lookup_mac(&req.ip)?;
    Ok(Json(ArpLookupResponse { mac }))
}
