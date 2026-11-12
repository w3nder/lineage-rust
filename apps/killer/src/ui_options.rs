use killer_ui::ini::{IniFile, Options};
use killer_ui::option_game::GamePage;
use killer_ui::option_game_state::{GameReadBackend, GameState};
use killer_ui::shortcut::ShortcutView;
#[cfg(test)]
use killer_render::display_modes::{DisplayMode, refresh_rates, resolutions};
#[cfg(test)]
use killer_ui::option_video::VideoPage;

pub fn synchronize_game_options(ui: &mut crate::ui2d::GameUi, option: IniFile) {
    let options = Options::from_ini(&option);
    ui.keys.enter_chatting = options.enter_chatting;
    ui.chat.old_chat = options.old_chatting;
    ui.system_msg.options = options.system_msg;
    if let Some(runtime) = &mut ui.option_game_runtime {
        runtime.option = option;
        runtime.state.my_name = i32::from(options.show_my_name);
        runtime.state.npc_name = i32::from(options.show_npc_name);
        runtime.state.pledge_name = i32::from(options.show_group_name && options.show_pledge_member_name);
        runtime.state.party_name = i32::from(options.show_group_name && options.show_party_member_name);
        runtime.state.other_pc_name = i32::from(options.show_group_name && options.show_other_pc_name);
        runtime.state.enter_chatting = i32::from(options.enter_chatting);
        runtime.state.show_zone = i32::from(options.show_zone_title);
        runtime.state.system_message = i32::from(options.system_msg.shown);
        runtime.state.system_damage = i32::from(options.system_msg.damage);
        runtime.state.system_consumables = i32::from(options.system_msg.expendable_items);
        runtime.state.auto_tracking = i32::from(options.auto_tracking_pawn);
        runtime.state.transparency = i32::from(options.transparency);
        runtime.state.arrow = i32::from(options.arrow_mode);
    }
    ui.options = options;
}

pub fn synchronize_name_options(display: &mut crate::characters::DisplaySettings, options: &Options) {
    display.show_my_name = options.show_my_name;
    display.show_npc_name = options.show_npc_name;
    display.show_pledge_member_name = options.show_group_name && options.show_pledge_member_name;
    display.show_party_member_name = options.show_group_name && options.show_party_member_name;
    display.show_other_pc_name = options.show_group_name && options.show_other_pc_name;
}

#[cfg(test)]
struct DisplayOptions {
    pub modes: Vec<DisplayMode>,
    pub current_mode: Option<(i32, i32, i32)>,
    pub current_refresh: Option<i32>,
}

#[cfg(test)]
impl DisplayOptions {
    pub fn fill(&self, page: &mut VideoPage) {
        page.combos[0].clear_entries();
        page.combos[1].clear_entries();
        for (index, mode) in resolutions(&self.modes, 100).into_iter().take(100).enumerate() {
            page.combos[0].entries.push(format!("{}*{} {}bit", mode.0, mode.1, mode.2));
            if Some(mode) == self.current_mode {
                page.combos[0].selected = index as i32;
            }
        }
        if let Some(mode) = self.current_mode {
            for (index, rate) in refresh_rates(&self.modes, mode, 100).into_iter().take(100).enumerate() {
                page.combos[1].entries.push(format!("{rate}Hz"));
                if Some(rate) == self.current_refresh {
                    page.combos[1].selected = index as i32;
                }
            }
        }
    }

    pub fn resolution_changed(&self, page: &mut VideoPage, selection: i32) -> bool {
        let modes = resolutions(&self.modes, 100);
        let Some(mode) = usize::try_from(selection).ok().filter(|index| *index < 100).and_then(|index| modes.get(index)).copied() else { return true };
        let rates = refresh_rates(&self.modes, mode, 100);
        let known_rate = usize::try_from(page.combos[1].selected).ok().and_then(|index| rates.get(index)).copied();
        let Some(current_refresh) = self.current_refresh.or(known_rate) else { return false };
        page.resolution_changed(selection, &modes, current_refresh, |_| rates);
        true
    }
}

