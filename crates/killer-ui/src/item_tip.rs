use crate::item_stats;
use crate::multisell_detail::{speed_name, type_name, weapon_name};
use crate::tooltip::{TooltipLine, VALUE_COLOR};

pub const SYS_PATK: u32 = 94;
pub const SYS_PDEF: u32 = 95;
pub const SYS_EVASION: u32 = 97;
pub const SYS_MATK: u32 = 98;
pub const SYS_MDEF: u32 = 99;
pub const SYS_ATK_SPEED: u32 = 111;
pub const SYS_WEIGHT: u32 = 52;
pub const SYS_MP_COST: u32 = 320;
pub const SYS_DEF_RATE: u32 = 317;
pub const SYS_MP_INCREASE: u32 = 388;
pub const SYS_SOULSHOT: u32 = 404;
pub const SYS_SPIRITSHOT: u32 = 496;
pub const SYS_LEVEL: u32 = 88;
pub const SYS_PET_NAME: u32 = 969;
pub const SYS_PET_NAMED: u32 = 970;
pub const SYS_PET_UNNAMED: u32 = 971;
pub const SYS_SEVEN_SIGNS: u32 = 972;
pub const SYS_EVENT_NUMBER: u32 = 670;
pub const SYS_SELECTED_NUMBER: u32 = 671;
pub const SYS_PURCHASE_PRICE: u32 = 744;
pub const ETC_PET_COLLAR: i32 = 7;
pub const ETC_LOTTO: i32 = 13;
pub const ETC_RACE_TICKET: i32 = 14;
pub const ETC_SEVEN_SIGNS: i32 = 15;
pub const SHIELD_BODY: i32 = 0x180;
pub const ROBE: i32 = 3;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ItemTipData {
    pub type2: i32,
    pub body_part: i32,
    pub weight: i32,
    pub grade: usize,
    pub weapon_kind: i32,
    pub physical_attack: i32,
    pub magical_attack: i32,
    pub speed: i32,
    pub mp_cost: i32,
    pub soulshots: i32,
    pub spiritshots: i32,
    pub armor_type: i32,
    pub evasion: i32,
    pub physical_defense: i32,
    pub magical_defense: i32,
    pub mp_bonus: i32,
    pub shield_defense: i32,
    pub shield_rate: i32,
    pub etc_family: i32,
    pub etc_stackable: i32,
    pub custom1: i32,
    pub custom2: i32,
}

impl ItemTipData {
    pub fn with_weapon(mut self, weapon: &l2_dat::Weapon) -> Self {
        self.weapon_kind = weapon.weapon_type as i32;
        self.grade = weapon.attributes_dc[0].max(0) as usize;
        self.physical_attack = weapon.attributes_cc[1];
        self.magical_attack = weapon.attributes_cc[2];
        self.evasion = weapon.attributes_dc[3];
        self.shield_defense = weapon.attributes_dc[4];
        self.shield_rate = weapon.attributes_dc[5];
        self.speed = weapon.attributes_dc[6];
        self.mp_cost = weapon.attributes_dc[7];
        self.soulshots = weapon.attributes_dc[8];
        self.spiritshots = weapon.attributes_dc[9];
        self
    }

    pub fn with_armor(mut self, armor: &l2_dat::Armor) -> Self {
        self.armor_type = armor.attributes_58c[0];
        self.grade = armor.attributes_58c[1].max(0) as usize;
        self.evasion = armor.attributes_58c[2];
        self.physical_defense = armor.attributes_58c[3];
        self.magical_defense = armor.attributes_58c[4];
        self.mp_bonus = armor.attributes_58c[5];
        self
    }

    pub fn with_etc(mut self, etc: &l2_dat::EtcItem) -> Self {
        self.etc_family = etc.family as i32;
        self.etc_stackable = etc.stackable as i32;
        self.grade = etc.grade as usize;
        self
    }

    pub fn counted(&self) -> bool {
        matches!(self.type2, 3..=5) && (1..=3).contains(&self.etc_stackable)
    }
}

fn row(label: u32, value: impl ToString, lookup: &impl Fn(u32) -> Option<String>) -> TooltipLine {
    TooltipLine { label: lookup(label).unwrap_or_default(), value: value.to_string(), color: VALUE_COLOR }
}

fn kind_row(value: String) -> TooltipLine {
    TooltipLine { label: String::new(), value, color: VALUE_COLOR }
}

