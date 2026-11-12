use crate::html_tags::TAGS;

pub const KIND_TEXT: u32 = 1;
pub const KIND_UNKNOWN: u32 = 2;
pub const KIND_COMMENT: u32 = 3;
pub const KIND_EXTEND: u32 = 4;
pub const KIND_IMG: u32 = 5;
pub const KIND_A: u32 = 14;
pub const KIND_BR: u32 = 15;
pub const KIND_CENTER: u32 = 18;
pub const KIND_TABLE: u32 = 40;
pub const KIND_TR: u32 = 41;
pub const KIND_TD: u32 = 42;
pub const KIND_HTML: u32 = 43;
pub const ATTR_HTML_TEXTANI: usize = 5;
pub const KIND_BODY: u32 = 44;
pub const KIND_BUTTON: u32 = 49;
pub const KIND_MULTIEDIT: u32 = 50;
pub const KIND_EDIT: u32 = 51;
pub const KIND_COMBOBOX: u32 = 52;
pub const KIND_BR1: u32 = 53;

const MARKER: char = '\u{1e}';

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Token {
    pub kind: u32,
    pub open: bool,
    pub attributes: Vec<(usize, String)>,
    pub text: String,
    pub colors: String,
}

impl Token {
    pub fn attribute(&self, id: usize) -> Option<&str> {
        self.attributes.iter().find(|(key, _)| *key == id).map(|(_, value)| value.as_str())
    }
}

fn is_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\r' | '\n')
}

fn at(text: &[char], index: usize) -> char {
    text.get(index).copied().unwrap_or('\0')
}

fn skip_space(text: &[char], mut index: usize) -> usize {
    while is_space(at(text, index)) {
        index += 1;
    }
    index
}

fn name_end(text: &[char], mut index: usize) -> usize {
    while !matches!(at(text, index), '\0' | ' ' | '\t' | '\r' | '\n' | '=' | '>' | '<') {
        index += 1;
    }
    index
}

fn token_end(text: &[char], mut index: usize) -> usize {
    while !matches!(at(text, index), '\0' | ' ' | '\t' | '\r' | '\n' | '>' | '<') {
        index += 1;
    }
    index
}

fn is_alphanumeric(c: char) -> bool {
    c.is_ascii_alphanumeric()
}

fn upper(c: char) -> char {
    c.to_uppercase().next().unwrap_or(c)
}

fn starts_with(text: &[char], index: usize, needle: &str) -> bool {
    needle.chars().enumerate().all(|(offset, c)| at(text, index + offset) == c)
}

fn lookup_tag(name: &str) -> Option<usize> {
    TAGS.binary_search_by(|tag| tag.name.cmp(name)).ok()
}

fn lookup_attribute(tag: usize, name: &str) -> Option<usize> {
    TAGS[tag].attributes.binary_search_by(|attribute| (*attribute).cmp(name)).ok().map(|index| index + 1)
}

const UNKNOWN_TAG: i32 = -10;
const NOT_A_TAG: i32 = -100;

fn tag_name(text: &[char], start: usize, token: &mut Token) -> (i32, usize) {
    let mut index = start;
    if at(text, index) == '!' {
        if at(text, index + 1) == '-' {
            index += 2;
            if at(text, index) == '-' {
                index += 1;
            }
            return (-1, index);
        }
        index = skip_space(text, index + 1);
    }
    if at(text, index) == '/' {
        token.open = false;
        index = skip_space(text, index + 1);
    } else {
        token.open = true;
    }
    if starts_with(text, index, "font") {
        return (NOT_A_TAG, index);
    }
    if !is_alphanumeric(at(text, index)) {
        return (NOT_A_TAG, index);
    }
    let end = token_end(text, index);
    if end == index {
        return (NOT_A_TAG, index);
    }
    let name: String = text[index..end].iter().map(|c| upper(*c)).collect();
    match lookup_tag(&name) {
        Some(tag) => (tag as i32, end),
        None => (UNKNOWN_TAG, end),
    }
}

fn skip_to_close(text: &[char], start: usize) -> usize {
    let mut index = start + 1;
    loop {
        match at(text, index) {
            '\0' | '<' => return index,
            '>' => return index + 1,
            _ => index += 1,
        }
    }
}

