use crate::button::TextButton;
use crate::chat::{CARET, CARET_BACK, InputKey, TextInput};
use crate::color::{Argb, TEXT};
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::window::Rect;

pub const MESSAGE_BAR_HEIGHT: i32 = 52;
pub const AUTH_SIZE: (i32, i32) = (256, 108);
pub const AUTH_BACK: &str = "L2UI_CH3.LoginWnd.logon_back";
pub const AUTH_ID_RECT: Rect = Rect::new(84, 20, 128, 17);
pub const AUTH_PASSWORD_RECT: Rect = Rect::new(84, 42, 128, 17);
pub const AUTH_LOGIN_RECT: Rect = Rect::new(31, 72, 96, 23);
pub const AUTH_EXIT_RECT: Rect = Rect::new(131, 72, 96, 23);
pub const AUTH_LABEL_RIGHT: i32 = 0x4e;
pub const AUTH_ID_LABEL_Y: i32 = 0x17;
pub const AUTH_PASSWORD_LABEL_Y: i32 = 0x2d;
pub const EDIT_TEXT_INSET: i32 = 2;
pub const EDIT_BACK: [&str; 3] = ["L2UI_CH3.Etc.inputbox1", "L2UI_CH3.Etc.inputbox2", "L2UI_CH3.Etc.inputbox3"];
pub const EDIT_PIECE: (i32, i32) = (8, 17);

pub fn paint_edit_back(list: &mut DrawList, rect: Rect) {
    let (piece, row) = EDIT_PIECE;
    list.draw_texture(rect.x, rect.y, piece, rect.h, 0, 0, piece, row, EDIT_BACK[0], 1.0);
    list.draw_texture(rect.x + piece, rect.y, rect.w - 2 * piece, rect.h, 0, 0, piece, row, EDIT_BACK[1], 1.0);
    list.draw_texture(rect.x + rect.w - piece, rect.y, piece, rect.h, 0, 0, piece, row, EDIT_BACK[2], 1.0);
}
pub const BIG_BUTTON: &str = "L2UI_CH3.BUTTON.bigbutton";
pub const BIG_BUTTON_DOWN: &str = "L2UI_CH3.BUTTON.bigbutton_down";
pub const FUNCTION_SIZE: (i32, i32) = (120, 138);
pub const FUNCTION_MARGIN: (i32, i32) = (154, 200);
pub const FUNCTION_BACK: &str = "L2UI_CH3.LobbyWnd.lobby_menuback";
pub const FUNCTION_BUTTON_RECTS: [Rect; 5] = [
    Rect::new(13, 8, 96, 23),
    Rect::new(13, 33, 96, 23),
    Rect::new(13, 58, 96, 23),
    Rect::new(13, 83, 96, 23),
    Rect::new(13, 107, 96, 23),
];
pub const MESSAGE_BAR_BACK: &str = "sek.cbui141";
pub const MESSAGE_WRAP_MARGIN: i32 = 100;
pub const MESSAGE_FALLBACK_POSITION: (i32, i32) = (50, 8);
pub const LOGO: &str = "L2Font-e.start_logo-e";
pub const MINI_LOGO: &str = "L2Font-e.mini_logo-e";
pub const MINI_LOGO_SIZE: (i32, i32) = (232, 120);
pub const LOGO_SIZE: i32 = 256;
pub const LOGO_TOP_FRACTION: f32 = 0.05;
pub const LOCK_TIMER_MS: u32 = 1000;
pub const SYSMSG_EMPTY_ID: u32 = 86;
pub const SYSMSG_EMPTY_PASSWORD: u32 = 87;
pub const SYSMSG_CONNECT_FAILED: u32 = 90;
pub const SYSMSG_CARD_EMPTY: u32 = 1236;
pub const SYSMSG_CARD_PROMPT: u32 = 1237;
pub const CARD_PROMPT_POS: (i32, i32) = (30, 23);
pub const CARD_DIGITS: usize = 4;
pub const CARD_WRONG: u8 = 31;
pub const SYSMSG_NEW_ACCOUNT: u32 = 1186;
pub const SYSMSG_LOST_ACCOUNT: u32 = 1187;
pub const SYSSTRINGS: [u32; 10] = [150, 151, 152, 153, 154, 155, 156, 805, 149, 1251];

