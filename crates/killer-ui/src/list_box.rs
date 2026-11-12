use crate::color::{Argb, TEXT};
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::window::Rect;

pub const SELECTION: Argb = Argb(0xFF646464);
pub const ROW_PADDING: i32 = 12;
pub const SELECTION_INSET: i32 = 18;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListBoxItem {
    pub text: String,
    pub color: Argb,
    pub param: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListBox {
    pub rect: Rect,
    pub background: Option<String>,
    pub items: Vec<ListBoxItem>,
    pub visible_rows: i32,
    pub first: i32,
    pub last_allowed: i32,
    pub selected: i32,
    pub text_origin: (i32, i32),
    pub text_height: i32,
    pub max_lines: i32,
    pub auto_scroll: bool,
}

impl ListBox {
    pub fn hit_test(&mut self, rx: i32, ry: i32) -> i32 {
        if rx < 0 || rx > self.rect.w || ry < 0 || ry > self.rect.h {
            return -1;
        }
        let pitch = self.row_pitch();
        let mut line = 0;
        let mut index = self.first;
        while index < self.visible_rows + self.first && index < self.last_allowed {
            if rx < self.rect.w && ry >= pitch * line + self.text_origin.1 && ry < (line + 1) * pitch + self.text_origin.1 {
                self.selected = index;
                return index;
            }
            index += 1;
            line += 1;
        }
        self.selected
    }

    pub fn mouse_down(&mut self, rx: i32, ry: i32) -> Option<i32> {
        self.selected = self.hit_test(rx, ry);
        (self.selected >= 0 && (self.selected as usize) < self.items.len()).then_some(self.selected)
    }

    pub fn add_string(&mut self, text: &str, color: Argb, param: i32) {
        if self.last_allowed < self.max_lines {
            self.last_allowed += 1;
            if self.auto_scroll && self.last_allowed > self.visible_rows {
                self.first += 1;
            }
        } else if !self.items.is_empty() {
            self.items.remove(0);
        }
        self.items.push(ListBoxItem { text: text.to_string(), color, param });
        if !self.items.is_empty() {
            self.last_allowed = self.items.len() as i32;
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.first = 0;
        self.last_allowed = 0;
        self.selected = -1;
    }

    pub fn item_param(&self, index: i32) -> i32 {
        self.items.get(index.max(0) as usize).map_or(-1, |item| item.param)
    }

    pub fn scroll_up(&mut self) {
        if self.last_allowed > self.visible_rows {
            self.first = (self.first - 1).max(0);
        }
    }

    pub fn scroll_down(&mut self) {
        if self.last_allowed > self.visible_rows {
            self.first = (self.first + 1).min(self.last_allowed - self.visible_rows);
        }
    }

    pub fn wheel(&mut self, delta: i32) {
        let notches = delta / 120;
        if notches > 0 {
            self.first = if self.first > notches { self.first - notches } else { 0 };
        } else if self.last_allowed >= self.visible_rows {
            self.first = if self.last_allowed - self.first + notches > self.visible_rows {
                self.first - notches
            } else {
                self.last_allowed - self.visible_rows
            };
        } else {
            self.first = if self.last_allowed - self.first + notches > self.visible_rows { self.first - notches } else { 0 };
        }
    }

    pub fn row_pitch(&self) -> i32 {
        self.text_height + ROW_PADDING
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont) {
        let x = origin.0 + self.rect.x;
        let y = origin.1 + self.rect.y;
        let mut body = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
        if let Some(texture) = &self.background {
            body.draw_texture(x, y, self.rect.w, self.rect.h, 0, 0, self.rect.w, self.rect.h, texture, 1.0);
        }
        let pitch = self.row_pitch();
        let mut index = self.first;
        let mut row = 0;
        while index < self.visible_rows + self.first && index <= self.last_allowed && (index as usize) < self.items.len() && index >= 0 {
            let item = &self.items[index as usize];
            let text_y = y + pitch * row + self.text_origin.1;
            if index == self.selected {
                body.draw_fill(x, y + pitch * row, self.rect.w - SELECTION_INSET, pitch, SELECTION);
                body.draw_text(x + self.text_origin.0, text_y, TEXT, &item.text, font);
            } else {
                body.draw_text(x + self.text_origin.0, text_y, item.color, &item.text, font);
            }
            index += 1;
            row += 1;
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

    fn list_box() -> ListBox {
        let items = (0..6).map(|i| ListBoxItem { text: format!("r{i}"), color: Argb(0xFF000000 | ((i as u32 + 1) * 0x101010)), param: i }).collect();
        ListBox {
            rect: Rect::new(5, 6, 120, 60),
            background: Some("back".into()),
            items,
            visible_rows: 3,
            first: 1,
            last_allowed: 10,
            selected: 2,
            text_origin: (4, 1),
            text_height: 13,
            max_lines: 100,
            auto_scroll: false,
        }
    }

    #[test]
    fn list_box_100208d0_paints_rows_at_a_pitch_of_text_height_plus_twelve() {
        let font = font();
        let mut list = DrawList::default();
        list_box().paint(&mut list, (10, 20), &font);
        let back: Vec<_> = list.quads.iter().filter(|q| q.texture == "back").collect();
        assert_eq!((back[0].x, back[0].y, back[0].w, back[0].h), (15.0, 26.0, 120.0, 60.0));
        let text: Vec<_> = list.quads.iter().filter(|q| q.texture == "f").collect();
        assert_eq!(text.len(), 6);
        assert_eq!((text[0].x, text[0].y), (19.0, 27.0));
        assert_eq!(text[0].color, Argb(0xFF202020).rgba());
        assert_eq!((text[2].x, text[2].y), (19.0, 27.0 + 25.0));
        assert_eq!(text[2].color, TEXT.rgba());
        assert_eq!(text[4].y, 27.0 + 50.0);
        assert_eq!(text[4].color, Argb(0xFF404040).rgba());
    }

    #[test]
    fn list_box_100208d0_highlights_only_the_selected_row_with_width_minus_eighteen() {
        let mut list = DrawList::default();
        list_box().paint(&mut list, (0, 0), &font());
        let fills: Vec<_> = list.quads.iter().filter(|q| q.texture != "f" && q.texture != "back").collect();
        assert_eq!(fills.len(), 1);
        assert_eq!((fills[0].x, fills[0].y, fills[0].w, fills[0].h), (5.0, 6.0 + 25.0, 102.0, 25.0));
        assert_eq!(fills[0].color, SELECTION.rgba());
    }

    #[test]
    fn list_box_100208d0_stops_at_the_last_allowed_index_and_the_item_count() {
        let mut bx = list_box();
        bx.last_allowed = 1;
        let mut list = DrawList::default();
        bx.paint(&mut list, (0, 0), &font());
        assert_eq!(list.quads.iter().filter(|q| q.texture == "f").count(), 2);
        let mut bx = list_box();
        bx.first = 5;
        bx.visible_rows = 9;
        let mut list = DrawList::default();
        bx.paint(&mut list, (0, 0), &font());
        assert_eq!(list.quads.iter().filter(|q| q.texture == "f").count(), 2);
    }

    #[test]
    fn list_box_100205c0_selects_the_row_under_the_point_and_keeps_the_old_one_between_rows() {
        let mut bx = list_box();
        assert_eq!(bx.hit_test(-1, 10), -1);
        assert_eq!(bx.hit_test(121, 10), -1);
        assert_eq!(bx.hit_test(10, 61), -1);
        assert_eq!(bx.mouse_down(10, 26), Some(2));
        assert_eq!(bx.hit_test(10, 0), 2);
        assert_eq!(bx.hit_test(120, 5), 2);
        bx.selected = -1;
        assert_eq!(bx.mouse_down(10, 59), Some(3));
        bx.items.truncate(3);
        assert_eq!(bx.mouse_down(10, 59), None);
    }

    #[test]
    fn list_box_10021260_drops_the_oldest_when_full_and_wheel_and_scroll_clamp() {
        let mut bx = list_box();
        bx.items.clear();
        bx.last_allowed = 0;
        bx.first = 0;
        bx.max_lines = 3;
        bx.auto_scroll = true;
        for i in 0..5 {
            bx.add_string(&format!("l{i}"), TEXT, i);
        }
        assert_eq!(bx.items.iter().map(|i| i.param).collect::<Vec<_>>(), vec![2, 3, 4]);
        assert_eq!(bx.item_param(0), 2);
        assert_eq!(bx.item_param(3), -1);
        bx.visible_rows = 2;
        bx.scroll_down();
        bx.scroll_down();
        assert_eq!(bx.first, 1);
        bx.scroll_up();
        bx.scroll_up();
        assert_eq!(bx.first, 0);
        bx.wheel(-120);
        assert_eq!(bx.first, 1);
        bx.wheel(240);
        assert_eq!(bx.first, 0);
        bx.clear();
        assert_eq!((bx.items.len(), bx.last_allowed, bx.selected), (0, 0, -1));
    }
}
