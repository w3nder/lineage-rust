use crate::color::{Argb, TEXT};
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::window::Rect;

pub const TEXTURES: [&str; 9] = [
    "L2UI.CandidateWnd.left_enable1",
    "L2UI.CandidateWnd.left_disable1",
    "L2UI.CandidateWnd.left_enable2",
    "L2UI.CandidateWnd.left_disable2",
    "L2UI.CandidateWnd.center1",
    "L2UI.CandidateWnd.center2",
    "L2UI.CandidateWnd.right1",
    "L2UI.CandidateWnd.right2",
    "NWindow.WhiteTexture",
];
pub const MIN_WIDTH: i32 = 70;
pub const KEY_PAGE_UP: u32 = 0x21;
pub const KEY_PAGE_DOWN: u32 = 0x22;
pub const KEY_UP: u32 = 0x26;
pub const KEY_DOWN: u32 = 0x28;
pub const LEFT_WIDTH: i32 = 11;
pub const RIGHT_WIDTH: i32 = 4;
pub const CENTER_TRIM: i32 = 15;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateBox {
    pub rect: Rect,
    pub visible: bool,
    pub page: i32,
    pub row_height: i32,
    pub items: Vec<String>,
    pub selected: i32,
    pub first: i32,
    pub last: i32,
    pub mode: i32,
    pub inset: i32,
    pub top_offset: i32,
    pub selected_color: Argb,
    pub textures: Vec<String>,
}

impl CandidateBox {
    pub fn new(rect: Rect) -> CandidateBox {
        CandidateBox {
            rect,
            visible: true,
            page: 0,
            row_height: 0,
            items: Vec::new(),
            selected: 0,
            first: 0,
            last: 0,
            mode: 0,
            inset: 0,
            top_offset: 0,
            selected_color: TEXT,
            textures: Vec::new(),
        }
    }

    pub fn on_create(&mut self) {
        self.textures = TEXTURES.iter().map(|name| name.to_string()).collect();
    }

    pub fn on_destroy(&mut self) {
        self.items.clear();
    }

    pub fn set_size(&mut self, width: i32, height: i32) {
        self.rect.w = if width < MIN_WIDTH { MIN_WIDTH } else { width };
        self.rect.h = height;
    }

    pub fn set_mode(&mut self, mode: i32) {
        self.mode = mode;
        if (2..=4).contains(&mode) {
            self.set_size(100, 0x96);
        } else {
            self.set_size(100, 0x87);
        }
    }

    pub fn add_item(&mut self, text: &str) {
        self.items.push(text.to_string());
        self.last += 1;
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.last = 0;
    }

    pub fn select_previous(&mut self) {
        if self.selected <= 0 {
            self.first = 0;
            return;
        }
        let next = self.selected - 1;
        self.selected = next;
        if next == 0 {
            self.first = 0;
        } else if self.page != 0 && next % self.page == self.page - 1 {
            self.first = next - self.page + 1;
        }
    }

