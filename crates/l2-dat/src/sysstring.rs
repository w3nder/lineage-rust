use crate::{DatReader, Error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SysString {
    pub id: u32,
    pub text: String,
}

pub fn read(body: &[u8]) -> Result<Vec<SysString>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count).map(|_| Ok(SysString { id: reader.u32()?, text: reader.ascf()? })).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sysstring_first_records() {
        let strings = read(include_bytes!("../tests/fixtures/sysstring-head.bin")).unwrap();
        assert_eq!(strings.len(), 4);
        assert_eq!(strings[0], SysString { id: 0, text: String::new() });
        assert_eq!(strings[1], SysString { id: 1, text: "Equipment".into() });
        assert_eq!(strings[3], SysString { id: 3, text: "Quest Item".into() });
    }

    #[test]
    fn truncated_body_is_an_error() {
        let body = include_bytes!("../tests/fixtures/sysstring-head.bin");
        assert!(read(&body[..body.len() - 1]).is_err());
    }
}
