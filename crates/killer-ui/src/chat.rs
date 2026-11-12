use crate::button::{CheckBox, TextButton};
use crate::color::{Argb, BUTTON_TEXT, CHAT_SYSTEM, TEXT, chat_color};
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::ini::ChatFilter;
use crate::resize_frame::ResizeFrame;
use crate::window::Rect;

pub fn creation_position(viewport: (i32, i32)) -> (i32, i32) {
    (0, crate::window_restore::x87_to_int(f64::from(viewport.1 as f32) - 187.0))
}

pub fn initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    let (x, y) = creation_position(viewport);
    crate::initial_position::InitialPosition {
        enabled: true, horizontal_mode: 0, vertical_mode: 4,
        x, y: viewport.1.wrapping_sub(y),
    }
}

/// Canal do sussurro — o único que leva destinatário.
pub const CHAT_TELL: i32 = 2;

/// Canal de cada aba quando a linha digitada não tem prefixo. O original decide pelo índice da
/// aba ativa (campo `+0x1d0` da `NCChatWnd`) num `switch` dentro de `FUN_10058330`
/// (`nwindow.dll`, `docs/evidence/decompiled/ui/FUN_10058330.c`): All→0, Trade→8, Party→3,
/// Clan→4, Alliance→9. Os números são os mesmos canais do protocolo (`CHAT_*` do `l2-net`).
pub const TAB_CHANNELS: [i32; 5] = [0, 8, 3, 4, 9];

/// Prefixo que troca o canal da linha, com as variantes de largura dupla que o cliente também
/// aceita. Sai da tabela de ponteiros em `0x1023fbd0` (`nwindow.dll`), que aponta para os
/// caracteres em `0x1019f300`..`0x1019f348`; o canal de cada um vem da cadeia de `if/else` em
/// `FUN_10058330`. O `~` volta pro canal geral, e o prefixo reconhecido é removido do texto
/// antes do envio.
pub const CHAT_PREFIXES: &[(char, i32)] = &[
    ('!', 1),
    ('！', 1),
    ('"', CHAT_TELL),
    ('”', CHAT_TELL),
    ('＂', CHAT_TELL),
    ('#', 3),
    ('＃', 3),
    ('@', 4),
    ('＠', 4),
    ('&', 6),
    ('＆', 6),
    ('*', 7),
    ('＊', 7),
    ('+', 8),
    ('＋', 8),
    ('$', 9),
    ('＄', 9),
    ('%', 17),
    ('％', 17),
    ('~', 0),
    ('＾', 0),
];

/// Separadores que fecham o nome do destinatário no sussurro: espaço comum e o espaço largo
/// (`U+3000`), os dois testados pelo original.
const TELL_SEPARATORS: [char; 2] = [' ', '\u{3000}'];

/// Linha de chat pronta pra virar pacote: canal, destinatário (só no sussurro) e o texto já sem
/// o prefixo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub kind: i32,
    pub target: String,
    pub text: String,
}

/// Decide canal, destinatário e texto de uma linha digitada, seguindo `FUN_10058330`: o canal
/// começa pelo da aba ativa, o primeiro caractere pode trocá-lo, e o prefixo reconhecido sai do
/// texto enviado. No sussurro o destinatário é o que vem entre o prefixo e o primeiro espaço —
/// se não sobrar mensagem depois do nome, o original não manda nada. Linha que fica vazia
/// depois de tirar o prefixo também não vai.
pub fn chat_message(typed: &str, tab: usize) -> Option<ChatMessage> {
    let first = typed.chars().next()?;
    let prefix = CHAT_PREFIXES.iter().find(|(c, _)| *c == first);
    let kind = prefix.map_or_else(|| TAB_CHANNELS[tab.min(TAB_CHANNELS.len() - 1)], |(_, kind)| *kind);
    let rest = match prefix {
        Some((c, _)) => &typed[c.len_utf8()..],
        None => typed,
    };
    let message = |target: String, text: &str| {
        (!text.is_empty()).then(|| ChatMessage { kind, target, text: text.to_string() })
    };
    if kind != CHAT_TELL {
        return message(String::new(), rest);
    }
    // Sussurro sem nome (a linha começa por espaço logo depois do prefixo): o original manda a
    // linha inteira, prefixo incluído, e sem destinatário.
    match rest.split_once(TELL_SEPARATORS) {
        Some((name, text)) if !name.is_empty() => message(name.to_string(), text),
        _ => message(String::new(), typed),
    }
}

pub const CHAT_WIDTH: i32 = 348;
pub const CHAT_LINE: i32 = 15;
pub const CHAT_HEADER: i32 = 18;
pub const CHAT_BELOW: i32 = 18;
pub const CHAT_FOOTER: i32 = 46;
pub const LIST_BOTTOM_INSET: i32 = 10;
pub const LINE_WRAP_INSET: i32 = 22;
pub const LINE_INDENT: i32 = 12;
pub const LINE_TEXT_X: i32 = 23;
pub const LINE_TEXT_Y: i32 = 14;
pub const CRITICAL_ANNOUNCE: i32 = 18;

pub const CHATTING_BACK1: &str = "L2UI_CH3.ChatWnd.Chatting_Back1";
pub const CHATTING_BACK2: &str = "L2UI_CH3.ChatWnd.Chatting_Back2";
pub const CHATTING_BACK3: &str = "L2UI_CH3.ChatWnd.Chatting_Back3";
pub const CHATTING_BACK4: &str = "L2UI_CH3.ChatWnd.Chatting_Back4";
pub const CHATTING_SIZE_CONTROL: &str = "L2UI_CH3.ChatWnd.Chatting_sizeControl";
pub const CHATTING_TAB_ON: &str = "L2UI_CH3.ChatWnd.Chatting_Tab1";
pub const CHATTING_TAB_OFF: &str = "L2UI_CH3.ChatWnd.Chatting_Tab2";
pub const CHATTING_IME_EN: &str = "L2UI_CH3.ChatWnd.Chatting_IMEen";

pub const SIZE_CONTROL_POS: (i32, i32) = (0, -3);
pub const SIZE_CONTROL_SIZE: (i32, i32) = (24, 10);
pub const SIZE_CONTROL_ID: i32 = 0xf007;
pub const TAB_X: [i32; 5] = [23, 87, 151, 215, 279];
pub const TAB_SIZE: (i32, i32) = (64, 23);
pub const TAB_TEXT_CENTER_X: i32 = 34;
pub const TAB_TEXT_Y: i32 = 6;
pub const TAB_SYSSTRING_IDS: [u32; 5] = [144, 355, 188, 128, 559];
pub const TAB_NAMES: [&str; 5] = ["All", "+Trade", "#Party", "@Clan", "$Alliance"];
pub const IME_X: i32 = 24;
pub const IME_BOTTOM_INSET: i32 = 20;
pub const IME_SIZE: i32 = 15;
pub const INPUT_X: i32 = 39;
pub const INPUT_BOTTOM_INSET: i32 = 22;
pub const INPUT_WIDTH: i32 = 303;
pub const INPUT_TEXT_INSET: i32 = 2;
pub const CARET: char = '|';
pub const CARET_BACK: i32 = 1;
pub const CHATTING_BACK5: &str = "L2UI_CH3.ChatWnd.Chatting_Back5";
pub const CHATTING_TAB_SPLIT: &str = "L2UI_CH3.ChatWnd.chatting_tab3";
pub const SPLIT_WINDOW_BASE: u32 = 0x1a10;
pub const CHATTING_OPTION: &str = "L2UI_CH3.ChatWnd.Chatting_Option1";
pub const CHATTING_OPTION_DOWN: &str = "L2UI_CH3.ChatWnd.Chatting_Option2";
pub const BTN_NORMAL: &str = "L2UI_ch3.button.btn1_normal";
pub const BTN_DOWN: &str = "L2UI_ch3.button.btn1_normalon";
pub const OPTION_RECT: Rect = Rect::new(5, -20, 15, 15);
pub const OPTION_TOOLTIP_SYSSTRING: u32 = 0x381;
pub const MESSENGER_RECT: Rect = Rect::new(5, -39, 15, 15);
pub const MESSENGER_TOOLTIP_SYSSTRING: u32 = 0x380;
pub const MESSENGER_NORMAL: &str = "L2UI_CH3.Msn.chatting_msn5";
pub const MESSENGER_DOWN: &str = "L2UI_CH3.Msn.chatting_msn5_down";
pub const FILTER_PANEL_POS: (i32, i32) = (0, -191);
pub const FILTER_PANEL_SIZE: (i32, i32) = (256, 191);
pub const FILTER_OK_RECT: Rect = Rect::new(51, 160, 76, 23);
pub const FILTER_CANCEL_RECT: Rect = Rect::new(131, 160, 76, 23);
pub const FILTER_OK_SYSSTRING: u32 = 0x8c;
pub const FILTER_CANCEL_SYSSTRING: u32 = 0x8d;
pub const FILTER_COUNT: usize = 12;
pub const FILTER_ROWS: [(u32, &str, &str, Rect); FILTER_COUNT] = [
    (0x164, "system", "System", Rect::new(17, 17, 100, 12)),
    (0x165, "damage", "Damage", Rect::new(28, 37, 80, 12)),
    (0x166, "chat", "Chat", Rect::new(17, 61, 100, 12)),
    (0x17f, "normal", "General", Rect::new(28, 81, 80, 12)),
    (0x167, "shout", "Shout", Rect::new(28, 99, 80, 12)),
    (0x168, "pledge", "Clan", Rect::new(28, 117, 80, 12)),
    (0x169, "party", "Party", Rect::new(138, 81, 80, 12)),
    (0x16a, "market", "Trade", Rect::new(138, 99, 80, 12)),
    (0x16b, "tell", "Whisper", Rect::new(138, 117, 80, 12)),
    (0x250, "ally", "Alliance", Rect::new(28, 135, 80, 12)),
    (0x31c, "useitems", "Consumables", Rect::new(138, 37, 92, 12)),
    (0x4bb, "hero", "Hero Voice", Rect::new(138, 135, 92, 12)),
];
pub const FILTER_SYSTEM: usize = 0;
pub const FILTER_CHAT: usize = 2;
pub const FILTER_SYSTEM_CHILDREN: [usize; 2] = [1, 10];
pub const FILTER_CHAT_CHILDREN: [usize; 8] = [3, 4, 5, 6, 7, 8, 9, 11];
pub const FILTER_TAB_LOCK: [Option<usize>; 5] = [None, Some(7), Some(6), Some(5), Some(9)];

/// O que o clique acertou no painel de filtro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterHit {
    Row(usize),
    Ok,
    Cancel,
}

