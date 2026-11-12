#[path = "petition_gm_controls_generated.rs"]
mod gm_controls;
#[path = "petition_gm_layout_generated.rs"]
mod gm_layout;
#[path = "petition_chat_controls_generated.rs"]
mod player_controls;
#[path = "petition_chat_layout_generated.rs"]
mod player_layout;

use crate::button::TextButton;
use crate::color::Argb;
use crate::draw::DrawList;
use crate::edit_box::{EditBox, FontMetrics, NOTICE_ENTER};
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::frame_contract::{FrameContract, FrameHit};
use crate::minimized::IconView;
use crate::text_list::TextList;
use crate::window::{Rect, WindowId};

pub const PLAYER_ID: WindowId = WindowId::PETITION;
pub const GM_ID: WindowId = WindowId::PETITION_GM;
pub const SIZE: (i32, i32) = player_layout::SIZE;
pub const BACK: &str = "L2UI_ch3.PetitionWnd.petition2_back";
pub const BACK_ORIGIN: (i32, i32) = (0, 20);
pub const LIST_MAX_LINES: usize = 200;
pub const LIST_VISIBLE_ROWS: i32 = 17;
pub const EDIT_MAX_LEN: i32 = 0x400;
pub const PLAYER_LIST_ID: u32 = 0xf0a0b1;
pub const GM_LIST_ID: u32 = 0xf0a0c1;
pub const SAY_PETITION_PLAYER: i32 = 6;
pub const SAY_PETITION_GM: i32 = 7;
pub const PLAYER_COLOR: Argb = Argb(0xFFDCDCDC);
pub const START_COMMAND: &str = "startpet";
pub const END_COMMAND: &str = "endpet";
pub const PLAYER_POSITION_X: i32 = 106;
pub const PLAYER_POSITION_Y_OFFSET: i32 = 192;
pub const GM_INITIAL_Y: i32 = 252;

pub fn player_position(viewport: (i32, i32)) -> (i32, i32) {
    (PLAYER_POSITION_X, viewport.1 / 2 - PLAYER_POSITION_Y_OFFSET)
}

pub fn gm_default_position(viewport: (i32, i32)) -> (i32, i32) {
    (0, (viewport.1 as f32 * 0.5 - GM_INITIAL_Y as f32) as i32)
}

pub fn gm_initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    let (x, y) = gm_default_position(viewport);
    crate::initial_position::InitialPosition {
        enabled: true,
        horizontal_mode: 0,
        vertical_mode: 5,
        x,
        y: (viewport.1 / 2).wrapping_sub(y),
    }
}

pub fn grip() -> Rect {
    player_layout::RECT_7C
}

pub fn gm_grip() -> Rect {
    gm_layout::RECT_7C
}

pub const REGISTRATION_FLAGS: u32 = 2;
pub const ICON_STYLE: u32 = 0x2000;

pub fn has_icon() -> bool {
    REGISTRATION_FLAGS & ICON_STYLE != 0
}

pub fn show_over_icon(window: &mut Rect, icon: Option<&mut IconView>) {
    let Some(icon) = icon else { return };
    window.x = icon.rect.x;
    window.y = icon.rect.y;
    icon.rect.x = window.x;
    icon.rect.y = window.y;
    icon.hide();
}

