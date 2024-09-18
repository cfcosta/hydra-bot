use std::convert::TryInto;

/// Estructura que representa un paquete de red.
pub struct NetPacket {
    pub data: Vec<u8>,
    pub pos: usize,
}

impl NetPacket {
    /// Crea un nuevo paquete de red con un tamaño inicial especificado.
    pub fn new(initial_size: usize) -> Self {
        NetPacket {
            data: Vec::with_capacity(initial_size),
            pos: 0,
        }
    }

    /// Duplica un paquete de red existente.
    pub fn dup(&self) -> Self {
        NetPacket {
            data: self.data.clone(),
            pos: self.pos,
        }
    }

    /// Libera el paquete de red.
    /// En Rust no es necesario liberar manualmente la memoria,
    /// pero se proporciona para mantener la compatibilidad con la implementación C.
    pub fn free(&mut self) {
        self.data.clear();
        self.pos = 0;
    }

    /// Escribe un byte sin signo en el paquete.
    pub fn write_u8(&mut self, value: u8) {
        self.data.push(value);
    }

    /// Escribe un byte con signo en el paquete.
    pub fn write_i8(&mut self, value: i8) {
        self.data.push(value as u8);
    }

    /// Escribe un entero de 16 bits sin signo en orden big-endian.
    pub fn write_u16(&mut self, value: u16) {
        self.data.extend(&value.to_be_bytes());
    }

    /// Escribe un entero de 16 bits con signo en orden big-endian.
    pub fn write_i16(&mut self, value: i16) {
        self.data.extend(&value.to_be_bytes());
    }

    /// Escribe un entero de 32 bits sin signo en orden big-endian.
    pub fn write_u32(&mut self, value: u32) {
        self.data.extend(&value.to_be_bytes());
    }

    /// Escribe un entero de 32 bits con signo en orden big-endian.
    pub fn write_i32(&mut self, value: i32) {
        self.data.extend(&value.to_be_bytes());
    }

    /// Escribe una cadena de caracteres en el paquete, terminada con un NUL.
    pub fn write_string(&mut self, string: &str) {
        self.data.extend(string.as_bytes());
        self.data.push(0); // Terminador NUL
    }

    /// Lee un byte sin signo del paquete.
    pub fn read_u8(&mut self) -> Option<u8> {
        if self.pos < self.data.len() {
            let value = self.data[self.pos];
            self.pos += 1;
            Some(value)
        } else {
            None
        }
    }

    /// Lee un byte con signo del paquete.
    pub fn read_i8(&mut self) -> Option<i8> {
        self.read_u8().map(|v| v as i8)
    }

    /// Lee un entero de 16 bits sin signo en orden big-endian del paquete.
    pub fn read_u16(&mut self) -> Option<u16> {
        if self.pos + 2 <= self.data.len() {
            let bytes = &self.data[self.pos..self.pos + 2];
            self.pos += 2;
            Some(u16::from_be_bytes(bytes.try_into().unwrap()))
        } else {
            None
        }
    }

    /// Lee un entero de 16 bits con signo en orden big-endian del paquete.
    pub fn read_i16(&mut self) -> Option<i16> {
        self.read_u16().map(|v| v as i16)
    }

    /// Lee un entero de 32 bits sin signo en orden big-endian del paquete.
    pub fn read_u32(&mut self) -> Option<u32> {
        if self.pos + 4 <= self.data.len() {
            let bytes = &self.data[self.pos..self.pos + 4];
            self.pos += 4;
            Some(u32::from_be_bytes(bytes.try_into().unwrap()))
        } else {
            None
        }
    }

    /// Lee un entero de 32 bits con signo en orden big-endian del paquete.
    pub fn read_i32(&mut self) -> Option<i32> {
        self.read_u32().map(|v| v as i32)
    }

    /// Lee una cadena de caracteres del paquete.
    /// Retorna `None` si no se encuentra un terminador NUL antes del final del paquete.
    pub fn read_string(&mut self) -> Option<String> {
        if let Some(terminator) = self.data[self.pos..].iter().position(|&c| c == 0) {
            let bytes = &self.data[self.pos..self.pos + terminator];
            let string = String::from_utf8_lossy(bytes).into_owned();
            self.pos += terminator + 1; // Salta el terminador NUL
            Some(string)
        } else {
            None
        }
    }

    /// Lee una cadena de caracteres segura del paquete, filtrando caracteres no imprimibles.
    pub fn read_safe_string(&mut self) -> Option<String> {
        self.read_string().map(|s| {
            s.chars()
                .filter(|c| c.is_ascii_graphic() || c.is_whitespace())
                .collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_and_read_u8() {
        let mut packet = NetPacket::new(10);
        packet.write_u8(255);
        packet.pos = 0;
        assert_eq!(packet.read_u8(), Some(255));
    }

    #[test]
    fn test_write_and_read_i16() {
        let mut packet = NetPacket::new(10);
        packet.write_i16(-12345);
        packet.pos = 0;
        assert_eq!(packet.read_i16(), Some(-12345));
    }

    #[test]
    fn test_write_and_read_string() {
        let mut packet = NetPacket::new(10);
        packet.write_string("Hello");
        packet.pos = 0;
        assert_eq!(packet.read_string(), Some("Hello".to_string()));
    }

    #[test]
    fn test_read_safe_string() {
        let mut packet = NetPacket::new(10);
        packet.write_string("Hello\x00World");
        packet.pos = 0;
        assert_eq!(packet.read_safe_string(), Some("Hello".to_string()));
    }

    #[test]
    fn test_dup_packet() {
        let mut packet = NetPacket::new(10);
        packet.write_u8(100);
        let dup = packet.dup();
        assert_eq!(dup.read_u8(), Some(100));
    }
}