pub const SYSTEM_GROUP_BROADCAST: u32 = 2;
pub const SYSTEM_GROUP_DAMAGE: u32 = 3;
pub const SYSTEM_GROUP_PETITION: u32 = 6;
pub const SYSTEM_GROUP_USE_ITEMS: u32 = 7;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatEntry {
    pub kind: i32,
    pub name: String,
    pub text: String,
    pub group: u32,
    pub color: Option<Argb>,
}

impl ChatEntry {
    pub fn new(kind: i32, name: impl Into<String>, text: impl Into<String>) -> ChatEntry {
        ChatEntry { kind, name: name.into(), text: text.into(), group: 0, color: None }
    }

    pub fn line(&self) -> String {
        if self.kind == CRITICAL_ANNOUNCE || self.name.is_empty() {
            self.text.clone()
        } else {
            format!("{}: {}", self.name, self.text)
        }
    }

    pub fn color(&self) -> Argb {
        self.color.unwrap_or_else(|| chat_color(self.kind))
    }
}

pub fn filter_name(kind: i32) -> Option<&'static str> {
    match kind {
        CHAT_SYSTEM => Some("system"),
        0 => Some("normal"),
        1 => Some("shout"),
        2 => Some("tell"),
        3 => Some("party"),
        4 => Some("pledge"),
        8 => Some("market"),
        9 => Some("ally"),
        17 => Some("hero"),
        _ => None,
    }
}

#[derive(Clone, Copy, Default)]
struct LineWidth {
    done: u32,
    segment: u32,
    previous: Option<char>,
}

impl LineWidth {
    fn push(&mut self, c: char, font: &BitmapFont) {
        if c == '\n' {
            if self.previous == Some('\r') {
                self.segment -= font.glyph('\r').map_or(0, |g| g.width);
            }
            self.done = self.done.max(self.segment);
            self.segment = 0;
        } else {
            self.segment += font.glyph(c).map_or(0, |g| g.width);
        }
        self.previous = Some(c);
    }

    fn value(&self) -> u32 {
        self.done.max(self.segment)
    }
}

fn measured(text: &str, font: &BitmapFont) -> LineWidth {
    let mut width = LineWidth::default();
    for c in text.chars() {
        width.push(c, font);
    }
    width
}

pub fn wrap_line(text: &str, width: i32, indent: i32, font: &BitmapFont) -> Vec<(i32, String)> {
    let mut rows: Vec<(i32, String)> = Vec::new();
    let mut current = String::new();
    let mut current_width = LineWidth::default();
    let mut current_indent = 0;
    for c in text.chars() {
        let mut candidate_width = current_width;
        candidate_width.push(c, font);
        if candidate_width.value() as i32 + current_indent <= width || current.is_empty() {
            current.push(c);
            current_width = candidate_width;
            continue;
        }
        let (kept, rest) = match current.rfind(' ') {
            Some(space) => {
                let mut rest = String::with_capacity(current.len() - space);
                rest.push_str(&current[space + 1..]);
                rest.push(c);
                current.truncate(space);
                (current, rest)
            }
            None => (current, c.to_string()),
        };
        rows.push((current_indent, kept));
        current_width = measured(&rest, font);
        current = rest;
        current_indent = indent;
    }
    rows.push((current_indent, current));
    rows
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKey {
    Char(char),
    Backspace,
    Left,
    Right,
    Home,
    End,
    Delete,
    Up,
    Down,
    Enter,
    Escape,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextInput {
    pub text: String,
    pub cursor: usize,
}

impl TextInput {
    pub fn key(&mut self, key: InputKey) -> Option<String> {
        match key {
            InputKey::Char(c) if !c.is_control() => {
                self.text.insert(self.byte_offset(self.cursor), c);
                self.cursor += 1;
            }
            InputKey::Char(_) => {}
            InputKey::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.text.remove(self.byte_offset(self.cursor));
                }
            }
            InputKey::Left => self.cursor = self.cursor.saturating_sub(1),
            InputKey::Right => self.cursor = (self.cursor + 1).min(self.text.chars().count()),
            InputKey::Home => self.cursor = 0,
            InputKey::End => self.cursor = self.text.chars().count(),
            InputKey::Delete => {
                if self.cursor < self.text.chars().count() {
                    self.text.remove(self.byte_offset(self.cursor));
                }
            }
            InputKey::Up | InputKey::Down => {}
            InputKey::Enter => {
                if self.text.is_empty() {
                    return None;
                }
                self.cursor = 0;
                return Some(std::mem::take(&mut self.text));
            }
            InputKey::Escape => {
                self.text.clear();
                self.cursor = 0;
            }
        }
        None
    }

    pub fn insert_at_cursor(&mut self, text: &str) {
        let at = self.byte_offset(self.cursor);
        self.text.insert_str(at, text);
        self.cursor += text.chars().count();
    }

    pub fn insert_at_start(&mut self, text: &str) {
        self.text.insert_str(0, text);
        self.cursor += text.chars().count();
    }

    pub fn delete_first(&mut self) {
        if self.text.is_empty() {
            return;
        }
        self.text.remove(0);
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }

    pub fn before_cursor(&self) -> &str {
        &self.text[..self.byte_offset(self.cursor)]
    }

    fn byte_offset(&self, chars: usize) -> usize {
        self.text.char_indices().nth(chars).map_or(self.text.len(), |(i, _)| i)
    }
}

pub type ChatRow = (i32, String, Argb);

pub fn rows_hash<'a>(entries: impl Iterator<Item = &'a ChatEntry>, font: &BitmapFont) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    font.atlas.hash(&mut hasher);
    font.measure("Wiy0.").hash(&mut hasher);
    for entry in entries {
        entry.kind.hash(&mut hasher);
        entry.name.hash(&mut hasher);
        entry.text.hash(&mut hasher);
        entry.color().0.hash(&mut hasher);
    }
    hasher.finish()
}

impl RowsCache {
    pub fn get_or_build(&self, key: u64, build: impl FnOnce() -> Vec<ChatRow>) -> std::sync::Arc<Vec<ChatRow>> {
        if let Some((cached, rows)) = self.0.borrow().as_ref()
            && *cached == key
        {
            return rows.clone();
        }
        let rows = std::sync::Arc::new(build());
        *self.0.borrow_mut() = Some((key, rows.clone()));
        rows
    }
}

#[derive(Debug, Clone, Default)]
pub struct RowsCache(std::cell::RefCell<Option<(u64, std::sync::Arc<Vec<ChatRow>>)>>);

impl PartialEq for RowsCache {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for RowsCache {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatView {
    pub height: i32,
    pub active_tab: usize,
    pub input: TextInput,
    pub filter: ChatFilter,
    pub scroll: usize,
    pub scroll_extra: std::cell::Cell<usize>,
    pub followed_rows: std::cell::Cell<usize>,
    pub scroll_pressed: Option<crate::list_scroll::ListScrollHit>,
    pub last_rows: std::cell::Cell<usize>,
    pub scroll_grab: Option<i32>,
    pub tab_names: [String; 5],
    pub focused: bool,
    pub filter_open: bool,
    pub filter_edit: [bool; FILTER_COUNT],
    pub filter_labels: [String; FILTER_COUNT],
    pub filter_ok: String,
    pub filter_cancel: String,
    pub option_tooltip: String,
    pub messenger_tooltip: String,
    pub pressed: Option<FilterHit>,
    pub resize_accumulator: i32,
    pub size_frame: ResizeFrame,
    pub rows_cache: RowsCache,
    pub limiter: crate::chat_line::ChatLimiter,
    pub tells: crate::chat_line::TellNames,
    pub old_chat: bool,
    pub capture: crate::text_capture::TextCapture,
    pub detached: [bool; 5],
    pub split_scroll: [usize; 5],
    pub tab_press: Option<(usize, i32, i32)>,
    pub lists: [Vec<ChatEntry>; 5],
    pub prefiltered: bool,
}

pub const LIST_CAPACITY: usize = 200;
pub const MIN_HEIGHT: f32 = 187.0;
pub const MAX_HEIGHT: f32 = 502.0;

impl ChatView {
    pub fn new(height: i32, filter: ChatFilter) -> ChatView {
        ChatView {
            height,
            active_tab: 0,
            input: TextInput::default(),
            filter,
            scroll: 0,
            scroll_extra: std::cell::Cell::new(0),
            followed_rows: std::cell::Cell::new(0),
            scroll_pressed: None,
            last_rows: std::cell::Cell::new(0),
            scroll_grab: None,
            tab_names: TAB_NAMES.map(str::to_string),
            focused: false,
            filter_open: false,
            filter_edit: [false; FILTER_COUNT],
            filter_labels: FILTER_ROWS.map(|(_, _, label, _)| label.to_string()),
            filter_ok: "OK".to_string(),
            filter_cancel: "Cancel".to_string(),
            option_tooltip: String::new(),
            messenger_tooltip: String::new(),
            pressed: None,
            resize_accumulator: 0,
            size_frame: ResizeFrame::new(
                SIZE_CONTROL_ID,
                Rect::new(SIZE_CONTROL_POS.0, SIZE_CONTROL_POS.1, SIZE_CONTROL_SIZE.0, SIZE_CONTROL_SIZE.1),
                crate::resize_frame::MODE_VERTICAL,
            ),
            rows_cache: RowsCache::default(),
            limiter: crate::chat_line::ChatLimiter::default(),
            tells: crate::chat_line::TellNames::default(),
            old_chat: true,
            capture: Default::default(),
            detached: [false; 5],
            split_scroll: [0; 5],
            tab_press: None,
            lists: Default::default(),
            prefiltered: false,
        }
    }

    pub fn resize_by(&mut self, delta: i32) -> Option<i32> {
        let accumulated = self.resize_accumulator.wrapping_add(delta);
        self.resize_accumulator = accumulated;
        if accumulated < 15 && accumulated > -15 {
            return None;
        }
        let step = accumulated / 15 * 15;
        self.resize_accumulator = 0;
        let height = step as f32 + self.height as f32;
        if !(MIN_HEIGHT..=MAX_HEIGHT).contains(&height) {
            return None;
        }
        self.height = crate::window_restore::float_size_to_int(height);
        Some(step)
    }

    pub fn size_press(&mut self, x: i32, y: i32) -> bool {
        self.size_frame.on_lbutton_down(x, y);
        self.size_frame.dragging
    }

    pub fn size_move(&mut self, x: i32, y: i32) -> Option<i32> {
        let buttons = u32::from(self.size_frame.dragging) * crate::resize_frame::BUTTON_LEFT;
        let effects = self.size_frame.on_mouse_move(x, y, buttons);
        let notify = effects.notify.filter(|notify| notify.id == SIZE_CONTROL_ID)?;
        self.resize_by(notify.param)
    }

