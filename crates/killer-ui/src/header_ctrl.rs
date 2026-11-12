use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::window::Rect;

pub const LABEL: Argb = Argb(0xFFE6DCBE);
pub const BLACK_TEXTURE: &str = "NWindow.BlackTexture";
pub const SINGLE_SOURCE: (i32, i32) = (0x43, 0xf);
pub const PART_SOURCE: (i32, i32) = (8, 0x13);
pub const PART_WIDTH: i32 = 8;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HeaderInfo {
    pub width: i32,
    pub height: i32,
    pub label: i32,
    pub field_22c: i32,
    pub field_230: i32,
    pub single: String,
    pub single_hover: String,
    pub parts: [String; 3],
    pub parts_hover: [String; 3],
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HeaderCtrl {
    pub rect: Rect,
    pub state: i32,
    pub single: Option<String>,
    pub single_hover: Option<String>,
    pub single_down: Option<String>,
    pub parts: [Option<String>; 3],
    pub parts_hover: [Option<String>; 3],
    pub parts_down: [Option<String>; 3],
    pub label: i32,
    pub field_22c: i32,
    pub field_230: i32,
    pub field_238: i32,
    pub field_23c: i32,
    pub notify_id: i32,
    pub disabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HeaderEvents {
    pub sound: Option<usize>,
    pub notify: Option<i32>,
}

fn over(name: &str) -> String {
    format!("{name}_over")
}

impl HeaderCtrl {
    pub fn new(rect: Rect) -> HeaderCtrl {
        HeaderCtrl { rect, notify_id: -1, field_22c: 1, ..Default::default() }
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        self.rect.x <= x && x <= self.rect.x + self.rect.w && self.rect.y <= y && y <= self.rect.y + self.rect.h
    }

    pub fn on_mouse_move(&mut self, x: i32, y: i32, left_down: bool) {
        self.state = if self.contains(x, y) {
            if left_down { 1 } else { 2 }
        } else {
            0
        };
    }

    pub fn on_lbutton_down(&mut self, x: i32, y: i32) -> HeaderEvents {
        if self.contains(x, y) {
            self.state = 1;
        }
        let sound = if self.disabled { crate::interface_sound::CLICK_FAILED } else { crate::interface_sound::CLICK };
        HeaderEvents { sound: Some(sound), notify: None }
    }

    pub fn on_lbutton_up(&mut self, x: i32, y: i32) -> HeaderEvents {
        let mut events = HeaderEvents::default();
        if self.contains(x, y) && self.field_22c != 0 && self.notify_id != -1 {
            events.notify = Some(self.notify_id);
        }
        self.state = if self.contains(x, y) { 2 } else { 0 };
        events
    }

    pub fn set_info(&mut self, info: &HeaderInfo) {
        self.label = info.label;
        self.field_22c = info.field_22c;
        self.field_230 = info.field_230;
        self.field_238 = info.width;
        self.field_23c = info.height;
        if !info.single.is_empty() {
            self.single = Some(info.single.clone());
        }
        if !info.single_hover.is_empty() {
            self.single_hover = Some(info.single_hover.clone());
        }
        if !info.single.is_empty() && self.single_down.is_none() {
            self.single_down = Some(over(&info.single));
        }
        for index in 0..3 {
            let name = &info.parts[index];
            if name.is_empty() {
                self.parts[index] = Some(BLACK_TEXTURE.to_string());
            } else {
                self.parts[index] = Some(name.clone());
                self.parts_down[index] = Some(over(name));
            }
            let hover = &info.parts_hover[index];
            self.parts_hover[index] = Some(if hover.is_empty() { BLACK_TEXTURE.to_string() } else { hover.clone() });
        }
    }

    fn selection(&self) -> (Option<&str>, [Option<&str>; 3]) {
        match self.state {
            0 => (self.single.as_deref(), std::array::from_fn(|i| self.parts[i].as_deref())),
            1 => (self.single_hover.as_deref(), std::array::from_fn(|i| self.parts_hover[i].as_deref())),
            2 => (
                self.single_down.as_deref().or(self.single.as_deref()),
                std::array::from_fn(|i| self.parts_down[i].as_deref().or(self.parts[i].as_deref())),
            ),
            _ => (None, [None; 3]),
        }
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), label: Option<&str>, font: &BitmapFont) {
        let x = origin.0 + self.rect.x;
        let y = origin.1 + self.rect.y;
        let (w, h) = (self.rect.w, self.rect.h);
        let mut body = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
        let (single, parts) = self.selection();
        if let Some(texture) = single {
            body.draw_texture(x, y, w, h, 0, 0, SINGLE_SOURCE.0, SINGLE_SOURCE.1, texture, 1.0);
        } else {
            let (sw, sh) = PART_SOURCE;
            if let Some(texture) = parts[0] {
                body.draw_texture(x, y, PART_WIDTH, h, 0, 0, sw, sh, texture, 1.0);
            }
            if let Some(texture) = parts[1] {
                body.draw_texture(x + PART_WIDTH, y, w - 16, h, 0, 0, sw, sh, texture, 1.0);
            }
            if let Some(texture) = parts[2] {
                body.draw_texture(x + w - PART_WIDTH, y, PART_WIDTH, h, 0, 0, sw, sh, texture, 1.0);
            }
        }
        if self.label >= 0
            && let Some(text) = label
        {
            let tw = font.measure(text) as i32;
            let th = font.line_height as i32;
            let tx = (w as f32 * 0.5 - (tw / 2) as f32) as i32;
            let ty = (h as f32 * 0.5 - (th / 2) as f32) as i32;
            body.draw_text(x + tx, y + ty + 1, LABEL, text, font);
        }
        list.append_clipped(body, Rect::new(x, y, w, h));
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

    fn info(single: &str) -> HeaderInfo {
        HeaderInfo {
            width: 70,
            height: 15,
            label: 4,
            single: single.into(),
            single_hover: "hov".into(),
            parts: ["l".into(), "m".into(), String::new()],
            parts_hover: ["lh".into(), String::new(), "rh".into()],
            ..Default::default()
        }
    }

    #[test]
    fn header_ctrl_10022060_derives_the_over_textures_and_black_fallbacks() {
        let mut header = HeaderCtrl::default();
        header.set_info(&info("tex"));
        assert_eq!(header.single.as_deref(), Some("tex"));
        assert_eq!(header.single_hover.as_deref(), Some("hov"));
        assert_eq!(header.single_down.as_deref(), Some("tex_over"));
        assert_eq!(header.parts[2].as_deref(), Some(BLACK_TEXTURE));
        assert_eq!(header.parts_down[0].as_deref(), Some("l_over"));
        assert_eq!(header.parts_down[2], None);
        assert_eq!(header.parts_hover[1].as_deref(), Some(BLACK_TEXTURE));
        assert_eq!((header.field_238, header.field_23c, header.label), (70, 15, 4));
        header.set_info(&info(""));
        assert_eq!(header.single.as_deref(), Some("tex"));
    }

    #[test]
    fn header_ctrl_100227e0_paints_the_single_texture_over_a_67_by_15_source() {
        let mut header = HeaderCtrl { rect: Rect::new(2, 3, 90, 19), ..Default::default() };
        header.set_info(&info("tex"));
        let mut list = DrawList::default();
        header.paint(&mut list, (10, 10), None, &font());
        assert_eq!(list.quads.len(), 1);
        let q = &list.quads[0];
        assert_eq!((q.texture.as_ref(), q.x, q.y, q.w, q.h, q.uw, q.vh), ("tex", 12.0, 13.0, 90.0, 19.0, 67.0, 15.0));
        header.state = 2;
        let mut list = DrawList::default();
        header.paint(&mut list, (0, 0), None, &font());
        assert_eq!(list.quads[0].texture, "tex_over");
        header.state = 1;
        let mut list = DrawList::default();
        header.paint(&mut list, (0, 0), None, &font());
        assert_eq!(list.quads[0].texture, "hov");
    }

    #[test]
    fn header_ctrl_100227e0_paints_three_slices_when_there_is_no_single_texture() {
        let mut header = HeaderCtrl { rect: Rect::new(0, 0, 90, 19), ..Default::default() };
        header.set_info(&info(""));
        let mut list = DrawList::default();
        header.paint(&mut list, (0, 0), None, &font());
        let geometry: Vec<_> = list.quads.iter().map(|q| (q.texture.to_string(), q.x, q.w, q.uw, q.vh)).collect();
        assert_eq!(geometry, vec![
            ("l".to_string(), 0.0, 8.0, 8.0, 19.0),
            ("m".to_string(), 8.0, 74.0, 8.0, 19.0),
            (BLACK_TEXTURE.to_string(), 82.0, 8.0, 8.0, 19.0),
        ]);
        header.state = 2;
        let mut list = DrawList::default();
        header.paint(&mut list, (0, 0), None, &font());
        assert_eq!(list.quads[0].texture, "l_over");
        assert_eq!(list.quads[2].texture, BLACK_TEXTURE);
    }

    #[test]
    fn header_ctrl_100227e0_centers_the_label_one_pixel_below_the_midline() {
        let font = font();
        let mut header = HeaderCtrl { rect: Rect::new(0, 0, 90, 19), ..Default::default() };
        header.set_info(&info("tex"));
        let mut list = DrawList::default();
        header.paint(&mut list, (0, 0), Some("Name"), &font);
        let text: Vec<_> = list.quads.iter().filter(|q| q.texture == "f").collect();
        let tw = font.measure("Name") as i32;
        assert_eq!(text[0].x, (45.0 - (tw / 2) as f32).trunc());
        assert_eq!(text[0].y, (9.5 - (font.line_height as i32 / 2) as f32).trunc() + 1.0);
        assert_eq!(text[0].color, LABEL.rgba());
        header.label = -1;
        let mut list = DrawList::default();
        header.paint(&mut list, (0, 0), Some("Name"), &font);
        assert!(list.quads.iter().all(|q| q.texture != "f"));
    }
}
