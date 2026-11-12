use killer_assets::client::DatTables;
use killer_ui::item_grid::GridEntry;
use killer_ui::option_audio::{AudioApplyBackend, AudioChannel};
use killer_ui::option_game_apply::GameApplyBackend;
use killer_ui::shortcut::ShortcutView;
use killer_ui::window::{Desktop, WindowId};

use crate::ui_option_config::ConfigEdit;
use crate::ui_options::GameOptionRuntime;

pub const FIRST_RUN: (&str, &str, &str) = ("FirstRun", "FirstRun", "2");

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GameApplyEffects {
    pub clear_game_data: bool,
    pub load_argument: Option<i32>,
    pub invalidate_zone_name: bool,
    pub refresh_localized_windows: bool,
    pub reset_cursor: bool,
    pub system_message_shown: Option<bool>,
    pub rearranged_system_message: bool,
}

pub struct AbsentAudio;

impl AudioApplyBackend for AbsentAudio {
    fn available(&mut self) -> bool { false }
    fn volume(&mut self, channel: AudioChannel) -> f32 {
        log::error!("áudio: volume de {channel:?} pedido sem interface");
        0.0
    }
    fn set_volume(&mut self, channel: AudioChannel, _value: f32) {
        log::error!("áudio: volume de {channel:?} alterado sem interface");
    }
    fn stop(&mut self, channel: AudioChannel) {
        log::error!("áudio: parada de {channel:?} sem interface");
    }
    fn has_music_controller(&mut self) -> bool { false }
    fn reset_music_controller(&mut self) {}
    fn restore_lobby_music(&mut self) {}
    fn write_volume(&mut self, _file: &str, section: &str, key: &str, _value: f32) {
        log::error!("áudio: {section}/{key} gravado sem interface");
    }
}

pub struct GameApplyHost<'a> {
    pub runtime: &'a mut GameOptionRuntime,
    pub shortcut: &'a mut ShortcutView,
    pub edits: &'a mut Vec<ConfigEdit>,
    pub effects: &'a mut GameApplyEffects,
}

impl GameApplyHost<'_> {
    fn option_file(file: &str) -> bool {
        let ok = file.eq_ignore_ascii_case("Option.ini");
        if !ok { log::error!("Settings: gravação em arquivo inesperado {file}"); }
        ok
    }
}

impl GameApplyBackend for GameApplyHost<'_> {
    fn read_native(&mut self) -> Option<i32> {
        self.runtime.option.bool("Game", "IsNative").map(i32::from)
    }
    fn clear_game_data(&mut self) {
        self.effects.clear_game_data = true;
    }
    fn load_game_data(&mut self, argument: i32) {
        self.effects.load_argument = Some(argument);
    }
    fn invalidate_zone_name(&mut self) {
        self.effects.invalidate_zone_name = true;
    }
    fn refresh_localized_windows(&mut self) {
        self.effects.refresh_localized_windows = true;
    }
    fn write_bool(&mut self, file: &str, section: &str, key: &str, value: i32) {
        if Self::option_file(file) { self.edits.push(ConfigEdit::boolean(section, key, value)); }
    }
    fn write_int(&mut self, file: &str, section: &str, key: &str, value: i32) {
        if Self::option_file(file) { self.edits.push(ConfigEdit::new(section, key, value.to_string())); }
    }
    fn write_string(&mut self, file: &str, section: &str, key: &str, value: &str) {
        if Self::option_file(file) { self.edits.push(ConfigEdit::new(section, key, value)); }
    }
    fn console_looting(&mut self) -> i32 { self.runtime.console_looting }
    fn set_console_looting(&mut self, value: i32) { self.runtime.console_looting = value; }
    fn set_cursor(&mut self, _kind: i32, _icon: i32) {
        if !self.runtime.cursor_locked { self.effects.reset_cursor = true; }
    }
    fn lock_cursor_changes(&mut self) { self.runtime.cursor_locked = true; }
    fn show_system_message(&mut self, shown: bool) { self.effects.system_message_shown = Some(shown); }
    fn rearrange_system_message(&mut self) { self.effects.rearranged_system_message = true; }
    fn show_joystick_child(&mut self, shown: bool) { self.shortcut.joypad_visible = shown; }
    fn joystick_parent_mode(&mut self) -> i32 {
        log::error!("Settings: joystick sem dispositivo, campo +1f8 da barra não modelado");
        0
    }
    fn set_joystick_parent_mode(&mut self, mode: i32) {
        log::error!("Settings: joystick sem dispositivo, modo {mode} ignorado");
    }
    fn show_joystick_alternate_child(&mut self) {
        log::error!("Settings: joystick sem dispositivo, filho +254 não modelado");
    }
    fn refresh_joystick_parent(&mut self) {
        log::error!("Settings: joystick sem dispositivo, slot +12c não modelado");
    }
    fn set_key_security(&mut self, active: bool) {
        if let Some(security) = &mut self.runtime.key_security { security.active = active; }
    }
}

