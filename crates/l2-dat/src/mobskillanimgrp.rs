use crate::{DatReader, Error};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcSkillAnim {
    pub npc_id: u32,
    pub skill_id: u32,
    pub seq_name: String,
    pub skill_name: String,
    pub npc_name: String,
    pub npc_class: String,
}

pub fn read(body: &[u8]) -> Result<Vec<NpcSkillAnim>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count).map(|_| record(&mut reader)).collect()
}

fn record(r: &mut DatReader<'_>) -> Result<NpcSkillAnim, Error> {
    Ok(NpcSkillAnim {
        npc_id: r.u32()?,
        skill_id: r.u32()?,
        seq_name: r.unicode()?,
        skill_name: r.ascf()?,
        npc_name: r.ascf()?,
        npc_class: r.ascf()?,
    })
}

pub fn by_npc_skill(rows: Vec<NpcSkillAnim>) -> HashMap<(u32, u32), String> {
    rows.into_iter().map(|row| ((row.npc_id, row.skill_id), row.seq_name)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mobskillanimgrp_first_records() {
        let rows = read(include_bytes!("../tests/fixtures/mobskillanimgrp-head.bin")).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(
            rows[0],
            NpcSkillAnim {
                npc_id: 12783,
                skill_id: 122,
                seq_name: "spatk02".into(),
                skill_name: "s_hex10".into(),
                npc_name: "tamed_kukaburo_war".into(),
                npc_class: "LineageMonster2.pet_ostrich".into(),
            }
        );
        assert_eq!((rows[2].npc_id, rows[2].seq_name.as_str()), (12787, "spatk01"));
        assert_eq!(rows[2].npc_class, "LineageMonster.saber_toothed_tiger");
        let table = by_npc_skill(rows);
        assert_eq!(table[&(12785, 122)], "spatk02");
        assert!(!table.contains_key(&(12785, 123)));
    }

    #[test]
    fn truncated_body_is_an_error() {
        let body = include_bytes!("../tests/fixtures/mobskillanimgrp-head.bin");
        assert!(read(&body[..body.len() - 1]).is_err());
    }
}
