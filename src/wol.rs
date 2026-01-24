use crate::error::AppError;
use crate::models::Device;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};

pub fn send_wol_packet(device: &Device) -> Result<(), AppError> {
    let mac = parse_mac_address(&device.mac_address).map_err(AppError::InvalidMac)?;

    let magic_packet = create_magic_packet(mac);

    let socket = UdpSocket::bind("0.0.0.0:0").map_err(AppError::Network)?;

    let target_addr = SocketAddr::new(Ipv4Addr::BROADCAST.into(), device.port);

    socket.set_broadcast(true).map_err(AppError::Network)?;

    socket
        .send_to(&magic_packet, target_addr)
        .map_err(AppError::Network)?;

    Ok(())
}

fn parse_mac_address(mac_str: &str) -> Result<[u8; 6], String> {
    let cleaned: String = mac_str.replace([':', '-', '.', ' '], "").to_lowercase();

    if cleaned.len() != 12 {
        return Err("MAC address must be 12 hex digits".to_string());
    }

    let mut mac = [0u8; 6];
    for (i, chunk) in cleaned.as_bytes().chunks(2).enumerate() {
        if i >= 6 {
            return Err("MAC address too long".to_string());
        }
        mac[i] = u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16)
            .map_err(|_| "Invalid hex digit".to_string())?;
    }

    Ok(mac)
}

fn create_magic_packet(mac: [u8; 6]) -> Vec<u8> {
    let mut packet = Vec::with_capacity(102);

    packet.extend(std::iter::repeat_n(0xFF, 6));

    for _ in 0..16 {
        packet.extend_from_slice(&mac);
    }

    packet
}
