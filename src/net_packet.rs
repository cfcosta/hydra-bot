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
    // TODO: Implement reading methods as needed
}
