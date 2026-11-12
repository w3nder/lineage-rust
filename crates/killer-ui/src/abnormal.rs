use crate::color::Argb;
use crate::draw::{DrawList, NUMBER_STRIP, XLARGE_NUMBER_STRIP};
use crate::font::BitmapFont;
use crate::frame::{FrameCtrl, FrameMode};
use crate::window::Rect;

pub const ABNORMAL_SIZE: (i32, i32) = (252, 50);
pub const ABNORMAL_DEFAULT_POS: (i32, i32) = (172, 0);
pub const ABNORMAL_INITIAL_POSITION: crate::initial_position::InitialPosition = crate::initial_position::InitialPosition {
    enabled: true, horizontal_mode: 0, vertical_mode: 3, x: ABNORMAL_DEFAULT_POS.0, y: ABNORMAL_DEFAULT_POS.1,
};
pub const ABNORMAL_BACK: &str = "L2UI.EtcWndBack.AbnormalBack";
pub const DEBUFF_OVERLAY: &str = "L2UI_CH3.PlayerStatusWnd.debuff";
pub const TOOLTIP_MINI: &str = "L2UI.tooltip.mini-tool01";
pub const TOOLTIP_TOOL: [&str; 3] = ["L2UI.Tooltip.Tool01", "L2UI.Tooltip.Tool02", "L2UI.Tooltip.Tool03"];
pub const GRIP_SIZE: (i32, i32) = (12, 26);
pub const ICONS_X: i32 = 12;
pub const CELL: i32 = 26;
pub const ICON: i32 = 24;
pub const ICON_INSET: i32 = 1;
pub const ROW_STEP: i32 = 25;
pub const ICONS_PER_ROW: usize = 10;
pub const MAX_ROWS: usize = 3;
pub const MAX_BUFFS: usize = ICONS_PER_ROW * MAX_ROWS;
pub const SKILL_ICON_TEXELS: i32 = 32;
pub const TIMER_LIMIT: i32 = 60;
pub const TIMER_COLOR: Argb = Argb(0xCCDCDCDC);
pub const SHORT_TIMER_COLOR: Argb = Argb(0xFFDCDCDC);
pub const TIMER_Y: i32 = 10;
pub const TIMER_X_ONE_DIGIT: i32 = 7;
pub const TIMER_X_TWO_DIGITS: i32 = 2;
pub const SHORT_TIMER_Y: i32 = 8;
pub const SHORT_TIMER_CENTER_X: i32 = 25;
pub const ETC_X_AFTER_SHORT: i32 = 38;
pub const BLINK_SECONDS: i32 = 30;
pub const BLINK_TICK_MS: u32 = 100;
pub const BLINK_STEP: i32 = 25;
pub const BLINK_CYCLE_TICKS: u32 = 22;
pub const TIME_LEFT_SYSSTRING: u32 = 1199;
pub const HOUR_SYSSTRING: u32 = 1110;
pub const MINUTE_SYSSTRING: u32 = 1111;
pub const SECOND_UNIT: &str = "\u{CD08}";

pub fn blink_alpha(ticks: u32) -> u8 {
    let (mut alpha, mut fade_out) = (255i32, true);
    for _ in 0..ticks % BLINK_CYCLE_TICKS {
        alpha -= if fade_out { BLINK_STEP } else { -BLINK_STEP };
        if alpha < 0 {
            alpha = 0;
            fade_out = false;
        } else if alpha > 255 {
            alpha = 255;
            fade_out = true;
        }
    }
    alpha as u8
}

pub fn blink_ticks(duration: i32, elapsed_ms: u64) -> u32 {
    let start = u64::try_from(duration - BLINK_SECONDS).unwrap_or(0) * 1000;
    (elapsed_ms.saturating_sub(start) / u64::from(BLINK_TICK_MS)) as u32
}

