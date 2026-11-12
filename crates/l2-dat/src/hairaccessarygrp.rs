use crate::{DatReader, Error};

pub const GROUPS: usize = 15;
pub const SLOTS: usize = 15;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HairAccessory {
    pub index: u32,
    pub a: [[i32; SLOTS]; GROUPS],
    pub b: [[i32; SLOTS]; GROUPS],
}

pub fn read(body: &[u8]) -> Result<Vec<HairAccessory>, Error> {
    read_with_tail(body).map(|(rows, _)| rows)
}

pub fn read_with_tail(body: &[u8]) -> Result<(Vec<HairAccessory>, &[u8]), Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    let rows = (0..count).map(|_| record(&mut reader)).collect::<Result<Vec<_>, _>>()?;
    Ok((rows, &body[body.len() - reader.remaining()..]))
}

fn record(r: &mut DatReader<'_>) -> Result<HairAccessory, Error> {
    let index = r.u32()?;
    let mut a = [[0; SLOTS]; GROUPS];
    let mut b = [[0; SLOTS]; GROUPS];
    for group in 0..GROUPS {
        for slot in 0..SLOTS {
            a[group][slot] = r.i32()?;
            b[group][slot] = r.i32()?;
        }
    }
    Ok(HairAccessory { index, a, b })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_interleaves_a_and_b_per_slot_and_leaves_the_footer() {
        let mut body = 1u32.to_le_bytes().to_vec();
        body.extend(7u32.to_le_bytes());
        for group in 0..GROUPS as i32 {
            for slot in 0..SLOTS as i32 {
                body.extend((group * 100 + slot).to_le_bytes());
                body.extend((-(group * 100 + slot) - 1).to_le_bytes());
            }
        }
        body.extend(b"\x0cSafePackage\x00");
        let (rows, tail) = read_with_tail(&body).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].index, 7);
        assert_eq!(rows[0].a[2][3], 203);
        assert_eq!(rows[0].b[2][3], -204);
        assert_eq!(rows[0].a[14][14], 1414);
        assert_eq!(tail, b"\x0cSafePackage\x00");
        assert!(read(&body[..body.len() - 20]).is_err());
    }
}
