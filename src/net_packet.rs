use serde::{Serialize, Deserialize};
use std::convert::TryInto;

/// Structure that represents a network packet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetPacket {
    pub data: Vec<u8>,
    pub pos: usize,
}

impl NetPacket {
    /// Creates a new network packet with a specified initial size.
    pub fn new() -> Self {
        NetPacket {
            data: Vec::new(),
            pos: 0,
        }
    }

    /// Writes an unsigned 8-bit integer to the packet.
    pub fn write_u8(&mut self, value: u8) {
        self.data.push(value);
    }

    /// Writes a signed 8-bit integer to the packet.
    pub fn write_i8(&mut self, value: i8) {
        self.data.push(value as u8);
    }

    /// Writes an unsigned 16-bit integer in big-endian order to the packet.
    pub fn write_u16(&mut self, value: u16) {
        self.data.extend(&value.to_be_bytes());
    }

    /// Writes a signed 16-bit integer in big-endian order to the packet.
    pub fn write_i16(&mut self, value: i16) {
        self.data.extend(&value.to_be_bytes());
    }

    /// Writes an unsigned 32-bit integer in big-endian order to the packet.
    pub fn write_u32(&mut self, value: u32) {
        self.data.extend(&value.to_be_bytes());
    }

    /// Writes a signed 32-bit integer in big-endian order to the packet.
    pub fn write_i32(&mut self, value: i32) {
        self.data.extend(&value.to_be_bytes());
    }

    /// Writes a string to the packet, terminated with a NUL byte.
    pub fn write_string(&mut self, string: &str) {
        self.data.extend(string.as_bytes());
        self.data.push(0); // NUL terminator
    }

    /// Reads an unsigned 8-bit integer from the packet.
    pub fn read_u8(&mut self) -> Option<u8> {
        if self.pos < self.data.len() {
            let value = self.data[self.pos];
            self.pos += 1;
            Some(value)
        } else {
            None
        }
    }

    /// Reads a signed 8-bit integer from the packet.
    pub fn read_i8(&mut self) -> Option<i8> {
        self.read_u8().map(|v| v as i8)
    }

    /// Reads an unsigned 16-bit integer in big-endian order from the packet.
    pub fn read_u16(&mut self) -> Option<u16> {
        if self.pos + 2 <= self.data.len() {
            let bytes = &self.data[self.pos..self.pos + 2];
            self.pos += 2;
            Some(u16::from_be_bytes(bytes.try_into().unwrap()))
        } else {
            None
        }
    }

    /// Reads a signed 16-bit integer in big-endian order from the packet.
    pub fn read_i16(&mut self) -> Option<i16> {
        self.read_u16().map(|v| v as i16)
    }

    /// Reads an unsigned 32-bit integer in big-endian order from the packet.
    pub fn read_u32(&mut self) -> Option<u32> {
        if self.pos + 4 <= self.data.len() {
            let bytes = &self.data[self.pos..self.pos + 4];
            self.pos += 4;
            Some(u32::from_be_bytes(bytes.try_into().unwrap()))
        } else {
            None
        }
    }

    /// Reads a signed 32-bit integer in big-endian order from the packet.
    pub fn read_i32(&mut self) -> Option<i32> {
        self.read_u32().map(|v| v as i32)
    }

    /// Reads a string from the packet.
    /// Returns `None` if a terminating NUL byte is not found before the end of the packet.
    pub fn read_string(&mut self) -> Option<String> {
        if let Some(terminator) = self.data[self.pos..].iter().position(|&c| c == 0) {
            let bytes = &self.data[self.pos..self.pos + terminator];
            let string = String::from_utf8_lossy(bytes).into_owned();
            self.pos += terminator + 1; // Skip the NUL terminator
            Some(string)
        } else {
            None
        }
    }

    /// Reads a safe string from the packet, filtering out non-printable characters.
    /// Returns `None` if a terminating NUL byte is not found.
    pub fn read_safe_string(&mut self) -> Option<String> {
        self.read_string().map(|s| {
            s.chars()
                .filter(|c| c.is_ascii_graphic() || c.is_whitespace())
                .collect()
        })
    }

