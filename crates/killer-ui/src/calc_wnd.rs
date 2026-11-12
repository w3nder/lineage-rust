#[path = "calc_controls_generated.rs"]
mod controls;
#[path = "calc_layout_generated.rs"]
mod layout;

use crate::button::TextButton;
use crate::calc::BUTTON_F0S;
use crate::draw::DrawList;
use crate::edit_box::{EditBox, FontMetrics, KEY_BACKSPACE};
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::frame_contract::FrameHit;
use crate::window::{Rect, WindowId};

pub const ID: WindowId = WindowId::CALC;
pub const SIZE: (i32, i32) = layout::SIZE;
pub const BACK: &str = crate::calc::CALCULATE2_BACK;
pub const BACK_ORIGIN: (i32, i32) = (0, 20);
pub const SHOW_ARGUMENT: u32 = 0x1129;
pub const OVERFLOW_LIMIT: f64 = 1.0000000200408773e+20;
pub const INITIAL_POSITION: crate::initial_position::InitialPosition =
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 2, vertical_mode: 2, x: SIZE.0, y: SIZE.1 };

pub const NO_OPERATOR: i32 = -1;
pub const ADD: i32 = 0;
pub const SUB: i32 = 1;
pub const MUL: i32 = 2;
pub const DIV: i32 = 3;

pub const STATE_FIRST: i32 = 0;
pub const STATE_FRESH: i32 = 1;
pub const STATE_APPLIED: i32 = 2;
pub const STATE_SECOND: i32 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Clear,
    ClearEntry,
    Backspace,
    Zeros,
}

pub const KEYS: [Key; 19] = [
    Key::Char('7'),
    Key::Char('8'),
    Key::Char('9'),
    Key::Char('+'),
    Key::ClearEntry,
    Key::Char('4'),
    Key::Char('5'),
    Key::Char('6'),
    Key::Char('-'),
    Key::Clear,
    Key::Char('1'),
    Key::Char('2'),
    Key::Char('3'),
    Key::Char('*'),
    Key::Backspace,
    Key::Char('0'),
    Key::Zeros,
    Key::Char('/'),
    Key::Char('='),
];

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    ((viewport.0 - SIZE.0) / 2, (viewport.1 - SIZE.1) / 2)
}

pub fn grip() -> Rect {
    layout::RECT_7C
}

pub fn atof(text: &str) -> f64 {
    let mut chars = text.chars().peekable();
    let negative = chars.peek() == Some(&'-');
    if negative || chars.peek() == Some(&'+') {
        chars.next();
    }
    let mut value = 0.0_f64;
    for ch in chars {
        let Some(digit) = ch.to_digit(10) else { break };
        value = value * 10.0 + f64::from(digit);
    }
    if negative { -value } else { value }
}

pub const CRT_MANTISSA_DIGITS: usize = 17;

pub fn format_integer(value: f64) -> String {
    if !value.is_finite() {
        return format!("{value:.0}");
    }
    let sign = if value.is_sign_negative() { "-" } else { "" };
    if value == 0.0 {
        return format!("{sign}0");
    }
    let scientific = format!("{:.*e}", CRT_MANTISSA_DIGITS - 1, value.abs());
    let (mantissa, exponent) = scientific.split_once('e').unwrap_or((&scientific, "0"));
    let mantissa: Vec<u8> = mantissa.bytes().filter(u8::is_ascii_digit).collect();
    let decimal_point = exponent.parse::<i32>().unwrap_or(0) + 1;
    format!("{sign}{}", fptostr_integer(&mantissa, decimal_point))
}

fn fptostr_integer(mantissa: &[u8], decimal_point: i32) -> String {
    if decimal_point < 0 {
        return "0".to_owned();
    }
    let count = decimal_point as usize;
    let mut buffer = vec![b'0'];
    buffer.extend((0..count).map(|index| mantissa.get(index).copied().unwrap_or(b'0')));
    if mantissa.get(count).is_some_and(|&next| next >= b'5') {
        let mut index = buffer.len() - 1;
        while buffer[index] == b'9' {
            buffer[index] = b'0';
            index -= 1;
        }
        buffer[index] += 1;
    }
    if buffer[0] == b'1' {
        return String::from_utf8_lossy(&buffer).into_owned();
    }
    let text = String::from_utf8_lossy(&buffer[1..]).into_owned();
    if text.is_empty() { "0".to_owned() } else { text }
}

pub fn compute(a: f64, b: f64, op: i32) -> f64 {
    match op {
        NO_OPERATOR => b,
        ADD => a + b,
        SUB => a - b,
        MUL => a * b,
        DIV if b != 0.0 => a / b,
        DIV => 0.0,
        _ => 0.0,
    }
}

