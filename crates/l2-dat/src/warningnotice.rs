use crate::{DatReader, Error};

const FOOTER: &str = "SafePackage";

pub fn read(body: &[u8]) -> Result<String, Error> {
    let mut reader = DatReader::new(body);
    let notice = reader.ascf()?;
    let at = body.len() - reader.remaining();
    if reader.ascf()? != FOOTER || reader.remaining() != 0 {
        return Err(Error::InvalidString(at));
    }
    Ok(notice)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf16(s: &str) -> Vec<u8> {
        let units: Vec<u16> = s.encode_utf16().chain([0]).collect();
        let mut out = vec![0x80 | units.len() as u8];
        out.extend(units.into_iter().flat_map(u16::to_le_bytes));
        out
    }

    fn ascii(s: &str) -> Vec<u8> {
        let mut out = vec![s.len() as u8 + 1];
        out.extend(s.bytes());
        out.push(0);
        out
    }

    #[test]
    fn reads_the_single_text_and_the_footer() {
        let mut file = utf16("Aviso importante");
        file.extend(ascii(FOOTER));
        assert_eq!(read(&file).unwrap(), "Aviso importante");
        assert!(read(&file[..file.len() - 1]).is_err());
        file.push(0);
        assert!(read(&file).is_err());
        assert!(read(&[]).is_err());
    }
}