pub fn apply_system_message(desktop: &mut Desktop, effects: &GameApplyEffects) {
    if let Some(shown) = effects.system_message_shown && let Some(window) = desktop.window_mut(WindowId::SYSTEM_MSG) {
        window.visible = shown;
    }
}

pub fn relocalize_entry(grid: &mut GridEntry, tables: &DatTables) {
    let name = u32::try_from(grid.item_id).ok().and_then(|id| tables.itemname.get(&id));
    grid.name = name.map(|n| n.name.clone()).unwrap_or_default();
    grid.description = name.map(|n| n.description.clone()).unwrap_or_default();
}

pub fn relocalize_windows(keys: &mut crate::ui_keys::KeyWindows, tables: &DatTables) {
    for shop in [&mut keys.shop, &mut keys.warehouse] {
        for entry in shop.view.top.iter_mut().chain(shop.view.bottom.iter_mut()) {
            relocalize_entry(&mut entry.grid, tables);
        }
    }
    let trade = &mut keys.trade.view;
    for entry in trade.mine.iter_mut().chain(trade.offered.iter_mut()).chain(trade.theirs.iter_mut()) {
        relocalize_entry(&mut entry.grid, tables);
    }
    keys.relocalize_actions(tables);
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::ini::parse_ini;
    use killer_ui::option_game::GamePage;

    fn new_runtime(option: &str) -> GameOptionRuntime {
        GameOptionRuntime::new(parse_ini(option), parse_ini(""), &parse_ini(""), false, None)
    }

    fn set(page: &mut GamePage, id: u32, checked: bool) {
        page.checks.iter_mut().find(|c| c.id == id).unwrap().check.checked = checked;
    }

    #[test]
    fn game_apply_100c8460_host_writes_option_ini_in_original_order_and_requests_reload_on_language_change() {
        for (ini_native, selected, reload) in [(true, 0, false), (true, 1, true), (false, 0, true), (false, 1, false), (true, 7, false)] {
            let mut runtime = new_runtime(&format!("[Game]\nIsNative={ini_native}\n"));
            runtime.state.is_native = i32::from(ini_native);
            let mut page = GamePage::new(false, |id| Some(id.to_string()));
            page.language.entries = vec!["a".into(), "b".into()];
            page.language.selected = selected;
            set(&mut page, 0xf0f01461, false);
            set(&mut page, 0xf0f01464, true);
            set(&mut page, 0xf0f01479, true);
            set(&mut page, 0x1f0f, false);
            let mut shortcut = ShortcutView::new(false);
            let (mut edits, mut effects) = (Vec::new(), GameApplyEffects::default());
            let mut state = runtime.state.clone();
            page.apply(&mut state, &mut GameApplyHost { runtime: &mut runtime, shortcut: &mut shortcut, edits: &mut edits, effects: &mut effects });
            let keys: Vec<_> = edits.iter().map(|e| format!("{}/{}", e.section, e.key)).collect();
            assert_eq!(&keys[..3], ["Game/IsNative", "Game/MyName", "Game/NPCName"]);
            let position = |key: &str| keys.iter().position(|k| k == key).unwrap();
            assert!(position("Game/ArrowMode") < position("Video/UseColorCursor"));
            assert_eq!(position("Video/UseColorCursor") + 1, position("Game/ShowZoneTitle"));
            assert_eq!(keys.last().map(String::as_str), Some("Game/SystemMsgWndExpendableItem"));
            let edit = |key: &str| edits.iter().find(|e| e.key == key).unwrap().value.clone();
            assert_eq!(edit("GroupName"), "False");
            assert_eq!(edit("OtherPCName"), "True");
            assert_eq!(edit("UseColorCursor"), "false");
            assert_eq!(state.other_pc_name, 0);
            assert_eq!(effects.clear_game_data, reload);
            assert_eq!(effects.load_argument, reload.then_some(0));
            assert_eq!((effects.invalidate_zone_name, effects.refresh_localized_windows), (reload, reload));
            if reload { assert_eq!(state.is_native, i32::from(selected == 0)); }
            assert_eq!(effects.system_message_shown, Some(true));
            assert!(effects.rearranged_system_message);
            assert!(effects.reset_cursor);
            assert!(runtime.cursor_locked);
        }
    }

    #[test]
    fn game_apply_1006a420_cursor_is_not_reset_after_the_lock() {
        let mut runtime = new_runtime("");
        runtime.cursor_locked = true;
        let mut page = GamePage::new(false, |_| None);
        set(&mut page, 0x1f0f, false);
        let (mut edits, mut effects) = (Vec::new(), GameApplyEffects::default());
        let mut state = runtime.state.clone();
        page.apply(&mut state, &mut GameApplyHost { runtime: &mut runtime, shortcut: &mut ShortcutView::new(false), edits: &mut edits, effects: &mut effects });
        assert!(!effects.reset_cursor);
        let mut page = GamePage::new(false, |_| None);
        set(&mut page, 0x1f0f, true);
        let mut runtime = new_runtime("");
        let mut effects = GameApplyEffects::default();
        page.apply(&mut state, &mut GameApplyHost { runtime: &mut runtime, shortcut: &mut ShortcutView::new(false), edits: &mut Vec::new(), effects: &mut effects });
        assert!(!effects.reset_cursor);
        assert!(!runtime.cursor_locked);
    }

    #[test]
    fn audio_apply_100c7250_without_interface_changes_nothing() {
        let mut page = killer_ui::option_audio::AudioPage::new(|_| None);
        for combo in &mut page.combos { combo.selected = 5; }
        page.apply(&mut AbsentAudio);
    }

    #[test]
    fn refresh_1006a810_rewrites_name_and_description_but_not_add_name() {
        let mut tables = DatTables::default();
        tables.itemname.insert(57, l2_dat::itemname::ItemName {
            id: 57, name: "Adena".into(), add_name: "novo".into(), description: "moeda".into(), popup: 0
        });
        let mut grid = GridEntry { item_id: 57, name: "antigo".into(), add_name: "velho".into(), description: "x".into(), ..Default::default() };
        relocalize_entry(&mut grid, &tables);
        assert_eq!((grid.name.as_str(), grid.add_name.as_str(), grid.description.as_str()), ("Adena", "velho", "moeda"));
        let mut missing = GridEntry { item_id: 9, name: "antigo".into(), ..Default::default() };
        relocalize_entry(&mut missing, &tables);
        assert!(missing.name.is_empty() && missing.description.is_empty());
    }
}

