use crate::color::Argb;
use crate::draw::DrawList;
use crate::edit_box::Metrics;
use crate::font::BitmapFont;
use crate::scroll::{ScrollHit, ScrollWnd, SliderBar};
use crate::window::Rect;

pub const ROW_STEP: i32 = 17;
pub const MARGIN_X: i32 = 2;
pub const WRAP_MARGIN: i32 = 16 + 6;
pub const CONTINUATION_MARGIN: i32 = 12;
pub const CONTINUATION_INDENT: i32 = 12;
pub const TEXT_TOP: i32 = 9;
pub const WHEEL_STEP: i32 = 120;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListLine {
    pub text: String,
    pub color: Argb,
    pub indent: i32,
}

#[derive(Clone, Debug)]
pub struct TextList {
    pub id: u32,
    pub rect: Rect,
    pub max_lines: usize,
    pub visible: i32,
    pub bar: bool,
    pub scroll_bar: ScrollWnd,
    lines: Vec<ListLine>,
    first: i32,
}

pub fn wrap(text: &str, first_width: i32, continuation_width: i32, m: &dyn Metrics) -> Vec<(String, i32)> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut width = 0;
    let mut limit = first_width;
    let mut indent = 0;
    for ch in text.chars() {
        let unit = ch as u32;
        if unit == 0x0d {
            current.push(ch);
            continue;
        }
        if unit == 0x0a {
            current.push(ch);
            chunks.push((std::mem::take(&mut current), indent));
            width = 0;
            limit = continuation_width;
            indent = CONTINUATION_INDENT;
            continue;
        }
        if unit < 0x20 {
            continue;
        }
        let glyph = m.char_width(ch as u16);
        if width + glyph > limit {
            if current.is_empty() {
                return chunks;
            }
            chunks.push((std::mem::take(&mut current), indent));
            width = 0;
            limit = continuation_width;
            indent = CONTINUATION_INDENT;
        }
        current.push(ch);
        width += glyph;
    }
    if !current.is_empty() {
        chunks.push((current, indent));
    }
    chunks
}

impl TextList {
    pub fn new(id: u32, rect: Rect, max_lines: usize, visible: i32, bar: bool) -> Self {
        let slider_height = if rect.h < 30 { 0 } else { rect.h - 30 };
        let mut list = Self {
            id,
            rect,
            max_lines,
            visible,
            bar,
            scroll_bar: ScrollWnd { slider: SliderBar::new(slider_height), up_pressed: false, down_pressed: false },
            lines: Vec::new(),
            first: 0,
        };
        list.update_bar();
        list
    }

    fn update_bar(&mut self) {
        self.scroll_bar.slider.set_range(self.lines.len() as i32, self.visible);
        self.scroll_bar.slider.set_pos(self.first);
    }

    pub fn add_string(&mut self, text: &str, color: Argb, m: &dyn Metrics) {
        let first_width = self.rect.w - WRAP_MARGIN;
        for (chunk, indent) in wrap(text, first_width, first_width - CONTINUATION_MARGIN, m) {
            let line = ListLine { text: chunk, color, indent };
            if self.lines.len() < self.max_lines {
                self.lines.push(line);
                if self.lines.len() as i32 > self.visible {
                    self.first += 1;
                }
            } else {
                self.lines.remove(0);
                self.lines.push(line);
            }
            self.update_bar();
        }
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.first = 0;
        self.scroll_bar.slider.set_range(0, self.visible);
        self.scroll_bar.slider.set_pos(0);
    }

    pub fn lines(&self) -> &[ListLine] {
        &self.lines
    }

    pub fn first(&self) -> i32 {
        self.first
    }

    pub fn wheel(&mut self, delta: i32) {
        let notches = delta / WHEEL_STEP;
        let total = self.lines.len() as i32;
        if notches > 0 {
            self.first = (self.first - notches).max(0);
        } else {
            self.first = (self.first - notches).min((total - self.visible).max(0));
        }
        self.scroll_bar.slider.set_pos(self.first);
    }

    pub fn line_up(&mut self) {
        if self.lines.len() as i32 > self.visible {
            self.first = (self.first - 1).max(0);
            self.scroll_bar.slider.set_pos(self.first);
        }
    }

    pub fn line_down(&mut self) {
        let total = self.lines.len() as i32;
        if total > self.visible {
            self.first = (self.first + 1).min(total - self.visible);
            self.scroll_bar.slider.set_pos(self.first);
        }
    }

