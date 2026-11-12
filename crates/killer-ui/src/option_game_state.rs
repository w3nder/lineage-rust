use crate::option_game::GamePage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    pub is_native: i32,
    pub my_name: i32,
    pub npc_name: i32,
    pub pledge_name: i32,
    pub party_name: i32,
    pub other_pc_name: i32,
    pub enter_chatting: i32,
    pub old_chatting: i32,
    pub party_looting: i32,
    pub screenshot_quality: i32,
    pub show_zone: i32,
    pub system_message: i32,
    pub system_damage: i32,
    pub system_consumables: i32,
    pub auto_tracking: i32,
    pub can_use_joystick: i32,
    pub using_joystick: i32,
    pub transparency: i32,
    pub arrow: i32,
    pub color_cursor: i32,
    pub use_key_crypt: i32,
}

pub trait GameReadBackend {
    fn read_bool(&mut self, file: &str, section: &str, key: &str) -> Option<i32>;
    fn read_int(&mut self, file: &str, section: &str, key: &str) -> Option<i32>;
    fn show_joystick_child(&mut self, shown: bool);
    fn key_security_active(&mut self) -> bool;
    fn key_security_available(&mut self) -> bool;
    fn set_console_looting(&mut self, selected: i32);
}

fn read_bool(backend: &mut impl GameReadBackend, key: &str, fallback: i32) -> i32 {
    backend.read_bool("Option.ini", "Game", key).unwrap_or(fallback)
}

impl GamePage {
    pub fn fill(&mut self, state: &mut GameState, backend: &mut impl GameReadBackend, enable_english: i32, lookup: impl Fn(u32) -> Option<String>) {
        let language = backend.read_int("localization.ini", "LanguageSet", "Language");
        self.append_choices(language, enable_english, lookup);
        self.read_settings(state, backend);
        self.set_checked(0xf0f01471, state.enter_chatting != 0);
        self.set_checked(0xf0f01472, state.old_chatting != 0);
        self.looting.selected = state.party_looting;
        backend.set_console_looting(self.looting.selected);
    }

    pub fn set_checked(&mut self, id: u32, value: bool) {
        if let Some(control) = self.checks.iter_mut().find(|control| control.id == id) {
            control.check.checked = value;
        }
    }

    pub fn set_enabled(&mut self, id: u32, value: bool) {
        if let Some(control) = self.checks.iter_mut().find(|control| control.id == id) {
            control.check.enabled = value;
        }
    }

