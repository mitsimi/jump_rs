use axum::Json;
use tracing::{info, instrument};
use utoipa::ToSchema;

use crate::{api::ApiResult, arp, error::ErrorResponse};

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct ArpLookupRequest {
    /// IPv4 address to look up in the ARP table
    #[schema(example = "192.168.1.100")]
    pub ip: String,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct ArpLookupResponse {
    /// MAC address found for the given IP
    #[schema(example = "00:11:22:33:44:55")]
    pub mac: String,
}

#[utoipa::path(
    post,
    path = "/api/arp-lookup",
    operation_id = "arpLookup",
    tag = "network",
    summary = "Look up MAC address by IP",
    description = "Queries the system's ARP table to find the MAC address for a given IPv4 address. The IP must have recently communicated with this host to appear in the ARP table.",
    request_body(content = ArpLookupRequest, description = "IP address to look up"),
    responses(
        (status = 200, description = "MAC address found", body = ArpLookupResponse),
        (status = 400, description = "Invalid IP address format", body = ErrorResponse),
        (status = 404, description = "IP not found in ARP table", body = ErrorResponse),
        (status = 500, description = "Error querying ARP table", body = ErrorResponse)
    )
)]
#[instrument(skip_all, fields(target_ip = %req.ip))]
pub async fn arp_lookup(Json(req): Json<ArpLookupRequest>) -> ApiResult<Json<ArpLookupResponse>> {
    let mac = arp::lookup_mac(&req.ip)?;
    info!(mac = %mac, "ARP lookup successful");
    Ok(Json(ArpLookupResponse { mac }))
}
