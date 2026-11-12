const LOCAL_LINK_PREFIX: &str = "\u{1}local-html:";
const CMD_PREFIX: &str = "\u{1}cmd:";

pub fn local_link_target(action: &str) -> Option<&str> {
    action.strip_prefix(LOCAL_LINK_PREFIX)
}

use crate::color::{Argb, TEXT};
use crate::font::BitmapFont;
use crate::html_layout::{LeafKind, Primitive, TextDraw};

pub const LINK_COLOR: Argb = Argb(0xFF6699FF);
pub const LEVEL_COLOR: Argb = Argb(0xFFFFCC00);
pub const LEVEL_NAME: &str = "LEVEL";
pub const LINE_SPACING: i32 = 6;
pub const BR_HEIGHT: i32 = 9;
pub const BUTTON_DEFAULT_SIZE: (i32, i32) = (10, 10);
pub const BUTTON_DEFAULT_BACK: &str = "NWindow.BlackTexture";
pub const BUTTON_DEFAULT_FORE: &str = "NWindow.WhiteTexture";
pub const HTML_CELL_PADDING: i32 = 0;
const COMMENT: &str = "!--";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Text(String),
    Br,
    Br1,
    Font { color: Option<Argb>, children: Vec<Node> },
    Link { action: String, children: Vec<Node> },
    Center(Vec<Node>),
    Table { width: Option<i32>, rows: Vec<Vec<Cell>> },
    Img { src: String, width: i32, height: i32 },
    Button { value: String, action: String, width: i32, height: i32, back: String, fore: String },
    Edit { var: String, width: i32, height: i32, multi: bool },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Cell {
    pub width: Option<i32>,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Document {
    pub title: Option<String>,
    pub body: Vec<Node>,
    pub(crate) tokens: Vec<crate::html_tokens::Token>,
}

impl Document {
    pub fn edits(&self) -> Vec<(&str, bool)> {
        let mut edits = Vec::new();
        collect_edits(&self.body, &mut edits);
        edits
    }

    pub fn actions(&self) -> Vec<&str> {
        let mut actions = Vec::new();
        collect_actions(&self.body, &mut actions);
        actions
    }
}

fn collect_edits<'a>(nodes: &'a [Node], edits: &mut Vec<(&'a str, bool)>) {
    for node in nodes {
        match node {
            Node::Edit { var, multi, .. } => edits.push((var, *multi)),
            Node::Link { children, .. } | Node::Font { children, .. } | Node::Center(children) => collect_edits(children, edits),
            Node::Table { rows, .. } => {
                for cell in rows.iter().flatten() {
                    collect_edits(&cell.children, edits);
                }
            }
            _ => {}
        }
    }
}

fn collect_actions<'a>(nodes: &'a [Node], actions: &mut Vec<&'a str>) {
    for node in nodes {
        match node {
            Node::Link { action, children } => {
                actions.push(action);
                collect_actions(children, actions);
            }
            Node::Button { action, .. } if !action.is_empty() => actions.push(action),
            Node::Font { children, .. } | Node::Center(children) => collect_actions(children, actions),
            Node::Table { rows, .. } => {
                for cell in rows.iter().flatten() {
                    collect_actions(&cell.children, actions);
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub x: i32,
    pub text: String,
    pub color: Argb,
    pub action: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinkPiece {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub fn link_pieces(item: &crate::html_layout::LeafBox, char_height: i32) -> Vec<LinkPiece> {
    let row = char_height.wrapping_add(6);
    let count = if item.h > row.wrapping_mul(2) { 3 } else { i32::from(item.h > row) + 1 };
    let left = item.parent_x;
    (0..count)
        .filter_map(|index| match (index, count) {
            (0, 1) => Some(LinkPiece { x: item.x, y: 0, width: item.w, height: char_height }),
            (0, _) => Some(LinkPiece { x: item.x, y: 0, width: item.wrap, height: char_height }),
            (1, 2) => Some(LinkPiece { x: left, y: row, width: item.end, height: char_height }),
            (1, 3) => Some(LinkPiece { x: left, y: row, width: item.wrap, height: item.h.wrapping_sub(row.wrapping_mul(2)) }),
            (2, _) => Some(LinkPiece { x: left, y: item.h.wrapping_sub(char_height), width: item.end, height: char_height }),
            _ => None,
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WidgetKind {
    Image(String),
    Button { value: String, back: String, fore: String },
    Edit { var: String, multi: bool, action: String, length: i32, kind: i32 },
    ComboBox { var: String, list: String, selected: String },
    Link,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Widget {
    pub x: i32,
    pub width: i32,
    pub height: i32,
    pub kind: WidgetKind,
    pub action: Option<String>,
    pub message: Option<String>,
    pub pieces: Vec<LinkPiece>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Line {
    pub y: i32,
    pub spans: Vec<Span>,
    pub widgets: Vec<Widget>,
    pub rules: Vec<Rule>,
    pub fills: Vec<Fill>,
    pub extent: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
    pub color: Argb,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fill {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub color: Argb,
}

pub fn font_color(name: &str) -> Argb {
    if name == LEVEL_NAME {
        return LEVEL_COLOR;
    }
    let digits: String = name.chars().take_while(char::is_ascii_hexdigit).collect();
    Argb(u32::from_str_radix(&format!("ff{digits}"), 16).unwrap_or(u32::MAX))
}

pub fn line_step(font: &BitmapFont) -> i32 {
    font.line_height as i32 + LINE_SPACING
}

pub const MESSAGE_SEPARATOR: char = '\u{1}';

pub fn split_message(action: &str) -> (Option<&str>, &str) {
    match action.split_once(MESSAGE_SEPARATOR) {
        Some((message, action)) => (Some(message), action),
        None => (None, action),
    }
}

pub fn message_code(message: &str) -> Option<(i32, String)> {
    let digits: String = message.trim_start().chars().take_while(char::is_ascii_digit).collect();
    let id: i32 = digits.parse().ok().filter(|id| *id > 0)?;
    let chars: Vec<char> = message.chars().collect();
    let argument = chars
        .len()
        .checked_sub(2)
        .and_then(|last| (0..=last).rev().find(|index| chars[*index] == ';'))
        .map(|semi| chars[semi + 1..].iter().collect())
        .unwrap_or_default();
    Some((id, argument))
}

pub fn cmd_target(action: &str) -> Option<&str> {
    action.strip_prefix(CMD_PREFIX).filter(|command| !command.is_empty())
}

pub fn bypass_command(action: &str) -> Option<String> {
    let mut tokens = split_message(action).1.split_whitespace();
    if !tokens.next()?.eq_ignore_ascii_case("bypass") {
        return None;
    }
    let tokens: Vec<&str> = tokens.collect();
    if tokens.is_empty() {
        return None;
    }
    let mut closed = false;
    let rest: Vec<&str> = tokens
        .into_iter()
        .filter(|token| {
            let flag = !closed && *token == "-h";
            closed |= flag;
            !flag
        })
        .collect();
    Some(rest.join(" "))
}

pub fn link_target(action: &str) -> Option<String> {
    let mut tokens = split_message(action).1.split_whitespace();
    if !tokens.next()?.eq_ignore_ascii_case("link") {
        return None;
    }
    Some(tokens.collect::<Vec<&str>>().join(" "))
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Text(String),
    Open { name: String, attrs: Vec<(String, String)> },
    Close(String),
}

fn tokenize(html: &str) -> Vec<Token> {
    let chars: Vec<char> = html.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    let mut text = String::new();
    while i < chars.len() {
        if chars[i] != '<' {
            text.push(chars[i]);
            i += 1;
            continue;
        }
        if chars[i..].starts_with(&['<', '!', '-', '-']) {
            if !text.is_empty() {
                tokens.push(Token::Text(std::mem::take(&mut text)));
            }
            tokens.push(Token::Open { name: COMMENT.into(), attrs: Vec::new() });
            i = find(&chars, i + 4, &['-', '-', '>']).map_or(chars.len(), |at| at + 3);
            continue;
        }
        let Some(end) = tag_end(&chars, i + 1) else { break };
        if !text.is_empty() {
            tokens.push(Token::Text(std::mem::take(&mut text)));
        }
        tokens.push(parse_tag(&chars[i + 1..end]));
        i = end + 1;
    }
    if !text.is_empty() {
        tokens.push(Token::Text(text));
    }
    tokens
}

fn find(chars: &[char], from: usize, pattern: &[char]) -> Option<usize> {
    (from..chars.len()).find(|&at| chars[at..].starts_with(pattern))
}

fn tag_end(chars: &[char], from: usize) -> Option<usize> {
    let mut quote = None;
    for (at, &c) in chars.iter().enumerate().skip(from) {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"' | '\'') => quote = Some(c),
            (None, '>') => return Some(at),
            _ => {}
        }
    }
    None
}

fn parse_tag(inner: &[char]) -> Token {
    let mut i = 0;
    let closing = inner.first() == Some(&'/');
    if closing {
        i = 1;
    }
    let name = read_while(inner, &mut i, |c| !c.is_whitespace() && c != '/' && c != '>').to_uppercase();
    if closing {
        return Token::Close(name);
    }
    let mut attrs = Vec::new();
    while i < inner.len() {
        skip_whitespace(inner, &mut i);
        let key = read_while(inner, &mut i, |c| !c.is_whitespace() && c != '=' && c != '/').to_uppercase();
        if key.is_empty() {
            i += 1;
            continue;
        }
        skip_whitespace(inner, &mut i);
        let mut value = String::new();
        if inner.get(i) == Some(&'=') {
            i += 1;
            skip_whitespace(inner, &mut i);
            match inner.get(i) {
                Some(&q @ ('"' | '\'')) => {
                    i += 1;
                    value = read_while(inner, &mut i, |c| c != q);
                    i += 1;
                }
                _ => value = read_while(inner, &mut i, |c| !c.is_whitespace()),
            }
        }
        attrs.push((key, value));
    }
    Token::Open { name, attrs }
}

fn read_while(chars: &[char], i: &mut usize, keep: impl Fn(char) -> bool) -> String {
    let start = *i;
    while *i < chars.len() && keep(chars[*i]) {
        *i += 1;
    }
    chars[start..*i].iter().collect()
}

fn skip_whitespace(chars: &[char], i: &mut usize) {
    while *i < chars.len() && chars[*i].is_whitespace() {
        *i += 1;
    }
}

fn attr<'a>(attrs: &'a [(String, String)], name: &str) -> Option<&'a str> {
    attrs.iter().find(|(key, _)| key == name).map(|(_, value)| value.as_str())
}

fn attr_int(attrs: &[(String, String)], name: &str) -> Option<i32> {
    attr(attrs, name).and_then(|value| value.trim_end_matches('%').parse().ok())
}

#[derive(Debug, PartialEq, Eq)]
enum Kind {
    Root,
    Font(Option<Argb>),
    Link(String),
    Center,
    Table { width: Option<i32>, rows: Vec<Vec<Cell>> },
    Row(Vec<Cell>),
    Cell(Option<i32>),
}

struct Frame {
    kind: Kind,
    children: Vec<Node>,
}

struct Builder<'a> {
    stack: Vec<Frame>,
    run_start: bool,
    pending_space: bool,
    title: Option<String>,
    in_title: bool,
    resolve: &'a dyn Fn(Marker, i32) -> Option<String>,
}

impl<'a> Builder<'a> {
    fn new(resolve: &'a dyn Fn(Marker, i32) -> Option<String>) -> Builder<'a> {
        Builder {
            stack: vec![Frame { kind: Kind::Root, children: Vec::new() }],
            run_start: true,
            pending_space: false,
            title: None,
            in_title: false,
            resolve,
        }
    }

    fn text(&mut self, raw: &str) {
        let mut text = String::new();
        for c in raw.chars() {
            if c.is_whitespace() {
                self.pending_space = !self.run_start;
                continue;
            }
            if self.pending_space {
                text.push(' ');
            }
            text.push(c);
            self.pending_space = false;
            self.run_start = false;
        }
        let text = expand_markers(&text, self.resolve);
        if text.is_empty() {
            return;
        }
        if self.in_title {
            self.title.get_or_insert_with(String::new).push_str(&text);
        }
        self.push_node(Node::Text(text));
    }

    fn break_run(&mut self) {
        self.run_start = true;
        self.pending_space = false;
    }

    fn push_node(&mut self, node: Node) {
        if let Some(frame) = self.stack.last_mut() {
            frame.children.push(node);
        }
    }

    fn open(&mut self, name: &str, attrs: &[(String, String)]) {
        match name {
            "FONT" => {
                let color = attr(attrs, "COLOR").map(font_color);
                self.stack.push(Frame { kind: Kind::Font(color), children: Vec::new() });
                return;
            }
            "A" => {
                if let Some(path) = attr(attrs, "LINK") {
                    self.stack.push(Frame { kind: Kind::Link(format!("{LOCAL_LINK_PREFIX}{path}")), children: Vec::new() });
                } else if let Some(action) = attr(attrs, "ACTION") {
                    self.stack.push(Frame { kind: Kind::Link(action.to_string()), children: Vec::new() });
                } else if let Some(command) = attr(attrs, "CMD") {
                    self.stack.push(Frame { kind: Kind::Link(format!("{CMD_PREFIX}{command}")), children: Vec::new() });
                }
            }
            "CENTER" => self.stack.push(Frame { kind: Kind::Center, children: Vec::new() }),
            "BR" => self.push_node(Node::Br),
            "BR1" => self.push_node(Node::Br1),
            "IMG" => self.push_node(Node::Img {
                src: attr(attrs, "SRC").unwrap_or_default().to_string(),
                width: attr_int(attrs, "WIDTH").unwrap_or(0),
                height: attr_int(attrs, "HEIGHT").unwrap_or(0),
            }),
            "BUTTON" => self.push_node(Node::Button {
                value: attr(attrs, "VALUE").unwrap_or_default().to_string(),
                action: attr(attrs, "ACTION").unwrap_or_default().to_string(),
                width: attr_int(attrs, "WIDTH").unwrap_or(BUTTON_DEFAULT_SIZE.0),
                height: attr_int(attrs, "HEIGHT").unwrap_or(BUTTON_DEFAULT_SIZE.1),
                back: attr(attrs, "BACK").unwrap_or(BUTTON_DEFAULT_BACK).to_string(),
                fore: attr(attrs, "FORE").unwrap_or(BUTTON_DEFAULT_FORE).to_string(),
            }),
            "EDIT" | "MULTIEDIT" => self.push_node(Node::Edit {
                var: attr(attrs, "VAR").unwrap_or_default().to_string(),
                width: attr_int(attrs, "WIDTH").unwrap_or(0),
                height: attr_int(attrs, "HEIGHT").unwrap_or(0),
                multi: name == "MULTIEDIT",
            }),
            "TABLE" => self.stack.push(Frame {
                kind: Kind::Table { width: attr_int(attrs, "WIDTH"), rows: Vec::new() },
                children: Vec::new(),
            }),
            "TR" if self.close_to(|kind| matches!(kind, Kind::Table { .. }), false) => {
                self.stack.push(Frame { kind: Kind::Row(Vec::new()), children: Vec::new() });
            }
            "TD" if self.close_to(|kind| matches!(kind, Kind::Row(_) | Kind::Table { .. }), false) => {
                if matches!(self.stack.last().map(|f| &f.kind), Some(Kind::Table { .. })) {
                    self.stack.push(Frame { kind: Kind::Row(Vec::new()), children: Vec::new() });
                }
                let width = attr_int(attrs, "WIDTH").or_else(|| attr_int(attrs, "FIXWIDTH"));
                self.stack.push(Frame { kind: Kind::Cell(width), children: Vec::new() });
            }
            "TITLE" => self.in_title = true,
            _ => {}
        }
        self.break_run();
    }

    fn close(&mut self, name: &str) {
        match name {
            "FONT" => {
                self.close_to(|kind| matches!(kind, Kind::Font(_)), true);
                return;
            }
            "A" => {
                self.close_to(|kind| matches!(kind, Kind::Link(_)), true);
            }
            "CENTER" => {
                self.close_to(|kind| matches!(kind, Kind::Center), true);
            }
            "TD" => {
                self.close_to(|kind| matches!(kind, Kind::Cell(_)), true);
            }
            "TR" => {
                self.close_to(|kind| matches!(kind, Kind::Row(_)), true);
            }
            "TABLE" => {
                self.close_to(|kind| matches!(kind, Kind::Table { .. }), true);
            }
            "TITLE" => self.in_title = false,
            _ => {}
        }
        self.break_run();
    }

    fn close_to(&mut self, wanted: impl Fn(&Kind) -> bool, inclusive: bool) -> bool {
        let Some(index) = self.stack.iter().rposition(|frame| wanted(&frame.kind)) else { return false };
        let keep = if inclusive { index } else { index + 1 };
        while self.stack.len() > keep {
            self.pop();
        }
        true
    }

    fn pop(&mut self) {
        let Some(frame) = self.stack.pop() else { return };
        let Some(parent) = self.stack.last_mut() else { return };
        match frame.kind {
            Kind::Root => {}
            Kind::Font(color) => parent.children.push(Node::Font { color, children: frame.children }),
            Kind::Link(action) => parent.children.push(Node::Link { action, children: frame.children }),
            Kind::Center => parent.children.push(Node::Center(frame.children)),
            Kind::Table { width, rows } => parent.children.push(Node::Table { width, rows }),
            Kind::Row(cells) => {
                if let Kind::Table { rows, .. } = &mut parent.kind {
                    rows.push(cells);
                }
            }
            Kind::Cell(width) => {
                if let Kind::Row(cells) = &mut parent.kind {
                    cells.push(Cell { width, children: frame.children });
                }
            }
        }
    }

    fn finish(mut self) -> Document {
        while self.stack.len() > 1 {
            self.pop();
        }
        let body = self.stack.pop().map(|frame| frame.children).unwrap_or_default();
        Document { title: self.title, body, tokens: Vec::new() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    SysString,
    CastleName,
    CastleLocation,
    CastleDescription,
    NpcName,
    ItemName,
}

const ENTITIES: [(&str, &str); 8] = [
    ("&nbsp;", " "),
    ("&apos;", "'"),
    ("&quot;", "\""),
    ("&amp;", "&"),
    ("&lt;", "<"),
    ("&gt;", "> "),
    ("&#160;", " "),
    ("&#10;", "&$"),
];

const MARKERS: [(&str, Marker); 6] = [
    ("&$", Marker::SysString),
    ("&%", Marker::CastleName),
    ("&^", Marker::CastleLocation),
    ("&*", Marker::CastleDescription),
    ("&@", Marker::NpcName),
    ("&#", Marker::ItemName),
];

const MISSING_MARKER: &str = "-";

fn find_from(text: &[char], needle: &[char], from: usize) -> Option<usize> {
    if needle.is_empty() || text.len() < needle.len() {
        return None;
    }
    (from..=text.len() - needle.len()).find(|&at| text[at..at + needle.len()] == *needle)
}

fn atoi(text: &[char]) -> i32 {
    let mut chars = text.iter().skip_while(|c| c.is_whitespace()).peekable();
    let negative = match chars.peek() {
        Some('-') => {
            chars.next();
            true
        }
        Some('+') => {
            chars.next();
            false
        }
        _ => false,
    };
    let mut value: i32 = 0;
    for digit in chars.map_while(|c| c.to_digit(10)) {
        value = value.wrapping_mul(10).wrapping_add(digit as i32);
    }
    if negative { value.wrapping_neg() } else { value }
}

pub fn expand_markers(text: &str, resolve: &dyn Fn(Marker, i32) -> Option<String>) -> String {
    let mut chars: Vec<char> = text.chars().collect();
    for (entity, replacement) in ENTITIES {
        let entity: Vec<char> = entity.chars().collect();
        let replacement: Vec<char> = replacement.chars().collect();
        while let Some(at) = find_from(&chars, &entity, 0) {
            chars.splice(at..at + entity.len(), replacement.iter().copied());
        }
    }
    for (prefix, marker) in MARKERS {
        let prefix: Vec<char> = prefix.chars().collect();
        let terminator = [';'];
        while let Some(start) = find_from(&chars, &prefix, 0) {
            let Some(end) = find_from(&chars, &terminator, start) else { break };
            let id = atoi(&chars[start + prefix.len()..end]);
            if end > start + prefix.len() && id != 0 {
                let name = resolve(marker, id).unwrap_or_else(|| if marker == Marker::SysString { String::new() } else { MISSING_MARKER.to_string() });
                chars.splice(start..=end, name.chars());
            } else {
                chars.drain(start..end);
                break;
            }
        }
    }
    chars.into_iter().collect()
}

pub fn parse(html: &str) -> Document {
    parse_with(html, &|_, _| None)
}

pub fn parse_with(html: &str, resolve: &dyn Fn(Marker, i32) -> Option<String>) -> Document {
    let mut builder = Builder::new(resolve);
    for token in tokenize(html) {
        match token {
            Token::Text(text) => builder.text(&text),
            Token::Open { name, attrs } => builder.open(&name, &attrs),
            Token::Close(name) => builder.close(&name),
        }
    }
    let mut document = builder.finish();
    document.tokens = crate::html_tokens::tokenize(html, resolve);
    document
}

impl Document {
    pub fn tokens(&self) -> &[crate::html_tokens::Token] {
        &self.tokens
    }

    pub fn text_anim(&self) -> bool {
        self.tokens
            .iter()
            .any(|token| token.kind == crate::html_tokens::KIND_HTML && token.open && token.attribute(crate::html_tokens::ATTR_HTML_TEXTANI) == Some("1"))
    }
}

pub fn atoi_text(text: &str) -> i32 {
    let trimmed = text.trim_start_matches([' ', '\t', '\n', '\r', '\u{b}', '\u{c}']);
    let (negative, digits) = match trimmed.chars().next() {
        Some('-') => (true, &trimmed[1..]),
        Some('+') => (false, &trimmed[1..]),
        _ => (false, trimmed),
    };
    let mut value: i32 = 0;
    for digit in digits.chars().map_while(|c| c.to_digit(10)) {
        value = value.wrapping_mul(10).wrapping_add(digit as i32);
    }
    if negative { value.wrapping_neg() } else { value }
}

pub fn layout(doc: &Document, width: i32, font: &BitmapFont) -> Vec<Line> {
    let painted = crate::html_layout::run(doc.tokens(), width, font);
    let mut lines: Vec<Line> = Vec::new();
    let mut spans: Vec<(i32, Span)> = Vec::new();
    let mut widgets: Vec<(i32, Widget)> = Vec::new();
    let mut rules: Vec<Rule> = Vec::new();
    let mut fills: Vec<Fill> = Vec::new();
    for primitive in &painted.primitives {
        match primitive {
            Primitive::Text(text) => {
                collect_text(text, font, &mut spans, &mut rules);
            }
            Primitive::Line { x0, y0, x1, y1 } => rules.push(Rule { x0: *x0, y0: *y0, x1: *x1, y1: *y1, color: TEXT }),
            Primitive::Fill { x, y, w, h, color } => fills.push(Fill { x: *x, y: *y, w: *w, h: *h, color: Argb(*color) }),
            Primitive::Image { x, y, w, h, src } => {
                widgets.push((*y, Widget { x: *x, width: *w, height: *h, kind: WidgetKind::Image(src.clone()), action: None, message: None, pieces: Vec::new() }));
            }
        }
    }
    for item in &painted.boxes {
        let (kind, action) = match &item.kind {
            LeafKind::Text => (WidgetKind::Link, Some(encode_action(item.subtype, &item.action))),
            LeafKind::Button { value, back, fore, action } => {
                (WidgetKind::Button { value: value.clone(), back: back.clone(), fore: fore.clone() }, Some(action.clone()).filter(|a| !a.is_empty()))
            }
            LeafKind::Edit { var, action, length, kind } => (WidgetKind::Edit { var: var.clone(), multi: false, action: action.clone(), length: *length, kind: *kind }, None),
            LeafKind::MultiEdit { var } => (WidgetKind::Edit { var: var.clone(), multi: true, action: String::new(), length: 0, kind: 0 }, None),
            LeafKind::ComboBox { var, list, selected } => (WidgetKind::ComboBox { var: var.clone(), list: list.clone(), selected: selected.clone() }, None),
            _ => continue,
        };
        let message = Some(item.message.clone()).filter(|message| matches!(item.kind, LeafKind::Text) && !message.is_empty());
        let pieces = if matches!(item.kind, LeafKind::Text) { link_pieces(item, crate::html_layout::glyph_a(font).1) } else { Vec::new() };
        widgets.push((item.y, Widget { x: item.x, width: item.w, height: item.h, kind, action, message, pieces }));
    }
    for (y, span) in spans {
        let index = line_at(&mut lines, y);
        lines[index].spans.push(span);
    }
    for (y, widget) in widgets {
        let index = line_at(&mut lines, y);
        lines[index].widgets.push(widget);
    }
    lines.sort_by_key(|line| line.y);
    if lines.is_empty() {
        lines.push(Line::default());
    }
    lines[0].rules = rules;
    lines[0].fills = fills;
    lines[0].extent = painted.height;
    lines
}

fn encode_action(subtype: i32, action: &str) -> String {
    match subtype {
        1 => format!("{LOCAL_LINK_PREFIX}{action}"),
        9 => format!("{CMD_PREFIX}{action}"),
        _ => action.to_string(),
    }
}

fn line_at(lines: &mut Vec<Line>, y: i32) -> usize {
    match lines.iter().position(|line| line.y == y) {
        Some(index) => index,
        None => {
            lines.push(Line { y, ..Line::default() });
            lines.len() - 1
        }
    }
}

fn collect_text(text: &TextDraw, font: &BitmapFont, spans: &mut Vec<(i32, Span)>, rules: &mut Vec<Rule>) {
    let action = Some(encode_action(text.subtype, &text.action)).filter(|_| text.link);
    let message = Some(text.message.clone()).filter(|message| text.link && !message.is_empty());
    let space = font.glyph(' ').map_or(0, |glyph| glyph.width as i32);
    let mut current: Option<(i32, Span, i32)> = None;
    for (x, y, c, color) in &text.glyphs {
        let advance = font.glyph(*c).map_or(0, |glyph| glyph.width as i32);
        if let Some((line_y, span, end)) = current.as_mut()
            && *line_y == *y
            && span.color == *color
            && (*end == *x || *end + space == *x)
        {
            if *end != *x {
                span.text.push(' ');
                *end += space;
            }
            span.text.push(*c);
            *end += advance;
            continue;
        }
        if let Some((line_y, span, _)) = current.take() {
            spans.push((line_y, span));
        }
        current = Some((*y, Span { x: *x, text: c.to_string(), color: *color, action: action.clone(), message: message.clone() }, *x + advance));
    }
    if let Some((line_y, span, _)) = current {
        spans.push((line_y, span));
    }
    let mut run: Option<Rule> = None;
    for (x0, x1, y) in &text.underline {
        match run.as_mut() {
            Some(rule) if rule.y0 == *y && rule.x1 == *x0 => rule.x1 = *x1,
            _ => {
                if let Some(rule) = run.take() {
                    rules.push(rule);
                }
                run = Some(Rule { x0: *x0, y0: *y, x1: *x1, y1: *y, color: text.base });
            }
        }
    }
    if let Some(rule) = run {
        rules.push(rule);
    }
}

pub fn height(lines: &[Line], font: &BitmapFont) -> i32 {
    let step = line_step(font);
    lines
        .iter()
        .map(|line| {
            let text = if line.spans.is_empty() { 0 } else { step };
            let widgets = line.widgets.iter().map(|w| w.height).max().unwrap_or(0);
            (line.y + text.max(widgets)).max(line.extent)
        })
        .max()
        .unwrap_or(0)
}

pub fn edit_at(lines: &[Line], x: i32, y: i32) -> Option<&str> {
    lines.iter().flat_map(|line| line.widgets.iter().map(move |widget| (line.y, widget))).find_map(|(top, widget)| match &widget.kind {
        WidgetKind::Edit { var, .. } if x >= widget.x && x < widget.x + widget.width && y >= top && y < top + widget.height => Some(var.as_str()),
        _ => None,
    })
}

pub fn link_at<'a>(lines: &'a [Line], x: i32, y: i32, _font: &BitmapFont) -> Option<&'a str> {
    link_widget_at(lines, x, y).and_then(|widget| widget.action.as_deref())
}

pub fn expand_system_msg_code(code: &str, lookup: impl Fn(u32) -> Option<String>) -> String {
    let digits: String = code.trim_start().chars().take_while(char::is_ascii_digit).collect();
    let id = digits.parse::<u32>().unwrap_or(0);
    if id == 0 || code.is_empty() {
        return code.to_string();
    }
    let text = lookup(id).unwrap_or_default();
    let chars: Vec<char> = code.chars().collect();
    let argument: String = (0..chars.len().saturating_sub(1))
        .rev()
        .find(|index| chars[*index] == ';')
        .map(|index| chars[index + 1..].iter().collect())
        .unwrap_or_default();
    match text.find("$s1") {
        Some(position) => format!("{}{}{}", &text[..position], argument, &text[position + 3..]),
        None => text,
    }
}

pub fn link_message_at(lines: &[Line], x: i32, y: i32) -> Option<&str> {
    link_widget_at(lines, x, y).and_then(|widget| widget.message.as_deref())
}

fn link_widget_at(lines: &[Line], x: i32, y: i32) -> Option<&Widget> {
    lines.iter().flat_map(|line| line.widgets.iter().map(move |widget| (line.y, widget))).find_map(|(top, widget)| {
        widget.action.as_deref()?;
        let inside = |p: &LinkPiece| x >= p.x && x < p.x + p.width && y >= top + p.y && y < top + p.y + p.height;
        if widget.pieces.is_empty() {
            (x >= widget.x && x < widget.x + widget.width && y >= top && y < top + widget.height).then_some(widget)
        } else {
            widget.pieces.iter().any(inside).then_some(widget)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::Glyphs;

    const SMALL: &[u8] = include_bytes!("../tests/fixtures/smallfont-e.gly");
    const MERCHANT: &str = include_str!("../tests/fixtures/npc-7829.htm");
    const PRIEST: &str = include_str!("../tests/fixtures/npc-dawn-priest-2c.htm");
    const SIGNBOARD: &str = include_str!("../tests/fixtures/npc-signboard.htm");
    const HEINE_PET: &str = include_str!("../tests/fixtures/npc-heine-8067.htm");
    const HEINE_TELEPORT: &str = include_str!("../tests/fixtures/npc-heine-7899-1.htm");
    const TERRITORY: &str = include_str!("../tests/fixtures/npc-territorystatus.htm");
    const AGIT_SALE: &str = include_str!("../tests/fixtures/npc-agitsale1.htm");

    fn font() -> BitmapFont {
        BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(SMALL).unwrap())
    }

    fn texts(nodes: &[Node]) -> Vec<String> {
        nodes
            .iter()
            .filter_map(|node| match node {
                Node::Text(text) => Some(text.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_message_of_an_action_link_expands_the_system_message_with_its_argument_1004b8e0() {
        let lookup = |id: u32| match id {
            1700 => Some("Pay $s1 adena?".to_string()),
            1701 => Some("Sure?".to_string()),
            _ => None,
        };
        assert_eq!(expand_system_msg_code("1700;500", lookup), "Pay 500 adena?");
        assert_eq!(expand_system_msg_code("1700;a;b", lookup), "Pay b adena?");
        assert_eq!(expand_system_msg_code("1700", lookup), "Pay  adena?");
        assert_eq!(expand_system_msg_code("1700;", lookup), "Pay  adena?");
        assert_eq!(expand_system_msg_code("1701;x", lookup), "Sure?");
        assert_eq!(expand_system_msg_code("hello", lookup), "hello");
        assert_eq!(expand_system_msg_code("0;1", lookup), "0;1");
        assert_eq!(expand_system_msg_code("-5;1", lookup), "-5;1");
        assert_eq!(expand_system_msg_code("9999;1", lookup), "");
    }

    #[test]
    fn only_action_links_carry_the_msg_attribute_into_the_widgets() {
        let doc = "<html><body><a action=\"bypass -h go\" msg=\"1700;5\">pay</a> <a link=\"x.htm\" msg=\"1701\">file</a> <a action=\"bypass -h plain\">plain</a></body></html>";
        let lines = layout(&parse_with(doc, &|_, _| None), 250, &font());
        let widgets: Vec<_> = lines.iter().flat_map(|line| line.widgets.iter()).collect();
        let pay = widgets.iter().find(|w| w.action.as_deref() == Some("bypass -h go")).unwrap();
        assert_eq!(pay.message.as_deref(), Some("1700;5"));
        let file = widgets.iter().find(|w| w.action.as_deref().is_some_and(|a| a.contains("x.htm"))).unwrap();
        assert_eq!(file.message, None);
        let plain = widgets.iter().find(|w| w.action.as_deref() == Some("bypass -h plain")).unwrap();
        assert_eq!(plain.message, None);
    }

    #[test]
    fn local_file_links_remain_distinct_from_server_link_actions() {
        let doc = parse(r#"<a link="..\L2text\help.htm">Local</a><a action="link tutorial.htm">Server</a>"#);
        let actions = doc.actions();
        assert_eq!(actions.len(), 2);
        assert_eq!(local_link_target(actions[0]), Some("..\\L2text\\help.htm"));
        assert_eq!(link_target(actions[0]), None);
        assert_eq!(local_link_target(actions[1]), None);
        assert_eq!(link_target(actions[1]), Some("tutorial.htm".into()));
    }

    #[test]
    fn parses_a_real_merchant_dialog_into_text_breaks_and_links() {
        let doc = parse(MERCHANT);
        assert_eq!(doc.title, None);
        assert_eq!(doc.body[0], Node::Text("Pet Manager Cooper:".into()));
        assert_eq!(doc.body[1], Node::Br);
        assert!(matches!(&doc.body[2], Node::Text(t) if t.starts_with("I'm glad you came") && t.ends_with("later.")));
        assert_eq!(doc.body[3], Node::Br);
        assert_eq!(
            doc.body[4],
            Node::Link {
                action: "bypass -h npc_%objectId%_Buy 92".into(),
                children: vec![Node::Text("Buy pet item".into())]
            }
        );
        let links = doc.body.iter().filter(|n| matches!(n, Node::Link { .. })).count();
        assert_eq!(links, 5);
        assert_eq!(doc.body.iter().filter(|n| **n == Node::Br).count(), 6);
        assert_eq!(doc.body.len(), 13);
    }

    #[test]
    fn whitespace_collapses_inside_a_run_and_is_trimmed_at_tag_boundaries() {
        let doc = parse("<html><body>  Hello   \n  world <br>  next\t line  </body></html>");
        assert_eq!(doc.body, vec![Node::Text("Hello world".into()), Node::Br, Node::Text("next line".into())]);
        let doc = parse(SIGNBOARD);
        assert_eq!(doc.body[0], Node::Text("This is the".into()));
        assert_eq!(
            doc.body[1],
            Node::Font { color: Some(LEVEL_COLOR), children: vec![Node::Text(" dueling area".into())] }
        );
        assert_eq!(doc.body[2], Node::Text(".".into()));
    }

    #[test]
    fn tags_and_attributes_are_case_insensitive_and_unknown_tags_are_skipped() {
        let doc = parse("<HTML><BODY><P align=left><B>x</B><Br1><A ACTION='bypass -h a'>y</A><tr><td>z</td></tr>");
        assert_eq!(
            doc.body,
            vec![
                Node::Text("x".into()),
                Node::Br1,
                Node::Link { action: "bypass -h a".into(), children: vec![Node::Text("y".into())] },
                Node::Text("z".into()),
            ]
        );
        let doc = parse("a<!-- comment <br> -->b<a href=\"x\">c</a><title>T</title>d");
        assert_eq!(texts(&doc.body), ["a", "b", "c", "T", "d"]);
        assert_eq!(doc.title.as_deref(), Some("T"));
    }

    #[test]
    fn font_colors_follow_the_client_table_and_hex_prefix_rule() {
        assert_eq!(font_color("LEVEL"), LEVEL_COLOR);
        assert_eq!(font_color("FF0000"), Argb(0xFFFF0000));
        assert_eq!(font_color("00ffff"), Argb(0xFF00FFFF));
        assert_eq!(font_color("#FF0000"), Argb(0x000000FF));
        assert_eq!(font_color("level"), Argb(0x000000FF));
        let doc = parse(PRIEST);
        let fonts: Vec<&Node> = doc.body.iter().filter(|n| matches!(n, Node::Font { .. })).collect();
        assert_eq!(fonts.len(), 4);
        assert_eq!(
            fonts[0],
            &Node::Font {
                color: Some(Argb(0xFFFF0000)),
                children: vec![Node::Text("[ The seal validation period is underway. ]".into())]
            }
        );
        assert!(matches!(fonts[1], Node::Font { color: Some(c), .. } if *c == LEVEL_COLOR));
    }

    #[test]
    fn center_tables_images_and_buttons_become_nodes() {
        let html = "<center><a action=\"bypass -h npc_1_Chat 1\">MP!</a><br></center>\
                    <table width=260><tr><td width=40>a</td><td>b</td></tr><tr><td fixwidth=60>c</td></tr></table>\
                    <img src=\"L2UI.SquareWhite\" width=270 height=1>\
                    <button value=\"Return\" action=\"bypass -h npc_1_Chat 0\" width=\"60\" height=\"15\" back=\"sek.cbui94\" fore=\"sek.cbui92\">\
                    <button value=x action=y>";
        let doc = parse(html);
        assert_eq!(
            doc.body[0],
            Node::Center(vec![
                Node::Link { action: "bypass -h npc_1_Chat 1".into(), children: vec![Node::Text("MP!".into())] },
                Node::Br
            ])
        );
        assert_eq!(
            doc.body[1],
            Node::Table {
                width: Some(260),
                rows: vec![
                    vec![
                        Cell { width: Some(40), children: vec![Node::Text("a".into())] },
                        Cell { width: None, children: vec![Node::Text("b".into())] }
                    ],
                    vec![Cell { width: Some(60), children: vec![Node::Text("c".into())] }],
                ]
            }
        );
        assert_eq!(doc.body[2], Node::Img { src: "L2UI.SquareWhite".into(), width: 270, height: 1 });
        assert_eq!(
            doc.body[3],
            Node::Button {
                value: "Return".into(),
                action: "bypass -h npc_1_Chat 0".into(),
                width: 60,
                height: 15,
                back: "sek.cbui94".into(),
                fore: "sek.cbui92".into()
            }
        );
        assert_eq!(
            doc.body[4],
            Node::Button {
                value: "x".into(),
                action: "y".into(),
                width: 10,
                height: 10,
                back: BUTTON_DEFAULT_BACK.into(),
                fore: BUTTON_DEFAULT_FORE.into()
            }
        );
    }

    #[test]
    fn unclosed_tags_are_closed_at_the_end_and_stray_closers_are_ignored() {
        let doc = parse("</a></center><a action=\"z\">open<font color=\"LEVEL\">gold");
        assert_eq!(
            doc.body,
            vec![Node::Link {
                action: "z".into(),
                children: vec![
                    Node::Text("open".into()),
                    Node::Font { color: Some(LEVEL_COLOR), children: vec![Node::Text("gold".into())] }
                ]
            }]
        );
        assert_eq!(parse("").body, Vec::<Node>::new());
        assert_eq!(parse("<html><body></body></html>").body, Vec::<Node>::new());
    }

    #[test]
    fn bypass_command_drops_the_prefix_and_the_h_flag() {
        assert_eq!(bypass_command("bypass -h npc_1234_Chat 1").as_deref(), Some("npc_1234_Chat 1"));
        assert_eq!(bypass_command("BYPASS npc_1_Sell").as_deref(), Some("npc_1_Sell"));
        assert_eq!(
            bypass_command("bypass -h  npc_1_SevenSigns 11 -80157 111344").as_deref(),
            Some("npc_1_SevenSigns 11 -80157 111344")
        );
        assert_eq!(bypass_command("bypass"), None, "NConsoleWnd::RunHtmlCommand 0x10072560: sem token nada é enviado");
        assert_eq!(bypass_command("bypass -h").as_deref(), Some(""));
        assert_eq!(bypass_command("bypass -h -h x").as_deref(), Some("-h x"));
        assert_eq!(bypass_command("link merchant/7829.htm"), None);
        assert_eq!(bypass_command(""), None);
        assert_eq!(link_target("link merchant/7829.htm").as_deref(), Some("merchant/7829.htm"));
        assert_eq!(link_target("bypass -h x"), None);
    }

    fn page(body: &str) -> Document {
        parse(&format!("<html><body>{body}</body></html>"))
    }

    fn link_lines(lines: &[Line]) -> Vec<&Line> {
        lines.iter().filter(|l| l.spans.iter().any(|s| s.action.is_some())).collect()
    }

    #[test]
    fn a_document_without_html_or_body_draws_nothing_like_the_original_builder() {
        let font = font();
        assert!(layout(&parse("A<br>B"), 296, &font).iter().all(|l| l.spans.is_empty()));
    }

    #[test]
    fn lines_step_nineteen_pixels_and_br_adds_nine_more() {
        let font = font();
        assert_eq!(line_step(&font), 19);
        let lines = layout(&page("A<br>B<br1>C"), 296, &font);
        assert_eq!(lines.len(), 3);
        assert_eq!((lines[0].y, lines[0].spans[0].text.as_str()), (0, "A"));
        assert_eq!((lines[1].y, lines[1].spans[0].text.as_str()), (28, "B"));
        assert_eq!((lines[2].y, lines[2].spans[0].text.as_str()), (47, "C"));
        assert_eq!(height(&lines, &font), 66);
        assert_eq!(layout(&page("A<br><br>B"), 296, &font)[1].y, 37);
    }

    #[test]
    fn links_take_the_link_color_underline_and_a_hit_box_of_the_leaf_rectangle() {
        let font = font();
        let lines = layout(&page("Hi <a action=\"bypass -h npc_1_Chat 1\">there</a>!"), 296, &font);
        assert_eq!(lines.len(), 1);
        let spans = &lines[0].spans;
        assert_eq!(spans.len(), 3);
        assert_eq!((spans[0].x, spans[0].text.as_str(), spans[0].color, spans[0].action.as_deref()), (0, "Hi", TEXT, None));
        assert_eq!((spans[1].text.as_str(), spans[1].color, spans[1].action.as_deref()), ("there", LINK_COLOR, Some("bypass -h npc_1_Chat 1")));
        assert_eq!(lines[0].rules.len(), 1);
        let rule = &lines[0].rules[0];
        assert_eq!((rule.x0, rule.x1, rule.color), (spans[1].x, spans[1].x + font.measure("there") as i32, LINK_COLOR));
        assert_eq!(link_at(&lines, spans[1].x + 2, 5, &font), Some("bypass -h npc_1_Chat 1"));
        assert_eq!(link_at(&lines, 1, 5, &font), None);
        assert_eq!(link_at(&lines, spans[1].x + 2, 19, &font), None);
    }

    #[test]
    fn font_color_inside_a_run_keeps_the_flow_and_splits_spans() {
        let font = font();
        let lines = layout(&parse(SIGNBOARD), 296, &font);
        let first = &lines[0].spans;
        assert_eq!(first[0].text, "This is the");
        assert_eq!(first[1].color, LEVEL_COLOR);
        assert!(first[1].text.contains("dueling"));
    }

    #[test]
    fn long_text_wraps_by_word_and_justifies_inside_the_width() {
        let font = font();
        let text = "I'm glad you came to the Pet Kennel of Giran Castle Town and I hope you enjoy the stay.";
        let lines = layout(&page(text), 120, &font);
        assert!(lines.len() > 1);
        for line in &lines {
            let end = line.spans.iter().map(|s| s.x + font.measure(&s.text) as i32).max().unwrap();
            assert!(end <= 120, "linha ultrapassa a largura: {end}");
        }
        assert_eq!(lines[1].y, 19);
        let words: Vec<&str> = lines.iter().flat_map(|l| l.spans.iter().flat_map(|s| s.text.split(' '))).collect();
        assert_eq!(words.join(" "), text);
        let long = layout(&page("abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz"), 40, &font);
        assert!(long.len() > 1);
    }

    #[test]
    fn centered_text_sits_in_the_middle_and_the_following_text_returns_to_the_left() {
        let font = font();
        let lines = layout(&page("<center>MP!<br>go</center>after"), 296, &font);
        assert_eq!(lines.len(), 3);
        assert_eq!((lines[0].y, lines[0].spans[0].x), (0, 139));
        assert_eq!((lines[1].y, lines[1].spans[0].text.as_str()), (28, "go"));
        assert_eq!((lines[2].y, lines[2].spans[0].x, lines[2].spans[0].text.as_str()), (47, 0, "after"));
    }

    #[test]
    fn images_and_buttons_are_leaves_with_their_own_boxes() {
        let font = font();
        let html = "x<img src=\"L2UI.SquareWhite\" width=270 height=1>y<button value=\"Return\" action=\"bypass -h npc_1_Chat 0\" width=60 height=15 back=\"sek.cbui94\" fore=\"sek.cbui92\">z";
        let lines = layout(&page(html), 296, &font);
        assert_eq!(lines.iter().map(|l| l.y).collect::<Vec<_>>(), [0, 19, 20, 39, 54]);
        assert_eq!(lines[1].widgets[0], Widget { x: 0, width: 270, height: 1, kind: WidgetKind::Image("L2UI.SquareWhite".into()), action: None, message: None, pieces: Vec::new() });
        let button = &lines[3].widgets[0];
        assert_eq!((button.width, button.height, button.action.as_deref()), (60, 15, Some("bypass -h npc_1_Chat 0")));
        assert_eq!(button.kind, WidgetKind::Button { value: "Return".into(), back: "sek.cbui94".into(), fore: "sek.cbui92".into() });
        assert_eq!(link_at(&lines, 30, 39 + 7, &font), Some("bypass -h npc_1_Chat 0"));
        assert_eq!(link_at(&lines, 70, 39 + 7, &font), None);
        assert_eq!(height(&lines, &font), 73);
    }

    #[test]
    fn tables_use_the_default_padding_and_spacing_of_the_original_builder() {
        let font = font();
        let html = "<table width=200><tr><td width=40>a</td><td>b</td><td>c</td></tr><tr><td>d<br>e</td><td>f</td></tr></table>after";
        let lines = layout(&page(html), 296, &font);
        assert_eq!((lines[0].y, lines[0].spans.iter().map(|s| s.x).collect::<Vec<_>>()), (3, vec![3, 135, 152]));
        assert_eq!(lines.last().map(|l| (l.y, l.spans[0].text.as_str())), Some((64, "after")));
        assert_eq!(layout(&page("<table></table>x"), 296, &font).iter().flat_map(|l| &l.spans).map(|s| s.text.as_str()).collect::<String>(), "x");
    }

    #[test]
    fn the_merchant_dialog_lays_out_five_links_below_the_wrapped_speech() {
        let font = font();
        let lines = layout(&parse(MERCHANT), 296, &font);
        assert_eq!((lines[0].y, lines[0].spans[0].text.as_str()), (0, "Pet Manager Cooper:"));
        assert_eq!(lines[1].y, 28);
        let links = link_lines(&lines);
        assert_eq!(links.len(), 5);
        assert!(links.iter().all(|l| l.spans[0].color == LINK_COLOR && l.spans[0].x == 0));
        let sell = links[1];
        assert_eq!(link_at(&lines, 10, sell.y + 6, &font), Some("bypass -h npc_%objectId%_Sell"));
        assert_eq!(bypass_command(link_at(&lines, 10, sell.y + 6, &font).unwrap()).as_deref(), Some("npc_%objectId%_Sell"));
    }

    #[test]
    fn heine_dialogs_keep_every_link_hittable_and_carry_the_msg_attribute_before_the_action() {
        let font = font();
        let lines = layout(&parse(HEINE_PET), 296, &font);
        let links = link_lines(&lines);
        let actions: Vec<&str> = links.iter().flat_map(|l| l.spans.iter().filter_map(|s| s.action.as_deref())).collect();
        assert_eq!(actions.iter().collect::<std::collections::BTreeSet<_>>().len(), 6);
        let buy = links[0];
        assert_eq!(link_at(&lines, 3, buy.y + 5, &font), Some("bypass -h npc_%objectId%_Buy 5710"));
        let doc = parse(HEINE_TELEPORT);
        let lines = layout(&doc, 296, &font);
        let links = link_lines(&lines);
        assert_eq!(links.len(), 6);
        assert_eq!(links.windows(2).map(|w| w[1].y - w[0].y).collect::<Vec<_>>(), [19; 5]);
        let goto = link_at(&lines, 40, links[5].y + 10, &font).unwrap();
        assert_eq!(split_message(goto), (None, "bypass -h npc_%objectId%_goto 68"));
        assert_eq!(link_message_at(&lines, 40, links[5].y + 10), Some("811;Entrance to Alligator Island"));
        assert_eq!(bypass_command(goto).as_deref(), Some("npc_%objectId%_goto 68"));
        assert_eq!(message_code("811;Entrance to Alligator Island"), Some((811, "Entrance to Alligator Island".to_string())));
        assert_eq!(doc.actions().len(), 6);
        assert_eq!(parse(HEINE_PET).actions()[0], "bypass -h npc_%objectId%_Buy 5710");
        assert_eq!(parse(AGIT_SALE).actions(), ["bypass -h npc_%objectId%_selectedItems", "bypass -h npc_%objectId%_sale2"]);
        assert_eq!(parse(TERRITORY).actions(), ["bypass -h npc_%objectId%_Chat 0"]);
    }

    #[test]
    fn buttons_inside_table_cells_are_hit_through_the_leaf_rectangle() {
        let font = font();
        let lines = layout(&parse(TERRITORY), 296, &font);
        let button = lines.iter().find(|l| l.widgets.iter().any(|w| matches!(w.kind, WidgetKind::Button { .. }))).unwrap();
        let widget = button.widgets.iter().find(|w| matches!(w.kind, WidgetKind::Button { .. })).unwrap();
        assert_eq!((widget.width, widget.height), (75, 21));
        assert_eq!(link_at(&lines, widget.x + 40, button.y + 10, &font), Some("bypass -h npc_%objectId%_Chat 0"));
        assert_eq!(link_at(&lines, widget.x + 80, button.y + 10, &font), None);
        let lines = layout(&parse(AGIT_SALE), 296, &font);
        let row = lines.iter().find(|l| l.widgets.iter().any(|w| matches!(w.kind, WidgetKind::Button { .. }))).unwrap();
        let buttons: Vec<&Widget> = row.widgets.iter().filter(|w| matches!(w.kind, WidgetKind::Button { .. })).collect();
        assert_eq!(buttons.len(), 2);
        assert_eq!(link_at(&lines, buttons[0].x + 35, row.y + 10, &font).map(|a| a.to_string()), buttons[0].action.clone());
        assert_eq!(link_at(&lines, buttons[1].x + 35, row.y + 10, &font).map(|a| a.to_string()), buttons[1].action.clone());
    }

    #[test]
    fn text_markers_follow_the_original_replace_entities_pass() {
        let resolve = |marker: Marker, id: i32| match (marker, id) {
            (Marker::NpcName, 30045) => Some("Malcom".to_string()),
            (Marker::ItemName, 57) => Some("Adena".to_string()),
            (Marker::SysString, 5) => Some("Cancel".to_string()),
            (Marker::CastleName, 1) => Some("Gludio".to_string()),
            (Marker::CastleLocation, 1) => Some("Gludio Territory".to_string()),
            (Marker::CastleDescription, 1) => Some("Castle".to_string()),
            _ => None,
        };
        assert_eq!(expand_markers("a&nbsp;b&amp;c&lt;d&quot;e&apos;f", &resolve), "a b&c<d\"e'f");
        assert_eq!(expand_markers("&@30045; sells &#57; at &%1; &^1; &*1; &$5;", &resolve), "Malcom sells Adena at Gludio Gludio Territory Castle Cancel");
        assert_eq!(expand_markers("&@999; &#58; &%2;", &resolve), "- - -");
        assert_eq!(expand_markers("&$999;|", &resolve), "|");
        assert_eq!(expand_markers("&@abc; &@30045;", &resolve), "; &@30045;");
        assert_eq!(expand_markers("&@30045", &resolve), "&@30045");
        let doc = parse_with("<html><body>Hi &@30045;&nbsp;there</body></html>", &resolve);
        assert_eq!(doc.body, vec![Node::Text("Hi Malcom there".into())]);
    }

    #[test]
    fn the_gm_command_page_links_carry_the_command_to_be_said() {
        let font = font();
        let page = crate::gm_event::command_html(&crate::ini::parse_ini("[Cmd]\nCmdCnt=2\nCmd0=eventmatch start 3\nCmd1=c\n"));
        let lines = layout(&parse(&page), 300, &font);
        let found: Vec<_> = lines.iter().filter_map(|line| line.widgets.iter().find_map(|w| w.action.as_deref().map(|a| (line.y, w.x, a)))).collect();
        assert_eq!(found.len(), 2);
        let (y, x, action) = found[0];
        assert_eq!(cmd_target(action), Some("eventmatch start 3"));
        assert_eq!(link_at(&lines, x, y, &font).and_then(cmd_target), Some("eventmatch start 3"));
        assert_eq!(cmd_target(found[1].2), Some("c"));
    }

    fn linker_box(h: i32) -> crate::html_layout::LeafBox {
        crate::html_layout::LeafBox {
            kind: LeafKind::Text, x: 30, y: 0, w: 90, h, action: String::new(), message: String::new(),
            subtype: 1, end: 40, wrap: 100, parent_x: 10,
        }
    }

    #[test]
    fn dynamic_linker_1004a800_splits_a_wrapped_link_into_one_two_or_three_pieces() {
        let one = link_pieces(&linker_box(19), 13);
        assert_eq!(one, [LinkPiece { x: 30, y: 0, width: 90, height: 13 }]);
        let two = link_pieces(&linker_box(38), 13);
        assert_eq!(
            two,
            [LinkPiece { x: 30, y: 0, width: 100, height: 13 }, LinkPiece { x: 10, y: 19, width: 40, height: 13 }]
        );
        let three = link_pieces(&linker_box(57), 13);
        assert_eq!(
            three,
            [
                LinkPiece { x: 30, y: 0, width: 100, height: 13 },
                LinkPiece { x: 10, y: 19, width: 100, height: 19 },
                LinkPiece { x: 10, y: 44, width: 40, height: 13 },
            ]
        );
        assert_eq!(link_pieces(&linker_box(38), 13).len(), 2);
        assert_eq!(link_pieces(&linker_box(39), 13).len(), 3);
    }

    #[test]
    fn a_wrapped_action_link_is_hit_only_inside_its_pieces() {
        let font = font();
        let lines = layout(&page("<a action=\"bypass -h go\">alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi</a>"), 80, &font);
        let widget = lines.iter().flat_map(|line| line.widgets.iter()).find(|w| w.action.is_some()).unwrap();
        assert!(widget.pieces.len() >= 2);
        let first = widget.pieces[0];
        assert_eq!(link_at(&lines, first.x + 1, first.y + 1, &font), Some("bypass -h go"));
        let last = *widget.pieces.last().unwrap();
        assert_eq!(link_at(&lines, last.x + 1, last.y + 1, &font), Some("bypass -h go"));
        assert_eq!(link_at(&lines, first.x + 1, first.y + first.height + 2, &font), None);
    }
}
