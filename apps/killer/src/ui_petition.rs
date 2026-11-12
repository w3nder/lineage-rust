use killer_ui::chat::InputKey;
use killer_ui::color::Argb;
use killer_ui::font::BitmapFont;
use killer_ui::petition_begin::{self, BeginAction, Category, PetitionBegin};
use killer_ui::petition_chat::{self, ChatAction, ChatKind, PetitionChat};
use killer_ui::petition_feedback::{self, FeedbackAction, PetitionFeedback};
use killer_ui::window::{Desktop, WindowId};
use killer_world::{NetCommand, WorldEvent};

use crate::ui_keys::UiCommand;

pub const CHAT_COLOR: Argb = Argb(0xFF80FFFF);
pub const SAY_PETITION_PLAYER: i32 = 6;
pub const SAY_PETITION_GM: i32 = 7;
pub const SYSTEM_GROUP_PETITION: u32 = 6;
pub const SYSTEM_MESSAGE_KIND: i32 = 5;

pub struct PetitionWindows {
    pub begin: PetitionBegin,
    pub player: PetitionChat,
    pub gm: PetitionChat,
    pub feedback: PetitionFeedback,
    pub icon: Option<killer_ui::minimized::IconView>,
    pub commands: Vec<UiCommand>,
    pub notices: Vec<u32>,
    categories: Vec<Category>,
    last_tick: Option<killer_platform::time::Instant>,
    line_height: i32,
}

impl PetitionWindows {
    pub fn new(lookup: impl Fn(u32) -> Option<String> + Copy, line_height: i32, categories: Vec<Category>) -> Self {
        Self {
            begin: PetitionBegin::new(lookup, line_height, categories.clone()),
            player: PetitionChat::player(lookup),
            gm: PetitionChat::gm(lookup),
            feedback: PetitionFeedback::new(lookup, line_height),
            icon: petition_chat::has_icon().then(|| killer_ui::minimized::IconView::new(petition_chat::PLAYER_ID, "", "")),
            commands: Vec::new(),
            notices: Vec::new(),
            categories,
            last_tick: None,
            line_height,
        }
    }

    pub fn line_height(&self) -> i32 {
        self.line_height
    }

    pub fn set_font(&mut self, font: &BitmapFont) {
        self.line_height = font.line_height as i32;
        self.begin.combo.text_height = self.line_height;
        self.begin.edit.set_line_height(self.line_height);
        self.feedback.edit.set_line_height(self.line_height);
    }

    pub fn categories(&self) -> &[Category] {
        &self.categories
    }

    pub fn toggle_begin(&mut self, desktop: &mut Desktop) {
        if visible(desktop, petition_begin::ID) {
            self.begin.close();
            show(desktop, petition_begin::ID, false);
        } else {
            self.open_begin(None, desktop);
        }
    }

    pub fn open_begin(&mut self, preset: Option<(&str, &BitmapFont)>, desktop: &mut Desktop) {
        self.begin.open(preset);
        show(desktop, petition_begin::ID, true);
    }

    pub fn world_event(&mut self, event: &WorldEvent) -> bool {
        if matches!(event, WorldEvent::PetitionVoteNoop(_)) {
            self.player.set_feedback_enabled(true);
            return true;
        }
        false
    }

    pub fn show_player(&mut self, desktop: &mut Desktop) {
        show(desktop, petition_chat::PLAYER_ID, true);
        if let Some(window) = desktop.window_mut(petition_chat::PLAYER_ID) {
            petition_chat::show_over_icon(&mut window.rect, self.icon.as_mut());
        }
    }

    pub fn chat_message(&mut self, kind: i32, name: &str, text: &str, font: &BitmapFont, desktop: &mut Desktop) -> bool {
        if kind != SAY_PETITION_PLAYER && kind != SAY_PETITION_GM {
            return false;
        }
        let line = format!("{name}: {text}");
        self.gm.add_message(&line, CHAT_COLOR, kind, font);
        if !visible(desktop, petition_chat::GM_ID) {
            self.show_player(desktop);
            self.player.add_message(&line, CHAT_COLOR, kind, font);
        }
        true
    }

