use crate::Error;
use ue2_core::Reader;

pub struct DatReader<'a> {
    inner: Reader<'a>,
}

impl<'a> DatReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { inner: Reader::at(data, 0) }
    }

    pub fn u8(&mut self) -> Result<u8, Error> {
        Ok(self.inner.u8()?)
    }

    pub fn u32(&mut self) -> Result<u32, Error> {
        Ok(self.inner.u32()?)
    }

    pub fn i32(&mut self) -> Result<i32, Error> {
        Ok(self.inner.i32()?)
    }

    pub fn f32(&mut self) -> Result<f32, Error> {
        Ok(self.inner.f32()?)
    }

    pub fn rgb(&mut self) -> Result<[u8; 3], Error> {
        Ok(self.inner.array()?)
    }

    pub fn unicode(&mut self) -> Result<String, Error> {
        let len = self.inner.u32()?;
        Ok(self.inner.utf16(len as usize)?)
    }

    pub fn ascf(&mut self) -> Result<String, Error> {
        Ok(self.inner.string()?)
    }

    pub fn count(&mut self) -> Result<usize, Error> {
        let at = self.inner.pos();
        usize::try_from(self.inner.compact()?).map_err(|_| Error::NegativeCount(at))
    }

    pub fn unicode_array(&mut self) -> Result<Vec<String>, Error> {
        let len = self.inner.u32()?;
        (0..len).map(|_| self.unicode()).collect()
    }

    pub fn remaining(&self) -> usize {
        self.inner.remaining().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_is_byte_length_utf16_without_nul() {
        let mut data = 44u32.to_le_bytes().to_vec();
        data.extend("LineageMonster.gremlin".encode_utf16().flat_map(u16::to_le_bytes));
        let mut r = DatReader::new(&data);
        assert_eq!(r.unicode().unwrap(), "LineageMonster.gremlin");
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn empty_unicode_is_four_zero_bytes() {
        let mut r = DatReader::new(&[0, 0, 0, 0, 1, 0, 0, 0]);
        assert_eq!(r.unicode().unwrap(), "");
        assert_eq!(r.u32().unwrap(), 1);
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn ascf_length_includes_nul_and_uses_compact_index() {
        let mut r = DatReader::new(b"\x07Lavier\x00");
        assert_eq!(r.ascf().unwrap(), "Lavier");
        assert_eq!(r.remaining(), 0);
        let long = format!("{}\0", "x".repeat(77));
        let mut data = vec![0x4e, 0x01];
        data.extend(long.bytes());
        let mut r = DatReader::new(&data);
        assert_eq!(r.ascf().unwrap().len(), 77);
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn ascf_of_length_zero_is_empty() {
        let mut r = DatReader::new(&[0x00, 0x9c]);
        assert_eq!(r.ascf().unwrap(), "");
        assert_eq!(r.u8().unwrap(), 0x9c);
    }

    #[test]
    fn count_is_a_compact_index() {
        let mut r = DatReader::new(&[0x01, 0xce, 0x10, 0x00, 0x00]);
        assert_eq!(r.count().unwrap(), 1);
        assert_eq!(r.u32().unwrap(), 4302);
        let mut r = DatReader::new(&[0x40, 0x01]);
        assert_eq!(r.count().unwrap(), 64);
        let mut r = DatReader::new(&[0x81]);
        assert_eq!(r.count(), Err(Error::NegativeCount(0)));
    }

    #[test]
    fn unicode_array_reads_count_then_strings() {
        let mut data = 2u32.to_le_bytes().to_vec();
        for s in ["ab", ""] {
            data.extend((s.len() as u32 * 2).to_le_bytes());
            data.extend(s.encode_utf16().flat_map(u16::to_le_bytes));
        }
        let mut r = DatReader::new(&data);
        assert_eq!(r.unicode_array().unwrap(), ["ab", ""]);
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn truncated_unicode_is_an_error() {
        let mut r = DatReader::new(&[10, 0, 0, 0, b'a', 0]);
        assert_eq!(r.unicode(), Err(Error::Truncated(4)));
        let mut r = DatReader::new(&[7, b'L']);
        assert!(r.ascf().is_err());
        let mut r = DatReader::new(&[1, 0]);
        assert!(r.u32().is_err());
        assert!(r.f32().is_err());
        assert!(r.rgb().is_err());
    }
}
