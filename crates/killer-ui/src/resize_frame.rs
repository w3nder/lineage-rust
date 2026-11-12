use crate::ex_button::{NOTIFY_MESSAGE, Notify};
use crate::window::Rect;

pub const MODE_HORIZONTAL: i32 = 1;
pub const MODE_VERTICAL: i32 = 2;
pub const CURSOR_VERTICAL: i32 = 5;
pub const CURSOR_HORIZONTAL: i32 = 6;
pub const BUTTON_LEFT: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorAction {
    Set(i32),
    Lock,
    Release,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResizeEffects {
    pub cursor: Vec<CursorAction>,
    pub notify: Option<Notify>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResizeFrame {
    pub id: i32,
    pub rect: Rect,
    pub mode: i32,
    pub last: (i32, i32),
    pub grab: (i32, i32),
    pub pressed: bool,
    pub dragging: bool,
    pub hover: bool,
}

impl ResizeFrame {
    pub fn new(id: i32, rect: Rect, mode: i32) -> ResizeFrame {
        ResizeFrame { id, rect, mode, ..Default::default() }
    }

    fn inside(&self, x: i32, y: i32) -> bool {
        self.rect.x <= x && x as f32 <= self.rect.x as f32 + self.rect.w as f32 && self.rect.y <= y && y as f32 <= self.rect.y as f32 + self.rect.h as f32
    }

    pub fn on_lbutton_down(&mut self, x: i32, y: i32) {
        self.last = (x, y);
        if self.inside(x, y) {
            self.grab = (x - self.rect.x, y - self.rect.y);
            self.dragging = true;
            self.pressed = true;
        }
    }

    pub fn on_lbutton_up(&mut self) {
        self.pressed = false;
        self.dragging = false;
        self.grab = (0, 0);
    }

    fn drag(&mut self, x: i32, y: i32, effects: &mut ResizeEffects) {
        let relative = (x - self.grab.0 - self.rect.x, y - self.grab.1 - self.rect.y);
        let param = match self.mode {
            MODE_VERTICAL => Some(relative.1),
            MODE_HORIZONTAL => Some(relative.0),
            _ => None,
        };
        effects.notify = param.map(|param| Notify { message: NOTIFY_MESSAGE, id: self.id, param });
        self.last = (x, y);
    }

    pub fn on_mouse_move(&mut self, x: i32, y: i32, buttons: u32) -> ResizeEffects {
        let mut effects = ResizeEffects::default();
        let left = buttons & BUTTON_LEFT != 0;
        if self.inside(x, y) {
            match self.mode {
                MODE_VERTICAL => effects.cursor.push(CursorAction::Set(CURSOR_VERTICAL)),
                MODE_HORIZONTAL => effects.cursor.push(CursorAction::Set(CURSOR_HORIZONTAL)),
                _ => {}
            }
            effects.cursor.push(CursorAction::Lock);
            self.hover = true;
            if left && self.dragging {
                self.drag(x, y, &mut effects);
            }
            return effects;
        }
        if self.dragging && left {
            self.drag(x, y, &mut effects);
            return effects;
        }
        if self.hover {
            self.hover = false;
            effects.cursor.push(CursorAction::Release);
        }
        effects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(mode: i32) -> ResizeFrame {
        ResizeFrame::new(0xf7, Rect::new(10, 10, 100, 6), mode)
    }

    #[test]
    fn resize_frame_100119b0_grabs_only_inside_and_10011a50_clears_the_drag() {
        let mut f = frame(MODE_VERTICAL);
        f.on_lbutton_down(500, 500);
        assert!(!f.dragging && f.last == (500, 500));
        f.on_lbutton_down(30, 12);
        assert!(f.dragging && f.pressed);
        assert_eq!(f.grab, (20, 2));
        f.on_lbutton_up();
        assert!(!f.dragging && !f.pressed);
        assert_eq!(f.grab, (0, 0));
    }

    #[test]
    fn resize_frame_10012200_sets_the_cursor_for_the_mode_while_the_pointer_is_inside() {
        let mut f = frame(MODE_VERTICAL);
        let effects = f.on_mouse_move(30, 12, 0);
        assert_eq!(effects.cursor, vec![CursorAction::Set(5), CursorAction::Lock]);
        assert!(f.hover);
        let mut g = frame(MODE_HORIZONTAL);
        assert_eq!(g.on_mouse_move(30, 12, 0).cursor[0], CursorAction::Set(6));
        let mut h = frame(0);
        assert_eq!(h.on_mouse_move(30, 12, 0).cursor, vec![CursorAction::Lock]);
    }

    #[test]
    fn resize_frame_10012200_notifies_the_parent_with_y_in_mode_two_and_x_in_mode_one() {
        let mut f = frame(MODE_VERTICAL);
        f.on_lbutton_down(30, 12);
        let effects = f.on_mouse_move(40, 52, 1);
        assert_eq!(effects.notify, Some(Notify { message: 0x501, id: 0xf7, param: 52 - 2 - 10 }));
        assert_eq!(f.last, (40, 52));
        let mut g = frame(MODE_HORIZONTAL);
        g.on_lbutton_down(30, 12);
        assert_eq!(g.on_mouse_move(45, 12, 1).notify.map(|n| n.param), Some(45 - 20 - 10));
    }

    #[test]
    fn resize_frame_10012200_keeps_dragging_outside_and_releases_the_cursor_after_leaving() {
        let mut f = frame(MODE_VERTICAL);
        f.on_lbutton_down(30, 12);
        let effects = f.on_mouse_move(300, 300, 1);
        assert!(effects.notify.is_some());
        f.on_lbutton_up();
        f.hover = true;
        let effects = f.on_mouse_move(300, 300, 0);
        assert_eq!(effects.cursor, vec![CursorAction::Release]);
        assert!(!f.hover);
        assert!(f.on_mouse_move(300, 300, 0).cursor.is_empty());
    }
}