    pub fn size_release(&mut self) {
        self.size_frame.on_lbutton_up();
    }

    pub fn visible_lines(&self) -> usize {
        ((self.height - CHAT_FOOTER - LIST_BOTTOM_INSET) / CHAT_LINE).max(0) as usize
    }

    pub fn list_height(&self) -> i32 {
        self.height - CHAT_FOOTER
    }

    pub fn offset(&self) -> usize {
        let max = self.last_rows.get().saturating_sub(self.visible_lines());
        (self.scroll + self.scroll_extra.get()).min(max)
    }

    fn settle(&mut self) {
        self.scroll = self.offset();
        self.scroll_extra.set(0);
    }

    pub fn scroll_by(&mut self, lines: i32) {
        self.settle();
        let max = self.last_rows.get().saturating_sub(self.visible_lines());
        self.scroll = (self.scroll as i64 + i64::from(lines)).clamp(0, max as i64) as usize;
    }

    pub fn scroll_press(&mut self, x: i32, y: i32) -> bool {
        self.settle();
        let Some(hit) = crate::list_scroll::hit(self.list_height(), x, y) else { return false };
        self.scroll_pressed = Some(hit);
        match hit {
            crate::list_scroll::ListScrollHit::Up => self.scroll_by(1),
            crate::list_scroll::ListScrollHit::Down => self.scroll_by(-1),
            crate::list_scroll::ListScrollHit::Track => {
                self.scroll_grab = crate::list_scroll::press_thumb(self.list_height(), self.last_rows.get(), self.visible_lines(), self.scroll, y - crate::list_scroll::TRACK_Y);
            }
        }
        true
    }

    pub fn scroll_drag_to(&mut self, y: i32) -> bool {
        let Some(grab) = self.scroll_grab else { return false };
        self.settle();
        if let Some(offset) = crate::list_scroll::drag(self.list_height(), self.last_rows.get(), self.visible_lines(), self.scroll, grab, y - crate::list_scroll::TRACK_Y) {
            self.scroll = offset;
        }
        true
    }

    pub fn background_rows(&self) -> i32 {
        ((self.height - CHAT_HEADER - CHAT_BELOW - CHAT_FOOTER) / CHAT_LINE).max(0)
    }

    pub fn size(&self) -> (i32, i32) {
        (CHAT_WIDTH, self.height)
    }

    pub fn select_tab(&mut self, tab: usize) {
        if tab >= self.tab_names.len() {
            return;
        }
        self.active_tab = tab;
        self.scroll = 0;
        self.scroll_extra.set(0);
        self.followed_rows.set(0);
        if self.filter_open {
            self.load_filter();
        }
        self.old_chat_tab_prefix();
    }

    fn old_chat_tab_prefix(&mut self) {
        use crate::chat_line::{is_tab_prefix, prefix_text, tab_prefix_kind};
        if !self.old_chat {
            return;
        }
        let first = self.input.text.chars().next();
        if self.active_tab == 0 {
            if first.is_some_and(is_tab_prefix) {
                self.input.delete_first();
            }
            return;
        }
        let wanted = tab_prefix_kind(self.active_tab);
        let has = first.and_then(|c| CHAT_PREFIXES.iter().find(|(p, _)| *p == c)).map(|(_, kind)| *kind);
        if has != Some(wanted) {
            if first.is_some_and(is_tab_prefix) {
                self.input.delete_first();
            }
            self.input.insert_at_start(&prefix_text(wanted));
        }
    }

    pub fn press(&mut self, key: InputKey) -> Option<String> {
        use crate::chat_line::{prefix_text, tab_prefix_kind};
        match key {
            InputKey::Up | InputKey::Down => {
                if let Some(text) = self.tells.browse(key == InputKey::Up) {
                    self.input.clear();
                    self.input.insert_at_cursor(&text);
                }
                None
            }
            InputKey::Enter => {
                self.tells.browsing = false;
                let line = self.input.key(InputKey::Enter);
                if line.is_some() && self.old_chat && self.active_tab > 0 {
                    self.input.insert_at_start(&prefix_text(tab_prefix_kind(self.active_tab)));
                }
                line
            }
            InputKey::Char('"') if self.input.before_cursor().is_empty() => {
                self.input.key(key);
                if let Some(text) = self.tells.begin(None) {
                    self.input.insert_at_cursor(&text);
                }
                None
            }
            _ => self.input.key(key),
        }
    }

    pub fn begin_tell(&mut self, name: &str) {
        if let Some(text) = self.tells.begin(Some(name)) {
            self.input.insert_at_cursor(&text);
        }
    }

    pub fn cycle_tab(&mut self, direction: i32) {
        let last = self.tab_names.len() - 1;
        let tab = if direction == 0 {
            if self.active_tab > 0 { self.active_tab - 1 } else { last }
        } else if self.active_tab < last {
            self.active_tab + 1
        } else {
            0
        };
        self.select_tab(tab);
    }

    pub fn tab_at(&self, x: i32, y: i32) -> Option<usize> {
        let footer_y = self.height - CHAT_FOOTER;
        if y < footer_y || y >= footer_y + TAB_SIZE.1 {
            return None;
        }
        TAB_X.iter().position(|tab_x| x >= *tab_x && x < tab_x + TAB_SIZE.0)
    }

    /// Retângulo do botão do martelo, relativo ao canto de cima da janela: `x = 5` e
    /// `y = altura - 20` (`0x1005b3a3`).
    pub fn option_rect(&self) -> Rect {
        OPTION_RECT.offset(0, self.height)
    }

    pub fn split_height(&self) -> i32 {
        self.height - CHAT_FOOTER
    }

    pub fn split_size(&self) -> (i32, i32) {
        (CHAT_WIDTH, self.split_height() + TAB_SIZE.1)
    }

    pub fn split_grip(&self, tab: usize) -> Rect {
        Rect::new(TAB_X[tab], self.split_height(), TAB_SIZE.0, TAB_SIZE.1)
    }

    pub fn split_window_id(tab: usize) -> u32 {
        SPLIT_WINDOW_BASE + tab as u32
    }

    pub fn split_tab_of(id: u32) -> Option<usize> {
        id.checked_sub(SPLIT_WINDOW_BASE).map(|tab| tab as usize).filter(|tab| (1..5).contains(tab))
    }

    pub fn first_attached(&self) -> Option<usize> {
        (0..5).find(|tab| !self.detached[*tab])
    }

    pub fn detach(&mut self, tab: usize) {
        if tab == 0 || tab >= 5 || self.detached[tab] {
            return;
        }
        self.detached[tab] = true;
        self.split_scroll[tab] = 0;
        if let Some(other) = self.first_attached() {
            self.select_tab(other);
        }
        self.select_tab(tab);
    }

    pub fn merge_touches(window_tab: Rect, bar_tab: Rect) -> bool {
        let corners = |rect: Rect| [(rect.x, rect.y), (rect.x + rect.w, rect.y), (rect.x, rect.y + rect.h), (rect.x + rect.w, rect.y + rect.h)];
        let inside = |rect: Rect, (x, y): (i32, i32)| x >= rect.x && x <= rect.x + rect.w && y >= rect.y && y <= rect.y + rect.h;
        corners(window_tab).into_iter().any(|corner| inside(bar_tab, corner)) || corners(bar_tab).into_iter().any(|corner| inside(window_tab, corner))
    }

    pub fn attach(&mut self, tab: usize) {
        if tab < 5 && self.detached[tab] {
            self.detached[tab] = false;
            self.select_tab(tab);
        }
    }

    pub fn split_scroll_by(&mut self, tab: usize, delta: i32, rows: usize) {
        let visible = ((self.split_height() - LIST_BOTTOM_INSET) / CHAT_LINE).max(0) as usize;
        let max = rows.saturating_sub(visible);
        self.split_scroll[tab] = (self.split_scroll[tab] as i64 + i64::from(delta)).clamp(0, max as i64) as usize;
    }

    pub fn paint_split(&self, list: &mut DrawList, x: i32, y: i32, tab: usize, font: &BitmapFont, alpha: f32) {
        let height = self.split_height();
        list.draw_texture(x, y, CHAT_WIDTH, CHAT_HEADER, 0, 0, CHAT_WIDTH, CHAT_HEADER, CHATTING_BACK2, alpha);
        for i in 0..((height - 36) / CHAT_LINE).max(0) {
            list.draw_texture(x, y + CHAT_HEADER + CHAT_LINE * i, CHAT_WIDTH, CHAT_LINE, 0, 0, CHAT_WIDTH, CHAT_LINE, CHATTING_BACK3, alpha);
        }
        list.draw_texture(x, y + height - CHAT_BELOW, CHAT_WIDTH, CHAT_BELOW, 0, 0, CHAT_WIDTH, CHAT_BELOW, CHATTING_BACK4, alpha);
        let rows = self.rows(&self.lists[tab], font);
        let visible = ((height - LIST_BOTTOM_INSET) / CHAT_LINE).max(0) as usize;
        let offset = self.split_scroll[tab].min(rows.len().saturating_sub(visible));
        let end = rows.len().saturating_sub(offset);
        let start = end.saturating_sub(visible);
        for (i, (indent, text, color)) in rows[start..end].iter().enumerate() {
            list.draw_text_clipped(x + LINE_TEXT_X + indent, y + LINE_TEXT_Y + CHAT_LINE * i as i32, CHAT_WIDTH - LINE_TEXT_X - indent, *color, text, font);
        }
        crate::list_scroll::paint(list, x, y, height, crate::list_scroll::ListScrollState { total: rows.len(), visible, offset, pressed: None });
        let grip = self.split_grip(tab);
        list.draw_texture(x + grip.x, y + grip.y, grip.w, grip.h, 0, 0, grip.w, grip.h, CHATTING_TAB_ON, 1.0);
        let name = &self.tab_names[tab];
        list.draw_text(x + grip.x + TAB_TEXT_CENTER_X - font.measure(name) as i32 / 2, y + grip.y + TAB_TEXT_Y, BUTTON_TEXT, name, font);
    }

    pub fn messenger_rect(&self) -> Rect {
        MESSENGER_RECT.offset(0, self.height)
    }

    /// Canto de cima do painel de filtro, relativo à janela de chat: ele fica acima dela
    /// (`SetPosSize(0, -191, 256, 191)` em `NCChatWnd::OnCreate`).
    pub fn filter_origin(&self) -> (i32, i32) {
        FILTER_PANEL_POS
    }

    /// O martelo: fechado, recarrega os filtros da aba ativa e abre; aberto, só fecha. É o que
    /// `FUN_10057630` faz, com o `LoadChatFilter` antes de mostrar.
    pub fn toggle_filter(&mut self) {
        self.filter_open = !self.filter_open;
        if self.filter_open {
            self.load_filter();
        }
    }

