use crate::models::Device;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use thiserror::Error;
use tracing::{debug, instrument};

#[derive(Debug, Error)]
pub enum WolError {
    #[error("Invalid MAC address: {0}")]
    InvalidMac(String),

    #[error("Network error: {0}")]
    Network(#[source] std::io::Error),
}

#[instrument(skip_all)]
pub fn send_wol_packet(device: &Device) -> Result<(), WolError> {
    let mac = parse_mac_address(&device.mac_address).map_err(WolError::InvalidMac)?;

    let magic_packet = create_magic_packet(mac);

    let socket = UdpSocket::bind("0.0.0.0:0").map_err(WolError::Network)?;

    let target_addr = SocketAddr::new(Ipv4Addr::BROADCAST.into(), device.port);

    socket.set_broadcast(true).map_err(WolError::Network)?;

    socket
        .send_to(&magic_packet, target_addr)
        .map_err(WolError::Network)?;

    debug!(target_addr = %target_addr, packet_size = magic_packet.len(), "Magic packet sent");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mac_with_colons() {
        let result = parse_mac_address("AA:BB:CC:DD:EE:FF");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    }

    #[test]
    fn parse_mac_with_dashes() {
        let result = parse_mac_address("AA-BB-CC-DD-EE-FF");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    }

    #[test]
    fn parse_mac_with_dots() {
        let result = parse_mac_address("AABB.CCDD.EEFF");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    }

    #[test]
    fn parse_mac_without_separators() {
        let result = parse_mac_address("AABBCCDDEEFF");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    }

    #[test]
    fn parse_mac_lowercase() {
        let result = parse_mac_address("aa:bb:cc:dd:ee:ff");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    }

    #[test]
    fn parse_mac_mixed_case() {
        let result = parse_mac_address("Aa:Bb:Cc:Dd:Ee:Ff");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    }

    #[test]
    fn parse_mac_all_zeros() {
        let result = parse_mac_address("00:00:00:00:00:00");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn parse_mac_all_ones() {
        let result = parse_mac_address("FF:FF:FF:FF:FF:FF");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn parse_mac_too_short_fails() {
        let result = parse_mac_address("AA:BB:CC:DD:EE");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("12 hex digits"));
    }

    #[test]
    fn parse_mac_too_long_fails() {
        let result = parse_mac_address("AA:BB:CC:DD:EE:FF:00");
        assert!(result.is_err());
    }

    #[test]
    fn parse_mac_empty_fails() {
        let result = parse_mac_address("");
        assert!(result.is_err());
    }

    #[test]
    fn parse_mac_invalid_hex_fails() {
        let result = parse_mac_address("GG:HH:II:JJ:KK:LL");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid hex"));
    }

    #[test]
    fn magic_packet_has_correct_size() {
        let mac = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        let packet = create_magic_packet(mac);
        assert_eq!(packet.len(), 102);
    }

    #[test]
    fn magic_packet_starts_with_six_ff_bytes() {
        let mac = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        let packet = create_magic_packet(mac);

        // First 6 bytes should be 0xFF
        for (i, byte) in packet.iter().enumerate().take(6) {
            assert_eq!(*byte, 0xFF, "Byte {i} should be 0xFF");
        }
    }

    #[test]
    fn magic_packet_contains_mac_16_times() {
        let mac = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        let packet = create_magic_packet(mac);

        // After the 6 header bytes, MAC should repeat 16 times
        for repetition in 0..16 {
            let offset = 6 + (repetition * 6);
            for (byte_idx, expected_byte) in mac.iter().enumerate() {
                assert_eq!(
                    packet[offset + byte_idx],
                    *expected_byte,
                    "MAC repetition {repetition} byte {byte_idx} mismatch",
                );
            }
        }
    }

    #[test]
    fn magic_packet_with_zero_mac() {
        let mac = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let packet = create_magic_packet(mac);

        assert_eq!(packet.len(), 102);
        // Header should still be 0xFF
        for byte in packet.iter().take(6) {
            assert_eq!(*byte, 0xFF);
        }
        // Rest should be 0x00
        for byte in packet.iter().skip(6) {
            assert_eq!(*byte, 0x00);
        }
    }

    #[test]
    fn magic_packet_with_broadcast_mac() {
        let mac = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
        let packet = create_magic_packet(mac);

        assert_eq!(packet.len(), 102);
        // Entire packet should be 0xFF
        for (i, byte) in packet.iter().enumerate() {
            assert_eq!(*byte, 0xFF, "Byte {i} should be 0xFF");
        }
    }

    // Test against the WoL magic packet specification
    // Verify structure: 6 bytes of 0xFF + 16 repetitions of MAC
    #[test]
    fn magic_packet_structure_matches_spec() {
        let mac = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
        let packet = create_magic_packet(mac);

        let expected_header = vec![0xFF; 6];
        let expected_mac_section: Vec<u8> = (0..16).flat_map(|_| mac.iter().copied()).collect();

        let mut expected = expected_header;
        expected.extend(expected_mac_section);

        assert_eq!(packet, expected);
    }
}
