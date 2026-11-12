use crate::button::TextButton;
use crate::chat::wrap_line;
use crate::color::TEXT;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::progress::DIALOG_TIMER;
use crate::window::Rect;

pub const DIALOG_SIZE: (i32, i32) = (256, 108);
pub const DIALOG_BOTTOM_X: i32 = 347;
pub const DIALOG_BOTTOM_RISE: i32 = 110;
pub const SYSTEM_BACK: &str = "L2UI_ch3.dialog.system_back";
pub const WARNING_ICON: &str = "L2UI_ch3.dialog.warningicon";
pub const CALCULATE_BACK: &str = "L2UI_ch3.Calculate.Calculate1_back";
pub const BTN1_NORMAL: &str = "L2UI_ch3.button.btn1_normal";
pub const BTN1_DOWN: &str = "L2UI_ch3.button.btn1_normalon";
pub const WARNING_ICON_POS: (i32, i32) = (11, 15);
pub const WARNING_ICON_SIZE: i32 = 24;
pub const TEXT_POS: (i32, i32) = (41, 16);
pub const TEXT_WIDTH: i32 = 200;
pub const EDIT_RECT: Rect = Rect::new(50, 48, 156, 17);
pub const TIMER_RECT: Rect = Rect::new(17, 61, 222, 6);
pub const TIMER_TOTAL_MS: i32 = 10000;
pub const BUTTON_SIZE: (i32, i32) = (76, 23);
pub const BUTTON_Y: i32 = 75;
pub const FIRST_BUTTON_X: i32 = 51;
pub const SECOND_BUTTON_X: i32 = 131;
pub const OK_SYSSTRING: u32 = 140;
pub const CANCEL_SYSSTRING: u32 = 141;
pub const YES_SYSSTRING: u32 = 184;
pub const NO_SYSSTRING: u32 = 185;
pub const ENTER_ACCEPTS_TYPES: [i32; 3] = [0x1d, 0x1f, 0x10];
pub const SINGLE_BUTTON_TYPES: [i32; 4] = [0x1d, 0x1f, 0x10, 0x50];
pub const HANDLED_TYPES: [i32; 63] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x08, 0x0a, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x14, 0x18, 0x1a, 0x1e, 0x1f, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x32,
    0x33, 0x34, 0x35, 0x36, 0x37, 0x39, 0x3c, 0x3d, 0x3e, 0x3f, 0x40, 0x41, 0x42, 0x43, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4d, 0x51, 0x52, 0x53, 0x54,
    0x55, 0x56, 0x57,
];
pub const KEYPAD_X: i32 = 256;
pub const KEYPAD_WIDTH: i32 = 108;
pub const KEYPAD_ORIGIN: (i32, i32) = (7, 7);
pub const KEYPAD_STEP: i32 = 25;
pub const KEYPAD_KEY: i32 = 23;
pub const KEYPAD_LONG: i32 = 47;
pub const KEYPAD_KEYS: usize = 13;
pub const KEY_ZERO: usize = 9;
pub const KEY_BACKSPACE: usize = 10;
pub const KEY_ALL: usize = 11;
pub const KEY_CLEAR: usize = 12;
pub const KEYPAD_TEXTURES: [(&str, &str); KEYPAD_KEYS] = [
    ("L2UI_CH3.Calculate.Calculate1_1", "L2UI_CH3.Calculate.Calculate1_1_down"),
    ("L2UI_CH3.Calculate.Calculate1_2", "L2UI_CH3.Calculate.Calculate1_2_down"),
    ("L2UI_CH3.Calculate.Calculate1_3", "L2UI_CH3.Calculate.Calculate1_3_down"),
    ("L2UI_CH3.Calculate.Calculate1_4", "L2UI_CH3.Calculate.Calculate1_4_down"),
    ("L2UI_CH3.Calculate.Calculate1_5", "L2UI_CH3.Calculate.Calculate1_5_down"),
    ("L2UI_CH3.Calculate.Calculate1_6", "L2UI_CH3.Calculate.Calculate1_6_down"),
    ("L2UI_CH3.Calculate.Calculate1_7", "L2UI_CH3.Calculate.Calculate1_7_down"),
    ("L2UI_CH3.Calculate.Calculate1_8", "L2UI_CH3.Calculate.Calculate1_8_down"),
    ("L2UI_CH3.Calculate.Calculate1_9", "L2UI_CH3.Calculate.Calculate1_9_down"),
    ("L2UI_CH3.Calculate.Calculate1_0", "L2UI_CH3.Calculate.Calculate1_0_down"),
    ("L2UI_CH3.Calculate.Calculate1_bs", "L2UI_CH3.Calculate.Calculate1_bs_down"),
    ("L2UI_CH3.Calculate.Calculate1_all", "L2UI_CH3.Calculate.Calculate1_all_down"),
    ("L2UI_CH3.Calculate.Calculate1_c", "L2UI_CH3.Calculate.Calculate1_c_down"),
];
pub const EDIT_TEXT_INSET: (i32, i32) = (2, 2);
pub const COUNT_LIMIT: i64 = 2_000_000_000;
pub const COUNT_LIMIT_LOW: i64 = 1_000_000;
pub const EDIT_MAX_CHARS: usize = 150;
pub const CURSOR_TEXT: &str = "|";

