use crate::option_game::GamePage;
use crate::option_game_state::GameState;

pub trait GameApplyBackend {
    fn read_native(&mut self) -> Option<i32>;
    fn clear_game_data(&mut self);
    fn load_game_data(&mut self, argument: i32);
    fn invalidate_zone_name(&mut self);
    fn refresh_localized_windows(&mut self);
    fn write_bool(&mut self, file: &str, section: &str, key: &str, value: i32);
    fn write_int(&mut self, file: &str, section: &str, key: &str, value: i32);
    fn write_string(&mut self, file: &str, section: &str, key: &str, value: &str);
    fn console_looting(&mut self) -> i32;
    fn set_console_looting(&mut self, value: i32);
    fn set_cursor(&mut self, kind: i32, icon: i32);
    fn lock_cursor_changes(&mut self);
    fn show_system_message(&mut self, shown: bool);
    fn rearrange_system_message(&mut self);
    fn show_joystick_child(&mut self, shown: bool);
    fn joystick_parent_mode(&mut self) -> i32;
    fn set_joystick_parent_mode(&mut self, mode: i32);
    fn show_joystick_alternate_child(&mut self);
    fn refresh_joystick_parent(&mut self);
    fn set_key_security(&mut self, active: bool);
}

impl GamePage {
    pub fn apply(&mut self, state: &mut GameState, backend: &mut impl GameApplyBackend) {
        let old_native = backend.read_native().unwrap_or(1);
        match self.language.selected {
            0 => state.is_native = 1,
            1 => state.is_native = 0,
            _ => {}
        }
        if state.is_native != old_native {
            backend.clear_game_data();
            backend.load_game_data(0);
            backend.invalidate_zone_name();
            backend.refresh_localized_windows();
        }
        let checked = |id| i32::from(self.checks.iter().any(|control| control.id == id && control.check.checked));
        state.my_name = checked(0xf0f01459);
        state.npc_name = checked(0xf0f01460);
        let group = checked(0xf0f01461);
        state.pledge_name = group * checked(0xf0f01462);
        state.party_name = group * checked(0xf0f01463);
        state.other_pc_name = group * checked(0xf0f01464);
        state.transparency = checked(0xf0f01475);
        state.arrow = checked(0xf0f01476);
        state.auto_tracking = checked(0xf0f01470);
        state.enter_chatting = checked(0xf0f01471);
        state.old_chatting = checked(0xf0f01472);
        state.show_zone = checked(0xf0f01478);
        state.system_message = checked(0xf0f01479);
        state.system_damage = checked(0xf0f01480);
        state.system_consumables = checked(0xf0f01481);
        for (key, value) in [
            ("IsNative", state.is_native), ("MyName", state.my_name), ("NPCName", state.npc_name),
            ("GroupName", group), ("PledgeMemberName", checked(0xf0f01462)),
            ("PartyMemberName", checked(0xf0f01463)), ("OtherPCName", checked(0xf0f01464)),
            ("AutoTrackingPawn", state.auto_tracking), ("EnterChatting", state.enter_chatting),
            ("OldChatting", state.old_chatting), ("TransparencyMode", state.transparency),
            ("ArrowMode", state.arrow),
        ] {
            backend.write_bool("Option.ini", "Game", key, value);
        }
        if self.looting.enabled {
            backend.set_console_looting(self.looting.selected);
            state.party_looting = self.looting.selected;
            backend.write_int("Option.ini", "Game", "PartyLooting", state.party_looting);
        } else {
            self.looting.selected = backend.console_looting();
        }
        state.color_cursor = checked(0x1f0f);
        backend.write_string("Option.ini", "Video", "UseColorCursor", if state.color_cursor == 0 { "false" } else { "true" });
        if state.color_cursor == 0 {
            backend.set_cursor(0, 0);
            backend.lock_cursor_changes();
        }
        for (key, value) in [
            ("ShowZoneTitle", state.show_zone), ("SystemMsgWnd", state.system_message),
            ("SystemMsgWndDamage", state.system_damage), ("SystemMsgWndExpendableItem", state.system_consumables),
        ] {
            backend.write_bool("Option.ini", "Game", key, value);
        }
        backend.show_system_message(state.system_message != 0);
        backend.rearrange_system_message();
        if state.can_use_joystick != 0 {
            state.using_joystick = checked(0xf0f01473);
            backend.show_joystick_child(state.using_joystick != 0);
            if state.using_joystick == 0 && backend.joystick_parent_mode() == 1 {
                backend.set_joystick_parent_mode(0);
                backend.show_joystick_alternate_child();
                backend.refresh_joystick_parent();
            }
            backend.write_bool("Option.ini", "Game", "UseJoystick", state.using_joystick);
        }
        backend.set_key_security(state.use_key_crypt != 0 && checked(0xf0f01474) != 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Backend {
        events: Vec<String>,
        native: Option<i32>,
        looting: i32,
        mode: i32,
    }

    impl GameApplyBackend for Backend {
        fn read_native(&mut self) -> Option<i32> { self.events.push("read native".into()); self.native }
        fn clear_game_data(&mut self) { self.events.push("clear".into()); }
        fn load_game_data(&mut self, argument: i32) { self.events.push(format!("load {argument}")); }
        fn invalidate_zone_name(&mut self) { self.events.push("invalidate zone".into()); }
        fn refresh_localized_windows(&mut self) { self.events.push("refresh windows".into()); }
        fn write_bool(&mut self, file: &str, section: &str, key: &str, value: i32) {
            assert_eq!((file, section), ("Option.ini", "Game"));
            self.events.push(format!("bool {key} {value}"));
        }
        fn write_int(&mut self, file: &str, section: &str, key: &str, value: i32) {
            assert_eq!((file, section), ("Option.ini", "Game"));
            self.events.push(format!("int {key} {value}"));
        }
        fn write_string(&mut self, file: &str, section: &str, key: &str, value: &str) {
            assert_eq!((file, section), ("Option.ini", "Video"));
            self.events.push(format!("string {key} {value}"));
        }
        fn console_looting(&mut self) -> i32 { self.events.push("read looting".into()); self.looting }
        fn set_console_looting(&mut self, value: i32) { self.events.push(format!("looting {value}")); self.looting = value; }
        fn set_cursor(&mut self, kind: i32, icon: i32) { self.events.push(format!("cursor {kind} {icon}")); }
        fn lock_cursor_changes(&mut self) { self.events.push("lock cursor".into()); }
        fn show_system_message(&mut self, shown: bool) { self.events.push(format!("system {shown}")); }
        fn rearrange_system_message(&mut self) { self.events.push("rearrange".into()); }
        fn show_joystick_child(&mut self, shown: bool) { self.events.push(format!("joystick {shown}")); }
        fn joystick_parent_mode(&mut self) -> i32 { self.events.push("read mode".into()); self.mode }
        fn set_joystick_parent_mode(&mut self, mode: i32) { self.events.push(format!("mode {mode}")); self.mode = mode; }
        fn show_joystick_alternate_child(&mut self) { self.events.push("alternate".into()); }
        fn refresh_joystick_parent(&mut self) { self.events.push("refresh joystick".into()); }
        fn set_key_security(&mut self, active: bool) { self.events.push(format!("security {active}")); }
    }

    fn state() -> GameState {
        GameState {
            is_native: 1, my_name: 0, npc_name: 0, pledge_name: 0, party_name: 0,
            other_pc_name: 0, enter_chatting: 0, old_chatting: 0, party_looting: 7,
            screenshot_quality: 9, show_zone: 0, system_message: 0, system_damage: 0,
            system_consumables: 0, auto_tracking: 0, can_use_joystick: 1, using_joystick: 7,
            transparency: 0, arrow: 0, color_cursor: 0, use_key_crypt: 1,
        }
    }

    #[test]
    fn game_apply_100c8460_orders_all_writes_and_effects_without_erasing_child_choices() {
        let mut page = GamePage::new(true, |_| None);
        page.language.selected = 1;
        page.looting.selected = -1;
        page.set_checked(0xf0f01461, false);
        page.set_checked(0xf0f01473, false);
        let mut state = state();
        let mut backend = Backend { mode: 1, ..Backend::default() };
        page.apply(&mut state, &mut backend);
        assert_eq!(backend.events, [
            "read native", "clear", "load 0", "invalidate zone", "refresh windows",
            "bool IsNative 0", "bool MyName 1", "bool NPCName 1", "bool GroupName 0",
            "bool PledgeMemberName 1", "bool PartyMemberName 1", "bool OtherPCName 1",
            "bool AutoTrackingPawn 1", "bool EnterChatting 1", "bool OldChatting 1",
            "bool TransparencyMode 1", "bool ArrowMode 1", "looting -1", "int PartyLooting -1",
            "string UseColorCursor false", "cursor 0 0", "lock cursor", "bool ShowZoneTitle 1",
            "bool SystemMsgWnd 1", "bool SystemMsgWndDamage 1", "bool SystemMsgWndExpendableItem 1",
            "system true", "rearrange", "joystick false", "read mode", "mode 0", "alternate",
            "refresh joystick", "bool UseJoystick 0", "security true",
        ]);
        assert_eq!((state.pledge_name, state.party_name, state.other_pc_name), (0, 0, 0));
        assert_eq!((state.party_looting, state.screenshot_quality), (-1, 9));
        assert!(page.checks.iter().find(|c| c.id == 0xf0f01464).unwrap().check.checked);
    }

    #[test]
    fn game_apply_100c8460_language_compares_ini_and_preserves_invalid_selection() {
        for prior in [-1, 0, 1, 2] {
            for selected in [-1, 0, 1, 2] {
                for saved in [None, Some(0), Some(1), Some(2)] {
                    let mut page = GamePage::new(true, |_| None);
                    page.language.selected = selected;
                    page.language.enabled = false;
                    let mut state = state();
                    state.is_native = prior;
                    let mut backend = Backend { native: saved, ..Backend::default() };
                    page.apply(&mut state, &mut backend);
                    let expected = match selected { 0 => 1, 1 => 0, _ => prior };
                    assert_eq!(state.is_native, expected);
                    assert_eq!(backend.events.contains(&"clear".into()), expected != saved.unwrap_or(1));
                }
            }
        }
    }

    #[test]
    fn game_apply_100c8460_disabled_looting_restores_console_without_writing_global() {
        for enabled in [false, true] {
            for selected in [-1, 0, 4, 99] {
                let mut page = GamePage::new(true, |_| None);
                page.looting.enabled = enabled;
                page.looting.selected = selected;
                let mut state = state();
                let mut backend = Backend { looting: 2, ..Backend::default() };
                page.apply(&mut state, &mut backend);
                assert_eq!(page.looting.selected, if enabled { selected } else { 2 });
                assert_eq!(state.party_looting, if enabled { selected } else { 7 });
                assert_eq!(backend.events.iter().any(|e| e.starts_with("int PartyLooting")), enabled);
            }
        }
    }

    #[test]
    fn game_apply_100c8460_conditional_effects_include_disabled_checks() {
        for can_use in [0, 1] {
            for checked in [false, true] {
                for mode in [-1, 0, 1, 2] {
                    for crypt in [0, 1] {
                        let mut page = GamePage::new(true, |_| None);
                        for id in [0x1f0f, 0xf0f01473, 0xf0f01474, 0xf0f01479] {
                            page.set_checked(id, checked);
                            page.set_enabled(id, false);
                        }
                        let mut state = state();
                        state.can_use_joystick = can_use;
                        state.use_key_crypt = crypt;
                        let mut backend = Backend { mode, ..Backend::default() };
                        page.apply(&mut state, &mut backend);
                        assert_eq!(backend.events.contains(&"lock cursor".into()), !checked);
                        assert_eq!(backend.events.contains(&"alternate".into()), can_use != 0 && !checked && mode == 1);
                        assert_eq!(backend.events.contains(&"read mode".into()), can_use != 0 && !checked);
                        assert_eq!(backend.events.iter().any(|e| e.starts_with("bool UseJoystick")), can_use != 0);
                        assert_eq!(state.using_joystick, if can_use == 0 { 7 } else { i32::from(checked) });
                        assert!(backend.events.contains(&format!("system {checked}")));
                        assert_eq!(backend.events.last(), Some(&format!("security {}", crypt != 0 && checked)));
                    }
                }
            }
        }
    }
}