pub struct KeySecurityState {
    pub active: bool,
    pub available: bool,
}

pub struct GameOptionRuntime {
    pub option: IniFile,
    localization: IniFile,
    pub state: GameState,
    pub console_looting: i32,
    pub key_security: Option<KeySecurityState>,
    pub cursor_locked: bool,
}

impl GameOptionRuntime {
    pub fn new(option: IniFile, localization: IniFile, system: &IniFile, joystick_available: bool, key_security: Option<KeySecurityState>) -> Self {
        let current = Options::from_ini(&option);
        let boolean = |section, key, fallback| i32::from(option.bool(section, key).unwrap_or(fallback));
        let state = GameState {
            is_native: boolean("Game", "IsNative", true),
            my_name: i32::from(current.show_my_name),
            npc_name: i32::from(current.show_npc_name),
            pledge_name: i32::from(current.show_pledge_member_name),
            party_name: i32::from(current.show_party_member_name),
            other_pc_name: i32::from(current.show_other_pc_name),
            enter_chatting: i32::from(current.enter_chatting),
            old_chatting: boolean("Game", "OldChatting", true),
            party_looting: option.int("Game", "PartyLooting").unwrap_or(0),
            screenshot_quality: option.int("Game", "ScreenShotQuality").unwrap_or(0),
            show_zone: i32::from(current.show_zone_title),
            system_message: i32::from(current.system_msg.shown),
            system_damage: i32::from(current.system_msg.damage),
            system_consumables: i32::from(current.system_msg.expendable_items),
            auto_tracking: i32::from(current.auto_tracking_pawn),
            can_use_joystick: i32::from(joystick_available),
            using_joystick: boolean("Game", "UseJoystick", true),
            transparency: i32::from(current.transparency),
            arrow: i32::from(current.arrow_mode),
            color_cursor: boolean("Video", "UseColorCursor", true),
            use_key_crypt: i32::from(system.bool("URL", "L2UseKeyCrypt").unwrap_or(false)),
        };
        Self { console_looting: state.party_looting, option, localization, state, key_security, cursor_locked: false }
    }

    pub fn show(&mut self, page: &mut GamePage, shortcut: &mut ShortcutView, lookup: impl Fn(u32) -> Option<String>) {
        let enable_english = i32::from(self.localization.bool("LanguageSet", "EnableEngSelection").unwrap_or(false));
        let mut backend = GameIniReader {
            option: &self.option, localization: &self.localization, shortcut,
            console_looting: &mut self.console_looting, security: self.key_security.as_ref(),
        };
        page.clear(self.state.can_use_joystick != 0);
        page.fill(&mut self.state, &mut backend, enable_english, lookup);
    }
}

struct GameIniReader<'a> {
    option: &'a IniFile,
    localization: &'a IniFile,
    shortcut: &'a mut ShortcutView,
    console_looting: &'a mut i32,
    security: Option<&'a KeySecurityState>,
}

impl GameIniReader<'_> {
    fn file(&self, name: &str) -> Option<&IniFile> {
        if name.eq_ignore_ascii_case("Option.ini") { Some(self.option) }
        else if name.eq_ignore_ascii_case("localization.ini") { Some(self.localization) }
        else { None }
    }
}

