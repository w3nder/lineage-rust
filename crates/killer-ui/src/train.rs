#[path = "train_layout_generated.rs"]
mod layout;

#[path = "train_controls_generated.rs"]
mod controls;
#[path = "train_frame_generated.rs"]
mod frame_controls;

use crate::button::TextButton;
use crate::chat::wrap_line;
use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::frame_contract::FrameHit;
use crate::scroll::{ScrollHit, ScrollWnd, line_down, line_up, rows_for};
use crate::window::Rect;

pub const SIZE: (i32, i32) = layout::SIZE;
pub const LIST_PAGE: Rect = layout::RECT_E8;
pub const INFO_PAGE: Rect = layout::RECT_EC;
pub const LIST_BACK: &str = "L2UI_CH3.SkillTrainWnd.SkillTrain1";
pub const ENCHANT_BACK: &str = "L2UI_CH3.SkillTrainWnd.skillenchant_back";
pub const INFO_BACK: &str = "L2UI_CH3.SkillTrainWnd.SkillTrain2";
pub const LIST_RECT: Rect = Rect::new(20, 47, 230, 274);
pub const LIST_ROWS: usize = 7;
pub const ROW_HEIGHT: i32 = 40;
pub const OK_RECT: Rect = controls::BUTTON_5C0.rect_at((INFO_PAGE.x, INFO_PAGE.y));
pub const CANCEL_RECT: Rect = controls::BUTTON_5C4.rect_at((INFO_PAGE.x, INFO_PAGE.y));
const OUTLINE_DOWN: &str = "l2ui_ch3.InventoryWnd.Inventory_OutLine_down";
const OUTLINE: &str = "l2ui_ch3.InventoryWnd.Inventory_OutLine";

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    crate::mainwnd::default_position(viewport)
}