fn parse_attributes(text: &[char], mut index: usize, tag: usize, token: &mut Token) -> usize {
    loop {
        if at(text, index) == '\0' {
            return index;
        }
        index = skip_space(text, index);
        match at(text, index) {
            '<' => return index,
            '>' => return index + 1,
            _ => {}
        }
        let mut name = String::new();
        if !matches!(at(text, index), '=' | '"') {
            let end = name_end(text, index);
            if end != index {
                name = text[index..end].iter().map(|c| upper(*c)).collect();
                index = skip_space(text, end);
            }
        }
        let mut value = String::new();
        let mut quoted = false;
        if at(text, index) == '=' || at(text, index) == '"' {
            if at(text, index) == '=' {
                index = skip_space(text, index + 1);
            }
            let end;
            if at(text, index) == '"' {
                index += 1;
                quoted = true;
                let mut scan = index;
                while !matches!(at(text, scan), '\0' | '"') {
                    scan += 1;
                }
                end = scan;
            } else {
                end = token_end(text, index);
            }
            value = text[index..end].iter().collect();
            index = end;
            if quoted && at(text, index) == '"' {
                index += 1;
            }
            index = skip_space(text, index);
        }
        if !name.is_empty() && let Some(id) = lookup_attribute(tag, &name) {
            token.attributes.push((id, value));
        }
    }
}

fn skip_extend(text: &[char], start: usize, tag: usize) -> usize {
    let mut index = start + 1;
    loop {
        match at(text, index) {
            '\0' => return index,
            '<' => {
                index = skip_space(text, index + 1);
                let mut probe = Token::default();
                let (kind, end) = tag_name(text, index, &mut probe);
                if !probe.open && kind == tag as i32 {
                    let mut scan = end;
                    while !matches!(at(text, scan), '\0' | '>') {
                        scan += 1;
                    }
                    return scan + 1;
                }
            }
            _ => index += 1,
        }
    }
}

fn find_text(haystack: &[char], needle: &str, from: usize) -> usize {
    let needle: Vec<char> = needle.chars().collect();
    if needle.is_empty() {
        return from.min(haystack.len());
    }
    if haystack.len() < needle.len() {
        return usize::MAX;
    }
    (from..=haystack.len() - needle.len()).find(|&start| haystack[start..start + needle.len()] == needle[..]).unwrap_or(usize::MAX)
}

fn collapse(text: &mut Vec<char>, markers: bool) {
    let mut index = 0;
    let (mut seen, mut pending) = (false, false);
    while index < text.len() {
        let c = text[index];
        if is_space(c) {
            text.remove(index);
            if seen && (!markers || index == 0 || text[index - 1] != MARKER) {
                pending = true;
            }
        } else {
            seen = true;
            if pending {
                text.insert(index, ' ');
                pending = false;
            }
            index += 1;
        }
    }
}

fn replace_range(text: &mut Vec<char>, start: usize, count: usize, with: &[char]) {
    if start > text.len() {
        return;
    }
    let end = start.saturating_add(count).min(text.len());
    text.splice(start..end, with.iter().copied());
}

fn erase(text: &mut Vec<char>, start: usize, count: usize) {
    replace_range(text, start, count, &[]);
}

fn mid(text: &[char], start: usize, count: usize) -> Vec<char> {
    if start >= text.len() {
        return Vec::new();
    }
    text[start..start.saturating_add(count).min(text.len())].to_vec()
}

