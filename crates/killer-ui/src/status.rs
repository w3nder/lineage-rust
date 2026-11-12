#[path = "status_frame_generated.rs"]
mod frame_controls;

use crate::color::{GOLD, TEXT};
use crate::draw::DrawList;
use crate::exp::{exp_fill, exp_percent};
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;

pub const STATUS_SIZE: (i32, i32) = (172, 84);
pub const STATUS_MIN_WIDTH: i32 = 172;

pub const SMALLWINDOW2_BACK1: &str = "L2UI_CH3.SmallWnd.Smallwindow2_back1";
pub const SMALLWINDOW2_BACK2: &str = "L2UI_CH3.SmallWnd.Smallwindow2_back2";
pub const PS_LEVELBACK: &str = "L2UI_CH3.PlayerStatusWnd.ps_levelback";
pub const PS_SIZECONTROL1: &str = "L2UI_CH3.PlayerStatusWnd.ps_sizecontrol1";
pub const PS_SIZECONTROL2: &str = "L2UI_CH3.PlayerStatusWnd.ps_sizecontrol2";
pub const PS_CPBAR: &str = "L2UI_CH3.PlayerStatusWnd.ps_cpbar";
pub const PS_CPBAR_BACK: &str = "L2UI_CH3.PlayerStatusWnd.ps_cpbar_back";
pub const PS_HPBAR: &str = "L2UI_CH3.PlayerStatusWnd.ps_hpbar";
pub const PS_HPBAR_BACK: &str = "L2UI_CH3.PlayerStatusWnd.ps_hpbar_back";
pub const PS_HPBARWARN1: &str = "L2UI_CH3.PlayerStatusWnd.ps_hpbarwarn1";
pub const PS_HPBARFILL: &str = "L2UI_CH3.PlayerStatusWnd.ps_hpbarfill";
pub const REGEN_ALPHA: f32 = 128.0 / 255.0;
pub const PS_MPBAR: &str = "L2UI_CH3.PlayerStatusWnd.ps_mpbar";
pub const PS_MPBAR_BACK: &str = "L2UI_CH3.PlayerStatusWnd.ps_mpbar_back";
pub const PS_EXPBAR: &str = "L2UI_CH3.PlayerStatusWnd.ps_expbar";
pub const PS_EXPBAR_BACK: &str = "L2UI_CH3.PlayerStatusWnd.ps_expbar_back";

pub const BODY_X: i32 = 12;
pub const BODY_END_WIDTH: i32 = 16;
pub const BODY_UW: i32 = 16;
pub const BODY_VH: i32 = 76;
pub const SIZE_CONTROL_WIDTH: i32 = 4;
pub const SIZE_CONTROL2_POS: (i32, i32) = (167, 4);
pub const SIZE_CONTROL2_SIZE: (i32, i32) = (9, 13);
pub const LEVEL_BACK_POS: (i32, i32) = (16, 6);
pub const LEVEL_BACK_SIZE: (i32, i32) = (22, 20);
pub const LEVEL_CENTER_X: i32 = 27;
pub const LEVEL_TENS_X: i32 = 20;
pub const LEVEL_Y: i32 = 12;
pub const BAR_X: i32 = 16;
pub const BAR_WIDTH: i32 = 150;
pub const BAR_HEIGHT: i32 = 12;
pub const BAR_UW: i32 = 8;
pub const CP_BAR_Y: i32 = 27;
pub const HP_BAR_Y: i32 = 41;
pub const MP_BAR_Y: i32 = 55;
pub const EXP_BAR_Y: i32 = 69;
pub const CP_VALUE_Y: i32 = 29;
pub const HP_VALUE_Y: i32 = 43;
pub const MP_VALUE_Y: i32 = 57;
pub const LABEL_X: i32 = 25;
pub const CURRENT_INSET: i32 = 8;
pub const MAX_INSET: i32 = 4;
pub const EXP_TEXT_INSET: i32 = 12;
pub const EXP_TEXT_Y: i32 = 71;
pub const NAME_POS: (i32, i32) = (40, 9);
pub const NAME_INSET: i32 = 50;
pub const HP_WARN_TENTHS: i32 = 3;
pub const ELLIPSIS: &str = "..";

