use crate::{DatReader, Error};

pub const ROWS: usize = 15;
pub const TAG_WEAPON: u32 = 0;
pub const TAG_ARMOR: u32 = 1;

pub mod body_part {
    pub const UNDERWEAR: u32 = 0;
    pub const REAR: u32 = 1;
    pub const LEAR: u32 = 2;
    pub const NECK: u32 = 3;
    pub const RFINGER: u32 = 4;
    pub const LFINGER: u32 = 5;
    pub const HEAD: u32 = 6;
    pub const RHAND: u32 = 7;
    pub const LHAND: u32 = 8;
    pub const GLOVES: u32 = 9;
    pub const CHEST: u32 = 10;
    pub const LEGS: u32 = 11;
    pub const FEET: u32 = 12;
    pub const BACK: u32 = 13;
    pub const LRHAND: u32 = 14;
    pub const ONEPIECE: u32 = 15;
    pub const HAIR: u32 = 16;
    pub const ALLDRESS: u32 = 17;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArmorRow {
    pub meshes: Vec<String>,
    pub textures: Vec<String>,
    pub ex_meshes: Vec<String>,
    pub ex_textures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Armor {
    pub id: u32,
    pub drop_sound: String,
    pub equip_sound: String,
    pub boots_sounds: Vec<String>,
    pub body_part: u32,
    pub rows: Vec<ArmorRow>,
    pub npc_meshes: Vec<String>,
    pub npc_textures: Vec<String>,
    pub damage_effect: String,
    pub attributes_58c: [i32; 6],
    pub crystallizable: bool,
}

impl Armor {
    pub fn mesh(&self, row: usize, index: usize) -> Option<&str> {
        self.rows.get(row)?.meshes.get(index).map(String::as_str).filter(|m| !m.is_empty())
    }

    pub fn texture(&self, row: usize, index: usize) -> Option<&str> {
        self.rows.get(row)?.textures.get(index).map(String::as_str).filter(|t| !t.is_empty())
    }
}

pub fn read(body: &[u8]) -> Result<Vec<Armor>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count).map(|_| record(&mut reader)).collect()
}

