use crate::{DatReader, Error};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq)]
pub struct Skill {
    pub id: u32,
    pub level: u32,
    pub oper_type: u32,
    pub mp_consume: u32,
    pub cast_range: i32,
    pub cast_style: u32,
    pub hit_time: f32,
    pub is_magic: i32,
    pub ani_char: String,
    pub visual_effect: String,
    pub icon: String,
    pub extra_eff: u32,
    pub is_ench: u32,
    pub ench_skill_id: u32,
    pub hp_consume: u32,
    pub rumble_self: i32,
    pub rumble_target: i32,
}

pub const TIP_TYPE_MAGIC_SYSSTRING: u32 = 313;
pub const TIP_TYPE_OPER_2_SYSSTRING: u32 = 312;
pub const TIP_TYPE_OTHER_SYSSTRING: u32 = 311;
pub const TIP_HP_SYSSTRING: u32 = 1195;
pub const TIP_MP_SYSSTRING: u32 = 320;
pub const TIP_RANGE_SYSSTRING: u32 = 321;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkillTip {
    pub type_sysstring: u32,
    pub hp_consume: u32,
    pub mp_consume: u32,
    pub cast_range: i32,
    pub enchant_level: Option<u32>,
}

impl SkillTip {
    pub fn numbers(&self) -> Vec<(u32, i64)> {
        let mut rows = Vec::new();
        if self.hp_consume != 0 {
            rows.push((TIP_HP_SYSSTRING, i64::from(self.hp_consume)));
        }
        if self.mp_consume != 0 {
            rows.push((TIP_MP_SYSSTRING, i64::from(self.mp_consume)));
        }
        if self.cast_range >= 0 {
            rows.push((TIP_RANGE_SYSSTRING, i64::from(self.cast_range)));
        }
        rows
    }

    pub fn display_level(&self, level: u32) -> u32 {
        self.enchant_level.unwrap_or(level)
    }
}

impl Skill {
    pub fn is_debuff(&self) -> bool {
        self.extra_eff != 0
    }

    pub fn tip(&self) -> SkillTip {
        let type_sysstring = if self.is_magic != 0 {
            TIP_TYPE_MAGIC_SYSSTRING
        } else if self.oper_type == 2 {
            TIP_TYPE_OPER_2_SYSSTRING
        } else {
            TIP_TYPE_OTHER_SYSSTRING
        };
        SkillTip {
            type_sysstring,
            hp_consume: self.hp_consume,
            mp_consume: self.mp_consume,
            cast_range: self.cast_range,
            enchant_level: (self.is_ench != 0).then_some(self.ench_skill_id),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SkillTable {
    rows: HashMap<(u32, u32), Skill>,
}

impl SkillTable {
    pub fn lookup(&self, id: u32, level: u32) -> Option<&Skill> {
        self.rows.get(&(id, level)).or_else(|| self.rows.get(&(id, 1)))
    }

    pub fn icons(&self) -> impl Iterator<Item = &str> {
        self.rows.values().map(|s| s.icon.as_str())
    }
}

impl Deref for SkillTable {
    type Target = HashMap<(u32, u32), Skill>;

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl DerefMut for SkillTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rows
    }
}

impl FromIterator<Skill> for SkillTable {
    fn from_iter<I: IntoIterator<Item = Skill>>(iter: I) -> Self {
        SkillTable { rows: iter.into_iter().map(|s| ((s.id, s.level), s)).collect() }
    }
}

pub fn by_id_level(skills: Vec<Skill>) -> SkillTable {
    skills.into_iter().collect()
}

pub fn read(body: &[u8]) -> Result<Vec<Skill>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count).map(|_| record(&mut reader)).collect()
}

