use crate::Error;
use crate::reader::DatReader;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerName {
    pub id: u32,
    pub tag: u32,
    pub name: String,
    pub description: String,
}

pub fn read(body: &[u8]) -> Result<Vec<ServerName>, Error> {
    let mut r = DatReader::new(body);
    let count = r.u32()? as usize;
    let mut rows = Vec::with_capacity(count.min(1024));
    for _ in 0..count {
        let id = r.u32()?;
        let tag = r.u32()?;
        let name = r.ascf()?.trim_end_matches('\0').to_string();
        let description = r.ascf()?.trim_end_matches('\0').to_string();
        rows.push(ServerName { id, tag, name, description });
    }
    Ok(rows)
}

pub fn by_id(rows: Vec<ServerName>) -> HashMap<u32, ServerName> {
    rows.into_iter().map(|row| (row.id, row)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_count_id_tag_name_and_description() {
        let mut body = 2u32.to_le_bytes().to_vec();
        body.extend_from_slice(&0u32.to_le_bytes());
        body.extend_from_slice(&1u32.to_le_bytes());
        body.extend_from_slice(b"\x0bKiller 70x\0\0");
        body.extend_from_slice(&1u32.to_le_bytes());
        body.extend_from_slice(&1u32.to_le_bytes());
        body.extend_from_slice(b"\x0aHonor 25x\0\x02x\0");
        let rows = read(&body).unwrap();
        assert_eq!(rows[0], ServerName { id: 0, tag: 1, name: "Killer 70x".into(), description: String::new() });
        assert_eq!(rows[1].name, "Honor 25x");
        assert_eq!(rows[1].description, "x");
        assert_eq!(by_id(rows)[&1].name, "Honor 25x");
        assert!(read(&body[..20]).is_err());
    }
}
