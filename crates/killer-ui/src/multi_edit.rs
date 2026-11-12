use crate::draw::DrawList;
use crate::edit_box::{
    BLINK_PERIOD_MS, EditInput, KEY_BACKSPACE, KEY_DELETE, KEY_DOWN, KEY_END, KEY_ENTER, KEY_HOME, KEY_LEFT, KEY_RIGHT, KEY_UP,
    Metrics, TEXT_COLOR, to_string,
};
use crate::font::BitmapFont;
use crate::scroll::{ScrollHit, ScrollWnd};
use crate::window::Rect;

pub const PADDING: i32 = 2;
pub const WRAP_MARGIN: i32 = 36;
pub const CHAR_MIN: u32 = 0x20;
pub const CHAR_MAX: u32 = 0x7e;
pub const WHEEL_STEP: i32 = 120;
pub const CARET_GLYPH: &str = "|";
pub const CARET_SHIFT: i32 = -3;
pub const ENTER_TEXT: &str = "\r\n";

#[derive(Clone, Debug)]
pub struct MultiEdit {
    pub id: u32,
    pub rect: Rect,
    pub focused: bool,
    pub blink: bool,
    pub scroll_bar: ScrollWnd,
    lines: Vec<Vec<u16>>,
    line: usize,
    col: usize,
    row: i32,
    x: i32,
    total_rows: i32,
    scroll: i32,
    visible: i32,
    line_height: i32,
    blink_ms: u32,
}

fn visible_len(line: &[u16]) -> usize {
    match line.last() {
        Some(&0x0a) if line.len() >= 2 && line[line.len() - 2] == 0x0d => line.len() - 2,
        Some(&0x0a) => line.len() - 1,
        _ => line.len(),
    }
}

impl MultiEdit {
    pub fn new(id: u32, rect: Rect, line_height: i32) -> Self {
        let slider_height = if rect.h < 30 { 0 } else { rect.h - 30 };
        let mut view = Self {
            id,
            rect,
            focused: false,
            blink: false,
            scroll_bar: ScrollWnd { slider: crate::scroll::SliderBar::new(slider_height), up_pressed: false, down_pressed: false },
            lines: vec![Vec::new()],
            line: 0,
            col: 0,
            row: 0,
            x: 0,
            total_rows: 1,
            scroll: 0,
            visible: (rect.h - PADDING) / (line_height + PADDING),
            line_height,
            blink_ms: 0,
        };
        view.update_bar();
        view
    }

    pub fn set_line_height(&mut self, line_height: i32) {
        self.line_height = line_height;
        self.visible = (self.rect.h - PADDING) / (line_height + PADDING);
        self.update_bar();
    }

    fn wrap_limit(&self) -> i32 {
        self.rect.w - WRAP_MARGIN
    }

    fn starts(&self, line: usize, m: &dyn Metrics) -> Vec<usize> {
        let limit = self.wrap_limit();
        let mut starts = vec![0];
        let mut acc = 0;
        for (j, &ch) in self.lines[line].iter().enumerate() {
            let width = m.char_width(ch);
            acc += width;
            if acc > limit && j > 0 {
                starts.push(j);
                acc = width;
            }
        }
        starts
    }

    fn rows_of(&self, line: usize, m: &dyn Metrics) -> i32 {
        self.starts(line, m).len() as i32
    }

    fn row_location(&self, row: i32, m: &dyn Metrics) -> Option<(usize, usize, usize)> {
        let mut first = 0;
        for line in 0..self.lines.len() {
            let starts = self.starts(line, m);
            let count = starts.len() as i32;
            if row < first + count {
                let index = (row - first) as usize;
                return Some((line, index, starts[index]));
            }
            first += count;
        }
        None
    }

    fn row_start(&self, row: i32, m: &dyn Metrics) -> i32 {
        self.row_location(row, m).map_or(-1, |(_, _, start)| start as i32)
    }

    fn next_row_start(&self, col: i32, line: usize, m: &dyn Metrics) -> i32 {
        if line >= self.lines.len() {
            return -1;
        }
        self.starts(line, m).into_iter().find(|&start| start as i32 > col).map_or(0, |start| start as i32)
    }

