use crate::color::TEXT;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::window::Rect;

pub const CARET: &str = "|";
pub const LIMIT: i32 = 0x61;
pub const TEXT_Y: i32 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeBox {
    pub rect: Rect,
    pub background: Option<(String, (i32, i32))>,
    pub text: Vec<char>,
    pub first: i32,
    pub caret: i32,
    pub caret_visible: bool,
}

pub const BACKGROUND: &str = "NWindow.BlackTexture";

impl CompositeBox {
    pub fn new(rect: Rect, caret_visible: bool) -> CompositeBox {
        CompositeBox { rect, background: None, text: Vec::new(), first: 0, caret: 0, caret_visible }
    }

    pub fn caret_left(&mut self) {
        if self.caret > 0 {
            self.caret -= 1;
        }
        if self.first > self.caret {
            self.first = self.caret;
        }
    }

    fn scroll_to_caret(&mut self, font: &BitmapFont) {
        while self.span_width(font) > LIMIT {
            self.first += 1;
        }
    }

    fn span_width(&self, font: &BitmapFont) -> i32 {
        let end = self.caret.clamp(0, self.count());
        let start = self.first.clamp(0, end);
        let span: String = self.text[start as usize..end as usize].iter().collect();
        font.measure(&span) as i32
    }

    pub fn caret_right(&mut self, font: &BitmapFont) {
        if self.count() > self.caret {
            self.caret += 1;
        }
        self.scroll_to_caret(font);
    }

    pub fn set_text(&mut self, text: &str, font: &BitmapFont) {
        self.first = 0;
        self.text = text.chars().chain(std::iter::once('\0')).collect();
        self.scroll_to_caret(font);
    }

    pub fn get_text(&self) -> String {
        self.text.iter().take(self.count() as usize).collect()
    }

    pub fn set_show_caret(&mut self, show: bool) {
        self.caret_visible = show;
    }

    fn count(&self) -> i32 {
        if self.text.is_empty() { 0 } else { self.text.len() as i32 - 1 }
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont) {
        let x = origin.0 + self.rect.x;
        let y = origin.1 + self.rect.y;
        let mut body = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
        if let Some((texture, (tw, th))) = &self.background {
            body.draw_texture(x, y, self.rect.w, self.rect.h, 0, 0, *tw, *th, texture, 1.0);
        }
        let mut index = self.first;
        let mut used = 0;
        while self.count() > index {
            if index == self.caret && self.caret_visible {
                body.draw_text(x + used + 1, y + TEXT_Y, TEXT, CARET, font);
            }
            let glyph = self.text.get(index.max(0) as usize).map(|c| c.to_string()).unwrap_or_default();
            let start = x + used + 3;
            used += body.draw_text(start, y + TEXT_Y, TEXT, &glyph, font) - start;
            if used > LIMIT {
                break;
            }
            index += 1;
        }
        if index == self.caret && self.caret_visible {
            body.draw_text(x + used + 1, y + TEXT_Y, TEXT, CARET, font);
        }
        list.append_clipped(body, Rect::new(x, y, self.rect.w, self.rect.h));
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

    fn boxed(text: &str, first: i32, caret: i32) -> CompositeBox {
        let mut chars: Vec<char> = text.chars().collect();
        chars.push('\0');
        CompositeBox { rect: Rect::new(0, 0, 100, 20), background: None, text: chars, first, caret, caret_visible: true }
    }

    #[test]
    fn composite_box_100040a0_10004a10_10004b80_move_the_caret_and_scroll_the_first_char() {
        let font = font();
        let mut b = CompositeBox::new(Rect::new(0, 0, 100, 20), true);
        b.set_text(&"m".repeat(30), &font);
        assert_eq!((b.first, b.caret, b.get_text().len()), (0, 0, 30));
        let step = font.measure("m") as i32;
        for _ in 0..30 {
            b.caret_right(&font);
        }
        assert_eq!(b.caret, 30);
        assert_eq!(b.first, 30 - LIMIT / step);
        b.caret_right(&font);
        assert_eq!(b.caret, 30);
        for _ in 0..30 {
            b.caret_left();
        }
        assert_eq!((b.caret, b.first), (0, 0));
        b.set_show_caret(false);
        assert!(!b.caret_visible);
    }

    #[test]
    fn composite_box_100048b0_draws_the_caret_before_the_glyph_it_sits_on() {
        let font = font();
        let a = font.measure("a") as f32;
        let mut list = DrawList::default();
        boxed("abc", 0, 1).paint(&mut list, (0, 0), &font);
        let xs: Vec<f32> = list.quads.iter().map(|q| q.x).collect();
        assert_eq!(xs.len(), 4);
        assert_eq!(xs[0], 3.0);
        assert_eq!(xs[1], a + 1.0);
        assert_eq!(xs[2], a + 3.0);
        let mut list = DrawList::default();
        boxed("abc", 0, 3).paint(&mut list, (0, 0), &font);
        assert_eq!(list.quads.len(), 4);
        assert_eq!(list.quads.last().unwrap().x, 3.0 * a + 1.0);
        let mut list = DrawList::default();
        boxed("abc", 0, 9).paint(&mut list, (0, 0), &font);
        assert_eq!(list.quads.len(), 3);
    }

    #[test]
    fn composite_box_100048b0_stops_once_the_used_width_passes_ninety_seven() {
        let font = font();
        let mut long = boxed(&"m".repeat(60), 0, -1);
        long.caret_visible = false;
        let mut list = DrawList::default();
        long.paint(&mut list, (0, 0), &font);
        let step = font.measure("m") as i32;
        let expected = (LIMIT / step + 1) as usize;
        assert_eq!(list.quads.len(), expected);
        let mut later = boxed("abcd", 2, -1);
        later.caret_visible = false;
        let mut list = DrawList::default();
        later.paint(&mut list, (0, 0), &font);
        assert_eq!(list.quads.len(), 2);
    }
}
