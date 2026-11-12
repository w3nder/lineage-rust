use crate::child_contract::{ChildContract, ChildKind};
use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::ime::{ImeHolder, Language, edit_box_paints_composition};
use crate::window::Rect;

pub const KIND_NOTIFY: u32 = 1;
pub const KIND_PASSWORD: u32 = 2;
pub const KIND_NUMERIC: u32 = 3;

pub const NOTICE_ENTER_EMPTY: u32 = 0;
pub const NOTICE_ENTER: u32 = 1;
pub const NOTICE_QUOTE: u32 = 3;
pub const NOTICE_UP: u32 = 4;
pub const NOTICE_DOWN: u32 = 5;

pub const KEY_BACKSPACE: u32 = 0x08;
pub const KEY_ENTER: u32 = 0x0d;
pub const KEY_SHIFT: u32 = 0x10;
pub const KEY_END: u32 = 0x23;
pub const KEY_HOME: u32 = 0x24;
pub const KEY_LEFT: u32 = 0x25;
pub const KEY_UP: u32 = 0x26;
pub const KEY_RIGHT: u32 = 0x27;
pub const KEY_DOWN: u32 = 0x28;
pub const KEY_DELETE: u32 = 0x2e;

pub const BLINK_TIMER_ID: u32 = 0xfa0d05;
pub const TEXT_COLOR: Argb = Argb(0xffdcdcdc);
pub const SELECTION_TEXTURE: &str = "NWindow.WhiteTexture";
pub const FOCUS_SKIN: [&str; 3] = ["L2UI_CH3.Etc.inputbox1", "L2UI_CH3.Etc.inputbox2", "L2UI_CH3.Etc.inputbox3"];
pub const DISABLED_SKIN: [&str; 3] = [
    "L2UI_CH3.Etc.inputbox1_disable",
    "L2UI_CH3.Etc.inputbox2_disable",
    "L2UI_CH3.Etc.inputbox3_disable",
];
pub const HIGHLIGHT_SKIN: &str = "L2UI.MacroWnd.Macro_Line_HighLight";
pub const HIGHLIGHT_WIDTH: i32 = 0xC1;
pub const HIGHLIGHT_ROW: i32 = 16;

const SKIN_ROW: i32 = 17;
const SKIN_CAP: i32 = 8;
const SKIN_CAP_FROM_RIGHT: f32 = 8.0;
const SKIN_MIDDLE_TRIM: f32 = 16.0;
const SCROLL_MARGIN: i32 = 12;
const END_MARGIN: f32 = 12.0;
const SELECT_MARGIN: f32 = 10.0;
const CHAR_MIN: u32 = 0x20;
const CHAR_MAX: u32 = 0x81;

pub const NATIVE_EXTRA_WIDTH: i32 = 0x108;
pub const HTML_EXTRA_WIDTH: i32 = 100;
const STATE_KIND: u16 = 0x214;
const STATE_CARET: u16 = 0x21c;
const STATE_ACCEPTS: u16 = 0x220;
const STATE_HIGHLIGHT: u16 = 0x228;
const STATE_BLINK: u16 = 0x224;
const STATE_EXTRA_WIDTH: u16 = 0x248;
const STATE_MAX_LEN: u16 = 0x254;
const STATE_FRESH: u16 = 0x2b8;
pub(crate) const BLINK_PERIOD_MS: u32 = 500;

pub trait Metrics {
    fn text_width(&self, text: &[u16]) -> i32;
    fn char_width(&self, ch: u16) -> i32;
}

pub struct FontMetrics<'a>(pub &'a BitmapFont);

pub fn to_string(text: &[u16]) -> String {
    String::from_utf16_lossy(text)
}

