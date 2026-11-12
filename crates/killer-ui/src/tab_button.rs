use crate::color::{BUTTON_TEXT, BUTTON_TEXT_DISABLED};
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::window::Rect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabButton {
    pub rect: Rect,
    pub textures: [Option<String>; 3],
    pub source: (i32, i32),
    pub state: i32,
    pub enabled: bool,
    pub label: String,
    pub text_offset: (i32, i32),
}

impl TabButton {
    pub fn texture(&self) -> Option<&str> {
        match self.state {
            0 => self.textures[0].as_deref(),
            1 => self.textures[1].as_deref(),
            2 => self.textures[2].as_deref().or(self.textures[0].as_deref()),
            _ => None,
        }
    }

    pub fn paint_body(&self, body: &mut DrawList, origin: (i32, i32), font: &BitmapFont) {
        let x = origin.0 + self.rect.x;
        let y = origin.1 + self.rect.y;
        if let Some(texture) = self.texture() {
            body.draw_button_texture(x, y, self.rect.w, self.rect.h, self.source.0, self.source.1,
                self.rect.w, self.rect.h, texture, 1.0, self.enabled);
        }
        if !self.label.is_empty() {
            let tx = (self.text_offset.0 as f64 + self.rect.w as f64 * 0.5 - (font.measure(&self.label) / 2) as f64) as i32;
            let ty = (self.text_offset.1 as f64 + self.rect.h as f64 * 0.5 - (font.line_height / 2) as f64) as i32;
            body.draw_text(x + tx, y + ty, if self.enabled { BUTTON_TEXT } else { BUTTON_TEXT_DISABLED }, &self.label, font);
        }
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont) {
        let x = origin.0 + self.rect.x;
        let y = origin.1 + self.rect.y;
        let mut body = DrawList {capture_text:list.capture_text,..Default::default()};
        self.paint_body(&mut body, origin, font);
        list.append_clipped(body,crate::window::Rect::new(x,y,self.rect.w,self.rect.h));
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

    fn button() -> TabButton {
        TabButton { rect: Rect::new(12, 25, 74, 23), textures: [Some("normal_tab_on".into()), Some("normal_tab".into()), None], source: (3, 5), state: 0, enabled: true, label: "Video".into(), text_offset: (2, 1) }
    }

    #[test]
    fn tab_button_10001000_uses_raw_state_slots_and_only_hover_falls_back() {
        for mask in 0..8 {
            let mut button = button();
            button.textures = std::array::from_fn(|index| if mask & (1 << index) != 0 { Some(index.to_string()) } else { None });
            for state in [-1, 0, 1, 2, 3] {
                button.state = state;
                let expected = match state {
                    0 => if mask & 1 != 0 { Some("0") } else { None },
                    1 => if mask & 2 != 0 { Some("1") } else { None },
                    2 => if mask & 4 != 0 { Some("2") } else if mask & 1 != 0 { Some("0") } else { None },
                    _ => None,
                };
                assert_eq!(button.texture(), expected);
                button.enabled = false;
                assert_eq!(button.texture(), expected);
            }
        }
    }

    #[test]
    fn tab_button_10001000_preserves_source_alpha_offsets_and_disabled_label_color() {
        let mut button = button();
        for enabled in [false, true] {
            button.enabled = enabled;
            let mut list = DrawList::default();
            button.paint(&mut list, (100, 200), &font());
            let background = &list.quads[0];
            assert_eq!((background.x, background.y, background.w, background.h), (112.0, 225.0, 74.0, 23.0));
            assert_eq!((background.u, background.v, background.uw, background.vh), (3.0, 5.0, 74.0, 23.0));
            assert_eq!(background.texture, "normal_tab_on");
            let text = list.quads.iter().find(|q| q.texture == "f").unwrap();
            assert_eq!((text.x, text.y), (136.0, 231.0));
            assert_eq!(text.color, if enabled { BUTTON_TEXT } else { BUTTON_TEXT_DISABLED }.rgba());
        }
    }

    #[test]
    fn tab_button_10002440_clips_long_text_on_all_edges_and_keeps_atlas_coordinates() {
        let mut button = button();
        button.rect = Rect::new(0, 0, 8, 7);
        button.text_offset = (0, 0);
        button.label = "VVV".into();
        let mut list = DrawList::default();
        button.paint(&mut list, (30, 40), &font());
        let text: Vec<_> = list.quads.iter().filter(|q| q.texture == "f").collect();
        assert_eq!(text.len(), 3);
        for quad in &text {
            assert!(quad.x >= 30.0 && quad.y >= 40.0);
            assert!(quad.x + quad.w <= 38.0 && quad.y + quad.h <= 47.0);
            assert_eq!((quad.y, quad.h, quad.v, quad.vh), (40.0, 7.0, 2.0, 7.0));
        }
        assert_eq!((text[0].w, text[0].u), (1.0, 383.0));
    }
}
