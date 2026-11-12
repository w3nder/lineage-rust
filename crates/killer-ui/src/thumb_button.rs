use crate::radio_button::{MoveEvents, RadioButton};
use crate::window::Rect;

pub const MOVABLE: u32 = 0x40_0000;
pub const MSG_DRAG: i32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThumbEffect {
    ParentMouseMove,
    ParentLButtonDown,
    ParentLButtonUp,
    Drag { dx: i32, dy: i32 },
    Capture,
    Release,
    OwnerBegin { index: i32, flag: bool },
    OwnerEnd { index: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ThumbButton {
    pub radio: RadioButton,
    pub remembered: Rect,
    pub index: i32,
    pub flags: u32,
    pub dragging: bool,
    pub captured: bool,
    pub last: (i32, i32),
}

impl ThumbButton {
    pub fn set_pos_size(&mut self, rect: Rect) {
        self.radio.rect = rect;
        self.remembered = rect;
    }

    pub fn on_mouse_move(&mut self, x: i32, y: i32, over: bool) -> (MoveEvents, Vec<ThumbEffect>) {
        let mut effects = Vec::new();
        if !self.captured {
            effects.push(ThumbEffect::ParentMouseMove);
        } else if self.dragging {
            effects.push(ThumbEffect::Drag { dx: x - self.last.0, dy: y - self.last.1 });
            self.last = (x, y);
        }
        (self.radio.on_mouse_move(x, y, over), effects)
    }

    pub fn on_lbutton_down(&mut self, x: i32, y: i32) -> (bool, Vec<ThumbEffect>) {
        self.dragging = true;
        let mut effects = Vec::new();
        if self.flags & MOVABLE == 0 {
            effects.push(ThumbEffect::ParentLButtonDown);
        } else {
            effects.push(ThumbEffect::Capture);
            self.captured = true;
            self.last = (x, y);
        }
        let consumed = self.radio.on_lbutton_down();
        effects.push(ThumbEffect::OwnerBegin { index: self.index, flag: true });
        (consumed, effects)
    }

    pub fn on_lbutton_up(&mut self) -> (bool, Vec<ThumbEffect>) {
        self.dragging = false;
        let mut effects = Vec::new();
        if self.flags & MOVABLE == 0 {
            effects.push(ThumbEffect::ParentLButtonUp);
        } else {
            effects.push(ThumbEffect::Release);
            self.captured = false;
        }
        effects.push(ThumbEffect::OwnerEnd { index: self.index });
        (true, effects)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumb_button_10002680_remembers_position_and_size_after_moving() {
        let mut thumb = ThumbButton::default();
        thumb.set_pos_size(Rect::new(5, 6, 7, 8));
        assert_eq!(thumb.radio.rect, Rect::new(5, 6, 7, 8));
        assert_eq!(thumb.remembered, Rect::new(5, 6, 7, 8));
    }

    #[test]
    fn thumb_button_10002a80_forwards_to_the_parent_unless_movable_and_tells_the_owner() {
        let mut thumb = ThumbButton { index: 4, ..Default::default() };
        let (consumed, effects) = thumb.on_lbutton_down(3, 4);
        assert!(consumed && thumb.dragging && !thumb.captured);
        assert_eq!(effects, vec![ThumbEffect::ParentLButtonDown, ThumbEffect::OwnerBegin { index: 4, flag: true }]);
        thumb.flags = MOVABLE;
        let (_, effects) = thumb.on_lbutton_down(3, 4);
        assert_eq!(effects[0], ThumbEffect::Capture);
        assert_eq!(thumb.last, (3, 4));
    }

    #[test]
    fn thumb_button_10002b30_releases_and_ends_with_the_owner_index() {
        let mut thumb = ThumbButton { index: 4, flags: MOVABLE, dragging: true, captured: true, ..Default::default() };
        let (consumed, effects) = thumb.on_lbutton_up();
        assert!(consumed && !thumb.dragging && !thumb.captured);
        assert_eq!(effects, vec![ThumbEffect::Release, ThumbEffect::OwnerEnd { index: 4 }]);
    }

    #[test]
    fn thumb_button_10002050_drags_by_the_delta_while_captured_and_runs_the_radio_hover() {
        let mut thumb = ThumbButton::default();
        thumb.radio.enabled = true;
        thumb.set_pos_size(Rect::new(0, 0, 10, 10));
        let (events, effects) = thumb.on_mouse_move(5, 5, true);
        assert!(events.enter);
        assert_eq!(effects, vec![ThumbEffect::ParentMouseMove]);
        thumb.captured = true;
        thumb.dragging = true;
        thumb.last = (2, 2);
        let (_, effects) = thumb.on_mouse_move(9, 4, true);
        assert_eq!(effects, vec![ThumbEffect::Drag { dx: 7, dy: 2 }]);
        assert_eq!(thumb.last, (9, 4));
    }
}