impl Metrics for FontMetrics<'_> {
    fn text_width(&self, text: &[u16]) -> i32 {
        self.0.measure(&to_string(text)) as i32
    }

    fn char_width(&self, ch: u16) -> i32 {
        self.0.measure(&to_string(&[ch])) as i32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditNotice {
    pub id: u32,
    pub code: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditBox {
    pub id: u32,
    pub kind: u32,
    pub rect: Rect,
    pub max_len: i32,
    pub extra_width: i32,
    pub enabled: bool,
    pub focused: bool,
    pub accepts: bool,
    pub caret_enabled: bool,
    pub caret_blink: bool,
    pub highlight: bool,
    left: Vec<u16>,
    right: Vec<u16>,
    shift: bool,
    sel_len: i32,
    sel_px: i32,
    caret_px: i32,
    scroll: i32,
    scroll_end: i32,
    client: i32,
    fresh: bool,
    blink_ms: u32,
    ime: ImeHolder,
    pub notices: Vec<EditNotice>,
    pub marked: bool,
}

pub fn group_thousands(digits: &[u16]) -> Vec<u16> {
    let count = digits.len();
    let mut out = vec![0u16; count + count.saturating_sub(1) / 3];
    let mut at = out.len();
    for (step, index) in (0..count).rev().enumerate() {
        if step % 3 == 0 && index + 1 < count {
            at -= 1;
            out[at] = u16::from(b',');
        }
        at -= 1;
        out[at] = digits[index];
    }
    out
}

pub fn numeric_tint(text: &[u16]) -> Argb {
    let commas = text.iter().filter(|&&c| c == u16::from(b',')).count() as i32;
    let digits = text.len() as i32 - commas;
    if digits < 5 {
        return TEXT_COLOR;
    }
    Argb(match (digits - 2) % 4 {
        0 => 0xffff80ff,
        1 => 0xffffff00,
        2 => 0xff00ff00,
        _ => 0xff00ffff,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditInput {
    Handled,
    Escape,
}

impl EditBox {
    pub fn input_key(&mut self, key: crate::chat::InputKey, m: &dyn Metrics) -> EditInput {
        use crate::chat::InputKey;
        let code = match key {
            InputKey::Char(c) => {
                self.char_input(c as u32, m);
                None
            }
            InputKey::Backspace => Some(KEY_BACKSPACE),
            InputKey::Left => Some(KEY_LEFT),
            InputKey::Right => Some(KEY_RIGHT),
            InputKey::Home => Some(KEY_HOME),
            InputKey::End => Some(KEY_END),
            InputKey::Delete => Some(KEY_DELETE),
            InputKey::Up => Some(KEY_UP),
            InputKey::Down => Some(KEY_DOWN),
            InputKey::Enter => Some(KEY_ENTER),
            InputKey::Escape => return EditInput::Escape,
        };
        if let Some(code) = code {
            self.key_down(code, m);
        }
        self.layout(m);
        EditInput::Handled
    }

    pub fn from_contract(contract: &ChildContract) -> Self {
        debug_assert_eq!(contract.kind, ChildKind::EditBox);
        Self {
            id: contract.id,
            kind: contract.field(STATE_KIND),
            rect: contract.rect,
            max_len: contract.field(STATE_MAX_LEN) as i32,
            extra_width: contract.field(STATE_EXTRA_WIDTH) as i32,
            enabled: true,
            focused: false,
            accepts: contract.field(STATE_ACCEPTS) != 0,
            caret_enabled: contract.field(STATE_CARET) != 0,
            caret_blink: contract.field(STATE_BLINK) != 0,
            highlight: contract.field(STATE_HIGHLIGHT) != 0,
            left: Vec::new(),
            right: Vec::new(),
            shift: false,
            sel_len: 0,
            sel_px: 0,
            caret_px: 0,
            scroll: 0,
            scroll_end: 0,
            client: 0,
            fresh: contract.field(STATE_FRESH) != 0,
            blink_ms: 0,
            ime: ImeHolder::create(Language::English, contract.field(STATE_KIND)),
            notices: Vec::new(),
            marked: false,
        }
    }

    pub fn native(id: u32, rect: Rect) -> Self {
        Self {
            id,
            kind: 0,
            rect,
            max_len: 0,
            extra_width: NATIVE_EXTRA_WIDTH,
            enabled: true,
            focused: false,
            accepts: true,
            caret_enabled: true,
            caret_blink: true,
            highlight: false,
            left: Vec::new(),
            right: Vec::new(),
            shift: false,
            sel_len: 0,
            sel_px: 0,
            caret_px: 0,
            scroll: 0,
            scroll_end: 0,
            client: 0,
            fresh: true,
            blink_ms: 0,
            ime: ImeHolder::create(Language::English, 0),
            notices: Vec::new(),
            marked: false,
        }
    }

    pub fn html(rect: Rect, kind: u32, max_len: i32) -> Self {
        let mut edit = Self::native(0, rect);
        edit.extra_width = HTML_EXTRA_WIDTH;
        edit.configure_html(rect, kind, max_len);
        edit
    }

    pub fn configure_html(&mut self, rect: Rect, kind: u32, max_len: i32) {
        self.rect = rect;
        if kind != self.kind {
            self.kind = kind;
            self.set_language(Language::English);
        }
        self.set_max_len(max_len);
    }

    pub fn set_language(&mut self, language: Language) {
        self.ime = ImeHolder::create(language, self.kind);
    }

    pub fn ime(&self) -> &ImeHolder {
        &self.ime
    }

    pub fn paints_composition(&self) -> bool {
        edit_box_paints_composition(Some(&self.ime))
    }

    pub fn left(&self) -> &[u16] {
        &self.left
    }

    pub fn right(&self) -> &[u16] {
        &self.right
    }

    pub fn scroll(&self) -> i32 {
        self.scroll
    }

    pub fn selection(&self) -> (i32, i32) {
        (self.sel_len, self.sel_px)
    }

    pub fn caret_px(&self) -> i32 {
        self.caret_px
    }

    fn len_left(&self) -> i32 {
        self.left.len() as i32
    }

    fn len_right(&self) -> i32 {
        self.right.len() as i32
    }

    fn notify(&mut self, code: u32) {
        self.notices.push(EditNotice { id: self.id, code });
    }

    fn update_accepts(&mut self) {
        if self.max_len != 0 {
            self.accepts = self.len_left() + self.len_right() < self.max_len;
        }
    }

    pub fn set_max_len(&mut self, value: i32) {
        if value > 0 {
            self.max_len = value;
        }
    }

    pub fn clear_left(&mut self) {
        self.left.clear();
    }

    pub fn clear(&mut self) {
        self.left.clear();
        self.right.clear();
        self.scroll = 0;
        self.accepts = true;
    }

    pub fn get_text(&self) -> Option<String> {
        let mut text: Vec<u16> = self.left.iter().chain(self.right.iter()).copied().collect();
        if text.is_empty() {
            return None;
        }
        if self.kind == KIND_NUMERIC {
            let mut index = 0;
            while index < text.len() {
                if text[index] == u16::from(b',') {
                    text.remove(index);
                }
                index += 1;
            }
        }
        Some(to_string(&text))
    }

    pub fn set_text(&mut self, text: &str) {
        self.left = text.encode_utf16().collect();
        self.right.clear();
    }

    pub fn toggle_blink(&mut self, timer: u32) -> bool {
        if timer == BLINK_TIMER_ID {
            self.caret_blink = !self.caret_blink;
        }
        true
    }

    pub fn key_down_shift(&mut self) {
        self.shift = true;
    }

    pub fn key_up(&mut self, key: u32) -> bool {
        if key == KEY_SHIFT {
            self.shift = false;
        }
        true
    }

    pub fn mouse_up(&mut self) -> bool {
        self.shift = false;
        self.fresh = false;
        true
    }

    pub fn focus(&mut self) {
        self.focused = true;
    }

    pub fn kill_focus(&mut self) {
        self.fresh = true;
        let right = std::mem::take(&mut self.right);
        self.left.extend(right);
        self.sel_len = 0;
        self.sel_px = 0;
        self.focused = false;
    }

    pub fn tick(&mut self, elapsed_ms: u32) {
        self.blink_ms += elapsed_ms;
        while self.blink_ms >= BLINK_PERIOD_MS {
            self.blink_ms -= BLINK_PERIOD_MS;
            self.toggle_blink(BLINK_TIMER_ID);
        }
    }

    pub fn insert_text(&mut self, text: &str, append: bool, m: &dyn Metrics) {
        if !self.accepts {
            return;
        }
        let units: Vec<u16> = text.encode_utf16().collect();
        if append {
            self.left.extend(units);
        } else {
            let old = std::mem::replace(&mut self.left, units);
            self.left.extend(old);
        }
        if self.kind == KIND_NUMERIC {
            self.reformat_numeric();
        }
        self.fit_view(m);
        self.update_accepts();
    }

    fn fit_view(&mut self, m: &dyn Metrics) {
        while self.len_left() >= self.scroll {
            let start = (self.scroll.max(0) as usize).min(self.left.len());
            if m.text_width(&self.left[start..]) <= self.client_width() - SCROLL_MARGIN {
                break;
            }
            self.scroll += 1;
        }
    }

    pub fn delete_char(&mut self, index: i32, m: &dyn Metrics) {
        let left = self.len_left();
        if index < left {
            if index >= 0 {
                self.left.remove(index as usize);
            }
            self.fit_view(m);
        } else if index < left + self.len_right() {
            self.right.remove((index - left) as usize);
        }
        self.update_accepts();
    }

    fn select_left_all(&mut self, m: &dyn Metrics) {
        self.sel_len = -self.len_left();
        self.sel_px = m.text_width(&self.left);
    }

    pub fn mouse_down(&mut self, x: i32, m: &dyn Metrics) -> bool {
        if self.fresh {
            self.select_left_all(m);
            return true;
        }
        let local = i32::from((x - self.rect.x) as u16);
        let caret = self.caret_px;
        if local < caret {
            let left = self.left.clone();
            let mut at = left.len();
            let mut acc = 0;
            loop {
                if at == 0 {
                    break;
                }
                at -= 1;
                acc += m.char_width(left[at]);
                self.key_down(KEY_LEFT, m);
                if acc >= caret - local {
                    break;
                }
            }
        } else if local > caret {
            let right = self.right.clone();
            let mut index = 0;
            let mut acc = 0;
            while index < right.len() {
                acc += m.char_width(right[index]);
                index += 1;
                self.key_down(KEY_RIGHT, m);
                if acc >= local - caret {
                    break;
                }
            }
        }
        self.sel_px = 0;
        self.sel_len = 0;
        self.shift = true;
        true
    }

    pub fn mouse_move(&mut self, x: i32, m: &dyn Metrics) -> bool {
        if !self.shift {
            return true;
        }
        self.sel_px = 0;
        self.sel_len = 0;
        let local = i32::from(x as u16) - self.rect.x;
        let caret = self.caret_px;
        if local < caret {
            let mut at = self.left.len();
            let target = caret - local;
            while at > 0 {
                at -= 1;
                self.sel_px += m.char_width(self.left[at]);
                self.sel_len -= 1;
                if self.sel_px >= target {
                    break;
                }
            }
        } else if local > caret {
            let mut target = local - caret;
            if caret == 0 {
                for index in 0..self.scroll.max(0) as usize {
                    if let Some(&ch) = self.right.get(index) {
                        target += m.char_width(ch);
                    }
                }
                target -= m.text_width(&self.left);
            }
            let mut index = 0;
            while index < self.right.len() {
                self.sel_px += m.char_width(self.right[index]);
                index += 1;
                self.sel_len += 1;
                if self.sel_px >= target {
                    break;
                }
            }
        }
        let scroll = (self.scroll.max(0) as usize).min(self.left.len() + self.right.len());
        let joined: Vec<u16> = self.left.iter().chain(self.right.iter()).copied().collect();
        let edge = self.client_width() as f32 - SELECT_MARGIN;
        let width = m.text_width(&joined[scroll..]);
        if local as f32 <= edge || width as f32 <= edge {
            if local < 10 && self.scroll > 0 {
                self.scroll -= 1;
            }
        } else {
            self.scroll += 1;
        }
        true
    }

    fn client_width(&self) -> i32 {
        self.rect.w
    }

    fn reformat_numeric(&mut self) {
        let old = self.left.clone();
        let old_len = old.len();
        let Some(text) = self.get_text() else { return };
        let digits: Vec<u16> = text.encode_utf16().collect();
        let formatted = group_thousands(&digits);
        let comma = u16::from(b',');
        let new_commas = formatted.iter().take(old_len).filter(|&&c| c == comma).count() as i32;
        let old_commas = old.iter().filter(|&&c| c == comma).count() as i32;
        let index = old_len as i32 + (new_commas - old_commas);
        self.left = formatted;
        if index < self.len_left() {
            let at = index.max(0) as usize;
            self.right = self.left[at..].to_vec();
            self.left.truncate(at);
        }
        self.update_accepts();
    }

    pub fn char_input(&mut self, ch: u32, m: &dyn Metrics) -> bool {
        if !self.enabled
            || (ch != KEY_BACKSPACE && !self.accepts)
            || (self.kind == KIND_NUMERIC && !(0x30..=0x39).contains(&ch))
        {
            return true;
        }
        if (CHAR_MIN..=CHAR_MAX).contains(&ch) {
            self.delete_selection(m);
            let unit = ch as u16;
            if self.max_len != 0 && self.max_len < self.len_left() + self.len_right() + 1 {
                return true;
            }
            if self.left.is_empty() {
                self.left = vec![unit];
                if ch == 0x22 && self.kind == KIND_NOTIFY {
                    self.notify(NOTICE_QUOTE);
                }
            } else {
                self.left.push(unit);
            }
            if self.kind == KIND_NUMERIC {
                self.reformat_numeric();
            }
            let tail = &self.left[(self.scroll.max(0) as usize).min(self.left.len())..];
            if self.client_width() < m.text_width(tail) {
                let first_two: Vec<u16> = self
                    .left
                    .iter()
                    .skip(self.scroll.max(0) as usize)
                    .take(2)
                    .copied()
                    .collect();
                let first = m.text_width(&first_two);
                let added = m.text_width(&[unit]);
                self.scroll += if added < first { 1 } else { 2 };
                self.scroll_end = self.scroll;
            }
        }
        self.update_accepts();
        true
    }

    fn delete_selection(&mut self, m: &dyn Metrics) -> bool {
        if self.sel_len < 0 {
            let count = -self.sel_len;
            self.sel_len = 0;
            self.sel_px = 0;
            for _ in 0..count {
                self.key_down(KEY_BACKSPACE, m);
            }
            true
        } else if self.sel_len > 0 {
            let count = self.sel_len;
            self.sel_len = 0;
            self.sel_px = 0;
            for _ in 0..count {
                self.key_down(KEY_DELETE, m);
            }
            true
        } else {
            false
        }
    }

    pub fn key_down(&mut self, key: u32, m: &dyn Metrics) -> bool {
        if !self.enabled {
            return false;
        }
        match key {
            KEY_BACKSPACE => self.backspace(m),
            KEY_ENTER => self.enter(),
            KEY_SHIFT => self.shift = true,
            KEY_END => self.end(m),
            KEY_HOME => self.home(m),
            KEY_LEFT => self.left_arrow(m),
            KEY_UP if self.kind == KIND_NOTIFY => self.notify(NOTICE_UP),
            KEY_RIGHT => self.right_arrow(m),
            KEY_DOWN if self.kind == KIND_NOTIFY => self.notify(NOTICE_DOWN),
            KEY_DELETE => self.delete(m),
            _ => {}
        }
        true
    }

    fn enter(&mut self) {
        self.scroll = 0;
        self.scroll_end = 0;
        if self.left.is_empty() && self.right.is_empty() {
            self.notify(NOTICE_ENTER_EMPTY);
        } else {
            self.notify(NOTICE_ENTER);
        }
    }

    fn backspace(&mut self, m: &dyn Metrics) {
        if self.delete_selection(m) || self.left.is_empty() {
            return;
        }
        let comma = u16::from(b',');
        if self.kind == KIND_NUMERIC && self.left.last() == Some(&comma) {
            if let Some(last) = self.left.pop() {
                self.right.insert(0, last);
            }
            self.key_down(KEY_BACKSPACE, m);
            return;
        }
        if self.scroll > 0 {
            self.scroll_for_backspace(m);
        }
        self.finish_backspace(m);
    }

    fn scroll_for_backspace(&mut self, m: &dyn Metrics) {
        let len = self.left.len();
        let scroll = (self.scroll as usize).min(len);
        let deleted = m.text_width(&self.left[len - 1..]);
        let prev_one = if scroll < 1 { 0 } else { m.text_width(&self.left[scroll - 1..scroll]) };
        let prev_two = if scroll < 2 { 0 } else { m.text_width(&self.left[scroll - 2..scroll]) };
        let tail_fills = m.text_width(&self.left[scroll..]) >= self.client_width() - SCROLL_MARGIN;
        let by = if deleted < 12 {
            if prev_one > 11 && tail_fills {
                return;
            }
            self.single_step(deleted, prev_one, tail_fills)
        } else if !(7..=12).contains(&prev_two) {
            self.single_step(deleted, prev_one, tail_fills)
        } else {
            2
        };
        self.scroll -= by;
        self.scroll_end -= by;
    }

    fn single_step(&self, deleted: i32, prev_one: i32, tail_fills: bool) -> i32 {
        if deleted < prev_one && tail_fills { 0 } else { 1 }
    }

    fn finish_backspace(&mut self, m: &dyn Metrics) {
        self.left.pop();
        self.accepts = true;
        let total = m.text_width(&self.left) + m.text_width(&self.right);
        if total <= self.client_width() {
            self.scroll = 0;
        }
        if self.kind == KIND_NUMERIC && self.len_left() + self.len_right() > 1 {
            self.reformat_numeric();
        }
    }

    fn delete(&mut self, m: &dyn Metrics) {
        if self.delete_selection(m) || self.shift {
            return;
        }
        if !self.right.is_empty() {
            self.key_down(KEY_RIGHT, m);
            self.key_down(KEY_BACKSPACE, m);
            if self.kind == KIND_NUMERIC && self.right.first() == Some(&u16::from(b',')) {
                self.key_down(KEY_RIGHT, m);
            }
        }
        self.update_accepts();
    }

    fn end(&mut self, m: &dyn Metrics) {
        if self.shift {
            self.sel_px = m.text_width(&self.right);
            self.sel_len = -self.len_right();
        }
        let right = std::mem::take(&mut self.right);
        self.left.extend(right);
        self.scroll = self.scroll_end;
    }

    fn home(&mut self, m: &dyn Metrics) {
        if self.shift {
            self.sel_px = m.text_width(&self.left);
            self.sel_len = self.len_left();
        }
        let mut joined = std::mem::take(&mut self.left);
        joined.extend(std::mem::take(&mut self.right));
        self.right = joined;
        self.scroll = 0;
    }

    fn left_arrow(&mut self, m: &dyn Metrics) {
        if !self.shift {
            self.sel_px = 0;
            self.sel_len = 0;
            if self.left.is_empty() {
                return;
            }
            if self.caret_px < 13 {
                self.scroll -= 1;
            }
            if self.scroll < 0 {
                self.scroll = 0;
            }
            if let Some(last) = self.left.pop() {
                self.right.insert(0, last);
            }
            if self.kind == KIND_NUMERIC && self.right.first() == Some(&u16::from(b',')) {
                self.key_down(KEY_LEFT, m);
            }
            return;
        }
        if self.sel_len < 1 {
            if -self.sel_len < self.len_left() {
                let index = (self.len_left() + self.sel_len - 1) as usize;
                self.sel_px += m.char_width(self.left[index]);
                self.sel_len -= 1;
                if self.caret_px <= self.sel_px {
                    self.scroll -= 1;
                    if self.scroll < 0 {
                        self.scroll = 0;
                    }
                }
            }
        } else {
            let index = (self.sel_len - 1) as usize;
            if let Some(&ch) = self.right.get(index) {
                self.sel_px -= m.char_width(ch);
            }
            self.sel_len -= 1;
        }
    }

    fn right_arrow(&mut self, m: &dyn Metrics) {
        if !self.shift {
            self.sel_px = 0;
            self.sel_len = 0;
            if self.right.is_empty() {
                return;
            }
            if self.caret_px as f32 >= self.client_width() as f32 - END_MARGIN {
                self.scroll += 1;
            }
            let first = self.right.remove(0);
            self.left.push(first);
            if self.kind == KIND_NUMERIC && self.right.first() == Some(&u16::from(b',')) {
                self.key_down(KEY_RIGHT, m);
            }
            return;
        }
        if self.sel_len < 0 {
            if -self.sel_len <= self.len_left() {
                let index = (self.len_left() + self.sel_len) as usize;
                self.sel_px -= m.char_width(self.left[index]);
                self.sel_len += 1;
            }
        } else if self.sel_len < self.len_right() {
            let ch = self.right[self.sel_len as usize];
            self.sel_px += m.char_width(ch);
            self.sel_len += 1;
            let mut joined = self.left.clone();
            joined.extend(self.right.iter().copied());
            joined.truncate((self.scroll.max(0) as usize).min(joined.len()));
            let hidden = m.text_width(&joined);
            let visible = m.text_width(&self.left);
            let edge = (self.caret_px - hidden + self.sel_px + visible) as f32;
            if edge >= self.client_width() as f32 - SELECT_MARGIN {
                self.scroll += 1;
                if self.scroll < 0 {
                    self.scroll = 0;
                }
            }
        }
    }

    pub fn layout(&mut self, m: &dyn Metrics) {
        let width = self.client_width();
        self.client = if self.extra_width > 0 { width + self.extra_width } else { width };
        let total = m.text_width(&self.left) + m.text_width(&self.right);
        if self.max_len == 0 {
            self.accepts = total < self.client - SCROLL_MARGIN;
        }
        let stars = self.kind == KIND_PASSWORD;
        let tail_start = (self.scroll.max(0) as usize).min(self.left.len());
        let shown = if stars {
            m.text_width(&vec![u16::from(b'*'); self.left.len()])
        } else if self.left.len() > tail_start {
            m.text_width(&self.left[tail_start..])
        } else {
            0
        };
        self.caret_px = shown;
        if self.sel_len == 0 {
            let whole = m.text_width(&self.left);
            if (self.caret_px as f32) < width as f32 {
                let hidden = whole - self.caret_px;
                self.scroll = self.scan_scroll(m, 0, hidden, self.scroll);
            } else {
                let overflow = self.caret_px - width;
                self.scroll = self.scan_scroll(m, self.scroll.max(0), overflow, self.scroll);
            }
        }
    }

    fn scan_scroll(&self, m: &dyn Metrics, start: i32, target: i32, current: i32) -> i32 {
        let mut index = start;
        let mut acc = 0;
        let len = self.len_left();
        while index < len {
            let step = m.char_width(self.left[index as usize]);
            if acc >= target {
                return index;
            }
            acc += step;
            index += 1;
        }
        if acc >= target { index } else { current }
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), m: &dyn Metrics, font: &BitmapFont) {
        let (x, y) = (origin.0 + self.rect.x, origin.1 + self.rect.y);
        let start = list.quads.len();
        let skin = if self.focused { FOCUS_SKIN } else { DISABLED_SKIN };
        let (w, h) = (self.rect.w, self.rect.h);
        list.draw_texture(x, y, SKIN_CAP, h, 0, 0, SKIN_CAP, SKIN_ROW, skin[0], 1.0);
        list.draw_texture(x + SKIN_CAP, y, (w as f32 - SKIN_MIDDLE_TRIM) as i32, h, 0, 0, SKIN_CAP, SKIN_ROW, skin[1], 1.0);
        list.draw_texture(x + (w as f32 - SKIN_CAP_FROM_RIGHT) as i32, y, SKIN_CAP, h, 0, 0, SKIN_CAP, SKIN_ROW, skin[2], 1.0);
        if self.marked || self.highlight {
            list.draw_texture(x, y, w, h, 0, 0, HIGHLIGHT_WIDTH, HIGHLIGHT_ROW, HIGHLIGHT_SKIN, 1.0);
        }
        let tint = if self.kind == KIND_NUMERIC {
            let joined: Vec<u16> = self.left.iter().chain(self.right.iter()).copied().collect();
            numeric_tint(&joined)
        } else {
            TEXT_COLOR
        };
        let left_len = self.left.len();
        let scroll = self.scroll.max(0) as usize;
        let left_shown = if self.kind == KIND_PASSWORD {
            "*".repeat(left_len)
        } else if left_len > scroll {
            to_string(&self.left[scroll..])
        } else {
            String::new()
        };
        let drawn = if left_shown.is_empty() { 0 } else { list.draw_text(x + 2, y + 2, tint, &left_shown, font) - (x + 2) };
        let right_shown = if self.kind == KIND_PASSWORD {
            "*".repeat(self.right.len())
        } else if left_len < scroll {
            to_string(&self.right[(scroll - left_len).min(self.right.len())..])
        } else {
            to_string(&self.right)
        };
        let color = if self.kind == KIND_PASSWORD { TEXT_COLOR } else { tint };
        if !right_shown.is_empty() {
            list.draw_text(x + 2 + drawn, y + 2, color, &right_shown, font);
        }
        if self.caret_enabled && self.focused && self.caret_blink {
            let at = if self.kind == KIND_PASSWORD { self.caret_px } else { self.caret_px - 1 };
            list.draw_text(x + 2 + at.max(0) - 2, y + 2, TEXT_COLOR, "|", font);
        }
        if self.sel_len < 0 {
            list.draw_texture(x + self.caret_px - self.sel_px + 2, y + 2, self.sel_px, 12, 0, 0, 1, 1, SELECTION_TEXTURE, 0.5);
        } else if self.sel_len > 0 {
            let mut acc = 0;
            for index in 0..self.sel_len {
                let visible = left_len as i32 + index >= self.scroll;
                if visible && (index as usize) < self.right.len() {
                    let width = m.char_width(self.right[index as usize]);
                    list.draw_texture(x + self.caret_px + 1 + acc, y + 2, width, 12, 0, 0, 1, 1, SELECTION_TEXTURE, 0.5);
                    acc += width;
                }
            }
        }
        list.clip_from(start, Rect::new(x, y, self.rect.w, self.rect.h));
    }
}

impl EditBox {
    pub fn paint_html(&mut self, list: &mut DrawList, origin: (i32, i32), clip: Rect, m: &dyn Metrics, font: &BitmapFont) {
        let (x, y) = (origin.0 + self.rect.x, origin.1 + self.rect.y);
        let (w, h) = (self.rect.w, self.rect.h);
        let start = list.quads.len();
        let skin = if self.focused { FOCUS_SKIN } else { DISABLED_SKIN };
        list.draw_texture(x, y, SKIN_CAP, h, 0, 0, SKIN_CAP, SKIN_ROW, skin[0], 1.0);
        list.draw_texture(x + SKIN_CAP, y, (w as f32 - SKIN_MIDDLE_TRIM) as i32, h, 0, 0, SKIN_CAP, SKIN_ROW, skin[1], 1.0);
        list.draw_texture(x + (w as f32 - SKIN_CAP_FROM_RIGHT) as i32, y, SKIN_CAP, h, 0, 0, SKIN_CAP, SKIN_ROW, skin[2], 1.0);
        if self.highlight {
            list.draw_texture(x, y, w, h, 0, 0, 0xc1, 16, HIGHLIGHT_SKIN, 1.0);
        }
        let max_width = if self.extra_width > 0 { w + self.extra_width } else { w };
        if self.max_len == 0 {
            self.accepts = m.text_width(&self.left) + m.text_width(&self.right) < max_width - SCROLL_MARGIN;
        }
        let stars = self.kind == KIND_PASSWORD;
        let view = self.scroll.max(0) as usize;
        let mut text_w = 0;
        if stars {
            let hidden = "*".repeat(self.left.len());
            if !hidden.is_empty() {
                text_w = list.draw_text(x + 2, y + 2, TEXT_COLOR, &hidden, font) - (x + 2);
            }
        } else if view < self.left.len() {
            list.draw_text(x + 2, y + 2, TEXT_COLOR, &to_string(&self.left[view..]), font);
            text_w = m.text_width(&self.left[view..]);
        }
        let right = if stars { "*".repeat(self.right.len()) } else { to_string(&self.right) };
        if !right.is_empty() {
            list.draw_text(x + text_w + 2, y + 2, TEXT_COLOR, &right, font);
        }
        let mut caret_x = text_w;
        if caret_x == w {
            caret_x -= 1;
        } else if caret_x > w {
            self.scroll += 1;
            self.scroll_end = self.scroll;
            caret_x = w - 1;
        }
        if self.caret_enabled && self.focused && self.caret_blink {
            let at = if stars { caret_x } else { caret_x - 1 };
            list.draw_text(x + at, y + 2, TEXT_COLOR, "|", font);
        }
        list.clip_from(start, clip);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixed;

    impl Metrics for Fixed {
        fn text_width(&self, text: &[u16]) -> i32 {
            7 * text.len() as i32
        }

        fn char_width(&self, _: u16) -> i32 {
            7
        }
    }

    fn contract(kind: u32, max_len: u32) -> ChildContract {
        let state: &'static [(u16, u32)] = Box::leak(
            vec![(STATE_KIND, kind), (STATE_CARET, 1), (STATE_ACCEPTS, 1), (STATE_BLINK, 1), (STATE_MAX_LEN, max_len)].into_boxed_slice(),
        );
        ChildContract {
            kind: ChildKind::EditBox,
            rect: Rect::new(10, 20, 63, 17),
            id: 0xf0e404,
            vtable: 0x101a2730,
            registration: (2, 0, 0),
            handles: &[],
            state,
            textures: &[],
        }
    }

    fn typed(kind: u32, max_len: u32, text: &str) -> EditBox {
        let mut edit = EditBox::from_contract(&contract(kind, max_len));
        for ch in text.chars() {
            edit.char_input(ch as u32, &Fixed);
        }
        edit
    }

    fn string(units: &[u16]) -> String {
        to_string(units)
    }

    #[test]
    fn contract_state_initializes_the_box() {
        let edit = EditBox::from_contract(&contract(3, 9));
        assert_eq!((edit.kind, edit.max_len, edit.id), (3, 9, 0xf0e404));
        assert!(edit.accepts && edit.caret_enabled && edit.caret_blink && !edit.focused);
        assert_eq!(edit.get_text(), None);
    }

    #[test]
    fn plain_characters_append_and_only_printable_ascii_is_accepted() {
        let mut edit = typed(0, 0, "ab");
        edit.char_input(0x07, &Fixed);
        edit.char_input(0x82, &Fixed);
        edit.char_input(0x81, &Fixed);
        assert_eq!(edit.left().len(), 3);
        assert_eq!(edit.get_text().map(|t| t.chars().count()), Some(3));
    }

    #[test]
    fn numeric_box_takes_digits_only_and_groups_thousands() {
        let mut edit = typed(3, 0, "12a34");
        assert_eq!(edit.get_text().as_deref(), Some("1234"));
        assert_eq!(string(edit.left()), "1,234");
        edit.char_input(u32::from(b'5'), &Fixed);
        assert_eq!(string(edit.left()), "12,345");
        assert_eq!(group_thousands(&"1234567".encode_utf16().collect::<Vec<_>>()), "1,234,567".encode_utf16().collect::<Vec<_>>());
        assert_eq!(group_thousands(&"123".encode_utf16().collect::<Vec<_>>()), "123".encode_utf16().collect::<Vec<_>>());
    }

    #[test]
    fn maximum_length_stops_input_and_clears_the_accept_flag() {
        let mut edit = typed(0, 3, "abc");
        assert!(!edit.accepts);
        edit.char_input(u32::from(b'd'), &Fixed);
        assert_eq!(string(edit.left()), "abc");
        edit.key_down(KEY_BACKSPACE, &Fixed);
        assert!(edit.accepts);
        assert_eq!(string(edit.left()), "ab");
    }

    #[test]
    fn quote_as_first_character_notifies_only_on_notify_boxes() {
        let mut notify = typed(1, 0, "\"");
        assert_eq!(notify.notices, vec![EditNotice { id: 0xf0e404, code: NOTICE_QUOTE }]);
        notify.char_input(0x22, &Fixed);
        assert_eq!(notify.notices.len(), 1);
        assert!(typed(0, 0, "\"").notices.is_empty());
    }

    #[test]
    fn arrows_move_text_between_the_two_halves() {
        let mut edit = typed(0, 0, "abcd");
        edit.key_down(KEY_LEFT, &Fixed);
        edit.key_down(KEY_LEFT, &Fixed);
        assert_eq!((string(edit.left()), string(edit.right())), ("ab".into(), "cd".into()));
        edit.char_input(u32::from(b'X'), &Fixed);
        assert_eq!((string(edit.left()), string(edit.right())), ("abX".into(), "cd".into()));
        edit.key_down(KEY_RIGHT, &Fixed);
        assert_eq!(string(edit.left()), "abXc");
        edit.key_down(KEY_HOME, &Fixed);
        assert_eq!((string(edit.left()), string(edit.right())), (String::new(), "abXcd".into()));
        edit.key_down(KEY_END, &Fixed);
        assert_eq!((string(edit.left()), string(edit.right())), ("abXcd".into(), String::new()));
    }

    #[test]
    fn delete_is_right_arrow_followed_by_backspace() {
        let mut edit = typed(0, 0, "abcd");
        edit.key_down(KEY_HOME, &Fixed);
        edit.key_down(KEY_DELETE, &Fixed);
        assert_eq!((string(edit.left()), string(edit.right())), (String::new(), "bcd".into()));
        edit.key_down(KEY_SHIFT, &Fixed);
        edit.key_down(KEY_DELETE, &Fixed);
        assert_eq!(string(edit.right()), "bcd");
    }

    #[test]
    fn shift_arrows_build_a_selection_that_typing_replaces() {
        let mut edit = typed(0, 0, "abcd");
        edit.key_down(KEY_SHIFT, &Fixed);
        edit.key_down(KEY_LEFT, &Fixed);
        edit.key_down(KEY_LEFT, &Fixed);
        assert_eq!(edit.selection(), (-2, 14));
        edit.key_up(KEY_SHIFT);
        edit.char_input(u32::from(b'Z'), &Fixed);
        assert_eq!(string(edit.left()), "abZ");
        assert_eq!(edit.selection(), (0, 0));
    }

    #[test]
    fn shift_home_and_end_select_the_traversed_side() {
        let mut edit = typed(0, 0, "abcd");
        edit.key_down(KEY_SHIFT, &Fixed);
        edit.key_down(KEY_HOME, &Fixed);
        assert_eq!(edit.selection(), (4, 28));
        let mut end = typed(0, 0, "abcd");
        end.key_down(KEY_HOME, &Fixed);
        end.key_down(KEY_SHIFT, &Fixed);
        end.key_down(KEY_END, &Fixed);
        assert_eq!(end.selection().0, -4);
    }

    #[test]
    fn right_selection_shrinks_with_left_and_extends_with_right() {
        let mut edit = typed(0, 0, "abcd");
        edit.key_down(KEY_HOME, &Fixed);
        edit.key_down(KEY_SHIFT, &Fixed);
        edit.key_down(KEY_RIGHT, &Fixed);
        edit.key_down(KEY_RIGHT, &Fixed);
        assert_eq!(edit.selection(), (2, 14));
        edit.key_down(KEY_LEFT, &Fixed);
        assert_eq!(edit.selection(), (1, 7));
    }

    #[test]
    fn numeric_backspace_over_a_separator_removes_the_digit_before_it() {
        let mut edit = typed(3, 0, "1234");
        assert_eq!(string(edit.left()), "1,234");
        for _ in 0..3 {
            edit.key_down(KEY_LEFT, &Fixed);
        }
        assert_eq!((string(edit.left()), string(edit.right())), ("1,".into(), "234".into()));
        edit.key_down(KEY_BACKSPACE, &Fixed);
        assert_eq!(edit.get_text().as_deref(), Some("234"));
        assert_eq!((string(edit.left()), string(edit.right())), (String::new(), "234".into()));
    }

    #[test]
    fn moving_left_over_a_separator_skips_it() {
        let mut edit = typed(3, 0, "1234");
        for _ in 0..4 {
            edit.key_down(KEY_LEFT, &Fixed);
        }
        assert_eq!((string(edit.left()), string(edit.right())), (String::new(), "1,234".into()));
        edit.key_down(KEY_RIGHT, &Fixed);
        edit.key_down(KEY_RIGHT, &Fixed);
        assert_eq!((string(edit.left()), string(edit.right())), ("1,2".into(), "34".into()));
    }

    #[test]
    fn enter_and_vertical_keys_report_to_the_owner() {
        let mut edit = typed(1, 0, "x");
        edit.key_down(KEY_ENTER, &Fixed);
        edit.key_down(KEY_UP, &Fixed);
        edit.key_down(KEY_DOWN, &Fixed);
        let codes: Vec<u32> = edit.notices.iter().map(|n| n.code).collect();
        assert_eq!(codes, vec![NOTICE_ENTER, NOTICE_UP, NOTICE_DOWN]);
        let mut empty = typed(0, 0, "");
        empty.key_down(KEY_ENTER, &Fixed);
        empty.key_down(KEY_UP, &Fixed);
        assert_eq!(empty.notices.iter().map(|n| n.code).collect::<Vec<_>>(), vec![NOTICE_ENTER_EMPTY]);
    }

    #[test]
    fn disabled_box_ignores_keys_and_characters() {
        let mut edit = typed(0, 0, "ab");
        edit.enabled = false;
        assert!(!edit.key_down(KEY_BACKSPACE, &Fixed));
        edit.char_input(u32::from(b'c'), &Fixed);
        assert_eq!(string(edit.left()), "ab");
    }

    #[test]
    fn overflow_scrolls_by_the_width_of_the_new_character() {
        let mut edit = typed(0, 40, "abcdefghij");
        assert_eq!(edit.scroll(), 1);
        edit.char_input(u32::from(b'k'), &Fixed);
        assert!(edit.scroll() >= 2);
    }

    #[test]
    fn layout_places_the_caret_at_the_width_of_the_visible_left_text() {
        let mut edit = typed(0, 40, "abc");
        edit.layout(&Fixed);
        assert_eq!(edit.caret_px(), 21);
        let mut long = typed(0, 40, "abcdefghijkl");
        long.layout(&Fixed);
        assert!(long.caret_px() <= 63);
    }

    #[test]
    fn unbounded_box_stops_accepting_when_the_text_fills_the_width() {
        let mut edit = typed(0, 0, "abcdefg");
        edit.layout(&Fixed);
        assert!(edit.accepts);
        edit.char_input(u32::from(b'h'), &Fixed);
        edit.layout(&Fixed);
        assert!(!edit.accepts);
        edit.char_input(u32::from(b'i'), &Fixed);
        assert_eq!(edit.left().len(), 8);
    }

    #[test]
    fn numeric_tint_follows_the_digit_count_and_ignores_separators() {
        let tint = |text: &str| numeric_tint(&text.encode_utf16().collect::<Vec<_>>());
        assert_eq!(tint("9,999"), TEXT_COLOR);
        assert_eq!(tint("10,000"), Argb(0xff00ffff));
        assert_eq!(tint("100,000"), Argb(0xffff80ff));
        assert_eq!(tint("1,000,000"), Argb(0xffffff00));
    }

    #[test]
    fn get_text_strips_separators_of_numeric_boxes_only() {
        let mut numeric = EditBox::from_contract(&contract(3, 0));
        numeric.set_text("1,234,567");
        assert_eq!(numeric.get_text().as_deref(), Some("1234567"));
        let mut plain = EditBox::from_contract(&contract(0, 0));
        plain.set_text("1,234");
        assert_eq!(plain.get_text().as_deref(), Some("1,234"));
    }

    #[test]
    fn clear_resets_text_scroll_and_acceptance_but_not_the_limit() {
        let mut edit = typed(0, 3, "abc");
        edit.clear();
        assert!(edit.left().is_empty() && edit.accepts);
        assert_eq!(edit.max_len, 3);
        edit.set_max_len(0);
        edit.set_max_len(-4);
        assert_eq!(edit.max_len, 3);
        edit.set_max_len(5);
        assert_eq!(edit.max_len, 5);
    }

    #[test]
    fn blink_toggles_only_for_its_timer_and_mouse_up_releases_shift() {
        let mut edit = EditBox::from_contract(&contract(0, 0));
        edit.toggle_blink(7);
        assert!(edit.caret_blink);
        edit.toggle_blink(BLINK_TIMER_ID);
        assert!(!edit.caret_blink);
        edit.key_down(KEY_SHIFT, &Fixed);
        edit.mouse_up();
        edit.char_input(u32::from(b'a'), &Fixed);
        edit.key_down(KEY_LEFT, &Fixed);
        assert_eq!(edit.selection(), (0, 0));
    }

    #[test]
    fn click_places_the_cursor_by_walking_character_widths_and_starts_a_drag() {
        let mut edit = typed(0, 40, "abcd");
        edit.layout(&Fixed);
        assert_eq!(edit.caret_px(), 28);
        edit.mouse_down(10 + 14, &Fixed);
        assert_eq!((string(edit.left()), string(edit.right())), ("ab".into(), "cd".into()));
        assert_eq!(edit.selection(), (0, 0));
        edit.mouse_up();
        edit.layout(&Fixed);
        edit.mouse_down(10 + 21, &Fixed);
        assert_eq!(string(edit.left()), "abc");
        edit.mouse_up();
    }

    #[test]
    fn dragging_after_a_click_selects_in_both_directions() {
        let mut edit = typed(0, 40, "abcd");
        edit.layout(&Fixed);
        edit.mouse_down(10 + 14, &Fixed);
        edit.layout(&Fixed);
        edit.mouse_move(10, &Fixed);
        assert_eq!(edit.selection(), (-2, 14));
        edit.mouse_move(10 + 21, &Fixed);
        assert_eq!(edit.selection(), (1, 7));
        edit.mouse_up();
        edit.mouse_move(10 + 28, &Fixed);
        assert_eq!(edit.selection(), (1, 7));
    }

    #[test]
    fn first_click_after_losing_focus_only_selects_the_text_before_the_cursor() {
        let mut edit = EditBox::from_contract(&ChildContract { state: Box::leak(vec![(STATE_FRESH, 1), (STATE_ACCEPTS, 1)].into_boxed_slice()), ..contract(0, 0) });
        edit.set_text("abcd");
        edit.mouse_down(10 + 7, &Fixed);
        assert_eq!(edit.selection(), (-4, 28));
        assert_eq!(string(edit.left()), "abcd");
        edit.mouse_up();
        edit.layout(&Fixed);
        edit.mouse_down(10 + 7, &Fixed);
        assert_eq!(string(edit.left()), "a");
    }

    #[test]
    fn losing_focus_commits_the_cursor_to_the_end_and_clears_the_selection() {
        let mut edit = typed(0, 0, "abcd");
        edit.focus();
        edit.key_down(KEY_HOME, &Fixed);
        edit.key_down(KEY_SHIFT, &Fixed);
        edit.key_down(KEY_RIGHT, &Fixed);
        edit.kill_focus();
        assert!(!edit.focused);
        assert_eq!((string(edit.left()), string(edit.right())), ("abcd".into(), String::new()));
        assert_eq!(edit.selection(), (0, 0));
        edit.mouse_down(10, &Fixed);
        assert_eq!(edit.selection(), (-4, 28));
    }

    #[test]
    fn insert_text_prepends_or_appends_and_respects_the_accept_flag() {
        let mut edit = typed(0, 40, "cd");
        edit.insert_text("ab", false, &Fixed);
        assert_eq!(string(edit.left()), "abcd");
        edit.insert_text("ef", true, &Fixed);
        assert_eq!(string(edit.left()), "abcdef");
        edit.accepts = false;
        edit.insert_text("zz", true, &Fixed);
        assert_eq!(string(edit.left()), "abcdef");
        let mut numeric = typed(3, 0, "12");
        numeric.insert_text("345", true, &Fixed);
        assert_eq!(string(numeric.left()), "12,345");
    }

    #[test]
    fn insert_text_scrolls_until_the_tail_fits() {
        let mut edit = typed(0, 40, "");
        edit.insert_text("abcdefghijkl", true, &Fixed);
        assert_eq!(edit.scroll(), 5);
    }

    #[test]
    fn blink_timer_toggles_every_five_hundred_milliseconds() {
        let mut edit = EditBox::from_contract(&contract(0, 0));
        edit.tick(499);
        assert!(edit.caret_blink);
        edit.tick(1);
        assert!(!edit.caret_blink);
        edit.tick(1200);
        assert!(!edit.caret_blink);
        edit.tick(300);
        assert!(edit.caret_blink);
    }

    fn glyph_font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), crate::font::Glyphs::parse(&crate::font::gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn html_edit(width: i32, kind: u32) -> EditBox {
        let mut edit = EditBox::html(Rect::new(3, 5, width, 15), kind, 0);
        edit.focus();
        edit
    }

    fn drawn_text(list: &DrawList) -> Vec<(f32, f32)> {
        list.quads.iter().filter(|q| q.texture == "f").map(|q| (q.x, q.y)).collect()
    }

    #[test]
    fn on_paint_100498e0_draws_the_right_half_after_the_left_one_and_the_caret_before_it() {
        let mut edit = html_edit(100, 0);
        for ch in "abcd".chars() {
            edit.char_input(ch as u32, &Fixed);
        }
        edit.key_down(KEY_LEFT, &Fixed);
        edit.key_down(KEY_LEFT, &Fixed);
        edit.layout(&Fixed);
        let mut list = DrawList::default();
        edit.paint_html(&mut list, (10, 20), Rect::new(0, 0, 500, 500), &Fixed, &glyph_font());
        let xs: Vec<f32> = drawn_text(&list).into_iter().map(|(x, _)| x).collect();
        assert_eq!(xs, [15.0, 21.0, 29.0, 35.0, 26.0]);
    }

    #[test]
    fn on_paint_100498e0_password_boxes_draw_stars_and_the_caret_without_the_minus_one() {
        let mut edit = html_edit(100, KIND_PASSWORD);
        for ch in "abc".chars() {
            edit.char_input(ch as u32, &Fixed);
        }
        edit.layout(&Fixed);
        let mut list = DrawList::default();
        edit.paint_html(&mut list, (0, 0), Rect::new(0, 0, 500, 500), &Fixed, &glyph_font());
        let glyph = |c: char| (c as u32 - 0x20) as f32 * 7.0;
        let sources: Vec<f32> = list.quads.iter().filter(|q| q.texture == "f").map(|q| q.u).collect();
        assert_eq!(sources, [glyph('*'), glyph('*'), glyph('*'), glyph('|')]);
        let caret = list.quads.iter().rev().find(|q| q.texture == "f").unwrap();
        assert_eq!(caret.x, 3.0 + 18.0);
    }

    #[test]
    fn on_paint_100498e0_scrolls_the_view_one_character_per_paint_when_the_caret_passes_the_width() {
        let mut edit = html_edit(30, 0);
        edit.left = "abcdef".encode_utf16().collect();
        edit.scroll = 0;
        let mut list = DrawList::default();
        edit.paint_html(&mut list, (0, 0), Rect::new(0, 0, 500, 500), &Fixed, &glyph_font());
        assert_eq!(edit.scroll(), 1);
        edit.paint_html(&mut list, (0, 0), Rect::new(0, 0, 500, 500), &Fixed, &glyph_font());
        assert_eq!(edit.scroll(), 2);
        let caret = list.quads.iter().rev().find(|q| q.texture == "f").unwrap();
        assert_eq!(caret.x, 3.0 + 29.0 - 1.0);
    }

    #[test]
    fn on_paint_100498e0_blinks_with_the_timer_and_hides_the_caret_without_focus() {
        let mut edit = html_edit(100, 0);
        let count = |edit: &mut EditBox| {
            let mut list = DrawList::default();
            edit.paint_html(&mut list, (0, 0), Rect::new(0, 0, 500, 500), &Fixed, &glyph_font());
            list.quads.iter().filter(|q| q.texture == "f").count()
        };
        assert_eq!(count(&mut edit), 1);
        edit.tick(BLINK_PERIOD_MS);
        assert_eq!(count(&mut edit), 0);
        edit.tick(BLINK_PERIOD_MS);
        assert_eq!(count(&mut edit), 1);
        edit.kill_focus();
        assert_eq!(count(&mut edit), 0);
    }

    #[test]
    fn on_paint_1000f080_draws_the_right_selection_from_the_caret_plus_one() {
        let mut edit = typed(0, 0, "abcd");
        edit.key_down(KEY_HOME, &Fixed);
        edit.key_down(KEY_SHIFT, &Fixed);
        edit.key_down(KEY_RIGHT, &Fixed);
        edit.key_down(KEY_RIGHT, &Fixed);
        edit.layout(&Fixed);
        let mut list = DrawList::default();
        edit.paint(&mut list, (0, 0), &Fixed, &glyph_font());
        let xs: Vec<f32> = list.quads.iter().filter(|q| q.texture == SELECTION_TEXTURE).map(|q| q.x).collect();
        assert_eq!(xs, [10.0 + 1.0, 10.0 + 1.0 + 7.0]);
    }

    #[test]
    fn on_paint_1000f080_draws_the_macro_line_highlight_over_the_whole_box() {
        let mut edit = typed(0, 0, "");
        let mut list = DrawList::default();
        edit.paint(&mut list, (0, 0), &Fixed, &glyph_font());
        assert!(list.quads.iter().all(|q| q.texture != HIGHLIGHT_SKIN));
        edit.highlight = true;
        let mut list = DrawList::default();
        edit.paint(&mut list, (0, 0), &Fixed, &glyph_font());
        let quad = list.quads.iter().find(|q| q.texture == HIGHLIGHT_SKIN).unwrap();
        assert_eq!((quad.x, quad.y, quad.w, quad.h, quad.uw, quad.vh), (10.0, 20.0, 63.0, 17.0, 193.0, 16.0));
    }

    #[test]
    fn key_down_10010020_home_leaves_the_end_view_alone_and_end_returns_to_it() {
        let mut edit = typed(0, 0, "abcdefghijkl");
        edit.layout(&Fixed);
        let end_view = edit.scroll_end;
        edit.key_down(KEY_HOME, &Fixed);
        assert_eq!((edit.scroll(), edit.scroll_end), (0, end_view));
        edit.key_down(KEY_END, &Fixed);
        assert_eq!(edit.scroll(), end_view);
    }

    #[test]
    fn delete_char_1000e790_removes_from_left_or_right_by_position() {
        let mut edit = typed(0, 0, "abcd");
        edit.key_down(KEY_LEFT, &Fixed);
        edit.key_down(KEY_LEFT, &Fixed);
        edit.delete_char(0, &Fixed);
        assert_eq!((string(edit.left()), string(edit.right())), ("b".to_string(), "cd".to_string()));
        edit.delete_char(2, &Fixed);
        assert_eq!((string(edit.left()), string(edit.right())), ("b".to_string(), "c".to_string()));
        edit.delete_char(9, &Fixed);
        assert_eq!(string(edit.right()), "c");
    }
}
