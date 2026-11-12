#[path = "board_layout_generated.rs"]
mod layout;

use crate::button::TextButton;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::html::{self, Document, Line};
use crate::initial_position::InitialPosition;
use crate::tab_control::TabEvents;
use crate::tab_strip::{TabSpec, TabStrip};
use crate::window::Rect;

pub const SIZE: (i32, i32) = layout::SIZE;
pub const INITIAL_POSITION: InitialPosition =
    InitialPosition { enabled: true, horizontal_mode: 2, vertical_mode: 2, x: SIZE.0, y: SIZE.1 };
pub const TITLE_SYSSTRING: u32 = 387;
pub const DEFAULT_TITLE: &str = "Community";
pub const BACK_TOP: &str = "L2UI_CH3.BoardWnd.board_back1";
pub const BACK_TOP_RECT: Rect = Rect::new(0, 20, SIZE.0, 28);
pub const BACK_BODY: &str = "L2UI_CH3.BoardWnd.board_back2";
pub const BACK_BODY_RECT: Rect = Rect::new(0, 48, SIZE.0, 482);
pub const VIEWER_RECT: Rect = layout::RECT_E8;
pub const VIEWER_CLIP: (i32, i32, i32, i32) = (3, 10, 6, 20);
pub const FAVORITE_RECT: Rect = layout::RECT_F4;
pub const FAVORITE_SYSSTRING: u32 = 380;
pub const FAVORITE_ID: u32 = 0xf0f0_1422;
pub const SMALL_BUTTON: &str = "L2UI_CH3.Button.SmallButton2";
pub const SMALL_BUTTON_DOWN: &str = "L2UI_CH3.Button.SmallButton2_down";
pub const TABS_POS: (i32, i32) = (12, 25);
pub const TABS_ID: u32 = 0xf0f0_1423;
pub const TAB_SIZE: (i32, i32) = (74, 23);
pub const COMMUNITY_TAB: usize = 3;
pub const TAB_NORMAL: &str = "L2UI_CH3.BoardWnd.Board_tab2";
pub const TAB_PRESSED: &str = "L2UI_CH3.BoardWnd.Board_tab1";
pub const TAB_SYSSTRINGS: [u32; 7] = [377, 379, 381, 382, 403, 905, 904];
pub const TAB_SPACING: [i32; 7] = [1, 8, 1, 8, 1, 1, 0];
pub const LINK_COUNT: usize = 8;
pub const PLACEHOLDERS: [&str; 20] = [
    "<!--ROOT_BID-->",
    "<!--THREAD_ORDER-->",
    "<!--BID-->",
    "<!--STEP-->",
    "<!--PARENT_BID-->",
    "<!--CHAR_ID-->",
    "<!--CHAR_NAME-->",
    "<!--ACCOUNT_ID-->",
    "<!--ACCOUNT_NAME-->",
    "<!--WORLD_ID-->",
    "<!--WRITER-->",
    "<!--TITLE-->",
    "<!--CONTENT-->",
    "<!--POST_DATE-->",
    "<!--MODIFY_DATE-->",
    "<!--READ_COUNT-->",
    "<!--STATUS-->",
    "<!--IP-->",
    "<!--COMMENT-->",
    "<!--THREAD-->",
];
const SPLIT: char = '\u{8}';
const LIST_PLACEHOLDERS: usize = 18;
const FORM_PLACEHOLDERS: usize = 17;
const WRITER: &str = "<!--WRITER-->";
const TITLE: &str = "<!--TITLE-->";
const CONTENT: &str = "<!--CONTENT-->";
const EMPTY_FIELD: &str = " ";

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    (
        crate::window_restore::x87_to_int(f64::from(viewport.0 as f32) * 0.5 - 323.0),
        crate::window_restore::x87_to_int(f64::from(viewport.1 as f32) * 0.5 - 265.0),
    )
}

pub fn grip() -> Rect {
    FrameCtrl::titled(DEFAULT_TITLE, true).grip(SIZE.0, SIZE.1)
}

