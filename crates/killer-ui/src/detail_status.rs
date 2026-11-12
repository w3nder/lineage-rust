use crate::color::{Argb, GOLD, TEXT};
use crate::draw::{DrawList, digits_width};
use crate::exp::exp_percent;
use crate::font::BitmapFont;
use crate::inventory::WEIGHT_THRESHOLDS;

pub const PAGE_SIZE: (i32, i32) = (256, 335);
pub const MYINFO_BACK: &str = "L2UI_CH3.PlayerStatusWnd.myinfo_back";
pub const PS_HPBAR: &str = "L2UI_CH3.PlayerStatusWnd.ps_hpbar";
pub const PS_MPBAR: &str = "L2UI_CH3.PlayerStatusWnd.ps_mpbar";
pub const PS_EXPBAR: &str = "L2UI_CH3.PlayerStatusWnd.ps_expbar";
pub const PS_CPBAR: &str = "L2UI_CH3.PlayerStatusWnd.ps_cpbar";
pub const PS_WEIGHT_BARS: [&str; 5] = [
    "L2UI_CH3.PlayerStatusWnd.ps_weightBar1",
    "L2UI_CH3.PlayerStatusWnd.ps_weightBar2",
    "L2UI_CH3.PlayerStatusWnd.ps_weightBar3",
    "L2UI_CH3.PlayerStatusWnd.ps_weightBar4",
    "L2UI_CH3.PlayerStatusWnd.ps_weightBar5",
];
pub const HERO_ICON: &str = "L2UI_CH3.PlayerStatusWnd.myinfo_heroicon";
pub const NOBLE_ICON: &str = "L2UI_CH3.PlayerStatusWnd.myinfo_nobleicon";

pub const LABEL: Argb = Argb(0xFFA3A3A3);
pub const SECTION: Argb = TEXT;
pub const VALUE: Argb = GOLD;
pub const NICK: Argb = Argb(0xFFA2F9EC);
pub const EXP_TEXT: Argb = Argb(0xFFB4B4B4);

pub const BAR_WIDTH: i32 = 85;
pub const BAR_HEIGHT: i32 = 12;
pub const BAR_OVERFLOW_HEIGHT: i32 = 10;
pub const BAR_UW: i32 = 8;
pub const HP_BAR: (i32, i32) = (35, 59);
pub const MP_BAR: (i32, i32) = (35, 73);
pub const EXP_BAR: (i32, i32) = (35, 87);
pub const CP_BAR: (i32, i32) = (156, 59);
pub const WEIGHT_BAR: (i32, i32) = (156, 73);
pub const LEFT_SLASH_X: i32 = 77;
pub const LEFT_CUR_END: i32 = 67;
pub const LEFT_MAX_X: i32 = 83;
pub const RIGHT_SLASH_X: i32 = 198;
pub const RIGHT_CUR_END: i32 = 188;
pub const RIGHT_MAX_X: i32 = 204;
pub const HP_DIGITS_Y: i32 = 61;
pub const MP_DIGITS_Y: i32 = 75;
pub const EXP_TEXT_Y: i32 = 89;
pub const SP_Y: i32 = 88;
pub const NAME_POS: (i32, i32) = (15, 9);
pub const NICK_GAP: i32 = 21;
pub const NAME_MAX_WIDTH: i32 = 220;
pub const NAME_PART_MAX: i32 = 109;
pub const SHORTENED_CHARS: usize = 9;
pub const CLAN_LABEL: (i32, i32) = (15, 25);
pub const CLAN_NAME_GAP: i32 = 20;
pub const CLAN_CREST_SIZE: (i32, i32) = (16, 12);
pub const CLAN_CREST_V: i32 = 4;
pub const CLAN_CREST_NAME_SHIFT: i32 = 20;
pub const CLAN_CREST_GAP: i32 = 40;
pub const LEVEL_POS: (i32, i32) = (35, 46);
pub const LEFT_LABEL_X: i32 = 16;
pub const RIGHT_LABEL_X: i32 = 137;
pub const SECTION_X: i32 = 22;
pub const LEFT_VALUE_RIGHT: i32 = 114;
pub const RIGHT_VALUE_RIGHT: i32 = 235;
pub const STAT_LABEL_X: [i32; 3] = [16, 96, 177];
pub const STAT_CENTER_X: [i32; 3] = [65, 148, 229];
pub const STAT_ROW_Y: [i32; 2] = [236, 252];
pub const HERO_ICON_POS: (i32, i32) = (192, 40);
pub const HERO_ICON_SIZE: (i32, i32) = (13, 15);
pub const CLASS_NAME_SYSSTRING_BASE: u32 = 247;
pub const CLASS_NAME_SYSSTRING_HIGH_BASE: u32 = 1071;
pub const CLASS_NAME_LOW_LIMIT: u32 = 0x58;
pub const CLAN_MEASURE_SYSSTRING: u32 = 873;
pub const CLAN_MEASURE_FALLBACK: u32 = CLAN_SYSSTRING;
pub const KARMA_BIG_FORMAT_LIMIT: i32 = 999_999;