fn record(r: &mut DatReader<'_>) -> Result<Skill, Error> {
    Ok(Skill {
        id: r.u32()?,
        level: r.u32()?,
        oper_type: r.u32()?,
        mp_consume: r.u32()?,
        cast_range: r.i32()?,
        cast_style: r.u32()?,
        hit_time: r.f32()?,
        is_magic: r.i32()?,
        ani_char: r.unicode()?,
        visual_effect: r.unicode()?,
        icon: r.unicode()?,
        extra_eff: r.u32()?,
        is_ench: r.u32()?,
        ench_skill_id: r.u32()?,
        hp_consume: r.u32()?,
        rumble_self: r.i32()?,
        rumble_target: r.i32()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skillgrp_first_records() {
        let skills = read(include_bytes!("../tests/fixtures/skillgrp-head.bin")).unwrap();
        assert_eq!(skills.len(), 3);
        assert_eq!((skills[0].id, skills[0].level), (3, 1));
        assert_eq!(
            (skills[0].oper_type, skills[0].mp_consume, skills[0].cast_range, skills[0].cast_style),
            (0, 10, 40, 3)
        );
        assert!((skills[0].hit_time - 1.08).abs() < 1e-6);
        assert_eq!(skills[0].is_magic, 0);
        assert_eq!(skills[0].ani_char, "S");
        assert_eq!(skills[0].visual_effect, "");
        assert_eq!(skills[0].icon, "icon.skill0003");
        assert_eq!(
            (skills[0].extra_eff, skills[0].is_ench, skills[0].ench_skill_id, skills[0].hp_consume),
            (0, 0, 0, 0)
        );
        assert_eq!((skills[0].rumble_self, skills[0].rumble_target), (9, 11));
        assert!(!skills[0].is_debuff());
        assert_eq!((skills[2].id, skills[2].level, skills[2].mp_consume), (3, 3, 11));
        let table: SkillTable = skills.into_iter().collect();
        assert_eq!(table.len(), 3);
        assert_eq!(table.lookup(3, 2).unwrap().mp_consume, 10);
        assert_eq!(table.lookup(3, 9).unwrap().level, 1);
        assert!(table.lookup(4, 1).is_none());
        assert_eq!(table.get(&(3, 2)).unwrap().level, 2);
        assert!(!table.contains_key(&(3, 9)));
        assert_eq!(table.icons().count(), 3);
    }

    fn unicode(s: &str) -> Vec<u8> {
        let mut out = (s.len() as u32 * 2).to_le_bytes().to_vec();
        out.extend(s.encode_utf16().flat_map(u16::to_le_bytes));
        out
    }

    pub(crate) fn power_strike(level: u32, mp: u32) -> Vec<u8> {
        let mut out = Vec::new();
        for v in [3u32, level, 0, mp] {
            out.extend(v.to_le_bytes());
        }
        out.extend(40i32.to_le_bytes());
        out.extend(3u32.to_le_bytes());
        out.extend(1.08f32.to_le_bytes());
        out.extend(0i32.to_le_bytes());
        out.extend(unicode("S"));
        out.extend(unicode(""));
        out.extend(unicode("icon.skill0003"));
        for v in [0u32, 0, 0, 0] {
            out.extend(v.to_le_bytes());
        }
        out.extend(9i32.to_le_bytes());
        out.extend(11i32.to_le_bytes());
        out
    }

    #[test]
    fn skill_records_follow_the_ms_item_serializer() {
        let mut body = 2u32.to_le_bytes().to_vec();
        body.extend(power_strike(1, 10));
        body.extend(power_strike(2, 10));
        let skills = read(&body).unwrap();
        assert_eq!(skills.len(), 2);
        let first = &skills[0];
        assert_eq!((first.id, first.level, first.mp_consume, first.cast_range, first.cast_style), (3, 1, 10, 40, 3));
        assert_eq!(first.hit_time, 1.08);
        assert_eq!(first.ani_char, "S");
        assert_eq!(first.icon, "icon.skill0003");
        assert_eq!((first.rumble_self, first.rumble_target), (9, 11));
        let table = by_id_level(skills);
        assert_eq!(table[&(3, 2)].level, 2);
        assert!(!table.contains_key(&(3, 3)));
        assert!(read(&body[..body.len() - 1]).is_err());
    }

    #[test]
    fn truncated_body_is_an_error() {
        let body = include_bytes!("../tests/fixtures/skillgrp-head.bin");
        assert!(read(&body[..body.len() - 1]).is_err());
    }

    fn blank() -> Skill {
        Skill {
            id: 1, level: 3, oper_type: 0, mp_consume: 0, cast_range: -1, cast_style: 0, hit_time: 0.0, is_magic: 0,
            ani_char: String::new(), visual_effect: String::new(), icon: String::new(), extra_eff: 0, is_ench: 0,
            ench_skill_id: 0, hp_consume: 0, rumble_self: 0, rumble_target: 0,
        }
    }

    #[test]
    fn the_tip_follows_the_tooltip_builder_100307a0() {
        let plain = blank().tip();
        assert_eq!(plain.type_sysstring, TIP_TYPE_OTHER_SYSSTRING);
        assert!(plain.numbers().is_empty());
        assert_eq!(plain.display_level(3), 3);
        let magic = Skill { is_magic: 1, oper_type: 2, hp_consume: 4, mp_consume: 9, cast_range: 0, ..blank() }.tip();
        assert_eq!(magic.type_sysstring, TIP_TYPE_MAGIC_SYSSTRING);
        assert_eq!(magic.numbers(), [(TIP_HP_SYSSTRING, 4), (TIP_MP_SYSSTRING, 9), (TIP_RANGE_SYSSTRING, 0)]);
        assert_eq!(Skill { oper_type: 2, ..blank() }.tip().type_sysstring, TIP_TYPE_OPER_2_SYSSTRING);
        let enchanted = Skill { is_ench: 1, ench_skill_id: 28, ..blank() }.tip();
        assert_eq!(enchanted.display_level(101), 28);
    }
}