pub fn refusal_ids(reason: u8) -> &'static [u32] {
    match reason {
        1 => &[448],
        2 | 3 => &[449, 450],
        4 | 6 | 8 | 9 | 10 | 11 | 13 | 14 => &[461, 462, 463],
        5 => &[453, 454],
        7 => &[455],
        12 => &[456],
        15 => &[1650],
        16 => &[457],
        17 => &[396],
        18 => &[458, 459, 460],
        19 => &[398],
        20 => &[399],
        21 => &[461],
        22 => &[621],
        30 => &[756],
        31 => &[1243],
        32 => &[1242],
        33 => &[1340],
        35 => &[1407],
        _ => &[],
    }
}

pub const BLOCKED_FIRST_ID: u32 = 428;
pub const BLOCKED_FLAG_BITS: u32 = 16;
pub const BLOCKED_EXTRA_ID: u32 = 437;
pub const BLOCKED_WITH_MESSAGE_IDS: [u32; 4] = [444, 445, 446, 447];

pub fn blocked_id(flags: u32) -> Option<u32> {
    let low = flags & 0xffff;
    if low != 0 {
        return Some(BLOCKED_FIRST_ID + low.trailing_zeros());
    }
    (flags & 0x30000 != 0).then_some(BLOCKED_EXTRA_ID)
}

pub fn blocked_text(flags: u32, lookup: impl Fn(u32) -> Option<String>) -> Option<String> {
    lookup(blocked_id(flags)?)
}

pub fn blocked_message_text(message: &str, lookup: impl Fn(u32) -> Option<String>) -> Option<String> {
    let parts: Vec<String> = BLOCKED_WITH_MESSAGE_IDS.iter().filter_map(|id| lookup(*id)).collect();
    (parts.len() == BLOCKED_WITH_MESSAGE_IDS.len()).then(|| parts.join(" ").replacen("%s", message, 1))
}