    pub fn set_first(&mut self, pos: i32) {
        self.first = pos;
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> bool {
        if !self.bar {
            return false;
        }
        match self.scroll_bar.hit(self.rect.w, self.rect.h, x - self.rect.x, y - self.rect.y) {
            Some(ScrollHit::Up) => {
                self.scroll_bar.up_pressed = true;
                self.line_up();
                true
            }
            Some(ScrollHit::Down) => {
                self.scroll_bar.down_pressed = true;
                self.line_down();
                true
            }
            Some(ScrollHit::Slider(y)) => {
                self.scroll_bar.slider.mouse_down(y);
                true
            }
            None => false,
        }
    }

    pub fn mouse_move(&mut self, y: i32) {
        if let Some(pos) = self.scroll_bar.slider.mouse_move(y - self.rect.y - crate::scroll::BUTTON_SIZE) {
            self.set_first(pos);
        }
    }

    pub fn mouse_up(&mut self) {
        self.scroll_bar.up_pressed = false;
        self.scroll_bar.down_pressed = false;
        self.scroll_bar.slider.mouse_up();
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont) {
        let (ox, oy) = (origin.0 + self.rect.x, origin.1 + self.rect.y);
        let start = list.quads.len();
        let half = font.line_height as i32 / 2;
        let first = self.first.max(0) as usize;
        for (row, line) in self.lines.iter().skip(first).take(self.visible.max(0) as usize).enumerate() {
            let y = oy - half + TEXT_TOP + row as i32 * ROW_STEP;
            let shown: String = line.text.chars().filter(|&c| c != '\r' && c != '\n').collect();
            if !shown.is_empty() {
                list.draw_text(ox + MARGIN_X + line.indent, y, line.color, &shown, font);
            }
        }
        list.clip_from(start, self.rect.offset(origin.0, origin.1));
        if self.bar {
            self.scroll_bar.paint(list, ox, oy, self.rect.w, self.rect.h);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixed;

    impl Metrics for Fixed {
        fn text_width(&self, text: &[u16]) -> i32 {
            text.len() as i32 * 7
        }

        fn char_width(&self, _: u16) -> i32 {
            7
        }
    }

    fn list() -> TextList {
        TextList::new(1, Rect::new(8, 37, 22 + 70, 312), 200, 3, true)
    }

    #[test]
    fn long_text_wraps_by_character_with_a_shorter_indented_continuation() {
        let chunks = wrap("abcdefghijkl", 70, 58, &Fixed);
        assert_eq!(chunks, vec![("abcdefghij".to_string(), 0), ("kl".to_string(), 12)]);
        let long = wrap(&"x".repeat(25), 70, 58, &Fixed);
        assert_eq!(long.iter().map(|(text, _)| text.len()).collect::<Vec<_>>(), vec![10, 8, 7]);
    }

    #[test]
    fn a_line_with_no_character_that_fits_ends_the_text_and_an_empty_text_adds_no_line() {
        assert!(wrap("", 70, 58, &Fixed).is_empty());
        assert!(wrap("\u{1}", 70, 58, &Fixed).is_empty());
        assert!(wrap("abc", 3, 3, &Fixed).is_empty());
        let mut list = list();
        list.add_string("", Argb(0xffffffff), &Fixed);
        assert!(list.lines().is_empty());
    }

    #[test]
    fn line_feed_closes_the_chunk_and_controls_below_space_are_dropped() {
        let chunks = wrap("ab\r\ncd\u{1}e", 70, 58, &Fixed);
        assert_eq!(chunks, vec![("ab\r\n".to_string(), 0), ("cde".to_string(), 12)]);
    }

    #[test]
    fn the_first_visible_line_follows_each_added_line_once_the_view_is_full() {
        let mut list = list();
        for text in ["a", "b", "c"] {
            list.add_string(text, Argb(0xffffffff), &Fixed);
        }
        assert_eq!(list.first(), 0);
        list.add_string("d", Argb(0xffffffff), &Fixed);
        assert_eq!(list.first(), 1);
        list.add_string("e", Argb(0xffffffff), &Fixed);
        assert_eq!(list.first(), 2);
    }

    #[test]
    fn the_oldest_line_is_discarded_when_the_list_is_full() {
        let mut list = TextList::new(1, Rect::new(0, 0, 92, 100), 2, 1, false);
        for text in ["a", "b", "c"] {
            list.add_string(text, Argb(0xffffffff), &Fixed);
        }
        let texts: Vec<_> = list.lines().iter().map(|line| line.text.as_str()).collect();
        assert_eq!(texts, vec!["b", "c"]);
    }

    #[test]
    fn clear_resets_the_window_into_the_list() {
        let mut list = list();
        for text in ["a", "b", "c", "d"] {
            list.add_string(text, Argb(0xffffffff), &Fixed);
        }
        list.clear();
        assert_eq!((list.lines().len(), list.first()), (0, 0));
    }

    #[test]
    fn the_wheel_moves_by_notches_and_clamps_to_the_ends() {
        let mut list = list();
        for text in ["a", "b", "c", "d", "e", "f"] {
            list.add_string(text, Argb(0xffffffff), &Fixed);
        }
        assert_eq!(list.first(), 3);
        list.wheel(240);
        assert_eq!(list.first(), 1);
        list.wheel(600);
        assert_eq!(list.first(), 0);
        list.wheel(-1200);
        assert_eq!(list.first(), 3);
    }

    #[test]
    fn line_up_and_down_only_act_when_there_are_more_lines_than_rows() {
        let mut list = list();
        list.add_string("a", Argb(0xffffffff), &Fixed);
        list.line_down();
        assert_eq!(list.first(), 0);
        for text in ["b", "c", "d", "e"] {
            list.add_string(text, Argb(0xffffffff), &Fixed);
        }
        list.line_up();
        assert_eq!(list.first(), 1);
        list.line_down();
        list.line_down();
        assert_eq!(list.first(), 2);
    }
}
