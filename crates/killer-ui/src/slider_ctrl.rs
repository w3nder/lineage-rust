use crate::draw::DrawList;
use crate::window::Rect;

pub const BLACK_TEXTURE: &str = "NWindow.BlackTexture";
pub const THUMB_TEXTURES: [&str; 2] = ["NWindow.MenuHighlight", "NWindow.WhiteTexture"];
pub const THUMB_ID: u32 = 0xF0F0_1415;
pub const MSG_NOTIFY: i32 = 0x501;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Thumb {
    pub rect: Rect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliderCtrl {
    pub rect: Rect,
    pub background: Option<String>,
    pub max: i32,
    pub pos: i32,
    pub thumb_w: i32,
    pub thumb_h: i32,
    pub thumb: Option<Thumb>,
}

impl SliderCtrl {
    pub fn new(rect: Rect, max: i32, pos: i32, thumb_w: i32, thumb_h: i32) -> SliderCtrl {
        SliderCtrl { rect, background: None, max, pos, thumb_w, thumb_h, thumb: None }
    }

    pub fn on_create(&mut self) {
        self.background = Some(BLACK_TEXTURE.to_string());
        let x = (f64::from(self.pos) * f64::from(self.rect.w) / f64::from(self.max) - f64::from(self.thumb_w / 2)) as i32;
        self.thumb = Some(Thumb { rect: Rect::new(self.rect.x + x, self.rect.y, self.thumb_w, self.thumb_h) });
    }

    pub fn on_destroy(&mut self) {
        self.thumb = None;
    }

    pub fn on_thumb_moved(&mut self, absolute_x: i32) -> i32 {
        let previous = self.pos;
        let relative = absolute_x - self.rect.x;
        self.pos = if relative < self.max {
            if relative > 0 { relative } else { 0 }
        } else {
            self.max
        };
        let delta = self.pos - previous;
        if let Some(thumb) = &mut self.thumb {
            thumb.rect.x += delta;
        }
        delta
    }

    pub fn wnd_proc(&mut self, message: i32, id: u32, param: i32) -> bool {
        if message == MSG_NOTIFY && id == THUMB_ID {
            self.on_thumb_moved(param);
            return true;
        }
        false
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32)) {
        let x = origin.0 + self.rect.x;
        let y = origin.1 + self.rect.y;
        let mut body = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
        if let Some(texture) = &self.background {
            body.draw_texture(x, y, self.rect.w, self.rect.h, 0, 0, self.rect.w, self.rect.h, texture, 1.0);
        }
        list.append_clipped(body, Rect::new(x, y, self.rect.w, self.rect.h));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slider() -> SliderCtrl {
        let mut s = SliderCtrl::new(Rect::new(10, 20, 100, 12), 100, 40, 8, 12);
        s.on_create();
        s
    }

    #[test]
    fn slider_ctrl_1002cf20_creates_the_black_back_and_places_the_thumb_by_the_position_share() {
        let s = slider();
        assert_eq!(s.background.as_deref(), Some("NWindow.BlackTexture"));
        assert_eq!(s.thumb.map(|t| t.rect), Some(Rect::new(10 + 36, 20, 8, 12)));
    }

    #[test]
    fn slider_ctrl_1002c740_clamps_to_zero_and_max_and_moves_the_thumb_by_the_difference() {
        let mut s = slider();
        assert_eq!(s.on_thumb_moved(10 + 70), 30);
        assert_eq!(s.pos, 70);
        assert_eq!(s.thumb.map(|t| t.rect.x), Some(46 + 30));
        assert_eq!(s.on_thumb_moved(10 + 500), 30);
        assert_eq!(s.pos, 100);
        assert_eq!(s.on_thumb_moved(3), -100);
        assert_eq!(s.pos, 0);
        assert_eq!(s.on_thumb_moved(10), 0);
    }

    #[test]
    fn slider_ctrl_1002c6e0_only_accepts_the_thumb_notification() {
        let mut s = slider();
        assert!(s.wnd_proc(0x501, THUMB_ID, 10 + 50));
        assert_eq!(s.pos, 50);
        assert!(!s.wnd_proc(0x501, 1, 10 + 10));
        assert!(!s.wnd_proc(0x201, THUMB_ID, 10 + 10));
        assert_eq!(s.pos, 50);
    }

    #[test]
    fn slider_ctrl_1002cdc0_paints_the_back_over_its_rect_and_1002cd20_drops_the_thumb() {
        let mut s = slider();
        let mut list = DrawList::default();
        s.paint(&mut list, (5, 5));
        let q = &list.quads[0];
        assert_eq!((q.texture.as_ref(), q.x, q.y, q.w, q.h, q.uw, q.vh), ("NWindow.BlackTexture", 15.0, 25.0, 100.0, 12.0, 100.0, 12.0));
        s.on_destroy();
        assert!(s.thumb.is_none());
    }
}