pub fn overflows(a: f64, b: f64, op: i32, result: f64) -> bool {
    matches!(op, ADD | MUL) && a > 0.0 && b > 0.0 && !(0.0..=OVERFLOW_LIMIT).contains(&result)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalcHit {
    Close,
    Key(usize),
    Edit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalcAction {
    Close,
}

#[derive(Clone, Debug)]
pub struct CalcView {
    pub frame: FrameCtrl,
    pub edit: EditBox,
    pub close: TextButton,
    pub acc: f64,
    pub op: i32,
    pub state: i32,
    pressed: Option<CalcHit>,
}

impl CalcView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        let mut edit = EditBox::from_contract(&controls::CHILD_EC);
        edit.set_text("0");
        Self {
            frame: controls::FRAME_7C.build(text),
            edit,
            close: controls::BUTTON_13C.build((0, 0), text),
            acc: 0.0,
            op: NO_OPERATOR,
            state: STATE_FRESH,
            pressed: None,
        }
    }

    fn text(&self) -> Option<String> {
        self.edit.get_text()
    }

    fn value(&self) -> f64 {
        self.text().map_or(0.0, |text| atof(&text))
    }

    fn set_acc_from_text(&mut self) {
        self.acc = self.text().map_or(0.0, |text| atof(&text));
    }

    fn set_display(&mut self, value: f64, font: &BitmapFont) {
        self.edit.clear();
        self.edit.insert_text(&format_integer(value), true, &FontMetrics(font));
    }

    pub fn reset(&mut self, font: &BitmapFont) {
        self.acc = 0.0;
        self.op = NO_OPERATOR;
        self.state = STATE_FRESH;
        self.set_display(0.0, font);
    }

    pub fn open(&mut self, font: &BitmapFont) {
        self.reset(font);
        self.edit.focus();
    }

    pub fn close(&mut self, font: &BitmapFont) {
        self.reset(font);
        self.edit.kill_focus();
        self.pressed = None;
    }

    pub fn clear_entry(&mut self, font: &BitmapFont) {
        self.edit.clear();
        self.edit.insert_text("0", true, &FontMetrics(font));
    }

    pub fn char_input(&mut self, key: char, font: &BitmapFont) {
        if key != '\u{8}' && !self.edit.accepts {
            return;
        }
        let key = if key == '\r' { '=' } else { key };
        match key {
            '0'..='9' => self.digit(key, font),
            '*' | '+' | '-' | '/' | '=' => self.operator(key, font),
            _ => {}
        }
    }

    fn digit(&mut self, key: char, font: &BitmapFont) {
        if matches!(self.state, STATE_FIRST | STATE_SECOND) && self.text().as_deref() == Some("0") {
            if key == '0' {
                return;
            }
            self.edit.clear();
        }
        match self.state {
            STATE_FRESH => {
                self.edit.clear();
                self.state = STATE_FIRST;
            }
            STATE_APPLIED => {
                self.set_acc_from_text();
                self.edit.clear();
                self.state = STATE_SECOND;
            }
            _ => {}
        }
        self.edit.insert_text(&key.to_string(), true, &FontMetrics(font));
    }

    fn operator(&mut self, key: char, font: &BitmapFont) {
        match self.state {
            STATE_FRESH => self.state = STATE_APPLIED,
            STATE_FIRST => {
                self.set_acc_from_text();
                self.state = STATE_APPLIED;
            }
            STATE_SECOND => {
                let value = self.value();
                let result = compute(self.acc, value, self.op);
                if overflows(self.acc, value, self.op, result) {
                    self.reset(font);
                } else {
                    self.set_acc_from_text();
                    self.set_display(result, font);
                    self.state = STATE_APPLIED;
                }
            }
            _ => {
                if key == '=' {
                    let display = self.value();
                    let result = compute(display, self.acc, self.op);
                    if overflows(display, self.acc, self.op, result) {
                        self.reset(font);
                        return;
                    }
                    self.set_display(result, font);
                    return;
                }
            }
        }
        match key {
            '*' => self.op = MUL,
            '+' => self.op = ADD,
            '-' => self.op = SUB,
            '/' => self.op = DIV,
            _ => {}
        }
    }

    pub fn press(&mut self, index: usize, font: &BitmapFont) {
        match KEYS.get(index).copied() {
            Some(Key::Char(ch)) => self.char_input(ch, font),
            Some(Key::Zeros) => {
                self.char_input('0', font);
                self.char_input('0', font);
            }
            Some(Key::ClearEntry) => self.clear_entry(font),
            Some(Key::Clear) => self.reset(font),
            Some(Key::Backspace) => {
                self.edit.key_down(KEY_BACKSPACE, &FontMetrics(font));
            }
            None => {}
        }
        self.edit.focus();
    }

    pub fn key(&mut self, key: crate::chat::InputKey, font: &BitmapFont) -> bool {
        use crate::chat::InputKey;
        if !self.edit.focused {
            return false;
        }
        match key {
            InputKey::Char(ch) => self.char_input(ch, font),
            InputKey::Enter => self.char_input('\r', font),
            InputKey::Escape => return false,
            other => {
                self.edit.input_key(other, &FontMetrics(font));
                self.edit.notices.clear();
            }
        }
        true
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<CalcHit> {
        if controls::FRAME_7C.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close) {
            return Some(CalcHit::Close);
        }
        if controls::BUTTON_13C.hit(&self.close, x, y).is_some() {
            return Some(CalcHit::Close);
        }
        if let Some(index) = BUTTON_F0S.iter().position(|(rect, _, _)| rect.contains(x, y)) {
            return Some(CalcHit::Key(index));
        }
        layout::RECT_EC.contains(x, y).then_some(CalcHit::Edit)
    }

    pub fn mouse_down(&mut self, x: i32, y: i32, font: &BitmapFont) -> Option<CalcHit> {
        let hit = self.hit(x, y);
        self.pressed = hit;
        if hit == Some(CalcHit::Edit) {
            let metrics = FontMetrics(font);
            self.edit.focus();
            self.edit.mouse_down(x, &metrics);
            self.edit.layout(&metrics);
        }
        hit
    }

    pub fn mouse_up(&mut self, x: i32, y: i32, font: &BitmapFont) -> Option<CalcAction> {
        self.edit.mouse_up();
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        match pressed {
            CalcHit::Close => {
                self.close(font);
                Some(CalcAction::Close)
            }
            CalcHit::Key(index) => {
                self.press(index, font);
                None
            }
            CalcHit::Edit => None,
        }
    }

    pub fn tick(&mut self, elapsed_ms: u32) {
        self.edit.tick(elapsed_ms);
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) {
        let (x, y) = origin;
        controls::FRAME_7C.paint(&self.frame, list, origin, font, alpha);
        let start = list.quads.len();
        list.draw_texture(x + BACK_ORIGIN.0, y + BACK_ORIGIN.1, SIZE.0, SIZE.1 - BACK_ORIGIN.1, 0, 0, SIZE.0, SIZE.1 - BACK_ORIGIN.1, BACK, alpha);
        list.clip_from(start, Rect::new(x, y, SIZE.0, SIZE.1));
        self.edit.paint(list, origin, &FontMetrics(font), font);
        for (index, (rect, _, textures)) in BUTTON_F0S.iter().enumerate() {
            let down = self.pressed == Some(CalcHit::Key(index));
            let texture = if down { textures[2] } else { textures[1] };
            list.draw_texture(x + rect.x, y + rect.y, rect.w, rect.h, 0, 0, rect.w, rect.h, texture, 1.0);
        }
        controls::BUTTON_13C.paint(&self.close, list, origin, self.pressed == Some(CalcHit::Close), font);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), crate::font::Glyphs::parse(&crate::font::gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn view() -> (CalcView, BitmapFont) {
        (CalcView::new(|id| Some(format!("s{id}"))), font())
    }

    fn typed(view: &mut CalcView, font: &BitmapFont, text: &str) {
        for ch in text.chars() {
            view.char_input(ch, font);
        }
    }

    fn shown(view: &CalcView) -> String {
        view.edit.get_text().unwrap_or_default()
    }

    #[test]
    fn the_layout_matches_the_on_create_snapshot() {
        let (view, _) = view();
        assert_eq!(SIZE, (256, 211));
        assert_eq!(grip(), Rect::new(0, 0, 256, 20));
        assert_eq!(view.edit.rect, Rect::new(34, 40, 188, 17));
        assert_eq!(view.close.rect, Rect::new(91, 182, 76, 23));
        assert_eq!(view.close.label, "s646");
        assert_eq!(view.frame.title, "s669");
        assert_eq!(default_position((1280, 800)), (512, 294));
        assert_eq!(BUTTON_F0S.len(), KEYS.len());
    }

    #[test]
    fn the_calculator_starts_fresh_showing_zero() {
        let (view, _) = view();
        assert_eq!((shown(&view), view.state, view.op), ("0".to_string(), STATE_FRESH, NO_OPERATOR));
    }

    #[test]
    fn typing_a_digit_replaces_the_zero_and_a_second_zero_is_ignored() {
        let (mut view, font) = view();
        typed(&mut view, &font, "0");
        assert_eq!(shown(&view), "0");
        typed(&mut view, &font, "05");
        assert_eq!(shown(&view), "5");
        assert_eq!(view.state, STATE_FIRST);
    }

    #[test]
    fn five_plus_three_equals_eight_and_equals_again_adds_three() {
        let (mut view, font) = view();
        typed(&mut view, &font, "5+3=");
        assert_eq!(shown(&view), "8");
        assert_eq!(view.state, STATE_APPLIED);
        assert_eq!(view.acc, 3.0);
        typed(&mut view, &font, "=");
        assert_eq!(shown(&view), "11");
        assert_eq!(view.state, STATE_APPLIED);
    }

    #[test]
    fn chained_operators_apply_the_pending_one() {
        let (mut view, font) = view();
        typed(&mut view, &font, "6*7+");
        assert_eq!(shown(&view), "42");
        typed(&mut view, &font, "8=");
        assert_eq!(shown(&view), "50");
    }

    #[test]
    fn division_by_zero_gives_zero_and_a_quotient_is_rounded_for_display() {
        let (mut view, font) = view();
        typed(&mut view, &font, "9/0=");
        assert_eq!(shown(&view), "0");
        view.reset(&font);
        typed(&mut view, &font, "8/3=");
        assert_eq!(shown(&view), "3");
    }

    #[test]
    fn display_follows_fptostr_of_msvcr70_with_seventeen_mantissa_digits_and_half_up_on_the_text() {
        let table: [(f64, &str); 14] = [
            (0.0, "0"),
            (-0.0, "-0"),
            (0.4, "0"),
            (0.5, "1"),
            (-0.4, "-0"),
            (1.5, "2"),
            (2.5, "3"),
            (0.049, "0"),
            (999.5, "1000"),
            (0.499_999_999_999_999_94, "0"),
            (9_223_372_036_854_775_808.0, "9223372036854775800"),
            (123_456_789_012_345_678.0, "123456789012345680"),
            (1e20, "100000000000000000000"),
            (4_503_599_627_370_497.5, "4503599627370498"),
        ];
        for (value, expected) in table {
            assert_eq!(format_integer(value), expected, "{value:e}");
        }
    }

    #[test]
    fn subtraction_can_go_negative_and_the_separator_counts_the_sign() {
        let (mut view, font) = view();
        typed(&mut view, &font, "1-2000=");
        assert_eq!(shown(&view), "-1999");
        assert_eq!(view.edit.left().iter().map(|&unit| char::from_u32(u32::from(unit)).unwrap()).collect::<String>(), "-1,999");
    }

    #[test]
    fn overflow_resets_the_calculator_and_shows_zero() {
        let (mut view, font) = view();
        typed(&mut view, &font, "99999999999*99999999999=");
        assert_eq!(shown(&view), "0");
        assert_eq!((view.state, view.op, view.acc), (STATE_FRESH, NO_OPERATOR, 0.0));
    }

    #[test]
    fn enter_acts_as_equals_and_letters_are_ignored() {
        let (mut view, font) = view();
        typed(&mut view, &font, "2+2a.,");
        view.char_input('\r', &font);
        assert_eq!(shown(&view), "4");
    }

    #[test]
    fn the_buttons_send_the_same_keys_as_the_binary_dispatch() {
        let (mut view, font) = view();
        for index in [0, 3, 1, 18] {
            view.press(index, &font);
        }
        assert_eq!(shown(&view), "15");
        for index in [10, 15, 16] {
            view.press(index, &font);
        }
        assert_eq!(shown(&view), "1000");
        view.press(14, &font);
        assert_eq!(shown(&view), "100");
        view.press(4, &font);
        assert_eq!(shown(&view), "0");
        view.press(9, &font);
        assert_eq!((view.state, view.op), (STATE_FRESH, NO_OPERATOR));
        assert!(view.edit.focused);
    }

    #[test]
    fn the_close_button_resets_and_reports_close() {
        let (mut view, font) = view();
        typed(&mut view, &font, "12");
        let rect = view.close.rect;
        view.mouse_down(rect.x + 2, rect.y + 2, &font);
        assert_eq!(view.mouse_up(rect.x + 2, rect.y + 2, &font), Some(CalcAction::Close));
        assert_eq!((shown(&view), view.state), ("0".to_string(), STATE_FRESH));
    }

    #[test]
    fn atof_reads_the_leading_integer_like_the_crt() {
        assert_eq!(atof("-12"), -12.0);
        assert_eq!(atof("-"), 0.0);
        assert_eq!(atof("45x"), 45.0);
    }

    #[test]
    fn on_char_1011f620_ignores_everything_but_backspace_while_the_display_cannot_take_more() {
        let (mut view, font) = view();
        typed(&mut view, &font, "5");
        view.edit.accepts = false;
        view.char_input('7', &font);
        view.char_input('+', &font);
        assert_eq!(shown(&view), "5");
        assert_eq!(view.state, STATE_FIRST);
        view.edit.accepts = true;
        view.char_input('+', &font);
        assert_eq!(view.state, STATE_APPLIED);
    }
}
