#[path = "petition_feedback_controls_generated.rs"]
mod controls;
#[path = "petition_feedback_layout_generated.rs"]
mod layout;

use crate::button::{CHECK_ART, CHECK_LABEL_GAP, TextButton};
use crate::color::{Argb, CHECK_TEXT};
use crate::draw::DrawList;
use crate::edit_box::FontMetrics;
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::frame_contract::FrameHit;
use crate::multi_edit::MultiEdit;
use crate::window::{Rect, WindowId};

pub const ID: WindowId = WindowId::PETITION_FEEDBACK;
pub const SIZE: (i32, i32) = layout::SIZE;
pub const BACK: &str = "L2UI_CH3.PetitionWnd.petition_fb_back";
pub const BACK_ORIGIN: (i32, i32) = (0, 20);
pub const MULTI_EDIT_ID: u32 = 0xff00a8;
pub const RADIOS: usize = 5;
pub const DEFAULT_RADIO: usize = 2;
pub const RADIO_SYSSTRINGS: [u32; RADIOS] = [850, 851, 852, 853, 854];
pub const TEXT_SYSSTRINGS: [u32; 2] = [855, 856];
pub const TEXT_POSITIONS: [(i32, i32); 2] = [(22, 32), (22, 151)];
pub const TEXT_COLOR: Argb = Argb(0xFFDCDCDC);
pub const VOTE_FIRST: i32 = 1;

pub fn position(viewport: (i32, i32)) -> (i32, i32) {
    (viewport.0 - SIZE.0, viewport.1 - SIZE.1)
}

pub fn grip() -> Rect {
    layout::RECT_7C
}

