use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::tab_button::TabButton;
use crate::window::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ancestor {
    pub is_control: bool,
    pub rect: Rect,
}

pub fn clip_ancestor(ancestors: &[Ancestor]) -> Option<Rect> {
    ancestors.iter().find(|ancestor| !ancestor.is_control).map(|ancestor| ancestor.rect)
}

pub fn paint_clipped(body: DrawList, list: &mut DrawList, ancestors: &[Ancestor]) {
    match clip_ancestor(ancestors) {
        Some(clip) => list.append_clipped(body, clip),
        None => list.append_clipped(body, Rect::new(i32::MIN / 2, i32::MIN / 2, i32::MAX, i32::MAX)),
    }
}

pub fn paint(button: &TabButton, list: &mut DrawList, origin: (i32, i32), ancestors: &[Ancestor], font: &BitmapFont) {
    let mut body = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
    button.paint_body(&mut body, origin, font);
    paint_clipped(body, list, ancestors);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn button() -> TabButton {
        TabButton { rect: Rect::new(0, 0, 40, 20), textures: [Some("n".into()), None, None], source: (0, 0), state: 0, enabled: true, label: String::new(), text_offset: (0, 0) }
    }

    #[test]
    fn html_push_button_10047e40_clips_to_the_first_ancestor_that_is_not_a_control() {
        let control = Ancestor { is_control: true, rect: Rect::new(0, 0, 10, 10) };
        let window = Ancestor { is_control: false, rect: Rect::new(0, 0, 25, 20) };
        let outer = Ancestor { is_control: false, rect: Rect::new(0, 0, 5, 5) };
        assert_eq!(clip_ancestor(&[control, control, window, outer]), Some(window.rect));
        assert_eq!(clip_ancestor(&[control, control]), None);
        assert_eq!(clip_ancestor(&[]), None);
    }

    #[test]
    fn html_push_button_10047e40_draws_the_base_button_inside_that_clip() {
        let font = font();
        let ancestors = [Ancestor { is_control: true, rect: Rect::new(0, 0, 10, 10) }, Ancestor { is_control: false, rect: Rect::new(0, 0, 25, 20) }];
        let mut list = DrawList::default();
        paint(&button(), &mut list, (0, 0), &ancestors, &font);
        assert_eq!((list.quads[0].w, list.quads[0].uw), (25.0, 25.0));
        let mut wide = button();
        wide.rect = Rect::new(0, 0, 40, 20);
        let mut list = DrawList::default();
        paint(&wide, &mut list, (0, 0), &[Ancestor { is_control: false, rect: Rect::new(0, 0, 200, 200) }], &font);
        assert_eq!(list.quads[0].w, 40.0);
        let mut list = DrawList::default();
        paint(&button(), &mut list, (0, 0), &[], &font);
        assert_eq!(list.quads[0].w, 40.0);
    }
}
