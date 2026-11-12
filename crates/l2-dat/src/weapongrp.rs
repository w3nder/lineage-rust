use crate::armorgrp::item_head;
use crate::{DatReader, Error};

pub mod anim_type {
    pub const HAND: u32 = 0;
    pub const ONE_HAND_SWORD: u32 = 1;
    pub const TWO_HAND_SWORD: u32 = 2;
    pub const DUAL: u32 = 3;
    pub const POLE: u32 = 4;
    pub const BOW: u32 = 5;
    pub const THROW: u32 = 6;
    pub const DUAL_FIST: u32 = 7;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Weapon {
    pub id: u32,
    pub drop_sound: String,
    pub equip_sound: String,
    pub attack_sounds: Vec<String>,
    pub body_part: u32,
    pub handness: u32,
    pub meshes: Vec<String>,
    pub textures: Vec<String>,
    pub weapon_type: u32,
    pub attributes_cc: [i32; 3],
    pub attributes_dc: [i32; 10],
    pub curvature: u32,
    pub sub_mesh_skipped: bool,
    pub crest_texture: u32,
    pub hero_weapon: i32,
    pub effects: Vec<WeaponEffect>,
    pub enchant_meshes: Vec<EnchantMesh>,
    pub crystallizable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeaponEffect {
    pub name: String,
    pub offset: [f32; 3],
    pub parameter_1c4: f32,
    pub parameter_1d4: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnchantMesh {
    pub name: String,
    pub vector_124: [f32; 3],
    pub vector_154: [f32; 3],
}

impl Weapon {
    pub fn mesh(&self, index: usize) -> Option<&str> {
        self.meshes.get(index).map(String::as_str).filter(|m| !m.is_empty())
    }

    pub fn texture(&self, index: usize) -> Option<&str> {
        self.textures.get(index).map(String::as_str).filter(|t| !t.is_empty())
    }

    pub fn is_dual(&self) -> bool {
        matches!(self.handness, anim_type::DUAL | anim_type::DUAL_FIST)
    }
}

pub fn read(body: &[u8]) -> Result<Vec<Weapon>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count).map(|_| record(&mut reader)).collect()
}

pub fn read_as(body: &[u8], chronicle: crate::Chronicle) -> Result<Vec<Weapon>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count).map(|_| record_as(&mut reader, chronicle)).collect()
}

fn record(r: &mut DatReader<'_>) -> Result<Weapon, Error> {
    record_as(r, crate::Chronicle::C4)
}

fn record_as(r: &mut DatReader<'_>, chronicle: crate::Chronicle) -> Result<Weapon, Error> {
    let head = item_head(r)?;
    let mut weapon = body_as(r, head.id, chronicle)?;
    weapon.crystallizable = head.crystallizable;
    Ok(weapon)
}

pub fn body(r: &mut DatReader<'_>, id: u32) -> Result<Weapon, Error> {
    body_as(r, id, crate::Chronicle::C4)
}

