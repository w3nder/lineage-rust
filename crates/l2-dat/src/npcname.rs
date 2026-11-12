use crate::{DatReader, Error};
use std::collections::HashMap;

pub const NO_NAME_NPC: &str = "NoNameNPC";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcName {
    pub id: u32,
    pub name: String,
    pub title: String,
    pub color: [u8; 3],
}

impl NpcName {
    pub fn no_name(id: u32) -> Self {
        Self { id, name: NO_NAME_NPC.to_string(), title: String::new(), color: [0; 3] }
    }
}

pub fn npc_records(npc_ids: impl IntoIterator<Item = u32>, names: impl IntoIterator<Item = NpcName>) -> HashMap<u32, NpcName> {
    let mut records: HashMap<u32, NpcName> = npc_ids.into_iter().map(|id| (id, NpcName::no_name(id))).collect();
    for name in names {
        if let Some(record) = records.get_mut(&name.id) {
            *record = name;
        }
    }
    records
}

pub fn read(body: &[u8]) -> Result<Vec<NpcName>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count).map(|_| record(&mut reader)).collect()
}

fn record(r: &mut DatReader<'_>) -> Result<NpcName, Error> {
    let id = r.u32()?;
    let name = r.ascf()?;
    let title = r.ascf()?;
    let color = r.rgb()?;
    r.u8()?;
    Ok(NpcName { id, name, title, color })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn npcname_first_records() {
        let names = read(include_bytes!("../tests/fixtures/npcname-head.bin")).unwrap();
        assert_eq!(names.len(), 3);
        assert_eq!(
            (names[0].id, names[0].name.as_str(), names[0].title.as_str(), names[0].color),
            (28086, "Lavier", "Capture Master", [0x3f, 0x8b, 0xfe])
        );
        assert_eq!((names[1].id, names[1].name.as_str(), names[1].title.as_str()), (35710, "Spemar", "Team Vs Team"));
        assert_eq!((names[2].id, names[2].name.as_str(), names[2].title.as_str()), (35711, "DM Manager", ""));
        assert_eq!(names[2].color, [0x9c, 0xe8, 0xa9]);
    }

    #[test]
    fn npc_data_load_callee_10460820_names_rows_without_npcname_and_drops_orphans() {
        let names = read(include_bytes!("../tests/fixtures/npcname-head.bin")).unwrap();
        let records = npc_records([28086, 35711, 1], names);
        assert_eq!(records.len(), 3);
        assert_eq!(records[&28086].name, "Lavier");
        assert_eq!(records[&28086].color, [0x3f, 0x8b, 0xfe]);
        assert_eq!(records[&35711].name, "DM Manager");
        assert_eq!(records[&1], NpcName::no_name(1));
        assert_eq!((records[&1].name.as_str(), records[&1].title.as_str(), records[&1].color), (NO_NAME_NPC, "", [0; 3]));
        assert!(!records.contains_key(&35710));
    }

    #[test]
    fn truncated_body_is_an_error() {
        let body = include_bytes!("../tests/fixtures/npcname-head.bin");
        assert!(read(&body[..body.len() - 1]).is_err());
    }
}
