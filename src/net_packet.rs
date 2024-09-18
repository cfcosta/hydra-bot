use std::io::{Cursor, Read, Write};

pub struct NetPacket {
    pub data: Vec<u8>,
    pub pos: usize,
}

impl NetPacket {
    pub fn new() -> Self {
        NetPacket {
            data: Vec::new(),
            pos: 0,
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        NetPacket {
            data: bytes,
            pos: 0,
        }
    }

    // Methods for writing data to the packet
    pub fn write_u8(&mut self, value: u8) {
        self.data.push(value);
    }

    pub fn write_i8(&mut self, value: i8) {
        self.data.push(value as u8);
    }

    pub fn write_u16(&mut self, value: u16) {
        self.data.extend(&value.to_le_bytes());
    }

    pub fn write_i16(&mut self, value: i16) {
        self.data.extend(&value.to_le_bytes());
    }

    pub fn write_u32(&mut self, value: u32) {
        self.data.extend(&value.to_le_bytes());
    }

    pub fn write_i32(&mut self, value: i32) {
        self.data.extend(&value.to_le_bytes());
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend(bytes);
    }

    // Methods for reading data from the packet
    pub fn read_u8(&mut self) -> Option<u8> {
        if self.pos + 1 <= self.data.len() {
            let value = self.data[self.pos];
            self.pos += 1;
            Some(value)
        } else {
            None
        }
    }

    pub fn read_i8(&mut self) -> Option<i8> {
        self.read_u8().map(|v| v as i8)
    }

    pub fn read_u16(&mut self) -> Option<u16> {
        if self.pos + 2 <= self.data.len() {
            let bytes = &self.data[self.pos..self.pos + 2];
            let value = u16::from_le_bytes(bytes.try_into().unwrap());
            self.pos += 2;
            Some(value)
        } else {
            None
        }
    }

    pub fn read_i16(&mut self) -> Option<i16> {
        self.read_u16().map(|v| v as i16)
    }

    pub fn read_u32(&mut self) -> Option<u32> {
        if self.pos + 4 <= self.data.len() {
            let bytes = &self.data[self.pos..self.pos + 4];
            let value = u32::from_le_bytes(bytes.try_into().unwrap());
            self.pos += 4;
            Some(value)
        } else {
            None
        }
    }

    pub fn read_i32(&mut self) -> Option<i32> {
        self.read_u32().map(|v| v as i32)
    }

    pub fn read_string(&mut self) -> Option<String> {
        if let Some(pos) = self.data[self.pos..].iter().position(|&c| c == 0) {
            let bytes = &self.data[self.pos..self.pos + pos];
            self.pos += pos + 1; // Skip null terminator
            Some(String::from_utf8_lossy(bytes).into_owned())
        } else {
            None
        }
    }

    pub fn read_safe_string(&mut self) -> Option<String> {
        self.read_string().map(|s| {
            s.chars()
                .filter(|c| c.is_ascii_graphic() || c.is_whitespace())
                .collect()
        })
    }
}