#[cfg(test)]
mod client_tests {
    use super::*;
    use killer_ui::ini::parse_ini;

    #[test]
    #[ignore]
    fn game_apply_without_changes_rewrites_the_same_values_from_the_client_option_ini() {
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let mut client = killer_assets::client::Client::open(&root).unwrap();
        let text = client.read_system_text("option.ini").unwrap();
        let ini = parse_ini(&text);
        let mut ui = crate::ui2d::GameUi::load(&mut client, (1558, 933)).unwrap();
        ui.desktop.window_mut(WindowId::OPTION).unwrap().visible = true;
        ui.sync_windows(&killer_world::World::default(), None, std::time::Instant::now());
        let runtime = ui.option_game_runtime.as_mut().unwrap();
        let mut state = runtime.state.clone();
        let (mut edits, mut effects) = (Vec::new(), GameApplyEffects::default());
        ui.keys.option.game_page.apply(&mut state, &mut GameApplyHost { runtime, shortcut: &mut ui.shortcut, edits: &mut edits, effects: &mut effects });
        let mut differences = Vec::new();
        for edit in &edits {
            let before = ini.get(&edit.section, &edit.key);
            if before.map(|v| v.to_ascii_lowercase()) != Some(edit.value.to_ascii_lowercase()) {
                differences.push(format!("{}/{}: {:?} -> {}", edit.section, edit.key, before, edit.value));
            }
        }
        assert!(differences.is_empty(), "{differences:#?}");
        assert!(!effects.clear_game_data);
    }
}