pub fn initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    crate::mainwnd::initial_position(viewport)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrainHit { Close, Skill(usize), Scroll(ScrollHit), Learn, Cancel }

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LearnableView {
    pub enchant_description: Option<String>,
    pub id: i32,
    pub level: i32,
    pub sp_cost: i32,
    pub icon: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrainRequirement {
    pub item_id: i32,
    pub count: i32,
    pub name: String,
    pub icon: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnchantInfo {
    pub exp_cost: i32,
    pub chance: i32,
    pub route: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TrainInfoView {
    pub enchant: Option<EnchantInfo>,
    pub id: i32,
    pub level: i32,
    pub mode: i32,
    pub sp_cost: i32,
    pub icon: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub mp: i32,
    pub range: i32,
    pub requirements: Vec<TrainRequirement>,
}

#[derive(Debug, Clone)]
pub struct TrainView {
    pub title: String,
    pub list: Vec<LearnableView>,
    pub info: Option<TrainInfoView>,
    pub selected: Option<usize>,
    pub scroll: ScrollWnd,
    pub pressed: Option<TrainHit>,
    pub learn: TextButton,
    pub cancel: TextButton,
    pub frame: FrameCtrl,
    pub first_row: usize,
    pub player_sp: Option<i32>,
    pub player_exp: Option<i32>,
    pub enchant_list: bool,
    labels: [String; 11],
}

impl TrainView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id| lookup(id).unwrap_or_default();
        let title = text(477);
        Self {
            title: title.clone(), list: Vec::new(), info: None, selected: None,
            scroll: ScrollWnd::new(LIST_RECT.w, LIST_RECT.h), pressed: None,
            learn: controls::BUTTON_5C0.build((0, 20), text),
            cancel: controls::BUTTON_5C4.build((0, 20), text),
            frame: frame_controls::FRAME_7C.build(text), first_row: 0, player_sp: None, player_exp: None, enchant_list: false,
            labels: [364, 92, 88, 371, 366, 367, 320, 321, 365, 642, 1219].map(text),
        }
    }

    pub fn clear(&mut self) {
        self.list.clear(); self.info = None; self.selected = None; self.first_row = 0;
        self.scroll.slider.mouse_up();
        self.scroll.slider.set_range(0, LIST_ROWS as i32);
        self.scroll.slider.set_pos(0); self.pressed = None;
    }

    pub fn set_list(&mut self, list: Vec<LearnableView>) {
        self.clear(); self.list = list;
        self.scroll.slider.set_range(rows_for(self.list.len(), 1), LIST_ROWS as i32);
    }

    pub fn show_info(&mut self, info: TrainInfoView) {
        self.info = Some(info);
        self.pressed = None;
        self.scroll.slider.mouse_up();
    }

    pub fn back(&mut self) { self.info = None; self.pressed = None; }

    pub fn hit(&self, x: i32, y: i32) -> Option<TrainHit> {
        if frame_controls::FRAME_7C.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close) {
            return Some(TrainHit::Close);
        }
        if self.info.is_some() {
            if controls::BUTTON_5C0.hit(&self.learn, x, y).is_some() { return Some(TrainHit::Learn); }
            if controls::BUTTON_5C4.hit(&self.cancel, x, y).is_some() { return Some(TrainHit::Cancel); }
            return None;
        }
        if let Some(hit) = self.scroll.hit(LIST_RECT.w, LIST_RECT.h, x - LIST_RECT.x, y - LIST_RECT.y) { return Some(TrainHit::Scroll(hit)); }
        self.skill_at(x, y).map(TrainHit::Skill)
    }

    pub fn skill_at(&self, x: i32, y: i32) -> Option<usize> {
        let (x, y) = (x - LIST_RECT.x, y - LIST_RECT.y);
        if !(0..=LIST_RECT.w).contains(&x) || !(0..=LIST_RECT.h).contains(&y) { return None; }
        if x < LIST_RECT.w && y >= 1 {
            let row = ((y - 1) / ROW_HEIGHT) as usize;
            let index = self.first_row + row;
            if row < LIST_ROWS && index < self.list.len() { return Some(index); }
        }
        self.selected
    }

    pub fn select(&mut self, index: usize) { if index < self.list.len() { self.selected = Some(index); } }

    pub fn scroll(&mut self, hit: ScrollHit) {
        if self.info.is_some() { return; }
        let first = self.first_row as i32;
        let total = rows_for(self.list.len(), 1);
        let next = match hit { ScrollHit::Up => line_up(first, LIST_ROWS as i32, total), ScrollHit::Down => line_down(first, LIST_ROWS as i32, total), _ => first };
        self.first_row = next.max(0) as usize;
        self.scroll.slider.set_pos(next);
    }

    pub fn mouse_move(&mut self, y: i32) -> bool {
        if let Some(first) = self.scroll.slider.mouse_move(y - LIST_RECT.y - crate::scroll::BUTTON_SIZE) {
            self.first_row = first.max(0) as usize;
            return true;
        }
        false
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        frame_controls::FRAME_7C.paint(&self.frame, list, (x, y), font, alpha);
        let page = if self.info.is_some() { INFO_PAGE } else { LIST_PAGE };
        let (x, y) = (x + page.x, y + page.y);
        list.draw_texture(x, y, page.w, page.h, 0, 0, page.w, page.h, if self.info.as_ref().is_some_and(|i| i.enchant.is_some()) { ENCHANT_BACK } else if self.info.is_some() { INFO_BACK } else { LIST_BACK }, alpha);
        if let Some(info) = &self.info {
            list.draw_text(x + 11, y + 12, Argb(0xffdcdcdc), &self.labels[4], font);
            list.draw_text(x + 11, y + if info.enchant.is_some() { 155 } else { 200 }, Argb(0xffdcdcdc), &self.labels[5], font);
            icon(list, x + 14, y + 30, &info.icon, alpha);
            list.draw_text(x + 51, y + 32, Argb(0xffdcdcdc), &info.name, font);
            if let Some(enchant) = &info.enchant {
                list.draw_text(x + 51, y + 48, Argb(0xffb09b79), &enchant.route, font);
                list.draw_text(x + 13, y + 68, Argb(0xffa3a3a3), &self.labels[9], font);
                list.draw_text(x + 18 + font.measure(&self.labels[9]) as i32, y + 68, Argb(0xffb09b79), &format!("{}%", enchant.chance), font);
                wrapped(list, (x + 13, y + 91), 230, &enchant.description, Argb(0xffb2becf), font);
                value(list, (x + 13, y + 175), &self.labels[10], enchant.exp_cost, font);
                value(list, (x + 13, y + 191), &self.labels[8], info.sp_cost, font);
                if let Some(exp) = self.player_exp {
                    let exp = exp.to_string();
                    list.draw_text(x + 224 - font.measure(&exp) as i32, y + 306, Argb(0xffdcdcdc), &exp, font);
                }
            } else {
            let level_x = x + 53 + font.measure(&self.labels[2]) as i32;
            list.draw_text(x + 51, y + 48, Argb(0xffa3a3a3), &self.labels[2], font);
            let level = info.level.to_string();
            list.draw_text(level_x, y + 48, Argb(0xffb09b79), &level, font);
            list.draw_text(level_x + 10 + font.measure(&level) as i32, y + 48, Argb(0xffb09b79), &info.category, font);
            value(list, (x + 13, y + 68), &self.labels[6], info.mp, font);
            if info.range >= 0 { value(list, (x + 13, y + 80), &self.labels[7], info.range, font); }
            wrapped(list, (x + 13, y + 102), 230, &info.description, Argb(0xffb2becf), font);
            value(list, (x + 13, y + 220), &self.labels[8], info.sp_cost, font);
            }
            for (index, req) in info.requirements.iter().take(4).enumerate() {
                let yy = y + if info.enchant.is_some() { 213 } else { 242 } + index as i32 * 34;
                list.draw_texture(x + 13, yy, 32, 32, 0, 0, 32, 32, &req.icon, alpha);
                wrapped(list, (x + 49, yy), 140, &format!("{} X {}", req.name, req.count), Argb(0xffdcdcdc), font);
            }
            controls::BUTTON_5C0.paint(&self.learn, list, (x, y - 20), self.pressed == Some(TrainHit::Learn), font);
            controls::BUTTON_5C4.paint(&self.cancel, list, (x, y - 20), self.pressed == Some(TrainHit::Cancel), font);
        } else {
            list.draw_text(x + 11, y + 12, Argb(0xffdcdcdc), &self.labels[0], font);
            for (index, entry) in self.list.iter().enumerate().skip(self.first_row).take(LIST_ROWS) {
                let yy = y + 28 + (index - self.first_row) as i32 * ROW_HEIGHT;
                let xx = x + LIST_RECT.x;
                if self.selected == Some(index) {
                    list.draw_texture(xx, yy - 4, LIST_RECT.w - 49, ROW_HEIGHT, 0, 0, 16, 13, crate::combo::SELECT, alpha);
                    list.draw_texture(xx + LIST_RECT.w - 49, yy - 4, 32, ROW_HEIGHT, 0, 0, 32, 13, crate::combo::SELECT_END, alpha);
                }
                icon(list, xx, yy, &entry.icon, alpha);
                list.draw_text(xx + 34, yy + 3, Argb(0xffdcdcdc), &entry.name, font);
                if let Some(description) = &entry.enchant_description {
                    list.draw_text(xx + 34, yy + 19, Argb(0xffb09b79), description, font);
                } else {
                list.draw_text(xx + 34, yy + 19, Argb(0xffa3a3a3), &self.labels[2], font);
                let level = if entry.level < 1 { self.labels[3].clone() } else { entry.level.to_string() };
                list.draw_text(xx + 36 + row_level_offset(font.measure(&self.labels[2]) as i32, &level), yy + 19, Argb(0xffb09b79), &level, font);
                value(list, (xx + 76, yy + 19), &self.labels[8], entry.sp_cost, font);
                }
            }
            self.scroll.paint(list, x + LIST_RECT.x, y - 20 + LIST_RECT.y, LIST_RECT.w, LIST_RECT.h);
        }
        if let Some(sp) = self.player_sp.filter(|_| self.info.is_some() || !self.enchant_list) {
            let sp = sp.to_string();
            list.draw_text(x + 224 - font.measure(&sp) as i32, y + 326, Argb(0xffdcdcdc), &sp, font);
            if self.info.as_ref().is_none_or(|i| i.enchant.is_none()) {
                list.draw_text(x + 130, y + 326, Argb(0xffdcdcdc), &self.labels[1], font);
            }
        }
    }
}