    pub fn system_message(&mut self, text: &str, font: &BitmapFont, desktop: &mut Desktop) {
        self.gm.add_message(text, CHAT_COLOR, SYSTEM_MESSAGE_KIND, font);
        if !visible(desktop, petition_chat::GM_ID) {
            self.show_player(desktop);
            self.player.add_message(text, CHAT_COLOR, SYSTEM_MESSAGE_KIND, font);
        }
    }

    pub fn mouse_down(&mut self, id: WindowId, x: i32, y: i32, font: &BitmapFont, desktop: &mut Desktop) -> bool {
        match id {
            petition_begin::ID => {
                if let Some(action) = self.begin.mouse_down(x, y) {
                    self.begin_action(action, font, desktop);
                }
            }
            petition_chat::PLAYER_ID => {
                self.player.mouse_down(x, y, font);
            }
            petition_chat::GM_ID => {
                self.gm.mouse_down(x, y, font);
            }
            petition_feedback::ID => {
                self.feedback.mouse_down(x, y);
            }
            _ => return false,
        }
        true
    }

    pub fn mouse_move(&mut self, id: WindowId, x: i32, y: i32, font: &BitmapFont) -> bool {
        match id {
            petition_begin::ID => self.begin.mouse_move(x, y),
            petition_chat::PLAYER_ID => self.player.mouse_move(x, y, font),
            petition_chat::GM_ID => self.gm.mouse_move(x, y, font),
            petition_feedback::ID => self.feedback.mouse_move(y),
            _ => return false,
        }
        true
    }

    pub fn mouse_up(&mut self, over: Option<(WindowId, i32, i32)>, font: &BitmapFont, desktop: &mut Desktop) {
        let outside = (i32::MIN / 2, i32::MIN / 2);
        let at = |id: WindowId| over.filter(|(over_id, _, _)| *over_id == id).map_or(outside, |(_, x, y)| (x, y));
        let (x, y) = at(petition_begin::ID);
        if let Some(action) = self.begin.mouse_up(x, y) {
            self.begin_action(action, font, desktop);
        }
        let (x, y) = at(petition_chat::PLAYER_ID);
        if let Some(action) = self.player.mouse_up(x, y) {
            self.chat_action(ChatKind::Player, action, desktop);
        }
        let (x, y) = at(petition_chat::GM_ID);
        if let Some(action) = self.gm.mouse_up(x, y) {
            self.chat_action(ChatKind::Gm, action, desktop);
        }
        let (x, y) = at(petition_feedback::ID);
        if let Some(action) = self.feedback.mouse_up(x, y) {
            self.feedback_action(action, desktop);
        }
    }

    pub fn wheel(&mut self, id: WindowId, x: i32, y: i32, delta: i32) -> bool {
        match id {
            petition_begin::ID => self.begin.wheel(x, y, delta),
            petition_chat::PLAYER_ID => {
                self.player.wheel(delta);
                true
            }
            petition_chat::GM_ID => {
                self.gm.wheel(delta);
                true
            }
            petition_feedback::ID => self.feedback.wheel(x, y, delta),
            _ => false,
        }
    }

    pub fn key(&mut self, key: InputKey, font: &BitmapFont, desktop: &mut Desktop) -> bool {
        if visible(desktop, petition_begin::ID) && self.begin.key(key, font) {
            return true;
        }
        if visible(desktop, petition_feedback::ID) && self.feedback.key(key, font) {
            return true;
        }
        if visible(desktop, petition_chat::PLAYER_ID)
            && let Some(actions) = self.player.key(key, font)
        {
            for action in actions {
                self.chat_action(ChatKind::Player, action, desktop);
            }
            return true;
        }
        if visible(desktop, petition_chat::GM_ID)
            && let Some(actions) = self.gm.key(key, font)
        {
            for action in actions {
                self.chat_action(ChatKind::Gm, action, desktop);
            }
            return true;
        }
        false
    }

    pub fn tick(&mut self, now: killer_platform::time::Instant) {
        if let Some(last) = self.last_tick.replace(now) {
            let elapsed = now.saturating_duration_since(last).as_millis().min(u128::from(u32::MAX)) as u32;
            self.begin.tick(elapsed);
            self.player.tick(elapsed);
            self.gm.tick(elapsed);
            self.feedback.tick(elapsed);
        }
    }