    pub fn load_filter(&mut self) {
        let tab = &self.filter.tabs[self.active_tab.min(self.filter.tabs.len() - 1)];
        for (row, (_, key, _, _)) in FILTER_ROWS.iter().enumerate() {
            self.filter_edit[row] = tab.shows(key);
        }
    }

    /// Uma linha só aceita clique quando a caixa que manda nela está marcada, e a linha do canal
    /// da própria aba fica travada (`FUN_10055170`, `0x100553a2` em diante).
    pub fn filter_enabled(&self, row: usize) -> bool {
        if FILTER_SYSTEM_CHILDREN.contains(&row) {
            return self.filter_edit[FILTER_SYSTEM];
        }
        if FILTER_CHAT_CHILDREN.contains(&row) {
            let locked = FILTER_TAB_LOCK[self.active_tab.min(FILTER_TAB_LOCK.len() - 1)];
            return self.filter_edit[FILTER_CHAT] && locked != Some(row);
        }
        true
    }

    pub fn filter_hit(&self, x: i32, y: i32) -> Option<FilterHit> {
        if !self.filter_open {
            return None;
        }
        let (px, py) = self.filter_origin();
        let (x, y) = (x - px, y - py);
        if FILTER_OK_RECT.contains(x, y) {
            return Some(FilterHit::Ok);
        }
        if FILTER_CANCEL_RECT.contains(x, y) {
            return Some(FilterHit::Cancel);
        }
        FILTER_ROWS.iter().position(|(_, _, _, rect)| rect.contains(x, y)).map(FilterHit::Row)
    }

    /// Clique no painel. `Ok` grava os doze valores na aba ativa e fecha; `Cancel` e o martelo
    /// fecham sem gravar. Devolve o nome da aba quando houve gravação, pro chamador salvar o
    /// `chatfilter.ini` como o original faz em `0x100555a0`.
    pub fn filter_click(&mut self, hit: FilterHit) -> Option<&'static str> {
        match hit {
            FilterHit::Row(row) if self.filter_enabled(row) => {
                self.filter_edit[row] = !self.filter_edit[row];
                None
            }
            FilterHit::Row(_) => None,
            FilterHit::Cancel => {
                self.filter_open = false;
                None
            }
            FilterHit::Ok => {
                self.filter_open = false;
                Some(self.apply_filter())
            }
        }
    }

    fn apply_filter(&mut self) -> &'static str {
        let tab = &mut self.filter.tabs[self.active_tab.min(FILTER_TAB_LOCK.len() - 1)];
        for (row, (_, key, _, _)) in FILTER_ROWS.iter().enumerate() {
            tab.filters.insert((*key).to_string(), self.filter_edit[row]);
        }
        tab.name
    }

    fn filter_boxes(&self) -> [CheckBox; FILTER_COUNT] {
        std::array::from_fn(|row| {
            let (_, _, _, rect) = FILTER_ROWS[row];
            CheckBox {
                rect,
                label: self.filter_labels[row].clone(),
                checked: self.filter_edit[row],
                enabled: self.filter_enabled(row),
                label_left: false,
            }
        })
    }

    /// Desenhado fora do `paint` porque o painel sai da janela e precisa ficar acima das outras.
    pub fn paint_filter(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        if !self.filter_open {
            return;
        }
        let (px, py) = self.filter_origin();
        let (x, y) = (x + px, y + py);
        let (w, h) = FILTER_PANEL_SIZE;
        list.draw_texture(x, y, w, h, 0, 0, w, h, CHATTING_BACK5, 1.0);
        for check in self.filter_boxes() {
            check.paint(list, x, y, font);
        }
        let ok = TextButton::new(FILTER_OK_RECT, BTN_NORMAL, BTN_DOWN, &self.filter_ok);
        ok.paint(list, x, y, self.pressed == Some(FilterHit::Ok), font);
        let cancel = TextButton::new(FILTER_CANCEL_RECT, BTN_NORMAL, BTN_DOWN, &self.filter_cancel);
        cancel.paint(list, x, y, self.pressed == Some(FilterHit::Cancel), font);
    }

    pub fn shows(&self, entry: &ChatEntry) -> bool {
        self.prefiltered || self.shows_in(self.active_tab, entry)
    }

    pub fn ingest(&mut self, entry: &ChatEntry) {
        for tab in 0..self.lists.len() {
            if self.shows_in(tab, entry) {
                self.lists[tab].push(entry.clone());
                if self.lists[tab].len() > LIST_CAPACITY {
                    self.lists[tab].remove(0);
                }
            }
        }
    }

    pub fn clear_lists(&mut self) {
        self.lists.iter_mut().for_each(Vec::clear);
    }

    pub fn active_list(&self) -> &[ChatEntry] {
        &self.lists[self.active_tab.min(self.lists.len() - 1)]
    }

    pub fn shows_in(&self, tab_index: usize, entry: &ChatEntry) -> bool {
        let Some(tab) = self.filter.tabs.get(tab_index) else { return true };
        if entry.kind == CHAT_SYSTEM {
            return match entry.group {
                SYSTEM_GROUP_BROADCAST | SYSTEM_GROUP_PETITION => true,
                SYSTEM_GROUP_DAMAGE => tab.shows("damage") && tab.shows("system"),
                SYSTEM_GROUP_USE_ITEMS => tab.shows("useitems") && tab.shows("system"),
                _ => tab.shows("system"),
            };
        }
        match entry.kind {
            6 | 7 | 10 | 18 => true,
            _ => filter_name(entry.kind).is_some_and(|name| tab.shows(name) && tab.shows("chat")),
        }
    }