fn text_token(text: &[char], start: usize, resolve: &dyn Fn(crate::html::Marker, i32) -> Option<String>) -> (Token, usize) {
    let mut index = start;
    if at(text, index) == '<' {
        index += 1;
    }
    let begin = index;
    let mut has_font = false;
    while at(text, index) != '\0' {
        if at(text, index) == '<' {
            if starts_with(text, index, "</font") || starts_with(text, index, "<font") {
                has_font = true;
            } else {
                break;
            }
        }
        index += 1;
    }
    let _ = begin;
    let mut display: Vec<char> = text[start..index].to_vec();
    let mut colors: Vec<char> = Vec::new();
    if has_font {
        let marker = [MARKER];
        loop {
            let color_at = find_text(&display, "color=", 1);
            let mut quote = find_text(&display, "\"", color_at);
            if quote != 0 && quote < display.len() {
                replace_range(&mut display, quote, 1, &marker);
            }
            quote = find_text(&display, "\"", quote);
            if quote != 0 && quote < display.len() {
                replace_range(&mut display, quote, 1, &marker);
            }
            let close_angle = find_text(&display, ">", color_at);
            let font_end = find_text(&display, "</font>", close_angle);
            if color_at > display.len() {
                break;
            }
            let span = close_angle.wrapping_sub(color_at);
            colors.extend(mid(&display, color_at + 6, span.wrapping_sub(6)));
            let tail = font_end.wrapping_sub(close_angle);
            colors.extend(mid(&display, close_angle.wrapping_add(1), tail.wrapping_sub(1)));
            erase(&mut display, color_at.wrapping_sub(6), span.wrapping_add(7));
            erase(&mut display, color_at.wrapping_add(tail).wrapping_sub(7), 7);
            if color_at > display.len() {
                break;
            }
        }
    }
    collapse(&mut display, false);
    collapse(&mut colors, true);
    let display = crate::html::expand_markers(&display.iter().collect::<String>(), resolve);
    let colors = crate::html::expand_markers(&colors.iter().collect::<String>(), resolve);
    (Token { kind: KIND_TEXT, open: true, text: display, colors, ..Token::default() }, index)
}

pub fn tokenize(html: &str, resolve: &dyn Fn(crate::html::Marker, i32) -> Option<String>) -> Vec<Token> {
    let text: Vec<char> = html.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < text.len() {
        let start = index;
        let mut token = Token::default();
        if at(&text, index) == '<' {
            let after = skip_space(&text, index + 1);
            let (kind, end) = tag_name(&text, after, &mut token);
            if kind <= NOT_A_TAG {
                let (parsed, next) = text_token(&text, start, resolve);
                token = parsed;
                index = next;
            } else if kind < 0 {
                token.kind = if kind == -1 { KIND_COMMENT } else { KIND_UNKNOWN };
                index = skip_to_close(&text, start);
            } else {
                let tag = kind as usize;
                token.kind = TAGS[tag].kind;
                index = if token.kind > KIND_EXTEND {
                    parse_attributes(&text, end, tag, &mut token)
                } else if token.kind == KIND_EXTEND {
                    skip_extend(&text, start, tag)
                } else {
                    skip_to_close(&text, start)
                };
            }
            if index == start {
                let (parsed, next) = text_token(&text, start, resolve);
                token = parsed;
                index = next;
            }
        } else {
            let (parsed, next) = text_token(&text, start, resolve);
            token = parsed;
            index = next;
        }
        if token.kind == KIND_TEXT && token.text.is_empty() {
            continue;
        }
        tokens.push(token);
    }
    tokens
}

pub fn text_token2_end(text: &[char], start: usize) -> usize {
    let mut index = start + 1;
    let mut current = at(text, index);
    while current != '\0' && current != '<' {
        index += 1;
        if current == '>' {
            break;
        }
        current = at(text, index);
    }
    index
}

pub fn analyzer_on_paint() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AnalyzerState {
    pub failed: bool,
}

impl AnalyzerState {
    pub fn skip_space(&mut self) {
        self.failed = false;
    }

    pub fn next_token(&mut self) -> Option<Token> {
        self.failed = false;
        Some(Token::default())
    }
}

#[cfg(test)]
mod analyzer_tests {
    use super::*;

    fn chars(text: &str) -> Vec<char> {
        text.chars().collect()
    }

    #[test]
    fn nhtmlanalyzer_10044950_scans_to_the_next_open_angle_or_just_past_a_close_angle() {
        let text = chars("ab<c>d");
        assert_eq!(text_token2_end(&text, 0), 2);
        let text = chars("ab>cd");
        assert_eq!(text_token2_end(&text, 0), 3);
        let text = chars("abc");
        assert_eq!(text_token2_end(&text, 0), 3);
        let text = chars("<x");
        assert_eq!(text_token2_end(&text, 0), 2);
    }

    #[test]
    fn nhtmlanalyzer_10037e60_10043b60_10046930_paint_nothing_clear_the_flag_and_make_a_blank_token() {
        assert!(analyzer_on_paint());
        let mut state = AnalyzerState { failed: true };
        state.skip_space();
        assert!(!state.failed);
        state.failed = true;
        assert_eq!(state.next_token(), Some(Token::default()));
        assert!(!state.failed);
    }
}