    fn begin_action(&mut self, action: BeginAction, font: &BitmapFont, desktop: &mut Desktop) {
        match action {
            BeginAction::Close => show(desktop, petition_begin::ID, false),
            BeginAction::Notice(id) => self.notices.push(id),
            BeginAction::LoadPage(path) => self.commands.push(UiCommand::LoadPetitionPage(path)),
            BeginAction::Cancel => {
                self.commands.push(UiCommand::Net(NetCommand::PetitionCancel { content: String::new() }));
                self.begin.close();
                show(desktop, petition_begin::ID, false);
            }
            BeginAction::Submit { text, kind } => {
                self.commands.push(UiCommand::Net(NetCommand::Petition { content: text.clone(), kind }));
                self.player.clear();
                let line = petition_begin::local_line(&self.begin.local_line_label, &text);
                self.player.add_message(&line, petition_begin::LOCAL_LINE_COLOR, 0, font);
                self.begin.close();
                show(desktop, petition_begin::ID, false);
            }
        }
    }

    fn chat_action(&mut self, kind: ChatKind, action: ChatAction, desktop: &mut Desktop) {
        match action {
            ChatAction::Say { kind, text } => {
                self.commands.push(UiCommand::Net(NetCommand::Say { text, kind, target: String::new() }));
            }
            ChatAction::Bypass(command) => self.commands.push(UiCommand::Net(NetCommand::BypassBuildCmd { command })),
            ChatAction::Cancel => self.commands.push(UiCommand::Net(NetCommand::PetitionCancel { content: String::new() })),
            ChatAction::Feedback => show(desktop, petition_feedback::ID, true),
            ChatAction::Close => match kind {
                ChatKind::Player => {
                    self.player.hide();
                    show(desktop, petition_chat::PLAYER_ID, false);
                }
                ChatKind::Gm => {
                    self.gm.hide();
                    show(desktop, petition_chat::GM_ID, false);
                }
            },
        }
    }

    fn feedback_action(&mut self, action: FeedbackAction, desktop: &mut Desktop) {
        if let FeedbackAction::Vote { first, index, text } = action {
            self.commands.push(UiCommand::Net(NetCommand::PetitionVote { first, index, text }));
        }
        self.player.set_feedback_enabled(false);
        show(desktop, petition_feedback::ID, false);
    }
}

fn visible(desktop: &Desktop, id: WindowId) -> bool {
    desktop.window(id).is_some_and(|window| window.visible)
}

