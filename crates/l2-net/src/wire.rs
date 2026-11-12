use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WireError {
    #[error("buffer curto")]
    Short,
    #[error("string UTF-16 inválida")]
    Utf16,
    #[error("{0} bytes sobrando")]
    Trailing(usize),
    #[error("campo maior que o buffer do engine")]
    TooLong,
}

pub type Result<T> = std::result::Result<T, WireError>;

pub struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    pub fn read_b(&mut self, n: usize) -> Result<&'a [u8]> {
        if self.remaining() < n {
            return Err(WireError::Short);
        }
        let out = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        Ok(out)
    }

    pub fn read_c(&mut self) -> Result<u8> {
        Ok(self.read_b(1)?[0])
    }

    pub fn read_h(&mut self) -> Result<u16> {
        let b = self.read_b(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    pub fn read_d(&mut self) -> Result<i32> {
        let b = self.read_b(4)?;
        Ok(i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn read_f(&mut self) -> Result<f64> {
        let b = self.read_b(8)?;
        let mut arr = [0u8; 8];
        arr.copy_from_slice(b);
        Ok(f64::from_le_bytes(arr))
    }

    pub fn read_s(&mut self) -> Result<String> {
        let mut units = Vec::new();
        loop {
            let unit = self.read_h()?;
            if unit == 0 {
                break;
            }
            units.push(unit);
        }
        String::from_utf16(&units).map_err(|_| WireError::Utf16)
    }

    pub fn finish(&self) -> Result<()> {
        match self.remaining() {
            0 => Ok(()),
            n => Err(WireError::Trailing(n)),
        }
    }
}

#[derive(Default)]
pub struct Writer {
    buf: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write_b(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    pub fn write_c(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn write_h(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_d(&mut self, v: i32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_f(&mut self, v: f64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_s(&mut self, s: &str) {
        for unit in s.encode_utf16() {
            self.write_h(unit);
        }
        self.write_h(0);
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_scalars() {
        let mut w = Writer::new();
        w.write_c(0x13);
        w.write_h(0xBEEF);
        w.write_d(-2);
        w.write_f(7620.0);
        let bytes = w.into_bytes();
        assert_eq!(bytes.len(), 1 + 2 + 4 + 8);
        let mut r = Reader::new(&bytes);
        assert_eq!(r.read_c().unwrap(), 0x13);
        assert_eq!(r.read_h().unwrap(), 0xBEEF);
        assert_eq!(r.read_d().unwrap(), -2);
        assert_eq!(r.read_f().unwrap(), 7620.0);
        assert!(r.finish().is_ok());
    }

    #[test]
    fn strings_are_utf16le_null_terminated() {
        let mut w = Writer::new();
        w.write_s("Luzal");
        let bytes = w.into_bytes();
        assert_eq!(bytes, [b'L', 0, b'u', 0, b'z', 0, b'a', 0, b'l', 0, 0, 0]);
        let mut r = Reader::new(&bytes);
        assert_eq!(r.read_s().unwrap(), "Luzal");
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn short_buffer_is_an_error() {
        let mut r = Reader::new(&[1, 2, 3]);
        assert!(matches!(r.read_d(), Err(WireError::Short)));
        let mut r = Reader::new(&[b'a', 0, b'b']);
        assert!(matches!(r.read_s(), Err(WireError::Short)));
    }

    #[test]
    fn finish_rejects_trailing_bytes() {
        let mut r = Reader::new(&[1, 2]);
        r.read_c().unwrap();
        assert!(matches!(r.finish(), Err(WireError::Trailing(1))));
    }
}
