use crate::{DatReader, Error};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Npc {
    pub id: u32,
    pub class: String,
    pub mesh: String,
    pub textures: Vec<String>,
    pub textures2: Vec<String>,
    pub speed: f32,
    pub effect: String,
    pub level_min: u32,
    pub level_max: u32,
    pub attack_sounds: Vec<String>,
    pub defense_sounds: Vec<String>,
    pub damage_sounds: Vec<String>,
    pub sound_radius: f32,
    pub sound_volume: f32,
    pub sound_random: f32,
    pub skills: Vec<u32>,
    pub shows_bars: u32,
}

pub fn read(body: &[u8]) -> Result<Vec<Npc>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count).map(|_| record(&mut reader)).collect()
}

pub fn read_as(body: &[u8], chronicle: crate::Chronicle) -> Result<Vec<Npc>, Error> {
    match chronicle {
        crate::Chronicle::C4 => read(body),
        crate::Chronicle::Interlude => read_interlude(body),
    }
}

pub fn read_interlude(body: &[u8]) -> Result<Vec<Npc>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count).map(|_| record_interlude(&mut reader)).collect()
}

fn record_interlude(r: &mut DatReader<'_>) -> Result<Npc, Error> {
    let id = r.u32()?;
    let class = r.unicode()?;
    let mesh = r.unicode()?;
    let textures = r.unicode_array()?;
    let textures2 = r.unicode_array()?;
    let skills = counted_u32s(r)?;
    let speed = r.f32()?;
    r.u32()?;
    let attack_sounds = r.unicode_array()?;
    let defense_sounds = r.unicode_array()?;
    let damage_sounds = r.unicode_array()?;
    let effects = r.u32()?;
    for _ in 0..effects {
        r.unicode()?;
        r.f32()?;
    }
    skip_counted_u32s(r)?;
    let effect = r.unicode()?;
    r.u32()?;
    let sound_radius = r.f32()?;
    let sound_volume = r.f32()?;
    let sound_random = r.f32()?;
    r.u32()?;
    let shows_bars = r.u32()?;
    Ok(Npc { id, class, mesh, textures, textures2, speed, effect, level_min: 0, level_max: 0, attack_sounds, defense_sounds, damage_sounds, sound_radius, sound_volume, sound_random, skills, shows_bars })
}

fn record(r: &mut DatReader<'_>) -> Result<Npc, Error> {
    let id = r.u32()?;
    let class = r.unicode()?;
    let mesh = r.unicode()?;
    let textures = r.unicode_array()?;
    let textures2 = r.unicode_array()?;
    let skills = counted_u32s(r)?;
    let speed = r.f32()?;
    r.u32()?;
    let attack_sounds = r.unicode_array()?;
    let defense_sounds = r.unicode_array()?;
    let damage_sounds = r.unicode_array()?;
    r.u32()?;
    skip_counted_u32s(r)?;
    let level_min = r.u32()?;
    let level_max = r.u32()?;
    let effect = r.unicode()?;
    r.u32()?;
    let sound_radius = r.f32()?;
    let sound_volume = r.f32()?;
    let sound_random = r.f32()?;
    r.u32()?;
    let shows_bars = r.u32()?;
    Ok(Npc { id, class, mesh, textures, textures2, speed, effect, level_min, level_max, attack_sounds, defense_sounds, damage_sounds, sound_radius, sound_volume, sound_random, skills, shows_bars })
}

fn counted_u32s(r: &mut DatReader<'_>) -> Result<Vec<u32>, Error> {
    let count = r.count()?;
    (0..count).map(|_| r.u32()).collect()
}

fn skip_counted_u32s(r: &mut DatReader<'_>) -> Result<(), Error> {
    let count = r.count()?;
    (0..count).try_for_each(|_| r.u32().map(drop))
}

#[derive(Debug, Default, Clone)]
pub struct NpcTable(HashMap<u32, Npc>);