    fn cursor_x(&mut self, m: &dyn Metrics) {
        let limit = self.wrap_limit();
        let mut acc = 0;
        for &ch in self.lines[self.line].iter().take(self.col) {
            let width = m.char_width(ch);
            acc += width;
            if acc > limit {
                acc = width;
            }
        }
        self.x = acc;
    }

    fn recalc(&mut self, m: &dyn Metrics) {
        let mut total = 0;
        let mut before = 0;
        for line in 0..self.lines.len() {
            let rows = self.rows_of(line, m);
            if line < self.line {
                before += rows;
            }
            total += rows;
        }
        self.total_rows = total;
        let starts = self.starts(self.line, m);
        let within = starts.iter().filter(|&&start| start > 0 && start < self.col).count() as i32;
        self.row = before + within;
        self.cursor_x(m);
    }

    fn ensure_visible(&mut self) {
        let (v, s, r, t) = (self.visible, self.scroll, self.row, self.total_rows);
        let mut s = s;
        if s + v <= r {
            s = r - v + 1;
        }
        if s + v > t && s > 0 {
            s = t - v;
        }
        if r < s {
            s = r;
        }
        self.scroll = s;
    }

    fn update_bar(&mut self) {
        self.scroll_bar.slider.set_range(self.total_rows, self.visible);
        self.scroll_bar.slider.set_pos(self.scroll);
    }

    pub fn clear(&mut self) {
        self.line = 0;
        self.col = 0;
        self.lines = vec![Vec::new()];
        self.total_rows = 1;
        self.scroll = 0;
        self.x = 0;
        self.row = 0;
    }

    pub fn get_text(&self) -> Option<String> {
        let all: Vec<u16> = self.lines.iter().flatten().copied().collect();
        (!all.is_empty()).then(|| to_string(&all))
    }

    pub fn set_text(&mut self, text: &str, m: &dyn Metrics) {
        for unit in text.encode_utf16() {
            self.insert_char(unit, m);
        }
        self.ensure_visible();
        self.update_bar();
    }

    fn insert_char(&mut self, ch: u16, m: &dyn Metrics) {
        if ch == 0x0a {
            let line = &mut self.lines[self.line];
            let tail = line.split_off(self.col);
            line.push(0x0a);
            self.lines.insert(self.line + 1, tail);
            self.line += 1;
            self.col = 0;
            self.x = 0;
            self.row += 1;
            self.total_rows += 1;
        } else {
            let before = self.rows_of(self.line, m);
            self.lines[self.line].insert(self.col, ch);
            self.col += 1;
            let width = m.char_width(ch);
            self.x += width;
            if self.x > self.wrap_limit() {
                self.row += 1;
                self.x = width;
            }
            let after = self.rows_of(self.line, m);
            if after > before {
                self.total_rows += after - before;
            }
        }
        self.ensure_visible();
        self.update_bar();
    }

    pub fn char_input(&mut self, ch: u32, m: &dyn Metrics) -> bool {
        if !(CHAR_MIN..=CHAR_MAX).contains(&ch) {
            return true;
        }
        self.insert_char(ch as u16, m);
        true
    }

    pub fn input_key(&mut self, key: crate::chat::InputKey, m: &dyn Metrics) -> EditInput {
        use crate::chat::InputKey;
        let code = match key {
            InputKey::Char(c) => {
                self.char_input(c as u32, m);
                return EditInput::Handled;
            }
            InputKey::Backspace => KEY_BACKSPACE,
            InputKey::Left => KEY_LEFT,
            InputKey::Right => KEY_RIGHT,
            InputKey::Home => KEY_HOME,
            InputKey::End => KEY_END,
            InputKey::Delete => KEY_DELETE,
            InputKey::Up => KEY_UP,
            InputKey::Down => KEY_DOWN,
            InputKey::Enter => KEY_ENTER,
            InputKey::Escape => return EditInput::Escape,
        };
        self.key_down(code, m);
        EditInput::Handled
    }