pub fn keypad_rect(key: usize) -> Rect {
    let (ox, oy) = KEYPAD_ORIGIN;
    match key {
        0..=8 => Rect::new(ox + (key as i32 % 3) * KEYPAD_STEP, oy + (key as i32 / 3) * KEYPAD_STEP, KEYPAD_KEY, KEYPAD_KEY),
        KEY_ZERO => Rect::new(ox, oy + 3 * KEYPAD_STEP, KEYPAD_KEY, KEYPAD_KEY),
        KEY_BACKSPACE => Rect::new(ox + 3 * KEYPAD_STEP, oy, KEYPAD_KEY, KEYPAD_LONG),
        KEY_ALL => Rect::new(ox + KEYPAD_STEP, oy + 3 * KEYPAD_STEP, KEYPAD_LONG, KEYPAD_KEY),
        _ => Rect::new(ox + 3 * KEYPAD_STEP, oy + 2 * KEYPAD_STEP, KEYPAD_KEY, KEYPAD_LONG),
    }
    .offset(KEYPAD_X, 0)
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CountInput {
    pub text: String,
    pub max: i64,
    pub free_text: bool,
}

impl CountInput {
    pub fn new(max: i64) -> CountInput {
        CountInput { text: String::new(), max, free_text: false }
    }

    pub fn text_mode() -> CountInput {
        CountInput { text: String::new(), max: 0, free_text: true }
    }

    pub fn key(&mut self, key: usize) {
        match key {
            0..=8 => self.push((b'1' + key as u8) as char),
            KEY_ZERO => self.push('0'),
            KEY_BACKSPACE => {
                self.text.pop();
            }
            KEY_ALL => self.text = self.max.to_string(),
            _ => self.text.clear(),
        }
    }

    pub fn push(&mut self, c: char) {
        let accepted = if self.free_text { !c.is_control() } else { c.is_ascii_digit() };
        if accepted && self.text.chars().count() < EDIT_MAX_CHARS {
            self.text.push(c);
        }
    }

    pub fn value(&self, high_limit: bool) -> Option<i64> {
        let value: i64 = self.text.parse().ok().filter(|v| *v >= 0)?;
        let limit = if high_limit { COUNT_LIMIT } else { COUNT_LIMIT_LOW };
        (value <= limit).then_some(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogButton {
    First,
    Second,
    Key(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogKind {
    OkCancel,
    YesNo,
}

impl DialogKind {
    pub fn sysstrings(self) -> (u32, u32) {
        match self {
            DialogKind::OkCancel => (OK_SYSSTRING, CANCEL_SYSSTRING),
            DialogKind::YesNo => (YES_SYSSTRING, NO_SYSSTRING),
        }
    }

    pub fn default_labels(self) -> (&'static str, &'static str) {
        match self {
            DialogKind::OkCancel => ("OK", "Cancel"),
            DialogKind::YesNo => ("Yes", "No"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogView {
    pub kind: DialogKind,
    pub message: String,
    pub labels: (String, String),
    pub timer_elapsed_ms: Option<i32>,
    pub dialog_type: i32,
    pub input: Option<CountInput>,
}

impl DialogView {
    pub fn new(kind: DialogKind, message: &str) -> DialogView {
        let (first, second) = kind.default_labels();
        DialogView {
            kind,
            message: message.to_string(),
            labels: (first.into(), second.into()),
            timer_elapsed_ms: None,
            dialog_type: 0,
            input: None,
        }
    }

    pub fn with_count(kind: DialogKind, message: &str, max: i64) -> DialogView {
        DialogView { input: Some(CountInput::new(max)), ..DialogView::new(kind, message) }
    }

    pub fn with_text(kind: DialogKind, message: &str) -> DialogView {
        DialogView { input: Some(CountInput::text_mode()), ..DialogView::new(kind, message) }
    }

    pub fn keypad(&self) -> bool {
        self.input.as_ref().is_some_and(|input| !input.free_text)
    }

    pub fn width(&self) -> i32 {
        if self.keypad() { DIALOG_SIZE.0 + KEYPAD_WIDTH } else { DIALOG_SIZE.0 }
    }

    pub fn single_button(&self) -> bool {
        SINGLE_BUTTON_TYPES.contains(&self.dialog_type)
    }

    pub fn buttons(&self) -> [TextButton; 2] {
        let (w, h) = BUTTON_SIZE;
        let first_x = if self.single_button() { (self.width() as f32 * 0.5 - 38.0) as i32 } else { FIRST_BUTTON_X };
        [
            TextButton::new(Rect::new(first_x, BUTTON_Y, w, h), BTN1_NORMAL, BTN1_DOWN, &self.labels.0),
            TextButton::new(Rect::new(SECOND_BUTTON_X, BUTTON_Y, w, h), BTN1_NORMAL, BTN1_DOWN, &self.labels.1),
        ]
    }

    pub fn button_at(&self, x: i32, y: i32) -> Option<DialogButton> {
        let [first, second] = self.buttons();
        if first.contains(x, y) {
            Some(DialogButton::First)
        } else if second.contains(x, y) && !self.single_button() {
            Some(DialogButton::Second)
        } else if self.keypad() {
            (0..KEYPAD_KEYS).find(|key| keypad_rect(*key).contains(x, y)).map(DialogButton::Key)
        } else {
            None
        }
    }

    pub fn enter_button(&self) -> DialogButton {
        if self.input.is_some() || ENTER_ACCEPTS_TYPES.contains(&self.dialog_type) { DialogButton::First } else { DialogButton::Second }
    }

    pub fn paint_message(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont) {
        let message = self.message.replace("\\n", "\r\n");
        let lines = crate::text_align::bitmap_lines(&message, TEXT_WIDTH, font, 1, false);
        crate::text_align::paint(list, origin, &lines, TEXT, font, true);
    }

    pub fn lines(&self, font: &BitmapFont) -> Vec<String> {
        self.message
            .split('\n')
            .flat_map(|part| wrap_line(part, TEXT_WIDTH, 0, font).into_iter().map(|(_, l)| l))
            .collect()
    }
}

pub fn default_position(id_bottom: bool, viewport: (i32, i32)) -> (i32, i32) {
    let (w, h) = DIALOG_SIZE;
    if id_bottom {
        (DIALOG_BOTTOM_X, crate::window_restore::x87_to_int(f64::from(viewport.1 as f32) - f64::from(h) - 2.0))
    } else {
        (
            crate::window_restore::x87_to_int(f64::from(viewport.0 as f32) * 0.5 - f64::from(w / 2)),
            crate::window_restore::x87_to_int(f64::from(viewport.1 as f32) * 0.5 - f64::from(h / 2)),
        )
    }
}

pub fn warning_position(viewport: (i32, i32)) -> (i32, i32) {
    (
        crate::window_restore::x87_to_int(f64::from(viewport.0 as f32) * 0.5 - 128.0),
        crate::window_restore::x87_to_int(f64::from(viewport.1 as f32) * 0.5 - 54.0),
    )
}

pub fn initial_position(id_bottom: bool, viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    if id_bottom {
        let (x, y) = default_position(true, viewport);
        crate::initial_position::InitialPosition {
            enabled: true, horizontal_mode: 0, vertical_mode: 4,
            x, y: viewport.1.wrapping_sub(y),
        }
    } else {
        crate::initial_position::InitialPosition {
            enabled: true, horizontal_mode: 2, vertical_mode: 2,
            x: DIALOG_SIZE.0, y: DIALOG_SIZE.1,
        }
    }
}

pub fn paint_dialog(
    list: &mut DrawList,
    x: i32,
    y: i32,
    view: &DialogView,
    pressed: Option<DialogButton>,
    font: &BitmapFont,
) {
    let (w, h) = DIALOG_SIZE;
    list.draw_texture(x, y, w, h, 0, 0, w, h, SYSTEM_BACK, 1.0);
    let (ix, iy) = WARNING_ICON_POS;
    list.draw_texture(
        x + ix,
        y + iy,
        WARNING_ICON_SIZE,
        WARNING_ICON_SIZE,
        0,
        0,
        WARNING_ICON_SIZE,
        WARNING_ICON_SIZE,
        WARNING_ICON,
        1.0,
    );
    let (tx, ty) = TEXT_POS;
    view.paint_message(list, (x + tx, y + ty), font);
    if let Some(elapsed) = view.timer_elapsed_ms {
        let Rect { x: rx, y: ry, w: rw, h: rh } = TIMER_RECT;
        DIALOG_TIMER.paint(list, x + rx, y + ry, rw, rh, elapsed.min(TIMER_TOTAL_MS), TIMER_TOTAL_MS);
    }
    let [first, second] = view.buttons();
    first.paint(list, x, y, pressed == Some(DialogButton::First), font);
    if !view.single_button() {
        second.paint(list, x, y, pressed == Some(DialogButton::Second), font);
    }
    if let Some(input) = &view.input {
        if view.keypad() {
            list.draw_texture(x + KEYPAD_X, y, KEYPAD_WIDTH, h, 0, 0, KEYPAD_WIDTH, h, CALCULATE_BACK, 1.0);
            for (key, &(normal, down)) in KEYPAD_TEXTURES.iter().enumerate().take(KEYPAD_KEYS) {
                let rect = keypad_rect(key);
                let texture = if pressed == Some(DialogButton::Key(key)) { down } else { normal };
                list.draw_texture(x + rect.x, y + rect.y, rect.w, rect.h, 0, 0, rect.w, rect.h, texture, 1.0);
            }
        }
        let (ex, ey) = (x + EDIT_RECT.x + EDIT_TEXT_INSET.0, y + EDIT_RECT.y + EDIT_TEXT_INSET.1);
        let end = list.draw_text(ex, ey, TEXT, &input.text, font);
        list.draw_text(end - 1, ey, TEXT, CURSOR_TEXT, font);
    }
}

pub const WARNING_OK_Y: i32 = 75;
pub const WARNING_OK_CENTER_INSET: i32 = 38;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningOutcome {
    Exit,
    Hide,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WarningDialog {
    pub message: String,
    pub exit: bool,
    pub shown: bool,
}

impl WarningDialog {
    pub fn show(&mut self, text: Option<&str>, exit: bool) {
        if exit {
            self.exit = true;
        }
        if let Some(text) = text {
            self.message = text.to_string();
            self.shown = true;
        }
    }

    pub fn ok(&mut self) -> WarningOutcome {
        if self.exit {
            return WarningOutcome::Exit;
        }
        self.shown = false;
        self.exit = false;
        WarningOutcome::Hide
    }

    pub fn ok_button(label: &str) -> TextButton {
        let (w, h) = BUTTON_SIZE;
        let x = (DIALOG_SIZE.0 as f32 * 0.5 - WARNING_OK_CENTER_INSET as f32) as i32;
        TextButton::new(Rect::new(x, WARNING_OK_Y, w, h), BTN1_NORMAL, BTN1_DOWN, label)
    }

    pub fn hit(x: i32, y: i32) -> bool {
        Self::ok_button("").contains(x, y)
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, label: &str, pressed: bool, font: &BitmapFont) {
        let view = DialogView::new(DialogKind::OkCancel, &self.message);
        let (w, h) = DIALOG_SIZE;
        list.draw_texture(x, y, w, h, 0, 0, w, h, SYSTEM_BACK, 1.0);
        let (ix, iy) = WARNING_ICON_POS;
        list.draw_texture(x + ix, y + iy, WARNING_ICON_SIZE, WARNING_ICON_SIZE, 0, 0, WARNING_ICON_SIZE, WARNING_ICON_SIZE, WARNING_ICON, 1.0);
        let (tx, ty) = TEXT_POS;
        view.paint_message(list, (x + tx, y + ty), font);
        Self::ok_button(label).paint(list, x, y, pressed, font);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::Quad;
    use crate::font::{Glyphs, gly};
    use crate::progress::{MINIBAR_BACK, MINIBAR_TIME};

#[test]
    fn the_text_mode_edit_takes_any_printable_char_and_has_no_keypad() {
        let mut view = DialogView::with_text(DialogKind::OkCancel, "name?");
        assert!(!view.keypad());
        assert_eq!(view.width(), DIALOG_SIZE.0);
        let input = view.input.as_mut().unwrap();
        for c in "Ab 1\u{7}".chars() { input.push(c); }
        assert_eq!(input.text, "Ab 1");
        assert_eq!(view.button_at(KEYPAD_X + 10, 10), None);
    }

        const TEXTURES: &str = include_str!("../tests/fixtures/nwindow-texturas-dialog.txt");

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, if i == 0 { 3 } else { 6 }, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn with_texture<'a>(quads: &'a [Quad], texture: &str) -> Vec<&'a Quad> {
        quads.iter().filter(|q| q.texture == texture).collect()
    }

    fn painted(view: &DialogView, pressed: Option<DialogButton>) -> Vec<Quad> {
        let mut list = DrawList::default();
        paint_dialog(&mut list, 0, 0, view, pressed, &font());
        list.quads
    }

    #[test]
    fn the_box_is_256_by_108_with_the_warning_icon_and_two_btn1_buttons() {
        let view = DialogView::new(DialogKind::YesNo, "Killer invites you");
        let quads = painted(&view, None);
        let back = with_texture(&quads, SYSTEM_BACK);
        assert_eq!(
            (back[0].x, back[0].y, back[0].w, back[0].h, back[0].uw, back[0].vh),
            (0.0, 0.0, 256.0, 108.0, 256.0, 108.0)
        );
        let icon = with_texture(&quads, WARNING_ICON);
        assert_eq!((icon[0].x, icon[0].y, icon[0].w, icon[0].h), (11.0, 15.0, 24.0, 24.0));
        let buttons = with_texture(&quads, BTN1_NORMAL);
        assert_eq!(buttons.len(), 2);
        assert_eq!((buttons[0].x, buttons[0].y, buttons[0].w, buttons[0].h), (51.0, 75.0, 76.0, 23.0));
        assert_eq!((buttons[1].x, buttons[1].y), (131.0, 75.0));
        assert!(with_texture(&quads, MINIBAR_BACK).is_empty());
        let text: Vec<&Quad> = quads.iter().filter(|q| q.texture == "f" && q.y == 16.0).collect();
        assert_eq!((text[0].x, text[0].color), (41.0, TEXT.rgba()));
        let pressed = painted(&view, Some(DialogButton::Second));
        assert_eq!(with_texture(&pressed, BTN1_DOWN)[0].x, 131.0);
        assert_eq!(with_texture(&pressed, BTN1_NORMAL).len(), 1);
    }

    #[test]
    fn long_messages_wrap_at_200_pixels_with_the_html_line_step() {
        let font = font();
        let view = DialogView::new(DialogKind::OkCancel, &"word ".repeat(12));
        let lines = view.lines(&font);
        assert_eq!(lines.len(), 2);
        assert!(lines.iter().all(|l| font.measure(l) as i32 <= 200));
        let quads = painted(&view, None);
        let second_line: Vec<&Quad> = quads.iter().filter(|q| q.texture == "f" && q.y == 16.0 + 19.0).collect();
        assert!(!second_line.is_empty());
        assert_eq!(DialogView::new(DialogKind::OkCancel, "a\nb").lines(&font), ["a", "b"]);
    }

    #[test]
    fn the_timer_bar_shrinks_over_ten_seconds() {
        let mut view = DialogView::new(DialogKind::YesNo, "x");
        view.timer_elapsed_ms = Some(5000);
        let quads = painted(&view, None);
        let back = with_texture(&quads, MINIBAR_BACK);
        assert_eq!((back[0].x, back[0].y, back[0].w, back[0].h), (17.0, 61.0, 222.0, 6.0));
        let fill = with_texture(&quads, MINIBAR_TIME);
        assert_eq!((fill[0].x, fill[0].y, fill[0].w, fill[0].h), (18.0, 62.0, 110.0, 4.0));
    }

    #[test]
    fn buttons_labels_and_enter_follow_the_dialog_kind_and_type() {
        let view = DialogView::new(DialogKind::YesNo, "x");
        assert_eq!(view.labels, ("Yes".to_string(), "No".to_string()));
        assert_eq!(DialogKind::OkCancel.sysstrings(), (140, 141));
        assert_eq!(DialogKind::YesNo.sysstrings(), (184, 185));
        assert_eq!(view.button_at(60, 80), Some(DialogButton::First));
        assert_eq!(view.button_at(140, 97), Some(DialogButton::Second));
        assert_eq!(view.button_at(128, 80), None);
        assert_eq!(view.button_at(60, 60), None);
        assert_eq!(view.enter_button(), DialogButton::Second);
        assert_eq!(DialogView { dialog_type: 0x1d, ..view.clone() }.enter_button(), DialogButton::First);
        assert_eq!(default_position(false, (1558, 933)), (651, 412));
        assert_eq!(default_position(true, (1558, 933)), (347, 823));
    }

    #[test]
    fn every_texture_exists_in_the_client_list() {
        let known: Vec<String> = TEXTURES.lines().map(|l| l.trim().to_lowercase()).collect();
        let mut view = DialogView::new(DialogKind::YesNo, "x");
        view.timer_elapsed_ms = Some(1);
        let quads = painted(&view, Some(DialogButton::First));
        for texture in quads.iter().map(|q| q.texture.as_str()).filter(|t| *t != "f") {
            assert!(known.contains(&texture.to_lowercase()), "textura desconhecida {texture}");
        }
        assert!(known.contains(&CALCULATE_BACK.to_lowercase()));
        for (normal, down) in KEYPAD_TEXTURES {
            assert!(known.contains(&normal.to_lowercase()) && known.contains(&down.to_lowercase()), "{normal}");
        }
    }

    #[test]
    fn the_count_dialog_adds_the_keypad_panel_right_of_the_box_and_edits_the_number() {
        let mut view = DialogView::with_count(DialogKind::OkCancel, "How many?", 250);
        assert_eq!(view.width(), 364);
        assert_eq!(keypad_rect(0), Rect::new(263, 7, 23, 23));
        assert_eq!(keypad_rect(8), Rect::new(313, 57, 23, 23));
        assert_eq!(keypad_rect(KEY_ZERO), Rect::new(263, 82, 23, 23));
        assert_eq!(keypad_rect(KEY_BACKSPACE), Rect::new(338, 7, 23, 47));
        assert_eq!(keypad_rect(KEY_ALL), Rect::new(288, 82, 47, 23));
        assert_eq!(keypad_rect(KEY_CLEAR), Rect::new(338, 57, 23, 47));
        assert_eq!(view.button_at(270, 10), Some(DialogButton::Key(0)));
        assert_eq!(view.button_at(340, 30), Some(DialogButton::Key(KEY_BACKSPACE)));
        assert_eq!(view.button_at(300, 90), Some(DialogButton::Key(KEY_ALL)));
        assert_eq!(DialogView::new(DialogKind::OkCancel, "x").button_at(270, 10), None);
        let input = view.input.as_mut().unwrap();
        input.key(0);
        input.key(KEY_ZERO);
        input.push('7');
        input.push('x');
        assert_eq!(input.text, "107");
        input.key(KEY_BACKSPACE);
        assert_eq!(input.text, "10");
        assert_eq!(input.value(true), Some(10));
        input.key(KEY_ALL);
        assert_eq!(input.text, "250");
        input.key(KEY_CLEAR);
        assert_eq!((input.text.as_str(), input.value(true)), ("", None));
        input.text = "2000000001".into();
        assert_eq!((input.value(true), input.value(false)), (None, None));
        input.text = "2000000000".into();
        assert_eq!((input.value(true), input.value(false)), (Some(2_000_000_000), None));
        input.text = "9".into();
        let quads = painted(&view, Some(DialogButton::Key(2)));
        let panel = with_texture(&quads, CALCULATE_BACK);
        assert_eq!((panel[0].x, panel[0].y, panel[0].w, panel[0].h), (256.0, 0.0, 108.0, 108.0));
        assert_eq!(with_texture(&quads, KEYPAD_TEXTURES[2].1).len(), 1);
        assert_eq!(with_texture(&quads, KEYPAD_TEXTURES[2].0).len(), 0);
        assert_eq!(with_texture(&quads, KEYPAD_TEXTURES[KEY_ALL].0)[0].w, 47.0);
        let digit: Vec<&Quad> = quads.iter().filter(|q| q.texture == "f" && q.y == 50.0).collect();
        assert_eq!(digit[0].x, 52.0);
        assert_eq!(digit.len(), 2);
    }

    #[test]
    fn the_warning_box_1000b120_10008f90_100091a0_centres_one_ok_and_exits_only_when_marked() {
        let mut warning = WarningDialog::default();
        warning.show(Some("fatal"), false);
        assert!(warning.shown);
        let mut list = DrawList::default();
        warning.paint(&mut list, 0, 0, "OK", false, &font());
        let quads = list.quads;
        let ok = with_texture(&quads, BTN1_NORMAL);
        assert_eq!(ok.len(), 1);
        assert_eq!((ok[0].x, ok[0].y, ok[0].w, ok[0].h), (90.0, 75.0, 76.0, 23.0));
        let back = with_texture(&quads, SYSTEM_BACK);
        assert_eq!((back[0].w, back[0].h), (256.0, 108.0));
        let icon = with_texture(&quads, WARNING_ICON);
        assert_eq!((icon[0].x, icon[0].y, icon[0].w, icon[0].h), (11.0, 15.0, 24.0, 24.0));
        assert!(WarningDialog::hit(100, 80) && !WarningDialog::hit(60, 80));
        assert_eq!(warning.ok(), WarningOutcome::Hide);
        assert!(!warning.shown);
        warning.show(Some("fatal"), true);
        assert_eq!(warning.ok(), WarningOutcome::Exit);
        assert!(warning.shown && warning.exit);
        warning.show(None, false);
        assert_eq!(warning.message, "fatal");
    }
}
