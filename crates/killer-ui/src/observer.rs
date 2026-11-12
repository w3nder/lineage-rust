#[path = "observer_controls_generated.rs"]
mod controls;
#[path = "observer_layout_generated.rs"]
mod layout;

use crate::button::TextButton;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::frame_contract::FrameHit;
use crate::window::{Rect, WindowId};

pub const ID: WindowId = WindowId::OBSERVER;
pub const SIZE: (i32, i32) = layout::SIZE;
pub const BACK: &str = "L2UI_ch3.etc.broadcast_back";
pub const BACK_RECT: Rect = Rect::new(12, 0, 102, 32);
pub const INITIAL_POSITION: crate::initial_position::InitialPosition =
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 2, vertical_mode: 3, x: 102, y: 0 };

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    ((viewport.0 as f32 * 0.5 - 51.0) as i32, 0)
}

pub fn grip() -> Rect {
    layout::RECT_7C
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserverAction {
    End,
}

#[derive(Clone, Debug)]
pub struct ObserverView {
    pub frame: FrameCtrl,
    pub button: TextButton,
    pressed: bool,
}

impl ObserverView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        Self { frame: controls::FRAME_7C.build(text), button: controls::BUTTON_EC.build((0, 0), text), pressed: false }
    }

    pub fn hit_button(&self, x: i32, y: i32) -> bool {
        controls::BUTTON_EC.hit(&self.button, x, y).is_some()
    }

    pub fn hit_grip(&self, x: i32, y: i32) -> bool {
        controls::FRAME_7C.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Grip)
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> bool {
        self.pressed = self.hit_button(x, y);
        self.pressed
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<ObserverAction> {
        let pressed = std::mem::take(&mut self.pressed);
        (pressed && self.hit_button(x, y)).then_some(ObserverAction::End)
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) {
        let (x, y) = origin;
        let start = list.quads.len();
        list.draw_texture(x + BACK_RECT.x, y + BACK_RECT.y, BACK_RECT.w, BACK_RECT.h, 0, 0, BACK_RECT.w, BACK_RECT.h, BACK, alpha);
        list.clip_from(start, Rect::new(x, y, SIZE.0, SIZE.1));
        controls::FRAME_7C.paint(&self.frame, list, origin, font, alpha);
        controls::BUTTON_EC.paint(&self.button, list, origin, self.pressed, font);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view() -> ObserverView {
        ObserverView::new(|id| Some(format!("s{id}")))
    }

    #[test]
    fn the_layout_matches_the_on_create_snapshot() {
        let view = view();
        assert_eq!(SIZE, (102, 32));
        assert_eq!(grip(), Rect::new(0, 0, 12, 32));
        assert_eq!(view.button.rect, Rect::new(19, 5, 76, 23));
        assert_eq!(view.button.label, "s611");
        assert_eq!(default_position((1280, 800)), (589, 0));
    }

    #[test]
    fn the_initial_position_restores_the_creation_coordinates() {
        let viewport = (1280, 800);
        let descriptor = INITIAL_POSITION;
        let x = crate::initial_position::InitialAxis::horizontal(descriptor.horizontal_mode, descriptor.x).unwrap().coordinate(viewport.0);
        let y = crate::initial_position::InitialAxis::vertical(descriptor.vertical_mode, descriptor.y).unwrap().coordinate(viewport.1);
        assert_eq!((x, y), default_position(viewport));
    }

    #[test]
    fn releasing_on_the_button_asks_to_end_the_observation() {
        let mut view = view();
        let rect = view.button.rect;
        assert!(view.mouse_down(rect.x + 2, rect.y + 2));
        assert_eq!(view.mouse_up(rect.x + 2, rect.y + 2), Some(ObserverAction::End));
        assert!(view.mouse_down(rect.x + 2, rect.y + 2));
        assert_eq!(view.mouse_up(0, 0), None);
        assert!(!view.mouse_down(0, 0));
    }
}
