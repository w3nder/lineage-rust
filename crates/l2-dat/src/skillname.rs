use crate::{DatReader, Error};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillName {
    pub id: u32,
    pub level: u32,
    pub name: String,
    pub description: String,
    pub desc_add1: String,
    pub desc_add2: String,
}

pub fn read(body: &[u8]) -> Result<Vec<SkillName>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count)
        .map(|_| {
            Ok(SkillName {
                id: reader.u32()?,
                level: reader.u32()?,
                name: reader.ascf()?,
                description: reader.ascf()?,
                desc_add1: reader.ascf()?,
                desc_add2: reader.ascf()?,
            })
        })
        .collect()
}

pub fn by_id_level(names: Vec<SkillName>) -> HashMap<(u32, u32), SkillName> {
    names.into_iter().map(|n| ((n.id, n.level), n)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_are_id_level_and_four_ascf_strings() {
        let mut body = 1u32.to_le_bytes().to_vec();
        body.extend(3u32.to_le_bytes());
        body.extend(1u32.to_le_bytes());
        body.extend(b"\x0dPower Strike\x00");
        body.extend(b"\x08Gathers\x00");
        body.extend(b"\x05none\x00");
        body.extend(b"\x05none\x00");
        let names = read(&body).unwrap();
        assert_eq!(names.len(), 1);
        assert_eq!((names[0].id, names[0].level, names[0].name.as_str()), (3, 1, "Power Strike"));
        assert_eq!(names[0].description, "Gathers");
        assert_eq!(names[0].desc_add2, "none");
        assert_eq!(by_id_level(names)[&(3, 1)].name, "Power Strike");
        assert!(read(&body[..body.len() - 1]).is_err());
    }
}