    pub fn select_next(&mut self) {
        let next = self.selected + 1;
        if next >= self.last {
            return;
        }
        self.selected = next;
        if self.page != 0 && next % self.page == 0 {
            self.first = next;
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn on_key_down(&mut self, key: u32) {
        match key {
            KEY_PAGE_UP | KEY_PAGE_DOWN => {
                if self.visible {
                    self.hide();
                } else {
                    self.show();
                }
            }
            KEY_UP => self.select_previous(),
            KEY_DOWN => self.select_next(),
            _ => {}
        }
    }

    pub fn on_lbutton_up(&mut self) {
        if self.visible {
            self.hide();
        }
    }

    fn row_hit(&self, x: i32, y: i32, row: i32) -> bool {
        let left = self.inset;
        let top = self.row_height * row + self.top_offset;
        let width = (self.rect.w - 2 * self.inset) as f32;
        let height = self.row_height as f32;
        left <= x && x as f32 <= left as f32 + width && top <= y && y as f32 <= top as f32 + height
    }

    pub fn on_lbutton_down(&mut self, x: i32, y: i32) {
        let (x, y) = (x - self.rect.x, y - self.rect.y);
        let mut index = self.first;
        let mut row = 0;
        if self.page > 0 {
            while index < self.last {
                if self.row_hit(x, y, row) {
                    self.selected = index;
                    return;
                }
                index += 1;
                row += 1;
                if self.page + self.first <= index {
                    return;
                }
            }
        }
    }

    fn texture(&self, slot: usize) -> Option<&str> {
        self.textures.get(slot).map(String::as_str)
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont) {
        let x = origin.0 + self.rect.x;
        let y = origin.1 + self.rect.y;
        let mut body = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
        let width = self.rect.w;
        let rh = self.row_height;
        let strip = |body: &mut DrawList, slot: usize, from: i32, rows: i32| {
            if self.last < self.page
                && let Some(texture) = self.texture(slot)
            {
                let n = (self.page - rows) * rh;
                let top = rh * rows;
                body.draw_texture(x + 1, y + top + 1, 9, n, 0, from, 9, n, texture, 1.0);
            }
        };
        match self.mode {
            0..=2 => {
                if let Some(t) = self.texture(0) {
                    body.draw_texture(x, y, LEFT_WIDTH, 0x88, 0, 0, LEFT_WIDTH, 0x88, t, 1.0);
                }
                strip(&mut body, 1, rh * self.last, self.last);
                if let Some(t) = self.texture(4) {
                    body.draw_texture(x + LEFT_WIDTH, y, width - CENTER_TRIM, 0x88, 0, 0, 0xc, 0x88, t, 1.0);
                }
                if let Some(t) = self.texture(6) {
                    body.draw_texture(x + width - RIGHT_WIDTH, y, RIGHT_WIDTH, 0x88, 0, 0, RIGHT_WIDTH, 0x88, t, 1.0);
                }
            }
            3 | 4 => {
                if let Some(t) = self.texture(2) {
                    body.draw_texture(x, y, LEFT_WIDTH, 0x97, 0, 0, LEFT_WIDTH, 0x97, t, 1.0);
                }
                strip(&mut body, 3, rh * self.last, self.last);
                if let Some(t) = self.texture(5) {
                    body.draw_texture(x + LEFT_WIDTH, y, width - CENTER_TRIM, 0x97, 0, 0, 0xc, 0x88, t, 1.0);
                }
                if let Some(t) = self.texture(7) {
                    body.draw_texture(x + width - RIGHT_WIDTH, y, RIGHT_WIDTH, 0x97, 0, 0, RIGHT_WIDTH, 0x88, t, 1.0);
                }
            }
            5 => {
                if let Some(t) = self.texture(2) {
                    body.draw_texture(x, y, LEFT_WIDTH, 1, 0, 0, LEFT_WIDTH, 1, t, 1.0);
                    body.draw_texture(x, y + 1, LEFT_WIDTH, 0x87, 0, 0x10, LEFT_WIDTH, 0x87, t, 1.0);
                    body.draw_texture(x, y + 0x87, LEFT_WIDTH, 0xf, 0, 1, LEFT_WIDTH, 0xf, t, 1.0);
                }
                if self.last < self.page
                    && let Some(t) = self.texture(3)
                {
                    let n = (self.page - self.last - 1) * rh;
                    body.draw_texture(x + 1, y + rh * self.last + 1, 9, n, 0, (self.last + 1) * rh, 9, n, t, 1.0);
                    body.draw_texture(x + 1, y + (self.page - 1) * rh + 1, 9, rh, 0, 0, 9, rh, t, 1.0);
                }
                if let Some(t) = self.texture(5) {
                    body.draw_texture(x + LEFT_WIDTH, y, width - CENTER_TRIM, 0x97, 0, 0, 0xc, 0x97, t, 1.0);
                }
                if let Some(t) = self.texture(7) {
                    body.draw_texture(x + width - RIGHT_WIDTH, y, RIGHT_WIDTH, 0x97, 0, 0, RIGHT_WIDTH, 0x97, t, 1.0);
                }
            }
            _ => {}
        }
        let mut cursor = self.first;
        let mut row = 0;
        while cursor < self.page + self.first && cursor <= self.last {
            let index = self.first + cursor;
            let text = (index >= 0).then(|| self.items.get(index as usize)).flatten();
            if cursor == self.selected {
                let text_width = text.map(|t| font.measure(t) as i32).unwrap_or(0);
                if let Some(t) = self.texture(8) {
                    let line = (self.selected % self.page.max(1)) * rh;
                    body.draw_texture(x + 0xc, y + line, text_width + 3, rh, 0, 0, text_width, rh, t, 1.0);
                }
                if let Some(text) = text {
                    body.draw_text(x + 0xe, y + rh * row + 2, self.selected_color, text, font);
                }
            } else if let Some(text) = text {
                body.draw_text(x + 0xe, y + rh * row + 2, TEXT, text, font);
            }
            cursor += 1;
            row += 1;
        }
        list.append_clipped(body, Rect::new(x, y, width, self.rect.h));
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

    fn candidate() -> CandidateBox {
        let mut c = CandidateBox::new(Rect::new(10, 20, 100, 0x88));
        c.on_create();
        c.page = 9;
        c.row_height = 15;
        c.items = (0..12).map(|i| format!("c{i}")).collect();
        c.last = 12;
        c.inset = 12;
        c.top_offset = 2;
        c
    }

    #[test]
    fn candidate_box_10003dd0_loads_the_nine_candidate_wnd_textures_in_field_order() {
        let c = candidate();
        assert_eq!(c.textures.len(), 9);
        assert_eq!(c.textures[0], "L2UI.CandidateWnd.left_enable1");
        assert_eq!(c.textures[6], "L2UI.CandidateWnd.right1");
        assert_eq!(c.textures[8], "NWindow.WhiteTexture");
    }

    #[test]
    fn candidate_box_10004070_never_shrinks_below_seventy_pixels() {
        let mut c = candidate();
        c.set_size(69, 40);
        assert_eq!((c.rect.w, c.rect.h), (70, 40));
        c.set_size(70, 41);
        assert_eq!((c.rect.w, c.rect.h), (70, 41));
        c.set_size(200, 41);
        assert_eq!(c.rect.w, 200);
    }

    #[test]
    fn candidate_box_10003ea0_and_10003ef0_move_the_selection_and_page_by_modulo() {
        let mut c = candidate();
        c.selected = 8;
        c.select_next();
        assert_eq!((c.selected, c.first), (9, 9));
        c.select_previous();
        assert_eq!((c.selected, c.first), (8, 0));
        c.selected = 1;
        c.first = 5;
        c.select_previous();
        assert_eq!((c.selected, c.first), (0, 0));
        c.select_previous();
        assert_eq!((c.selected, c.first), (0, 0));
        c.selected = 11;
        c.select_next();
        assert_eq!(c.selected, 11);
    }

    #[test]
    fn candidate_box_10003f20_page_keys_toggle_visibility_and_up_down_move() {
        let mut c = candidate();
        c.on_key_down(KEY_PAGE_UP);
        assert!(!c.visible);
        c.on_key_down(KEY_PAGE_DOWN);
        assert!(c.visible);
        c.on_key_down(KEY_DOWN);
        assert_eq!(c.selected, 1);
        c.on_key_down(KEY_UP);
        assert_eq!(c.selected, 0);
        c.on_key_down(0x41);
        assert_eq!(c.selected, 0);
        c.on_lbutton_up();
        assert!(!c.visible);
    }

    #[test]
    fn candidate_box_10004790_selects_the_row_under_the_cursor_inside_the_inset() {
        let mut c = candidate();
        c.on_lbutton_down(10 + 20, 20 + 2 + 15 * 3 + 4);
        assert_eq!(c.selected, 3);
        c.on_lbutton_down(10 + 5, 20 + 2 + 15);
        assert_eq!(c.selected, 3);
        c.first = 9;
        c.on_lbutton_down(10 + 20, 20 + 2 + 1);
        assert_eq!(c.selected, 9);
    }

    #[test]
    fn candidate_box_10004230_paints_the_three_slices_and_the_rows() {
        let font = font();
        let mut c = candidate();
        let mut list = DrawList::default();
        c.paint(&mut list, (0, 0), &font);
        let left = &list.quads[0];
        assert_eq!((left.texture.as_ref(), left.x, left.w, left.h), ("L2UI.CandidateWnd.left_enable1", 10.0, 11.0, 136.0));
        let center = list.quads.iter().find(|q| q.texture == "L2UI.CandidateWnd.center1").unwrap();
        assert_eq!((center.x, center.w, center.uw), (21.0, 85.0, 12.0));
        let right = list.quads.iter().find(|q| q.texture == "L2UI.CandidateWnd.right1").unwrap();
        assert_eq!((right.x, right.w), (106.0, 4.0));
        let white = list.quads.iter().find(|q| q.texture == "NWindow.WhiteTexture").unwrap();
        assert_eq!((white.x, white.y, white.h), (22.0, 20.0, 15.0));
        let text: Vec<_> = list.quads.iter().filter(|q| q.texture == "f").collect();
        assert!(!text.is_empty());
        assert_eq!(text[0].x, 24.0);
        c.mode = 5;
        let mut list = DrawList::default();
        c.paint(&mut list, (0, 0), &font);
        assert_eq!(list.quads.iter().filter(|q| q.texture == "L2UI.CandidateWnd.left_enable2").count(), 3);
        c.mode = 9;
        let mut list = DrawList::default();
        c.paint(&mut list, (0, 0), &font);
        assert!(list.quads.iter().all(|q| q.texture == "f" || q.texture == "NWindow.WhiteTexture"));
    }

    #[test]
    fn candidate_box_10004eb0_drops_the_candidates() {
        let mut c = candidate();
        c.on_destroy();
        assert!(c.items.is_empty());
    }

    #[test]
    fn candidate_box_100040c0_sets_the_height_by_mode_and_the_list_counts_its_items() {
        let mut c = candidate();
        for (mode, h) in [(0, 0x87), (1, 0x87), (2, 0x96), (3, 0x96), (4, 0x96), (5, 0x87)] {
            c.set_mode(mode);
            assert_eq!((c.mode, c.rect.w, c.rect.h), (mode, 100, h));
        }
        c.clear();
        c.add_item("a");
        c.add_item("b");
        assert_eq!((c.items.len(), c.last), (2, 2));
        c.clear();
        assert_eq!((c.items.len(), c.last), (0, 0));
    }
}