pub fn short_time_text(seconds: i32, hour: &str, minute: &str) -> String {
    let rest = seconds % 86400;
    let (hours, minutes, secs) = (rest / 3600, rest % 3600 / 60, rest % 3600 % 60);
    if hours > 0 {
        format!("{hours}{hour}")
    } else if minutes > 0 {
        format!("{minutes}{minute}")
    } else {
        format!("{secs}{SECOND_UNIT}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BuffIcon {
    pub skill_id: i32,
    pub level: i32,
    pub icon: Option<String>,
    pub remaining: Option<i32>,
    pub debuff: bool,
    pub fade_ticks: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AbnormalView {
    pub buffs: Vec<BuffIcon>,
    pub short_buff: Option<BuffIcon>,
    pub etc: Vec<Option<String>>,
    pub etc_skills: Vec<(i32, i32)>,
}

impl BuffIcon {
    pub fn blinks(&self) -> bool {
        self.remaining.is_some_and(|r| (0..=BLINK_SECONDS).contains(&r))
    }

    pub fn alpha(&self) -> f32 {
        if self.blinks() { f32::from(blink_alpha(self.fade_ticks)) / 255.0 } else { 1.0 }
    }

    pub fn shows_time_left(&self) -> bool {
        self.remaining.is_some_and(|r| r >= 0) && !self.debuff
    }
}

impl AbnormalView {
    pub fn is_empty(&self) -> bool {
        self.buffs.is_empty() && self.short_buff.is_none() && self.etc.is_empty()
    }

    pub fn extra_row(&self) -> i32 {
        match self.buffs.len() {
            0 => 0,
            1..=10 => 1,
            11..=20 => 2,
            _ => 3,
        }
    }

    pub fn rows(&self) -> i32 {
        let buff_rows = self.buffs.len().min(MAX_BUFFS).div_ceil(ICONS_PER_ROW) as i32;
        if self.short_buff.is_some() || !self.etc.is_empty() { self.extra_row() + 1 } else { buff_rows }
    }

    pub fn height(&self) -> i32 {
        self.size().1
    }

    pub fn size(&self) -> (i32, i32) {
        let n = self.buffs.len() as i32;
        let extra = i32::from(self.short_buff.is_some()) + self.etc.len() as i32;
        let full = ICONS_X + ICONS_PER_ROW as i32 * CELL;
        if n > 10 {
            return if n < 21 && extra == 0 {
                (full, 2 * ROW_STEP + 1)
            } else if n > 30 || (n > 20 && extra > 0) {
                (full, 4 * ROW_STEP + 1)
            } else {
                (full, 3 * ROW_STEP + 1)
            };
        }
        if extra == 0 {
            (n * CELL + ICONS_X, ROW_STEP + 1)
        } else if extra <= n {
            (n * CELL + ICONS_X, 2 * ROW_STEP + 1)
        } else {
            let x = if self.short_buff.is_some() { ETC_X_AFTER_SHORT } else { ICONS_X };
            (self.etc.len() as i32 * CELL + x, if n != 0 { 2 * ROW_STEP + 1 } else { ROW_STEP + 1 })
        }
    }

    pub fn grip(&self) -> Rect {
        Rect::new(0, 0, GRIP_SIZE.0, self.size().1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbnormalHit {
    Buff(usize),
    ShortBuff,
    Etc(usize),
}

pub fn buff_cell(index: usize) -> Option<Rect> {
    if index >= MAX_BUFFS {
        return None;
    }
    let (row, col) = ((index / ICONS_PER_ROW) as i32, (index % ICONS_PER_ROW) as i32);
    Some(Rect::new(ICONS_X + col * CELL, row * ROW_STEP, CELL, CELL))
}

pub fn short_buff_cell(view: &AbnormalView) -> Rect {
    Rect::new(ICONS_X, view.extra_row() * ROW_STEP, CELL, CELL)
}

pub fn etc_cell(view: &AbnormalView, index: usize) -> Rect {
    let x = if view.short_buff.is_some() { ETC_X_AFTER_SHORT } else { ICONS_X };
    Rect::new(x + index as i32 * CELL, view.extra_row() * ROW_STEP, CELL, CELL)
}

pub const HIT_ICON_SIZE: i32 = 26;
pub const HIT_GRID_COLUMNS: i32 = 10;
pub const HIT_MAX_INDEX: i32 = 40;

pub fn grid_index(x: i32, y: i32) -> Option<i32> {
    grid_index_sized(x, y, HIT_ICON_SIZE)
}

pub fn grid_index_sized(x: i32, y: i32, icon_size: i32) -> Option<i32> {
    let (rx, ry) = (x - ICONS_X, y);
    if !(0..=(icon_size * 5 + 5) * 2).contains(&rx) || !(0..=icon_size * 4 + 4).contains(&ry) {
        return None;
    }
    Some(rx / (icon_size + 1) + ry / (icon_size + 1) * HIT_GRID_COLUMNS)
}

pub fn hit(view: &AbnormalView, x: i32, y: i32) -> Option<AbnormalHit> {
    let index = grid_index(x, y).filter(|index| (0..=HIT_MAX_INDEX).contains(index))? as usize;
    let rows = match view.buffs.len() {
        0 => 0,
        1..=10 => 10,
        11..=20 => 20,
        _ => 30,
    };
    if index < rows {
        return (index < view.buffs.len()).then_some(AbnormalHit::Buff(index));
    }
    let mut slot = index - rows;
    if view.short_buff.is_some() {
        if slot == 0 {
            return Some(AbnormalHit::ShortBuff);
        }
        slot -= 1;
    }
    (slot < view.etc.len()).then_some(AbnormalHit::Etc(slot))
}

pub fn paint_abnormal(list: &mut DrawList, x: i32, y: i32, view: &AbnormalView, font: &BitmapFont, alpha: f32) {
    if view.is_empty() {
        return;
    }
    let grip = view.grip();
    FrameCtrl::new(FrameMode::VerticalGrip).paint(list, x, y, grip.w, grip.h, font, alpha);
    for (index, buff) in view.buffs.iter().enumerate().take(MAX_BUFFS) {
        let Some(cell) = buff_cell(index) else { continue };
        let (cx, cy) = (x + cell.x, y + cell.y);
        paint_back(list, cx, cy, alpha);
        let (ix, iy) = (cx + ICON_INSET, cy + ICON_INSET);
        if let Some(icon) = &buff.icon {
            paint_icon_alpha(list, ix, iy, icon, buff.alpha());
        }
        if let Some(remaining) = buff.remaining.filter(|r| (0..TIMER_LIMIT).contains(r))
            && !buff.debuff
        {
            let text = remaining.to_string();
            let dx = if remaining < 10 { TIMER_X_ONE_DIGIT } else { TIMER_X_TWO_DIGITS };
            list.draw_digits_with(XLARGE_NUMBER_STRIP, ix + dx, iy + TIMER_Y, &text, TIMER_COLOR);
        }
        if buff.debuff && buff.icon.is_some() {
            list.draw_texture(ix, iy, ICON, ICON, 0, 0, CELL, CELL, DEBUFF_OVERLAY, buff.alpha());
        }
    }
    if let Some(short) = &view.short_buff {
        let cell = short_buff_cell(view);
        let (cx, cy) = (x + cell.x, y + cell.y);
        paint_back(list, cx, cy, alpha);
        if let Some(icon) = &short.icon {
            paint_icon_alpha(list, cx + ICON_INSET, cy + ICON_INSET, icon, short.alpha());
        }
        if let Some(remaining) = short.remaining.filter(|r| (0..TIMER_LIMIT).contains(r))
            && !short.debuff
        {
            let text = remaining.to_string();
            let width = font.measure(&text) as i32;
            list.draw_digits_with(
                NUMBER_STRIP,
                x + SHORT_TIMER_CENTER_X - width / 2,
                cy + ICON_INSET + SHORT_TIMER_Y,
                &text,
                SHORT_TIMER_COLOR,
            );
        }
    }
    for (index, icon) in view.etc.iter().enumerate() {
        let cell = etc_cell(view, index);
        let (cx, cy) = (x + cell.x, y + cell.y);
        paint_back(list, cx, cy, alpha);
        if let Some(icon) = icon {
            paint_icon(list, cx + ICON_INSET, cy + ICON_INSET, icon);
        }
    }
}

fn paint_back(list: &mut DrawList, x: i32, y: i32, alpha: f32) {
    list.draw_texture(x, y, CELL, CELL, 0, 0, CELL, CELL, ABNORMAL_BACK, alpha);
}

pub fn paint_icon(list: &mut DrawList, x: i32, y: i32, icon: &str) {
    paint_icon_alpha(list, x, y, icon, 1.0);
}

pub fn paint_icon_alpha(list: &mut DrawList, x: i32, y: i32, icon: &str, alpha: f32) {
    list.draw_texture(x, y, ICON, ICON, 0, 0, SKILL_ICON_TEXELS, SKILL_ICON_TEXELS, icon, alpha);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::{DIGIT_TEXTURE, Quad, XLARGE_DIGIT_TEXTURE};
    use crate::font::{Glyphs, gly};

    const TEXTURES: &str = include_str!("../tests/fixtures/nwindow-texturas-abnormal.txt");

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn buff(icon: &str, remaining: Option<i32>, debuff: bool) -> BuffIcon {
        BuffIcon { icon: Some(icon.to_string()), remaining, debuff, ..Default::default() }
    }

    fn painted(view: &AbnormalView) -> Vec<Quad> {
        let mut list = DrawList::default();
        paint_abnormal(&mut list, 0, 0, view, &font(), 1.0);
        list.quads
    }

    fn with_texture<'a>(quads: &'a [Quad], texture: &str) -> Vec<&'a Quad> {
        quads.iter().filter(|q| q.texture == texture).collect()
    }

    fn twelve_buffs() -> Vec<BuffIcon> {
        (0..12).map(|i| buff(&format!("Icon.skill{i}"), Some(1200), false)).collect()
    }

    #[test]
    fn buffs_fill_rows_of_ten_cells_of_twenty_six_pixels_every_twenty_five() {
        let view = AbnormalView { buffs: twelve_buffs(), ..Default::default() };
        let quads = painted(&view);
        let backs = with_texture(&quads, ABNORMAL_BACK);
        assert_eq!(backs.len(), 12);
        assert_eq!(
            (backs[0].x, backs[0].y, backs[0].w, backs[0].h, backs[0].uw, backs[0].vh),
            (12.0, 0.0, 26.0, 26.0, 26.0, 26.0)
        );
        assert_eq!((backs[9].x, backs[9].y), (12.0 + 9.0 * 26.0, 0.0));
        assert_eq!((backs[10].x, backs[10].y), (12.0, 25.0));
        assert_eq!((backs[11].x, backs[11].y), (38.0, 25.0));
        let icon = with_texture(&quads, "Icon.skill11");
        assert_eq!(
            (icon[0].x, icon[0].y, icon[0].w, icon[0].h, icon[0].uw, icon[0].vh),
            (39.0, 26.0, 24.0, 24.0, 32.0, 32.0)
        );
        let grip = with_texture(&quads, crate::frame::SMALLBAR_2);
        assert_eq!((grip[0].x, grip[0].y, grip[0].w, grip[0].h), (0.0, 8.0, 12.0, 35.0));
        assert!(with_texture(&quads, XLARGE_DIGIT_TEXTURE).is_empty());
        assert_eq!(view.rows(), 2);
        assert_eq!((view.size(), view.grip()), ((272, 51), Rect::new(0, 0, 12, 51)));
    }

    #[test]
    fn timers_under_sixty_seconds_use_the_xlarge_strip_shifted_by_the_digit_count() {
        let buffs = vec![
            buff("Icon.skill1204", Some(59), false),
            buff("Icon.skill1040", Some(7), false),
            buff("Icon.skill1", Some(60), false),
        ];
        let quads = painted(&AbnormalView { buffs, ..Default::default() });
        let digits = with_texture(&quads, XLARGE_DIGIT_TEXTURE);
        assert_eq!(digits.len(), 3);
        assert_eq!((digits[0].x, digits[0].y, digits[0].w, digits[0].h), (13.0 + 2.0, 1.0 + 10.0, 11.0, 14.0));
        assert_eq!(digits[1].x, 13.0 + 2.0 + 11.0);
        assert_eq!((digits[2].x, digits[2].y), (39.0 + 7.0, 11.0));
        assert!(digits.iter().all(|q| q.color == TIMER_COLOR.rgba()));
        assert_eq!(TIMER_COLOR.rgba()[3], 0xCC as f32 / 255.0);
    }

    #[test]
    fn debuffs_get_the_overlay_and_no_timer() {
        let buffs =
            vec![buff("Icon.skill1069", Some(5), true), BuffIcon { icon: None, remaining: Some(5), debuff: true, ..Default::default() }];
        let quads = painted(&AbnormalView { buffs, ..Default::default() });
        let overlay = with_texture(&quads, DEBUFF_OVERLAY);
        assert_eq!(overlay.len(), 1);
        assert_eq!(
            (overlay[0].x, overlay[0].y, overlay[0].w, overlay[0].h, overlay[0].uw, overlay[0].vh),
            (13.0, 1.0, 24.0, 24.0, 26.0, 26.0)
        );
        assert!(with_texture(&quads, XLARGE_DIGIT_TEXTURE).is_empty());
        assert_eq!(with_texture(&quads, ABNORMAL_BACK).len(), 2);
    }

    #[test]
    fn the_short_buff_sits_on_the_row_after_the_buffs_with_a_centered_small_timer() {
        let view = AbnormalView {
            buffs: twelve_buffs(),
            short_buff: Some(buff("Icon.skill2274", Some(12), false)),
            ..Default::default()
        };
        let quads = painted(&view);
        let backs = with_texture(&quads, ABNORMAL_BACK);
        assert_eq!((backs[12].x, backs[12].y), (12.0, 50.0));
        let icon = with_texture(&quads, "Icon.skill2274");
        assert_eq!((icon[0].x, icon[0].y), (13.0, 51.0));
        let digits = with_texture(&quads, DIGIT_TEXTURE);
        assert_eq!(digits.len(), 2);
        assert_eq!((digits[0].x, digits[0].y), (25.0 - 6.0, 59.0));
        assert_eq!(digits[0].color, SHORT_TIMER_COLOR.rgba());
        assert_eq!(view.rows(), 3);
        let empty = AbnormalView { short_buff: Some(buff("Icon.skill2274", None, false)), ..Default::default() };
        assert_eq!(with_texture(&painted(&empty), ABNORMAL_BACK)[0].y, 0.0);
        assert!(with_texture(&painted(&empty), DIGIT_TEXTURE).is_empty());
        let one_row = AbnormalView {
            buffs: twelve_buffs()[..10].to_vec(),
            short_buff: Some(buff("a", None, false)),
            ..Default::default()
        };
        assert_eq!(short_buff_cell(&one_row), Rect::new(12, 25, 26, 26));
        let three_rows = AbnormalView {
            buffs: (0..25).map(|_| buff("a", None, false)).collect(),
            short_buff: Some(buff("a", None, false)),
            ..Default::default()
        };
        assert_eq!(short_buff_cell(&three_rows), Rect::new(12, 75, 26, 26));
    }

    #[test]
    fn etc_icons_share_the_extra_row_after_the_short_buff() {
        let view = AbnormalView {
            buffs: vec![buff("Icon.skill1", None, false)],
            short_buff: Some(buff("Icon.skill2", None, false)),
            etc: vec![Some("Icon.etc_weight".into()), None],
            ..Default::default()
        };
        let quads = painted(&view);
        let backs = with_texture(&quads, ABNORMAL_BACK);
        assert_eq!(backs.len(), 4);
        assert_eq!((backs[2].x, backs[2].y), (38.0, 25.0));
        assert_eq!((backs[3].x, backs[3].y), (64.0, 25.0));
        assert_eq!(with_texture(&quads, "Icon.etc_weight")[0].x, 39.0);
        let without_short = AbnormalView { short_buff: None, ..view.clone() };
        assert_eq!(with_texture(&painted(&without_short), ABNORMAL_BACK)[1].x, 12.0);
        assert_eq!(hit(&view, 40, 30), Some(AbnormalHit::Etc(0)));
        assert_eq!(hit(&view, 20, 30), Some(AbnormalHit::ShortBuff));
        assert_eq!(hit(&view, 20, 5), Some(AbnormalHit::Buff(0)));
        assert_eq!(hit(&view, 5, 5), None);
        assert_eq!(hit(&view, 200, 30), None);
    }

    #[test]
    fn the_window_resizes_to_the_icons_like_the_resize_slot() {
        let buffs = |n: usize| -> Vec<BuffIcon> { (0..n).map(|_| buff("a", None, false)).collect() };
        let short = Some(buff("s", None, false));
        assert_eq!(AbnormalView { buffs: buffs(3), ..Default::default() }.size(), (3 * 26 + 12, 26));
        assert_eq!(AbnormalView { buffs: buffs(0), short_buff: short.clone(), ..Default::default() }.size(), (38, 26));
        assert_eq!(AbnormalView { buffs: buffs(0), etc: vec![None, None], ..Default::default() }.size(), (64, 26));
        assert_eq!(AbnormalView { buffs: buffs(1), short_buff: short.clone(), etc: vec![None, None], etc_skills: Vec::new() }.size(), (90, 51));
        assert_eq!(AbnormalView { buffs: buffs(3), short_buff: short.clone(), ..Default::default() }.size(), (90, 51));
        assert_eq!(AbnormalView { buffs: buffs(12), ..Default::default() }.size(), (272, 51));
        assert_eq!(
            AbnormalView { buffs: buffs(12), short_buff: short.clone(), ..Default::default() }.size(),
            (272, 76)
        );
        assert_eq!(AbnormalView { buffs: buffs(25), ..Default::default() }.size(), (272, 76));
        assert_eq!(AbnormalView { buffs: buffs(25), short_buff: short, ..Default::default() }.size(), (272, 101));
        assert_eq!(AbnormalView { buffs: buffs(31), ..Default::default() }.size(), (272, 101));
    }

    #[test]
    fn an_empty_view_paints_nothing_and_more_than_thirty_buffs_are_cut() {
        assert!(painted(&AbnormalView::default()).is_empty());
        let view = AbnormalView { buffs: (0..35).map(|_| buff("a", None, false)).collect(), ..Default::default() };
        assert_eq!(with_texture(&painted(&view), ABNORMAL_BACK).len(), 30);
        assert_eq!(view.rows(), 3);
        assert_eq!(view.height(), 101);
        assert_eq!(buff_cell(30), None);
        assert_eq!(buff_cell(29), Some(Rect::new(12 + 9 * 26, 50, 26, 26)));
    }

    #[test]
    fn backgrounds_carry_the_window_alpha_and_icons_do_not() {
        let mut list = DrawList::default();
        let view = AbnormalView { buffs: vec![buff("Icon.skill1", Some(3), true)], ..Default::default() };
        paint_abnormal(&mut list, 100, 20, &view, &font(), 0.5);
        assert_eq!(with_texture(&list.quads, ABNORMAL_BACK)[0].color[3], 0.5);
        assert_eq!(
            (with_texture(&list.quads, ABNORMAL_BACK)[0].x, with_texture(&list.quads, ABNORMAL_BACK)[0].y),
            (112.0, 20.0)
        );
        assert_eq!(with_texture(&list.quads, "Icon.skill1")[0].color[3], 1.0);
        assert_eq!(with_texture(&list.quads, DEBUFF_OVERLAY)[0].color[3], 1.0);
        assert_eq!(with_texture(&list.quads, crate::frame::SMALLBAR_1)[0].color[3], 0.5);
    }

    #[test]
    fn every_texture_exists_in_the_client_list() {
        let known: Vec<String> = TEXTURES.lines().map(|l| l.trim().to_lowercase()).collect();
        let view = AbnormalView {
            buffs: vec![buff("Icon.skill1", Some(3), true), buff("Icon.skill2", Some(3), false)],
            short_buff: Some(buff("Icon.skill3", Some(3), false)),
            etc: vec![Some("Icon.skill4".into())],
            etc_skills: Vec::new(),
        };
        for texture in painted(&view).iter().map(|q| q.texture.as_str()).filter(|t| !t.starts_with("Icon.")) {
            assert!(known.contains(&texture.to_lowercase()), "textura desconhecida {texture}");
        }
        for texture in [TOOLTIP_MINI, TOOLTIP_TOOL[0], TOOLTIP_TOOL[1], TOOLTIP_TOOL[2]] {
            assert!(known.contains(&texture.to_lowercase()), "textura desconhecida {texture}");
        }
    }

    fn replay(ticks: u32) -> (i32, bool) {
        let (mut alpha, mut out) = (255, true);
        for _ in 0..ticks {
            if alpha < 0 {
                alpha = 0;
                out = false;
            } else if alpha > 255 {
                alpha = 255;
                out = true;
            }
            alpha += if out { -25 } else { 25 };
            if alpha < 0 {
                alpha = 0;
                out = false;
            } else if alpha > 255 {
                alpha = 255;
                out = true;
            }
        }
        (alpha, out)
    }

    #[test]
    fn the_blink_follows_the_fade_timer_state_machine_and_repeats_every_22_ticks() {
        for ticks in 0..80 {
            assert_eq!(i32::from(blink_alpha(ticks)), replay(ticks).0, "{ticks}");
        }
        assert_eq!(blink_alpha(0), 255);
        assert_eq!(blink_alpha(1), 230);
        assert_eq!(blink_alpha(10), 5);
        assert_eq!(blink_alpha(11), 0);
        assert_eq!(blink_alpha(21), 250);
        assert_eq!(blink_alpha(22), 255);
        assert_eq!(blink_ticks(120, 89_999), 0);
        assert_eq!(blink_ticks(120, 90_000), 0);
        assert_eq!(blink_ticks(120, 90_350), 3);
        assert_eq!(blink_ticks(10, 350), 3);
    }

    #[test]
    fn icons_in_their_last_thirty_seconds_blink_and_the_debuff_frame_follows_the_icon_alpha() {
        let mut calm = buff("Icon.skill1", Some(31), false);
        calm.fade_ticks = 1;
        let mut blinking = buff("Icon.skill2", Some(30), false);
        blinking.fade_ticks = 1;
        let mut debuff = buff("Icon.skill3", Some(0), true);
        debuff.fade_ticks = 1;
        let quads = painted(&AbnormalView { buffs: vec![calm, blinking, debuff], ..Default::default() });
        let alpha = |texture: &str| (with_texture(&quads, texture)[0].color[3] * 255.0).round() as i32;
        assert_eq!(alpha("Icon.skill1"), 255);
        assert_eq!(alpha("Icon.skill2"), 230);
        assert_eq!(alpha("Icon.skill3"), 230);
        assert_eq!(alpha(DEBUFF_OVERLAY), 230);
        assert_eq!(alpha(ABNORMAL_BACK), 255);
        assert!(buff("a", Some(0), false).blinks() && !buff("a", Some(31), false).blinks() && !buff("a", None, false).blinks());
    }

    #[test]
    fn the_tooltip_time_left_uses_the_hour_minute_or_second_unit() {
        assert_eq!(short_time_text(7200 + 5, "h", "m"), "2h");
        assert_eq!(short_time_text(3599, "h", "m"), "59m");
        assert_eq!(short_time_text(59, "h", "m"), "59\u{CD08}");
        assert_eq!(short_time_text(86400 + 61, "h", "m"), "1m");
        assert!(buff("a", Some(10), false).shows_time_left());
        assert!(!buff("a", Some(10), true).shows_time_left() && !buff("a", None, false).shows_time_left());
    }

    #[test]
    fn the_hit_test_keeps_the_stock_stride_of_27_the_12_pixel_shift_and_the_ten_wide_grid() {
        let ten = AbnormalView { buffs: (0..10).map(|_| buff("a", None, false)).collect(), ..Default::default() };
        assert_eq!(grid_index(12, 0), Some(0));
        assert_eq!(grid_index(12 + 26, 0), Some(0));
        assert_eq!(grid_index(12 + 27, 0), Some(1));
        assert_eq!(grid_index(12 + 270, 0), Some(10));
        assert_eq!(grid_index(12 + 271, 0), None);
        assert_eq!(grid_index(11, 0), None);
        assert_eq!(grid_index(12, 108), Some(40));
        assert_eq!(grid_index(12, 109), None);
        assert_eq!(hit(&ten, 12 + 9 * 26 + 1, 3), Some(AbnormalHit::Buff(8)));
        assert_eq!(hit(&ten, 12 + 9 * 27, 3), Some(AbnormalHit::Buff(9)));
        assert_eq!(hit(&ten, 12 + 270, 3), None);
        let with_etc = AbnormalView { etc: vec![None, None], etc_skills: vec![(4271, 1), (4270, 2)], ..ten.clone() };
        assert_eq!(hit(&with_etc, 12, 27), Some(AbnormalHit::Etc(0)));
        assert_eq!(hit(&with_etc, 12 + 27, 27), Some(AbnormalHit::Etc(1)));
        assert_eq!(hit(&with_etc, 12 + 54, 27), None);
        let twelve = AbnormalView { buffs: twelve_buffs(), short_buff: Some(buff("s", None, false)), etc: vec![None], ..Default::default() };
        assert_eq!(hit(&twelve, 12 + 27, 27), Some(AbnormalHit::Buff(11)));
        assert_eq!(hit(&twelve, 12, 54), Some(AbnormalHit::ShortBuff));
        assert_eq!(hit(&twelve, 12 + 27, 54), Some(AbnormalHit::Etc(0)));
    }
}