pub const LEFT_ROWS: [(u32, i32); 5] = [(94, 132), (95, 147), (96, 162), (113, 177), (111, 192)];
pub const RIGHT_ROWS: [(u32, i32); 5] = [(98, 132), (99, 148), (97, 163), (432, 178), (112, 193)];
pub const SECTIONS: [(u32, i32); 3] = [(93, 110), (103, 214), (100, 273)];
pub const STAT_LABELS: [[u32; 3]; 2] = [[104, 105, 106], [107, 108, 109]];
pub const SOCIAL_ROWS: [(u32, u32, i32); 2] = [(101, 102, 294), (709, 710, 310)];
pub const LV_SYSSTRING: u32 = 88;
pub const CLAN_SYSSTRING: u32 = 430;
pub const NO_CLAN_SYSSTRING: u32 = 431;

pub fn class_name_sysstring(class_id: u32) -> u32 {
    if class_id < CLASS_NAME_LOW_LIMIT {
        CLASS_NAME_SYSSTRING_BASE + class_id
    } else {
        CLASS_NAME_SYSSTRING_HIGH_BASE + class_id
    }
}

pub fn karma_name_color(karma: i32) -> Argb {
    let k = if karma >= 0x100 {
        0xff
    } else if karma > 0 {
        let step = karma / 16 + 100;
        if step <= karma { karma.min(0xff) } else { step }
    } else {
        0
    } as u32;
    Argb(0xFFFF_0000 | ((0xff - k) << 8) | (0xff - k))
}

pub fn stat_text(base: i32, bonus: i32) -> String {
    if bonus > 0 {
        format!("{base} (+{bonus})")
    } else if bonus < 0 {
        format!("{base}({bonus})")
    } else {
        base.to_string()
    }
}

pub fn karma_text(karma: i32) -> String {
    if karma < KARMA_BIG_FORMAT_LIMIT { karma.to_string() } else { format!("{karma}+") }
}

pub fn percent_text(percent: f32) -> String {
    format!("{percent:.2}%")
}

fn raw_fill(current: i32, max: i32) -> i32 {
    if max <= 0 {
        return 0;
    }
    (i64::from(current) * i64::from(BAR_WIDTH) / i64::from(max)).clamp(0, i64::from(i32::MAX)) as i32
}

pub fn bar_size(current: i32, max: i32) -> Option<(i32, i32)> {
    let width = raw_fill(current, max);
    if width < 1 {
        return None;
    }
    Some(if width > BAR_WIDTH { (BAR_WIDTH, BAR_OVERFLOW_HEIGHT) } else { (width, BAR_HEIGHT) })
}

pub const GM_Y_SHIFT: i32 = 20;
pub const GM_RIGHT_ROWS: [(u32, i32); 5] = [(98, 132), (99, 148), (97, 163), (112, 178), (432, 193)];
pub const GM_CAST_VALUE_Y: i32 = 177;
pub const GM_SPEED_VALUE_Y: i32 = 208;
pub const GM_SPEED_LEGEND_Y: i32 = 220;
pub const GM_SPEED_LEGEND: &str = "Water/Air/Ground";
pub const GM_WEIGHT_THRESHOLDS: [f64; 3] = [50.0, 66.66, 80.0];