impl NpcTable {
    pub fn get(&self, id: u32) -> Option<&Npc> {
        self.0.get(&id)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Npc> {
        self.0.values()
    }
}

impl FromIterator<Npc> for NpcTable {
    fn from_iter<I: IntoIterator<Item = Npc>>(npcs: I) -> Self {
        Self(crate::by_id(npcs, |npc| npc.id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn npcgrp_first_records() {
        let npcs = read(include_bytes!("../tests/fixtures/npcgrp-head.bin")).unwrap();
        assert_eq!(npcs.len(), 3);
        assert_eq!(npcs[0].id, 1);
        assert_eq!(npcs[0].class, "LineageMonster.gremlin");
        assert_eq!(npcs[0].mesh, "LineageMonsters.gremlin_m00");
        assert_eq!(npcs[0].textures, ["LineageMonstersTex.gremlin_t00"]);
        assert!(npcs[0].textures2.is_empty());
        assert_eq!(npcs[0].speed, 1.0);
        assert_eq!(npcs[0].effect, "LineageEffect.p_u002_a");
        assert_eq!((npcs[0].level_min, npcs[0].level_max), (0, 0));
        assert_eq!(npcs[1].mesh, "LineageMonsters.rabbit_m00");
        assert_eq!(npcs[2].id, 3);
        assert_eq!(npcs[2].textures, ["LineageMonstersTex.goblin_t00", "LineageMonstersTex.goblin_t01"]);
    }

    fn interlude_record(id: u32, halo: bool) -> Vec<u8> {
        fn unicode(out: &mut Vec<u8>, text: &str) {
            let units: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
            out.extend((units.len() as u32).to_le_bytes());
            out.extend(units);
        }
        let mut out = id.to_le_bytes().to_vec();
        unicode(&mut out, "LineageMonster.gremlin");
        unicode(&mut out, "LineageMonsters.gremlin_m00");
        out.extend(1u32.to_le_bytes());
        unicode(&mut out, "LineageMonstersTex.gremlin_t00");
        out.extend(0u32.to_le_bytes());
        out.push(2);
        out.extend(4416u32.to_le_bytes());
        out.extend(13u32.to_le_bytes());
        out.extend(1.0f32.to_le_bytes());
        out.extend(0u32.to_le_bytes());
        out.extend(1u32.to_le_bytes());
        unicode(&mut out, "");
        out.extend(1u32.to_le_bytes());
        unicode(&mut out, "MonSound.Hit_Wet_4");
        out.extend(1u32.to_le_bytes());
        unicode(&mut out, "MonSound.gremlin_dmg_1");
        if halo {
            out.extend(1u32.to_le_bytes());
            unicode(&mut out, "LineageEffect.ra_boss_halo_a_ca");
            out.extend(1.5f32.to_le_bytes());
        } else {
            out.extend(0u32.to_le_bytes());
        }
        out.push(1);
        out.extend(258u32.to_le_bytes());
        unicode(&mut out, "LineageEffect.p_u002_a");
        out.extend(0u32.to_le_bytes());
        out.extend(50.0f32.to_le_bytes());
        out.extend(250.0f32.to_le_bytes());
        out.extend(70.0f32.to_le_bytes());
        out.extend(0u32.to_le_bytes());
        out.extend(1u32.to_le_bytes());
        out
    }

    #[test]
    fn interlude_layout_has_effect_pairs_and_no_level_limit() {
        let mut body = 2u32.to_le_bytes().to_vec();
        body.extend(interlude_record(20001, false));
        body.extend(interlude_record(25035, true));
        let npcs = read_as(&body, crate::Chronicle::Interlude).unwrap();
        assert_eq!(npcs.len(), 2);
        assert_eq!(npcs[0].id, 20001);
        assert_eq!(npcs[0].skills, [4416, 13]);
        assert_eq!(npcs[0].attack_sounds, [""]);
        assert_eq!(npcs[0].damage_sounds, ["MonSound.gremlin_dmg_1"]);
        assert_eq!(npcs[0].effect, "LineageEffect.p_u002_a");
        assert_eq!((npcs[0].sound_radius, npcs[0].sound_volume, npcs[0].sound_random), (50.0, 250.0, 70.0));
        assert_eq!(npcs[0].shows_bars, 1);
        assert_eq!(npcs[1].id, 25035);
        assert_eq!(npcs[1].effect, "LineageEffect.p_u002_a");
        assert!(read_as(&body[..body.len() - 1], crate::Chronicle::Interlude).is_err());
    }

    #[test]
    fn truncated_body_is_an_error() {
        let body = include_bytes!("../tests/fixtures/npcgrp-head.bin");
        assert!(read(&body[..body.len() - 1]).is_err());
        assert!(read(&[]).is_err());
    }

    #[test]
    fn table_keeps_the_last_row_of_a_duplicated_id() {
        let npcs = read(include_bytes!("../tests/fixtures/npcgrp-head.bin")).unwrap();
        let mut duplicate = npcs[2].clone();
        duplicate.id = 1;
        let table: NpcTable = npcs.into_iter().chain([duplicate]).collect();
        assert_eq!(table.len(), 3);
        assert_eq!(table.get(1).unwrap().mesh, "LineageMonsters.goblin_m00");
        assert!(table.get(4).is_none());
    }
}
