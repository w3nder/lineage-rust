use crate::chat::wrap_line;
use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::item_stats;

fn text(offset: u32, lookup: &impl Fn(u32) -> Option<String>) -> String {
    lookup((offset - 0x9871c) / 12).unwrap_or_default()
}

pub fn weapon_name(kind: i32, lookup: &impl Fn(u32) -> Option<String>) -> String {
    match kind {
        1..=7 => text(0x98920 + (kind as u32 - 1) * 12, lookup),
        8 => text(0x99ebc, lookup),
        _ => String::new(),
    }
}

pub fn speed_name(speed: i32, lookup: &impl Fn(u32) -> Option<String>) -> String {
    text(match speed {
        ..260 => 0x995a4,
        260..309 => 0x99598,
        309..352 => 0x9958c,
        352..406 => 0x99580,
        _ => 0x99574,
    }, lookup)
}

pub fn type_name(kind: i32, body: i32, armor_type: i32, lookup: &impl Fn(u32) -> Option<String>) -> String {
    let offset = match (kind, body) {
        (0, 0x80 | 0x100) => 0x99274,
        (0, 0x4000) => 0x99280,
        (1, 1) => 0x991d8,
        (1, 0x40) => 0x991e4,
        (1, 0x80 | 0x100) => 0x991f0,
        (1, 0x200) => 0x991fc,
        (1, 0x1000) => 0x99208,
        (1, 0x2000) => 0x99214,
        (1, 0x10000) => 0x9b71c,
        (1, 0x400 | 0x800 | 0x8000) => {
            let prefix = text(match body { 0x400 => 0x99220, 0x800 => 0x9922c, _ => 0x9a084 }, lookup);
            let suffix = match armor_type {
                1 => text(0x99298, lookup),
                2 => text(0x992a4, lookup),
                3 => text(0x9928c, lookup),
                _ => String::new(),
            };
            return format!("{prefix} / {suffix}");
        }
        (2, mask) if mask & 6 != 0 => 0x99238,
        (2, mask) if mask & 8 != 0 => 0x99244,
        (2, mask) if mask & 0x30 != 0 => 0x99250,
        (3, _) => 0x9925c,
        (4, _) => 0x99268,
        _ => return String::new(),
    };
    text(offset, lookup)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemDetail {
    pub icon: String,
    pub name: String,
    pub additional: Option<String>,
    pub description: String,
    pub type2: i32,
    pub body_part: i32,
    pub weapon_type: i32,
    pub armor_type: i32,
    pub grade: usize,
    pub enchant: i32,
    pub count: i32,
    pub packet_type2: i32,
    pub physical_attack: i32,
    pub magical_attack: i32,
    pub physical_defense: i32,
    pub magical_defense: i32,
    pub shield_defense: i32,
    pub shield_rate: i32,
    pub mp_bonus: i32,
    pub evasion: i32,
    pub speed: i32,
    pub soulshots: i32,
    pub spiritshots: i32,
    pub weight: i32,
    pub shows_mp_bonus: bool,
}

impl ItemDetail {
    pub fn name_label(&self) -> crate::item_name::ItemNameLabel {
        crate::item_name::ItemNameLabel {
            name: self.name.clone(),
            additional: self.additional.clone(),
            type2: self.packet_type2,
            enchant: self.enchant,
            grade: self.grade as i32,
            count: self.count,
        }
    }

    pub fn with_weapon(mut self, weapon: &l2_dat::Weapon) -> Self {
        self.weapon_type = weapon.weapon_type as i32;
        self.grade = weapon.attributes_dc[0] as usize;
        self.physical_attack = weapon.attributes_cc[1];
        self.magical_attack = weapon.attributes_cc[2];
        self.evasion = weapon.attributes_dc[3];
        self.shield_defense = weapon.attributes_dc[4];
        self.shield_rate = weapon.attributes_dc[5];
        self.speed = weapon.attributes_dc[6];
        self.soulshots = weapon.attributes_dc[8];
        self.spiritshots = weapon.attributes_dc[9];
        self
    }

    pub fn with_armor(mut self, armor: &l2_dat::Armor) -> Self {
        self.armor_type = armor.attributes_58c[0];
        self.shows_mp_bonus = self.armor_type == 3;
        self.grade = armor.attributes_58c[1] as usize;
        self.evasion = armor.attributes_58c[2];
        self.physical_defense = armor.attributes_58c[3];
        self.magical_defense = armor.attributes_58c[4];
        self.mp_bonus = armor.attributes_58c[5];
        self
    }

    pub fn description_rows(&self, font: &BitmapFont) -> Vec<String> {
        if self.description.is_empty() { return Vec::new(); }
        self.description.replace("\\n", "\r\n").split('\n')
            .flat_map(|line| wrap_line(line.trim_end_matches('\r'), 200, 0, font).into_iter().map(|(_, row)| row))
            .collect()
    }

    pub fn rows(&self, lookup: &impl Fn(u32) -> Option<String>) -> Vec<(String, String)> {
        let mut rows = Vec::new();
        let mut add = |offset, value: Option<i32>| {
            if let Some(value) = value { rows.push((text(offset, lookup), value.to_string())); }
        };
        match self.type2 {
            0 => {
                add(0x98b84, item_stats::physical_attack(self.weapon_type, self.body_part, self.grade, self.enchant, self.physical_attack));
                add(0x98bb4, item_stats::magical_attack(self.weapon_type, self.body_part, self.grade, self.enchant, self.magical_attack));
                rows.push((text(0x98c50, lookup), speed_name(self.speed, lookup)));
                rows.push((text(0x99a0c, lookup), format!("X {}", self.soulshots)));
                rows.push((text(0x99e5c, lookup), format!("X {}", self.spiritshots)));
            }
            1 if self.body_part & 0x180 != 0 => {
                add(0x98b90, item_stats::defense(self.grade, self.enchant, self.shield_defense));
                add(0x995f8, Some(self.shield_rate));
                add(0x98ba8, Some(self.evasion));
            }
            1 => {
                if self.shows_mp_bonus { add(0x9994c, Some(self.mp_bonus)); }
                add(0x98b90, item_stats::defense(self.grade, self.enchant, self.physical_defense));
            }
            2 => add(0x98bc0, item_stats::defense(self.grade, self.enchant, self.magical_defense)),
            _ => {}
        }
        if matches!(self.type2, 0..=2 | 5) {
            rows.push((text(0x9898c, lookup), self.weight.to_string()));
        }
        rows
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, lookup: &impl Fn(u32) -> Option<String>) -> i32 {
        let (x, y) = origin;
        list.draw_texture(x + 9, y + 3, 34, 34, 0, 0, 34, 34, "L2ui_ch3.etc.iconbox", 1.0);
        list.draw_texture(x + 10, y + 4, 32, 32, 0, 0, 32, 32, &self.icon, 1.0);
        let line = font.line_height as i32;
        let mut baseline = 12 + line;
        let type_name = type_name(self.type2, self.body_part, self.armor_type, lookup);
        let heading = if self.type2 == 0 { format!("{} / {type_name}", weapon_name(self.weapon_type, lookup)) } else { type_name };
        if matches!(self.type2, 0..=3) {
            list.draw_text(x + 46, y + baseline, Argb(0xffb09b79), &heading, font);
        }
        if matches!(self.type2, 0..=2) { baseline += 20; }
        let rows = self.rows(lookup);
        let row_x = x + if self.type2 == 5 { 46 } else { 10 };
        for (index, (label, value)) in rows.iter().enumerate() {
            if index > 0 { baseline += line + 4; }
            let end = list.draw_text(row_x, y + baseline, Argb(0xffa3a3a3), label, font);
            let end = list.draw_text(end, y + baseline, Argb(0xffdcdcdc), " : ", font);
            list.draw_text(end, y + baseline, Argb(0xffb09b79), value, font);
        }
        if !self.description.is_empty() {
            baseline += line + 4;
            for row in self.description_rows(font) {
                list.draw_text(x + 10, y + baseline, Argb(0xffb2becf), &row, font);
                baseline += line;
            }
        }
        baseline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup(id: u32) -> Option<String> { Some(id.to_string()) }

    fn font() -> BitmapFont {
        let glyphs = (32u8..=126).map(|c| (char::from(c), crate::font::GlyphRect { x: i32::from(c), width: 6, y: 0, height: 12 })).collect();
        BitmapFont::new("font".into(), crate::font::Glyphs { tex_width: 256, tex_height: 16, glyphs })
    }

    #[test]
    fn draw_single_item_100a0300_places_icon_header_rows_and_description() {
        let item = ItemDetail { type2: 2, body_part: 8, grade: 1, enchant: 4, magical_defense: 20, weight: 90, icon: "icon.item".into(), description: "a\\nb".into(), ..Default::default() };
        let mut list = DrawList::default();
        let height = item.paint(&mut list, (100, 200), &font(), &lookup);
        assert_eq!(height, 100);
        assert_eq!((list.quads[0].x, list.quads[0].y, list.quads[0].w, list.quads[0].h), (109.0, 203.0, 34.0, 34.0));
        assert_eq!((list.quads[1].x, list.quads[1].y), (110.0, 204.0));
        assert_eq!((list.quads[2].x, list.quads[2].y), (146.0, 224.0));
        let description: Vec<_> = list.quads.iter().filter(|q| q.color == Argb(0xffb2becf).rgba()).collect();
        assert_eq!(description.iter().map(|q| (q.x, q.y)).collect::<Vec<_>>(), [(110.0, 276.0), (110.0, 288.0)]);
        let mut consumable = item;
        consumable.type2 = 5;
        consumable.description.clear();
        list.quads.clear();
        assert_eq!(consumable.paint(&mut list, (0, 0), &font(), &lookup), 24);
        assert_eq!((list.quads[2].x, list.quads[2].y), (46.0, 24.0));
    }

    #[test]
    fn description_1006b290_converts_literal_newlines_and_wraps_at_200() {
        let item = ItemDetail { description: "a\\n\\nb\r\nc".into(), ..Default::default() };
        assert_eq!(item.description_rows(&font()), ["a", "", "b", "c"]);
        let item = ItemDetail { description: "x".repeat(34), ..Default::default() };
        assert_eq!(item.description_rows(&font()), ["x".repeat(33), "x".into()]);
    }

    #[test]
    fn atk_speed_name_1006b120_preserves_every_boundary() {
        for (value, offset) in [(i32::MIN, 0x995a4), (259, 0x995a4), (260, 0x99598), (308, 0x99598), (309, 0x9958c), (351, 0x9958c), (352, 0x99580), (405, 0x99580), (406, 0x99574), (i32::MAX, 0x99574)] {
            assert_eq!(speed_name(value, &lookup), text(offset, &lookup));
        }
    }

    #[test]
    fn item_type_name_1006ae30_preserves_masks_and_armor_suffix() {
        assert_eq!(type_name(2, 0x3e, 0, &lookup), text(0x99238, &lookup));
        assert_eq!(type_name(2, 0x38, 0, &lookup), text(0x99244, &lookup));
        assert_eq!(type_name(2, 0x30, 0, &lookup), text(0x99250, &lookup));
        for body in [0x400, 0x800, 0x8000] {
            assert!(type_name(1, body, 0, &lookup).ends_with(" / "));
            assert!(type_name(1, body, 3, &lookup).ends_with(&text(0x9928c, &lookup)));
        }
        assert_eq!(type_name(0, 0, 0, &lookup), "");
        assert_eq!(weapon_name(0, &lookup), "");
        assert_eq!(weapon_name(8, &lookup), text(0x99ebc, &lookup));
    }

    #[test]
    fn draw_single_item_100a0300_uses_all_defense_branches() {
        let mut item = ItemDetail { type2: 1, grade: 1, enchant: 4, physical_defense: 10, magical_defense: 20, shield_defense: 30, mp_bonus: 40, shield_rate: 50, evasion: -8, weight: 90, ..Default::default() };
        assert_eq!(item.rows(&lookup).iter().map(|r| r.1.as_str()).collect::<Vec<_>>(), ["16", "90"]);
        item.shows_mp_bonus = true;
        assert_eq!(item.rows(&lookup).iter().map(|r| r.1.as_str()).collect::<Vec<_>>(), ["40", "16", "90"]);
        item.body_part = 0x180;
        assert_eq!(item.rows(&lookup).iter().map(|r| r.1.as_str()).collect::<Vec<_>>(), ["36", "50", "-8", "90"]);
        item.type2 = 2;
        assert_eq!(item.rows(&lookup).iter().map(|r| r.1.as_str()).collect::<Vec<_>>(), ["26", "90"]);
        item.type2 = 5;
        assert_eq!(item.rows(&lookup).iter().map(|r| r.1.as_str()).collect::<Vec<_>>(), ["90"]);
        item.type2 = 3;
        assert!(item.rows(&lookup).is_empty());
    }
}