    pub fn key_down(&mut self, vk: u32, m: &dyn Metrics) -> bool {
        match vk {
            KEY_BACKSPACE => self.backspace(m),
            KEY_ENTER => self.set_text(ENTER_TEXT, m),
            KEY_END => self.end(m),
            KEY_HOME => self.home(m),
            KEY_LEFT => self.left(m),
            KEY_UP => self.up(m),
            KEY_RIGHT => self.right(m),
            KEY_DOWN => self.down(m),
            KEY_DELETE => self.delete(m),
            _ => {}
        }
        true
    }

    fn finish_move(&mut self) {
        self.ensure_visible();
        self.update_bar();
    }

    fn backspace(&mut self, m: &dyn Metrics) {
        if self.col > 0 {
            self.lines[self.line].remove(self.col - 1);
            self.col -= 1;
        } else if self.line > 0 {
            let previous = self.lines[self.line - 1].clone();
            self.col = visible_len(&previous);
            let mut joined = previous;
            joined.pop();
            joined.extend_from_slice(&self.lines[self.line]);
            self.lines[self.line - 1] = joined;
            self.lines.remove(self.line);
            self.line -= 1;
        }
        self.recalc(m);
        self.finish_move();
    }

    fn delete(&mut self, m: &dyn Metrics) {
        if self.line + 1 < self.lines.len() || self.lines[self.line].len() > self.col {
            self.key_down(KEY_RIGHT, m);
            self.key_down(KEY_BACKSPACE, m);
        }
        self.finish_move();
    }

    fn home(&mut self, m: &dyn Metrics) {
        self.col = self.row_start(self.row, m).max(0) as usize;
        self.x = 0;
    }

    fn end(&mut self, m: &dyn Metrics) {
        let len = self.lines[self.line].len();
        if self.row + 1 < self.total_rows {
            let next = self.row_start(self.row + 1, m);
            self.col = if next != 0 { next.max(0) as usize } else { len.saturating_sub(1) };
        } else {
            self.col = len;
        }
        self.cursor_x(m);
    }

    fn left(&mut self, m: &dyn Metrics) {
        if self.col > 0 {
            self.col -= 1;
            if self.x > 0 {
                self.x -= m.char_width(self.lines[self.line][self.col]);
            } else {
                self.row -= 1;
                self.cursor_x(m);
            }
        } else if self.line > 0 {
            self.line -= 1;
            self.col = self.lines[self.line].len().saturating_sub(1);
            if self.row > 0 {
                self.row -= 1;
            }
            self.cursor_x(m);
        }
        self.finish_move();
    }

    fn right(&mut self, m: &dyn Metrics) {
        let len = self.lines[self.line].len();
        let limit = self.wrap_limit();
        if self.line + 1 < self.lines.len() {
            if self.col + 1 < len {
                let width = m.char_width(self.lines[self.line][self.col]);
                self.col += 1;
                self.x += width;
                if self.x > limit {
                    self.row += 1;
                    self.x = width;
                }
            } else {
                self.line += 1;
                self.row += 1;
                self.x = 0;
                self.col = 0;
            }
        } else if self.col < len {
            let width = m.char_width(self.lines[self.line][self.col]);
            self.col += 1;
            self.x += width;
            if self.x > limit {
                self.row += 1;
                self.x = width;
            }
        }
        self.finish_move();
    }

    fn up(&mut self, m: &dyn Metrics) {
        if self.row > 0 {
            let a = self.row_start(self.row, m);
            let b = self.row_start(self.row - 1, m);
            let col = self.col as i32;
            if a != 0 {
                self.col = if col - a < a - b { (col - a + b) as usize } else { a as usize };
            } else {
                let n = self.lines[self.line - 1].len() as i32;
                let last = (n - 1) - b;
                self.col = if last < col { (n - 1) as usize } else { (b + col) as usize };
                self.line -= 1;
            }
            self.row -= 1;
            if self.x > 0 {
                self.cursor_x(m);
            }
        }
        self.finish_move();
    }

