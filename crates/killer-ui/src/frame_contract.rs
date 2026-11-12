use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{self, FrameCtrl, FrameMode};
use crate::window::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameTitle {
    None,
    Literal(&'static str),
    Sysstring(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameContract {
    pub rect: Rect,
    pub id: u32,
    pub mode: FrameMode,
    pub title: FrameTitle,
    pub close_button: bool,
    pub minimize_button: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameHit {
    Close,
    Minimize,
    Grip,
}

impl FrameContract {
    pub const fn rect_at(&self, origin: (i32, i32)) -> Rect {
        Rect::new(self.rect.x + origin.0, self.rect.y + origin.1, self.rect.w, self.rect.h)
    }

    pub fn build(&self, text: impl Fn(u32) -> String) -> FrameCtrl {
        FrameCtrl {
            mode: self.mode,
            title: match self.title {
                FrameTitle::None => String::new(),
                FrameTitle::Literal(title) => title.to_string(),
                FrameTitle::Sysstring(id) => text(id),
            },
            close_button: self.close_button,
            minimize_button: self.minimize_button,
        }
    }

    pub fn paint(&self, frame: &FrameCtrl, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) {
        let rect = self.rect_at(origin);
        let start = list.quads.len();
        frame.paint(list, rect.x, rect.y, rect.w, rect.h, font, alpha);
        list.clip_from(start, rect);
    }

    pub fn grip(&self, frame: &FrameCtrl, origin: (i32, i32)) -> Rect {
        let rect = self.rect_at(origin);
        let grip = frame.grip(rect.w, rect.h);
        Rect::new(rect.x + grip.x, rect.y + grip.y, grip.w, grip.h)
    }

    pub fn button_rect(&self, frame: &FrameCtrl, origin: (i32, i32), button: FrameHit) -> Option<Rect> {
        if frame.mode != FrameMode::Title {
            return None;
        }
        let inset = match button {
            FrameHit::Close if frame.close_button => frame::CLOSE_INSET,
            FrameHit::Minimize if frame.minimize_button => frame::MINI_INSET,
            _ => return None,
        };
        let rect = self.rect_at(origin);
        Some(Rect::new(rect.x + rect.w - inset, rect.y + frame::BUTTON_Y, frame::BUTTON_SIZE, frame::BUTTON_SIZE))
    }

    pub fn hit(&self, frame: &FrameCtrl, origin: (i32, i32), point: (i32, i32)) -> Option<FrameHit> {
        for button in [FrameHit::Close, FrameHit::Minimize] {
            if self.button_rect(frame, origin, button).is_some_and(|rect| rect.contains(point.0, point.1)) {
                return Some(button);
            }
        }
        self.grip(frame, origin).contains(point.0, point.1).then_some(FrameHit::Grip)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    #[test]
    fn ncframectrl_10012860_title_source_and_existing_api_are_preserved() {
        let base = FrameContract { rect: Rect::new(4, 7, 256, 20), id: 4, mode: FrameMode::Title,
            title: FrameTitle::Sysstring(477), close_button: true, minimize_button: false };
        let mut view = base.build(|id| format!("id{id}"));
        assert_eq!(view, FrameCtrl::titled("id477", true));
        view.title = "changed".into();
        view.minimize_button = true;
        let mut actual = DrawList::default();
        let mut expected = DrawList::default();
        base.paint(&view, &mut actual, (10, 30), &font(), 0.5);
        view.paint(&mut expected, 14, 37, 256, 20, &font(), 0.5);
        assert_eq!(actual.quads, expected.quads);
        assert_eq!(base.grip(&view, (10, 30)), Rect::new(14, 37, 256, 20));
        for (title, expected) in [(FrameTitle::None, ""), (FrameTitle::Literal("title"), "title")] {
            assert_eq!(FrameContract { title, ..base }.build(|_| panic!("lookup não deveria ser chamada")).title, expected);
        }
    }

    #[test]
    fn ncframectrl_on_paint_10011d80_clips_long_titles_without_touching_previous_quads() {
        let contract = FrameContract { rect: Rect::new(0, 0, 40, 20), id: 0, mode: FrameMode::Title,
            title: FrameTitle::Literal("long title beyond frame width"), close_button: false, minimize_button: false };
        let frame = contract.build(|_| String::new());
        let mut list = DrawList::default();
        list.draw_texture(-100, -100, 10, 10, 0, 0, 10, 10, "prior", 1.0);
        let before = list.quads[0].clone();
        contract.paint(&frame, &mut list, (10, 30), &font(), 1.0);
        assert_eq!(list.quads[0], before);
        assert!(list.quads[1..].iter().all(|q| q.x >= 10.0 && q.y >= 30.0 && q.x + q.w <= 50.0 && q.y + q.h <= 50.0));
        assert!(list.quads[1..].iter().any(|q| q.texture == "f"));
    }

    #[test]
    fn ncframectrl_create_buttons_10012380_geometry_has_no_window_actions() {
        let contract = FrameContract { rect: Rect::new(0, 0, 256, 20), id: 4, mode: FrameMode::Title,
            title: FrameTitle::None, close_button: true, minimize_button: true };
        let mut frame = contract.build(|_| String::new());
        assert_eq!(contract.button_rect(&frame, (10, 30), FrameHit::Close), Some(Rect::new(243, 33, 15, 15)));
        assert_eq!(contract.button_rect(&frame, (10, 30), FrameHit::Minimize), Some(Rect::new(226, 33, 15, 15)));
        for x in 9..=267 {
            for y in 29..=51 {
                let expected = if Rect::new(243, 33, 15, 15).contains(x, y) { Some(FrameHit::Close) }
                    else if Rect::new(226, 33, 15, 15).contains(x, y) { Some(FrameHit::Minimize) }
                    else if Rect::new(10, 30, 256, 20).contains(x, y) { Some(FrameHit::Grip) }
                    else { None };
                assert_eq!(contract.hit(&frame, (10, 30), (x, y)), expected);
            }
        }
        frame.close_button = false;
        assert_eq!(contract.hit(&frame, (10, 30), (244, 34)), Some(FrameHit::Grip));
    }

    #[test]
    fn ncframectrl_on_paint_10011d80_vertical_geometry_matches_the_existing_control() {
        let contract = FrameContract { rect: Rect::new(0, 19, 12, 84), id: 0, mode: FrameMode::VerticalGrip,
            title: FrameTitle::None, close_button: false, minimize_button: false };
        let frame = contract.build(|_| String::new());
        let mut list = DrawList::default();
        contract.paint(&frame, &mut list, (20, 30), &font(), 0.75);
        assert_eq!(list.quads.len(), 3);
        assert_eq!((list.quads[0].x, list.quads[0].y, list.quads[0].w, list.quads[0].h), (20.0, 49.0, 12.0, 8.0));
        assert_eq!((list.quads[1].y, list.quads[1].h, list.quads[1].uw, list.quads[1].vh), (57.0, 68.0, 12.0, 8.0));
        assert_eq!(contract.grip(&frame, (20, 30)), Rect::new(20, 49, 12, 84));
        assert_eq!(contract.button_rect(&frame, (20, 30), FrameHit::Close), None);
    }
}
