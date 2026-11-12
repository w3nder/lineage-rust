use crate::draw::DrawList;
use crate::window::Rect;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ViewportWnd {
    pub offset: (i32, i32),
    pub view: (f32, f32),
}

impl ViewportWnd {
    pub const fn new(offset: (i32, i32), view: (f32, f32)) -> ViewportWnd {
        ViewportWnd { offset, view }
    }

    pub fn region(&self) -> Rect {
        Rect::new(self.offset.0, self.offset.1, self.view.0 as i32, self.view.1 as i32)
    }

    pub fn draw(&self, list: &mut DrawList, origin: (i32, i32), texture: &str, source: i32, alpha: f32) {
        let region = self.region();
        list.draw_texture(origin.0 + region.x, origin.1 + region.y, region.w, region.h, 0, 0, source, source, texture, alpha);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_view_is_empty_and_the_region_truncates_the_float_size() {
        assert_eq!(ViewportWnd::default().region(), Rect::new(0, 0, 0, 0));
        assert_eq!(ViewportWnd::new((2, 20), (252.9, 254.5)).region(), Rect::new(2, 20, 252, 254));
    }

    #[test]
    fn draw_places_the_picture_at_the_window_origin_plus_the_offset() {
        let mut list = DrawList::default();
        ViewportWnd::new((2, 20), (252.0, 254.0)).draw(&mut list, (100, 100), "view", 256, 1.0);
        let quad = &list.quads[0];
        assert_eq!((quad.x, quad.y, quad.w, quad.h, quad.uw, quad.vh), (102.0, 120.0, 252.0, 254.0, 256.0, 256.0));
    }
}