    fn down(&mut self, m: &dyn Metrics) {
        if self.row < self.total_rows - 1 {
            let start = self.row_start(self.row, m);
            let n1 = self.next_row_start(start, self.line, m);
            let n2 = self.next_row_start(n1, self.line, m);
            let off = self.col as i32 - start;
            if n1 == 0 {
                self.line += 1;
            }
            let visible_now = visible_len(&self.lines[self.line]) as i32;
            let next_len = if self.row <= self.total_rows - 3 && n2 != 0 { n2 - n1 } else { visible_now - n1 };
            self.col = if off <= next_len { (n1 + off) as usize } else if n2 != 0 { n2 as usize } else { visible_now as usize };
            self.row += 1;
            if self.x > 0 {
                self.cursor_x(m);
            }
        }
        self.finish_move();
    }

    pub fn wheel(&mut self, delta: i32) {
        let notches = delta / WHEEL_STEP;
        let (total, visible) = (self.total_rows, self.visible);
        if notches > 0 {
            self.scroll = if self.scroll > notches { self.scroll - notches } else { 0 };
        } else {
            let room = total - self.scroll + notches;
            self.scroll = if room > visible {
                self.scroll - notches
            } else if total >= visible {
                total - visible
            } else {
                0
            };
        }
        self.scroll_bar.slider.set_pos(self.scroll);
    }

    pub fn scroll_up(&mut self) {
        if self.total_rows > self.visible {
            self.scroll = (self.scroll - 1).max(0);
            self.scroll_bar.slider.set_pos(self.scroll);
        }
    }

    pub fn scroll_down(&mut self) {
        if self.total_rows > self.visible {
            self.scroll = (self.scroll + 1).min(self.total_rows - self.visible);
            self.scroll_bar.slider.set_pos(self.scroll);
        }
    }

