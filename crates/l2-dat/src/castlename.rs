use crate::{DatReader, Error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Castle {
    pub index: u32,
    pub present: u32,
    pub id: u32,
    pub name: String,
    pub location: String,
    pub description: String,
}

pub fn read(body: &[u8]) -> Result<Vec<Castle>, Error> {
    read_with_tail(body).map(|(rows, _)| rows)
}

pub fn read_with_tail(body: &[u8]) -> Result<(Vec<Castle>, &[u8]), Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    let rows = (0..count).map(|_| record(&mut reader)).collect::<Result<Vec<_>, _>>()?;
    Ok((rows, &body[body.len() - reader.remaining()..]))
}

fn record(r: &mut DatReader<'_>) -> Result<Castle, Error> {
    let index = r.u32()?;
    let present = r.u32()?;
    let id = r.u32()?;
    let name = r.ascf()?;
    let location = r.ascf()?;
    let description = r.ascf()?;
    Ok(Castle { index, present, id, name, location, description })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gludio_is_castle_one_and_the_tail_is_left_over() {
        let mut body = 1u32.to_le_bytes().to_vec();
        for v in [0u32, 1, 1] {
            body.extend(v.to_le_bytes());
        }
        body.extend(b"\x0eGludio Castle\x00");
        body.extend(b"\x07Gludio\x00");
        body.push(0);
        body.extend(b"\x0cSafePackage\x00");
        let (castles, tail) = read_with_tail(&body).unwrap();
        assert_eq!(castles.len(), 1);
        let gludio = &castles[0];
        assert_eq!((gludio.index, gludio.present, gludio.id), (0, 1, 1));
        assert_eq!((gludio.name.as_str(), gludio.location.as_str(), gludio.description.as_str()), ("Gludio Castle", "Gludio", ""));
        assert_eq!(tail, b"\x0cSafePackage\x00");
        assert!(read(&body[..20]).is_err());
    }
}