pub fn body_as(r: &mut DatReader<'_>, id: u32, chronicle: crate::Chronicle) -> Result<Weapon, Error> {
    let body_part = r.u32()?;
    let handness = r.u32()?;
    let meshes = r.unicode_array()?;
    let textures = r.unicode_array()?;
    let attack_sounds = r.unicode_array()?;
    let drop_sound = r.unicode()?;
    let equip_sound = r.unicode()?;
    r.unicode()?;
    let mut attributes_cc = [0; 3];
    for value in &mut attributes_cc {
        *value = r.i32()?;
    }
    let weapon_type = r.u32()?;
    let mut attributes_dc = [0; 10];
    for value in &mut attributes_dc {
        *value = r.i32()?;
    }
    let curvature = r.u32()?;
    let sub_mesh_skipped = r.u32()? != 0;
    let hero_weapon = r.i32()?;
    let crest_texture = r.u32()?;
    let per_mesh = meshes.len();
    let mut effects = Vec::with_capacity(per_mesh);
    for _ in 0..per_mesh {
        effects.push(WeaponEffect { name: r.unicode()?, offset: [0.0; 3], parameter_1c4: 0.0, parameter_1d4: 0.0 });
    }
    for effect in &mut effects {
        effect.offset = [r.f32()?, r.f32()?, r.f32()?];
    }
    for effect in &mut effects {
        effect.parameter_1c4 = r.f32()?;
    }
    for effect in &mut effects {
        effect.parameter_1d4 = r.f32()?;
    }
    let mut enchant_meshes = Vec::with_capacity(per_mesh);
    for _ in 0..per_mesh {
        enchant_meshes.push(EnchantMesh { name: r.unicode()?, vector_124: [0.0; 3], vector_154: [0.0; 3] });
    }
    for enchant in &mut enchant_meshes {
        enchant.vector_124 = [r.f32()?, r.f32()?, r.f32()?];
    }
    for enchant in &mut enchant_meshes {
        enchant.vector_154 = [r.f32()?, r.f32()?, r.f32()?];
    }
    if chronicle == crate::Chronicle::Interlude {
        for _ in 0..4 {
            r.i32()?;
        }
        for _ in 0..4 {
            r.unicode()?;
        }
    }
    Ok(Weapon { id, drop_sound, equip_sound, attack_sounds, body_part, handness, meshes, textures, weapon_type, attributes_cc, attributes_dc, curvature, sub_mesh_skipped, crest_texture, hero_weapon, effects, enchant_meshes, crystallizable: false })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::armorgrp::body_part;

    fn fixture() -> Vec<Weapon> {
        read(include_bytes!("../tests/fixtures/weapongrp-head.bin")).unwrap()
    }

    #[test]
    fn serialize_10439bf0_keeps_separate_arrays_for_two_weapon_meshes() {
        fn string(bytes: &mut Vec<u8>, value: &str) {
            bytes.extend(((value.len() * 2) as u32).to_le_bytes());
            bytes.extend(value.encode_utf16().flat_map(u16::to_le_bytes));
        }
        let mut bytes = vec![0; 8];
        bytes.extend(2u32.to_le_bytes());
        string(&mut bytes, "Weapon.Right");
        string(&mut bytes, "Weapon.Left");
        bytes.extend([0u32; 5].into_iter().flat_map(u32::to_le_bytes));
        bytes.extend([0u32; 16].into_iter().flat_map(u32::to_le_bytes));
        bytes.extend((-1i32).to_le_bytes());
        bytes.extend(0u32.to_le_bytes());
        string(&mut bytes, "Effect.Right");
        string(&mut bytes, "Effect.Left");
        bytes.extend((1..=10).flat_map(|v| (v as f32).to_le_bytes()));
        string(&mut bytes, "Aura.Right");
        string(&mut bytes, "Aura.Left");
        bytes.extend((11..=22).flat_map(|v| (v as f32).to_le_bytes()));
        let mut reader = DatReader::new(&bytes);
        let weapon = body(&mut reader, 42).unwrap();
        assert_eq!(reader.remaining(), 0);
        assert_eq!(weapon.hero_weapon, -1);
        assert_eq!(weapon.effects, [
            WeaponEffect { name: "Effect.Right".into(), offset: [1.0, 2.0, 3.0], parameter_1c4: 7.0, parameter_1d4: 9.0 },
            WeaponEffect { name: "Effect.Left".into(), offset: [4.0, 5.0, 6.0], parameter_1c4: 8.0, parameter_1d4: 10.0 },
        ]);
        assert_eq!(weapon.enchant_meshes, [
            EnchantMesh { name: "Aura.Right".into(), vector_124: [11.0, 12.0, 13.0], vector_154: [17.0, 18.0, 19.0] },
            EnchantMesh { name: "Aura.Left".into(), vector_124: [14.0, 15.0, 16.0], vector_154: [20.0, 21.0, 22.0] },
        ]);
        for end in 0..bytes.len() {
            assert!(body(&mut DatReader::new(&bytes[..end]), 42).is_err());
        }
    }

    #[test]
    fn serialize_10439bf0_preserves_all_signed_attribute_slots() {
        let cc = [i32::MIN, -1, i32::MAX];
        let dc = [-9, -8, -7, -6, -5, -4, -3, -2, -1, 0];
        let mut bytes = vec![0; 8 * 4];
        bytes.extend(cc.into_iter().flat_map(i32::to_le_bytes));
        bytes.extend(6u32.to_le_bytes());
        bytes.extend(dc.into_iter().flat_map(i32::to_le_bytes));
        bytes.extend([1000u32, 1, 0, 7].into_iter().flat_map(u32::to_le_bytes));
        let mut reader = DatReader::new(&bytes);
        let weapon = body(&mut reader, 42).unwrap();
        assert_eq!(weapon.attributes_cc, cc);
        assert_eq!(weapon.attributes_dc, dc);
        assert_eq!((weapon.weapon_type, weapon.curvature, weapon.crest_texture), (6, 1000, 7));
        assert!(weapon.sub_mesh_skipped);
        assert_eq!(reader.remaining(), 0);
        for end in 0..bytes.len() {
            assert!(body(&mut DatReader::new(&bytes[..end]), 42).is_err());
        }
    }

    #[test]
    fn weapongrp_records_carry_handness_meshes_and_textures() {
        let weapons = fixture();
        assert_eq!(weapons.len(), 5);
        let sword = &weapons[0];
        assert_eq!((sword.id, sword.body_part, sword.handness), (1, body_part::RHAND, anim_type::ONE_HAND_SWORD));
        assert_eq!(sword.meshes, ["LineageWeapons.small_sword_m00_wp"]);
        assert_eq!(sword.textures, ["LineageWeaponsTex.small_sword_t00_wp"]);
        assert_eq!((sword.weapon_type, sword.curvature, sword.sub_mesh_skipped), (1, 1000, false));
        assert!(!sword.is_dual());
        let shield = &weapons[1];
        assert_eq!((shield.id, shield.body_part, shield.handness), (18, body_part::LHAND, anim_type::HAND));
        assert_eq!(shield.meshes, ["LineageWeapons.leather_shield_m00_sh"]);
        assert_eq!(shield.textures.len(), 2);
        let bow = &weapons[2];
        assert_eq!((bow.id, bow.body_part, bow.handness), (13, body_part::LRHAND, anim_type::BOW));
        assert_eq!(bow.mesh(0), Some("LineageWeapons.short_bow_m00_wp"));
    }

    #[test]
    fn dual_swords_have_two_meshes_and_per_mesh_trailers() {
        let weapons = fixture();
        let dual = &weapons[3];
        assert_eq!((dual.id, dual.handness), (1299, anim_type::DUAL));
        assert!(dual.is_dual());
        assert_eq!(dual.meshes, ["LineageWeapons.sword_of_delusion_m00_wp", "LineageWeapons.sword_of_magic_m00_wp"]);
        assert_eq!(dual.texture(1), Some("LineageWeaponsTex.sword_of_magic_t00_wp"));
        assert_eq!(dual.mesh(2), None);
        let rod = &weapons[4];
        assert_eq!((rod.id, rod.handness, rod.sub_mesh_skipped), (6529, anim_type::POLE, true));
        assert!(weapons.iter().all(|w| w.crest_texture == 0));
    }

    #[test]
    fn interlude_body_appends_four_slots_and_four_icon_strings() {
        fn string(bytes: &mut Vec<u8>, value: &str) {
            bytes.extend(((value.len() * 2) as u32).to_le_bytes());
            bytes.extend(value.encode_utf16().flat_map(u16::to_le_bytes));
        }
        let mut bytes = vec![0; 8];
        bytes.extend(1u32.to_le_bytes());
        string(&mut bytes, "Weapon.Right");
        bytes.extend([0u32; 5].into_iter().flat_map(u32::to_le_bytes));
        bytes.extend([0u32; 16].into_iter().flat_map(u32::to_le_bytes));
        bytes.extend((-1i32).to_le_bytes());
        bytes.extend(0u32.to_le_bytes());
        string(&mut bytes, "");
        bytes.extend([0u8; 12 + 8]);
        string(&mut bytes, "");
        bytes.extend([0u8; 24]);
        let c4_len = bytes.len();
        bytes.extend([4i32, 4, -1, -1].into_iter().flat_map(i32::to_le_bytes));
        string(&mut bytes, "icon.a");
        string(&mut bytes, "icon.b");
        string(&mut bytes, "");
        string(&mut bytes, "");
        let mut reader = DatReader::new(&bytes);
        let weapon = body_as(&mut reader, 7, crate::Chronicle::Interlude).unwrap();
        assert_eq!(reader.remaining(), 0);
        assert_eq!(weapon.id, 7);
        let mut c4 = DatReader::new(&bytes);
        let weapon = body_as(&mut c4, 7, crate::Chronicle::C4).unwrap();
        assert_eq!(c4.remaining(), bytes.len() - c4_len);
        assert_eq!(weapon.id, 7);
        for end in c4_len..bytes.len() {
            assert!(body_as(&mut DatReader::new(&bytes[..end]), 7, crate::Chronicle::Interlude).is_err());
        }
    }

    #[test]
    fn truncated_record_is_an_error() {
        let body = include_bytes!("../tests/fixtures/weapongrp-head.bin");
        assert!(read(&body[..body.len() - 3]).is_err());
        assert!(read(&[0, 0, 0, 0]).unwrap().is_empty());
    }
}