    pub fn filtered<'a>(&self, lines: &'a [ChatEntry]) -> Vec<&'a ChatEntry> {
        lines.iter().filter(|line| self.shows(line)).collect()
    }

    fn rows_key(&self, lines: &[ChatEntry], font: &BitmapFont) -> u64 {
        rows_hash(lines.iter().filter(|entry| self.shows(entry)), font)
    }

    pub fn cached_rows(&self, lines: &[ChatEntry], font: &BitmapFont) -> std::sync::Arc<Vec<ChatRow>> {
        self.rows_cache.get_or_build(self.rows_key(lines, font), || self.rows(lines, font))
    }

    pub fn rows(&self, lines: &[ChatEntry], font: &BitmapFont) -> Vec<(i32, String, Argb)> {
        let width = CHAT_WIDTH - LINE_WRAP_INSET;
        self.filtered(lines)
            .into_iter()
            .flat_map(|entry| {
                let color = entry.color();
                wrap_line(&entry.line(), width, LINE_INDENT, font).into_iter().map(move |(indent, text)| (indent, text, color))
            })
            .collect()
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, lines: &[ChatEntry], font: &BitmapFont, alpha: f32) {
        let below_y = y + self.height - CHAT_FOOTER - CHAT_BELOW;
        let footer_y = y + self.height - CHAT_FOOTER;
        list.draw_texture(x, y, CHAT_WIDTH, CHAT_HEADER, 0, 0, CHAT_WIDTH, CHAT_HEADER, CHATTING_BACK2, alpha);
        for i in 0..self.background_rows() {
            let row_y = y + CHAT_HEADER + CHAT_LINE * i;
            list.draw_texture(x, row_y, CHAT_WIDTH, CHAT_LINE, 0, 0, CHAT_WIDTH, CHAT_LINE, CHATTING_BACK3, alpha);
        }
        list.draw_texture(x, below_y, CHAT_WIDTH, CHAT_BELOW, 0, 0, CHAT_WIDTH, CHAT_BELOW, CHATTING_BACK4, alpha);
        list.draw_texture(x, footer_y, CHAT_WIDTH, CHAT_FOOTER, 0, 0, CHAT_WIDTH, CHAT_FOOTER, CHATTING_BACK1, alpha);

        let (gw, gh) = SIZE_CONTROL_SIZE;
        list.draw_texture(x + SIZE_CONTROL_POS.0, y + SIZE_CONTROL_POS.1, gw, gh, 0, 0, gw, gh, CHATTING_SIZE_CONTROL, 1.0);
        let (tw, th) = TAB_SIZE;
        for (i, tab_x) in TAB_X.iter().enumerate() {
            let texture = if i == self.active_tab { CHATTING_TAB_ON } else { CHATTING_TAB_OFF };
            list.draw_texture(x + tab_x, footer_y, tw, th, 0, 0, tw, th, texture, 1.0);
            let name = &self.tab_names[i];
            let text_x = x + tab_x + TAB_TEXT_CENTER_X - font.measure(name) as i32 / 2;
            list.draw_text(text_x, footer_y + TAB_TEXT_Y, BUTTON_TEXT, name, font);
        }
        for (i, tab_x) in TAB_X.iter().enumerate() {
            if self.detached[i] {
                list.draw_texture(x + tab_x, footer_y, tw, th, 0, 0, tw, th, CHATTING_TAB_SPLIT, 1.0);
            }
        }
        let icon_y = y + self.height - IME_BOTTOM_INSET;
        list.draw_texture(x + IME_X, icon_y, IME_SIZE, IME_SIZE, 0, 0, IME_SIZE, IME_SIZE, CHATTING_IME_EN, 1.0);
        let option = self.option_rect();
        let option_texture = if self.filter_open { CHATTING_OPTION_DOWN } else { CHATTING_OPTION };
        list.draw_texture(x + option.x, y + option.y, option.w, option.h, 0, 0, option.w, option.h, option_texture, 1.0);
        let messenger = self.messenger_rect();
        list.draw_texture(x + messenger.x, y + messenger.y, messenger.w, messenger.h, 0, 0, messenger.w, messenger.h, MESSENGER_NORMAL, 1.0);

        let empty = self.detached[self.active_tab.min(4)];
        let rows = if empty { std::sync::Arc::new(Vec::new()) } else { self.cached_rows(lines, font) };
        let visible = self.visible_lines();
        self.last_rows.set(rows.len());
        let previous = self.followed_rows.replace(rows.len());
        if rows.len() > previous && previous > 0 && self.scroll + self.scroll_extra.get() > 0 {
            self.scroll_extra.set(self.scroll_extra.get() + rows.len() - previous);
        }
        let offset = (self.scroll + self.scroll_extra.get()).min(rows.len().saturating_sub(visible));
        let end = rows.len().saturating_sub(offset);
        let start = end.saturating_sub(visible);
        for (i, (indent, text, color)) in rows[start..end].iter().enumerate() {
            let row_y = y + CHAT_LINE * i as i32 + LINE_TEXT_Y;
            list.draw_text_clipped(x + LINE_TEXT_X + indent, row_y, CHAT_WIDTH - LINE_TEXT_X - indent, *color, text, font);
        }
        crate::list_scroll::paint(list, x, y, self.list_height(), crate::list_scroll::ListScrollState { total: rows.len(), visible, offset, pressed: self.scroll_pressed });

        let input_x = x + INPUT_X + INPUT_TEXT_INSET;
        let input_y = y + self.height - INPUT_BOTTOM_INSET + INPUT_TEXT_INSET;
        let input_width = INPUT_WIDTH - INPUT_TEXT_INSET;
        list.draw_text_clipped(input_x, input_y, input_width, TEXT, &self.input.text, font);
        if self.focused {
            let caret_x = input_x + font.measure(self.input.before_cursor()) as i32 - CARET_BACK;
            list.draw_text(caret_x, input_y, TEXT, &CARET.to_string(), font);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::Quad;
    use crate::font::{Glyphs, gly};
    use crate::ini::parse_ini;

    fn sent(typed: &str, tab: usize) -> Option<(i32, String, String)> {
        chat_message(typed, tab).map(|m| (m.kind, m.target, m.text))
    }

    /// Sem prefixo, o canal é o da aba ativa — o `switch` sobre `+0x1d0` em `FUN_10058330`.
    #[test]
    fn resize_10054da0_quantizes_by_fifteen_and_keeps_the_native_limits() {
        let mut view = view();
        assert_eq!(view.resize_by(14), None);
        assert_eq!((view.height, view.resize_accumulator), (187, 14));
        assert_eq!(view.resize_by(1), Some(15));
        assert_eq!((view.height, view.resize_accumulator), (202, 0));
        assert_eq!(view.resize_by(-44), None);
        assert_eq!((view.height, view.resize_accumulator), (202, 0));
        assert_eq!(view.resize_by(-29), Some(-15));
        assert_eq!((view.height, view.resize_accumulator), (187, 0));
        view.height = 490;
        assert_eq!(view.resize_by(15), None);
        assert_eq!((view.height, view.resize_accumulator), (490, 0));
        assert_eq!(view.resize_by(-29), Some(-15));
        assert_eq!(view.height, 475);
    }

    #[test]
    fn a_line_without_prefix_goes_to_the_channel_of_the_active_tab() {
        assert_eq!(sent("oi", 0), Some((0, String::new(), "oi".into())), "All");
        assert_eq!(sent("oi", 1), Some((8, String::new(), "oi".into())), "+Trade");
        assert_eq!(sent("oi", 2), Some((3, String::new(), "oi".into())), "#Party");
        assert_eq!(sent("oi", 3), Some((4, String::new(), "oi".into())), "@Clan");
        assert_eq!(sent("oi", 4), Some((9, String::new(), "oi".into())), "$Alliance");
    }

    /// O prefixo troca o canal e sai do texto; vale também na variante de largura dupla.
    #[test]
    fn a_prefix_changes_the_channel_and_is_stripped_from_the_text() {
        assert_eq!(sent("!grito", 0), Some((1, String::new(), "grito".into())));
        assert_eq!(sent("#grupo", 0), Some((3, String::new(), "grupo".into())));
        assert_eq!(sent("@cla", 0), Some((4, String::new(), "cla".into())));
        assert_eq!(sent("&pet", 0), Some((6, String::new(), "pet".into())));
        assert_eq!(sent("*gm", 0), Some((7, String::new(), "gm".into())));
        assert_eq!(sent("+venda", 0), Some((8, String::new(), "venda".into())));
        assert_eq!(sent("$ali", 0), Some((9, String::new(), "ali".into())));
        assert_eq!(sent("＃largura dupla", 0), Some((3, String::new(), "largura dupla".into())));
        // o prefixo vence a aba
        assert_eq!(sent("!grito", 2), Some((1, String::new(), "grito".into())));
        assert_eq!(sent("%heroi", 0), Some((17, String::new(), "heroi".into())));
        // e o `~` volta pro canal geral mesmo numa aba de outro canal
        assert_eq!(sent("~geral", 3), Some((0, String::new(), "geral".into())));
    }

    /// Sussurro: o nome vai do prefixo até o primeiro espaço, e o resto é a mensagem.
    #[test]
    fn a_whisper_takes_the_name_up_to_the_first_space() {
        assert_eq!(sent("\"Fulano oi tudo bem", 0), Some((CHAT_TELL, "Fulano".into(), "oi tudo bem".into())));
        // espaço largo também fecha o nome
        assert_eq!(sent("\"Fulano\u{3000}oi", 0), Some((CHAT_TELL, "Fulano".into(), "oi".into())));
        // sem mensagem depois do nome o original não manda nada
        assert_eq!(sent("\"Fulano ", 0), None);
        assert_eq!(sent("\"Fulano", 0), Some((CHAT_TELL, String::new(), "\"Fulano".into())), "sem espaço não há nome");
    }

    /// Linha vazia, ou que só tem o prefixo, não vira pacote.
    #[test]
    fn an_empty_line_is_not_sent() {
        assert_eq!(sent("", 0), None);
        assert_eq!(sent("#", 0), None);
        assert_eq!(sent("!", 2), None);
    }

    const TEXTURES: &str = include_str!("../tests/fixtures/nwindow-texturas-chat.txt");
    const CHAT_FILTER: &str = include_str!("../tests/fixtures/chatfilter.ini");

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        let bytes = gly(&[(0x20, &glyphs)], (1024, 128));
        BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&bytes).unwrap())
    }

    #[test]
    fn cached_rows_are_reused_until_the_lines_tab_or_font_change() {
        let font = font();
        let mut view = view();
        let mut lines = lines();
        let first = view.cached_rows(&lines, &font);
        assert_eq!(*first, view.rows(&lines, &font));
        assert!(std::sync::Arc::ptr_eq(&first, &view.cached_rows(&lines, &font)));
        lines.push(entry(0, "Novo", "uma mensagem que ainda não existia"));
        let second = view.cached_rows(&lines, &font);
        assert!(!std::sync::Arc::ptr_eq(&first, &second));
        assert_eq!(*second, view.rows(&lines, &font));
        view.active_tab = 4;
        let other_tab = view.cached_rows(&lines, &font);
        assert_eq!(*other_tab, view.rows(&lines, &font));
        let other_font = BitmapFont::new("outra".into(), font.glyphs.clone());
        assert!(!std::sync::Arc::ptr_eq(&other_tab, &view.cached_rows(&lines, &other_font)));
    }

    fn wrap_line_reference(text: &str, width: i32, indent: i32, font: &BitmapFont) -> Vec<(i32, String)> {
        let mut rows: Vec<(i32, String)> = Vec::new();
        let mut current = String::new();
        let mut current_indent = 0;
        for c in text.chars() {
            let candidate = format!("{current}{c}");
            if font.measure(&candidate) as i32 + current_indent <= width || current.is_empty() {
                current = candidate;
                continue;
            }
            let (kept, rest) = match current.rfind(' ') {
                Some(space) => (current[..space].to_string(), format!("{}{c}", &current[space + 1..])),
                None => (current.clone(), c.to_string()),
            };
            rows.push((current_indent, kept));
            current = rest;
            current_indent = indent;
        }
        rows.push((current_indent, current));
        rows
    }

    #[test]
    fn incremental_wrap_matches_the_remeasuring_wrap_on_every_shape_of_text() {
        let font = font();
        let long = "palavra ".repeat(60);
        let samples = [
            "",
            "x",
            "a b c d e f g h i j k l m n o p q r s t u v w x y z",
            "semespacoalgumqueprecisaquebrarnomeiodapalavramuitolongaqueexcedeaslinhas",
            "linha um\nlinha dois muito comprida que passa do limite da largura configurada\nfim",
            "com retorno\r\ne outra linha",
            "tabs\te acentos \u{e9}\u{e3}\u{fc} fora do atlas \u{4e2d}\u{6587} no meio",
            "  espacos  duplos   e finais   ",
            "\n\n\n",
            "a\n",
            long.as_str(),
        ];
        for text in samples {
            for width in [20, 60, 97, 326] {
                for indent in [0, 12] {
                    assert_eq!(wrap_line(text, width, indent, &font), wrap_line_reference(text, width, indent, &font), "{text:?} {width} {indent}");
                }
            }
        }
    }

    fn view() -> ChatView {
        ChatView::new(187, ChatFilter::from_ini(&parse_ini(CHAT_FILTER)))
    }

    fn entry(kind: i32, name: &str, text: &str) -> ChatEntry {
        ChatEntry::new(kind, name, text)
    }

    fn lines() -> Vec<ChatEntry> {
        vec![entry(0, "Killer", "ola"), entry(1, "Luzal", "wtb sls"), entry(2, "Bob", "psiu")]
    }

    fn with_texture<'a>(quads: &'a [Quad], texture: &str) -> Vec<&'a Quad> {
        quads.iter().filter(|q| q.texture == texture).collect()
    }

    fn painted(view: &ChatView, lines: &[ChatEntry], alpha: f32) -> Vec<Quad> {
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, lines, &font(), alpha);
        view.paint_filter(&mut list, 0, 0, &font());
        list.quads
    }

    fn text_at<'a>(quads: &'a [Quad], font: &BitmapFont, y: f32) -> Vec<&'a Quad> {
        quads.iter().filter(|q| q.texture == font.atlas && q.y == y).collect()
    }

    #[test]
    fn default_height_shows_eight_lines_over_seven_background_rows() {
        assert_eq!(view().visible_lines(), 8);
        assert_eq!(view().background_rows(), 7);
        assert_eq!(ChatView::new(187 + 14, view().filter).visible_lines(), 9);
        assert_eq!(ChatView::new(10, view().filter).visible_lines(), 0);
        assert_eq!(view().size(), (CHAT_WIDTH, 187));
    }

    #[test]
    fn backgrounds_stack_header_lines_bottom_and_footer() {
        let quads = painted(&view(), &lines(), 0.5);
        let header = with_texture(&quads, CHATTING_BACK2);
        assert_eq!(header.len(), 1);
        assert_eq!((header[0].x, header[0].y, header[0].w, header[0].h), (0.0, 0.0, 348.0, 18.0));
        assert_eq!((header[0].u, header[0].v, header[0].uw, header[0].vh), (0.0, 0.0, 348.0, 18.0));
        assert_eq!(header[0].color[3], 0.5);
        let rows = with_texture(&quads, CHATTING_BACK3);
        assert_eq!(rows.len(), 7);
        for (i, row) in rows.iter().enumerate() {
            assert_eq!((row.x, row.y, row.w, row.h), (0.0, 18.0 + 15.0 * i as f32, 348.0, 15.0));
            assert_eq!((row.uw, row.vh, row.color[3]), (348.0, 15.0, 0.5));
        }
        let below = with_texture(&quads, CHATTING_BACK4);
        assert_eq!((below[0].y, below[0].h, below[0].vh, below[0].color[3]), (123.0, 18.0, 18.0, 0.5));
        let footer = with_texture(&quads, CHATTING_BACK1);
        assert_eq!((footer[0].y, footer[0].w, footer[0].h), (141.0, 348.0, 46.0));
        assert_eq!((footer[0].uw, footer[0].vh, footer[0].color[3]), (348.0, 46.0, 0.5));
    }

    #[test]
    fn the_size_control_is_a_resize_frame_that_feeds_resize_by_0x10054da0() {
        let mut view = view();
        assert!(!view.size_press(100, 100));
        assert!(view.size_press(10, 2));
        assert_eq!(view.size_frame.grab, (10, 5));
        view.height = 300;
        assert_eq!(view.size_move(10, 2 - 14), None);
        assert_eq!(view.resize_accumulator, -14);
        assert_eq!(view.size_move(10, 2 - 16), Some(-30));
        assert_eq!(view.height, 270);
        view.size_release();
        assert!(!view.size_frame.dragging);
        assert_eq!(view.size_move(10, 2 - 40), None);
    }

    #[test]
    fn tabs_sit_on_the_footer_with_the_active_texture_and_centered_names() {
        let font = font();
        let quads = painted(&view(), &[], 0.5);
        let grip = with_texture(&quads, CHATTING_SIZE_CONTROL);
        assert_eq!((grip[0].x, grip[0].y, grip[0].w, grip[0].h), (0.0, -3.0, 24.0, 10.0));
        assert_eq!((grip[0].uw, grip[0].vh, grip[0].color[3]), (24.0, 10.0, 1.0));
        let on = with_texture(&quads, CHATTING_TAB_ON);
        assert_eq!(on.len(), 1);
        assert_eq!((on[0].x, on[0].y, on[0].w, on[0].h, on[0].uw, on[0].vh, on[0].color[3]), (23.0, 141.0, 64.0, 23.0, 64.0, 23.0, 1.0));
        let off = with_texture(&quads, CHATTING_TAB_OFF);
        let xs: Vec<f32> = off.iter().map(|q| q.x).collect();
        assert_eq!(xs, [87.0, 151.0, 215.0, 279.0]);
        let names = text_at(&quads, &font, 147.0);
        assert_eq!(names.len(), TAB_NAMES.iter().map(|n| n.len()).sum::<usize>());
        assert_eq!(names[0].x, (23 + 34 - 9) as f32);
        assert!(names.iter().all(|q| q.color == BUTTON_TEXT.rgba()));
        let mut second = view();
        second.active_tab = 1;
        let quads = painted(&second, &[], 1.0);
        assert_eq!(with_texture(&quads, CHATTING_TAB_ON)[0].x, 87.0);
        assert_eq!(view().tab_at(30, 150), Some(0));
        assert_eq!(view().tab_at(300, 163), Some(4));
        assert_eq!(view().tab_at(10, 150), None);
        assert_eq!(view().tab_at(30, 170), None);
    }

    #[test]
    fn channel_icon_sits_twenty_pixels_above_the_bottom() {
        let quads = painted(&view(), &[], 0.5);
        let icon = with_texture(&quads, CHATTING_IME_EN);
        assert_eq!((icon[0].x, icon[0].y, icon[0].w, icon[0].h), (24.0, 167.0, 15.0, 15.0));
        assert_eq!((icon[0].uw, icon[0].vh, icon[0].color[3]), (15.0, 15.0, 1.0));
    }

    #[test]
    fn lines_are_name_colon_text_in_the_channel_color_from_the_top_left() {
        let font = font();
        let mut list = DrawList::default();
        view().paint(&mut list, 10, 20, &lines(), &font, 1.0);
        let row = |i: usize| text_at(&list.quads, &font, 20.0 + LINE_TEXT_Y as f32 + 15.0 * i as f32);
        let first = row(0);
        assert_eq!(first.len(), "Killer: ola".len() - 1);
        assert_eq!(first[0].x, 10.0 + LINE_TEXT_X as f32);
        assert!(first.iter().all(|q| q.color == chat_color(0).rgba()));
        assert!(row(1).iter().all(|q| q.color == chat_color(1).rgba()));
        assert_eq!(row(1).len(), "Luzal: wtb sls".len() - 2);
        assert!(row(2).iter().all(|q| q.color == chat_color(2).rgba()));
        assert!(row(3).is_empty());
    }

    #[test]
    fn critical_announce_and_system_lines_show_only_the_text_with_their_own_color() {
        let font = font();
        let mut list = DrawList::default();
        let mut system = entry(CHAT_SYSTEM, "", "Welcome");
        system.color = Some(Argb(0xFFB09B79));
        let lines = [entry(18, "GM", "Boss"), system];
        view().paint(&mut list, 0, 0, &lines, &font, 1.0);
        let row = |i: usize| text_at(&list.quads, &font, LINE_TEXT_Y as f32 + 15.0 * i as f32);
        assert_eq!(row(0).len(), 4);
        assert!(row(0).iter().all(|q| q.color == chat_color(18).rgba()));
        assert_eq!(row(1).len(), 7);
        assert!(row(1).iter().all(|q| q.color == Argb(0xFFB09B79).rgba()));
        assert_eq!(entry(CHAT_SYSTEM, "", "x").color(), TEXT);
    }

    #[test]
    fn long_lines_wrap_at_the_width_minus_twenty_two_with_indented_continuations() {
        let font = font();
        let rows = wrap_line(&"x".repeat(100), 326, 12, &font);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], (0, "x".repeat(54)));
        assert_eq!(rows[1], (12, "x".repeat(46)));
        let rows = wrap_line(&format!("{} {}", "a".repeat(50), "b".repeat(10)), 326, 12, &font);
        assert_eq!(rows[0], (0, "a".repeat(50)));
        assert_eq!(rows[1], (12, "b".repeat(10)));
        assert_eq!(wrap_line("", 326, 12, &font), [(0, String::new())]);
        let mut list = DrawList::default();
        view().paint(&mut list, 0, 0, &[entry(0, "a", &"x".repeat(100))], &font, 1.0);
        let second = text_at(&list.quads, &font, LINE_TEXT_Y as f32 + 15.0);
        assert_eq!(second[0].x, 12.0 + LINE_TEXT_X as f32);
        assert!(list.quads.iter().filter(|q| q.texture == font.atlas && q.y < 141.0).all(|q| q.x + q.w <= 326.0 + LINE_TEXT_X as f32));
    }

    #[test]
    fn the_scroll_column_moves_the_view_by_buttons_and_wheel_inside_the_painted_rows() {
        let font = font();
        let many: Vec<ChatEntry> = (0..12).map(|i| entry(0, "n", &i.to_string())).collect();
        let mut view = view();
        view.paint(&mut DrawList::default(), 0, 0, &many, &font, 1.0);
        let up = crate::list_scroll::up_rect();
        let down = crate::list_scroll::down_rect(view.list_height());
        assert!(view.scroll_press(up.x + 2, up.y + 2));
        assert_eq!((view.scroll, view.scroll_pressed), (1, Some(crate::list_scroll::ListScrollHit::Up)));
        view.scroll_by(50);
        assert_eq!(view.scroll, 12 - view.visible_lines());
        assert!(view.scroll_press(down.x + 2, down.y + 2));
        assert_eq!(view.scroll, 12 - view.visible_lines() - 1);
        view.scroll_by(-50);
        assert_eq!(view.scroll, 0);
        assert!(!view.scroll_press(100, 60));
    }

    #[test]
    fn dragging_the_thumb_scrolls_the_chat_and_releasing_stops_it() {
        let font = font();
        let many: Vec<ChatEntry> = (0..40).map(|i| entry(0, "n", &i.to_string())).collect();
        let mut view = view();
        view.paint(&mut DrawList::default(), 0, 0, &many, &font, 1.0);
        let track = crate::list_scroll::track_rect(view.list_height());
        let thumb_y = track.y + (track.h - 15).max(0);
        assert!(view.scroll_press(track.x + 3, thumb_y + 2));
        assert!(view.scroll_grab.is_some());
        assert!(view.scroll_drag_to(track.y));
        assert_eq!(view.scroll, 40 - view.visible_lines());
        view.scroll_grab = None;
        assert!(!view.scroll_drag_to(track.y + 20));
    }

    #[test]
    fn only_the_last_visible_rows_are_shown_minus_scroll() {
        let font = font();
        let many: Vec<ChatEntry> = (0..12).map(|i| entry(0, "n", &i.to_string())).collect();
        let first_glyph_u = |view: &ChatView| -> Vec<f32> {
            let mut list = DrawList::default();
            view.paint(&mut list, 0, 0, &many, &font, 1.0);
            list.quads.iter().filter(|q| q.texture == font.atlas && q.x == 18.0 + LINE_TEXT_X as f32 && q.y < 141.0).map(|q| q.u).collect()
        };
        let digit = |c: char| -> f32 { font.glyph(c).unwrap().x as f32 };
        assert_eq!(first_glyph_u(&view()), ['4', '5', '6', '7', '8', '9', '1', '1'].map(digit));
        let mut scrolled = view();
        scrolled.scroll = 2;
        assert_eq!(first_glyph_u(&scrolled), ['2', '3', '4', '5', '6', '7', '8', '9'].map(digit));
        scrolled.scroll = 50;
        assert_eq!(first_glyph_u(&scrolled), ['0', '1', '2', '3', '4', '5', '6', '7'].map(digit));
    }

    #[test]
    fn input_text_is_drawn_in_the_edit_box_with_a_caret_when_focused() {
        let font = font();
        let mut view = view();
        view.focused = true;
        for c in "hi".chars() {
            view.input.key(InputKey::Char(c));
        }
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &[], &font, 1.0);
        let text = text_at(&list.quads, &font, 167.0);
        assert_eq!(text.len(), 3);
        assert_eq!((text[0].x, text[0].y), (41.0, 167.0));
        assert_eq!(text[2].x, 41.0 + 12.0 - 1.0);
        assert_eq!(text[2].u, font.glyph('|').unwrap().x as f32);
        assert!(text.iter().all(|q| q.color == TEXT.rgba()));
        view.input.key(InputKey::Left);
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &[], &font, 1.0);
        assert_eq!(text_at(&list.quads, &font, 167.0)[2].x, 41.0 + 6.0 - 1.0);
        view.focused = false;
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &[], &font, 1.0);
        assert_eq!(text_at(&list.quads, &font, 167.0).len(), 2);
        let mut empty = DrawList::default();
        ChatView::new(187, view.filter.clone()).paint(&mut empty, 0, 0, &[], &font, 1.0);
        assert!(text_at(&empty.quads, &font, 167.0).is_empty());
    }

    #[test]
    fn text_input_edits_and_submits() {
        let mut input = TextInput::default();
        for c in "abc".chars() {
            assert_eq!(input.key(InputKey::Char(c)), None);
        }
        assert_eq!(input.key(InputKey::Backspace), None);
        assert_eq!((input.text.as_str(), input.cursor), ("ab", 2));
        assert_eq!(input.key(InputKey::Left), None);
        input.key(InputKey::Char('x'));
        assert_eq!((input.text.as_str(), input.cursor), ("axb", 2));
        assert_eq!(input.before_cursor(), "ax");
        input.key(InputKey::Right);
        input.key(InputKey::Right);
        assert_eq!(input.cursor, 3);
        input.key(InputKey::Left);
        input.key(InputKey::Left);
        input.key(InputKey::Left);
        input.key(InputKey::Left);
        assert_eq!(input.cursor, 0);
        assert_eq!(input.key(InputKey::Backspace), None);
        assert_eq!(input.text, "axb");
        assert_eq!(input.key(InputKey::Enter), Some("axb".to_string()));
        assert_eq!((input.text.as_str(), input.cursor), ("", 0));
        assert_eq!(input.key(InputKey::Enter), None);
        input.key(InputKey::Char('é'));
        input.key(InputKey::Char('\u{8}'));
        assert_eq!(input.before_cursor(), "é");
        assert_eq!(input.key(InputKey::Escape), None);
        assert_eq!(input.text, "");
    }

    #[test]
    fn party_tab_shows_party_and_hides_trade() {
        let mut view = view();
        view.active_tab = 2;
        let lines = [
            entry(3, "a", "p"),
            entry(8, "b", "t"),
            entry(0, "c", "n"),
            entry(2, "d", "w"),
            entry(17, "e", "h"),
            entry(CHAT_SYSTEM, "", "s"),
            entry(14, "f", "?"),
        ];
        let shown: Vec<i32> = view.filtered(&lines).iter().map(|l| l.kind).collect();
        assert_eq!(shown, [3, 2, 17]);
        view.active_tab = 0;
        assert_eq!(view.filtered(&lines).len(), 6);
        view.active_tab = 1;
        assert!(view.filtered(&lines).is_empty());
        view.filter.tabs[1].filters.insert("chat".to_string(), true);
        let market: Vec<i32> = view.filtered(&lines).iter().map(|l| l.kind).collect();
        assert_eq!(market, [3, 8, 0, 2, 17]);
        view.active_tab = 9;
        assert_eq!(view.filtered(&lines).len(), 7);
    }

    #[test]
    fn system_messages_route_by_group_through_the_system_damage_and_useitems_filters() {
        let mut view = view();
        let with_group = |group: u32| ChatEntry { group, ..entry(CHAT_SYSTEM, "", "s") };
        view.active_tab = 0;
        assert!(view.shows(&with_group(0)));
        assert!(view.shows(&with_group(3)));
        assert!(view.shows(&with_group(7)));
        view.active_tab = 3;
        assert!(!view.shows(&with_group(0)));
        assert!(!view.shows(&with_group(3)));
        assert!(!view.shows(&with_group(7)));
        assert!(view.shows(&with_group(2)));
    }

    /// O martelo fica em `x = 5`, `altura - 20`, 15x15 (`0x1005b3a3`), e troca de arte quando o
    /// painel está aberto.
    #[test]
    fn the_hammer_sits_at_the_left_of_the_input_line_and_opens_the_filter_panel() {
        let quads = painted(&view(), &[], 0.5);
        let button = with_texture(&quads, CHATTING_OPTION);
        assert_eq!((button[0].x, button[0].y, button[0].w, button[0].h), (5.0, 167.0, 15.0, 15.0));
        assert_eq!((button[0].uw, button[0].vh, button[0].color[3]), (15.0, 15.0, 1.0));
        assert!(with_texture(&quads, CHATTING_BACK5).is_empty());
        let mut view = view();
        view.toggle_filter();
        assert!(view.filter_open);
        let quads = painted(&view, &[], 0.5);
        assert_eq!(with_texture(&quads, CHATTING_OPTION).len(), 0);
        assert_eq!(with_texture(&quads, CHATTING_OPTION_DOWN).len(), 1);
        let back = with_texture(&quads, CHATTING_BACK5);
        assert_eq!((back[0].x, back[0].y, back[0].w, back[0].h), (0.0, -191.0, 256.0, 191.0));
        assert_eq!((back[0].uw, back[0].vh, back[0].color[3]), (256.0, 191.0, 1.0));
        view.toggle_filter();
        assert!(!view.filter_open);
        assert!(with_texture(&painted(&view, &[], 1.0), CHATTING_BACK5).is_empty());
    }

    /// `FUN_10053cd0` (Alt+PageUp = 0, Alt+PageDown = 1): `NCTabCtrl::SelectTab` da aba vizinha, com volta de 0 a 4.
    #[test]
    fn alt_pageup_and_pagedown_cycle_the_five_chat_tabs() {
        let mut view = view();
        view.scroll = 7;
        view.cycle_tab(0);
        assert_eq!((view.active_tab, view.scroll), (4, 0));
        view.cycle_tab(0);
        assert_eq!(view.active_tab, 3);
        view.cycle_tab(1);
        assert_eq!(view.active_tab, 4);
        view.cycle_tab(1);
        assert_eq!(view.active_tab, 0);
        view.filter_open = true;
        view.cycle_tab(1);
        assert_eq!(view.active_tab, 1);
        assert!(view.filter_open);
        view.select_tab(9);
        assert_eq!(view.active_tab, 1);
    }

    /// Abrir carrega os doze valores da aba ativa, e cada caixa fica no lugar do original.
    #[test]
    fn the_panel_shows_the_twelve_filters_of_the_active_tab_at_the_original_coordinates() {
        let font = font();
        let mut view = view();
        view.toggle_filter();
        let expected: Vec<bool> = FILTER_ROWS.iter().map(|(_, key, _, _)| view.filter.tabs[0].shows(key)).collect();
        assert_eq!(view.filter_edit.to_vec(), expected);
        assert!(view.filter_edit.iter().all(|on| *on), "entire_tab liga tudo");
        let mut list = DrawList::default();
        view.paint(&mut list, 10, 20, &[], &font, 1.0);
        view.paint_filter(&mut list, 10, 20, &font);
        let checked = with_texture(&list.quads, crate::button::CHECK_BOX_CHECKED);
        assert_eq!(checked.len(), FILTER_COUNT);
        let places: Vec<(f32, f32)> = checked.iter().map(|q| (q.x, q.y)).collect();
        let wanted: Vec<(f32, f32)> =
            FILTER_ROWS.iter().map(|(_, _, _, r)| ((10 + r.x) as f32, (20 - 191 + r.y) as f32)).collect();
        assert_eq!(places, wanted);
        assert!(checked.iter().all(|q| (q.w, q.h) == (12.0, 12.0)));
        let labels: Vec<&str> = FILTER_ROWS.iter().map(|(_, _, label, _)| *label).collect();
        assert_eq!(labels, ["System", "Damage", "Chat", "General", "Shout", "Clan", "Party", "Trade", "Whisper", "Alliance", "Consumables", "Hero Voice"]);
        let ids: Vec<u32> = FILTER_ROWS.iter().map(|(id, _, _, _)| *id).collect();
        assert_eq!(ids, [0x164, 0x165, 0x166, 0x17f, 0x167, 0x168, 0x169, 0x16a, 0x16b, 0x250, 0x31c, 0x4bb]);
        let buttons = with_texture(&list.quads, BTN_NORMAL);
        assert_eq!(buttons.len(), 2);
        assert_eq!((buttons[0].x, buttons[0].y, buttons[0].w, buttons[0].h), (61.0, -11.0, 76.0, 23.0));
        assert_eq!(buttons[1].x, 141.0);
    }

    /// System manda em Damage e Consumables; Chat manda nas outras oito, menos a do canal da
    /// própria aba, que fica travada.
    #[test]
    fn the_parent_boxes_turn_the_children_off_and_the_tab_channel_stays_locked() {
        let mut view = view();
        view.toggle_filter();
        assert!(view.filter_enabled(1) && view.filter_enabled(10));
        view.filter_click(FilterHit::Row(FILTER_SYSTEM));
        assert!(!view.filter_edit[FILTER_SYSTEM]);
        assert!(!view.filter_enabled(1) && !view.filter_enabled(10));
        view.filter_click(FilterHit::Row(1));
        assert!(view.filter_edit[1], "caixa desligada não muda de valor");
        view.filter_click(FilterHit::Row(FILTER_CHAT));
        assert!(FILTER_CHAT_CHILDREN.iter().all(|row| !view.filter_enabled(*row)));
        view.filter_click(FilterHit::Row(FILTER_CHAT));
        assert!(FILTER_CHAT_CHILDREN.iter().all(|row| view.filter_enabled(*row)), "aba All não trava nada");
        for (tab, locked) in [(1usize, 7usize), (2, 6), (3, 5), (4, 9)] {
            let mut other = ChatView::new(187, view.filter.clone());
            other.active_tab = tab;
            other.toggle_filter();
            other.filter_edit[FILTER_CHAT] = true;
            assert!(!other.filter_enabled(locked), "aba {tab} trava a linha {locked}");
            assert!(other.filter_enabled(4), "as outras seguem livres");
        }
    }

    /// `Ok` grava na aba ativa e fecha; `Cancel` e o martelo fecham sem gravar.
    #[test]
    fn ok_writes_the_filters_of_the_tab_and_cancel_throws_them_away() {
        let mut view = view();
        view.toggle_filter();
        assert_eq!(view.filter_hit(17 + 2, -191 + 17 + 2), Some(FilterHit::Row(FILTER_SYSTEM)));
        assert_eq!(view.filter_hit(139, -191 + 136), Some(FilterHit::Row(11)));
        assert_eq!(view.filter_hit(60, -191 + 170), Some(FilterHit::Ok));
        assert_eq!(view.filter_hit(140, -191 + 170), Some(FilterHit::Cancel));
        assert_eq!(view.filter_hit(1, -191 + 1), None);
        assert_eq!(view.filter_hit(17, 17), None, "o painel fica acima da janela");
        view.filter_click(FilterHit::Row(4));
        assert_eq!(view.filter_click(FilterHit::Cancel), None);
        assert!(!view.filter_open);
        assert!(view.filter.tabs[0].shows("shout"), "cancel não grava");
        view.toggle_filter();
        assert!(view.filter_edit[4], "reabrir recarrega do filtro salvo");
        view.filter_click(FilterHit::Row(4));
        assert_eq!(view.filter_click(FilterHit::Ok), Some("entire"));
        assert!(!view.filter_open);
        assert!(!view.filter.tabs[0].shows("shout"));
        assert!(!view.shows(&entry(1, "a", "grito")));
        assert!(view.shows(&entry(0, "a", "normal")));
        let mut market = view.clone();
        market.active_tab = 1;
        market.toggle_filter();
        market.filter_click(FilterHit::Row(FILTER_SYSTEM));
        assert_eq!(market.filter_click(FilterHit::Ok), Some("market"));
        assert!(market.filter.tabs[1].shows("system"));
        assert!(market.filter.tabs[1].shows("shout"), "só a aba ativa muda");
        assert!(!market.filter.tabs[0].shows("shout"));
        assert_eq!(view.filter_hit(60, -191 + 170), None, "painel fechado não recebe clique");
    }

    /// As doze linhas do painel são as mesmas chaves, na mesma ordem, que o `chatfilter.ini`
    /// do cliente traz em cada seção.
    #[test]
    fn the_panel_rows_are_the_twelve_keys_of_the_ini_in_the_order_of_the_file() {
        let keys: Vec<&str> = FILTER_ROWS.iter().map(|(_, key, _, _)| *key).collect();
        assert_eq!(keys, crate::ini::CHAT_FILTER_KEYS);
        let channels = [CHAT_SYSTEM, 0, 1, 2, 3, 4, 8, 9, 17];
        for kind in channels {
            let name = filter_name(kind).expect("canal com filtro");
            assert!(keys.contains(&name), "canal {kind} usa a chave {name}");
        }
        assert_eq!(FILTER_ROWS[FILTER_SYSTEM].1, "system");
        assert_eq!(FILTER_ROWS[FILTER_CHAT].1, "chat");
        assert_eq!(FILTER_SYSTEM_CHILDREN.map(|row| FILTER_ROWS[row].1), ["damage", "useitems"]);
        assert_eq!(
            FILTER_CHAT_CHILDREN.map(|row| FILTER_ROWS[row].1),
            ["normal", "shout", "pledge", "party", "market", "tell", "ally", "hero"]
        );
        let locked: Vec<Option<&str>> = FILTER_TAB_LOCK.iter().map(|row| row.map(|row| FILTER_ROWS[row].1)).collect();
        assert_eq!(locked, [None, Some("market"), Some("party"), Some("pledge"), Some("ally")]);
    }

    #[test]
    fn every_texture_exists_in_the_client_list() {
        let known: Vec<String> = TEXTURES.lines().map(|l| l.trim().to_lowercase()).collect();
        let font = font();
        let mut view = view();
        view.input.key(InputKey::Char('a'));
        view.toggle_filter();
        view.filter_click(FilterHit::Row(FILTER_SYSTEM));
        view.filter_click(FilterHit::Row(FILTER_CHAT));
        view.pressed = Some(FilterHit::Ok);
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &lines(), &font, 1.0);
        view.paint_filter(&mut list, 0, 0, &font);
        for texture in list.quads.iter().map(|q| q.texture.as_str()).filter(|t| *t != font.atlas && *t != MESSENGER_NORMAL) {
            assert!(known.contains(&texture.to_lowercase()), "textura desconhecida {texture}");
        }
    }

    #[test]
    fn the_lines_start_fourteen_pixels_below_the_list_top_as_chat_list_box_on_create_sets_text_y() {
        let font = font();
        let mut list = DrawList::default();
        view().paint(&mut list, 0, 0, &lines(), &font, 1.0);
        assert_eq!(LINE_TEXT_Y, 14);
        assert!(!text_at(&list.quads, &font, 14.0).is_empty());
        assert!(text_at(&list.quads, &font, 0.0).is_empty());
    }

    #[test]
    fn a_chat_kind_needs_its_own_switch_and_the_chat_master_and_unlisted_kinds_never_show() {
        let mut view = view();
        let normal = entry(0, "a", "b");
        assert!(view.shows(&normal));
        view.filter.tabs[0].filters.insert("chat".to_string(), false);
        assert!(!view.shows(&normal));
        assert!(view.shows(&entry(6, "a", "b")));
        assert!(view.shows(&entry(7, "a", "b")));
        assert!(view.shows(&entry(10, "a", "b")));
        assert!(view.shows(&entry(18, "a", "b")));
        for kind in [11, 12, 13, 14, 15, 16, 19, 40] {
            assert!(!view.shows(&entry(kind, "a", "b")), "{kind}");
        }
    }

    #[test]
    fn the_old_chat_puts_the_tab_prefix_in_the_input_and_enter_puts_it_back() {
        let mut view = view();
        view.old_chat = true;
        view.select_tab(2);
        assert_eq!(view.input.text, "#");
        view.select_tab(3);
        assert_eq!(view.input.text, "@");
        view.input.text = "@hello".to_string();
        view.input.cursor = 6;
        view.select_tab(0);
        assert_eq!(view.input.text, "hello");
        view.select_tab(4);
        assert_eq!(view.input.text, "$hello");
        assert_eq!(view.press(InputKey::Enter), Some("$hello".to_string()));
        assert_eq!(view.input.text, "$");
        view.select_tab(0);
        assert_eq!(view.input.text, "");
        view.old_chat = false;
        view.select_tab(2);
        assert_eq!(view.input.text, "");
    }

    #[test]
    fn a_quote_typed_first_starts_a_tell_to_the_last_partner_and_up_down_walk_the_names() {
        let mut view = view();
        view.tells.add("Ann");
        view.tells.add("Bob");
        view.press(InputKey::Char('"'));
        assert_eq!(view.input.text, "\"Bob ");
        view.press(InputKey::Up);
        assert_eq!(view.input.text, "\"Ann ");
        view.press(InputKey::Down);
        view.press(InputKey::Down);
        assert_eq!(view.input.text, "\"");
        view.press(InputKey::Char('x'));
        assert_eq!(view.input.text, "\"x");
        view.press(InputKey::Enter);
        view.press(InputKey::Up);
        assert_eq!(view.input.text, "");
    }

    #[test]
    fn a_new_row_keeps_a_scrolled_view_on_the_same_lines() {
        let font = font();
        let many: Vec<ChatEntry> = (0..20).map(|i| entry(0, "n", &i.to_string())).collect();
        let mut view = view();
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &many, &font, 1.0);
        view.scroll_by(3);
        view.paint(&mut DrawList::default(), 0, 0, &many, &font, 1.0);
        let mut more = many.clone();
        more.push(entry(0, "n", "20"));
        view.paint(&mut DrawList::default(), 0, 0, &more, &font, 1.0);
        assert_eq!(view.offset(), 4);
        view.scroll_by(0);
        assert_eq!(view.scroll, 4);
        let bottom = ChatView::new(187, view.filter.clone());
        bottom.paint(&mut DrawList::default(), 0, 0, &many, &font, 1.0);
        bottom.paint(&mut DrawList::default(), 0, 0, &more, &font, 1.0);
        assert_eq!(bottom.offset(), 0);
    }

    #[test]
    fn each_tab_keeps_its_own_list_filtered_when_the_line_arrives_and_a_later_filter_change_does_not_touch_it() {
        let mut view = view();
        view.filter.tabs[1].filters.insert("chat".to_string(), true);
        for entry in [entry(3, "a", "party"), entry(8, "b", "trade"), entry(0, "c", "say")] {
            view.ingest(&entry);
        }
        let kinds = |tab: usize| view.lists[tab].iter().map(|e| e.kind).collect::<Vec<_>>();
        assert_eq!(kinds(0), [3, 8, 0]);
        assert_eq!(kinds(1), [3, 8, 0]);
        assert_eq!(kinds(2), [3]);
        view.filter.tabs[0].filters.insert("normal".to_string(), false);
        assert_eq!(kinds(0), [3, 8, 0]);
        view.ingest(&entry(0, "d", "new"));
        assert_eq!(view.lists[0].len(), 3);
        view.active_tab = 2;
        assert_eq!(view.active_list().len(), 1);
        for i in 0..250 {
            view.ingest(&entry(3, "x", &i.to_string()));
        }
        assert_eq!(view.lists[2].len(), LIST_CAPACITY);
        assert_eq!(view.lists[2].last().unwrap().text, "249");
        view.prefiltered = true;
        assert!(view.shows(&entry(11, "a", "b")));
        view.clear_lists();
        assert!(view.lists.iter().all(Vec::is_empty));
    }

    #[test]
    fn a_detached_tab_covers_its_bar_slot_with_tab3_and_floats_its_own_list_with_the_tab_under_it() {
        let font = font();
        let mut view = view();
        view.filter.tabs[2].filters.insert("chat".to_string(), true);
        for i in 0..4 {
            view.ingest(&entry(3, "a", &format!("p{i}")));
        }
        view.detach(2);
        assert_eq!((view.detached[2], view.active_tab), (true, 2));
        let quads = painted(&view, view.active_list(), 1.0);
        let cover = with_texture(&quads, CHATTING_TAB_SPLIT);
        assert_eq!((cover.len(), cover[0].x, cover[0].y), (1, 151.0, 141.0));
        assert!(text_at(&quads, &font, LINE_TEXT_Y as f32).is_empty());
        let mut list = DrawList::default();
        view.paint_split(&mut list, 30, 40, 2, &font, 1.0);
        assert!(!with_texture(&list.quads, CHATTING_BACK2).is_empty());
        assert_eq!(with_texture(&list.quads, CHATTING_BACK3).len(), 7);
        let tab = with_texture(&list.quads, CHATTING_TAB_ON);
        assert_eq!((tab[0].x, tab[0].y), (30.0 + 151.0, 40.0 + 141.0));
        assert!(!text_at(&list.quads, &font, 40.0 + LINE_TEXT_Y as f32).is_empty());
        assert_eq!(view.split_size(), (348, 141 + 23));
        assert_eq!(view.split_grip(2), Rect::new(151, 141, 64, 23));
        assert_eq!(ChatView::split_tab_of(ChatView::split_window_id(2)), Some(2));
        assert_eq!(ChatView::split_tab_of(ChatView::split_window_id(0)), None);
        assert!(ChatView::merge_touches(Rect::new(140, 130, 64, 23), Rect::new(151, 141, 64, 23)));
        assert!(!ChatView::merge_touches(Rect::new(400, 300, 64, 23), Rect::new(151, 141, 64, 23)));
        view.attach(2);
        assert!(!view.detached[2]);
        view.detach(0);
        assert!(!view.detached[0]);
    }
}