pub fn row_level_offset(label_width: i32, text: &str) -> i32 {
    let units: Vec<u16> = text.encode_utf16().collect();
    if units.len() < 16 {
        return label_width;
    }
    let unit = |index: usize| i32::from(units.get(index).copied().unwrap_or(0));
    let low = unit(16);
    let high = match units.len() {
        16 => label_width >> 16,
        17 => 0,
        _ => unit(17),
    };
    (high << 16) | low
}

fn icon(list: &mut DrawList, x: i32, y: i32, texture: &str, alpha: f32) {
    if texture.is_empty() { return; }
    list.draw_texture(x - 1, y - 1, 34, 34, 0, 0, 34, 34, OUTLINE_DOWN, alpha);
    list.draw_texture(x - 1, y - 1, 35, 35, 0, 0, 35, 35, OUTLINE, alpha);
    list.draw_texture(x, y, 32, 32, 0, 0, 32, 32, texture, alpha);
}

fn value(list: &mut DrawList, (x, y): (i32, i32), label: &str, amount: i32, font: &BitmapFont) {
    list.draw_text(x, y, Argb(0xffa3a3a3), label, font);
    list.draw_text(x + font.measure(label) as i32, y, Argb(0xffdcdcdc), " : ", font);
    list.draw_text(x + font.measure(&format!("{label} : ")) as i32, y, Argb(0xffb09b79), &amount.to_string(), font);
}

