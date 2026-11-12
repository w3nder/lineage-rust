#[path = "gm_fence_controls_generated.rs"]
mod controls;
#[path = "gm_fence_layout_generated.rs"]
mod layout;

use crate::button::TextButton;
use crate::chat::InputKey;
use crate::color::Argb;
use crate::draw::DrawList;
use crate::edit_box::{EditBox, FontMetrics};
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::frame_contract::FrameHit;
use crate::window::{Rect, WindowId};

pub const ID: WindowId = WindowId::GM_FENCE;
pub const SIZE: (i32, i32) = layout::SIZE;
pub const BACK: &str = "L2UI_CH3.Minimap.MapBack";
pub const BACK_ORIGIN: (i32, i32) = (0, 20);
pub const BACK_SOURCE: (i32, i32) = (0x100, 0x6c);
pub const TEXT_COLOR: Argb = Argb(0xFFDCDCDC);
pub const TEXTS: [(u32, (i32, i32)); 3] = [(1065, (25, 35)), (1066, (25, 70)), (1067, (150, 70))];
pub const RANGE_ERROR_SYSMSG: u32 = 1414;
pub const MINIMUM: i32 = 100;
pub const MAXIMUM: i32 = 5000;

pub fn initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    let (x, _) = default_position(viewport);
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 1, vertical_mode: 2, x: viewport.0.wrapping_sub(x), y: 300 }
}

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    ((viewport.0 as f32 * 0.5 - 250.0) as i32, (viewport.1 as f32 * 0.5 - 150.0) as i32)
}

