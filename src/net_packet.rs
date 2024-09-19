use serde::{Serialize, Deserialize};
use std::convert::TryInto;
use std::io::{self, Read, Write};
use std::net::UdpSocket;

use crate::net_structs::*;

/// Structure that represents a network packet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetPacket {
    pub data: Vec<u8>,
    pub pos: usize,
}

impl NetPacket {
    /// Creates a new network packet.
    pub fn new() -> Self {
        NetPacket {
            data: Vec::new(),
            pos: 0,
        }
    }

    /// Reads a ticcmd diff from the packet.
    pub fn read_ticcmd_diff(&mut self, lowres_turn: bool) -> Option<NetTicDiff> {
        let mut diff = NetTicDiff::default();
        diff.diff = self.read_u8()? as u32;

        if diff.diff & NET_TICDIFF_FORWARD != 0 {
            diff.cmd.forwardmove = self.read_i8()?;
        }

        if diff.diff & NET_TICDIFF_SIDE != 0 {
            diff.cmd.sidemove = self.read_i8()?;
        }

        if diff.diff & NET_TICDIFF_TURN != 0 {
            if lowres_turn {
                diff.cmd.angleturn = (self.read_i8()? as i16) * 256;
            } else {
                diff.cmd.angleturn = self.read_i16()?;
            }
        }

        if diff.diff & NET_TICDIFF_BUTTONS != 0 {
            diff.cmd.buttons = self.read_u8()?;
        }

        if diff.diff & NET_TICDIFF_CONSISTANCY != 0 {
            diff.cmd.consistancy = self.read_u8()?;
        }

        if diff.diff & NET_TICDIFF_CHATCHAR != 0 {
            diff.cmd.chatchar = self.read_u8()?;
        } else {
            diff.cmd.chatchar = 0;
        }

        if diff.diff & NET_TICDIFF_RAVEN != 0 {
            diff.cmd.lookfly = self.read_u8()?;
            diff.cmd.arti = self.read_u8()?;
        } else {
            diff.cmd.arti = 0;
        }

        if diff.diff & NET_TICDIFF_STRIFE != 0 {
            diff.cmd.buttons2 = self.read_u8()?;
            diff.cmd.inventory = self.read_i16()? as i32;
        } else {
            diff.cmd.inventory = 0;
        }

        Some(diff)
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
        self.data.extend_from_slice(string.as_bytes());
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

    /// Resets the reading position to the beginning of the packet.
    pub fn reset(&mut self) {
        self.pos = 0;
    }

    /// Writes connect data to the packet.
    pub fn write_connect_data(&mut self, data: &ConnectData) {
        self.write_u8(data.gamemode as u8);
        self.write_u8(data.gamemission as u8);
        self.write_u8(data.lowres_turn as u8);
        self.write_u8(data.drone as u8);
        self.write_u8(data.max_players as u8);
        self.write_u8(data.is_freedoom as u8);
        self.data.extend_from_slice(&data.wad_sha1sum);
        self.data.extend_from_slice(&data.deh_sha1sum);
        self.write_u8(data.player_class as u8);
    }

    /// Reads wait data from the packet.
    pub fn read_wait_data(&mut self) -> Option<NetWaitData> {
        let mut data = NetWaitData::default();
        data.num_players = self.read_u8()? as i32;
        data.num_drones = self.read_u8()? as i32;
        data.ready_players = self.read_u8()? as i32;
        data.max_players = self.read_u8()? as i32;
        data.is_controller = self.read_u8()? as i32;
        data.consoleplayer = self.read_i8()? as i32;
        for i in 0..data.num_players as usize {
            let name = self.read_string()?;
            if name.len() >= MAXPLAYERNAME {
                return None;
            }
            data.player_names[i] = ['\0'; MAXPLAYERNAME];
            for (j, c) in name.chars().enumerate().take(MAXPLAYERNAME) {
                data.player_names[i][j] = c;
            }
            let addr = self.read_string()?;
            if addr.len() >= MAXPLAYERNAME {
                return None;
            }
            data.player_addrs[i] = ['\0'; MAXPLAYERNAME];
            for (j, c) in addr.chars().enumerate().take(MAXPLAYERNAME) {
                data.player_addrs[i][j] = c;
            }
        }
        self.data[self.pos..self.pos + 20].copy_from_slice(&mut data.wad_sha1sum);
        self.pos += 20;
        self.data[self.pos..self.pos + 20].copy_from_slice(&mut data.deh_sha1sum);
        self.pos += 20;
        data.is_freedoom = self.read_u8()? as i32;
        Some(data)
    }

    /// Reads settings from the packet.
    pub fn read_settings(&mut self) -> Option<GameSettings> {
        let mut settings = GameSettings::default();
        settings.ticdup = self.read_u8()? as i32;
        settings.extratics = self.read_u8()? as i32;
        settings.deathmatch = self.read_u8()? as i32;
        settings.nomonsters = self.read_u8()? as i32;
        settings.fast_monsters = self.read_u8()? as i32;
        settings.respawn_monsters = self.read_u8()? as i32;
        settings.episode = self.read_u8()? as i32;
        settings.map = self.read_u8()? as i32;
        settings.skill = self.read_i8()? as i32;
        settings.gameversion = self.read_u8()? as i32;
        settings.lowres_turn = self.read_u8()? as i32;
        settings.new_sync = self.read_u8()? as i32;
        settings.timelimit = self.read_u32()?;
        settings.loadgame = self.read_i8()? as i32;
        settings.random = self.read_u8()? as i32;
        settings.num_players = self.read_u8()? as i32;
        settings.consoleplayer = self.read_i8()? as i32;
        for i in 0..settings.num_players as usize {
            settings.player_classes[i] = self.read_u8()? as i32;
        }
        Some(settings)
    }

    /// Writes settings to the packet.
    pub fn write_settings(&mut self, settings: &GameSettings) {
        self.write_u8(settings.ticdup as u8);
        self.write_u8(settings.extratics as u8);
        self.write_u8(settings.deathmatch as u8);
        self.write_u8(settings.nomonsters as u8);
        self.write_u8(settings.fast_monsters as u8);
        self.write_u8(settings.respawn_monsters as u8);
        self.write_u8(settings.episode as u8);
        self.write_u8(settings.map as u8);
        self.write_i8(settings.skill as i8);
        self.write_u8(settings.gameversion as u8);
        self.write_u8(settings.lowres_turn as u8);
        self.write_u8(settings.new_sync as u8);
        self.write_u32(settings.timelimit);
        self.write_i8(settings.loadgame as i8);
        self.write_u8(settings.random as u8);
        self.write_u8(settings.num_players as u8);
        self.write_i8(settings.consoleplayer as i8);
        for i in 0..settings.num_players as usize {
            self.write_u8(settings.player_classes[i] as u8);
        }
    }

    /// Reads a full ticcmd from the packet.
    pub fn read_full_ticcmd(&mut self, lowres_turn: bool) -> Option<NetFullTicCmd> {
        let mut cmd = NetFullTicCmd::default();
        cmd.latency = self.read_i16()? as i32;

        let bitfield = self.read_u8()?;
        for i in 0..NET_MAXPLAYERS {
            cmd.playeringame[i] = (bitfield & (1 << i)) != 0;
        }

        for i in 0..NET_MAXPLAYERS {
            if cmd.playeringame[i] {
                cmd.cmds[i] = self.read_ticcmd_diff(lowres_turn)?;
            }
        }
        Some(cmd)
    }

    /// Writes a ticcmd diff to the packet.
    pub fn write_ticcmd_diff(&mut self, diff: &NetTicDiff, lowres_turn: bool) {
        self.write_u8(diff.diff as u8);

        if diff.diff & NET_TICDIFF_FORWARD != 0 {
            self.write_i8(diff.cmd.forwardmove);
        }

        if diff.diff & NET_TICDIFF_SIDE != 0 {
            self.write_i8(diff.cmd.sidemove);
        }

        if diff.diff & NET_TICDIFF_TURN != 0 {
            if lowres_turn {
                self.write_i8((diff.cmd.angleturn / 256) as i8);
            } else {
                self.write_i16(diff.cmd.angleturn);
            }
        }

        if diff.diff & NET_TICDIFF_BUTTONS != 0 {
            self.write_u8(diff.cmd.buttons);
        }

        if diff.diff & NET_TICDIFF_CONSISTANCY != 0 {
            self.write_u8(diff.cmd.consistancy);
        }

        if diff.diff & NET_TICDIFF_CHATCHAR != 0 {
            self.write_u8(diff.cmd.chatchar);
        }

        if diff.diff & NET_TICDIFF_RAVEN != 0 {
            self.write_u8(diff.cmd.lookfly);
            self.write_u8(diff.cmd.arti);
        }

        if diff.diff & NET_TICDIFF_STRIFE != 0 {
            self.write_u8(diff.cmd.buttons2);
            self.write_i16(diff.cmd.inventory as i16);
        }
    }

    /// Sends the packet over UDP.
    pub fn send(&self, socket: &UdpSocket, addr: &std::net::SocketAddr) -> io::Result<usize> {
        socket.send_to(&self.data, addr)
    }

    /// Receives a packet over UDP.
    pub fn receive(socket: &UdpSocket) -> io::Result<(Self, std::net::SocketAddr)> {
        let mut buf = [0u8; 1024]; // Adjust buffer size as needed
        let (size, src) = socket.recv_from(&mut buf)?;
        let mut packet = NetPacket::new();
        packet.data.extend_from_slice(&buf[..size]);
        Ok((packet, src))
    }
}

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
    fn test_reset_position() {
        let mut packet = NetPacket::new();
        packet.write_u8(1);
        packet.write_u8(2);
        packet.reset();
        assert_eq!(packet.read_u8(), Some(1));
    }
}
