use crate::color::{Argb, TEXT};
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::window::Rect;

pub const ROW_HEIGHT: i32 = 17;
pub const DISABLED_TEXT: Argb = Argb(0xFF7E7E7E);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ancestor {
    pub matches_class: bool,
    pub rect: Rect,
}

pub fn clip_ancestor(ancestors: &[Ancestor]) -> Option<Rect> {
    ancestors.iter().find(|ancestor| ancestor.matches_class).map(|ancestor| ancestor.rect)
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HtmlComboBox {
    pub rect: Rect,
    pub enabled: bool,
    pub list_visible: bool,
    pub open: bool,
    pub entries: Vec<String>,
    pub selected: i32,
    pub hovered: i32,
    pub indent: (i32, i32),
    pub text_height: i32,
    pub active: [String; 3],
    pub inactive: [String; 3],
    pub select: [String; 2],
}

impl HtmlComboBox {
    fn row(&self, body: &mut DrawList, x: i32, y: i32, textures: &[String; 3]) {
        let w = self.rect.w;
        body.draw_texture(x, y, 8, ROW_HEIGHT, 0, 0, 8, ROW_HEIGHT, &textures[0], 1.0);
        body.draw_texture(x + 8, y, w - 16, ROW_HEIGHT, 0, 0, 8, ROW_HEIGHT, &textures[1], 1.0);
        body.draw_texture(x + w - 8, y, 8, ROW_HEIGHT, 0, 0, 8, ROW_HEIGHT, &textures[2], 1.0);
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), ancestors: &[Ancestor], font: &BitmapFont) {
        let x = origin.0 + self.rect.x;
        let y = origin.1 + self.rect.y;
        let clip = if self.open { Some(Rect::new(x, y, self.rect.w, self.rect.h)) } else { clip_ancestor(ancestors) };
        let mut body = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
        let text_y = 9 - self.text_height / 2;
        let count = self.entries.len() as i32;
        let (row_textures, color) = if self.enabled { (&self.active, TEXT) } else { (&self.inactive, DISABLED_TEXT) };
        self.row(&mut body, x, y, row_textures);
        if count > self.selected && self.selected >= 0 {
            body.draw_text(x + self.indent.0, y + self.indent.1 + text_y, color, &self.entries[self.selected as usize], font);
        }
        if self.open && self.list_visible {
            for index in 0..=count {
                self.row(&mut body, x, y + index * ROW_HEIGHT, &self.active);
            }
            for (index, entry) in (0..count).zip(&self.entries) {
                let row_y = y + index * ROW_HEIGHT + ROW_HEIGHT;
                if self.hovered == index {
                    body.draw_texture(x + 1, row_y, self.rect.w - 32, 16, 0, 0, 0x10, 0xd, &self.select[0], 1.0);
                    body.draw_texture(x + self.rect.w - 31, row_y, 0x20, 16, 0, 0, 0x20, 0xd, &self.select[1], 1.0);
                }
                body.draw_text(x + self.indent.0, row_y + text_y, TEXT, entry, font);
            }
        }
        match clip {
            Some(clip) => list.append_clipped(body, clip),
            None => list.append_clipped(body, Rect::new(i32::MIN / 2, i32::MIN / 2, i32::MAX, i32::MAX)),
        }
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

    fn names(prefix: &str) -> [String; 3] {
        [format!("{prefix}1"), format!("{prefix}2"), format!("{prefix}3")]
    }

    fn combo() -> HtmlComboBox {
        HtmlComboBox {
            rect: Rect::new(10, 20, 100, 17),
            enabled: true,
            list_visible: true,
            open: false,
            entries: vec!["A".into(), "B".into()],
            selected: 1,
            hovered: -1,
            indent: (4, 1),
            text_height: 12,
            active: names("on"),
            inactive: names("off"),
            select: ["s1".into(), "s2".into()],
        }
    }

    #[test]
    fn html_combo_box_10047740_draws_the_row_in_three_slices_and_the_selected_text() {
        let mut list = DrawList::default();
        combo().paint(&mut list, (0, 0), &[], &font());
        let rows: Vec<_> = list.quads.iter().filter(|q| q.texture != "f").map(|q| (q.texture.to_string(), q.x, q.w, q.h)).collect();
        assert_eq!(rows, vec![("on1".to_string(), 10.0, 8.0, 17.0), ("on2".to_string(), 18.0, 84.0, 17.0), ("on3".to_string(), 102.0, 8.0, 17.0)]);
        let text: Vec<_> = list.quads.iter().filter(|q| q.texture == "f").collect();
        assert_eq!((text[0].x, text[0].y, text[0].color), (14.0, 20.0 + 1.0 + 3.0, TEXT.rgba()));
    }

    #[test]
    fn html_combo_box_10047740_uses_inactive_art_and_grey_text_when_disabled() {
        let mut c = combo();
        c.enabled = false;
        let mut list = DrawList::default();
        c.paint(&mut list, (0, 0), &[], &font());
        assert!(list.quads.iter().any(|q| q.texture == "off1"));
        assert!(list.quads.iter().filter(|q| q.texture == "f").all(|q| q.color == DISABLED_TEXT.rgba()));
        c.selected = -1;
        let mut list = DrawList::default();
        c.paint(&mut list, (0, 0), &[], &font());
        assert!(list.quads.iter().all(|q| q.texture != "f"));
    }

    #[test]
    fn html_combo_box_10047740_open_list_adds_a_row_per_entry_and_highlights_the_hovered_one() {
        let mut c = combo();
        c.open = true;
        c.rect.h = 17 * 3;
        c.hovered = 1;
        let mut list = DrawList::default();
        c.paint(&mut list, (0, 0), &[], &font());
        assert_eq!(list.quads.iter().filter(|q| q.texture == "on1").count(), 4);
        let highlight = list.quads.iter().find(|q| q.texture == "s1").unwrap();
        assert_eq!((highlight.x, highlight.y + 0.0, highlight.w, highlight.h), (11.0, 20.0 + 34.0, 68.0, 16.0));
        let end = list.quads.iter().find(|q| q.texture == "s2").unwrap();
        assert_eq!((end.x, end.w), (79.0, 31.0));
        let texts: Vec<_> = list.quads.iter().filter(|q| q.texture == "f").map(|q| q.y).collect();
        assert_eq!(texts, vec![20.0 + 1.0 + 3.0, 20.0 + 17.0 + 3.0, 20.0 + 34.0 + 3.0]);
        c.list_visible = false;
        let mut list = DrawList::default();
        c.paint(&mut list, (0, 0), &[], &font());
        assert_eq!(list.quads.iter().filter(|q| q.texture == "on1").count(), 1);
    }

    #[test]
    fn html_combo_box_10047740_closed_clips_to_the_nearest_matching_ancestor() {
        let ancestors = [
            Ancestor { matches_class: false, rect: Rect::new(0, 0, 1, 1) },
            Ancestor { matches_class: true, rect: Rect::new(0, 0, 50, 30) },
        ];
        assert_eq!(clip_ancestor(&ancestors), Some(Rect::new(0, 0, 50, 30)));
        let mut list = DrawList::default();
        combo().paint(&mut list, (0, 0), &ancestors, &font());
        assert!(list.quads.iter().all(|q| q.x + q.w <= 50.0));
        assert_eq!(clip_ancestor(&[]), None);
    }
}
