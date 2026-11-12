use crate::{DatReader, Error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemName {
    pub id: u32,
    pub name: String,
    pub add_name: String,
    pub description: String,
    pub popup: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemNameSet {
    pub set_ids: String,
    pub set_effect: String,
    pub extra_ids: String,
    pub extra_effect: String,
    pub enchant_ids: String,
    pub enchant_effect: String,
    pub enchant_level: u32,
    pub enchant_text: String,
}

pub fn read(body: &[u8]) -> Result<Vec<ItemName>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count).map(|_| record(&mut reader)).collect()
}

pub fn read_as(body: &[u8], chronicle: crate::Chronicle) -> Result<Vec<ItemName>, Error> {
    match chronicle {
        crate::Chronicle::C4 => read(body),
        crate::Chronicle::Interlude => read_interlude(body).map(|rows| rows.into_iter().map(|(item, _)| item).collect()),
    }
}

pub fn read_interlude(body: &[u8]) -> Result<Vec<(ItemName, ItemNameSet)>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count)
        .map(|_| {
            let item = record(&mut reader)?;
            Ok((item, set_block(&mut reader)?))
        })
        .collect()
}

fn record(reader: &mut DatReader<'_>) -> Result<ItemName, Error> {
    Ok(ItemName {
        id: reader.u32()?,
        name: reader.unicode()?,
        add_name: reader.unicode()?,
        description: reader.ascf()?,
        popup: reader.i32()?,
    })
}
fn set_block(reader: &mut DatReader<'_>) -> Result<ItemNameSet, Error> {
    Ok(ItemNameSet {
        set_ids: reader.ascf()?,
        set_effect: reader.ascf()?,
        extra_ids: reader.ascf()?,
        extra_effect: reader.ascf()?,
        enchant_ids: reader.ascf()?,
        enchant_effect: reader.ascf()?,
        enchant_level: reader.u32()?,
        enchant_text: reader.ascf()?,
    })
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
    fn records_are_id_two_unicode_names_an_ascf_description_and_popup() {
        let mut body = 2u32.to_le_bytes().to_vec();
        body.extend(17u32.to_le_bytes());
        body.extend(unicode("Wooden Arrow"));
        body.extend(unicode(""));
        body.extend(b"\x07arrow.\x00");
        body.extend((-1i32).to_le_bytes());
        body.extend(57u32.to_le_bytes());
        body.extend(unicode("Adena"));
        body.extend(unicode(""));
        body.extend(b"\x01\x00");
        body.extend((-1i32).to_le_bytes());
        let names = read(&body).unwrap();
        assert_eq!(names.len(), 2);
        assert_eq!(
            (names[0].id, names[0].name.as_str(), names[0].description.as_str(), names[0].popup),
            (17, "Wooden Arrow", "arrow.", -1)
        );
        assert_eq!((names[1].id, names[1].name.as_str()), (57, "Adena"));
        assert!(read(&body[..body.len() - 1]).is_err());
    }

    fn ascf(s: &str) -> Vec<u8> {
        if s.is_empty() {
            return vec![0];
        }
        let mut out = vec![(s.len() + 1) as u8];
        out.extend(s.bytes());
        out.push(0);
        out
    }

    #[test]
    fn interlude_records_carry_the_set_block_after_the_popup() {
        let mut body = 2u32.to_le_bytes().to_vec();
        body.extend(23u32.to_le_bytes());
        body.extend(unicode("Wooden Breastplate"));
        body.extend(unicode(""));
        body.extend(ascf(""));
        body.extend((-1i32).to_le_bytes());
        body.extend(ascf("23,2386,43"));
        body.extend(ascf("P. Def. +2%."));
        for _ in 0..4 {
            body.extend(ascf(""));
        }
        body.extend(0u32.to_le_bytes());
        body.extend(ascf(""));
        body.extend(58u32.to_le_bytes());
        body.extend(unicode("Mithril Breastplate"));
        body.extend(unicode(""));
        body.extend(ascf(""));
        body.extend((-1i32).to_le_bytes());
        body.extend(ascf("58,59,47"));
        body.extend(ascf("Chance -20%."));
        body.extend(ascf("628"));
        body.extend(ascf("Maximum HP +126."));
        body.extend(ascf(""));
        body.extend(ascf(""));
        body.extend(6u32.to_le_bytes());
        body.extend(ascf("When all set items are enchanted by 6."));
        let rows = read_interlude(&body).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!((rows[0].0.id, rows[1].0.name.as_str()), (23, "Mithril Breastplate"));
        let first = &rows[0].1;
        assert_eq!((first.set_ids.as_str(), first.set_effect.as_str(), first.enchant_level), ("23,2386,43", "P. Def. +2%.", 0));
        let second = &rows[1].1;
        assert_eq!((second.extra_ids.as_str(), second.extra_effect.as_str()), ("628", "Maximum HP +126."));
        assert_eq!((second.enchant_level, second.enchant_text.as_str()), (6, "When all set items are enchanted by 6."));
        assert!(read_interlude(&body[..body.len() - 1]).is_err());
        assert_eq!(read_as(&body, crate::Chronicle::Interlude).unwrap().len(), 2);
    }

    #[test]
    fn c4_rows_end_at_the_popup() {
        let mut body = 1u32.to_le_bytes().to_vec();
        body.extend(57u32.to_le_bytes());
        body.extend(unicode("Adena"));
        body.extend(unicode(""));
        body.extend(b"\x01\x00");
        body.extend((-1i32).to_le_bytes());
        let rows = read_as(&body, crate::Chronicle::C4).unwrap();
        assert_eq!(rows[0].popup, -1);
        assert!(read_as(&body, crate::Chronicle::Interlude).is_err());
    }
}