pub fn gm_weight_bar(load: i32, max_load: i32) -> Option<(i32, i32, &'static str)> {
    if max_load <= 0 {
        return None;
    }
    let width = raw_fill(load, max_load);
    if width < 1 {
        return None;
    }
    let percent = f64::from(load as f32) / f64::from(max_load as f32) * 100.0;
    match GM_WEIGHT_THRESHOLDS.iter().position(|limit| percent <= *limit) {
        Some(i) => Some((width, BAR_HEIGHT, PS_WEIGHT_BARS[i])),
        None => Some((width, BAR_OVERFLOW_HEIGHT, PS_WEIGHT_BARS[3])),
    }
}

pub fn weight_bar(load: i32, max_load: i32) -> Option<(i32, i32, &'static str)> {
    if max_load <= 0 {
        return None;
    }
    let width = raw_fill(load, max_load).min(BAR_WIDTH);
    if width < 1 {
        return None;
    }
    let exact = f64::from(load) / f64::from(max_load) * 100.0;
    let stored = exact as f32;
    let index = if exact <= 50.0 {
        0
    } else {
        1 + WEIGHT_THRESHOLDS[1..].iter().position(|limit| stored <= *limit).unwrap_or(WEIGHT_THRESHOLDS.len() - 1)
    };
    match index {
        0..=3 => Some((width, BAR_HEIGHT, PS_WEIGHT_BARS[index])),
        _ => Some((width, BAR_OVERFLOW_HEIGHT, PS_WEIGHT_BARS[4])),
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DetailStatusView {
    pub name: String,
    pub nick: String,
    pub karma: i32,
    pub clan_name: Option<String>,
    pub clan_crest_id: i32,
    pub clan_crest: Option<String>,
    pub level: i32,
    pub class_id: u32,
    pub hp: (i32, i32),
    pub mp: (i32, i32),
    pub cp: (i32, i32),
    pub exp: i64,
    pub sp: i32,
    pub load: i32,
    pub max_load: i32,
    pub p_atk: i32,
    pub p_def: i32,
    pub accuracy: i32,
    pub critical: i32,
    pub atk_spd: i32,
    pub m_atk: i32,
    pub m_def: i32,
    pub evasion: i32,
    pub speed: i32,
    pub cast_spd: i32,
    pub stats: [i32; 6],
    pub stat_bonus: [i32; 6],
    pub pvp_kills: i32,
    pub pk_kills: i32,
    pub recom_have: i32,
    pub recom_left: i32,
    pub hero: bool,
    pub noble: bool,
}

pub struct Sysstrings<'a>(pub &'a dyn Fn(u32) -> Option<String>);

impl Sysstrings<'_> {
    fn text(&self, id: u32) -> String {
        (self.0)(id).unwrap_or_else(|| format!("sysstring {id}"))
    }
}

impl DetailStatusView {
    pub fn exp_percent(&self) -> f32 {
        exp_percent(self.level, self.exp)
    }

    pub fn weight_percent(&self) -> f32 {
        if self.max_load > 0 { self.load as f32 * 100.0 / self.max_load as f32 } else { 0.0 }
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, strings: &Sysstrings<'_>) {
        self.paint_variant(list, x, y, font, strings, None);
    }

    pub fn paint_gm(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, strings: &Sysstrings<'_>, speeds: &str) {
        self.paint_variant(list, x, y + GM_Y_SHIFT, font, strings, Some(speeds));
    }