pub fn refusal_text(reason: u8, lookup: impl Fn(u32) -> Option<String>) -> Option<String> {
    let ids = refusal_ids(reason);
    if ids.is_empty() {
        return None;
    }
    let parts: Vec<String> = ids.iter().filter_map(|id| lookup(*id)).collect();
    (parts.len() == ids.len()).then(|| parts.join(" "))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Id,
    Password,
    Login,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hit {
    Field(Field),
    Login,
    Exit,
    Function(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginAction {
    Login { account: String, password: String },
    Exit,
    CardCheck { card_id: i32, value: i32 },
    Message(u32),
    Options,
    Credits,
    Replay,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoginLabels {
    pub id: String,
    pub password: String,
    pub login: String,
    pub exit: String,
    pub function: [String; 5],
    pub title: String,
}

impl LoginLabels {
    pub fn from_sysstrings(lookup: impl Fn(u32) -> Option<String>) -> LoginLabels {
        let text = |id: u32| lookup(id).unwrap_or_default();
        LoginLabels {
            id: text(SYSSTRINGS[0]),
            password: text(SYSSTRINGS[1]),
            login: text(SYSSTRINGS[2]),
            exit: text(SYSSTRINGS[3]),
            function: [text(SYSSTRINGS[4]), text(SYSSTRINGS[5]), text(SYSSTRINGS[6]), text(SYSSTRINGS[7]), text(SYSSTRINGS[9])],
            title: text(SYSSTRINGS[8]),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardPrompt {
    pub card_id: i32,
    pub position: i32,
}

pub fn card_value(text: &str) -> i32 {
    let digits = text.chars().count();
    if digits != CARD_DIGITS || !text.chars().all(|c| c.is_ascii_digit()) {
        return 0;
    }
    text.parse().unwrap_or(0)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginScreen {
    pub screen: (i32, i32),
    pub labels: LoginLabels,
    pub id: TextInput,
    pub password: TextInput,
    pub focus: Field,
    pub locked: bool,
    pub timer_ms: u32,
    pub message: String,
    pub pressed: Option<Hit>,
    pub auth_visible: bool,
    pub card: Option<CardPrompt>,
    pub card_prompt: String,
    pub logo: String,
    pub mini_logo: String,
}

impl LoginScreen {
    pub fn new(screen: (i32, i32), labels: LoginLabels) -> LoginScreen {
        LoginScreen {
            screen,
            labels,
            id: TextInput::default(),
            password: TextInput::default(),
            focus: Field::Id,
            locked: false,
            timer_ms: 0,
            message: String::new(),
            pressed: None,
            auth_visible: true,
            card: None,
            card_prompt: String::new(),
            logo: LOGO.to_string(),
            mini_logo: MINI_LOGO.to_string(),
        }
    }

    pub fn login_height(&self) -> i32 {
        self.screen.1 - MESSAGE_BAR_HEIGHT
    }

    pub fn auth_origin(&self) -> (i32, i32) {
        (self.screen.0 / 2 - AUTH_SIZE.0 / 2, self.login_height() / 2 - AUTH_SIZE.1 / 2)
    }

    pub fn function_origin(&self) -> (i32, i32) {
        (self.screen.0 - FUNCTION_MARGIN.0, self.screen.1 - FUNCTION_MARGIN.1)
    }

    pub fn prefill(&mut self, account: &str, password: &str) {
        self.id = TextInput { text: account.to_string(), cursor: account.chars().count() };
        self.password = TextInput { text: password.to_string(), cursor: password.chars().count() };
        self.focus = Field::Login;
    }

    pub fn reset(&mut self) {
        self.password = TextInput::default();
        self.focus = Field::Id;
        self.auth_visible = true;
        self.card = None;
    }

    pub fn set_card(&mut self, card_id: i32, position: i32) {
        self.card = Some(CardPrompt { card_id, position });
        self.password = TextInput::default();
        self.focus = Field::Password;
    }

    pub fn set_message(&mut self, text: &str) {
        self.message = text.to_string();
    }

    pub fn tick(&mut self, delta_ms: u32) {
        self.timer_ms = self.timer_ms.saturating_add(delta_ms);
        if self.timer_ms > LOCK_TIMER_MS {
            self.timer_ms -= LOCK_TIMER_MS;
            self.locked = false;
        }
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<Hit> {
        let function = self.function_origin();
        for (index, rect) in FUNCTION_BUTTON_RECTS.iter().enumerate() {
            if rect.offset(function.0, function.1).contains(x, y) {
                return Some(Hit::Function(index));
            }
        }
        if !self.auth_visible {
            return None;
        }
        let origin = self.auth_origin();
        let (x, y) = (x - origin.0, y - origin.1);
        if self.card.is_none() && AUTH_ID_RECT.contains(x, y) {
            return Some(Hit::Field(Field::Id));
        }
        if AUTH_PASSWORD_RECT.contains(x, y) {
            return Some(Hit::Field(Field::Password));
        }
        if AUTH_LOGIN_RECT.contains(x, y) {
            return Some(Hit::Login);
        }
        if AUTH_EXIT_RECT.contains(x, y) {
            return Some(Hit::Exit);
        }
        None
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) {
        self.pressed = self.hit(x, y);
        if let Some(Hit::Field(field)) = self.pressed {
            self.focus = field;
        }
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<LoginAction> {
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        match pressed {
            Hit::Field(_) => None,
            Hit::Login => self.attempt(),
            Hit::Exit => Some(LoginAction::Exit),
            Hit::Function(0) => Some(LoginAction::Message(SYSMSG_NEW_ACCOUNT)),
            Hit::Function(1) => Some(LoginAction::Message(SYSMSG_LOST_ACCOUNT)),
            Hit::Function(2) => Some(LoginAction::Options),
            Hit::Function(3) => Some(LoginAction::Credits),
            Hit::Function(_) => Some(LoginAction::Replay),
        }
    }

    pub fn key(&mut self, key: InputKey, shift: bool) -> Option<LoginAction> {
        if !self.auth_visible {
            return None;
        }
        match key {
            InputKey::Char('\t') => {
                if self.card.is_some() {
                    return None;
                }
                self.focus = match (self.focus, shift) {
                    (Field::Id, _) => Field::Password,
                    (Field::Password | Field::Login, _) => Field::Id,
                };
                None
            }
            InputKey::Enter => if self.focus != Field::Id { self.attempt() } else { None },
            InputKey::Escape => None,
            other => {
                let input = match self.focus {
                    Field::Id => &mut self.id,
                    Field::Password => &mut self.password,
                    Field::Login => return None,
                };
                input.key(other);
                None
            }
        }
    }

    fn attempt(&mut self) -> Option<LoginAction> {
        if let Some(card) = self.card {
            if self.password.text.is_empty() {
                return Some(LoginAction::Message(SYSMSG_CARD_EMPTY));
            }
            if self.locked {
                return None;
            }
            self.locked = true;
            return Some(LoginAction::CardCheck { card_id: card.card_id, value: card_value(&self.password.text) });
        }
        if self.id.text.is_empty() {
            return Some(LoginAction::Message(SYSMSG_EMPTY_ID));
        }
        if self.password.text.is_empty() {
            self.locked = true;
            return Some(LoginAction::Message(SYSMSG_EMPTY_PASSWORD));
        }
        if self.locked {
            return None;
        }
        self.locked = true;
        Some(LoginAction::Login { account: self.id.text.clone(), password: self.password.text.clone() })
    }

    pub fn paint_backdrop(&self, list: &mut DrawList, font: &BitmapFont) {
        self.paint_logo(list, font, true);
    }

    pub fn paint_logo(&self, list: &mut DrawList, font: &BitmapFont, function: bool) {
        let (w, h) = self.screen;
        let login_h = self.login_height();
        list.draw_texture(0, 0, MINI_LOGO_SIZE.0, MINI_LOGO_SIZE.1, 0, 0, MINI_LOGO_SIZE.0, MINI_LOGO_SIZE.1, &self.mini_logo, 1.0);
        let logo_x = (w as f32 * 0.5 - LOGO_SIZE as f32 / 2.0) as i32;
        let logo_y = (login_h as f32 * LOGO_TOP_FRACTION) as i32;
        list.draw_texture(logo_x, logo_y, LOGO_SIZE, LOGO_SIZE, 0, 0, LOGO_SIZE, LOGO_SIZE, &self.logo, 1.0);
        if function {
            self.paint_function(list, font);
        }
        self.paint_message(list, font, w, h);
    }

    pub fn paint(&self, list: &mut DrawList, font: &BitmapFont) {
        self.paint_backdrop(list, font);
        if self.auth_visible {
            self.paint_auth(list, font);
        }
    }

    fn paint_auth(&self, list: &mut DrawList, font: &BitmapFont) {
        let (ox, oy) = self.auth_origin();
        list.draw_texture(ox, oy, AUTH_SIZE.0, AUTH_SIZE.1, 0, 0, AUTH_SIZE.0, AUTH_SIZE.1, AUTH_BACK, 1.0);
        if let Some(card) = self.card {
            let text = self.card_prompt.replace("$s1", &card.position.to_string());
            list.draw_text(ox + CARD_PROMPT_POS.0, oy + CARD_PROMPT_POS.1, TEXT, &text, font);
        } else {
            for (label, y) in [(&self.labels.id, AUTH_ID_LABEL_Y), (&self.labels.password, AUTH_PASSWORD_LABEL_Y)] {
                if label.is_empty() {
                    continue;
                }
                let width = font.measure(label) as i32;
                list.draw_text(ox + AUTH_LABEL_RIGHT - width, oy + y, TEXT, label, font);
            }
        }
        let masked: String = "*".repeat(self.password.text.chars().count());
        for (field, rect, text) in [(Field::Id, AUTH_ID_RECT, self.id.text.as_str()), (Field::Password, AUTH_PASSWORD_RECT, masked.as_str())] {
            if field == Field::Id && self.card.is_some() {
                continue;
            }
            paint_edit_back(list, Rect::new(ox + rect.x, oy + rect.y, rect.w, rect.h));
            let (tx, ty) = (ox + rect.x + EDIT_TEXT_INSET, oy + rect.y + EDIT_TEXT_INSET);
            list.draw_text_clipped(tx, ty, rect.w - EDIT_TEXT_INSET, TEXT, text, font);
            if self.focus == field {
                let before = match field {
                    Field::Id => self.id.before_cursor().to_string(),
                    Field::Password => "*".repeat(self.password.before_cursor().chars().count()),
                    Field::Login => String::new(),
                };
                list.draw_text(tx + font.measure(&before) as i32 - CARET_BACK, ty, TEXT, &CARET.to_string(), font);
            }
        }
        let pressed = |hit: Hit| self.pressed == Some(hit);
        TextButton::new(AUTH_LOGIN_RECT, BIG_BUTTON, BIG_BUTTON_DOWN, &self.labels.login).paint(list, ox, oy, pressed(Hit::Login), font);
        TextButton::new(AUTH_EXIT_RECT, BIG_BUTTON, BIG_BUTTON_DOWN, &self.labels.exit).paint(list, ox, oy, pressed(Hit::Exit), font);
    }

    fn paint_function(&self, list: &mut DrawList, font: &BitmapFont) {
        let (ox, oy) = self.function_origin();
        list.draw_texture(ox, oy, FUNCTION_SIZE.0, FUNCTION_SIZE.1, 0, 0, FUNCTION_SIZE.0, FUNCTION_SIZE.1, FUNCTION_BACK, 1.0);
        for (index, rect) in FUNCTION_BUTTON_RECTS.iter().enumerate() {
            TextButton::new(*rect, BIG_BUTTON, BIG_BUTTON_DOWN, &self.labels.function[index]).paint(list, ox, oy, self.pressed == Some(Hit::Function(index)), font);
        }
    }

    pub fn paint_message(&self, list: &mut DrawList, font: &BitmapFont, w: i32, h: i32) {
        let top = h - MESSAGE_BAR_HEIGHT;
        list.draw_texture(0, top, w, MESSAGE_BAR_HEIGHT, 0, 0, w, MESSAGE_BAR_HEIGHT, MESSAGE_BAR_BACK, 1.0);
        if self.message.is_empty() {
            return;
        }
        let width = font.measure(&self.message) as i32;
        let color: Argb = TEXT;
        if width < w {
            let x = (w as f32 * 0.5 - (width / 2) as f32) as i32;
            let y = (MESSAGE_BAR_HEIGHT as f32 * 0.5 - (font.line_height as i32 / 2) as f32) as i32;
            list.draw_text(x, top + y, color, &self.message, font);
        } else {
            let lines = crate::text_align::bitmap_lines(&self.message, w - MESSAGE_WRAP_MARGIN, font, 1, false);
            crate::text_align::paint(list, (MESSAGE_FALLBACK_POSITION.0, top + MESSAGE_FALLBACK_POSITION.1), &lines, color, font, true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};

    fn login_screen() -> LoginScreen {
        LoginScreen::new((1024, 768), LoginLabels::from_sysstrings(|id| Some(format!("s{id}"))))
    }

    #[test]
    fn the_card_mode_of_the_auth_box_follows_slot128_mode_1_at_0x1009e5aa() {
        let mut screen = login_screen();
        screen.set_card(77, 3);
        screen.card_prompt = "card $s1".into();
        assert_eq!(screen.focus, Field::Password);
        let (ox, oy) = screen.auth_origin();
        assert_eq!(screen.hit(ox + AUTH_ID_RECT.x + 1, oy + AUTH_ID_RECT.y + 1), None);
        assert_eq!(screen.key(InputKey::Char('\t'), false), None);
        assert_eq!(screen.focus, Field::Password);
        assert_eq!(screen.key(InputKey::Enter, false), Some(LoginAction::Message(SYSMSG_CARD_EMPTY)));
        assert!(!screen.locked);
        for c in "12a4".chars() {
            screen.key(InputKey::Char(c), false);
        }
        assert_eq!(screen.key(InputKey::Enter, false), Some(LoginAction::CardCheck { card_id: 77, value: 0 }));
        assert!(screen.locked);
        assert_eq!(screen.key(InputKey::Enter, false), None);
        screen.locked = false;
        screen.set_card(77, 3);
        for c in "1234".chars() {
            screen.key(InputKey::Char(c), false);
        }
        assert_eq!(screen.key(InputKey::Enter, false), Some(LoginAction::CardCheck { card_id: 77, value: 1234 }));
        let mut list = DrawList { capture_text: true, ..Default::default() };
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        let font = BitmapFont::new("f".into(), crate::font::Glyphs::parse(&crate::font::gly(&[(0x20, &glyphs)], (1024, 128))).unwrap());
        screen.paint(&mut list, &font);
        let texts: Vec<&str> = list.text_runs.iter().map(|run| run.text.as_str()).collect();
        assert!(texts.contains(&"card 3"));
        assert!(!texts.contains(&"s150"));
        screen.reset();
        assert!(screen.card.is_none());
    }

    #[test]
    fn card_value_accepts_exactly_four_digits_and_everything_else_is_zero() {
        assert_eq!(card_value("0123"), 123);
        assert_eq!(card_value("123"), 0);
        assert_eq!(card_value("12345"), 0);
        assert_eq!(card_value("12a4"), 0);
        assert_eq!(card_value(""), 0);
    }

    #[test]
    fn blocked_flags_pick_the_lowest_bit_like_1006f040() {
        assert_eq!(blocked_id(0), None);
        assert_eq!(blocked_id(1), Some(428));
        assert_eq!(blocked_id(0x10), Some(432));
        assert_eq!(blocked_id(0x20 | 0x400), Some(433));
        assert_eq!(blocked_id(0x200), Some(437));
        assert_eq!(blocked_id(0x8000), Some(443));
        assert_eq!(blocked_id(0x10000), Some(437));
        assert_eq!(blocked_id(0x20000 | 0x8000), Some(443));
        let lookup = |id: u32| Some(format!("m{id}"));
        assert_eq!(blocked_text(2, lookup).as_deref(), Some("m429"));
        assert_eq!(blocked_message_text("x", lookup).as_deref(), Some("m444 m445 m446 m447"));
        let with_slot = |id: u32| Some(if id == 445 { "ref %s".to_string() } else { format!("m{id}") });
        assert_eq!(blocked_message_text("123", with_slot).as_deref(), Some("m444 ref 123 m446 m447"));
    }

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn screen() -> LoginScreen {
        LoginScreen::new((1024, 768), LoginLabels::from_sysstrings(|id| Some(format!("s{id}"))))
    }

    fn type_text(screen: &mut LoginScreen, text: &str) {
        for c in text.chars() {
            screen.key(InputKey::Char(c), false);
        }
    }

    #[test]
    fn auth_box_is_centered_in_the_login_window_that_leaves_the_message_bar_out() {
        let screen = screen();
        assert_eq!(screen.auth_origin(), (1024 / 2 - 128, (768 - 52) / 2 - 54));
        assert_eq!(screen.function_origin(), (1024 - 154, 768 - 200));
    }

    #[test]
    fn validation_follows_slot128_empty_id_then_empty_password_then_the_request() {
        let mut screen = screen();
        assert_eq!(screen.attempt(), Some(LoginAction::Message(SYSMSG_EMPTY_ID)));
        type_text(&mut screen, "conta");
        assert_eq!(screen.attempt(), Some(LoginAction::Message(SYSMSG_EMPTY_PASSWORD)));
        screen.locked = false;
        screen.key(InputKey::Char('\t'), false);
        type_text(&mut screen, "senha");
        assert_eq!(screen.attempt(), Some(LoginAction::Login { account: "conta".into(), password: "senha".into() }));
        assert_eq!(screen.attempt(), None);
        screen.tick(LOCK_TIMER_MS);
        assert_eq!(screen.attempt(), None);
        screen.tick(1);
        assert!(matches!(screen.attempt(), Some(LoginAction::Login { .. })));
        assert_eq!(screen.attempt(), None);
        screen.tick(LOCK_TIMER_MS);
        assert!(matches!(screen.attempt(), Some(LoginAction::Login { .. })));
        assert_eq!(screen.attempt(), None);
        screen.tick(LOCK_TIMER_MS);
        assert!(matches!(screen.attempt(), Some(LoginAction::Login { .. })));
    }

    #[test]
    fn tab_switches_the_field_and_enter_only_counts_in_the_password() {
        let mut screen = screen();
        type_text(&mut screen, "conta");
        assert_eq!(screen.key(InputKey::Enter, false), None);
        assert_eq!(screen.focus, Field::Id);
        screen.key(InputKey::Char('\t'), false);
        assert_eq!(screen.focus, Field::Password);
        type_text(&mut screen, "senha");
        assert!(matches!(screen.key(InputKey::Enter, false), Some(LoginAction::Login { .. })));
        screen.key(InputKey::Char('\t'), true);
        assert_eq!(screen.focus, Field::Id);
    }

    #[test]
    fn clicks_hit_the_fields_the_buttons_and_the_function_menu() {
        let mut screen = screen();
        let (ox, oy) = screen.auth_origin();
        screen.mouse_down(ox + AUTH_PASSWORD_RECT.x + 1, oy + AUTH_PASSWORD_RECT.y + 1);
        assert_eq!(screen.focus, Field::Password);
        assert_eq!(screen.mouse_up(ox + AUTH_PASSWORD_RECT.x + 1, oy + AUTH_PASSWORD_RECT.y + 1), None);
        let (x, y) = (ox + AUTH_EXIT_RECT.x + 2, oy + AUTH_EXIT_RECT.y + 2);
        screen.mouse_down(x, y);
        assert_eq!(screen.mouse_up(x, y), Some(LoginAction::Exit));
        let (fx, fy) = screen.function_origin();
        for (index, expected) in [
            LoginAction::Message(SYSMSG_NEW_ACCOUNT),
            LoginAction::Message(SYSMSG_LOST_ACCOUNT),
            LoginAction::Options,
            LoginAction::Credits,
            LoginAction::Replay,
        ]
        .into_iter()
        .enumerate()
        {
            let rect = FUNCTION_BUTTON_RECTS[index];
            let (x, y) = (fx + rect.x + 1, fy + rect.y + 1);
            screen.mouse_down(x, y);
            assert_eq!(screen.mouse_up(x, y), Some(expected));
        }
    }

    #[test]
    fn the_password_is_painted_as_asterisks_and_the_message_is_centered_when_it_fits() {
        let mut screen = screen();
        type_text(&mut screen, "ab");
        screen.key(InputKey::Char('\t'), false);
        type_text(&mut screen, "segredo");
        screen.set_message("Hello");
        let mut list = DrawList { capture_text: true, ..Default::default() };
        screen.paint(&mut list, &font());
        let texts: Vec<String> = list.text_runs.iter().map(|t| t.text.clone()).collect();
        assert!(texts.iter().any(|t| t == "*******"));
        assert!(!texts.iter().any(|t| t == "segredo"));
        assert!(texts.iter().any(|t| t == "Hello"));
    }

    #[test]
    fn message_bar_100b84e0_paints_the_back_over_52_pixels_and_centers_or_falls_back_to_fifty_and_eight() {
        let mut screen = screen();
        let font = font();
        let (w, h) = (1024, 768);
        screen.set_message("Hello");
        let mut list = DrawList { capture_text: true, ..Default::default() };
        screen.paint_message(&mut list, &font, w, h);
        let back = list.quads.iter().find(|quad| quad.texture == MESSAGE_BAR_BACK).unwrap();
        assert_eq!((back.x, back.y, back.w, back.h), (0.0, (h - MESSAGE_BAR_HEIGHT) as f32, w as f32, MESSAGE_BAR_HEIGHT as f32));
        let width = font.measure("Hello") as i32;
        let line = font.line_height as i32;
        let run = &list.text_runs[0];
        assert_eq!((run.rect[0] as i32, run.rect[1] as i32), ((w as f32 * 0.5 - (width / 2) as f32) as i32, h - MESSAGE_BAR_HEIGHT + (MESSAGE_BAR_HEIGHT as f32 * 0.5 - (line / 2) as f32) as i32));
        let long = "word ".repeat(120);
        screen.set_message(&long);
        let mut list = DrawList { capture_text: true, ..Default::default() };
        screen.paint_message(&mut list, &font, w, h);
        assert!(list.text_runs.len() > 1);
        let run = &list.text_runs[0];
        assert_eq!((run.rect[0] as i32, run.rect[1] as i32), (MESSAGE_FALLBACK_POSITION.0, h - MESSAGE_BAR_HEIGHT + MESSAGE_FALLBACK_POSITION.1));
    }

    #[test]
    fn refusal_reasons_use_the_original_message_table_and_join_with_spaces() {
        let lookup = |id: u32| Some(format!("m{id}"));
        assert_eq!(refusal_text(1, lookup).as_deref(), Some("m448"));
        assert_eq!(refusal_text(2, lookup).as_deref(), Some("m449 m450"));
        assert_eq!(refusal_text(3, lookup), refusal_text(2, lookup));
        assert_eq!(refusal_text(18, lookup).as_deref(), Some("m458 m459 m460"));
        assert_eq!(refusal_text(31, lookup).as_deref(), Some("m1243"));
        assert_eq!(refusal_text(35, lookup).as_deref(), Some("m1407"));
        for silent in [0u8, 23, 24, 25, 26, 27, 28, 29, 34, 36, 200] {
            assert_eq!(refusal_text(silent, lookup), None, "{silent}");
        }
        assert_eq!(refusal_text(2, |id| (id == 449).then(|| "x".to_string())), None);
        let table_reasons = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 30, 31, 32, 33, 35];
        assert!(table_reasons.iter().all(|reason| !refusal_ids(*reason).is_empty()));
    }

    #[test]
    fn prefill_fills_both_boxes_and_focuses_the_ok_button_where_enter_and_tab_behave_like_the_ok() {
        let mut screen = LoginScreen::new((1024, 768), LoginLabels::default());
        screen.prefill("conta", "senha");
        assert_eq!((screen.id.text.as_str(), screen.password.text.as_str()), ("conta", "senha"));
        assert_eq!(screen.focus, Field::Login);
        assert_eq!(screen.key(InputKey::Char('x'), false), None);
        assert_eq!(screen.id.text, "conta");
        assert_eq!(screen.key(InputKey::Enter, false), Some(LoginAction::Login { account: "conta".into(), password: "senha".into() }));
        screen.focus = Field::Login;
        screen.key(InputKey::Char('\t'), false);
        assert_eq!(screen.focus, Field::Id);
    }
}
