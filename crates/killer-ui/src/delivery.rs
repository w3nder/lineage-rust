#[path = "delivery_select_controls_generated.rs"]
mod controls;
#[path = "delivery_select_layout_generated.rs"]
mod layout;

use crate::color::TEXT;
use crate::combo::ComboBox;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::frame_contract::FrameHit;
use crate::window::Rect;

pub const SELECT_SIZE: (i32, i32) = layout::SIZE;
pub const SELECT_BACK: &str = "L2UI_ch3.DeliverWnd.Warehouse_back";
pub const SELECT_MESSAGE: u32 = 664;
pub const COMBO_RECT: Rect = layout::RECT_EC;
pub const OK_RECT: Rect = controls::BUTTON_F0.rect_at((0, 0));
pub const CANCEL_RECT: Rect = controls::BUTTON_F4.rect_at((0, 0));
pub const TARGET_CAPACITY: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectHit { Close, Ok, Cancel, Combo, Title, Body }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectAction { Close, Request(i32) }

#[derive(Debug, Clone)]
pub struct SelectDeliveryView {
    pub title: String,
    pub message: String,
    pub ok: String,
    pub cancel: String,
    pub combo: ComboBox,
    pub targets: Vec<i32>,
    pub pressed: Option<SelectHit>,
}

impl SelectDeliveryView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        Self {
            title: lookup(558).unwrap_or_default(), message: String::new(),
            ok: lookup(140).unwrap_or_default(), cancel: lookup(141).unwrap_or_default(),
            combo: ComboBox::from_contract(&controls::CHILD_EC, Vec::new(), 13), targets: Vec::new(), pressed: None,
        }
    }

    pub fn clear(&mut self) {
        self.targets.clear();
        self.combo.clear_entries();
        self.combo.lose_focus();
        self.pressed = None;
    }

    pub fn open(&mut self, targets: impl IntoIterator<Item = (i32, String)>) {
        self.clear();
        for (id, name) in targets.into_iter().take(TARGET_CAPACITY) {
            self.targets.push(id);
            self.combo.entries.push(name);
        }
    }

    fn frame(&self) -> FrameCtrl { controls::FRAME_7C.build(|_| self.title.clone()) }

    pub fn hit(&self, x: i32, y: i32) -> SelectHit {
        if self.combo.expanded_rect().contains(x, y) { SelectHit::Combo }
        else if controls::FRAME_7C.hit(&self.frame(), (0, 0), (x, y)) == Some(FrameHit::Close) { SelectHit::Close }
        else if controls::BUTTON_F0.hit(&controls::BUTTON_F0.build((0, 0), |_| self.ok.clone()), x, y).is_some() { SelectHit::Ok }
        else if controls::BUTTON_F4.hit(&controls::BUTTON_F4.build((0, 0), |_| self.cancel.clone()), x, y).is_some() { SelectHit::Cancel }
        else if y < crate::frame::TITLE_HEIGHT { SelectHit::Title }
        else { SelectHit::Body }
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> Option<SelectAction> {
        let hit = self.hit(x, y);
        self.pressed = Some(hit);
        if hit == SelectHit::Combo {
            self.combo.mouse_move(x, y);
            self.combo.mouse_down();
        } else {
            self.combo.lose_focus();
        }
        (hit == SelectHit::Close).then_some(SelectAction::Close)
    }

    pub fn mouse_up(&mut self, at: Option<(i32, i32)>) -> Option<SelectAction> {
        let pressed = self.pressed.take()?;
        if at.map(|(x, y)| self.hit(x, y)) != Some(pressed) { return None; }
        match pressed {
            SelectHit::Ok => usize::try_from(self.combo.selected).ok()
                .and_then(|i| self.targets.get(i)).copied().map(SelectAction::Request),
            SelectHit::Cancel => Some(SelectAction::Close),
            _ => None,
        }
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        controls::FRAME_7C.paint(&self.frame(), list, (x, y), font, 1.0);
        list.draw_texture(x, y + 20, 256, 181, 0, 0, 256, 181, SELECT_BACK, 1.0);
        list.draw_text(x + 11, y + 32, TEXT, &self.message, font);
        controls::BUTTON_F0.paint(&controls::BUTTON_F0.build((0, 0), |_| self.ok.clone()), list, (x, y), self.pressed == Some(SelectHit::Ok), font);
        controls::BUTTON_F4.paint(&controls::BUTTON_F4.build((0, 0), |_| self.cancel.clone()), list, (x, y), self.pressed == Some(SelectHit::Cancel), font);
        self.combo.paint(list, (x, y), font, true, self.pressed == Some(SelectHit::Combo));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combo_comes_from_the_constructor_state_of_the_native_combo() {
        let view = SelectDeliveryView::new(|_| None);
        assert_eq!(view.combo.rect, COMBO_RECT);
        assert_eq!(controls::CHILD_EC.kind, crate::child_contract::ChildKind::ComboBox);
        assert_eq!((view.combo.indent, view.combo.row_height), (5, 17));
        assert_eq!((view.combo.selected, view.combo.hovered), (-1, -1));
        assert!(!view.combo.open);
    }

    #[test]
    fn selector_preserves_order_caps_eight_and_requires_explicit_choice() {
        let mut view = SelectDeliveryView::new(|_| None);
        view.open((0..12).map(|i| (100 + i, format!("char{i}"))));
        assert_eq!(view.targets, (100..108).collect::<Vec<_>>());
        assert_eq!(view.combo.selected, -1);
        view.mouse_down(60, 175);
        assert_eq!(view.mouse_up(Some((60, 175))), None);
        view.mouse_down(40, 62);
        view.mouse_up(Some((40, 62)));
        view.mouse_down(40, 96);
        assert_eq!(view.combo.selected, 1);
        view.mouse_down(60, 175);
        assert_eq!(view.mouse_up(Some((60, 175))), Some(SelectAction::Request(101)));
        view.clear();
        assert!(view.targets.is_empty());
        assert_eq!(view.combo.selected, -1);
        assert!(!view.combo.open);
    }

    #[test]
    fn selector_cancel_and_release_outside_do_not_send() {
        let mut view = SelectDeliveryView::new(|_| None);
        view.open([(9, "name".into())]);
        view.combo.selected = 0;
        view.mouse_down(60, 175);
        assert_eq!(view.mouse_up(None), None);
        view.mouse_down(140, 175);
        assert_eq!(view.mouse_up(Some((140, 175))), Some(SelectAction::Close));
        assert_eq!(view.mouse_down(240, 8), Some(SelectAction::Close));
    }
}