    fn paint_variant(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, strings: &Sysstrings<'_>, gm: Option<&str>) {
        let (w, h) = PAGE_SIZE;
        list.draw_texture(x, y, w, h, 0, 0, w, h, MYINFO_BACK, 1.0);
        list.draw_text(x + CLAN_LABEL.0, y + CLAN_LABEL.1, LABEL, &strings.text(CLAN_SYSSTRING), font);
        list.draw_text(x + 15, y + 46, LABEL, &strings.text(LV_SYSSTRING), font);
        for (id, row_y) in SECTIONS {
            list.draw_text(x + SECTION_X, y + row_y, SECTION, &strings.text(id), font);
        }
        for (id, row_y) in LEFT_ROWS {
            list.draw_text(x + LEFT_LABEL_X, y + row_y, LABEL, &strings.text(id), font);
        }
        let right_rows = if gm.is_some() { GM_RIGHT_ROWS } else { RIGHT_ROWS };
        for (id, row_y) in right_rows {
            list.draw_text(x + RIGHT_LABEL_X, y + row_y, LABEL, &strings.text(id), font);
        }
        for (row, ids) in STAT_LABELS.iter().enumerate() {
            for (col, id) in ids.iter().enumerate() {
                list.draw_text(x + STAT_LABEL_X[col], y + STAT_ROW_Y[row], LABEL, &strings.text(*id), font);
            }
        }
        for (left, right, row_y) in SOCIAL_ROWS {
            list.draw_text(x + LEFT_LABEL_X, y + row_y, LABEL, &strings.text(left), font);
            list.draw_text(x + RIGHT_LABEL_X, y + row_y, LABEL, &strings.text(right), font);
        }
        self.paint_bars(list, x, y, gm.is_some());
        self.paint_digits(list, x, y, font);
        self.paint_name(list, x, y, font);
        self.paint_clan(list, x, y, font, strings);
        let level = format!("{} {}", self.level, strings.text(class_name_sysstring(self.class_id)));
        list.draw_text(x + LEVEL_POS.0, y + LEVEL_POS.1, VALUE, &level, font);
        let left = [self.p_atk, self.p_def, self.accuracy, self.critical, self.atk_spd];
        for ((_, row_y), value) in LEFT_ROWS.iter().zip(left) {
            right_aligned(list, x + LEFT_VALUE_RIGHT, y + row_y, &value.to_string(), font);
        }
        if let Some(speeds) = gm {
            let right = [self.m_atk, self.m_def, self.evasion, self.cast_spd];
            for ((_, row_y), value) in right_rows.iter().zip(right) {
                let row_y = if *row_y == GM_RIGHT_ROWS[3].1 { GM_CAST_VALUE_Y } else { *row_y };
                right_aligned(list, x + RIGHT_VALUE_RIGHT, y + row_y, &value.to_string(), font);
            }
            right_aligned(list, x + RIGHT_VALUE_RIGHT, y + GM_SPEED_VALUE_Y, speeds, font);
            let width = font.measure(GM_SPEED_LEGEND) as i32;
            list.draw_text(x + RIGHT_VALUE_RIGHT - width, y + GM_SPEED_LEGEND_Y, LABEL, GM_SPEED_LEGEND, font);
        } else {
            let right = [self.m_atk, self.m_def, self.evasion, self.speed, self.cast_spd];
            for ((_, row_y), value) in right_rows.iter().zip(right) {
                right_aligned(list, x + RIGHT_VALUE_RIGHT, y + row_y, &value.to_string(), font);
            }
        }
        for (i, (base, bonus)) in self.stats.iter().zip(self.stat_bonus).enumerate() {
            let text = if gm.is_some() && i == 0 && bonus > 0 { format!("{base}(+{bonus})") } else { stat_text(*base, bonus) };
            let width = font.measure(&text) as i32;
            list.draw_text(x + STAT_CENTER_X[i % 3] - width / 2, y + STAT_ROW_Y[i / 3], VALUE, &text, font);
        }
        right_aligned(list, x + LEFT_VALUE_RIGHT, y + SOCIAL_ROWS[0].2, &karma_text(self.karma), font);
        right_aligned(
            list,
            x + RIGHT_VALUE_RIGHT,
            y + SOCIAL_ROWS[0].2,
            &format!("{} / {}", self.pvp_kills, self.pk_kills),
            font,
        );
        right_aligned(list, x + LEFT_VALUE_RIGHT, y + SOCIAL_ROWS[1].2, &self.recom_have.to_string(), font);
        right_aligned(list, x + RIGHT_VALUE_RIGHT, y + SOCIAL_ROWS[1].2, &self.recom_left.to_string(), font);
        let icon = if gm.is_some() {
            None
        } else if self.hero {
            Some(HERO_ICON)
        } else if self.noble {
            Some(NOBLE_ICON)
        } else {
            None
        };
        if let Some(icon) = icon {
            let (iw, ih) = HERO_ICON_SIZE;
            list.draw_texture(x + HERO_ICON_POS.0, y + HERO_ICON_POS.1, iw, ih, 0, 0, iw, ih, icon, 1.0);
        }
    }