pub fn edit_rect() -> Rect {
    layout::RECT_F0
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeedbackAction {
    Close,
    Vote { first: i32, index: i32, text: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedbackHit {
    Close,
    Radio(usize),
    Ok,
    Edit,
}

#[derive(Clone, Debug)]
pub struct PetitionFeedback {
    pub frame: FrameCtrl,
    pub radios: Vec<crate::button::CheckBox>,
    pub edit: MultiEdit,
    pub ok: TextButton,
    pub texts: [String; 2],
    pressed: Option<FeedbackHit>,
}

impl PetitionFeedback {
    pub fn new(lookup: impl Fn(u32) -> Option<String>, line_height: i32) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        let contracts = [&controls::CHILD_F4, &controls::CHILD_F8, &controls::CHILD_FC, &controls::CHILD_100, &controls::CHILD_104];
        let radios = contracts
            .iter()
            .zip(RADIO_SYSSTRINGS)
            .map(|(contract, sysstring)| crate::button::CheckBox::new(contract.rect, &text(sysstring)))
            .collect();
        let mut view = Self {
            frame: controls::FRAME_7C.build(text),
            radios,
            edit: MultiEdit::new(MULTI_EDIT_ID, layout::RECT_F0, line_height),
            ok: controls::BUTTON_EC.build((0, 0), text),
            texts: TEXT_SYSSTRINGS.map(text),
            pressed: None,
        };
        view.select(DEFAULT_RADIO);
        view
    }

    pub fn select(&mut self, index: usize) {
        for (position, radio) in self.radios.iter_mut().enumerate() {
            radio.checked = position == index;
        }
    }

    pub fn selected(&self) -> Vec<usize> {
        (0..RADIOS).filter(|&index| self.radios[index].checked).collect()
    }

    pub fn hide(&mut self) {
        self.edit.clear();
        self.edit.kill_focus();
        self.select(DEFAULT_RADIO);
        self.pressed = None;
    }

    pub fn ok(&mut self) -> FeedbackAction {
        let index = self.selected().first().copied().unwrap_or(0) as i32;
        let text = self.edit.get_text().unwrap_or_default();
        self.hide();
        FeedbackAction::Vote { first: VOTE_FIRST, index, text }
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<FeedbackHit> {
        if controls::FRAME_7C.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close) {
            return Some(FeedbackHit::Close);
        }
        if controls::BUTTON_EC.hit(&self.ok, x, y).is_some() {
            return Some(FeedbackHit::Ok);
        }
        if let Some(index) = self.radios.iter().position(|radio| radio.contains(x, y)) {
            return Some(FeedbackHit::Radio(index));
        }
        layout::RECT_F0.contains(x, y).then_some(FeedbackHit::Edit)
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> Option<FeedbackHit> {
        let hit = self.hit(x, y);
        self.pressed = hit;
        match hit {
            Some(FeedbackHit::Radio(index)) => self.select(index),
            Some(FeedbackHit::Edit) => {
                self.edit.focus();
                self.edit.mouse_down(x, y);
            }
            _ => {}
        }
        hit
    }

    pub fn mouse_move(&mut self, y: i32) {
        self.edit.mouse_move(y);
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<FeedbackAction> {
        self.edit.mouse_up();
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        match pressed {
            FeedbackHit::Close => {
                self.hide();
                Some(FeedbackAction::Close)
            }
            FeedbackHit::Ok => Some(self.ok()),
            _ => None,
        }
    }

    pub fn wheel(&mut self, x: i32, y: i32, delta: i32) -> bool {
        if layout::RECT_F0.contains(x, y) {
            self.edit.wheel(delta);
            true
        } else {
            false
        }
    }

    pub fn key(&mut self, key: crate::chat::InputKey, font: &BitmapFont) -> bool {
        if !self.edit.focused {
            return false;
        }
        self.edit.input_key(key, &FontMetrics(font));
        true
    }

    pub fn tick(&mut self, elapsed_ms: u32) {
        self.edit.tick(elapsed_ms);
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) {
        let (x, y) = origin;
        controls::FRAME_7C.paint(&self.frame, list, origin, font, alpha);
        let start = list.quads.len();
        list.draw_texture(x + BACK_ORIGIN.0, y + BACK_ORIGIN.1, SIZE.0, SIZE.1 - BACK_ORIGIN.1, 0, 0, SIZE.0, SIZE.1 - BACK_ORIGIN.1, BACK, alpha);
        list.clip_from(start, Rect::new(x, y, SIZE.0, SIZE.1));
        for (position, text) in TEXT_POSITIONS.iter().zip(&self.texts) {
            list.draw_text(x + position.0, y + position.1, TEXT_COLOR, text, font);
        }
        let normal = controls::CHILD_F4.texture_str(0x274);
        let checked = controls::CHILD_F4.texture_str(0x278);
        for radio in &self.radios {
            let texture = if radio.checked { checked } else { normal };
            list.draw_texture(x + radio.rect.x, y + radio.rect.y, CHECK_ART, CHECK_ART, 0, 0, CHECK_ART, CHECK_ART, texture, 1.0);
            list.draw_text(x + radio.rect.x + CHECK_ART + CHECK_LABEL_GAP, y + radio.rect.y, CHECK_TEXT, &radio.label, font);
        }
        self.edit.paint(list, origin, &FontMetrics(font), font);
        controls::BUTTON_EC.paint(&self.ok, list, origin, self.pressed == Some(FeedbackHit::Ok), font);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixed;

    impl crate::edit_box::Metrics for Fixed {
        fn text_width(&self, text: &[u16]) -> i32 {
            text.len() as i32 * 7
        }

        fn char_width(&self, _: u16) -> i32 {
            7
        }
    }

    fn view() -> PetitionFeedback {
        PetitionFeedback::new(|id| Some(format!("s{id}")), 13)
    }

    #[test]
    fn the_layout_matches_the_on_create_snapshot() {
        let view = view();
        assert_eq!(SIZE, (256, 401));
        assert_eq!(grip(), Rect::new(0, 0, 256, 21));
        assert_eq!(edit_rect(), Rect::new(8, 166, 240, 199));
        assert_eq!(view.radios[0].rect, Rect::new(21, 52, 100, 12));
        assert_eq!(view.radios[4].rect, Rect::new(21, 128, 100, 12));
        assert_eq!(view.radios[0].label, "s850");
        assert_eq!(view.ok.rect, Rect::new(90, 372, 76, 23));
        assert_eq!(view.ok.label, "s140");
        assert_eq!(view.texts, ["s855".to_string(), "s856".to_string()]);
        assert_eq!(controls::CHILD_F4.texture_str(0x274), "L2UI_CH3.RadioCtrl.RadioButton2");
        assert_eq!(controls::CHILD_F4.texture_str(0x278), "L2UI_CH3.RadioCtrl.RadioButton1");
        assert_eq!(position((1280, 800)), (1024, 399));
    }

    #[test]
    fn the_default_radio_is_the_third_and_picking_one_unmarks_the_others() {
        let mut view = view();
        assert_eq!(view.selected(), vec![2]);
        let rect = view.radios[4].rect;
        view.mouse_down(rect.x + 2, rect.y + 2);
        assert_eq!(view.selected(), vec![4]);
        view.mouse_down(rect.x + 2, rect.y + 2);
        assert_eq!(view.selected(), vec![4]);
    }

    #[test]
    fn ok_votes_with_the_marked_index_and_the_text_then_restores_the_defaults() {
        let mut view = view();
        view.select(0);
        view.edit.set_text("thanks", &Fixed);
        assert_eq!(view.ok(), FeedbackAction::Vote { first: 1, index: 0, text: "thanks".into() });
        assert_eq!(view.selected(), vec![2]);
        assert_eq!(view.edit.get_text(), None);
    }

    #[test]
    fn ok_without_text_votes_with_an_empty_string() {
        let mut view = view();
        assert_eq!(view.ok(), FeedbackAction::Vote { first: 1, index: 2, text: String::new() });
    }

    #[test]
    fn the_ok_button_acts_on_release_inside_the_button() {
        let mut view = view();
        let rect = view.ok.rect;
        view.mouse_down(rect.x + 2, rect.y + 2);
        assert_eq!(view.mouse_up(rect.x + 2, rect.y + 2), Some(FeedbackAction::Vote { first: 1, index: 2, text: String::new() }));
        view.mouse_down(rect.x + 2, rect.y + 2);
        assert_eq!(view.mouse_up(0, 100), None);
    }
}