#[derive(Debug, Clone, PartialEq)]
pub struct StatusView {
    pub name: String,
    pub level: i32,
    pub hp: (i32, i32),
    pub mp: (i32, i32),
    pub cp: (i32, i32),
    pub exp: i64,
    pub regen: f64,
}

impl StatusView {
    pub fn exp_percent(&self) -> f32 {
        exp_percent(self.level, self.exp)
    }
}

pub fn fill_width(value: (i32, i32), width: i32) -> i32 {
    let (current, max) = (i64::from(value.0), i64::from(value.1));
    if max == 0 {
        return 0;
    }
    (current * i64::from(width) / max).clamp(0, i64::from(width)) as i32
}

pub fn hp_is_low(hp: (i32, i32)) -> bool {
    hp.0 < (hp.1 / 10) * HP_WARN_TENTHS
}

pub fn ellipsize(text: &str, max_width: i32, font: &BitmapFont) -> String {
    if font.measure(text) as i32 <= max_width {
        return text.to_string();
    }
    let limit = max_width - font.measure(ELLIPSIS) as i32;
    let mut kept: Vec<char> = text.chars().collect();
    loop {
        kept.pop();
        let prefix: String = kept.iter().collect();
        if kept.is_empty() || (font.measure(&prefix) as i32) < limit {
            return prefix + ELLIPSIS;
        }
    }
}

pub fn paint_status(list: &mut DrawList, x: i32, y: i32, view: &StatusView, font: &BitmapFont, alpha: f32) {
    paint_status_sized(list, x, y, STATUS_SIZE.0, view, font, alpha);
}

#[allow(clippy::too_many_arguments)]
pub fn paint_status_sized(
    list: &mut DrawList,
    x: i32,
    y: i32,
    w: i32,
    view: &StatusView,
    font: &BitmapFont,
    alpha: f32,
) {
    let h = STATUS_SIZE.1;
    let gap = (w - STATUS_MIN_WIDTH).max(0);
    let bar_w = BAR_WIDTH + gap;
    let contract = frame_controls::FRAME_7C;
    contract.paint(&FrameCtrl::new(contract.mode), list, (x, y), font, alpha);
    list.draw_texture(x + BODY_X, y, BODY_END_WIDTH, h, 0, 0, BODY_UW, BODY_VH, SMALLWINDOW2_BACK1, alpha);
    let mid_x = BODY_X + BODY_END_WIDTH;
    let mid_w = w - mid_x - SIZE_CONTROL_WIDTH;
    list.draw_texture(x + mid_x, y, mid_w, h, 0, 0, BODY_UW, BODY_VH, SMALLWINDOW2_BACK2, alpha);
    let end_x = w - SIZE_CONTROL_WIDTH;
    list.draw_texture(x + end_x, y, SIZE_CONTROL_WIDTH, h, 0, 0, SIZE_CONTROL_WIDTH, BODY_VH, PS_SIZECONTROL1, alpha);

    let (lw, lh) = LEVEL_BACK_SIZE;
    list.draw_texture(x + LEVEL_BACK_POS.0, y + LEVEL_BACK_POS.1, lw, lh, 0, 0, lw, lh, PS_LEVELBACK, 1.0);
    let (sw, sh) = SIZE_CONTROL2_SIZE;
    let (sx, sy) = (x + SIZE_CONTROL2_POS.0 + gap, y + SIZE_CONTROL2_POS.1);
    list.draw_texture(sx, sy, sw, sh, 0, 0, sw, sh, PS_SIZECONTROL2, 1.0);

    let hp_fill = fill_width(view.hp, bar_w);
    let mp_fill = fill_width(view.mp, bar_w);
    let cp_fill = fill_width(view.cp, bar_w);
    let exp_fill = exp_fill(view.level, view.exp, bar_w).clamp(0, bar_w);
    paint_bar(list, x + BAR_X, y + CP_BAR_Y, bar_w, PS_CPBAR_BACK, PS_CPBAR, cp_fill);
    paint_bar(list, x + BAR_X, y + MP_BAR_Y, bar_w, PS_MPBAR_BACK, PS_MPBAR, mp_fill);
    paint_bar(list, x + BAR_X, y + EXP_BAR_Y, bar_w, PS_EXPBAR_BACK, PS_EXPBAR, exp_fill);
    paint_bar(list, x + BAR_X, y + HP_BAR_Y, bar_w, PS_HPBAR_BACK, PS_HPBAR, hp_fill);
    let regen_w = if view.regen == 0.0 || view.hp.1 == 0 { 0 } else { (f64::from(bar_w) * view.regen / f64::from(view.hp.1)) as i32 };
    if regen_w > 0 {
        list.draw_texture(x + BAR_X + hp_fill, y + HP_BAR_Y, regen_w, BAR_HEIGHT, 0, 0, BAR_UW, BAR_HEIGHT, PS_HPBARFILL, REGEN_ALPHA);
    }
    if hp_is_low(view.hp) && hp_fill > 0 {
        list.draw_texture(x + BAR_X, y + HP_BAR_Y, hp_fill, BAR_HEIGHT, 0, 0, BAR_UW, BAR_HEIGHT, PS_HPBARWARN1, 1.0);
    }

    let half = bar_w / 2;
    paint_row(list, x, y + CP_VALUE_Y, half, "CP", view.cp, font);
    paint_row(list, x, y + HP_VALUE_Y, half, "HP", view.hp, font);
    paint_row(list, x, y + MP_VALUE_Y, half, "MP", view.mp, font);

    let exp = format!("{:.2}%", view.exp_percent());
    let exp_w = font.measure(&exp) as i32;
    list.draw_digits(x + half - exp_w / 2 + EXP_TEXT_INSET, y + EXP_TEXT_Y, &exp, TEXT);

    let name = ellipsize(&view.name, w - NAME_INSET, font);
    list.draw_text(x + NAME_POS.0, y + NAME_POS.1, TEXT, &name, font);

    let level = view.level.to_string();
    if view.level < 10 {
        let lw = font.measure(&level) as i32;
        list.draw_digits(x + LEVEL_CENTER_X - lw / 2, y + LEVEL_Y, &level, GOLD);
    } else {
        let mut chars = level.chars();
        let tens = chars.next().map(String::from).unwrap_or_default();
        let ones = chars.next().map(String::from).unwrap_or_default();
        list.draw_digits(x + LEVEL_TENS_X, y + LEVEL_Y, &tens, GOLD);
        list.draw_digits(x + LEVEL_CENTER_X, y + LEVEL_Y, &ones, GOLD);
    }
}