    pub fn read_settings(&mut self, state: &mut GameState, backend: &mut impl GameReadBackend) {
        let native = read_bool(backend, "IsNative", 1);
        state.my_name = read_bool(backend, "MyName", 1);
        state.npc_name = read_bool(backend, "NPCName", 1);
        let group = read_bool(backend, "GroupName", 1);
        state.pledge_name = read_bool(backend, "PledgeMemberName", 1);
        state.party_name = read_bool(backend, "PartyMemberName", 1);
        state.other_pc_name = read_bool(backend, "OtherPCName", 1);
        state.enter_chatting = read_bool(backend, "EnterChatting", 0);
        state.old_chatting = read_bool(backend, "OldChatting", 1);
        state.party_looting = backend.read_int("Option.ini", "Game", "PartyLooting").unwrap_or(0);
        state.screenshot_quality = backend.read_int("Option.ini", "Game", "ScreenShotQuality").unwrap_or(0);
        state.show_zone = read_bool(backend, "ShowZoneTitle", 1);
        state.system_message = read_bool(backend, "SystemMsgWnd", 1);
        state.system_damage = read_bool(backend, "SystemMsgWndDamage", 1);
        state.system_consumables = read_bool(backend, "SystemMsgWndExpendableItem", 1);
        for (id, value) in [(0xf0f01461, group), (0xf0f01459, state.my_name), (0xf0f01460, state.npc_name), (0xf0f01462, state.pledge_name), (0xf0f01463, state.party_name), (0xf0f01464, state.other_pc_name)] {
            self.set_checked(id, value != 0);
        }
        self.message(0x501, 0xf0f01461, false);
        if group == 0 {
            state.pledge_name = 0;
            state.party_name = 0;
            state.other_pc_name = 0;
        }
        self.set_checked(0xf0f01470, state.auto_tracking != 0);
        self.language.selected = i32::from(native == 0);
        state.using_joystick = read_bool(backend, "UseJoystick", 1);
        if state.can_use_joystick == 0 {
            self.set_checked(0xf0f01473, true);
            self.set_enabled(0xf0f01473, false);
        }
        self.set_checked(0xf0f01473, true);
        backend.show_joystick_child(state.using_joystick != 0);
        self.set_checked(0xf0f01471, state.enter_chatting != 0);
        state.transparency = read_bool(backend, "TransparencyMode", 1);
        self.set_checked(0xf0f01475, state.transparency != 0);
        state.arrow = read_bool(backend, "ArrowMode", 1);
        for (id, value) in [(0xf0f01476, state.arrow), (0x1f0f, state.color_cursor), (0xf0f01478, state.show_zone), (0xf0f01479, state.system_message), (0xf0f01480, state.system_damage), (0xf0f01481, state.system_consumables)] {
            self.set_checked(id, value != 0);
        }
        self.set_checked(0xf0f01474, state.use_key_crypt != 0 && backend.key_security_active());
        self.set_enabled(0xf0f01474, state.use_key_crypt != 0 && backend.key_security_available());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct Backend {
        values: HashMap<String, i32>,
        reads: Vec<(String, String, String, bool)>,
        joystick: Vec<bool>,
        security: (bool, bool),
        security_calls: Vec<&'static str>,
        looting: Vec<i32>,
    }

    impl Backend {
        fn new(values: &[(&str, i32)]) -> Self {
            Self { values: values.iter().map(|(key, value)| ((*key).into(), *value)).collect(), reads: Vec::new(), joystick: Vec::new(), security: (false, false), security_calls: Vec::new(), looting: Vec::new() }
        }
    }

    impl GameReadBackend for Backend {
        fn read_bool(&mut self, file: &str, section: &str, key: &str) -> Option<i32> {
            self.reads.push((file.into(), section.into(), key.into(), false));
            self.values.get(key).copied()
        }
        fn read_int(&mut self, file: &str, section: &str, key: &str) -> Option<i32> {
            self.reads.push((file.into(), section.into(), key.into(), true));
            self.values.get(key).copied()
        }
        fn show_joystick_child(&mut self, shown: bool) { self.joystick.push(shown); }
        fn key_security_active(&mut self) -> bool { self.security_calls.push("active"); self.security.0 }
        fn key_security_available(&mut self) -> bool { self.security_calls.push("available"); self.security.1 }
        fn set_console_looting(&mut self, selected: i32) { self.looting.push(selected); }
    }

    fn state() -> GameState {
        GameState { is_native: 1, my_name: 7, npc_name: 7, pledge_name: 7, party_name: 7, other_pc_name: 7,
            enter_chatting: 7, old_chatting: 7, party_looting: 7, screenshot_quality: 7,
            show_zone: 7, system_message: 7, system_damage: 7, system_consumables: 7,
            auto_tracking: 0, can_use_joystick: 1, using_joystick: 7, transparency: 7,
            arrow: 7, color_cursor: 0, use_key_crypt: 0 }
    }

    fn check(page: &GamePage, id: u32) -> &crate::button::CheckBox {
        &page.checks.iter().find(|control| control.id == id).unwrap().check
    }

    #[test]
    fn game_read_100c7e90_defaults_query_order_and_globals_not_read_from_ini() {
        let mut page = GamePage::new(true, |_| None);
        page.looting.selected = 99;
        page.set_checked(0xf0f01472, false);
        let mut state = state();
        let mut backend = Backend::new(&[]);
        page.read_settings(&mut state, &mut backend);
        let keys = ["IsNative", "MyName", "NPCName", "GroupName", "PledgeMemberName", "PartyMemberName", "OtherPCName", "EnterChatting", "OldChatting", "PartyLooting", "ScreenShotQuality", "ShowZoneTitle", "SystemMsgWnd", "SystemMsgWndDamage", "SystemMsgWndExpendableItem", "UseJoystick", "TransparencyMode", "ArrowMode"];
        assert_eq!(backend.reads, keys.map(|key| ("Option.ini".into(), "Game".into(), key.into(), matches!(key, "PartyLooting" | "ScreenShotQuality"))));
        for value in [state.my_name, state.npc_name, state.pledge_name, state.party_name, state.other_pc_name, state.old_chatting, state.show_zone, state.system_message, state.system_damage, state.system_consumables, state.using_joystick, state.transparency, state.arrow] { assert_eq!(value, 1); }
        assert_eq!((state.enter_chatting, state.party_looting, state.screenshot_quality), (0, 0, 0));
        assert_eq!(page.language.selected, 0);
        assert_eq!(page.looting.selected, 99);
        assert!(!check(&page, 0xf0f01472).checked);
        assert!(!check(&page, 0xf0f01470).checked);
        assert!(!check(&page, 0x1f0f).checked);
        assert!(!check(&page, 0xf0f01474).checked && !check(&page, 0xf0f01474).enabled);
        assert!(backend.security_calls.is_empty() && backend.looting.is_empty());
        assert_eq!(backend.joystick, [true]);
    }

    #[test]
    fn game_read_100c7e90_preserves_child_choices_before_zeroing_effective_names() {
        for mask in 0..16 {
            let mut page = GamePage::new(false, |_| None);
            let mut state = state();
            let mut backend = Backend::new(&[("GroupName", mask & 1), ("PledgeMemberName", mask & 2), ("PartyMemberName", mask & 4), ("OtherPCName", mask & 8), ("SystemMsgWnd", 0)]);
            page.read_settings(&mut state, &mut backend);
            for (index, id) in [0xf0f01462, 0xf0f01463, 0xf0f01464].into_iter().enumerate() {
                assert_eq!(check(&page, id).checked, mask & (2 << index) != 0);
                assert_eq!(check(&page, id).enabled, mask & 1 != 0);
            }
            assert_eq!((state.pledge_name, state.party_name, state.other_pc_name), if mask & 1 == 0 { (0, 0, 0) } else { (mask & 2, mask & 4, mask & 8) });
            assert!(check(&page, 0xf0f01480).enabled && check(&page, 0xf0f01481).enabled);
        }
    }

    #[test]
    fn game_read_100c7e90_joystick_and_key_security_keep_independent_native_branches() {
        for can_use in [0, 1] {
            for using in [0, 1] {
                for crypt in [0, 1] {
                    for active in [false, true] {
                        for available in [false, true] {
                            let mut page = GamePage::new(true, |_| None);
                            let mut state = state();
                            state.can_use_joystick = can_use;
                            state.use_key_crypt = crypt;
                            let mut backend = Backend::new(&[("UseJoystick", using)]);
                            backend.security = (active, available);
                            page.read_settings(&mut state, &mut backend);
                            assert!(check(&page, 0xf0f01473).checked);
                            assert_eq!(check(&page, 0xf0f01473).enabled, can_use != 0);
                            assert_eq!(backend.joystick, [using != 0]);
                            assert_eq!(check(&page, 0xf0f01474).checked, crypt != 0 && active);
                            assert_eq!(check(&page, 0xf0f01474).enabled, crypt != 0 && available);
                            assert_eq!(backend.security_calls, if crypt != 0 { vec!["active", "available"] } else { vec![] });
                            state.can_use_joystick = 1;
                            page.read_settings(&mut state, &mut backend);
                            assert_eq!(check(&page, 0xf0f01473).enabled, can_use != 0);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn game_fill_100c9700_finishes_reading_chat_checks_and_raw_looting_index() {
        let mut page = GamePage::new(false, |id| Some(id.to_string()));
        let mut state = state();
        let mut backend = Backend::new(&[("Language", 2), ("IsNative", 0), ("EnterChatting", -1), ("OldChatting", 0), ("PartyLooting", 99)]);
        page.fill(&mut state, &mut backend, 1, |id| Some(id.to_string()));
        assert_eq!(backend.reads[0], ("localization.ini".into(), "LanguageSet".into(), "Language".into(), true));
        assert_eq!(page.language.entries, ["Japanese", "English"]);
        assert_eq!(page.language.selected, 1);
        assert_eq!(page.looting.entries, ["487", "488", "798", "799", "800"]);
        assert_eq!(page.looting.selected, 99);
        assert_eq!(backend.looting, [99]);
        assert!(check(&page, 0xf0f01471).checked);
        assert!(!check(&page, 0xf0f01472).checked);
    }

    #[test]
    fn game_show_100c98b0_clear_then_fill_does_not_duplicate_choices() {
        let mut page = GamePage::new(false, |_| None);
        let mut state = state();
        state.can_use_joystick = 0;
        let mut backend = Backend::new(&[("Language", 3), ("PartyLooting", 2)]);
        for _ in 0..3 {
            page.clear(state.can_use_joystick != 0);
            page.fill(&mut state, &mut backend, 1, |id| Some(id.to_string()));
            assert_eq!(page.language.entries, ["Chinese(Taiwan)", "English"]);
            assert_eq!(page.looting.entries, ["487", "488", "798", "799", "800"]);
            assert_eq!(page.looting.selected, 2);
        }
        assert_eq!(backend.looting, [2, 2, 2]);
    }
}
