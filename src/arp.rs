use crate::error::AppError;
use regex::Regex;
use std::net::Ipv4Addr;
use std::process::Command;

pub fn lookup_mac(ip: &str) -> Result<Option<String>, AppError> {
    let ip_addr: Ipv4Addr = ip.parse().map_err(AppError::InvalidIp)?;

    ping_ip(ip_addr).ok();

    get_mac_from_arp(ip)
}

fn ping_ip(ip: Ipv4Addr) -> Result<(), AppError> {
    let output = Command::new("ping")
        .args(["-c", "1", "-W", "1", ip.to_string().as_str()])
        .output();

    match output {
        Ok(_) => Ok(()),
        Err(_) => Ok(()),
    }
}

fn get_mac_from_arp(ip: &str) -> Result<Option<String>, AppError> {
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
        return Ok(Some(mac));
    }

    Ok(None)
}