pub fn close_button() -> Rect {
    Rect::new(SIZE.0 - crate::frame::CLOSE_INSET, crate::frame::BUTTON_Y, crate::frame::BUTTON_SIZE, crate::frame::BUTTON_SIZE)
}

pub fn content_rect() -> Rect {
    let (dx, dy, dw, dh) = VIEWER_CLIP;
    Rect::new(VIEWER_RECT.x + dx, VIEWER_RECT.y + dy, VIEWER_RECT.w - dw, VIEWER_RECT.h - dh)
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FormFields {
    pub writer: String,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardHtml {
    pub text: String,
    pub form: Option<FormFields>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BoardPages {
    pub template: [String; 4],
    pub template_count: i32,
    pub form: [String; 2],
    pub form_count: i32,
    pub list: [String; 3],
    pub list_count: i32,
}

fn split_token(text: &str) -> (Option<&str>, &str) {
    let trimmed = text.trim_start_matches(SPLIT);
    if trimmed.is_empty() {
        return (None, trimmed);
    }
    match trimmed.find(SPLIT) {
        Some(at) => (Some(&trimmed[..at]), &trimmed[at + SPLIT.len_utf8()..]),
        None => (Some(trimmed), &trimmed[trimmed.len()..]),
    }
}

fn replace_first(text: &mut String, placeholder: &str, value: &str) {
    if let Some(at) = text.find(placeholder).filter(|at| *at > 0) {
        text.replace_range(at..at + placeholder.len(), value);
    }
}

impl BoardPages {
    pub fn receive(&mut self, html: &str) -> Option<BoardHtml> {
        if html.is_empty() {
            return None;
        }
        if html.starts_with('<') {
            return Some(BoardHtml { text: html.to_string(), form: None });
        }
        let (token, rest) = split_token(html);
        let token = token.unwrap_or(html);
        let body = rest.to_string();
        match token {
            "1" | "2" | "3" | "4" => {
                let index = usize::from(token.as_bytes()[0] - b'1');
                self.template[index] = body;
                self.template_count += 1;
            }
            "1001" | "1002" => {
                self.form[usize::from(token == "1002")] = body;
                self.form_count += 1;
            }
            "101" | "102" | "103" => {
                let index = usize::from(token.as_bytes()[2] - b'1');
                self.list[index] = body;
                self.list_count += 1;
            }
            _ => {}
        }
        let mut shown = None;
        if self.list_count == 3 {
            shown = Some(self.assemble_list());
        }
        if self.template_count == 4 {
            shown = Some(self.assemble_template());
        }
        if self.form_count == 2 {
            shown = Some(self.assemble_form());
        }
        shown
    }

    fn assemble_list(&mut self) -> BoardHtml {
        let mut text = self.list[0].clone();
        for part in &self.list[1..] {
            if !part.is_empty() {
                text.push_str(part);
            }
        }
        self.list = Default::default();
        self.list_count = 0;
        BoardHtml { text, form: None }
    }

    fn assemble_template(&mut self) -> BoardHtml {
        let mut text = self.template[0].clone();
        let (mut value, mut rest) = split_token(self.template[1].as_str());
        if value.is_some() {
            for placeholder in &PLACEHOLDERS[..LIST_PLACEHOLDERS] {
                if let Some(field) = value {
                    replace_first(&mut text, placeholder, field);
                }
                let (next_value, next_rest) = split_token(rest);
                value = next_value.or(value);
                rest = next_rest;
            }
        }
        replace_first(&mut text, PLACEHOLDERS[18], &self.template[2]);
        replace_first(&mut text, PLACEHOLDERS[19], &self.template[3]);
        self.template = Default::default();
        self.template_count = 0;
        BoardHtml { text, form: None }
    }

    fn assemble_form(&mut self) -> BoardHtml {
        let mut text = self.form[0].clone();
        let mut fields = FormFields::default();
        let (mut value, mut rest) = split_token(self.form[1].as_str());
        for placeholder in &PLACEHOLDERS[..FORM_PLACEHOLDERS] {
            let Some(current) = value else { break };
            let slot = match *placeholder {
                TITLE => Some(&mut fields.title),
                CONTENT => Some(&mut fields.content),
                WRITER => Some(&mut fields.writer),
                _ => None,
            };
            match slot {
                Some(slot) => {
                    *slot = if current != EMPTY_FIELD { current.to_string() } else { String::new() };
                }
                None => replace_first(&mut text, placeholder, current),
            }
            let (next_value, next_rest) = split_token(rest);
            value = next_value.or(value);
            rest = next_rest;
        }
        self.form = Default::default();
        self.form_count = 0;
        BoardHtml { text, form: Some(fields) }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoardHit {
    Tab(usize),
    Favorite,
    Edit(String),
    Link(String),
}

#[derive(Debug, Clone)]
pub struct BoardView {
    pub frame: FrameCtrl,
    pub tabs: TabStrip,
    pub favorite: TextButton,
    pub script: crate::scripted::board_wnd::BoardWnd,
    pub pages: BoardPages,
    pub doc: Document,
    pub lines: Vec<Line>,
    pub height: i32,
    pub scroll: i32,
    pub form: Option<FormFields>,
    pub html_form: crate::html_form::HtmlForm,
    pub locked: bool,
    unlock_ms: u32,
}

pub const UNLOCK_MS: u32 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardWindowEvent {
    Hide,
    Show,
    Focus,
}

struct BoardPacketHost<'a> {
    view: &'a mut BoardView,
    packet: Option<(&'a [String], &'a str)>,
    font: &'a BitmapFont,
    windows: &'a mut dyn FnMut(BoardWindowEvent),
    resolve: &'a dyn Fn(html::Marker, i32) -> Option<String>,
}

impl crate::scripted::board_wnd::BoardWndPacketHost for BoardPacketHost<'_> {
    fn parse_int(&mut self, _: &str, key: &str) -> i32 {
        match key { "OK" => i32::from(self.packet.is_some()), _ => unreachable!("compiled board integer parameter") }
    }
    fn parse_string(&mut self, _: &str, key: &str) -> String {
        let Some((links, html)) = self.packet else { return String::new() };
        if key == "Address" { return html.to_owned(); }
        key.strip_prefix("Command").and_then(|index| index.parse::<usize>().ok())
            .and_then(|index| index.checked_sub(1)).and_then(|index| links.get(index)).cloned().unwrap_or_default()
    }
    fn uiapi_htmlctrl_set_html_buff_data(&mut self, _: &str, html: &str) { self.view.load_html(html, self.font, self.resolve); }
    fn uiapi_window_hide_window(&mut self, _: &str) { (self.windows)(BoardWindowEvent::Hide); }
    fn uiapi_window_show_window(&mut self, _: &str) { (self.windows)(BoardWindowEvent::Show); }
    fn uiapi_window_set_focus(&mut self, _: &str) { (self.windows)(BoardWindowEvent::Focus); }
}

impl BoardView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> BoardView {
        let text = |id: u32| lookup(id).unwrap_or_default();
        let mut tabs = TabStrip::new(TABS_ID, 0);
        for (index, label) in TAB_SYSSTRINGS.into_iter().enumerate() {
            tabs.add_fixed(TabSpec {
                label: Some(text(label)),
                page: Some(0),
                size: TAB_SIZE,
                textures: [Some(TAB_NORMAL.into()), Some(TAB_PRESSED.into())],
                spacing: TAB_SPACING[index],
                tooltip: -1,
                special_attachment: false,
            });
        }
        for button in &mut tabs.buttons {
            button.visible = true;
        }
        let title = lookup(TITLE_SYSSTRING).unwrap_or_else(|| DEFAULT_TITLE.to_string());
        BoardView {
            frame: FrameCtrl::titled(&title, true),
            tabs,
            favorite: TextButton::new(FAVORITE_RECT, SMALL_BUTTON, SMALL_BUTTON_DOWN, &text(FAVORITE_SYSSTRING)),
            script: Default::default(),
            pages: BoardPages::default(),
            doc: Document::default(),
            lines: Vec::new(),
            height: 0,
            scroll: 0,
            form: None,
            html_form: crate::html_form::HtmlForm::default(),
            locked: false,
            unlock_ms: 0,
        }
    }

    pub fn tick(&mut self, elapsed: u32) {
        self.unlock_ms = self.unlock_ms.saturating_add(elapsed);
        while self.unlock_ms >= UNLOCK_MS {
            self.unlock_ms -= UNLOCK_MS;
            self.locked = false;
        }
    }

    pub fn begin_link(&mut self) -> bool {
        if self.locked {
            return false;
        }
        self.pages.template_count = 0;
        self.pages.form_count = 0;
        self.pages.list_count = 0;
        true
    }

    pub fn end_link(&mut self, result: i32) {
        if result == 1 {
            self.locked = true;
        }
    }

    pub fn relabel(&mut self, lookup: impl Fn(u32) -> Option<String>) {
        for (button, id) in self.tabs.buttons.iter_mut().zip(TAB_SYSSTRINGS) {
            button.paint.label = lookup(id).unwrap_or_default();
        }
        self.favorite.label = lookup(FAVORITE_SYSSTRING).unwrap_or_default();
        self.frame = FrameCtrl::titled(&lookup(TITLE_SYSSTRING).unwrap_or_else(|| DEFAULT_TITLE.to_string()), true);
    }

    pub fn show(&mut self, links: &[String], html_text: &str, font: &BitmapFont) {
        self.receive(Some((links, html_text)), font, &mut |_| {});
    }

    pub fn receive(&mut self, packet: Option<(&[String], &str)>, font: &BitmapFont, windows: &mut impl FnMut(BoardWindowEvent)) {
        self.receive_with(packet, font, windows, &|_, _| None);
    }

    pub fn receive_with(&mut self, packet: Option<(&[String], &str)>, font: &BitmapFont, windows: &mut impl FnMut(BoardWindowEvent), resolve: &dyn Fn(html::Marker, i32) -> Option<String>) {
        let mut script = std::mem::take(&mut self.script);
        let mut host = BoardPacketHost { view: self, packet, font, windows, resolve };
        script.handle_show_board_packet(&mut host, String::new());
        self.script = script;
    }

    fn load_html(&mut self, html_text: &str, font: &BitmapFont, resolve: &dyn Fn(html::Marker, i32) -> Option<String>) {
        if let Some(shown) = self.pages.receive(html_text) {
            self.locked = false;
            self.doc = html::parse_with(&shown.text, resolve);
            self.lines = html::layout(&self.doc, content_rect().w, font);
            self.height = html::height(&self.lines, font);
            self.scroll = 0;
            self.html_form = crate::html_form::HtmlForm::from_document(&self.doc);
            if let Some(fields) = &shown.form {
                self.html_form.fill(&fields.writer, &fields.title, &fields.content);
            }
            self.form = shown.form;
        }
    }

    pub fn max_scroll(&self) -> i32 {
        (self.height - content_rect().h).max(0)
    }

    pub fn scroll_by(&mut self, dy: i32) {
        self.scroll = (self.scroll + dy).clamp(0, self.max_scroll());
    }

    pub fn press_tab(&mut self, index: usize, host: &mut impl crate::scripted::board_wnd::BoardWndHost) {
        let mut events = TabEvents::default();
        self.tabs.control.press(index, true, &mut events);
        if let Ok(index) = i32::try_from(index) {
            self.script.show_bbs_tab(host, index);
        }
    }

    pub fn run_tab(&mut self, index: usize, host: &mut impl crate::scripted::board_wnd::BoardWndHost) {
        if let Ok(index) = i32::try_from(index) {
            self.script.show_bbs_tab(host, index);
        }
    }

    pub fn favorite(&mut self, host: &mut impl crate::scripted::board_wnd::BoardWndHost) {
        self.script.on_click_bookmark(host);
    }

    pub fn hit(&self, x: i32, y: i32, font: &BitmapFont) -> Option<BoardHit> {
        if self.favorite.contains(x, y) {
            return Some(BoardHit::Favorite);
        }
        let (tx, ty) = TABS_POS;
        if let Some(index) = self.tabs.buttons.iter().position(|b| b.visible && b.paint.rect.offset(tx, ty).contains(x, y)) {
            return Some(BoardHit::Tab(index));
        }
        let content = content_rect();
        if !content.contains(x, y) {
            return None;
        }
        if let Some(var) = html::edit_at(&self.lines, x - content.x, y - content.y + self.scroll) {
            return Some(BoardHit::Edit(var.to_string()));
        }
        html::link_at(&self.lines, x - content.x, y - content.y + self.scroll, font).map(|action| BoardHit::Link(action.to_string()))
    }

    pub fn link_message_at(&self, x: i32, y: i32) -> Option<&str> {
        let content = content_rect();
        if !content.contains(x, y) {
            return None;
        }
        html::link_message_at(&self.lines, x - content.x, y - content.y + self.scroll)
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, favorite_pressed: bool, font: &BitmapFont, alpha: f32) {
        let (w, h) = SIZE;
        self.frame.paint(list, x, y, w, h, font, alpha);
        for (texture, rect) in [(BACK_TOP, BACK_TOP_RECT), (BACK_BODY, BACK_BODY_RECT)] {
            list.draw_texture(x + rect.x, y + rect.y, rect.w, rect.h, 0, 0, rect.w, rect.h, texture, 1.0);
        }
        self.tabs.paint(list, (x + TABS_POS.0, y + TABS_POS.1), font);
        self.favorite.paint(list, x, y, favorite_pressed, font);
        let content = content_rect();
        crate::npc_viewer::paint_html_lines(list, x + content.x, y + content.y, (content.w, content.h), &self.lines, self.scroll, font, alpha);
        crate::html_form::paint_form(list, (x + content.x, y + content.y), (content.w, content.h), &self.lines, self.scroll, &self.html_form, font);
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

    fn view() -> BoardView {
        BoardView::new(|id| Some(format!("s{id}")))
    }

    #[test]
    fn layout_follows_the_executed_on_create_1003f9e0() {
        let board = view();
        let rects: Vec<_> = board.tabs.buttons.iter().map(|b| b.paint.rect.offset(TABS_POS.0, TABS_POS.1)).collect();
        let xs: Vec<_> = rects.iter().map(|r| r.x - TABS_POS.0).collect();
        assert_eq!(xs, [0, 75, 157, 232, 314, 389, 464]);
        assert_eq!(rects.last().map(|r| r.x + r.w - TABS_POS.0), Some(538));
        assert!(rects.iter().all(|r| (r.w, r.h) == TAB_SIZE));
        assert_eq!(board.tabs.buttons.iter().map(|b| b.paint.label.clone()).collect::<Vec<_>>(), TAB_SYSSTRINGS.map(|id| format!("s{id}")));
        assert_eq!(board.favorite.label, "s380");
        assert_eq!(content_rect(), Rect::new(11, 61, 627, 454));
        assert_eq!(default_position((1024, 768)), (189, 119));
    }

    #[test]
    fn paint_follows_on_paint_1003ca70() {
        let board = view();
        let mut list = DrawList::default();
        board.paint(&mut list, 100, 50, false, &font(), 1.0);
        let backs: Vec<_> = list.quads.iter().filter(|q| q.texture.contains("board_back"))
            .map(|q| (q.x, q.y, q.w, q.h, q.uw, q.vh, q.texture.clone())).collect();
        assert_eq!(backs, [
            (100.0, 70.0, 646.0, 28.0, 646.0, 28.0, crate::draw::Texture::from(BACK_TOP)),
            (100.0, 98.0, 646.0, 482.0, 646.0, 482.0, crate::draw::Texture::from(BACK_BODY)),
        ]);
    }

    #[test]
    fn tabs_send_the_fixed_bypass_and_favorite_waits_for_the_next_board() {
        let mut board = view();
        let mut commands = Vec::new();
        board.press_tab(2, &mut |command: &str| { commands.push(command.to_owned()); 1 });
        assert_eq!(commands, ["bypass _bbsloc"]);
        assert_eq!(board.tabs.control.entries.iter().position(|e| e.pressed), Some(2));
        board.favorite(&mut |command: &str| { commands.push(command.to_owned()); 1 });
        assert_eq!(commands.len(), 1);
        let links: Vec<String> = (0..8).map(|i| if i == 7 { "bypass _bbsaddfav".into() } else { String::new() }).collect();
        board.show(&links, "<html>a</html>", &font());
        board.favorite(&mut |command: &str| { commands.push(command.to_owned()); 1 });
        board.favorite(&mut |command: &str| { commands.push(command.to_owned()); 1 });
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[1], "bypass _bbsaddfav");
        board.show(&links, "", &font());
        board.favorite(&mut |command: &str| { commands.push(command.to_owned()); 2 });
        assert!(!board.script.m_b_btn_lock);
    }

    #[test]
    fn set_html_10042450_joins_list_parts_only_after_the_third() {
        let mut pages = BoardPages::default();
        assert_eq!(pages.receive("101\u{8}<html>um "), None);
        assert_eq!(pages.receive("102\u{8}dois "), None);
        assert_eq!(pages.receive("103\u{8}"), Some(BoardHtml { text: "<html>um dois ".into(), form: None }));
        assert_eq!(pages, BoardPages::default());
        assert_eq!(pages.receive("<html>x</html>").map(|h| h.text), Some("<html>x</html>".into()));
        assert_eq!(pages.receive("999\u{8}lixo"), None);
        assert_eq!(pages.receive(""), None);
    }

    #[test]
    fn set_html_template_mode_replaces_the_first_placeholder_after_position_zero() {
        let mut pages = BoardPages::default();
        pages.receive("1\u{8}x<!--ROOT_BID--> <!--BID--> <!--ROOT_BID--> <!--COMMENT--> <!--THREAD-->");
        pages.receive("2\u{8}10\u{8}\u{8}20\u{8}30");
        pages.receive("3\u{8}c");
        let shown = pages.receive("4\u{8}t").unwrap();
        assert_eq!(shown.text, "x10 30 <!--ROOT_BID--> c t");
        let mut start = BoardPages::default();
        start.receive("1\u{8}<!--ROOT_BID-->");
        start.receive("2\u{8}9");
        start.receive("3\u{8}");
        assert_eq!(start.receive("4\u{8}").unwrap().text, "<!--ROOT_BID-->");
    }

    #[test]
    fn set_html_form_mode_keeps_writer_title_and_content_for_the_edit_boxes() {
        let mut pages = BoardPages::default();
        pages.receive("1001\u{8}<html><!--BID--> <!--TITLE--></html>");
        let values = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "Autor", " ", "Texto"].join("\u{8}");
        let shown = pages.receive(&format!("1002\u{8}{values}")).unwrap();
        assert_eq!(shown.text, "<html>3 <!--TITLE--></html>");
        assert_eq!(shown.form, Some(FormFields { writer: "Autor".into(), title: String::new(), content: "Texto".into() }));
        assert_eq!(pages.form_count, 0);
    }

    #[test]
    fn missing_fields_reuse_the_last_one_as_in_10041b30() {
        let mut pages = BoardPages::default();
        pages.receive("1\u{8}x <!--STEP--> <!--IP-->");
        pages.receive("2\u{8}a\u{8}b");
        pages.receive("3\u{8}");
        assert_eq!(pages.receive("4\u{8}").unwrap().text, "x b b");
        let mut form = BoardPages::default();
        form.receive("1001\u{8}x <!--STATUS-->");
        let shown = form.receive("1002\u{8}a\u{8}b").unwrap();
        assert_eq!(shown.text, "x b");
        assert_eq!(shown.form.unwrap().content, "b");
    }
}
