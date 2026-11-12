use crate::{DatReader, Error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticObject {
    pub id: u32,
    pub name: String,
}

pub fn read(body: &[u8]) -> Result<Vec<StaticObject>, Error> {
    read_with_tail(body).map(|(rows, _)| rows)
}

pub fn read_with_tail(body: &[u8]) -> Result<(Vec<StaticObject>, &[u8]), Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    let rows = (0..count).map(|_| record(&mut reader)).collect::<Result<Vec<_>, _>>()?;
    Ok((rows, &body[body.len() - reader.remaining()..]))
}

fn record(r: &mut DatReader<'_>) -> Result<StaticObject, Error> {
    let id = r.u32()?;
    let name = r.unicode()?;
    Ok(StaticObject { id, name })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unicode(s: &str) -> Vec<u8> {
        let mut out = (s.len() as u32 * 2).to_le_bytes().to_vec();
        out.extend(s.encode_utf16().flat_map(u16::to_le_bytes));
        out
    }

    #[test]
    fn clan_hall_door_keeps_its_id_and_the_tail_is_left_over() {
        let mut body = 2u32.to_le_bytes().to_vec();
        body.extend(17220001u32.to_le_bytes());
        body.extend(unicode("Clan Hall Door"));
        body.extend(24160024u32.to_le_bytes());
        body.extend(unicode("Throne"));
        body.extend(b"\x0cSafePackage\x00");
        let (objects, tail) = read_with_tail(&body).unwrap();
        assert_eq!(objects.len(), 2);
        assert_eq!((objects[0].id, objects[0].name.as_str()), (17220001, "Clan Hall Door"));
        assert_eq!((objects[1].id, objects[1].name.as_str()), (24160024, "Throne"));
        assert_eq!(tail, b"\x0cSafePackage\x00");
        assert!(read(&body[..10]).is_err());
    }
}