pub const GM_SIZE: (i32, i32) = gm_layout::SIZE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatKind {
    Player,
    Gm,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChatAction {
    Say { kind: i32, text: String },
    Bypass(String),
    Cancel,
    Feedback,
    Close,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatHit {
    Close,
    Button(usize),
    Edit,
    List,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Say,
    Build,
    Ignored,
}

pub fn classify(text: &str) -> Command {
    let first = text.chars().next();
    if first != Some('/') && first != Some('\u{ff0f}') {
        return Command::Say;
    }
    let token = text.split_whitespace().next().unwrap_or("");
    let slashes = token.chars().take_while(|&c| c == '/').count();
    if slashes >= 3 && token.chars().count() > 3 {
        Command::Ignored
    } else if slashes >= 2 && token.chars().count() > 2 {
        Command::Build
    } else {
        Command::Ignored
    }
}

#[derive(Clone, Debug)]
pub struct PetitionChat {
    pub kind: ChatKind,
    pub frame: FrameCtrl,
    pub list: TextList,
    pub edit: EditBox,
    pub buttons: [TextButton; 2],
    frame_contract: &'static FrameContract,
    pressed: Option<ChatHit>,
}

impl PetitionChat {
    pub fn player(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        let mut buttons = [player_controls::BUTTON_F8.build((0, 0), text), player_controls::BUTTON_FC.build((0, 0), text)];
        buttons[1].enabled = false;
        Self::assemble(
            ChatKind::Player,
            &player_controls::FRAME_7C,
            &player_controls::CHILD_EC,
            TextList::new(PLAYER_LIST_ID, player_layout::RECT_F0, LIST_MAX_LINES, LIST_VISIBLE_ROWS, true),
            buttons,
            &lookup,
        )
    }

    pub fn gm(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        let buttons = [gm_controls::BUTTON_F4.build((0, 0), text), gm_controls::BUTTON_F8.build((0, 0), text)];
        Self::assemble(
            ChatKind::Gm,
            &gm_controls::FRAME_7C,
            &gm_controls::CHILD_EC,
            TextList::new(GM_LIST_ID, gm_layout::RECT_F0, LIST_MAX_LINES, LIST_VISIBLE_ROWS, true),
            buttons,
            &lookup,
        )
    }

    fn assemble(
        kind: ChatKind,
        frame_contract: &'static FrameContract,
        edit: &crate::child_contract::ChildContract,
        list: TextList,
        buttons: [TextButton; 2],
        lookup: &dyn Fn(u32) -> Option<String>,
    ) -> Self {
        let mut edit = EditBox::from_contract(edit);
        edit.set_max_len(EDIT_MAX_LEN);
        Self {
            kind,
            frame: frame_contract.build(|id| lookup(id).unwrap_or_default()),
            list,
            edit,
            buttons,
            frame_contract,
            pressed: None,
        }
    }

    pub fn feedback_enabled(&self) -> bool {
        self.kind == ChatKind::Player && self.buttons[1].enabled
    }

    pub fn set_feedback_enabled(&mut self, enabled: bool) {
        if self.kind == ChatKind::Player {
            self.buttons[1].enabled = enabled;
        }
    }

    pub fn hide(&mut self) {
        self.set_feedback_enabled(false);
        self.edit.kill_focus();
        self.pressed = None;
    }

    pub fn add_message(&mut self, text: &str, color: Argb, kind: i32, font: &BitmapFont) {
        let color = if kind == SAY_PETITION_PLAYER { PLAYER_COLOR } else { color };
        self.list.add_string(text, color, &FontMetrics(font));
    }

    pub fn clear(&mut self) {
        self.edit.clear();
        self.list.clear();
    }

    fn button_hit(&self, index: usize, x: i32, y: i32) -> bool {
        let button = &self.buttons[index];
        match (self.kind, index) {
            (ChatKind::Player, 0) => player_controls::BUTTON_F8.hit(button, x, y),
            (ChatKind::Player, _) => player_controls::BUTTON_FC.hit(button, x, y),
            (ChatKind::Gm, 0) => gm_controls::BUTTON_F4.hit(button, x, y),
            (ChatKind::Gm, _) => gm_controls::BUTTON_F8.hit(button, x, y),
        }
        .is_some()
    }

    fn paint_button(&self, index: usize, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont) {
        let button = &self.buttons[index];
        let pressed = self.pressed == Some(ChatHit::Button(index));
        match (self.kind, index) {
            (ChatKind::Player, 0) => player_controls::BUTTON_F8.paint(button, list, origin, pressed, font),
            (ChatKind::Player, _) => player_controls::BUTTON_FC.paint(button, list, origin, pressed, font),
            (ChatKind::Gm, 0) => gm_controls::BUTTON_F4.paint(button, list, origin, pressed, font),
            (ChatKind::Gm, _) => gm_controls::BUTTON_F8.paint(button, list, origin, pressed, font),
        }
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<ChatHit> {
        if self.frame_contract.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close) {
            return Some(ChatHit::Close);
        }
        if let Some(index) = (0..2).find(|&index| self.button_hit(index, x, y)) {
            return Some(ChatHit::Button(index));
        }
        if self.edit.rect.contains(x, y) {
            return Some(ChatHit::Edit);
        }
        self.list.rect.contains(x, y).then_some(ChatHit::List)
    }

    pub fn mouse_down(&mut self, x: i32, y: i32, font: &BitmapFont) -> Option<ChatHit> {
        let hit = self.hit(x, y);
        self.pressed = hit;
        match hit {
            Some(ChatHit::Edit) => {
                if !self.edit.focused {
                    self.edit.focus();
                }
                let metrics = FontMetrics(font);
                self.edit.mouse_down(x, &metrics);
                self.edit.layout(&metrics);
            }
            Some(ChatHit::List) => {
                self.list.mouse_down(x, y);
            }
            _ => {}
        }
        hit
    }

    pub fn mouse_move(&mut self, x: i32, y: i32, font: &BitmapFont) {
        if self.edit.focused {
            let metrics = FontMetrics(font);
            self.edit.mouse_move(x, &metrics);
            self.edit.layout(&metrics);
        }
        self.list.mouse_move(y);
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<ChatAction> {
        self.edit.mouse_up();
        self.list.mouse_up();
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        match (pressed, self.kind) {
            (ChatHit::Close, _) => Some(ChatAction::Close),
            (ChatHit::Button(index), ChatKind::Gm) => Some(ChatAction::Bypass(
                if index == 0 { START_COMMAND } else { END_COMMAND }.to_string(),
            )),
            (ChatHit::Button(0), ChatKind::Player) => {
                self.set_feedback_enabled(false);
                Some(ChatAction::Cancel)
            }
            (ChatHit::Button(_), ChatKind::Player) if self.buttons[1].enabled => Some(ChatAction::Feedback),
            _ => None,
        }
    }

    pub fn wheel(&mut self, delta: i32) {
        self.list.wheel(delta);
    }

    pub fn key(&mut self, key: crate::chat::InputKey, font: &BitmapFont) -> Option<Vec<ChatAction>> {
        if !self.edit.focused {
            return None;
        }
        if self.edit.input_key(key, &FontMetrics(font)) == crate::edit_box::EditInput::Escape {
            self.edit.kill_focus();
            return Some(Vec::new());
        }
        let entered = self.edit.notices.drain(..).any(|notice| notice.code == NOTICE_ENTER);
        Some(if entered { self.enter() } else { Vec::new() })
    }

    pub fn enter(&mut self) -> Vec<ChatAction> {
        let text = self.edit.get_text();
        let mut actions = Vec::new();
        if let Some(text) = text {
            match self.kind {
                ChatKind::Player => actions.push(ChatAction::Say { kind: SAY_PETITION_PLAYER, text }),
                ChatKind::Gm => match classify(&text) {
                    Command::Say => actions.push(ChatAction::Say { kind: SAY_PETITION_GM, text }),
                    Command::Build => actions.push(ChatAction::Bypass(text.chars().skip(2).collect())),
                    Command::Ignored => {}
                },
            }
        }
        self.edit.clear();
        actions
    }

    pub fn tick(&mut self, elapsed_ms: u32) {
        self.edit.tick(elapsed_ms);
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) {
        let (x, y) = origin;
        self.frame_contract.paint(&self.frame, list, origin, font, alpha);
        let start = list.quads.len();
        list.draw_texture(
            x + BACK_ORIGIN.0,
            y + BACK_ORIGIN.1,
            SIZE.0,
            SIZE.1 - BACK_ORIGIN.1,
            0,
            0,
            SIZE.0,
            SIZE.1 - BACK_ORIGIN.1,
            BACK,
            alpha,
        );
        list.clip_from(start, Rect::new(x, y, SIZE.0, SIZE.1));
        self.list.paint(list, origin, font);
        self.edit.paint(list, origin, &FontMetrics(font), font);
        for index in 0..2 {
            self.paint_button(index, list, origin, font);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), crate::font::Glyphs::parse(&crate::font::gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn player() -> PetitionChat {
        PetitionChat::player(|id| Some(format!("s{id}")))
    }

    fn gm() -> PetitionChat {
        PetitionChat::gm(|id| Some(format!("s{id}")))
    }

    #[test]
    fn the_layouts_match_the_on_create_snapshots() {
        let player = player();
        assert_eq!(SIZE, (310, 412));
        assert_eq!(player.list.rect, Rect::new(8, 37, 294, 312));
        assert_eq!(player.edit.rect, Rect::new(7, 357, 296, 17));
        assert_eq!(player.buttons[0].rect, Rect::new(80, 383, 76, 23));
        assert_eq!(player.buttons[1].rect, Rect::new(160, 383, 76, 23));
        assert_eq!(player.buttons[0].label, "s473");
        assert_eq!(player.buttons[1].label, "s849");
        assert!(!player.buttons[1].enabled);
        let gm = gm();
        assert_eq!(GM_SIZE, SIZE);
        assert_eq!(gm_grip(), grip());
        assert_eq!(gm.buttons[0].label, "s474");
        assert_eq!(gm.buttons[1].label, "s475");
        assert!(gm.buttons[0].enabled && gm.buttons[1].enabled);
        assert_eq!(player.edit.max_len, EDIT_MAX_LEN);
    }

    #[test]
    fn the_gm_position_restores_the_creation_coordinates() {
        let viewport = (1280, 800);
        let descriptor = gm_initial_position(viewport);
        let (x, y) = gm_default_position(viewport);
        assert_eq!((descriptor.horizontal_mode, descriptor.vertical_mode), (0, 5));
        assert_eq!(crate::initial_position::InitialAxis::horizontal(0, descriptor.x).unwrap().coordinate(viewport.0), x);
        assert_eq!(crate::initial_position::InitialAxis::vertical(5, descriptor.y).unwrap().coordinate(viewport.1), y);
        assert_eq!(player_position(viewport), (106, 208));
    }

    #[test]
    fn the_player_enter_sends_say_6_for_any_text_and_always_clears_the_box() {
        let mut chat = player();
        chat.edit.set_text("/hello");
        assert_eq!(chat.enter(), vec![ChatAction::Say { kind: 6, text: "/hello".into() }]);
        assert_eq!(chat.edit.get_text(), None);
        assert!(chat.enter().is_empty());
    }

    #[test]
    fn the_gm_enter_routes_plain_text_to_say_7_and_double_slash_to_the_build_command() {
        let mut chat = gm();
        chat.edit.set_text("hello");
        assert_eq!(chat.enter(), vec![ChatAction::Say { kind: 7, text: "hello".into() }]);
        chat.edit.set_text("//admin x");
        assert_eq!(chat.enter(), vec![ChatAction::Bypass("admin x".into())]);
        chat.edit.set_text("///admin");
        assert!(chat.enter().is_empty());
        chat.edit.set_text("/target");
        assert!(chat.enter().is_empty());
        chat.edit.set_text("//");
        assert!(chat.enter().is_empty());
        assert_eq!(chat.edit.get_text(), None);
    }

    #[test]
    fn classify_follows_the_command_type_rules() {
        assert_eq!(classify("hi"), Command::Say);
        assert_eq!(classify("//a"), Command::Build);
        assert_eq!(classify("//"), Command::Ignored);
        assert_eq!(classify("///"), Command::Build);
        assert_eq!(classify("///a"), Command::Ignored);
        assert_eq!(classify("/a"), Command::Ignored);
        assert_eq!(classify("\u{ff0f}a"), Command::Ignored);
    }

    #[test]
    fn player_messages_use_the_petition_color_and_gm_keeps_the_given_one() {
        let font = test_font();
        let mut chat = player();
        chat.add_message("a", Argb(0xff112233), 6, &font);
        chat.add_message("b", Argb(0xff112233), 7, &font);
        assert_eq!(chat.list.lines()[0].color, PLAYER_COLOR);
        assert_eq!(chat.list.lines()[1].color, Argb(0xff112233));
    }

    #[test]
    fn the_player_cancel_disables_feedback_and_the_feedback_button_needs_the_enable() {
        let mut chat = player();
        let feedback = chat.buttons[1].rect;
        let cancel = chat.buttons[0].rect;
        chat.mouse_down(feedback.x + 2, feedback.y + 2, &test_font());
        assert_eq!(chat.mouse_up(feedback.x + 2, feedback.y + 2), None);
        chat.set_feedback_enabled(true);
        chat.mouse_down(feedback.x + 2, feedback.y + 2, &test_font());
        assert_eq!(chat.mouse_up(feedback.x + 2, feedback.y + 2), Some(ChatAction::Feedback));
        chat.mouse_down(cancel.x + 2, cancel.y + 2, &test_font());
        assert_eq!(chat.mouse_up(cancel.x + 2, cancel.y + 2), Some(ChatAction::Cancel));
        assert!(!chat.feedback_enabled());
    }

    #[test]
    fn the_gm_buttons_send_the_consultation_commands() {
        let mut chat = gm();
        for (index, command) in [(0, "startpet"), (1, "endpet")] {
            let rect = chat.buttons[index].rect;
            chat.mouse_down(rect.x + 2, rect.y + 2, &test_font());
            assert_eq!(chat.mouse_up(rect.x + 2, rect.y + 2), Some(ChatAction::Bypass(command.into())));
        }
    }

    #[test]
    fn show_100e0290_moves_the_window_to_the_icon_and_hides_the_icon_only_when_the_icon_exists() {
        let mut icon = IconView::new(PLAYER_ID, "tex", "");
        icon.show_at(300, 200, false);
        let mut window = Rect::new(10, 20, 310, 412);
        show_over_icon(&mut window, Some(&mut icon));
        assert_eq!(window, Rect::new(300, 200, 310, 412));
        assert!(!icon.visible);
        assert_eq!((icon.rect.x, icon.rect.y, icon.rect.w, icon.rect.h), (300, 200, 32, 32));
        let mut untouched = Rect::new(10, 20, 310, 412);
        show_over_icon(&mut untouched, None);
        assert_eq!(untouched, Rect::new(10, 20, 310, 412));
    }

    #[test]
    fn the_console_registers_the_petition_windows_with_flags_2_so_ncwnd_creates_no_icon_10035f00() {
        assert_eq!(REGISTRATION_FLAGS, 2);
        assert!(!has_icon());
    }
}