fn paint_bar(list: &mut DrawList, x: i32, y: i32, w: i32, back: &str, bar: &str, fill: i32) {
    list.draw_texture(x, y, w, BAR_HEIGHT, 0, 0, BAR_UW, BAR_HEIGHT, back, 1.0);
    if fill > 0 {
        list.draw_texture(x, y, fill, BAR_HEIGHT, 0, 0, BAR_UW, BAR_HEIGHT, bar, 1.0);
    }
}

fn paint_row(list: &mut DrawList, x: i32, y: i32, half: i32, label: &str, value: (i32, i32), font: &BitmapFont) {
    let measure = |text: &str| font.measure(text) as i32;
    let center = x + BAR_X + half;
    list.draw_digits(x + LABEL_X, y, label, TEXT);
    list.draw_digits(center - measure("/") / 2, y, "/", TEXT);
    let current = value.0.to_string();
    list.draw_digits(center - measure(&current) - CURRENT_INSET, y, &current, TEXT);
    list.draw_digits(center + MAX_INSET, y, &value.1.to_string(), TEXT);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::{DIGIT_CELLS, DIGIT_TEXTURE, Quad};
    use crate::font::{Glyphs, gly};

    const TEXTURES: &str = include_str!("../tests/fixtures/nwindow-texturas-status.txt");

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95)
            .map(|i| {
                let width = match (0x20 + i) as u8 as char {
                    ' ' => 3,
                    '/' | '.' => 4,
                    '%' => 11,
                    _ => 6,
                };
                [i * 12, width, 0, 13]
            })
            .collect();
        let bytes = gly(&[(0x20, &glyphs)], (1024, 128));
        BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&bytes).unwrap())
    }

    fn view() -> StatusView {
        StatusView { name: "Killer".into(), level: 52, hp: (450, 900), mp: (200, 300), cp: (100, 100), exp: 52262490, regen: 0.0 }
    }

    fn painted(view: &StatusView) -> Vec<Quad> {
        let mut list = DrawList::default();
        paint_status(&mut list, 0, 0, view, &font(), 1.0);
        list.quads
    }

    fn with_texture<'a>(quads: &'a [Quad], texture: &str) -> Vec<&'a Quad> {
        quads.iter().filter(|q| q.texture == texture).collect()
    }

    fn digits(quads: &[Quad], y: f32) -> Vec<&Quad> {
        quads.iter().filter(|q| q.texture == DIGIT_TEXTURE && q.y == y).collect()
    }

    fn cell(c: char) -> f32 {
        DIGIT_CELLS.find(c).unwrap() as f32 * 8.0
    }

    fn strip_text(quads: &[&Quad]) -> String {
        quads.iter().map(|q| DIGIT_CELLS.chars().nth((q.u / 8.0) as usize).unwrap()).collect()
    }

    #[test]
    fn regen_fill_starts_at_the_end_of_the_hp_bar_with_half_alpha() {
        let mut list = DrawList::default();
        paint_status(&mut list, 0, 0, &StatusView { regen: 90.0, ..view() }, &font(), 1.0);
        let fill = list.quads.iter().find(|quad| quad.texture.as_str() == PS_HPBARFILL).expect("preenchimento");
        let bar = list.quads.iter().find(|quad| quad.texture.as_str() == PS_HPBAR).unwrap();
        assert_eq!(fill.x, bar.x + bar.w);
        assert_eq!(fill.color[3], REGEN_ALPHA);
    }

    #[test]
    fn body_is_the_grip_two_slices_and_the_size_control_over_172_pixels() {
        let quads = painted(&view());
        let back1 = with_texture(&quads, SMALLWINDOW2_BACK1);
        assert_eq!((back1[0].x, back1[0].y, back1[0].w, back1[0].h), (12.0, 0.0, 16.0, 84.0));
        assert_eq!((back1[0].uw, back1[0].vh), (16.0, 76.0));
        let back2 = with_texture(&quads, SMALLWINDOW2_BACK2);
        assert_eq!((back2[0].x, back2[0].w, back2[0].uw, back2[0].vh), (28.0, 140.0, 16.0, 76.0));
        let end = with_texture(&quads, PS_SIZECONTROL1);
        assert_eq!((end[0].x, end[0].w, end[0].uw, end[0].vh), (168.0, 4.0, 4.0, 76.0));
        let grip = with_texture(&quads, crate::frame::SMALLBAR_2);
        assert_eq!((grip[0].x, grip[0].y, grip[0].h), (0.0, 8.0, 68.0));
        let handle = with_texture(&quads, PS_SIZECONTROL2);
        assert_eq!((handle[0].x, handle[0].y, handle[0].w, handle[0].h), (167.0, 4.0, 9.0, 13.0));
    }

    #[test]
    fn a_wider_window_stretches_the_body_and_the_bars() {
        let mut list = DrawList::default();
        paint_status_sized(&mut list, 0, 0, 202, &view(), &font(), 1.0);
        let back2 = with_texture(&list.quads, SMALLWINDOW2_BACK2);
        assert_eq!((back2[0].x, back2[0].w), (28.0, 170.0));
        assert_eq!(with_texture(&list.quads, PS_SIZECONTROL1)[0].x, 198.0);
        assert_eq!(with_texture(&list.quads, PS_SIZECONTROL2)[0].x, 197.0);
        assert_eq!(with_texture(&list.quads, PS_HPBAR_BACK)[0].w, 180.0);
        assert_eq!(with_texture(&list.quads, PS_HPBAR)[0].w, 90.0);
        let slash: Vec<&Quad> = digits(&list.quads, 29.0).into_iter().filter(|q| q.u == cell('/')).collect();
        assert_eq!(slash[0].x, 16.0 + 90.0 - 2.0);
    }

    #[test]
    fn bars_fill_by_current_over_max_with_the_hp_bar_drawn_last() {
        let quads = painted(&view());
        let back = with_texture(&quads, PS_HPBAR_BACK);
        assert_eq!((back[0].x, back[0].y, back[0].w, back[0].h), (16.0, 41.0, 150.0, 12.0));
        assert_eq!((back[0].u, back[0].v, back[0].uw, back[0].vh), (0.0, 0.0, 8.0, 12.0));
        let fill = with_texture(&quads, PS_HPBAR);
        assert_eq!((fill[0].x, fill[0].y, fill[0].w, fill[0].h), (16.0, 41.0, 75.0, 12.0));
        assert!(with_texture(&quads, PS_HPBARWARN1).is_empty());
        assert_eq!(with_texture(&quads, PS_CPBAR)[0].w, 150.0);
        assert_eq!(with_texture(&quads, PS_CPBAR_BACK)[0].y, 27.0);
        assert_eq!(with_texture(&quads, PS_MPBAR_BACK)[0].y, 55.0);
        assert_eq!(with_texture(&quads, PS_MPBAR)[0].w, 100.0);
        assert_eq!(with_texture(&quads, PS_EXPBAR_BACK)[0].y, 69.0);
        let exp = with_texture(&quads, PS_EXPBAR);
        assert_eq!((exp[0].w, exp[0].y), (24.0, 69.0));
        let order: Vec<&str> = quads
            .iter()
            .map(|q| q.texture.as_str())
            .filter(|t| [PS_CPBAR_BACK, PS_MPBAR_BACK, PS_EXPBAR_BACK, PS_HPBAR_BACK].contains(t))
            .collect();
        assert_eq!(order, [PS_CPBAR_BACK, PS_MPBAR_BACK, PS_EXPBAR_BACK, PS_HPBAR_BACK]);
    }

    #[test]
    fn hp_below_three_tenths_gets_the_warning_bar_on_top() {
        let mut low = view();
        low.hp = (269, 900);
        let quads = painted(&low);
        let bar = with_texture(&quads, PS_HPBAR);
        let warn = with_texture(&quads, PS_HPBARWARN1);
        assert_eq!((bar[0].w, warn[0].x, warn[0].w, warn[0].h), (44.0, 16.0, 44.0, 12.0));
        let mut at_limit = view();
        at_limit.hp = (270, 900);
        assert!(with_texture(&painted(&at_limit), PS_HPBARWARN1).is_empty());
        assert!(hp_is_low((26, 99)));
        assert!(!hp_is_low((27, 90)));
        assert!(!hp_is_low((0, 0)));
    }

    #[test]
    fn level_digits_split_tens_and_ones_at_twenty_and_twenty_seven() {
        let quads = painted(&view());
        let back = with_texture(&quads, PS_LEVELBACK);
        assert_eq!((back[0].x, back[0].y, back[0].w, back[0].h), (16.0, 6.0, 22.0, 20.0));
        let gold: Vec<&Quad> =
            with_texture(&quads, DIGIT_TEXTURE).into_iter().filter(|q| q.color == GOLD.rgba()).collect();
        assert_eq!(gold.len(), 2);
        assert_eq!((gold[0].x, gold[0].y, gold[0].u), (20.0, 12.0, cell('5')));
        assert_eq!((gold[1].x, gold[1].y, gold[1].u), (27.0, 12.0, cell('2')));
        let mut low = view();
        low.level = 7;
        let quads = painted(&low);
        let gold: Vec<&Quad> =
            with_texture(&quads, DIGIT_TEXTURE).into_iter().filter(|q| q.color == GOLD.rgba()).collect();
        assert_eq!(gold.len(), 1);
        assert_eq!((gold[0].x, gold[0].u), (24.0, cell('7')));
    }

    #[test]
    fn rows_show_label_slash_current_and_max_in_digit_cells() {
        let quads = painted(&view());
        for (y, label, current, max) in [(29.0, "CP", "100", "100"), (43.0, "HP", "450", "900"), (57.0, "MP", "200", "300")]
        {
            let row = digits(&quads, y);
            assert_eq!(row.len(), 2 + 1 + current.len() + max.len(), "linha {label}");
            assert_eq!(strip_text(&row), format!("{label}/{current}{max}"));
            assert_eq!(row[0].x, 25.0);
            assert_eq!(row[1].x, 33.0);
            assert_eq!(row[2].x, 89.0);
            assert_eq!(row[3].x, 83.0 - 18.0);
            assert_eq!(row[6].x, 95.0);
            assert!(row.iter().all(|q| q.color == TEXT.rgba()));
        }
    }

    #[test]
    fn exp_percent_is_drawn_with_the_digit_strip_centered_by_the_font_width() {
        let quads = painted(&view());
        let row = digits(&quads, 71.0);
        assert_eq!(strip_text(&row), "16.34%");
        assert_eq!(row[0].x, (75 - (6 + 6 + 4 + 6 + 6 + 11) / 2 + 12) as f32);
        let mut list = DrawList::default();
        paint_status(&mut list, 0, 0, &StatusView { exp: 51262490, ..view() }, &font(), 1.0);
        assert_eq!(strip_text(&digits(&list.quads, 71.0)), "0.00%");
    }

    #[test]
    fn name_sits_at_forty_nine_and_is_ellipsized_to_the_control_width() {
        let font = font();
        let mut list = DrawList::default();
        paint_status(&mut list, 100, 50, &view(), &font, 1.0);
        let text: Vec<&Quad> = list.quads.iter().filter(|q| q.texture == font.atlas).collect();
        assert_eq!(text.len(), 6);
        assert_eq!((text[0].x, text[0].y), (140.0, 59.0));
        assert_eq!(text[0].color, TEXT.rgba());
        assert_eq!(ellipsize("Killer", 122, &font), "Killer");
        assert_eq!(ellipsize(&"a".repeat(30), 122, &font), format!("{}..", "a".repeat(18)));
        assert_eq!(ellipsize(&"a".repeat(21), 122, &font), format!("{}..", "a".repeat(18)));
        assert_eq!(ellipsize("", 122, &font), "");
        let mut list = DrawList::default();
        paint_status(&mut list, 0, 0, &StatusView { name: "a".repeat(40), ..view() }, &font, 1.0);
        let name: Vec<&Quad> = list.quads.iter().filter(|q| q.texture == font.atlas).collect();
        assert_eq!(name.len(), 20);
        assert!(name.iter().all(|q| q.x + q.w <= 40.0 + 122.0));
    }

    #[test]
    fn zero_maximums_do_not_divide_by_zero() {
        let empty = StatusView { name: "".into(), level: 1, hp: (0, 0), mp: (0, 0), cp: (0, 0), exp: 0, regen: 0.0 };
        let quads = painted(&empty);
        assert!(with_texture(&quads, PS_CPBAR).is_empty());
        assert!(with_texture(&quads, PS_HPBAR).is_empty());
        assert!(with_texture(&quads, PS_HPBARWARN1).is_empty());
        assert_eq!(with_texture(&quads, PS_CPBAR_BACK).len(), 1);
        let mut over = view();
        over.hp = (2000, 900);
        assert_eq!(with_texture(&painted(&over), PS_HPBAR)[0].w, 150.0);
        assert_eq!(fill_width((-5, 100), 150), 0);
    }

    #[test]
    fn frame_and_body_carry_the_window_alpha() {
        let mut list = DrawList::default();
        paint_status(&mut list, 10, 20, &view(), &font(), 0.5);
        let grip = with_texture(&list.quads, crate::frame::SMALLBAR_2);
        assert_eq!((grip[0].x, grip[0].y, grip[0].h, grip[0].color[3]), (10.0, 28.0, 68.0, 0.5));
        let back1 = with_texture(&list.quads, SMALLWINDOW2_BACK1);
        assert_eq!((back1[0].x, back1[0].y, back1[0].vh, back1[0].color[3]), (22.0, 20.0, 76.0, 0.5));
        let end = with_texture(&list.quads, PS_SIZECONTROL1);
        assert_eq!((end[0].x, end[0].w, end[0].color[3]), (178.0, 4.0, 0.5));
        assert_eq!(with_texture(&list.quads, PS_HPBAR_BACK)[0].color[3], 1.0);
        assert_eq!(with_texture(&list.quads, PS_LEVELBACK)[0].color[3], 1.0);
    }

    #[test]
    fn every_texture_exists_in_the_client_list() {
        let known: Vec<String> = TEXTURES.lines().map(|l| l.trim().to_lowercase()).collect();
        let mut low = view();
        low.hp = (1, 900);
        let quads = painted(&low);
        for texture in quads.iter().map(|q| q.texture.as_str()).filter(|t| *t != "L2Font-e.SmallFont-e") {
            assert!(known.contains(&texture.to_lowercase()), "textura desconhecida {texture}");
        }
    }
}
