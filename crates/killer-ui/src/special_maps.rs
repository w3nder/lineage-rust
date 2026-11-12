#[path = "xmas_layout_generated.rs"]
mod xmas_layout;

#[path = "town_map_layout_generated.rs"]
mod town_layout;

use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::window::Rect;

pub const SIZE: (i32, i32) = town_layout::SIZE;
pub const BACK: &str = "L2UI.MinimapWnd.MinimapBack";
pub const MARKER: &str = "town_map.icon_i00";
pub const CLOSE: Rect = Rect::new(SIZE.0 - crate::frame::CLOSE_INSET, crate::frame::BUTTON_Y, crate::frame::BUTTON_SIZE, crate::frame::BUTTON_SIZE);

#[derive(Debug, Clone)]
pub struct TownMapView {
    pub frame: FrameCtrl,
    pub texture: String,
    pub marker: (i32, i32),
    pub scroll: (i32, i32),
    drag: Option<(i32, i32)>,
}

impl TownMapView {
    pub fn new(title: &str) -> Self {
        Self { frame: FrameCtrl::titled(title, true), texture: String::new(), marker: (0, 0), scroll: (0, 0), drag: None }
    }

    pub fn receive(&mut self, texture: String, x: i32, y: i32) {
        self.texture = texture;
        let coordinate = |v| if (0..=512).contains(&v) { v } else { 0 };
        self.marker = (coordinate(x), coordinate(y));
        self.scroll = ((self.marker.0 - 164).clamp(0, 184), (self.marker.1 - 164).clamp(0, 184));
    }

    fn point(x: i32, y: i32) -> Option<(i32, i32)> {
        let p = (x.wrapping_sub(3).max(3), y.wrapping_sub(21).max(21));
        (p.0 > 3 && p.0 <= SIZE.0 && p.1 > 21 && p.1 <= SIZE.1).then_some(p)
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) {
        if let Some(p) = Self::point(x, y) { self.drag = Some(p); }
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) -> bool {
        let (Some(last), Some(p)) = (self.drag, Self::point(x, y)) else { return false };
        self.scroll = ((self.scroll.0 + last.0 - p.0).clamp(0, 184), (self.scroll.1 + last.1 - p.1).clamp(0, 184));
        self.drag = Some(p);
        true
    }

    pub fn is_dragging(&self) -> bool { self.drag.is_some() }

    pub fn mouse_up(&mut self) { self.drag = None; }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        if !self.texture.is_empty() {
            list.draw_texture(x + 3, y + 21, 328, 328, self.scroll.0, self.scroll.1, 328, 328, &self.texture, alpha);
        }
        let canvas = crate::canvas_clip::CanvasClip { flags: 8, origin: [x as f32, y as f32], size: [SIZE.0 as f32, SIZE.1 as f32], cursor: [0.0; 2], saved_cursor: [0.0; 2], saved_origin: [0.0; 2], saved_size: [0.0; 2] };
        list.draw_canvas_texture(&canvas, 0.0, crate::canvas_texture::CanvasTextureInput {
            texture: MARKER, rect: [x + 3 + self.marker.0 - self.scroll.0, y + 21 + self.marker.1 - self.scroll.1, 32, 32],
            uv: [0.0, 0.0, 32.0, 32.0], alpha: (alpha * 255.0) as u32, clip: 1,
        });
        list.draw_texture(x, y + 21, 334, 333, 0, 0, 334, 333, BACK, alpha);
        self.frame.paint(list, x, y, town_layout::RECT_7C.w, town_layout::RECT_7C.h, font, alpha);
    }
}

#[derive(Debug, Clone)]
pub struct XmasSealView {
    pub frame: FrameCtrl,
    pub texture: Option<String>,
}

impl XmasSealView {
    pub fn new(title: &str) -> Self { Self { frame: FrameCtrl::titled(title, true), texture: None } }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        list.draw_texture(x, y + 20, 334, 333, 0, 0, 334, 333, BACK, alpha);
        if let Some(texture) = &self.texture {
            list.draw_texture(x + 3, y + 21, 328, 329, 0, 0, 328, 329, texture, alpha);
        }
        self.frame.paint(list, x, y, xmas_layout::SIZE.0, xmas_layout::RECT_7C.h, font, alpha);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn town_map_100b8ce0_bounds_center_and_replacement() {
        let mut v = TownMapView::new("");
        for (input, marker, scroll) in [(-1,0,0),(0,0,0),(164,164,0),(256,256,92),(348,348,184),(512,512,184),(513,0,0)] {
            v.receive("map".into(), input, input);
            assert_eq!((v.marker, v.scroll), ((marker,marker),(scroll,scroll)));
        }
        v.receive("first".into(),256,256);
        v.mouse_down(100,100);
        assert!(v.mouse_move(50,70));
        assert_eq!(v.scroll,(142,122));
        assert!(!v.mouse_move(0,0));
        v.receive("second".into(),0,512);
        assert!(v.mouse_move(70,90));
        assert_eq!(v.scroll,(0,164));
        v.mouse_up();
        assert!(!v.mouse_move(100,100));
    }
}