    fn paint_bars(&self, list: &mut DrawList, x: i32, y: i32, gm: bool) {
        for ((bx, by), value, texture) in
            [(HP_BAR, self.hp, PS_HPBAR), (MP_BAR, self.mp, PS_MPBAR), (CP_BAR, self.cp, PS_CPBAR)]
        {
            if let Some((bw, bh)) = bar_size(value.0, value.1) {
                list.draw_texture(x + bx, y + by, bw, bh, 0, 0, BAR_UW, BAR_HEIGHT, texture, 1.0);
            }
        }
        let exp = (self.exp_percent() * BAR_WIDTH as f32 / 100.0) as i32;
        if exp >= 1 {
            list.draw_texture(
                x + EXP_BAR.0,
                y + EXP_BAR.1,
                exp.min(BAR_WIDTH),
                BAR_HEIGHT,
                0,
                0,
                BAR_UW,
                BAR_HEIGHT,
                PS_EXPBAR,
                1.0,
            );
        }
        let weight = if gm { gm_weight_bar(self.load, self.max_load) } else { weight_bar(self.load, self.max_load) };
        if let Some((bw, bh, texture)) = weight {
            list.draw_texture(x + WEIGHT_BAR.0, y + WEIGHT_BAR.1, bw, bh, 0, 0, bw, BAR_HEIGHT, texture, 1.0);
        }
    }

    fn paint_digits(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        let pair = |list: &mut DrawList, value: (i32, i32), slash_x: i32, cur_end: i32, max_x: i32, row_y: i32| {
            list.draw_digits(x + slash_x - digits_width("/") / 2, y + row_y, "/", TEXT);
            let cur = value.0.to_string();
            list.draw_digits(x + cur_end - digits_width(&cur), y + row_y, &cur, TEXT);
            list.draw_digits(x + max_x, y + row_y, &value.1.to_string(), TEXT);
        };
        pair(list, self.hp, LEFT_SLASH_X, LEFT_CUR_END, LEFT_MAX_X, HP_DIGITS_Y);
        pair(list, self.mp, LEFT_SLASH_X, LEFT_CUR_END, LEFT_MAX_X, MP_DIGITS_Y);
        pair(list, self.cp, RIGHT_SLASH_X, RIGHT_CUR_END, RIGHT_MAX_X, HP_DIGITS_Y);
        let exp = percent_text(self.exp_percent());
        list.draw_digits(x + LEFT_SLASH_X - digits_width(&exp) / 2, y + EXP_TEXT_Y, &exp, EXP_TEXT);
        let weight = percent_text(self.weight_percent());
        list.draw_digits(x + RIGHT_SLASH_X - digits_width(&weight) / 2, y + MP_DIGITS_Y, &weight, TEXT);
        let sp = self.sp.to_string();
        list.draw_text(x + RIGHT_SLASH_X - font.measure(&sp) as i32 / 2, y + SP_Y, VALUE, &sp, font);
    }

    fn paint_name(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        let color = karma_name_color(self.karma);
        if self.nick.is_empty() {
            list.draw_text(x + NAME_POS.0, y + NAME_POS.1, color, &self.name, font);
            return;
        }
        let mut nick = self.nick.clone();
        let mut name = self.name.clone();
        if font.measure(&nick) as i32 + font.measure(&name) as i32 > NAME_MAX_WIDTH {
            if font.measure(&nick) as i32 > NAME_PART_MAX {
                nick = shorten(&nick);
            }
            if font.measure(&name) as i32 > NAME_PART_MAX {
                name = shorten(&name);
            }
        }
        list.draw_text(x + NAME_POS.0, y + NAME_POS.1, NICK, &nick, font);
        list.draw_text(x + font.measure(&nick) as i32 + NICK_GAP, y + NAME_POS.1, color, &name, font);
    }