fn wrapped(list: &mut DrawList, (x, y): (i32, i32), width: i32, text: &str, color: Argb, font: &BitmapFont) {
    let mut yy = y;
    for line in text.lines() {
        for (_, row) in wrap_line(line.trim_end_matches('\r'), width, 0, font) {
            list.draw_text(x, yy, color, &row, font);
            yy += font.line_height as i32;
        }
    }
}

pub fn grip() -> Rect {
    let contract = frame_controls::FRAME_7C;
    contract.grip(&FrameCtrl::new(contract.mode), (0, 0))
}

const EXP_THRESHOLDS: [i32; 79] = [0, 0, 68, 363, 1168, 2884, 6038, 11287, 19423, 31378, 48229, 71202, 101677, 141193, 191454, 254330, 331867, 426288, 540000, 675596, 835862, 1023784, 1242546, 1495543, 1786379, 2118876, 2497077, 2925250, 3407897, 3949754, 4555796, 5231246, 5981576, 6812513, 7730044, 8740422, 9850166, 11066072, 12395215, 13844951, 15422929, 17137087, 18995665, 21007203, 23180550, 25524868, 28049635, 30764654, 33680052, 36806289, 40154162, 45525133, 51262490, 57383988, 63907911, 70853089, 80700831, 91162654, 102265881, 114038596, 126509653, 146308200, 167244337, 189364894, 212717908, 237352644, 271975263, 308443198, 346827154, 387199547, 429634523, 474207979, 532694979, 606322775, 696381369, 804225364, 931275828, 1151275834, 1511275834];

pub fn enchant_available_exp(level: i32, exp: i32) -> Option<i32> {
    (1..=78).contains(&level).then(|| exp.wrapping_sub(EXP_THRESHOLDS[level as usize]))
}

#[cfg(test)]
mod generated_control_tests {
    use super::*;
    use crate::font::{Glyphs, gly};

    #[test]
    fn on_create_1011e060_generated_buttons_preserve_public_state_paint_and_hits() {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        let font = BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap());
        let mut view = TrainView::new(|id| Some(id.to_string()));
        assert_eq!(OK_RECT, Rect::new(51, 372, 76, 23));
        assert_eq!(CANCEL_RECT, Rect::new(131, 372, 76, 23));
        for (contract, button, rect, id, label) in [
            (&controls::BUTTON_5C0, &view.learn, OK_RECT, 2, "368"),
            (&controls::BUTTON_5C4, &view.cancel, CANCEL_RECT, 3, "369"),
        ] {
            let legacy = TextButton::new(rect, "L2UI_CH3.Button.Btn1_Normal", "L2UI_CH3.Button.Btn1_NormalOn", label);
            assert_eq!(button, &legacy);
            for enabled in [false, true] {
                for pressed in [false, true] {
                    for moved in [false, true] {
                        let mut button = button.clone();
                        button.enabled = enabled;
                        if moved {
                            button.rect = Rect::new(-20, 7, 93, 29);
                            button.label = "changed".into();
                            button.normal = "normal_changed";
                            button.down = "down_changed";
                        }
                        let mut expected = DrawList::default();
                        let mut actual = DrawList::default();
                        button.paint(&mut expected, 11, -3, pressed, &font);
                        contract.paint(&button, &mut actual, (11, -3), pressed, &font);
                        assert_eq!(actual.quads, expected.quads);
                        for x in button.rect.x - 1..=button.rect.x + button.rect.w + 1 {
                            for y in button.rect.y - 1..=button.rect.y + button.rect.h + 1 {
                                assert_eq!(contract.hit(&button, x, y), button.contains(x, y).then_some(id));
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(view.hit(52, 373), None);
        view.info = Some(TrainInfoView::default());
        assert_eq!(view.hit(52, 373), Some(TrainHit::Learn));
        assert_eq!(view.hit(132, 373), Some(TrainHit::Cancel));
        view.learn.rect = Rect::new(1, 21, 5, 5);
        assert_eq!(view.hit(52, 373), None);
        assert_eq!(view.hit(2, 22), Some(TrainHit::Learn));
    }

    #[test]
    fn list_row_1011c640_unacquired_text_of_sixteen_characters_zeroes_the_measured_width_of_lv() {
        assert_eq!(row_level_offset(14, "3"), 14);
        assert_eq!(row_level_offset(14, "Unacquired stat"), 14);
        assert_eq!(row_level_offset(14, "Unacquired state"), 0);
        assert_eq!(row_level_offset(14, "Unacquired states"), i32::from(b's'));
        assert_eq!(row_level_offset(14, "Unacquired stateXY"), i32::from(b'X') | (i32::from(b'Y') << 16));
    }
}