    pub fn set_scroll(&mut self, pos: i32) {
        self.scroll = pos;
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> bool {
        match self.scroll_bar.hit(self.rect.w, self.rect.h, x - self.rect.x, y - self.rect.y) {
            Some(ScrollHit::Up) => {
                self.scroll_bar.up_pressed = true;
                self.scroll_up();
                true
            }
            Some(ScrollHit::Down) => {
                self.scroll_bar.down_pressed = true;
                self.scroll_down();
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
            self.set_scroll(pos);
        }
    }

    pub fn mouse_up(&mut self) {
        self.scroll_bar.up_pressed = false;
        self.scroll_bar.down_pressed = false;
        self.scroll_bar.slider.mouse_up();
    }

    pub fn tick(&mut self, elapsed_ms: u32) {
        self.blink_ms += elapsed_ms;
        while self.blink_ms >= BLINK_PERIOD_MS {
            self.blink_ms -= BLINK_PERIOD_MS;
            self.blink = !self.blink;
        }
    }

    pub fn focus(&mut self) {
        self.focused = true;
    }

    pub fn kill_focus(&mut self) {
        self.focused = false;
    }

    pub fn cursor(&self) -> (usize, usize, i32, i32) {
        (self.line, self.col, self.row, self.x)
    }

    pub fn rows(&self) -> i32 {
        self.total_rows
    }

    pub fn scroll(&self) -> i32 {
        self.scroll
    }

    pub fn visible_rows(&self) -> i32 {
        self.visible
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), m: &dyn Metrics, font: &BitmapFont) {
        let (ox, oy) = (origin.0 + self.rect.x, origin.1 + self.rect.y);
        let start = list.quads.len();
        let mut y = oy + PADDING;
        let last = (self.scroll + self.visible).min(self.total_rows);
        for row in self.scroll..last {
            if let Some((line, _, from)) = self.row_location(row, m) {
                let text = &self.lines[line];
                let next = self.next_row_start(from as i32, line, m);
                let end = if next > 0 { next as usize } else { text.len() };
                let draw = |list: &mut DrawList, x: i32, part: &[u16]| -> i32 {
                    let shown: Vec<u16> = part.iter().copied().filter(|&c| c != 0x0d && c != 0x0a).collect();
                    if shown.is_empty() { 0 } else { list.draw_text(x, y, TEXT_COLOR, &to_string(&shown), font) }
                };
                if row == self.row {
                    let mut width = 0;
                    if self.x != 0 {
                        width = draw(list, ox + PADDING, &text[from..self.col.clamp(from, end)]);
                    }
                    if self.focused && self.blink {
                        list.draw_text(ox + PADDING + width + CARET_SHIFT, y, TEXT_COLOR, CARET_GLYPH, font);
                    }
                    draw(list, ox + PADDING + width, &text[self.col.clamp(from, end)..end]);
                } else {
                    draw(list, ox + PADDING, &text[from..end]);
                }
            }
            y += self.line_height + PADDING;
        }
        list.clip_from(start, Rect::new(ox, oy, self.rect.w, self.rect.h));
        self.scroll_bar.paint(list, ox, oy, self.rect.w, self.rect.h);
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

    fn edit() -> MultiEdit {
        MultiEdit::new(1, Rect::new(8, 241, 36 + 70, 124), 12)
    }

    fn type_str(edit: &mut MultiEdit, text: &str) {
        for c in text.chars() {
            edit.char_input(c as u32, &Fixed);
        }
    }

    #[test]
    fn the_visible_rows_follow_the_window_height_and_the_line_height() {
        assert_eq!(edit().visible_rows(), (124 - 2) / (12 + 2));
        assert_eq!(edit().rows(), 1);
    }

    #[test]
    fn typing_past_the_limit_opens_a_row_and_the_overflowing_char_starts_it() {
        let mut edit = edit();
        type_str(&mut edit, "abcdefghij");
        assert_eq!((edit.rows(), edit.cursor()), (1, (0, 10, 0, 70)));
        type_str(&mut edit, "k");
        assert_eq!((edit.rows(), edit.cursor()), (2, (0, 11, 1, 7)));
    }

    #[test]
    fn get_text_is_none_when_empty_and_joins_lines_with_their_terminators() {
        let mut edit = edit();
        assert_eq!(edit.get_text(), None);
        type_str(&mut edit, "ab");
        edit.key_down(KEY_ENTER, &Fixed);
        type_str(&mut edit, "cd");
        assert_eq!(edit.get_text().as_deref(), Some("ab\r\ncd"));
        assert_eq!(edit.cursor(), (1, 2, 1, 14));
        assert_eq!(edit.rows(), 2);
    }

    #[test]
    fn set_text_inserts_at_the_cursor_and_clear_keeps_one_empty_line_without_touching_the_bar() {
        let mut edit = edit();
        type_str(&mut edit, "ad");
        edit.key_down(KEY_LEFT, &Fixed);
        edit.set_text("bc", &Fixed);
        assert_eq!(edit.get_text().as_deref(), Some("abcd"));
        edit.clear();
        assert_eq!((edit.get_text(), edit.cursor(), edit.rows(), edit.scroll()), (None, (0, 0, 0, 0), 1, 0));
    }

    #[test]
    fn on_char_accepts_only_the_ascii_printable_range() {
        let mut edit = edit();
        for ch in [0x1f, 0x7f, 0x80, 0x81, 0xe3] {
            edit.char_input(ch, &Fixed);
        }
        assert_eq!(edit.get_text(), None);
        edit.char_input(0x20, &Fixed);
        edit.char_input(0x7e, &Fixed);
        assert_eq!(edit.get_text().as_deref(), Some(" ~"));
    }

    #[test]
    fn backspace_at_the_line_start_removes_only_the_lf_and_leaves_the_cr() {
        let mut edit = edit();
        type_str(&mut edit, "ab");
        edit.key_down(KEY_ENTER, &Fixed);
        type_str(&mut edit, "c");
        edit.key_down(KEY_LEFT, &Fixed);
        edit.key_down(KEY_BACKSPACE, &Fixed);
        assert_eq!(edit.get_text().as_deref(), Some("ab\rc"));
        assert_eq!(edit.cursor().0, 0);
        assert_eq!(edit.cursor().1, 2);
    }

    #[test]
    fn delete_needs_two_presses_to_remove_a_crlf() {
        let mut edit = edit();
        type_str(&mut edit, "ab");
        edit.key_down(KEY_ENTER, &Fixed);
        type_str(&mut edit, "c");
        edit.key_down(KEY_UP, &Fixed);
        edit.key_down(KEY_END, &Fixed);
        edit.key_down(KEY_DELETE, &Fixed);
        assert_eq!(edit.get_text().as_deref(), Some("ab\rc"));
        edit.key_down(KEY_DELETE, &Fixed);
        assert_eq!(edit.get_text().as_deref(), Some("abc"));
    }

    #[test]
    fn left_crosses_into_the_previous_line_before_the_lf_and_right_skips_it() {
        let mut edit = edit();
        type_str(&mut edit, "ab");
        edit.key_down(KEY_ENTER, &Fixed);
        edit.key_down(KEY_LEFT, &Fixed);
        assert_eq!(edit.cursor().0, 0);
        assert_eq!(edit.cursor().1, 3);
        edit.key_down(KEY_RIGHT, &Fixed);
        assert_eq!(edit.cursor().0, 1);
        assert_eq!(edit.cursor().1, 0);
    }

    #[test]
    fn up_and_down_keep_the_column_index_and_clamp_to_the_row() {
        let mut edit = edit();
        type_str(&mut edit, "abcdef");
        edit.key_down(KEY_ENTER, &Fixed);
        type_str(&mut edit, "xy");
        edit.key_down(KEY_UP, &Fixed);
        assert_eq!(edit.cursor().0, 0);
        assert_eq!(edit.cursor().1, 2);
        edit.key_down(KEY_DOWN, &Fixed);
        assert_eq!(edit.cursor().0, 1);
        assert_eq!(edit.cursor().1, 2);
    }

    #[test]
    fn the_wheel_scrolls_by_rows_and_clamps_to_the_range() {
        let mut edit = edit();
        for _ in 0..12 {
            edit.key_down(KEY_ENTER, &Fixed);
        }
        assert_eq!(edit.rows(), 13);
        edit.wheel(0);
        assert_eq!(edit.scroll(), edit.rows() - edit.visible_rows());
        edit.wheel(120);
        assert_eq!(edit.scroll(), edit.rows() - edit.visible_rows() - 1);
        edit.wheel(120 * 50);
        assert_eq!(edit.scroll(), 0);
        edit.wheel(-120);
        assert_eq!(edit.scroll(), 1);
        edit.scroll_up();
        assert_eq!(edit.scroll(), 0);
        edit.scroll_down();
        assert_eq!(edit.scroll(), 1);
    }

    #[test]
    fn the_cursor_follows_the_window_when_it_leaves_the_visible_rows() {
        let mut edit = edit();
        for _ in 0..12 {
            edit.key_down(KEY_ENTER, &Fixed);
        }
        assert_eq!(edit.cursor().2, 12);
        assert_eq!(edit.scroll(), 12 - edit.visible_rows() + 1);
        for _ in 0..12 {
            edit.key_down(KEY_UP, &Fixed);
        }
        assert_eq!(edit.scroll(), 0);
    }

    #[test]
    fn the_caret_blinks_every_half_second_even_without_focus_and_paints_only_with_focus() {
        let font = BitmapFont::new(
            "L2Font-e.SmallFont-e".into(),
            crate::font::Glyphs::parse(include_bytes!("../tests/fixtures/smallfont-e.gly")).unwrap(),
        );
        let metrics = crate::edit_box::FontMetrics(&font);
        let mut edit = edit();
        type_str(&mut edit, "a");
        assert!(!edit.blink);
        edit.tick(499);
        assert!(!edit.blink);
        edit.tick(1);
        assert!(edit.blink);
        let count = |edit: &MultiEdit| {
            let mut list = DrawList::default();
            edit.paint(&mut list, (0, 0), &metrics, &font);
            list.quads.len()
        };
        let without = count(&edit);
        edit.focus();
        assert!(count(&edit) > without);
        edit.tick(500);
        assert_eq!(count(&edit), without);
    }
}