pub fn grip() -> Rect {
    layout::RECT_7C
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FenceAction {
    Close,
    Create { point: [String; 3], width: String, height: String },
    Notice(u32),
    MyCoordinates,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FenceHit {
    Close,
    Ok,
    Cancel,
    Coordinates,
    Edit(usize),
}

pub fn atoi(text: &str) -> i32 {
    crate::gm_wnd::atoi(text)
}

#[derive(Clone, Debug)]
pub struct GmFenceView {
    pub frame: FrameCtrl,
    pub edits: Vec<EditBox>,
    pub buttons: [TextButton; 3],
    texts: Vec<String>,
    pressed: Option<FenceHit>,
    focus: Option<usize>,
}

impl GmFenceView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        Self {
            frame: controls::FRAME_7C.build(text),
            edits: [&controls::CHILD_204, &controls::CHILD_208, &controls::CHILD_20C, &controls::CHILD_210, &controls::CHILD_214]
                .into_iter()
                .map(EditBox::from_contract)
                .collect(),
            buttons: [
                controls::BUTTON_1F8.build((0, 0), text),
                controls::BUTTON_1FC.build((0, 0), text),
                controls::BUTTON_200.build((0, 0), text),
            ],
            texts: TEXTS.iter().map(|(id, _)| text(*id)).collect(),
            pressed: None,
            focus: None,
        }
    }

    pub fn reset(&mut self) {
        for edit in &mut self.edits {
            edit.kill_focus();
        }
        self.focus = None;
        self.pressed = None;
    }

    pub fn text(&self, index: usize) -> String {
        self.edits[index].get_text().unwrap_or_default()
    }

    pub fn set_coordinates(&mut self, point: [i32; 3], font: &BitmapFont) {
        for (edit, value) in self.edits.iter_mut().zip(point) {
            edit.clear();
            edit.insert_text(&value.to_string(), true, &FontMetrics(font));
        }
    }

    pub fn ok(&self) -> FenceAction {
        let ordinate = atoi(&self.text(3));
        let abscissa = atoi(&self.text(4));
        let range = MINIMUM..MAXIMUM;
        if !range.contains(&ordinate) || !range.contains(&abscissa) {
            return FenceAction::Notice(RANGE_ERROR_SYSMSG);
        }
        FenceAction::Create { point: [self.text(0), self.text(1), self.text(2)], width: self.text(3), height: self.text(4) }
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<FenceHit> {
        if controls::FRAME_7C.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close) {
            return Some(FenceHit::Close);
        }
        if controls::BUTTON_1F8.hit(&self.buttons[0], x, y).is_some() {
            return Some(FenceHit::Ok);
        }
        if controls::BUTTON_1FC.hit(&self.buttons[1], x, y).is_some() {
            return Some(FenceHit::Cancel);
        }
        if controls::BUTTON_200.hit(&self.buttons[2], x, y).is_some() {
            return Some(FenceHit::Coordinates);
        }
        self.edits.iter().position(|edit| edit.rect.contains(x, y)).map(FenceHit::Edit)
    }

    pub fn mouse_down(&mut self, x: i32, y: i32, font: &BitmapFont) -> Option<FenceHit> {
        let hit = self.hit(x, y);
        self.pressed = hit;
        if let Some(FenceHit::Edit(index)) = hit {
            if let Some(previous) = self.focus.replace(index).filter(|previous| *previous != index) {
                self.edits[previous].kill_focus();
            }
            let metrics = FontMetrics(font);
            self.edits[index].focus();
            self.edits[index].mouse_down(x, &metrics);
            self.edits[index].layout(&metrics);
        }
        hit
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<FenceAction> {
        for edit in &mut self.edits {
            edit.mouse_up();
        }
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        match pressed {
            FenceHit::Close | FenceHit::Cancel => Some(FenceAction::Close),
            FenceHit::Ok => Some(self.ok()),
            FenceHit::Coordinates => Some(FenceAction::MyCoordinates),
            FenceHit::Edit(_) => None,
        }
    }

    pub fn key(&mut self, key: InputKey, font: &BitmapFont) -> bool {
        let Some(index) = self.focus else { return false };
        if !self.edits[index].focused {
            return false;
        }
        self.edits[index].input_key(key, &FontMetrics(font));
        self.edits[index].notices.clear();
        true
    }

    pub fn tick(&mut self, elapsed_ms: u32) {
        for edit in &mut self.edits {
            edit.tick(elapsed_ms);
        }
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) {
        let (x, y) = origin;
        controls::FRAME_7C.paint(&self.frame, list, origin, font, alpha);
        let start = list.quads.len();
        list.draw_texture(
            x + BACK_ORIGIN.0,
            y + BACK_ORIGIN.1,
            SIZE.0,
            SIZE.1 - BACK_ORIGIN.1,
            0,
            0,
            BACK_SOURCE.0,
            BACK_SOURCE.1,
            BACK,
            alpha,
        );
        list.clip_from(start, Rect::new(x, y, SIZE.0, SIZE.1));
        for (text, (_, position)) in self.texts.iter().zip(TEXTS.iter()) {
            list.draw_text(x + position.0, y + position.1, TEXT_COLOR, text, font);
        }
        for edit in &self.edits {
            edit.paint(list, origin, &FontMetrics(font), font);
        }
        controls::BUTTON_1F8.paint(&self.buttons[0], list, origin, self.pressed == Some(FenceHit::Ok), font);
        controls::BUTTON_1FC.paint(&self.buttons[1], list, origin, self.pressed == Some(FenceHit::Cancel), font);
        controls::BUTTON_200.paint(&self.buttons[2], list, origin, self.pressed == Some(FenceHit::Coordinates), font);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), crate::font::Glyphs::parse(&crate::font::gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn view() -> GmFenceView {
        GmFenceView::new(|id| Some(format!("s{id}")))
    }

    fn fill(view: &mut GmFenceView, values: [&str; 5], font: &BitmapFont) {
        for (edit, value) in view.edits.iter_mut().zip(values) {
            edit.clear();
            edit.insert_text(value, true, &FontMetrics(font));
        }
    }

    #[test]
    fn the_layout_matches_the_on_create_snapshot() {
        let view = view();
        assert_eq!(SIZE, (500, 170));
        assert_eq!(view.edits[0].rect, Rect::new(115, 32, 80, 16));
        assert_eq!(view.edits[4].rect, Rect::new(180, 68, 80, 16));
        assert_eq!(view.buttons[0].rect, Rect::new(150, 100, 76, 23));
        assert_eq!(view.buttons[0].label, "s140");
        assert_eq!(view.buttons[2].label, "s1064");
        assert_eq!(default_position((1280, 800)), (390, 250));
    }

    #[test]
    fn the_initial_position_restores_the_creation_coordinates() {
        use crate::initial_position::InitialAxis;
        let viewport = (1280, 800);
        let descriptor = initial_position(viewport);
        let x = InitialAxis::horizontal(descriptor.horizontal_mode, descriptor.x).unwrap().coordinate(viewport.0);
        let y = InitialAxis::vertical(descriptor.vertical_mode, descriptor.y).unwrap().coordinate(viewport.1);
        assert_eq!((x, y), default_position(viewport));
    }

    #[test]
    fn ok_needs_both_sizes_in_the_range_100_to_4999_and_returns_the_arguments_in_order() {
        let font = font();
        let mut view = view();
        fill(&mut view, ["1", "2", "3", "99", "200"], &font);
        assert_eq!(view.ok(), FenceAction::Notice(1414));
        fill(&mut view, ["1", "2", "3", "100", "5000"], &font);
        assert_eq!(view.ok(), FenceAction::Notice(1414));
        fill(&mut view, ["-10", "20", "30", "100", "4999"], &font);
        assert_eq!(
            view.ok(),
            FenceAction::Create { point: ["-10".into(), "20".into(), "30".into()], width: "100".into(), height: "4999".into() }
        );
        fill(&mut view, ["007", "x", "", "0100", "4999"], &font);
        assert_eq!(
            view.ok(),
            FenceAction::Create { point: ["007".into(), "x".into(), String::new()], width: "0100".into(), height: "4999".into() }
        );
    }

    #[test]
    fn cancel_and_my_coordinates_buttons_report_their_actions() {
        let font = font();
        let mut view = view();
        let cancel = view.buttons[1].rect;
        view.mouse_down(cancel.x + 2, cancel.y + 2, &font);
        assert_eq!(view.mouse_up(cancel.x + 2, cancel.y + 2), Some(FenceAction::Close));
        let mine = view.buttons[2].rect;
        view.mouse_down(mine.x + 2, mine.y + 2, &font);
        assert_eq!(view.mouse_up(mine.x + 2, mine.y + 2), Some(FenceAction::MyCoordinates));
        view.set_coordinates([7, 8, 9], &font);
        assert_eq!((view.text(0), view.text(1), view.text(2)), ("7".to_string(), "8".to_string(), "9".to_string()));
    }
}