    /// Resets the reading position to the beginning of the packet.
    pub fn reset(&mut self) {
        self.pos = 0;
    }

    /// Reads a protocol from the packet.
    pub fn read_protocol(&mut self) -> NetProtocol {
        // Placeholder implementation
        NetProtocol::Unknown
    }

    /// Writes a protocol list to the packet.
    pub fn write_protocol_list(&mut self) {
        // Placeholder implementation
    }

    /// Writes connect data to the packet.
    pub fn write_connect_data(&mut self, _data: &ConnectData) {
        // Placeholder implementation
    }

    /// Reads wait data from the packet.
    pub fn read_wait_data(&mut self) -> Option<NetWaitData> {
        // Placeholder implementation
        None
    }

    /// Reads settings from the packet.
    pub fn read_settings(&mut self) -> Option<GameSettings> {
        // Placeholder implementation
        None
    }

    /// Writes settings to the packet.
    pub fn write_settings(&mut self, _settings: &GameSettings) {
        // Placeholder implementation
    }

    /// Reads a full ticcmd from the packet.
    pub fn read_full_ticcmd(&mut self) -> Option<NetFullTicCmd> {
        // Placeholder implementation
        None
    }

    /// Writes a ticcmd diff to the packet.
    pub fn write_ticcmd_diff(&mut self, _diff: &NetTicDiff) {
        // Placeholder implementation
    }
}

use crate::net_structs::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_and_read_u8() {
        let mut packet = NetPacket::new();
        packet.write_u8(255);
        packet.reset();
        assert_eq!(packet.read_u8(), Some(255));
    }

    #[test]
    fn test_write_and_read_i8() {
        let mut packet = NetPacket::new();
        packet.write_i8(-128);
        packet.reset();
        assert_eq!(packet.read_i8(), Some(-128));
    }

    #[test]
    fn test_write_and_read_u16() {
        let mut packet = NetPacket::new();
        packet.write_u16(65535);
        packet.reset();
        assert_eq!(packet.read_u16(), Some(65535));
    }

    #[test]
    fn test_write_and_read_i16() {
        let mut packet = NetPacket::new();
        packet.write_i16(-12345);
        packet.reset();
        assert_eq!(packet.read_i16(), Some(-12345));
    }

    #[test]
    fn test_write_and_read_u32() {
        let mut packet = NetPacket::new();
        packet.write_u32(4294967295);
        packet.reset();
        assert_eq!(packet.read_u32(), Some(4294967295));
    }

    #[test]
    fn test_write_and_read_i32() {
        let mut packet = NetPacket::new();
        packet.write_i32(-123456789);
        packet.reset();
        assert_eq!(packet.read_i32(), Some(-123456789));
    }

    #[test]
    fn test_write_and_read_string() {
        let mut packet = NetPacket::new();
        packet.write_string("Hello");
        packet.reset();
        assert_eq!(packet.read_string(), Some("Hello".to_string()));
    }

    #[test]
    fn test_write_and_read_safe_string() {
        let mut packet = NetPacket::new();
        packet.write_string("Hello\x00World\x1F!");
        packet.reset();
        assert_eq!(packet.read_safe_string(), Some("Hello".to_string()));
    }

    #[test]
    fn test_reset_position() {
        let mut packet = NetPacket::new();
        packet.write_u8(1);
        packet.write_u8(2);
        packet.reset();
        assert_eq!(packet.read_u8(), Some(1));
    }

    #[test]
    fn test_debug_trait() {
        let mut packet = NetPacket::new();
        packet.write_u8(0xAB);
        packet.write_u8(0xCD);
        packet.write_u8(0xEF);
        packet.reset();
        let debug_str = format!("{:?}", packet);
        assert_eq!(debug_str, "NetPacket { data: \"AB CD EF\", pos: 0 }");
    }
}