impl GameReadBackend for GameIniReader<'_> {
    fn read_bool(&mut self, file: &str, section: &str, key: &str) -> Option<i32> {
        self.file(file)?.bool(section, key).map(i32::from)
    }
    fn read_int(&mut self, file: &str, section: &str, key: &str) -> Option<i32> {
        self.file(file)?.int(section, key)
    }
    fn show_joystick_child(&mut self, shown: bool) { self.shortcut.joypad_visible = shown; }
    fn key_security_active(&mut self) -> bool { self.security.is_some_and(|security| security.active) }
    fn key_security_available(&mut self) -> bool { self.security.is_some_and(|security| security.available) }
    fn set_console_looting(&mut self, selected: i32) { *self.console_looting = selected; }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::ini::parse_ini;

    #[test]
    fn game_apply_100c8460_synchronizes_consumers_without_erasing_saved_children() {
        use killer_ui::font::{BitmapFont, Glyphs};
        let font = BitmapFont::new(String::new(), Glyphs {
            tex_width: 1, tex_height: 1, glyphs: Default::default(),
        });
        let mut ui = crate::ui2d::GameUi::new(font.clone(), font, Options::default(),
            Default::default(), killer_ui::ini::ChatFilter::from_ini(&parse_ini("")), (1558, 933)).unwrap();
        ui.option_game_runtime = Some(GameOptionRuntime::new(parse_ini(""), parse_ini(""), &parse_ini(""), false, None));
        ui.system_msg.scroll = 7;
        for mask in 0..128 {
            let flag = |bit| mask & (1 << bit) != 0;
            let text = format!("[Game]\nGroupName={}\nPledgeMemberName={}\nPartyMemberName={}\nOtherPCName={}\nEnterChatting={}\nSystemMsgWndDamage={}\nSystemMsgWndExpendableItem={}\n",
                flag(0), flag(1), flag(2), flag(3), flag(4), flag(5), flag(6));
            synchronize_game_options(&mut ui, parse_ini(&text));
            let mut display = crate::characters::DisplaySettings {
                hide_names: true, name_dist: 871, render_limit: 99,
                ..Default::default()
            };
            synchronize_name_options(&mut display, &ui.options);
            assert_eq!(ui.keys.enter_chatting, flag(4));
            assert_eq!(ui.system_msg.options.damage, flag(5));
            assert_eq!(ui.system_msg.options.expendable_items, flag(6));
            assert_eq!(ui.system_msg.scroll, 7);
            assert_eq!((display.hide_names, display.name_dist, display.render_limit), (true, 871, 99));
            let runtime = ui.option_game_runtime.as_ref().unwrap();
            assert_eq!(runtime.state.pledge_name, i32::from(flag(0) && flag(1)));
            assert_eq!(runtime.state.party_name, i32::from(flag(0) && flag(2)));
            assert_eq!(runtime.state.other_pc_name, i32::from(flag(0) && flag(3)));
            assert_eq!(display.show_pledge_member_name, flag(0) && flag(1));
            assert_eq!(display.show_party_member_name, flag(0) && flag(2));
            assert_eq!(display.show_other_pc_name, flag(0) && flag(3));
            assert_eq!(ui.options.show_pledge_member_name, flag(1));
            assert_eq!(ui.options.show_party_member_name, flag(2));
            assert_eq!(ui.options.show_other_pc_name, flag(3));
            assert_eq!(runtime.option.bool("Game", "PledgeMemberName"), Some(flag(1)));
        }
    }

    fn display() -> DisplayOptions {
        let mode = |width, height, bits, refresh| DisplayMode { width, height, bits, refresh };
        DisplayOptions {
            modes: vec![mode(800, 600, 16, 85), mode(800, 600, 32, 60), mode(800, 600, 32, 60),
                mode(1024, 768, 32, 75), mode(1024, 768, 32, 120), mode(799, 600, 32, 60)],
            current_mode: Some((800, 600, 32)),
            current_refresh: Some(60),
        }
    }

    #[test]
    fn option_display_fill_100c8ff0_100c56d0_uses_backend_modes_and_last_rate_match() {
        let mut page = VideoPage::new(|id| Some(id.to_string()));
        page.combos[2].entries = vec!["preserved".into()];
        page.combos[2].selected = 0;
        let display = display();
        for _ in 0..3 {
            display.fill(&mut page);
            assert_eq!(page.combos[0].entries, ["800*600 32bit", "1024*768 32bit", "800*600 16bit"]);
            assert_eq!(page.combos[1].entries, ["60Hz", "60Hz"]);
            assert_eq!((page.combos[0].selected, page.combos[1].selected), (0, 1));
            assert_eq!(page.combos[2].entries, ["preserved"]);
            assert_eq!(page.combos[2].selected, 0);
            page.combos[0].selected = 2;
            page.combos[1].selected = -1;
        }
    }

    #[test]
    fn option_display_100c5580_reads_old_index_from_new_rates_without_applying() {
        let mut display = display();
        let mut page = VideoPage::new(|_| None);
        display.fill(&mut page);
        page.combos[0].selected = 1;
        assert!(display.resolution_changed(&mut page, 1));
        assert_eq!(page.combos[1].entries, ["75Hz", "120Hz"]);
        assert_eq!(page.combos[1].selected, 1);
        assert_eq!(display.current_mode, Some((800, 600, 32)));
        assert_eq!(display.current_refresh, Some(60));
        display.current_refresh = None;
        assert!(display.resolution_changed(&mut page, 1));
        page.combos[1].selected = 3;
        let previous = page.combos[1].clone();
        assert!(!display.resolution_changed(&mut page, 1));
        assert_eq!(page.combos[1], previous);
        for index in [-1, 3, 100, i32::MAX] {
            assert!(display.resolution_changed(&mut page, index));
            assert_eq!(page.combos[1], previous);
        }
    }

    #[test]
    fn option_display_100c56d0_does_not_fabricate_selection_for_absent_host_data() {
        let mut display = display();
        let mut page = VideoPage::new(|_| None);
        display.current_mode = Some((1558, 933, 32));
        display.fill(&mut page);
        assert_eq!(page.combos[0].entries.len(), 3);
        assert_eq!(page.combos[0].selected, -1);
        assert!(page.combos[1].entries.is_empty());
        assert_eq!(page.combos[1].selected, -1);
        display.current_mode = None;
        display.current_refresh = None;
        display.fill(&mut page);
        assert_eq!((page.combos[0].selected, page.combos[1].selected), (-1, -1));
        display.modes.clear();
        display.fill(&mut page);
        assert!(page.combos[0].entries.is_empty());
        assert!(page.combos[1].entries.is_empty());
    }

    #[test]
    #[ignore]
    fn option_game_runtime_full_client_files_100c9700() {
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let mut client = killer_assets::client::Client::open(&root).unwrap();
        let ini = parse_ini(&client.read_system_text("option.ini").unwrap());
        let mut ui = crate::ui2d::GameUi::load(&mut client, (1558, 933)).unwrap();
        let strings: std::collections::HashMap<_, _> = l2_dat::sysstring::read(&client.read_system_file("sysstring-e.dat").unwrap())
            .unwrap().into_iter().map(|row| (row.id, row.text)).collect();
        ui.keys.sysstrings = strings.clone();
        ui.keys.option = killer_ui::option::OptionView::new(|id| strings.get(&id).cloned());
        ui.keys.option.set_font(&ui.small);
        for _ in 0..3 {
            ui.desktop.window_mut(killer_ui::window::WindowId::OPTION).unwrap().visible = true;
            ui.sync_windows(&killer_world::World::default(), None, killer_platform::time::Instant::now());
            let runtime = ui.option_game_runtime.as_ref().unwrap();
            let page = &ui.keys.option.game_page;
            assert_eq!(page.checks.len(), 17);
            for control in &page.checks {
                assert_eq!(control.check.label, strings[&control.sysstring]);
            }
            assert_eq!(page.looting.entries, killer_ui::option_game::LOOTING_CHOICES.map(|id| strings[&id].clone()));
            assert_eq!(page.looting.selected, ini.int("Game", "PartyLooting").unwrap_or(0));
            assert_eq!(runtime.console_looting, page.looting.selected);
            assert_eq!(ui.shortcut.joypad_visible, ini.bool("Game", "UseJoystick").unwrap_or(true));
            assert_eq!(ui.keys.option.checked(0xf0f01475), ini.bool("Game", "TransparencyMode").unwrap_or(true));
            assert_eq!(ui.keys.option.checked(0xf0f01472), ini.bool("Game", "OldChatting").unwrap_or(true));
            assert_eq!(ui.keys.option.checked(0x1f0f), ini.bool("Video", "UseColorCursor").unwrap_or(true));
            ui.desktop.window_mut(killer_ui::window::WindowId::OPTION).unwrap().visible = false;
            ui.sync_windows(&killer_world::World::default(), None, killer_platform::time::Instant::now());
        }
    }

    #[test]
    fn option_game_runtime_100c9700_reads_ini_and_changes_the_real_shortcut_child() {
        for using in [false, true] {
            for available in [false, true] {
                let option = parse_ini(&format!("[Game]\nIsNative=False\nGroupName=False\nPledgeMemberName=True\nPartyMemberName=True\nOtherPCName=True\nUseJoystick={using}\nPartyLooting=4\nTransparencyMode=False\n[Video]\nUseColorCursor=False\n"));
                let localization = parse_ini("[LanguageSet]\nLanguage=2\nEnableEngSelection=True\n");
                let mut runtime = GameOptionRuntime::new(option, localization, &parse_ini("[URL]\nL2UseKeyCrypt=True\n"), available, None);
                let mut page = GamePage::new(available, |id| Some(id.to_string()));
                let mut shortcut = ShortcutView::new(false);
                runtime.state.auto_tracking = 0;
                for _ in 0..3 {
                    runtime.show(&mut page, &mut shortcut, |id| Some(id.to_string()));
                    assert_eq!(page.language.entries, ["Japanese", "English"]);
                    assert_eq!(page.language.selected, 1);
                    assert!(page.language.enabled);
                    assert_eq!(page.looting.entries.len(), 5);
                    assert_eq!((page.looting.selected, runtime.console_looting), (4, 4));
                    assert_eq!(shortcut.joypad_visible, using);
                    assert_eq!((runtime.state.pledge_name, runtime.state.party_name, runtime.state.other_pc_name), (0, 0, 0));
                    for id in [0xf0f01462, 0xf0f01463, 0xf0f01464] {
                        let child = page.checks.iter().find(|control| control.id == id).unwrap();
                        assert!(child.check.checked && !child.check.enabled);
                    }
                    for id in [0xf0f01470, 0xf0f01475, 0x1f0f, 0xf0f01474] {
                        assert!(!page.checks.iter().find(|control| control.id == id).unwrap().check.checked);
                    }
                    assert!(!page.checks.iter().find(|control| control.id == 0xf0f01474).unwrap().check.enabled);
                    assert_eq!(page.checks.len(), if available { 18 } else { 17 });
                }
            }
        }
    }

    #[test]
    fn option_game_runtime_10545eb6_respects_language_gate_and_security_provider() {
        for enabled in [false, true] {
            for active in [false, true] {
                for available in [false, true] {
                    let option = parse_ini("");
                    let localization = parse_ini(&format!("[LanguageSet]\nLanguage=5\nEnableEngSelection={enabled}\n"));
                    let mut runtime = GameOptionRuntime::new(option, localization, &parse_ini("[URL]\nL2UseKeyCrypt=True\n"), false,
                        Some(KeySecurityState { active, available }));
                    let mut page = GamePage::new(false, |_| None);
                    runtime.show(&mut page, &mut ShortcutView::new(true), |_| None);
                    assert_eq!(page.language.entries, ["Thai", "English"]);
                    assert_eq!(page.language.enabled, enabled);
                    let security = &page.checks.iter().find(|control| control.id == 0xf0f01474).unwrap().check;
                    assert_eq!((security.checked, security.enabled), (active, available));
                    assert_eq!(runtime.state.color_cursor, 1);
                    assert_eq!(runtime.state.is_native, 1);
                }
            }
        }
    }
}