fn lotto_numbers(first: i32, second: i32) -> String {
    let mut numbers = Vec::new();
    for bit in 0..16 {
        if first & (1 << bit) != 0 {
            numbers.push(bit + 1);
        }
    }
    for bit in 0..16 {
        if second & (1 << bit) != 0 {
            numbers.push(bit + 17);
        }
    }
    numbers.iter().map(i32::to_string).collect::<Vec<_>>().join(",")
}

fn ticket_numbers(custom1: i32) -> String {
    let packed = custom1 & 0xffff;
    let mut text = String::new();
    if packed & 0xf != 0 {
        text.push_str(&(packed & 0xf).to_string());
    }
    if packed >> 4 > 0 {
        text.push(',');
        text.push_str(&(packed >> 4).to_string());
    }
    text
}

pub fn lines(data: &ItemTipData, enchant: i32, lookup: &impl Fn(u32) -> Option<String>) -> Vec<TooltipLine> {
    let mut out = Vec::new();
    let weight = || row(SYS_WEIGHT, data.weight, lookup);
    match data.type2 {
        0 => {
            let kind = weapon_name(data.weapon_kind, lookup);
            if !kind.is_empty() {
                out.push(kind_row(format!("{kind} / {}", type_name(0, data.body_part, 0, lookup))));
            }
            if let Some(value) =
                item_stats::physical_attack(data.weapon_kind, data.body_part, data.grade, enchant, data.physical_attack)
            {
                out.push(row(SYS_PATK, value, lookup));
            }
            if let Some(value) =
                item_stats::magical_attack(data.weapon_kind, data.body_part, data.grade, enchant, data.magical_attack)
            {
                out.push(row(SYS_MATK, value, lookup));
            }
            out.push(row(SYS_ATK_SPEED, speed_name(data.speed, lookup), lookup));
            if data.soulshots > 0 {
                out.push(row(SYS_SOULSHOT, format!("X {}", data.soulshots), lookup));
            }
            if data.spiritshots > 0 {
                out.push(row(SYS_SPIRITSHOT, format!("X {}", data.spiritshots), lookup));
            }
            out.push(weight());
            if data.mp_cost != 0 {
                out.push(row(SYS_MP_COST, data.mp_cost, lookup));
            }
        }
        1 if data.body_part & SHIELD_BODY != 0 => {
            if let Some(value) = item_stats::defense(data.grade, enchant, data.shield_defense) {
                out.push(row(SYS_PDEF, value, lookup));
            }
            out.push(row(SYS_DEF_RATE, data.shield_rate, lookup));
            out.push(row(SYS_EVASION, data.evasion, lookup));
            out.push(weight());
        }
        1 => {
            out.push(kind_row(type_name(1, data.body_part, data.armor_type, lookup)));
            if data.armor_type == ROBE {
                out.push(row(SYS_MP_INCREASE, data.mp_bonus, lookup));
            }
            if let Some(value) = item_stats::defense(data.grade, enchant, data.physical_defense) {
                out.push(row(SYS_PDEF, value, lookup));
            }
            out.push(weight());
        }
        2 => {
            out.push(kind_row(type_name(2, data.body_part, 0, lookup)));
            if let Some(value) = item_stats::defense(data.grade, enchant, data.magical_defense) {
                out.push(row(SYS_MDEF, value, lookup));
            }
            out.push(weight());
        }
        3 => out.push(kind_row(type_name(3, data.body_part, 0, lookup))),
        4 => {}
        _ => match data.etc_family {
            ETC_PET_COLLAR => {
                let named = match data.custom2 {
                    0 => lookup(SYS_PET_UNNAMED).unwrap_or_default(),
                    1 => lookup(SYS_PET_NAMED).unwrap_or_default(),
                    _ => String::new(),
                };
                out.push(row(SYS_PET_NAME, named, lookup));
                out.push(row(SYS_LEVEL, enchant, lookup));
                out.push(weight());
            }
            ETC_SEVEN_SIGNS => {
                out.push(row(SYS_SEVEN_SIGNS, enchant, lookup));
                out.push(weight());
            }
            ETC_LOTTO => {
                out.push(row(SYS_EVENT_NUMBER, data.custom1 & 0xffff, lookup));
                out.push(row(SYS_SELECTED_NUMBER, lotto_numbers(enchant, data.custom2), lookup));
                out.push(weight());
            }
            ETC_RACE_TICKET => {
                out.push(row(SYS_EVENT_NUMBER, enchant, lookup));
                out.push(row(SYS_SELECTED_NUMBER, ticket_numbers(data.custom1), lookup));
                out.push(row(SYS_PURCHASE_PRICE, i64::from(data.custom2) * 100, lookup));
                out.push(weight());
            }
            _ => out.push(weight()),
        },
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup(id: u32) -> Option<String> {
        Some(format!("s{id}"))
    }

    fn values(lines: &[TooltipLine]) -> Vec<(&str, &str)> {
        lines.iter().map(|line| (line.label.as_str(), line.value.as_str())).collect()
    }

    #[test]
    fn a_weapon_lists_type_attacks_speed_shots_weight_and_mp_cost_in_that_order() {
        let data = ItemTipData {
            type2: 0,
            weapon_kind: 1,
            body_part: 0x80,
            grade: 1,
            physical_attack: 50,
            magical_attack: 30,
            speed: 330,
            soulshots: 1,
            spiritshots: 0,
            weight: 1300,
            mp_cost: 4,
            ..Default::default()
        };
        let lines = lines(&data, 0, &lookup);
        let labels: Vec<&str> = lines.iter().map(|line| line.label.as_str()).collect();
        assert_eq!(labels, ["", "s94", "s98", "s111", "s404", "s52", "s320"]);
        assert!(lines[0].value.contains(" / "));
        assert_eq!(lines[1].value, "50");
        assert_eq!(lines[4].value, "X 1");
    }

    #[test]
    fn shields_read_the_weapon_columns_and_other_armor_has_a_type_row_and_robes_add_the_mp_bonus() {
        let shield = ItemTipData { type2: 1, body_part: 0x100, grade: 1, shield_defense: 20, shield_rate: 9, evasion: -8, weight: 5, ..Default::default() };
        assert_eq!(values(&lines(&shield, 0, &lookup)), [("s95", "20"), ("s317", "9"), ("s97", "-8"), ("s52", "5")]);
        let robe = ItemTipData { type2: 1, body_part: 0x400, armor_type: 3, mp_bonus: 30, physical_defense: 10, weight: 2, ..Default::default() };
        let result = lines(&robe, 0, &lookup);
        assert_eq!(values(&result)[1..], [("s388", "30"), ("s95", "10"), ("s52", "2")]);
        assert_eq!(result[0].label, "");
    }

    #[test]
    fn jewelry_quest_items_and_adena_follow_their_own_rules() {
        let jewel = ItemTipData { type2: 2, body_part: 8, magical_defense: 12, weight: 1, ..Default::default() };
        assert_eq!(values(&lines(&jewel, 0, &lookup))[1..], [("s99", "12"), ("s52", "1")]);
        let quest = ItemTipData { type2: 3, weight: 7, ..Default::default() };
        assert_eq!(lines(&quest, 0, &lookup).len(), 1);
        let adena = ItemTipData { type2: 4, weight: 0, ..Default::default() };
        assert!(lines(&adena, 0, &lookup).is_empty());
    }

    #[test]
    fn etc_items_branch_on_the_family() {
        let plain = ItemTipData { type2: 5, weight: 3, ..Default::default() };
        assert_eq!(values(&lines(&plain, 0, &lookup)), [("s52", "3")]);
        let collar = ItemTipData { type2: 5, etc_family: 7, custom2: 1, weight: 1, ..Default::default() };
        assert_eq!(values(&lines(&collar, 25, &lookup)), [("s969", "s970"), ("s88", "25"), ("s52", "1")]);
        let lotto = ItemTipData { type2: 5, etc_family: 13, custom1: 0x1_0007, custom2: 0b11, weight: 1, ..Default::default() };
        let result = lines(&lotto, 0b101, &lookup);
        assert_eq!(result[0].value, "7");
        assert_eq!(result[1].value, "1,3,17,18");
        let ticket = ItemTipData { type2: 5, etc_family: 14, custom1: 0x23, custom2: 4, weight: 1, ..Default::default() };
        let result = lines(&ticket, 9, &lookup);
        assert_eq!(values(&result)[..3], [("s670", "9"), ("s671", "3,2"), ("s744", "400")]);
    }

    #[test]
    fn only_stackable_etc_style_items_show_a_count() {
        assert!(ItemTipData { type2: 5, etc_stackable: 2, ..Default::default() }.counted());
        assert!(!ItemTipData { type2: 5, etc_stackable: 0, ..Default::default() }.counted());
        assert!(!ItemTipData { type2: 0, etc_stackable: 2, ..Default::default() }.counted());
    }
}
