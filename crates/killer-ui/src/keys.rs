pub const WM_KEYDOWN: u32 = 0x100;
pub const WM_KEYUP: u32 = 0x101;
pub const WM_CHAR: u32 = 0x102;
pub const WM_SYSKEYDOWN: u32 = 0x104;
pub const WM_SYSKEYUP: u32 = 0x105;
pub const WM_IME_STARTCOMPOSITION: u32 = 0x10d;
pub const WM_LBUTTONDOWN: u32 = 0x201;
pub const WM_IME_NOTIFY: u32 = 0x282;

pub const VK_TAB: u32 = 0x09;
pub const VK_RETURN: u32 = 0x0d;
pub const VK_SHIFT: u32 = 0x10;
pub const VK_CONTROL: u32 = 0x11;
pub const VK_MENU: u32 = 0x12;
pub const VK_ESCAPE: u32 = 0x1b;
pub const VK_PRIOR: u32 = 0x21;
pub const VK_NEXT: u32 = 0x22;
pub const VK_END: u32 = 0x23;
pub const VK_HOME: u32 = 0x24;
pub const VK_0: u32 = 0x30;
pub const VK_9: u32 = 0x39;
pub const VK_F1: u32 = 0x70;
pub const VK_F3: u32 = 0x72;
pub const VK_F10: u32 = 0x79;
pub const VK_F12: u32 = 0x7b;
pub const VK_OEM_PLUS: u32 = 0xbb;
pub const VK_OEM_MINUS: u32 = 0xbd;
pub const VK_OEM_3: u32 = 0xc0;

pub const F10_SCAN_DOWN: u32 = 0x44;
pub const F10_SCAN_REPEAT: u32 = 0x4044;
pub const F10_SCAN_UP: u32 = 0xc044;

pub const OBSERVER_NORMAL: i32 = 0;
pub const OBSERVER_OLYMPIAD: i32 = 1;
pub const GM_LEVEL_MIN: i32 = 1;
pub const GM_LEVEL_MAX: i32 = 6;

pub const SOUND_CLOSE: i32 = 0xc;
pub const SOUND_INVENTORY_OPEN: i32 = 0xd;
pub const SOUND_INVENTORY_CLOSE: i32 = 0xe;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MainTab {
    Status,
    Skills,
    Actions,
    Clan,
    Quests,
}

impl MainTab {
    pub const ALL: [MainTab; 5] = [MainTab::Status, MainTab::Skills, MainTab::Actions, MainTab::Clan, MainTab::Quests];

    pub fn index(self) -> usize {
        match self {
            MainTab::Status => 0,
            MainTab::Skills => 1,
            MainTab::Actions => 2,
            MainTab::Clan => 3,
            MainTab::Quests => 4,
        }
    }

    pub fn from_index(index: i32) -> Option<MainTab> {
        MainTab::ALL.get(usize::try_from(index).ok()?).copied()
    }

    pub fn title_sysstring(self) -> u32 {
        match self {
            MainTab::Status => 433,
            MainTab::Skills => 119,
            MainTab::Actions => 127,
            MainTab::Clan => 439,
            MainTab::Quests => 118,
        }
    }