fn show(desktop: &mut Desktop, id: WindowId, visible: bool) {
    if let Some(window) = desktop.window_mut(id) {
        window.visible = visible;
    }
    if visible && let Some(index) = desktop.windows.iter().position(|window| window.id == id) {
        let window = desktop.windows.remove(index);
        desktop.windows.push(window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::window::{Rect, Window};

    fn font() -> BitmapFont {
        BitmapFont::new(
            "L2Font-e.SmallFont-e".into(),
            killer_ui::font::Glyphs::parse(include_bytes!("../../../crates/killer-ui/tests/fixtures/smallfont-e.gly")).unwrap(),
        )
    }

    fn setup() -> (PetitionWindows, Desktop) {
        let mut desktop = Desktop::new((1024, 768));
        for id in [petition_begin::ID, petition_chat::PLAYER_ID, petition_chat::GM_ID, petition_feedback::ID] {
            let mut window = Window::new(id, Rect::new(0, 0, 300, 400), Rect::new(0, 0, 300, 20));
            window.visible = false;
            desktop.windows.push(window);
        }
        let categories = vec![Category { title: 696, link: "pet_help_move.htm".into() }];
        (PetitionWindows::new(|id| Some(format!("s{id}")), 13, categories), desktop)
    }

    #[test]
    fn a_chat_line_of_kind_6_opens_the_player_window_while_the_gm_one_is_hidden() {
        let (mut windows, mut desktop) = setup();
        let font = font();
        assert!(!windows.chat_message(1, "a", "b", &font, &mut desktop));
        assert!(windows.chat_message(6, "GM", "hello", &font, &mut desktop));
        assert!(visible(&desktop, petition_chat::PLAYER_ID));
        assert_eq!(windows.player.list.lines()[0].text, "GM: hello");
        assert_eq!(windows.gm.list.lines()[0].text, "GM: hello");
    }

    #[test]
    fn a_system_message_of_group_6_goes_to_the_petition_windows_with_the_color_kept_10052f90() {
        let (mut windows, mut desktop) = setup();
        windows.system_message("Petition accepted", &font(), &mut desktop);
        assert!(visible(&desktop, petition_chat::PLAYER_ID));
        assert_eq!(windows.gm.list.lines()[0].text, "Petition accepted");
        assert_eq!(windows.player.list.lines()[0].text, "Petition accepted");
        assert_eq!(windows.gm.list.lines()[0].color, CHAT_COLOR);
        assert_eq!(windows.player.list.lines()[0].color, CHAT_COLOR);
    }

    #[test]
    fn a_visible_gm_window_keeps_the_player_one_closed() {
        let (mut windows, mut desktop) = setup();
        show(&mut desktop, petition_chat::GM_ID, true);
        windows.chat_message(7, "p", "x", &font(), &mut desktop);
        assert!(!visible(&desktop, petition_chat::PLAYER_ID));
        assert_eq!(windows.player.list.lines().len(), 0);
        assert_eq!(windows.gm.list.lines().len(), 1);
    }

    #[test]
    fn the_empty_vote_packet_event_enables_the_feedback_button() {
        let (mut windows, _) = setup();
        assert!(!windows.player.feedback_enabled());
        assert!(windows.world_event(&WorldEvent::PetitionVoteNoop(l2_net::game::PetitionVoteNoop)));
        assert!(windows.player.feedback_enabled());
    }

    #[test]
    fn submitting_sends_the_petition_echoes_the_line_and_closes_the_begin_window() {
        let (mut windows, mut desktop) = setup();
        let font = font();
        windows.open_begin(Some(("help me please", &font)), &mut desktop);
        assert!(visible(&desktop, petition_begin::ID));
        windows.begin.combo.selected = 1;
        let action = windows.begin.submit();
        windows.begin_action(action, &font, &mut desktop);
        assert!(!visible(&desktop, petition_begin::ID));
        assert!(matches!(
            windows.commands.as_slice(),
            [UiCommand::Net(NetCommand::Petition { content, kind: 1 })] if content == "help me please"
        ));
        assert_eq!(windows.player.list.lines()[0].text, "s708 : help me please");
    }

    #[test]
    fn a_refused_petition_queues_the_notice_and_keeps_the_window_open() {
        let (mut windows, mut desktop) = setup();
        let font = font();
        windows.open_begin(None, &mut desktop);
        let action = windows.begin.submit();
        windows.begin_action(action, &font, &mut desktop);
        assert_eq!(windows.notices, vec![petition_begin::SELECT_TYPE_SYSMSG]);
        assert!(visible(&desktop, petition_begin::ID));
        assert!(windows.commands.is_empty());
    }

    #[test]
    fn the_vote_goes_out_and_closes_the_feedback_window() {
        let (mut windows, mut desktop) = setup();
        show(&mut desktop, petition_feedback::ID, true);
        windows.player.set_feedback_enabled(true);
        windows.feedback_action(FeedbackAction::Vote { first: 1, index: 3, text: "ok".into() }, &mut desktop);
        assert!(!visible(&desktop, petition_feedback::ID));
        assert!(!windows.player.feedback_enabled());
        assert!(matches!(
            windows.commands.as_slice(),
            [UiCommand::Net(NetCommand::PetitionVote { first: 1, index: 3, text })] if text == "ok"
        ));
    }

    #[test]
    fn the_system_menu_toggle_opens_and_closes_the_begin_window() {
        let (mut windows, mut desktop) = setup();
        windows.toggle_begin(&mut desktop);
        assert!(visible(&desktop, petition_begin::ID));
        assert_eq!(windows.begin.combo.selected, 0);
        windows.toggle_begin(&mut desktop);
        assert!(!visible(&desktop, petition_begin::ID));
    }
}
