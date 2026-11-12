use crate::{DatReader, Error};

const FOOTER: &str = "SafePackage";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Eula {
    pub eula: String,
    pub test_server: String,
}

impl Eula {
    pub fn text(&self, test_server: bool) -> &str {
        if test_server { &self.test_server } else { &self.eula }
    }
}

pub fn read(body: &[u8]) -> Result<Eula, Error> {
    let mut reader = DatReader::new(body);
    let eula = reader.ascf()?;
    let test_server = reader.ascf()?;
    let at = body.len() - reader.remaining();
    if reader.ascf()? != FOOTER || reader.remaining() != 0 {
        return Err(Error::InvalidString(at));
    }
    Ok(Eula { eula, test_server })
}

pub fn read_as(body: &[u8], chronicle: crate::Chronicle) -> Result<Eula, Error> {
    match chronicle {
        crate::Chronicle::C4 => read(body),
        crate::Chronicle::Interlude => Ok(read_interlude(body)?.0),
    }
}

pub fn read_interlude(body: &[u8]) -> Result<(Eula, String), Error> {
    let mut reader = DatReader::new(body);
    let eula = reader.ascf()?;
    let test_server = reader.ascf()?;
    let third = reader.ascf()?;
    let at = body.len() - reader.remaining();
    if reader.ascf()? != FOOTER || reader.remaining() != 0 {
        return Err(Error::InvalidString(at));
    }
    Ok((Eula { eula, test_server }, third))
}


#[cfg(test)]
mod tests {
    use super::*;

    fn ascii(s: &str) -> Vec<u8> {
        let mut out = vec![s.len() as u8 + 1];
        out.extend(s.bytes());
        out.push(0);
        out
    }

    fn utf16(s: &str) -> Vec<u8> {
        let units: Vec<u16> = s.encode_utf16().chain([0]).collect();
        let mut out = vec![0x80 | units.len() as u8];
        out.extend(units.into_iter().flat_map(u16::to_le_bytes));
        out
    }

    #[test]
    fn reads_the_unicode_text_the_test_server_text_and_the_footer() {
        let mut file = utf16("Lineage II \u{201c}Agreement\u{201d}");
        file.extend(ascii("\r\n"));
        file.extend(ascii(FOOTER));
        let eula = read(&file).unwrap();
        assert_eq!(eula.eula, "Lineage II \u{201c}Agreement\u{201d}");
        assert_eq!(eula.test_server, "\r\n");
        assert_eq!(eula.text(false), eula.eula);
        assert_eq!(eula.text(true), "\r\n");
        assert!(read(&file[..file.len() - 1]).is_err());
        file.push(0);
        assert!(read(&file).is_err());
    }

    #[test]
    fn interlude_has_a_third_text_before_the_footer() {
        let mut file = utf16("Agreement");
        file.extend(ascii("one"));
        file.extend(ascii("two"));
        file.extend(ascii(FOOTER));
        let (eula, third) = read_interlude(&file).unwrap();
        assert_eq!((eula.eula.as_str(), eula.test_server.as_str(), third.as_str()), ("Agreement", "one", "two"));
        assert_eq!(read_as(&file, crate::Chronicle::Interlude).unwrap(), eula);
        assert!(read_as(&file, crate::Chronicle::C4).is_err());
        assert!(read_interlude(&file[..file.len() - 1]).is_err());
        file.push(0);
        assert!(read_interlude(&file).is_err());
    }
}
