use axum::{Json, Router, routing::post};
use tracing::{info, instrument};
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, error::ErrorResponse};

#[derive(OpenApi)]
#[openapi(
    paths(
        arp_lookup,
    ),
    components(
        schemas(
            ArpLookupRequest,
            ArpLookupResponse,
            crate::error::ErrorResponse,
        )
    ),
    tags(
        (name = "network", description = "Network utility endpoints")
    )
)]
pub struct NetworkApiDoc;

pub fn router() -> Router {
    Router::new().route("/api/arp-lookup", post(arp_lookup))
}

#[derive(Debug, serde::Deserialize, ToSchema)]
pub struct ArpLookupRequest {
    /// IPv4 address or hostname to resolve and look up in the ARP table
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
    summary = "Look up MAC address by IP or hostname",
    description = "Resolves an IPv4 address or hostname using the server's system resolver, then probes the first IPv4 address and queries the ARP table. The target must be on the same layer-2 network as the server.",
    request_body(content = ArpLookupRequest, description = "IPv4 address or hostname to look up (in the ip field)"),
    responses(
        (status = 200, description = "MAC address found", body = ArpLookupResponse),
        (status = 400, description = "Hostname resolution failed or no IPv4 address available", body = ErrorResponse),
        (status = 404, description = "IP not found in ARP table", body = ErrorResponse),
        (status = 500, description = "Error querying ARP table", body = ErrorResponse)
    )
)]
#[instrument(skip_all, fields(target_ip = %req.ip))]
pub async fn arp_lookup(Json(req): Json<ArpLookupRequest>) -> ApiResult<Json<ArpLookupResponse>> {
    let mac = crate::devices::arp_lookup(&req.ip)?;
    info!(mac = %mac, "ARP lookup successful");
    Ok(Json(ArpLookupResponse { mac }))
}
