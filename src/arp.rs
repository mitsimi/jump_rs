use std::net::Ipv4Addr;
use std::process::{Command, Output};
use thiserror::Error;
use tracing::{debug, instrument, warn};

#[derive(Debug, Error)]
pub enum ArpError {
    #[error("Invalid IP address format: {0}")]
    InvalidIp(#[from] std::net::AddrParseError),

    #[error("Failed to query ARP table: {0}")]
    Query(#[source] std::io::Error),

    #[error("MAC lookup needs direct LAN access from this runtime.")]
    NotDirectlyConnected { ip: String, route: String },

    #[error("MAC address not found for IP {0}")]
    NotFound(String),
}

impl ArpError {
    pub const fn hint(&self) -> Option<&'static str> {
        match self {
            Self::NotDirectlyConnected { .. } => Some(
                "ARP-based MAC lookup only works when jump_rs can access the target device on the same layer-2 network. Docker Desktop, OrbStack, and other VM-backed Docker runtimes may hide LAN devices even with host networking. Running jump_rs directly on the host or in a Linux host-network container usually fixes this.",
            ),
            Self::InvalidIp(_) | Self::Query(_) | Self::NotFound(_) => None,
        }
    }
}

/// Looks up the MAC address for a given IP by pinging it and checking the ARP table.
#[instrument(skip_all)]
pub fn lookup_mac(ip: &str) -> Result<String, ArpError> {
    let ip_addr: Ipv4Addr = ip.parse()?;

    ensure_direct_route(ip)?;

    if let Some(mac) = arping_ip(ip_addr)? {
        return Ok(mac);
    }

    debug!("Pinging IP to populate ARP cache");
    ping_ip(ip_addr).ok();
    get_mac_from_arp(ip)
}

fn ensure_direct_route(ip: &str) -> Result<(), ArpError> {
    let Some(output) = run_command("ip", &["route", "get", ip])? else {
        return Ok(());
    };

    if !output.status.success() {
        log_command_failure("ip route get", &output);
        return Ok(());
    }

    let route = String::from_utf8_lossy(&output.stdout);
    let route = route.lines().next().unwrap_or_default().trim();
    if is_indirect_route(route) {
        warn!(route = %route, "Target IP is not directly reachable for ARP lookup");
        return Err(ArpError::NotDirectlyConnected {
            ip: ip.to_string(),
            route: route.to_string(),
        });
    }

    Ok(())
}

#[instrument(skip_all)]
fn arping_ip(ip: Ipv4Addr) -> Result<Option<String>, ArpError> {
    debug!("Sending ARP probe");
    let ip = ip.to_string();
    let args = ["-c", "1", "-w", "1", "-f", ip.as_str()];
    let Some(output) = run_command("arping", &args)? else {
        debug!("arping command not available");
        return Ok(None);
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(mac) = parse_arping_output(&stdout, &ip) {
        debug!(mac = %mac, "MAC address found via arping");
        return Ok(Some(mac));
    }

    if !output.status.success() {
        log_command_failure("arping", &output);
    }

    Ok(None)
}

#[instrument(skip_all)]
fn ping_ip(ip: Ipv4Addr) -> Result<(), ArpError> {
    let ip = ip.to_string();
    let Some(output) = run_command("ping", &["-c", "1", "-W", "1", ip.as_str()])? else {
        debug!("ping command not available");
        return Ok(());
    };

    if output.status.success() {
        debug!("Ping successful");
    } else {
        debug!("Ping failed (host may be unreachable)");
        log_command_failure("ping", &output);
    }
    Ok(())
}

#[instrument(skip_all)]
fn get_mac_from_arp(ip: &str) -> Result<String, ArpError> {
    debug!("Querying ARP table");
    if let Some(mac) = query_arp(["-n", ip], ip)? {
        return Ok(mac);
    }

    if let Some(mac) = query_ip_neighbor(ip)? {
        return Ok(mac);
    }

    if let Some(mac) = query_arp(["-a"], ip)? {
        return Ok(mac);
    }

    warn!("MAC address not found in ARP table");
    Err(ArpError::NotFound(ip.to_string()))
}

fn query_arp<const N: usize>(args: [&str; N], ip: &str) -> Result<Option<String>, ArpError> {
    let Some(output) = run_command("arp", &args)? else {
        return Ok(None);
    };
    let output_str = String::from_utf8_lossy(&output.stdout);

    if let Some(mac) = parse_arp_output(&output_str, ip) {
        debug!(mac = %mac, "MAC address found in ARP table");
        return Ok(Some(mac));
    }

    if !output.status.success() {
        log_command_failure("arp", &output);
    }

    Ok(None)
}

fn query_ip_neighbor(ip: &str) -> Result<Option<String>, ArpError> {
    let Some(output) = run_command("ip", &["neigh", "show", ip])? else {
        return Ok(None);
    };
    let output_str = String::from_utf8_lossy(&output.stdout);

    if let Some(mac) = parse_ip_neighbor_output(&output_str, ip) {
        debug!(mac = %mac, "MAC address found in neighbor table");
        return Ok(Some(mac));
    }

    if !output.status.success() {
        log_command_failure("ip neigh show", &output);
    }

    Ok(None)
}

fn run_command(command: &str, args: &[&str]) -> Result<Option<Output>, ArpError> {
    match Command::new(command).args(args).output() {
        Ok(output) => Ok(Some(output)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(ArpError::Query(err)),
    }
}

fn log_command_failure(command: &str, output: &Output) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    debug!(
        command = command,
        status = %output.status,
        stderr = %stderr.trim(),
        stdout = %stdout.trim(),
        "ARP helper command did not return a usable result"
    );
}

fn is_indirect_route(route: &str) -> bool {
    route.split_whitespace().any(|field| field == "via")
}

fn parse_arping_output(output: &str, ip: &str) -> Option<String> {
    output
        .lines()
        .find(|line| line.contains(ip))
        .and_then(parse_bracketed_mac)
}

fn parse_bracketed_mac(line: &str) -> Option<String> {
    let start = line.find('[')?;
    let rest = line.get(start + 1..)?;
    let end = rest.find(']')?;
    normalize_mac(rest.get(..end)?)
}

fn parse_arp_output(output: &str, ip: &str) -> Option<String> {
    output.lines().find_map(|line| parse_arp_line_mac(line, ip))
}

fn parse_arp_line_mac(line: &str, ip: &str) -> Option<String> {
    if let Some(mac) = parse_bsd_arp_line(line, ip) {
        return Some(mac);
    }

    parse_linux_arp_line(line, ip)
}

fn parse_bsd_arp_line(line: &str, ip: &str) -> Option<String> {
    if !line.contains(&format!("({ip}) ")) {
        return None;
    }

    let after_at = line.split_once(" at ")?.1;
    let mac = after_at.split_whitespace().next()?;
    normalize_mac(mac)
}

fn parse_linux_arp_line(line: &str, ip: &str) -> Option<String> {
    let mut fields = line.split_whitespace();
    if fields.next()? != ip {
        return None;
    }

    let _hardware_type = fields.next()?;
    let mac = fields.next()?;
    normalize_mac(mac)
}

fn parse_ip_neighbor_output(output: &str, ip: &str) -> Option<String> {
    output
        .lines()
        .find_map(|line| parse_ip_neighbor_line(line, ip))
}

fn parse_ip_neighbor_line(line: &str, ip: &str) -> Option<String> {
    let mut fields = line.split_whitespace();
    if fields.next()? != ip {
        return None;
    }

    while let Some(field) = fields.next() {
        if field == "lladdr" {
            return fields.next().and_then(normalize_mac);
        }
    }

    None
}

fn normalize_mac(mac: &str) -> Option<String> {
    let octets: Vec<&str> = mac.split(':').collect();
    if octets.len() != 6 {
        return None;
    }

    let mut normalized = Vec::with_capacity(6);
    for octet in octets {
        if octet.is_empty() || octet.len() > 2 || !octet.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return None;
        }
        normalized.push(format!("{octet:0>2}").to_uppercase());
    }

    Some(normalized.join(":"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_darwin_padded_mac() {
        let output = "? (192.168.0.20) at 10:ff:e0:6b:65:3b on en0 ifscope [ethernet]";
        assert_eq!(
            parse_arp_output(output, "192.168.0.20").as_deref(),
            Some("10:FF:E0:6B:65:3B")
        );
    }

    #[test]
    fn parse_darwin_unpadded_mac() {
        let output = "? (192.168.0.1) at 2:10:18:50:19:8c on en0 ifscope [ethernet]";
        assert_eq!(
            parse_arp_output(output, "192.168.0.1").as_deref(),
            Some("02:10:18:50:19:8C")
        );
    }

    #[test]
    fn ignores_other_ip_entries() {
        let output = "? (192.168.0.1) at 2:10:18:50:19:8c on en0 ifscope [ethernet]";
        assert_eq!(parse_arp_output(output, "192.168.0.20"), None);
    }

    #[test]
    fn rejects_non_mac_entries() {
        let output = "? (192.168.0.20) at incomplete on en0 ifscope [ethernet]";
        assert_eq!(parse_arp_output(output, "192.168.0.20"), None);
    }

    #[test]
    fn parse_linux_net_tools_arp_entry() {
        let output = "\
Address                  HWtype  HWaddress           Flags Mask            Iface
192.168.0.20             ether   10:ff:e0:6b:65:3b   C                     eth0";
        assert_eq!(
            parse_arp_output(output, "192.168.0.20").as_deref(),
            Some("10:FF:E0:6B:65:3B")
        );
    }

    #[test]
    fn parse_linux_net_tools_unpadded_arp_entry() {
        let output = "\
Address                  HWtype  HWaddress           Flags Mask            Iface
192.168.0.1              ether   2:10:18:50:19:8c    C                     eth0";
        assert_eq!(
            parse_arp_output(output, "192.168.0.1").as_deref(),
            Some("02:10:18:50:19:8C")
        );
    }

    #[test]
    fn parse_ip_neighbor_entry() {
        let output = "192.168.0.20 dev eth0 lladdr 10:ff:e0:6b:65:3b REACHABLE";
        assert_eq!(
            parse_ip_neighbor_output(output, "192.168.0.20").as_deref(),
            Some("10:FF:E0:6B:65:3B")
        );
    }

    #[test]
    fn ignores_ip_neighbor_entry_without_mac() {
        let output = "192.168.0.20 dev eth0 FAILED";
        assert_eq!(parse_ip_neighbor_output(output, "192.168.0.20"), None);
    }

    #[test]
    fn parse_arping_reply() {
        let output = "Unicast reply from 192.168.0.20 [10:ff:e0:6b:65:3b]  1.123ms";
        assert_eq!(
            parse_arping_output(output, "192.168.0.20").as_deref(),
            Some("10:FF:E0:6B:65:3B")
        );
    }

    #[test]
    fn ignores_arping_reply_for_other_ip() {
        let output = "Unicast reply from 192.168.0.1 [10:ff:e0:6b:65:3b]  1.123ms";
        assert_eq!(parse_arping_output(output, "192.168.0.20"), None);
    }

    #[test]
    fn detects_indirect_route() {
        let route = "192.168.0.20 via 192.168.139.1 dev eth0 src 192.168.139.2 uid 1000";
        assert!(is_indirect_route(route));
    }

    #[test]
    fn accepts_direct_route() {
        let route = "192.168.0.20 dev eth0 src 192.168.0.10 uid 1000";
        assert!(!is_indirect_route(route));
    }
}
