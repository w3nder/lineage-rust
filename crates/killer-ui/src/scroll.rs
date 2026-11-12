use crate::draw::DrawList;
use crate::window::Rect;

pub const SCROLL_WIDTH: i32 = 15;
pub const BUTTON_SIZE: i32 = 15;
pub const MIN_THUMB: i32 = 15;
pub const THUMB_CAP: i32 = 8;
pub const UP_BTN: &str = "L2UI_CH3.ScrollBar.ScrollBarUpBtn";
pub const UP_ON_BTN: &str = "L2UI_CH3.ScrollBar.ScrollBarUpOnBtn";
pub const DOWN_BTN: &str = "L2UI_CH3.ScrollBar.ScrollBarDownBtn";
pub const DOWN_ON_BTN: &str = "L2UI_CH3.ScrollBar.ScrollBarDownOnBtn";
pub const SLIDER_TOP: &str = "L2UI_CH3.ScrollBar.SliderBarTop";
pub const SLIDER_CENTER: &str = "L2UI_CH3.ScrollBar.SliderBarCenter";
pub const SLIDER_BOTTOM: &str = "L2UI_CH3.ScrollBar.SliderBarBottom";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollHit {
    Up,
    Down,
    Slider(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollRects {
    pub up: Rect,
    pub down: Rect,
    pub slider: Rect,
}

pub fn scroll_rects(w: i32, h: i32) -> ScrollRects {
    let x = w - SCROLL_WIDTH;
    ScrollRects {
        up: Rect::new(x, 0, BUTTON_SIZE, BUTTON_SIZE),
        down: Rect::new(x, h - BUTTON_SIZE, BUTTON_SIZE, BUTTON_SIZE),
        slider: Rect::new(x, BUTTON_SIZE, SCROLL_WIDTH, h - 2 * BUTTON_SIZE),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SliderBar {
    pub height: i32,
    pub total: i32,
    pub visible: i32,
    pub thumb: i32,
    pub thumb_y: i32,
    pub grab: i32,
    pub pos: i32,
    pub track: i32,
    pub dragging: bool,
    pub shown: bool,
}

impl SliderBar {
    pub fn new(height: i32) -> SliderBar {
        SliderBar { height, ..Default::default() }
    }

    pub fn set_range(&mut self, total: i32, visible: i32) {
        self.total = total;
        self.visible = visible;
        if total > visible {
            self.thumb = (f64::from(visible) * f64::from(self.height) / f64::from(total)) as i32;
            self.shown = true;
        } else {
            self.thumb = self.height;
            self.shown = false;
        }
        self.thumb = self.thumb.max(MIN_THUMB);
        self.track = self.height - self.thumb;
    }

    pub fn reset(&mut self) {
        self.thumb_y = 0;
        self.pos = 0;
        self.visible = 0;
        self.thumb = 0;
        self.grab = 0;
        self.dragging = false;
    }

    pub fn set_pos(&mut self, pos: i32) {
        if self.pos > self.total || self.pos < 0 {
            self.pos = 0;
            return;
        }
        self.pos = pos;
        self.thumb_y = if self.total <= self.visible { 0 } else { self.track * pos / (self.total - self.visible) };
    }

    pub fn mouse_down(&mut self, y: i32) -> bool {
        if self.thumb_y <= y && y <= self.thumb_y + self.thumb {
            if self.total > self.visible {
                self.dragging = true;
            }
            self.grab = y - self.thumb_y;
        }
        true
    }

    pub fn mouse_up(&mut self) {
        self.dragging = false;
        self.grab = 0;
    }

    pub fn mouse_move(&mut self, y: i32) -> Option<i32> {
        if !self.dragging {
            return None;
        }
        let delta = y - self.grab - self.thumb_y;
        if delta == 0 {
            return None;
        }
        if self.thumb_y <= 0 {
            if self.grab > y {
                return None;
            }
        } else if self.thumb_y >= self.track && y > self.thumb_y + self.grab {
            return None;
        }
        if delta > 0 {
            if self.thumb_y >= self.track {
                return None;
            }
            self.thumb_y = (self.thumb_y + delta).min(self.track);
        } else {
            if self.thumb_y <= 0 {
                return None;
            }
            self.thumb_y = (self.thumb_y + delta).max(0);
        }
        self.pos = (self.total - self.visible) * self.thumb_y / self.track;
        Some(self.pos)
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32) {
        if !self.shown || self.thumb <= 0 {
            return;
        }
        let ty = y + self.thumb_y;
        list.draw_texture(x, ty, SCROLL_WIDTH, THUMB_CAP, 0, 0, SCROLL_WIDTH, THUMB_CAP, SLIDER_TOP, 1.0);
        list.draw_texture(
            x,
            ty + self.thumb - THUMB_CAP,
            SCROLL_WIDTH,
            THUMB_CAP,
            0,
            0,
            SCROLL_WIDTH,
            THUMB_CAP,
            SLIDER_BOTTOM,
            1.0,
        );
        list.draw_texture(
            x,
            ty + THUMB_CAP,
            SCROLL_WIDTH,
            self.thumb - 2 * THUMB_CAP,
            0,
            0,
            SCROLL_WIDTH,
            THUMB_CAP,
            SLIDER_CENTER,
            1.0,
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScrollWnd {
    pub slider: SliderBar,
    pub up_pressed: bool,
    pub down_pressed: bool,
}

impl ScrollWnd {
    pub fn new(w: i32, h: i32) -> ScrollWnd {
        ScrollWnd { slider: SliderBar::new(scroll_rects(w, h).slider.h), up_pressed: false, down_pressed: false }
    }

    pub fn hit(&self, w: i32, h: i32, x: i32, y: i32) -> Option<ScrollHit> {
        let rects = scroll_rects(w, h);
        if rects.up.contains(x, y) {
            return Some(ScrollHit::Up);
        }
        if rects.down.contains(x, y) {
            return Some(ScrollHit::Down);
        }
        if self.slider.shown && rects.slider.contains(x, y) {
            return Some(ScrollHit::Slider(y - rects.slider.y));
        }
        None
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, w: i32, h: i32) {
        let rects = scroll_rects(w, h);
        let up = if self.up_pressed { UP_ON_BTN } else { UP_BTN };
        let down = if self.down_pressed { DOWN_ON_BTN } else { DOWN_BTN };
        let (bw, bh) = (BUTTON_SIZE, BUTTON_SIZE);
        list.draw_texture(x + rects.up.x, y + rects.up.y, bw, bh, 0, 0, bw, bh, up, 1.0);
        list.draw_texture(x + rects.down.x, y + rects.down.y, bw, bh, 0, 0, bw, bh, down, 1.0);
        self.slider.paint(list, x + rects.slider.x, y + rects.slider.y);
    }
}

pub fn rows_for(count: usize, cols: usize) -> i32 {
    let total = (count / cols) as i32;
    if count.is_multiple_of(cols) { total } else { total + 1 }
}

pub fn clamp_first_row(first: i32, visible: i32, total: i32) -> i32 {
    if total >= visible && total - first < visible { total - visible } else { first }
}

pub fn line_up(first: i32, visible: i32, total: i32) -> i32 {
    if visible < total { (first - 1).max(0) } else { first }
}

pub fn line_down(first: i32, visible: i32, total: i32) -> i32 {
    if visible < total { (first + 1).min(total - visible) } else { first }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scroll_bar_hugs_the_right_edge_with_the_buttons_at_both_ends() {
        let rects = scroll_rects(236, 139);
        assert_eq!(rects.up, Rect::new(221, 0, 15, 15));
        assert_eq!(rects.down, Rect::new(221, 124, 15, 15));
        assert_eq!(rects.slider, Rect::new(221, 15, 15, 109));
    }

    #[test]
    fn rows_round_up_and_the_first_row_is_pulled_back_when_the_list_shrinks() {
        assert_eq!(rows_for(0, 6), 0);
        assert_eq!(rows_for(6, 6), 1);
        assert_eq!(rows_for(7, 6), 2);
        assert_eq!(rows_for(62, 6), 11);
        assert_eq!(clamp_first_row(9, 4, 11), 7);
        assert_eq!(clamp_first_row(3, 4, 11), 3);
        assert_eq!(clamp_first_row(3, 4, 2), 3);
        assert_eq!(line_up(0, 4, 11), 0);
        assert_eq!(line_up(3, 4, 11), 2);
        assert_eq!(line_up(3, 4, 2), 3);
        assert_eq!(line_down(6, 4, 11), 7);
        assert_eq!(line_down(7, 4, 11), 7);
        assert_eq!(line_down(0, 4, 2), 0);
    }

    #[test]
    fn set_range_sizes_the_thumb_by_the_visible_share_and_hides_it_when_everything_fits() {
        let mut bar = SliderBar::new(109);
        bar.set_range(11, 4);
        assert!(bar.shown);
        assert_eq!(bar.thumb, 39);
        assert_eq!(bar.track, 70);
        bar.set_pos(7);
        assert_eq!(bar.thumb_y, 70);
        bar.set_pos(3);
        assert_eq!(bar.thumb_y, 30);
        bar.set_range(100, 4);
        assert_eq!(bar.thumb, MIN_THUMB);
        bar.set_range(2, 4);
        assert!(!bar.shown);
        assert_eq!(bar.thumb, 109);
        assert_eq!(bar.track, 0);
        bar.set_pos(0);
        assert_eq!((bar.pos, bar.thumb_y), (0, 30));
        bar.set_pos(0);
        assert_eq!(bar.thumb_y, 0);
    }

    #[test]
    fn reset_1002c330_zeroes_thumb_page_and_grab_but_keeps_the_total() {
        let mut bar = SliderBar::new(100);
        bar.set_range(11, 4);
        bar.mouse_down(5);
        bar.reset();
        assert_eq!((bar.thumb_y, bar.pos, bar.visible, bar.thumb, bar.grab, bar.dragging), (0, 0, 0, 0, 0, false));
        assert_eq!(bar.total, 11);
    }

    #[test]
    fn set_range_1002c0a0_divides_in_double_precision() {
        let mut bar = SliderBar::new(16_777_217);
        bar.set_range(3, 1);
        assert_eq!(bar.thumb, 5_592_405);
    }

    #[test]
    fn dragging_the_thumb_moves_it_within_the_track_and_reports_the_row() {
        let mut bar = SliderBar::new(109);
        bar.set_range(11, 4);
        bar.set_pos(0);
        assert!(bar.mouse_down(50));
        assert!(!bar.dragging);
        assert!(bar.mouse_down(10));
        assert!(bar.dragging);
        assert_eq!(bar.grab, 10);
        assert_eq!(bar.mouse_move(10), None);
        assert_eq!(bar.mouse_move(5), None);
        assert_eq!(bar.mouse_move(45), Some(3));
        assert_eq!(bar.thumb_y, 35);
        assert_eq!(bar.mouse_move(200), Some(7));
        assert_eq!(bar.thumb_y, 70);
        assert_eq!(bar.mouse_move(300), None);
        assert_eq!(bar.mouse_move(10), Some(0));
        assert_eq!(bar.thumb_y, 0);
        bar.mouse_up();
        assert!(!bar.dragging);
        assert_eq!(bar.mouse_move(50), None);
    }

    #[test]
    fn paint_draws_the_buttons_and_the_thumb_from_three_pieces() {
        let mut wnd = ScrollWnd::new(236, 139);
        wnd.slider.set_range(11, 4);
        wnd.slider.set_pos(3);
        let mut list = DrawList::default();
        wnd.paint(&mut list, 9, 189, 236, 139);
        let up = list.quads.iter().find(|q| q.texture == UP_BTN).unwrap();
        assert_eq!((up.x, up.y, up.w, up.h), (230.0, 189.0, 15.0, 15.0));
        let down = list.quads.iter().find(|q| q.texture == DOWN_BTN).unwrap();
        assert_eq!((down.x, down.y), (230.0, 313.0));
        let top = list.quads.iter().find(|q| q.texture == SLIDER_TOP).unwrap();
        assert_eq!((top.x, top.y, top.w, top.h), (230.0, 234.0, 15.0, 8.0));
        let center = list.quads.iter().find(|q| q.texture == SLIDER_CENTER).unwrap();
        assert_eq!((center.y, center.h), (242.0, 23.0));
        let bottom = list.quads.iter().find(|q| q.texture == SLIDER_BOTTOM).unwrap();
        assert_eq!((bottom.y, bottom.h), (265.0, 8.0));
        assert_eq!(wnd.hit(236, 139, 225, 5), Some(ScrollHit::Up));
        assert_eq!(wnd.hit(236, 139, 225, 130), Some(ScrollHit::Down));
        assert_eq!(wnd.hit(236, 139, 225, 50), Some(ScrollHit::Slider(35)));
        assert_eq!(wnd.hit(236, 139, 100, 50), None);
        wnd.slider.set_range(2, 4);
        let mut list = DrawList::default();
        wnd.paint(&mut list, 0, 0, 236, 139);
        assert!(list.quads.iter().all(|q| q.texture != SLIDER_TOP));
        assert_eq!(wnd.hit(236, 139, 225, 50), None);
    }
}
