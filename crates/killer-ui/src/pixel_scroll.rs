use crate::draw::DrawList;
use crate::scroll::{ScrollHit, ScrollWnd, scroll_rects};

pub const LINE_STEP: i32 = 15;
pub const WHEEL_DELTA: i32 = 120;
pub const WHEEL_LINES: i32 = 3;
pub const PAGE_STEP: i32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollKey {
    PageUp,
    PageDown,
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PixelScroll {
    pub size: (i32, i32),
    pub offset: i32,
    pub minimum: i32,
    pub bar: ScrollWnd,
}

impl PixelScroll {
    pub fn new(size: (i32, i32)) -> PixelScroll {
        PixelScroll { size, offset: 0, minimum: 0, bar: ScrollWnd::new(size.0, size.1) }
    }

    pub fn set_content_height(&mut self, height: i32) {
        if height > self.size.1 {
            self.minimum = self.size.1 - height;
            self.bar.slider.set_range(height, self.size.1);
        } else {
            self.minimum = 0;
            self.bar.slider.set_range(0, self.size.1);
        }
        self.bar.slider.set_pos(-self.offset);
    }

    pub fn up(&mut self) {
        if self.offset < 0 {
            self.offset = (self.offset + LINE_STEP).min(0);
            self.bar.slider.set_pos(-self.offset);
        }
    }

    pub fn down(&mut self) {
        if self.offset > self.minimum {
            self.offset = (self.offset - LINE_STEP).max(self.minimum);
            self.bar.slider.set_pos(-self.offset);
        }
    }

    pub fn key(&mut self, key: ScrollKey) -> bool {
        let old = self.offset;
        match key {
            ScrollKey::PageUp | ScrollKey::Up => {
                if self.offset < 0 {
                    let step = if key == ScrollKey::PageUp { PAGE_STEP } else { LINE_STEP };
                    self.offset = (self.offset + step).min(0);
                }
            }
            ScrollKey::PageDown | ScrollKey::Down => {
                if self.offset > self.minimum {
                    let step = if key == ScrollKey::PageDown { PAGE_STEP } else { LINE_STEP };
                    self.offset = (self.offset - step).max(self.minimum);
                }
            }
        }
        if self.offset != old {
            self.bar.slider.set_pos(-self.offset);
        }
        true
    }

    pub fn wheel(&mut self, delta: i32) {
        let notches = delta / WHEEL_DELTA;
        if notches < 1 {
            if self.minimum < self.offset {
                self.offset = (self.offset + LINE_STEP * notches * WHEEL_LINES).max(self.minimum);
                self.bar.slider.set_pos(-self.offset);
            }
        } else if self.offset < 0 {
            self.offset = (self.offset + LINE_STEP * notches * WHEEL_LINES).min(0);
            self.bar.slider.set_pos(-self.offset);
        }
    }

    pub fn set_slider(&mut self, position: i32) {
        self.offset = -((position + LINE_STEP - 1) / LINE_STEP * LINE_STEP);
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<ScrollHit> {
        self.bar.hit(self.size.0, self.size.1, x, y)
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> bool {
        match self.hit(x, y) {
            Some(ScrollHit::Up) => {
                self.bar.up_pressed = true;
                self.up();
                true
            }
            Some(ScrollHit::Down) => {
                self.bar.down_pressed = true;
                self.down();
                true
            }
            Some(ScrollHit::Slider(position)) => {
                self.bar.slider.mouse_down(position);
                true
            }
            None => false,
        }
    }

    pub fn mouse_move(&mut self, y: i32) -> bool {
        if !self.bar.slider.dragging {
            return false;
        }
        if let Some(position) = self.bar.slider.mouse_move(y - scroll_rects(self.size.0, self.size.1).slider.y) {
            self.set_slider(position);
        }
        true
    }

    pub fn mouse_up(&mut self) {
        self.bar.up_pressed = false;
        self.bar.down_pressed = false;
        self.bar.slider.mouse_up();
    }

    pub fn paint_bar(&self, list: &mut DrawList, x: i32, y: i32) {
        self.bar.paint(list, x, y, self.size.0, self.size.1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scroll() -> PixelScroll {
        let mut scroll = PixelScroll::new((200, 100));
        scroll.set_content_height(5 + 36 * 6);
        scroll
    }

    #[test]
    fn content_shorter_than_the_window_keeps_the_minimum_at_zero_and_hides_the_thumb() {
        let mut scroll = PixelScroll::new((200, 100));
        scroll.set_content_height(5 + 36 * 2);
        assert_eq!((scroll.minimum, scroll.bar.slider.shown), (0, false));
        scroll.down();
        assert_eq!(scroll.offset, 0);
    }

    #[test]
    fn buttons_move_fifteen_pixels_and_the_wheel_forty_five_per_notch_inside_the_limits() {
        let mut scroll = scroll();
        assert_eq!(scroll.minimum, 100 - 221);
        scroll.down();
        assert_eq!(scroll.offset, -15);
        scroll.wheel(-120);
        assert_eq!(scroll.offset, -60);
        scroll.wheel(-120 * 10);
        assert_eq!(scroll.offset, scroll.minimum);
        scroll.up();
        assert_eq!(scroll.offset, scroll.minimum + 15);
        scroll.wheel(120 * 10);
        assert_eq!(scroll.offset, 0);
        assert_eq!(scroll.bar.slider.thumb_y, 0);
    }

    #[test]
    fn keys_move_a_line_or_a_hundred_pixel_page_and_stop_at_the_limits() {
        let mut scroll = scroll();
        assert!(scroll.key(ScrollKey::Up));
        assert_eq!(scroll.offset, 0);
        assert!(scroll.key(ScrollKey::Down));
        assert_eq!(scroll.offset, -15);
        scroll.key(ScrollKey::PageDown);
        assert_eq!(scroll.offset, -115);
        assert_eq!(scroll.bar.slider.pos, 115);
        scroll.key(ScrollKey::PageDown);
        assert_eq!(scroll.offset, scroll.minimum);
        scroll.key(ScrollKey::PageUp);
        assert_eq!(scroll.offset, scroll.minimum + 100);
        scroll.key(ScrollKey::PageUp);
        scroll.key(ScrollKey::PageUp);
        assert_eq!(scroll.offset, 0);
    }

    #[test]
    fn the_slider_rounds_the_position_up_to_whole_lines() {
        let mut scroll = scroll();
        scroll.set_slider(1);
        assert_eq!(scroll.offset, -15);
        scroll.set_slider(30);
        assert_eq!(scroll.offset, -30);
        scroll.set_slider(31);
        assert_eq!(scroll.offset, -45);
    }

    #[test]
    fn dragging_the_thumb_follows_the_track() {
        let mut scroll = scroll();
        let slider = scroll_rects(200, 100).slider;
        assert!(scroll.mouse_down(slider.x, slider.y + 2));
        assert!(scroll.mouse_move(slider.y + 2 + 20));
        assert!(scroll.offset < 0);
        scroll.mouse_up();
        assert!(!scroll.mouse_move(slider.y + 50));
    }
}
