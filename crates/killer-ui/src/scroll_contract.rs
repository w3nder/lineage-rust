use crate::draw::DrawList;
use crate::scroll::{ScrollRects, SliderBar};
use crate::window::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollButtonContract {
    pub rect: Rect,
    pub id: u32,
    pub normal: &'static str,
    pub pressed: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollContract {
    pub up: ScrollButtonContract,
    pub down: ScrollButtonContract,
    pub slider_rect: Rect,
    pub slider_id: u32,
    pub top: &'static str,
    pub center: &'static str,
    pub bottom: &'static str,
    pub on_create: u32,
    pub slider_field: u32,
    pub slider_on_paint: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollContractError {
    Geometry,
    State,
    UnprovenThumb,
    Resource,
    NativePaint,
}

fn exact_rect(rect: Rect) -> bool {
    rect.w > 0
        && rect.h > 0
        && [
            Some(rect.x),
            Some(rect.y),
            Some(rect.w),
            Some(rect.h),
            rect.x.checked_add(rect.w),
            rect.y.checked_add(rect.h),
        ]
        .into_iter()
        .all(|value| value.is_some_and(|value| (-16_777_216..=16_777_216).contains(&value)))
}

impl ScrollContract {
    pub fn rects(&self) -> ScrollRects {
        ScrollRects { up: self.up.rect, down: self.down.rect, slider: self.slider_rect }
    }

    pub fn paint_slider(
        &self,
        list: &mut DrawList,
        origin: (i32, i32),
        slider: &SliderBar,
    ) -> Result<(), ScrollContractError> {
        if self.slider_on_paint != 0x1002c3a0 {
            return Err(ScrollContractError::NativePaint);
        }
        let rect = self.slider_rect;
        if rect.w != 15 || !exact_rect(rect) || !exact_rect(self.up.rect) || !exact_rect(self.down.rect) {
            return Err(ScrollContractError::Geometry);
        }
        if slider.height != rect.h {
            return Err(ScrollContractError::State);
        }
        if slider.thumb <= 0 {
            return Ok(());
        }
        if slider.thumb < 15 {
            return Err(ScrollContractError::UnprovenThumb);
        }
        if slider.thumb_y < 0 || i64::from(slider.thumb_y) + i64::from(slider.thumb) > i64::from(rect.h) {
            return Err(ScrollContractError::State);
        }
        if [self.top, self.center, self.bottom].iter().any(|texture| texture.is_empty()) {
            return Err(ScrollContractError::Resource);
        }
        let x = rect.x.checked_add(origin.0).ok_or(ScrollContractError::Geometry)?;
        let y = rect.y.checked_add(origin.1).ok_or(ScrollContractError::Geometry)?;
        if !exact_rect(Rect::new(x, y, rect.w, rect.h)) {
            return Err(ScrollContractError::Geometry);
        }
        let top_y = y.checked_add(slider.thumb_y).ok_or(ScrollContractError::Geometry)?;
        list.draw_texture(x, top_y, 15, 8, 0, 0, 15, 8, self.top, 1.0);
        list.draw_texture(x, top_y + slider.thumb - 8, 15, 8, 0, 0, 15, 8, self.bottom, 1.0);
        if slider.thumb > 16 {
            list.draw_texture(x, top_y + 8, 15, slider.thumb - 16, 0, 0, 15, 8, self.center, 1.0);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contract() -> ScrollContract {
        ScrollContract {
            up: ScrollButtonContract { rect: Rect::new(221, 0, 15, 15), id: 1, normal: "up", pressed: "up_pressed" },
            down: ScrollButtonContract {
                rect: Rect::new(221, 124, 15, 15),
                id: 2,
                normal: "down",
                pressed: "down_pressed",
            },
            slider_rect: Rect::new(221, 15, 15, 109),
            slider_id: 3,
            top: "top",
            center: "center",
            bottom: "bottom",
            on_create: 0x1002bda0,
            slider_field: 0x114,
            slider_on_paint: 0x1002c3a0,
        }
    }

    #[test]
    fn preserves_explicit_layout_skin_and_native_paint_order() {
        let mut contract = contract();
        contract.slider_rect = Rect::new(5, 20, 15, 109);
        let state = SliderBar { height: 109, thumb: 39, thumb_y: 30, ..Default::default() };
        let mut list = DrawList::default();
        contract.paint_slider(&mut list, (10, 20), &state).unwrap();
        assert_eq!(contract.rects().slider, Rect::new(5, 20, 15, 109));
        assert_eq!(list.quads.len(), 3);
        for (quad, y, h, texture) in [
            (&list.quads[0], 70.0, 8.0, "top"),
            (&list.quads[1], 101.0, 8.0, "bottom"),
            (&list.quads[2], 78.0, 23.0, "center"),
        ] {
            assert_eq!((quad.x, quad.y, quad.w, quad.h), (15.0, y, 15.0, h));
            assert_eq!((quad.u, quad.v, quad.uw, quad.vh), (0.0, 0.0, 15.0, 8.0));
            assert_eq!(quad.texture, texture);
            assert_eq!(quad.color, [1.0; 4]);
        }
    }

    #[test]
    fn rejects_unproven_thumb_clipping_and_overflow_without_partial_draw() {
        let contract = contract();
        let mut list = DrawList::default();
        for (height, thumb, thumb_y) in
            [(109, 1, 0), (109, 14, 0), (108, 39, 0), (109, 39, -1), (109, 39, 71), (109, i32::MAX, 0)]
        {
            let state = SliderBar { height, thumb, thumb_y, ..Default::default() };
            assert!(contract.paint_slider(&mut list, (0, 0), &state).is_err());
        }
        let state = SliderBar { height: 109, thumb: 39, thumb_y: 30, ..Default::default() };
        assert!(contract.paint_slider(&mut list, (i32::MAX, 0), &state).is_err());
        assert!(contract.paint_slider(&mut list, (0, 16_777_216), &state).is_err());
        assert!(ScrollContract { top: "", ..contract }.paint_slider(&mut list, (0, 0), &state).is_err());
        assert!(ScrollContract { slider_on_paint: 0, ..contract }.paint_slider(&mut list, (0, 0), &state).is_err());
        assert!(list.quads.is_empty());
    }

    #[test]
    fn nonpositive_thumb_draws_nothing_and_shown_belongs_to_traversal() {
        let mut list = DrawList::default();
        for thumb in [-1, 0] {
            contract()
                .paint_slider(&mut list, (0, 0), &SliderBar { height: 109, thumb, ..Default::default() })
                .unwrap();
        }
        assert!(list.quads.is_empty());
        let state = SliderBar { height: 109, thumb: 17, shown: false, ..Default::default() };
        contract().paint_slider(&mut list, (0, 0), &state).unwrap();
        assert_eq!(list.quads.len(), 3);
    }

    #[test]
    fn canvas_10306230_rejects_nonpositive_center_but_preserves_cap_order() {
        for thumb in [15, 16] {
            let mut list = DrawList::default();
            let state = SliderBar { height: 109, thumb, thumb_y: 30, ..Default::default() };
            contract().paint_slider(&mut list, (0, 0), &state).unwrap();
            assert_eq!(list.quads.len(), 2);
            assert_eq!((&*list.quads[0].texture, &*list.quads[1].texture), ("top", "bottom"));
            assert_eq!(list.quads[0].y, 45.0);
            assert_eq!(list.quads[1].y, (45 + thumb - 8) as f32);
        }
    }
}
