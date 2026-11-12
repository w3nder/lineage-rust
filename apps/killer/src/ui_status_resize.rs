use killer_ui::resize_frame::{CursorAction, MODE_HORIZONTAL, ResizeFrame};
use killer_ui::window::{Rect, Window, WindowId};

pub const STATUS_BASE_WIDTH: i32 = 172;
pub const STATUS_MAXIMUM_SLACK: i32 = 202;
pub const GRIP_WIDTH: i32 = 10;
pub const GRIP_ID: i32 = 0x00f2_0f04;
pub const TARGET_GRIP_ID: i32 = 0x00f2_0f15;
pub const PET_GRIP_ID: i32 = 0x0f0f_00a2;
pub const SUMMONED_GRIP_ID: i32 = 0x00f0_a0b2;
pub const PARTY_GRIP_ID: i32 = 0xff00_01f3_u32 as i32;

pub fn grip_id(id: WindowId) -> Option<i32> {
    match id {
        WindowId::STATUS => Some(GRIP_ID),
        WindowId::TARGET => Some(TARGET_GRIP_ID),
        WindowId::PET_STATUS => Some(PET_GRIP_ID),
        WindowId::SUMMONED_STATUS => Some(SUMMONED_GRIP_ID),
        WindowId::PARTY_STATUS => Some(PARTY_GRIP_ID),
        _ => None,
    }
}

pub fn hover_cursor(window: &Window, x: i32, y: i32) -> Option<i32> {
    let id = grip_id(window.id)?;
    if !window.visible {
        return None;
    }
    let mut frame = ResizeFrame::new(id, grip_rect(window.rect.w, window.rect.h), MODE_HORIZONTAL);
    frame
        .on_mouse_move(x - window.rect.x, y - window.rect.y, 0)
        .cursor
        .into_iter()
        .find_map(|action| if let CursorAction::Set(index) = action { Some(index) } else { None })
}

pub fn status_width_after(width: i32, delta: i32) -> i32 {
    let mut accumulated = width - STATUS_BASE_WIDTH;
    let mut delta = delta;
    if delta > 0 && accumulated < STATUS_MAXIMUM_SLACK {
        accumulated = accumulated.wrapping_add(delta);
        if accumulated > STATUS_MAXIMUM_SLACK {
            delta = delta.wrapping_add(STATUS_MAXIMUM_SLACK - accumulated);
        }
    } else if delta < 0 && accumulated > 0 {
        accumulated = accumulated.wrapping_add(delta);
        if accumulated < 0 {
            delta = delta.wrapping_sub(accumulated);
        }
    } else {
        return width;
    }
    width.wrapping_add(delta)
}

pub fn grip_rect(width: i32, height: i32) -> Rect {
    Rect::new(width - GRIP_WIDTH, 0, GRIP_WIDTH, height)
}

#[derive(Debug, Clone, Default)]
pub struct StatusResize {
    frame: ResizeFrame,
    window: Option<WindowId>,
}

impl StatusResize {
    pub fn window(&self) -> Option<WindowId> {
        self.window.filter(|_| self.frame.dragging)
    }

    pub fn press(&mut self, window: &Window, x: i32, y: i32) -> bool {
        let Some(id) = grip_id(window.id).filter(|_| window.visible) else {
            return false;
        };
        let rect = grip_rect(window.rect.w, window.rect.h);
        self.frame = ResizeFrame::new(id, rect, MODE_HORIZONTAL);
        self.window = Some(window.id);
        self.frame.on_lbutton_down(x - window.rect.x, y - window.rect.y);
        self.frame.dragging
    }

    pub fn drag(&mut self, window: &mut Window, x: i32, y: i32) {
        if !self.frame.dragging || self.window != Some(window.id) {
            return;
        }
        let effects = self.frame.on_mouse_move(x - window.rect.x, y - window.rect.y, killer_ui::resize_frame::BUTTON_LEFT);
        if let Some(notify) = effects.notify {
            window.rect.w = status_width_after(window.rect.w, notify.param);
            self.frame.rect = grip_rect(window.rect.w, window.rect.h);
        }
    }

    pub fn release(&mut self) {
        self.frame.on_lbutton_up();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::window::WindowId;

    fn status() -> Window {
        Window::new(WindowId::STATUS, Rect::new(100, 50, 172, 84), Rect::new(0, 0, 12, 84))
    }

    #[test]
    fn width_follows_slot_12c_of_101100c0_clamped_to_zero_and_202() {
        assert_eq!(status_width_after(172, 30), 202);
        assert_eq!(status_width_after(172, -5), 172);
        assert_eq!(status_width_after(172, 500), 374);
        assert_eq!(status_width_after(374, 1), 374);
        assert_eq!(status_width_after(200, -100), 172);
        assert_eq!(status_width_after(200, 0), 200);
    }

    #[test]
    fn every_window_with_an_ncresizeframe_child_has_its_grip_id_and_the_cursor_6_on_hover() {
        let ids = [
            (WindowId::STATUS, 0xf20f04),
            (WindowId::TARGET, 0xf20f15),
            (WindowId::PET_STATUS, 0xf0f00a2),
            (WindowId::SUMMONED_STATUS, 0xf0a0b2),
            (WindowId::PARTY_STATUS, 0xff0001f3_u32 as i32),
        ];
        for (id, grip) in ids {
            assert_eq!(grip_id(id), Some(grip));
            let window = Window::new(id, Rect::new(100, 50, 172, 46), Rect::new(0, 0, 12, 46));
            assert_eq!(hover_cursor(&window, 100 + 165, 60), Some(6));
            assert_eq!(hover_cursor(&window, 100 + 100, 60), None);
        }
        assert_eq!(grip_id(WindowId::CHAT), None);
    }

    #[test]
    fn dragging_the_target_grip_widens_only_the_target_by_slot_12c_of_1010f600() {
        let mut target = Window::new(WindowId::TARGET, Rect::new(300, 0, 172, 46), Rect::new(0, 0, 12, 46));
        let mut resize = StatusResize::default();
        assert!(resize.press(&target, 300 + 165, 10));
        assert_eq!(resize.window(), Some(WindowId::TARGET));
        let mut status = status();
        resize.drag(&mut status, 400, 10);
        assert_eq!(status.rect.w, 172);
        resize.drag(&mut target, 300 + 265, 10);
        assert_eq!(target.rect.w, 272);
        resize.drag(&mut target, 300 + 900, 10);
        assert_eq!(target.rect.w, 374);
        resize.release();
    }

    #[test]
    fn dragging_the_right_grip_widens_the_window_and_the_grip_follows() {
        let mut window = status();
        let mut resize = StatusResize::default();
        assert!(!resize.press(&window, 100 + 100, 60));
        assert!(resize.press(&window, 100 + 165, 60));
        resize.drag(&mut window, 100 + 185, 60);
        assert_eq!(window.rect.w, 192);
        resize.drag(&mut window, 100 + 205, 60);
        assert_eq!(window.rect.w, 212);
        resize.drag(&mut window, 100 + 195, 60);
        assert_eq!(window.rect.w, 202);
        resize.release();
        assert!(resize.window().is_none());
        resize.drag(&mut window, 100 + 400, 60);
        assert_eq!(window.rect.w, 202);
    }
}