    fn paint_clan(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, strings: &Sysstrings<'_>) {
        let measured = (strings.0)(CLAN_MEASURE_SYSSTRING).unwrap_or_else(|| strings.text(CLAN_MEASURE_FALLBACK));
        let base = font.measure(&measured) as i32 + CLAN_NAME_GAP;
        match &self.clan_name {
            None => {
                list.draw_text(x + base, y + CLAN_LABEL.1, TEXT, &strings.text(NO_CLAN_SYSSTRING), font);
            }
            Some(name) => {
                if let Some(crest) = &self.clan_crest {
                    list.draw_texture(x + base, y + CLAN_LABEL.1, CLAN_CREST_SIZE.0, CLAN_CREST_SIZE.1, 0, CLAN_CREST_V, CLAN_CREST_SIZE.0, CLAN_CREST_SIZE.1, crest, 1.0);
                }
                if !name.is_empty() {
                    let shift = if self.clan_crest_id > 0 { CLAN_CREST_NAME_SHIFT } else { 0 };
                    list.draw_text(x + base + shift, y + CLAN_LABEL.1, VALUE, name, font);
                }
            }
        }
    }
}

fn right_aligned(list: &mut DrawList, right: i32, y: i32, text: &str, font: &BitmapFont) {
    list.draw_text(right - font.measure(text) as i32, y, VALUE, text, font);
}

