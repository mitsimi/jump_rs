use crate::error::AppError;
use regex::Regex;
use std::net::Ipv4Addr;
use std::process::Command;
use tracing::{debug, instrument, warn};

/// Looks up the MAC address for a given IP by pinging it and checking the ARP table.
#[instrument(skip_all)]
pub fn lookup_mac(ip: &str) -> Result<String, AppError> {
    let ip_addr: Ipv4Addr = ip.parse().map_err(AppError::InvalidIp)?;

    debug!("Pinging IP to populate ARP cache");
    ping_ip(ip_addr).ok();

    get_mac_from_arp(ip)
}

#[instrument(skip_all)]
fn ping_ip(ip: Ipv4Addr) -> Result<(), AppError> {
    let output = Command::new("ping")
        .args(["-c", "1", "-W", "1", ip.to_string().as_str()])
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                debug!("Ping successful");
            } else {
                debug!("Ping failed (host may be unreachable)");
            }
            Ok(())
        }
        Err(e) => {
            warn!(error = %e, "Ping command failed to execute");
            Ok(())
        }
    }
}

#[instrument(skip_all)]
fn get_mac_from_arp(ip: &str) -> Result<String, AppError> {
    debug!("Querying ARP table");
    let output = Command::new("arp")
        .args(["-a"])
        .output()
        .map_err(AppError::ArpQuery)?;

    let output_str = String::from_utf8_lossy(&output.stdout);

    let ip_pattern = format!(r"\? \({}\) at ([0-9A-Fa-f:]{{17}})", regex::escape(ip));
    let ip_mac_pattern = Regex::new(&ip_pattern).unwrap();

    if let Some(caps) = ip_mac_pattern.captures(&output_str)
        && let Some(mac_match) = caps.get(1)
    {
        let mac = mac_match.as_str().to_uppercase();
        debug!(mac = %mac, "MAC address found in ARP table");
        return Ok(mac);
    }

    warn!("MAC address not found in ARP table");
    Err(AppError::DeviceNotFound(ip.to_string()))
}