    pub fn tooltip_sysstring(self) -> u32 {
        match self {
            MainTab::Status => 194,
            MainTab::Skills => 196,
            MainTab::Actions => 197,
            MainTab::Clan => 895,
            MainTab::Quests => 198,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OlympiadKey {
    G,
    I,
    L,
    M,
    N,
    P,
    J,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Effect {
    ChatTab(i32),
    Board,
    SelectTab(MainTab),
    ToggleMain(Option<MainTab>),
    GroupCompact,
    GmWindow,
    GmPetition,
    ToggleChat,
    HideUi,
    Olympiad(OlympiadKey),
    Minimap,
    MinFrame,
    Macros,
    Inventory,
    CloseAll,
    SystemMenu,
    Messenger,
    ShortcutPage(i32),
    TargetCancel,
    FocusChat,
    HideTooltips,
    ImeReset,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct KeyState {
    pub observer_mode: i32,
    pub enter_chatting: bool,
    pub language_korean: bool,
    pub dialog_visible: bool,
    pub chat_focused: bool,
    pub chat_locked: bool,
    pub in_game: bool,
    pub tab_blocked: bool,
    pub camera_fixed: bool,
    pub gm_level: i32,
    pub messenger: bool,
    pub tooltips: bool,
}

impl KeyState {
    fn gm(&self) -> bool {
        self.gm_level >= GM_LEVEL_MIN && self.gm_level <= GM_LEVEL_MAX
    }
}

pub trait ShortcutKeys {
    fn key_down(&mut self, vk: u32) -> bool;
    fn key_up(&mut self, vk: u32) -> bool;
}

pub struct NoShortcuts;

impl ShortcutKeys for NoShortcuts {
    fn key_down(&mut self, _: u32) -> bool {
        false
    }

    fn key_up(&mut self, _: u32) -> bool {
        false
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Hooked {
    pub effects: Vec<Effect>,
    pub handled: bool,
}

impl Hooked {
    fn push(&mut self, effect: Effect) {
        self.effects.push(effect);
    }
}

pub fn hook_shortcut_key(msg: u32, vk: u32, lparam: u32, state: &KeyState, bar: &mut dyn ShortcutKeys) -> Hooked {
    let mut out = Hooked::default();
    let mut msg = msg;
    if (msg == WM_SYSKEYDOWN || msg == WM_SYSKEYUP) && vk == VK_F10 {
        match lparam >> 16 {
            F10_SCAN_DOWN | F10_SCAN_REPEAT => msg = WM_KEYDOWN,
            F10_SCAN_UP => msg = WM_KEYUP,
            _ if msg == WM_SYSKEYUP => return out,
            _ => return alt_key(vk, state, out),
        }
        return plain_key(msg, vk, state, bar, out);
    }
    if msg > WM_SYSKEYDOWN {
        if msg != WM_IME_STARTCOMPOSITION {
            if msg == WM_LBUTTONDOWN {
                if state.tooltips {
                    out.push(Effect::HideTooltips);
                }
                return out;
            }
            if msg != WM_IME_NOTIFY {
                return out;
            }
        }
        if state.tooltips {
            out.push(Effect::HideTooltips);
        }
        if state.enter_chatting && state.language_korean {
            if !state.dialog_visible {
                out.push(Effect::ImeReset);
            }
            return out;
        }
        if !state.dialog_visible && !state.chat_focused {
            out.push(Effect::FocusChat);
        }
        return out;
    }
    if msg != WM_SYSKEYDOWN {
        return plain_key(msg, vk, state, bar, out);
    }
    alt_key(vk, state, out)
}

fn alt_key(vk: u32, state: &KeyState, mut out: Hooked) -> Hooked {
    if state.observer_mode != OBSERVER_NORMAL {
        if state.observer_mode == OBSERVER_OLYMPIAD && vk == 0x4a {
            out.push(Effect::ToggleChat);
        }
        return finish_alt(vk, state, out);
    }
    match vk {
        VK_PRIOR => out.push(Effect::ChatTab(0)),
        VK_NEXT => out.push(Effect::ChatTab(1)),
        0x42 => out.push(Effect::Board),
        0x43 => out.push(Effect::SelectTab(MainTab::Actions)),
        0x46 => out.push(Effect::GroupCompact),
        0x47 if state.gm() => out.push(Effect::GmWindow),
        0x48 => out.push(Effect::HideUi),
        0x49 => out.push(Effect::ToggleMain(None)),
        0x4a => out.push(Effect::ToggleChat),
        0x4b => out.push(Effect::SelectTab(MainTab::Skills)),
        0x4d => out.push(Effect::Minimap),
        0x4e => out.push(Effect::SelectTab(MainTab::Clan)),
        0x4f if state.gm() => out.push(Effect::GmPetition),
        0x50 => out.push(Effect::MinFrame),
        0x52 => out.push(Effect::Macros),
        0x54 => out.push(Effect::SelectTab(MainTab::Status)),
        0x55 => out.push(Effect::SelectTab(MainTab::Quests)),
        0x56 => out.push(Effect::Inventory),
        0x57 => out.push(Effect::CloseAll),
        0x58 => out.push(Effect::SystemMenu),
        0x59 if state.messenger => out.push(Effect::Messenger),
        VK_F1..=VK_F10 => out.push(Effect::ShortcutPage((vk - VK_F1) as i32)),
        _ => {}
    }
    finish_alt(vk, state, out)
}

fn finish_alt(vk: u32, state: &KeyState, mut out: Hooked) -> Hooked {
    if state.enter_chatting && (VK_0..=VK_9).contains(&vk) {
        let page = if vk == VK_0 { 9 } else { (vk - VK_0 - 1) as i32 };
        out.push(Effect::ShortcutPage(page));
    }
    out.handled = true;
    out
}

fn plain_key(msg: u32, vk: u32, state: &KeyState, bar: &mut dyn ShortcutKeys, mut out: Hooked) -> Hooked {
    match msg {
        WM_KEYDOWN => key_down(vk, state, bar, out),
        WM_KEYUP => {
            let used = bar.key_up(vk);
            out.handled = used || vk == VK_OEM_3;
            out
        }
        WM_CHAR => {
            focus_chat_if_typing(vk, state, &mut out);
            out
        }
        _ => out,
    }
}

fn key_down(vk: u32, state: &KeyState, bar: &mut dyn ShortcutKeys, mut out: Hooked) -> Hooked {
    let mut tooltips = state.tooltips;
    if bar.key_down(vk) {
        out.handled = true;
        return out;
    }
    if vk == VK_ESCAPE {
        if state.camera_fixed {
            return out;
        }
        out.push(Effect::TargetCancel);
        out.handled = true;
    } else if vk == VK_RETURN {
        if state.enter_chatting {
            if !state.chat_focused && !state.chat_locked && !state.dialog_visible {
                out.push(Effect::FocusChat);
                out.handled = true;
            }
        } else {
            hide_tooltips(&mut tooltips, &mut out);
            focus_chat_if_typing(vk, state, &mut out);
            return out;
        }
    } else if vk == VK_TAB && state.in_game {
        if state.tab_blocked {
            return out;
        }
        out.push(Effect::Inventory);
        tooltips = false;
        out.handled = true;
    }
    if !state.enter_chatting || state.dialog_visible {
        hide_tooltips(&mut tooltips, &mut out);
        focus_chat_if_typing(vk, state, &mut out);
        return out;
    }
    if state.observer_mode == OBSERVER_NORMAL {
        match vk {
            0x42 => out.push(Effect::Board),
            0x43 => out.push(Effect::ToggleMain(Some(MainTab::Actions))),
            0x46 => out.push(Effect::GroupCompact),
            0x49 => out.push(Effect::ToggleMain(None)),
            0x4b => out.push(Effect::ToggleMain(Some(MainTab::Skills))),
            0x4d => out.push(Effect::Minimap),
            0x4e => out.push(Effect::ToggleMain(Some(MainTab::Clan))),
            0x50 => out.push(Effect::MinFrame),
            0x52 => out.push(Effect::Macros),
            0x54 => out.push(Effect::ToggleMain(Some(MainTab::Status))),
            0x55 => out.push(Effect::ToggleMain(Some(MainTab::Quests))),
            0x56 => out.push(Effect::Inventory),
            0x58 => out.push(Effect::SystemMenu),
            0x59 => {
                if state.messenger {
                    out.push(Effect::Messenger);
                }
            }
            _ => {
                hide_tooltips(&mut tooltips, &mut out);
                focus_chat_if_typing(vk, state, &mut out);
                return out;
            }
        }
        out.handled = true;
    } else if state.observer_mode == OBSERVER_OLYMPIAD {
        let key = match vk {
            0x47 => Some(OlympiadKey::G),
            0x49 => Some(OlympiadKey::I),
            0x4c => Some(OlympiadKey::L),
            0x4d => Some(OlympiadKey::M),
            0x4e => Some(OlympiadKey::N),
            0x50 => Some(OlympiadKey::P),
            _ => None,
        };
        if let Some(key) = key {
            out.push(Effect::Olympiad(key));
        }
    }
    focus_chat_if_typing(vk, state, &mut out);
    out
}

fn hide_tooltips(tooltips: &mut bool, out: &mut Hooked) {
    if std::mem::take(tooltips) {
        out.push(Effect::HideTooltips);
    }
}

fn focus_chat_if_typing(vk: u32, state: &KeyState, out: &mut Hooked) {
    if state.enter_chatting
        || vk > 0x7f
        || state.dialog_visible
        || state.chat_focused
        || matches!(vk, VK_TAB | VK_CONTROL | VK_ESCAPE | VK_MENU)
    {
        return;
    }
    out.push(Effect::FocusChat);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Bar {
        consumed: Vec<u32>,
        downs: Vec<u32>,
        ups: Vec<u32>,
    }

    impl Bar {
        fn new() -> Bar {
            Bar { consumed: Vec::new(), downs: Vec::new(), ups: Vec::new() }
        }
    }

    impl ShortcutKeys for Bar {
        fn key_down(&mut self, vk: u32) -> bool {
            self.downs.push(vk);
            self.consumed.contains(&vk)
        }

        fn key_up(&mut self, vk: u32) -> bool {
            self.ups.push(vk);
            self.consumed.contains(&vk)
        }
    }

    fn game() -> KeyState {
        KeyState { in_game: true, ..KeyState::default() }
    }

    fn alt(vk: u32, state: &KeyState) -> Hooked {
        hook_shortcut_key(WM_SYSKEYDOWN, vk, 0x2000_0000, state, &mut NoShortcuts)
    }

    fn down(vk: u32, state: &KeyState) -> Hooked {
        hook_shortcut_key(WM_KEYDOWN, vk, 0, state, &mut NoShortcuts)
    }

    #[test]
    fn alt_letters_follow_the_syskeydown_jump_table() {
        let state = game();
        let table = [
            (0x54, Effect::SelectTab(MainTab::Status)),
            (0x4b, Effect::SelectTab(MainTab::Skills)),
            (0x43, Effect::SelectTab(MainTab::Actions)),
            (0x4e, Effect::SelectTab(MainTab::Clan)),
            (0x55, Effect::SelectTab(MainTab::Quests)),
            (0x49, Effect::ToggleMain(None)),
            (0x56, Effect::Inventory),
            (0x42, Effect::Board),
            (0x52, Effect::Macros),
            (0x58, Effect::SystemMenu),
            (0x4d, Effect::Minimap),
            (0x48, Effect::HideUi),
            (0x57, Effect::CloseAll),
            (0x46, Effect::GroupCompact),
            (0x50, Effect::MinFrame),
            (0x4a, Effect::ToggleChat),
            (VK_PRIOR, Effect::ChatTab(0)),
            (VK_NEXT, Effect::ChatTab(1)),
        ];
        for (vk, effect) in table {
            let hooked = alt(vk, &state);
            assert_eq!(hooked.effects, vec![effect.clone()], "vk {vk:#x}");
            assert!(hooked.handled);
        }
        assert!(alt(0x41, &state).effects.is_empty());
        assert!(alt(0x41, &state).handled);
    }

    #[test]
    fn alt_f_keys_change_the_page_and_digits_only_with_enter_chatting() {
        let state = game();
        assert_eq!(alt(VK_F1, &state).effects, vec![Effect::ShortcutPage(0)]);
        assert_eq!(alt(VK_F10, &state).effects, vec![Effect::ShortcutPage(9)]);
        assert!(alt(VK_F12, &state).effects.is_empty());
        assert!(alt(0x31, &state).effects.is_empty());
        let chatting = KeyState { enter_chatting: true, ..game() };
        assert_eq!(alt(0x31, &chatting).effects, vec![Effect::ShortcutPage(0)]);
        assert_eq!(alt(0x39, &chatting).effects, vec![Effect::ShortcutPage(8)]);
        assert_eq!(alt(VK_0, &chatting).effects, vec![Effect::ShortcutPage(9)]);
    }

    #[test]
    fn alt_y_g_o_depend_on_messenger_and_gm_level() {
        let state = game();
        assert!(alt(0x59, &state).effects.is_empty());
        assert_eq!(alt(0x59, &KeyState { messenger: true, ..state }).effects, vec![Effect::Messenger]);
        assert!(alt(0x47, &state).effects.is_empty());
        assert_eq!(alt(0x47, &KeyState { gm_level: 3, ..state }).effects, vec![Effect::GmWindow]);
        assert_eq!(alt(0x4f, &KeyState { gm_level: 6, ..state }).effects, vec![Effect::GmPetition]);
        assert!(alt(0x4f, &KeyState { gm_level: 7, ..state }).effects.is_empty());
    }

    #[test]
    fn the_olympiad_observer_only_answers_alt_j() {
        let observer = KeyState { observer_mode: OBSERVER_OLYMPIAD, ..game() };
        assert!(alt(0x54, &observer).effects.is_empty());
        assert!(alt(0x54, &observer).handled);
        assert_eq!(alt(0x4a, &observer).effects, vec![Effect::ToggleChat]);
        let plain = KeyState { enter_chatting: true, ..observer };
        assert_eq!(down(0x47, &plain).effects, vec![Effect::Olympiad(OlympiadKey::G)]);
        assert_eq!(down(0x50, &plain).effects, vec![Effect::Olympiad(OlympiadKey::P)]);
        assert!(!down(0x50, &plain).handled);
    }

    #[test]
    fn f10_without_alt_is_a_plain_key() {
        let state = game();
        let mut bar = Bar::new();
        let hooked = hook_shortcut_key(WM_SYSKEYDOWN, VK_F10, F10_SCAN_DOWN << 16, &state, &mut bar);
        assert_eq!(bar.downs, vec![VK_F10]);
        assert_eq!(hooked.effects, vec![Effect::FocusChat]);
        let mut bar = Bar::new();
        hook_shortcut_key(WM_SYSKEYUP, VK_F10, F10_SCAN_UP << 16, &state, &mut bar);
        assert_eq!(bar.ups, vec![VK_F10]);
        let mut bar = Bar::new();
        let repeat = hook_shortcut_key(WM_SYSKEYDOWN, VK_F10, F10_SCAN_REPEAT << 16, &state, &mut bar);
        assert_eq!(bar.downs, vec![VK_F10]);
        assert!(!repeat.handled);
        let mut bar = Bar::new();
        assert_eq!(
            hook_shortcut_key(WM_SYSKEYDOWN, VK_F10, 0x2044 << 16, &state, &mut bar).effects,
            vec![Effect::ShortcutPage(9)]
        );
        assert!(bar.downs.is_empty());
        let released = hook_shortcut_key(WM_SYSKEYUP, VK_F10, 0xe044 << 16, &state, &mut bar);
        assert!(!released.handled && released.effects.is_empty() && bar.ups.is_empty());
    }

    #[test]
    fn f_keys_go_to_the_shortcut_bar_first_and_a_repeat_is_swallowed() {
        let state = game();
        let mut bar = Bar::new();
        bar.consumed.push(VK_F1);
        let hooked = hook_shortcut_key(WM_KEYDOWN, VK_F1, 0, &state, &mut bar);
        assert!(hooked.handled);
        assert!(hooked.effects.is_empty());
        let mut bar = Bar::new();
        let hooked = hook_shortcut_key(WM_KEYDOWN, VK_F1, 0, &state, &mut bar);
        assert!(!hooked.handled);
        assert_eq!(hooked.effects, vec![Effect::FocusChat]);
        assert_eq!(bar.downs, vec![VK_F1]);
        let mut bar = Bar::new();
        bar.consumed.push(VK_F1);
        assert!(hook_shortcut_key(WM_KEYUP, VK_F1, 0, &state, &mut bar).handled);
        assert!(!hook_shortcut_key(WM_KEYUP, 0x41, 0, &state, &mut bar).handled);
        assert!(hook_shortcut_key(WM_KEYUP, VK_OEM_3, 0, &state, &mut bar).handled);
    }

    #[test]
    fn escape_cancels_the_target_unless_camera_fixed() {
        let state = game();
        let hooked = down(VK_ESCAPE, &state);
        assert_eq!(hooked.effects, vec![Effect::TargetCancel]);
        assert!(hooked.handled);
        let fixed = down(VK_ESCAPE, &KeyState { camera_fixed: true, ..state });
        assert!(fixed.effects.is_empty());
        assert!(!fixed.handled);
    }

    #[test]
    fn tab_opens_the_inventory_only_in_game_and_when_no_store_blocks_it() {
        let hooked = down(VK_TAB, &game());
        assert_eq!(hooked.effects, vec![Effect::Inventory]);
        assert!(hooked.handled);
        assert!(down(VK_TAB, &KeyState::default()).effects.is_empty());
        let blocked = down(VK_TAB, &KeyState { tab_blocked: true, ..game() });
        assert!(blocked.effects.is_empty());
        assert!(!blocked.handled);
    }

    #[test]
    fn without_enter_chatting_any_typing_key_focuses_the_chat() {
        let state = game();
        let hooked = down(0x41, &state);
        assert_eq!(hooked.effects, vec![Effect::FocusChat]);
        assert!(!hooked.handled);
        assert_eq!(down(VK_RETURN, &state).effects, vec![Effect::FocusChat]);
        assert!(down(0x41, &KeyState { chat_focused: true, ..state }).effects.is_empty());
        assert!(down(0x41, &KeyState { dialog_visible: true, ..state }).effects.is_empty());
        assert!(down(VK_CONTROL, &state).effects.is_empty());
        assert!(down(VK_MENU, &state).effects.is_empty());
        assert!(down(0x91, &state).effects.is_empty());
        assert_eq!(hook_shortcut_key(WM_CHAR, 0x61, 0, &state, &mut NoShortcuts).effects, vec![Effect::FocusChat]);
        let with_tooltip = down(0x41, &KeyState { tooltips: true, ..state });
        assert_eq!(with_tooltip.effects, vec![Effect::HideTooltips, Effect::FocusChat]);
    }

    #[test]
    fn with_enter_chatting_the_letters_toggle_windows_and_enter_focuses_the_chat() {
        let state = KeyState { enter_chatting: true, ..game() };
        assert_eq!(down(0x54, &state).effects, vec![Effect::ToggleMain(Some(MainTab::Status))]);
        assert_eq!(down(0x43, &state).effects, vec![Effect::ToggleMain(Some(MainTab::Actions))]);
        assert_eq!(down(0x56, &state).effects, vec![Effect::Inventory]);
        assert_eq!(down(0x58, &state).effects, vec![Effect::SystemMenu]);
        assert!(down(0x54, &state).handled);
        assert!(down(0x41, &state).effects.is_empty());
        assert!(!down(0x41, &state).handled);
        let enter = down(VK_RETURN, &state);
        assert_eq!(enter.effects, vec![Effect::FocusChat]);
        assert!(enter.handled);
        assert!(down(VK_RETURN, &KeyState { chat_focused: true, ..state }).effects.is_empty());
        assert!(down(VK_RETURN, &KeyState { chat_locked: true, ..state }).effects.is_empty());
        assert!(down(0x54, &KeyState { dialog_visible: true, ..state }).effects.is_empty());
        let escape = down(VK_ESCAPE, &state);
        assert_eq!(escape.effects, vec![Effect::TargetCancel]);
    }

    #[test]
    fn mouse_and_ime_messages_hide_tooltips_and_focus_the_chat() {
        let state = KeyState { tooltips: true, ..game() };
        assert_eq!(
            hook_shortcut_key(WM_LBUTTONDOWN, 0, 0, &state, &mut NoShortcuts).effects,
            vec![Effect::HideTooltips]
        );
        assert_eq!(
            hook_shortcut_key(WM_IME_NOTIFY, 0, 0, &state, &mut NoShortcuts).effects,
            vec![Effect::HideTooltips, Effect::FocusChat]
        );
        let korean = KeyState { enter_chatting: true, language_korean: true, ..game() };
        assert_eq!(
            hook_shortcut_key(WM_IME_STARTCOMPOSITION, 0, 0, &korean, &mut NoShortcuts).effects,
            vec![Effect::ImeReset]
        );
        assert!(hook_shortcut_key(0x200, 0, 0, &state, &mut NoShortcuts).effects.is_empty());
    }

    #[test]
    fn main_tab_titles_and_tooltips_are_the_sysstrings_of_select_tab() {
        assert_eq!(MainTab::ALL.map(MainTab::title_sysstring), [433, 119, 127, 439, 118]);
        assert_eq!(MainTab::ALL.map(MainTab::tooltip_sysstring), [194, 196, 197, 895, 198]);
        assert_eq!(MainTab::from_index(2), Some(MainTab::Actions));
        assert_eq!(MainTab::from_index(5), None);
        assert_eq!(MainTab::from_index(-1), None);
        assert_eq!(MainTab::Quests.index(), 4);
    }
}