fn shorten(text: &str) -> String {
    let mut out: String = text.chars().take(SHORTENED_CHARS).collect();
    out.push_str("..");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        let bytes = gly(&[(0x20, &glyphs)], (1024, 128));
        BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&bytes).unwrap())
    }

    fn view() -> DetailStatusView {
        DetailStatusView {
            name: "Killer".into(),
            level: 52,
            class_id: 2,
            hp: (450, 900),
            mp: (200, 300),
            cp: (100, 100),
            exp: 52262490,
            sp: 12345,
            load: 300,
            max_load: 1000,
            p_atk: 123,
            p_def: 456,
            accuracy: 78,
            critical: 90,
            atk_spd: 300,
            m_atk: 54,
            m_def: 321,
            evasion: 77,
            speed: 120,
            cast_spd: 333,
            stats: [40, 30, 43, 21, 11, 25],
            stat_bonus: [0, 2, -1, 0, 0, 0],
            pvp_kills: 3,
            pk_kills: 1,
            recom_have: 7,
            recom_left: 5,
            ..Default::default()
        }
    }

    #[test]
    fn class_names_karma_color_and_stat_formats_follow_on_paint() {
        assert_eq!(class_name_sysstring(0), 247);
        assert_eq!(class_name_sysstring(87), 334);
        assert_eq!(class_name_sysstring(88), 1159);
        assert_eq!(karma_name_color(0), Argb(0xFFFFFFFF));
        assert_eq!(karma_name_color(16), Argb(0xFFFF9A9A));
        assert_eq!(karma_name_color(200), Argb(0xFFFF3737));
        assert_eq!(karma_name_color(255), Argb(0xFFFF0000));
        assert_eq!(karma_name_color(5000), Argb(0xFFFF0000));
        assert_eq!(stat_text(40, 0), "40");
        assert_eq!(stat_text(40, 3), "40 (+3)");
        assert_eq!(stat_text(40, -2), "40(-2)");
        assert_eq!(karma_text(10), "10");
        assert_eq!(karma_text(999_999), "999999+");
        assert_eq!(percent_text(16.342), "16.34%");
        assert_eq!(bar_size(450, 900), Some((42, 12)));
        assert_eq!(bar_size(0, 900), None);
        assert_eq!(bar_size(1000, 900), Some((85, 10)));
        assert_eq!(weight_bar(300, 1000), Some((25, 12, PS_WEIGHT_BARS[0])));
        assert_eq!(weight_bar(1100, 1000), Some((85, 10, PS_WEIGHT_BARS[4])));
        assert_eq!(weight_bar(0, 1000), None);
    }

    #[test]
    fn nick_clan_and_hero_icon_take_their_places() {
        let lookup = |id: u32| Some(format!("s{id}"));
        let strings = Sysstrings(&lookup);
        let mut view = view();
        view.nick = "Lord".into();
        view.hero = true;
        view.clan_name = Some("Killers".into());
        view.karma = 300;
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font(), &strings);
        let texts: Vec<_> = list.quads.iter().filter(|q| q.texture == "L2Font-e.SmallFont-e").collect();
        let nick: Vec<_> = texts.iter().filter(|q| q.y == 9.0 && q.color == NICK.rgba()).collect();
        assert_eq!(nick[0].x, 15.0);
        let name: Vec<_> = texts.iter().filter(|q| q.y == 9.0 && q.color == Argb(0xFFFF0000).rgba()).collect();
        assert_eq!(name[0].x, 24.0 + 21.0);
        let clan: Vec<_> = texts.iter().filter(|q| q.y == 25.0 && q.color == VALUE.rgba()).collect();
        assert_eq!(clan[0].x, 24.0 + 20.0);
        let short = |id: u32| (id != CLAN_MEASURE_SYSSTRING).then(|| format!("s{id}"));
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font(), &Sysstrings(&short));
        let clan_x = list.quads.iter().filter(|q| q.texture == "L2Font-e.SmallFont-e" && q.y == 25.0 && q.color == VALUE.rgba()).map(|q| q.x).fold(f32::MAX, f32::min);
        assert_eq!(clan_x, 24.0 + 20.0);
        let hero: Vec<_> = list.quads.iter().filter(|q| q.texture == HERO_ICON).collect();
        assert_eq!((hero[0].x, hero[0].y, hero[0].w, hero[0].h), (192.0, 40.0, 13.0, 15.0));
        let mut noble = view.clone();
        noble.hero = false;
        noble.noble = true;
        noble.nick = "VeryLongNickNameHere".into();
        noble.name = "VeryLongCharacterName".into();
        let mut list = DrawList::default();
        noble.paint(&mut list, 0, 0, &font(), &strings);
        assert!(list.quads.iter().any(|q| q.texture == NOBLE_ICON));
        let nick_end =
            list.quads.iter().filter(|q| q.y == 9.0 && q.color == NICK.rgba()).map(|q| q.x + q.w).fold(0.0, f32::max);
        assert_eq!(nick_end, 15.0 + 11.0 * 6.0);
    }

    #[test]
    fn the_clan_block_follows_the_reference_crest_and_name_rules() {
        let lookup = |id: u32| Some(format!("s{id}"));
        let strings = Sysstrings(&lookup);
        let value_x = |list: &DrawList| list.quads.iter().filter(|q| q.texture == "L2Font-e.SmallFont-e" && q.y == 25.0 && q.color == VALUE.rgba()).map(|q| q.x).fold(f32::MAX, f32::min);
        let mut view = view();
        view.clan_name = Some("Killers".into());
        view.clan_crest_id = 7;
        view.clan_crest = Some("crest.crest_7".into());
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font(), &strings);
        let crest: Vec<_> = list.quads.iter().filter(|q| q.texture == "crest.crest_7").collect();
        assert_eq!((crest[0].x, crest[0].y, crest[0].w, crest[0].h, crest[0].v, crest[0].vh), (44.0, 25.0, 16.0, 12.0, 4.0, 12.0));
        assert_eq!(value_x(&list), 64.0);
        view.clan_crest_id = 0;
        view.clan_crest = None;
        view.clan_name = Some(String::new());
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font(), &strings);
        assert_eq!(value_x(&list), f32::MAX);
        assert!(list.quads.iter().all(|q| q.texture != "crest.crest_7"));
    }
}