pub const ICONS: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemDrop {
    pub style: u32,
    pub parameter_14: u32,
    pub radius: u32,
    pub height: u32,
    pub parameter_20: u32,
    pub meshes: [String; 3],
    pub materials: [String; 3],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemHead {
    pub tag: u32,
    pub id: u32,
    pub drop: ItemDrop,
    pub icons: [String; ICONS],
    pub weight: u32,
    pub crystallizable: bool,
    pub type1: u32,
}

pub fn item_head(r: &mut DatReader<'_>) -> Result<ItemHead, Error> {
    let tag = r.u32()?;
    let id = r.u32()?;
    let mut drop = ItemDrop { style: r.u32()?, parameter_14: r.u32()?, radius: r.u32()?, height: r.u32()?, parameter_20: r.u32()?, ..Default::default() };
    for mesh in &mut drop.meshes { *mesh = r.unicode()?; }
    for material in &mut drop.materials { *material = r.unicode()?; }
    let mut icons: [String; ICONS] = Default::default();
    for icon in &mut icons {
        *icon = r.unicode()?;
    }
    r.i32()?;
    let weight = r.u32()?;
    r.u32()?;
    let crystallizable = r.u32()? != 0;
    let type1 = r.u32()?;
    Ok(ItemHead { tag, id, drop, icons, weight, crystallizable, type1 })
}

fn record(r: &mut DatReader<'_>) -> Result<Armor, Error> {
    let head = item_head(r)?;
    let mut armor = body(r, head.id)?;
    armor.crystallizable = head.crystallizable;
    Ok(armor)
}

pub fn body(r: &mut DatReader<'_>, id: u32) -> Result<Armor, Error> {
    let body_part = r.u32()?;
    let mut rows = Vec::with_capacity(ROWS);
    for _ in 0..ROWS {
        rows.push(ArmorRow {
            meshes: r.unicode_array()?,
            textures: r.unicode_array()?,
            ex_meshes: r.unicode_array()?,
            ex_textures: r.unicode_array()?,
        });
    }
    let npc_meshes = r.unicode_array()?;
    let npc_textures = r.unicode_array()?;
    let damage_effect = r.unicode()?;
    let boots_sounds = r.unicode_array()?;
    let drop_sound = r.unicode()?;
    let equip_sound = r.unicode()?;
    r.unicode_array()?;
    let mut attributes_58c = [0; 6];
    for value in &mut attributes_58c {
        *value = r.i32()?;
    }
    Ok(Armor { id, drop_sound, equip_sound, boots_sounds, body_part, rows, npc_meshes, npc_textures, damage_effect, attributes_58c, crystallizable: false })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Vec<Armor> {
        read(include_bytes!("../tests/fixtures/armorgrp-head.bin")).unwrap()
    }

    #[test]
    fn serialize_10438eb0_preserves_all_signed_attribute_slots() {
        let values = [i32::MIN, -1, 0, 1, 2, i32::MAX];
        let mut bytes = vec![0; (1 + 15 * 4 + 7) * 4];
        bytes.extend(values.into_iter().flat_map(i32::to_le_bytes));
        let mut reader = DatReader::new(&bytes);
        let armor = body(&mut reader, 42).unwrap();
        assert_eq!(armor.attributes_58c, values);
        assert_eq!(reader.remaining(), 0);
        for end in 0..bytes.len() {
            assert!(body(&mut DatReader::new(&bytes[..end]), 42).is_err());
        }
    }

    #[test]
    fn serialize_10438eb0_reads_the_damage_effect_string_at_558() {
        let name: Vec<u8> = "LineageEffect.p_u002_a".encode_utf16().chain([0]).flat_map(u16::to_le_bytes).collect();
        let mut bytes = vec![0u8; (1 + 15 * 4 + 2) * 4];
        bytes.extend((name.len() as u32).to_le_bytes());
        bytes.extend(&name);
        bytes.extend([0u8; 4 * 4]);
        bytes.extend([0u8; 6 * 4]);
        let mut reader = DatReader::new(&bytes);
        let armor = body(&mut reader, 7).unwrap();
        assert_eq!(armor.damage_effect, "LineageEffect.p_u002_a");
        assert!(armor.boots_sounds.is_empty() && armor.drop_sound.is_empty());
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn armorgrp_rows_are_pairs_of_main_and_ex_lists_per_chargrp_row() {
        let armors = fixture();
        assert_eq!(armors.len(), 3);
        let tunic = &armors[0];
        assert_eq!((tunic.id, tunic.body_part), (21, body_part::CHEST));
        assert!(armors.iter().all(|armor| armor.damage_effect == "LineageEffect.p_u002_a"));
        assert_eq!(tunic.rows.len(), ROWS);
        assert_eq!(tunic.rows[0].meshes, ["Fighter.MFighter_m001_u"]);
        assert_eq!(tunic.rows[0].textures, ["MFighter.MFighter_m001_t02_u"]);
        assert_eq!(tunic.rows[0].ex_meshes, [""]);
        assert!(tunic.rows[0].ex_textures.is_empty());
        assert_eq!(tunic.rows[1].meshes, ["Fighter.FFighter_m001_u"]);
        assert_eq!(tunic.rows[13].textures, ["FShaman.FShaman_m001_t02_u"]);
        assert!(tunic.rows[14].meshes.is_empty());
        assert_eq!(tunic.npc_meshes, [""]);
        assert_eq!(tunic.mesh(0, 0), Some("Fighter.MFighter_m001_u"));
        assert_eq!(tunic.texture(8, 0), Some("MMagic.MMagic_m005_t02_u"));
        assert_eq!(tunic.mesh(0, 1), None);
        assert_eq!(tunic.mesh(14, 0), None);
        assert_eq!(tunic.mesh(15, 0), None);
    }

    #[test]
    fn onepiece_and_alldress_carry_the_extra_pieces_by_index() {
        let armors = fixture();
        let onepiece = &armors[1];
        assert_eq!((onepiece.id, onepiece.body_part), (60, body_part::ONEPIECE));
        assert_eq!(onepiece.rows[0].meshes, ["Fighter.MFighter_m003_u", "Fighter.MFighter_m003_l"]);
        assert_eq!(onepiece.rows[0].textures, ["mfighter.mfighter_m003_t61_u", "MFighter.MFighter_m003_t61_l"]);
        let dress = &armors[2];
        assert_eq!((dress.id, dress.body_part), (6408, body_part::ALLDRESS));
        assert_eq!(
            dress.rows[0].meshes,
            [
                "Fighter.MFighter_m013_u",
                "Fighter.MFighter_m013_l",
                "Fighter.MFighter_m013_g",
                "Fighter.MFighter_m013_b"
            ]
        );
        assert_eq!(dress.texture(0, 3), Some("MFighter.MFighter_m013_t2000_b"));
        assert_eq!(dress.rows[0].ex_meshes, ["", "", "", ""]);
    }

    #[test]
    fn truncated_record_is_an_error() {
        let body = include_bytes!("../tests/fixtures/armorgrp-head.bin");
        assert!(read(&body[..body.len() - 3]).is_err());
        assert!(read(&[0, 0, 0, 0]).unwrap().is_empty());
    }
}
