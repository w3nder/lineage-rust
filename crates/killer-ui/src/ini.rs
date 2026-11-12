use crate::sysmsg::SystemMsgOptions;
use crate::window::WindowId;
use std::collections::HashMap;

pub struct IniFile(pub HashMap<String, HashMap<String, String>>);

pub fn parse_ini(text: &str) -> IniFile {
    let mut sections: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut current = String::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            current = name.trim().to_lowercase();
            sections.entry(current.clone()).or_default();
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            sections.entry(current.clone()).or_default().insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }
    IniFile(sections)
}

impl IniFile {
    pub fn has_section(&self, section: &str) -> bool {
        self.0.contains_key(&section.to_lowercase())
    }

    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.0.get(&section.to_lowercase())?.get(&key.to_lowercase()).map(String::as_str)
    }

    pub fn int(&self, section: &str, key: &str) -> Option<i32> {
        self.get(section, key)?.parse().ok()
    }

    pub fn bool(&self, section: &str, key: &str) -> Option<bool> {
        let value = self.get(section, key)?;
        if value.eq_ignore_ascii_case("true") {
            Some(true)
        } else if value.eq_ignore_ascii_case("false") {
            Some(false)
        } else {
            None
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WindowsInfo {
    pub positions: HashMap<u32, (i32, i32)>,
    pub widths: HashMap<u32, i32>,
    pub chat_height: Option<i32>,
    pub shortcut_rotate: Option<bool>,
}

impl WindowsInfo {
    pub fn from_ini(ini: &IniFile) -> WindowsInfo {
        let positions = ini
            .0
            .keys()
            .filter_map(|name| name.parse::<u32>().ok())
            .filter_map(|id| {
                let section = id.to_string();
                Some((id, (ini.int(&section, "posX")?, ini.int(&section, "posY")?)))
            })
            .collect();
        let widths = ini
            .0
            .keys()
            .filter_map(|name| name.parse::<u32>().ok())
            .filter_map(|id| Some((id, ini.int(&id.to_string(), "width")?)))
            .collect();
        WindowsInfo {
            positions,
            widths,
            chat_height: ini.int(&WindowId::CHAT.0.to_string(), "Height"),
            shortcut_rotate: ini.bool(&WindowId::SHORTCUT.0.to_string(), "rotate"),
        }
    }

    pub fn position(&self, id: WindowId) -> Option<(i32, i32)> {
        self.positions.get(&id.0).copied()
    }

    pub fn width(&self, id: WindowId) -> Option<i32> {
        self.widths.get(&id.0).copied()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Options {
    pub transparency: bool,
    pub viewport: Option<(i32, i32)>,
    pub system_msg: SystemMsgOptions,
    pub show_my_name: bool,
    pub show_npc_name: bool,
    pub show_group_name: bool,
    pub show_pledge_member_name: bool,
    pub show_party_member_name: bool,
    pub show_other_pc_name: bool,
    pub auto_tracking_pawn: bool,
    pub arrow_mode: bool,
    pub show_zone_title: bool,
    pub enter_chatting: bool,
    pub old_chatting: bool,
}

impl Options {
    pub fn from_ini(ini: &IniFile) -> Options {
        Options {
            transparency: ini.bool("Game", "TransparencyMode").unwrap_or(true),
            viewport: ini.int("Video", "GamePlayViewportX").zip(ini.int("Video", "GamePlayViewportY")),
            system_msg: SystemMsgOptions::from_ini(ini),
            show_my_name: ini.bool("Game", "MyName").unwrap_or(true),
            show_npc_name: ini.bool("Game", "NPCName").unwrap_or(true),
            show_group_name: ini.bool("Game", "GroupName").unwrap_or(true),
            show_pledge_member_name: ini.bool("Game", "PledgeMemberName").unwrap_or(true),
            show_party_member_name: ini.bool("Game", "PartyMemberName").unwrap_or(true),
            show_other_pc_name: ini.bool("Game", "OtherPCName").unwrap_or(true),
            auto_tracking_pawn: ini.bool("Game", "AutoTrackingPawn").unwrap_or(true),
            arrow_mode: ini.bool("Game", "ArrowMode").unwrap_or(true),
            show_zone_title: ini.bool("Game", "ShowZoneTitle").unwrap_or(true),
            enter_chatting: ini.bool("Game", "EnterChatting").unwrap_or(false),
            old_chatting: ini.bool("Game", "OldChatting").unwrap_or(true),
        }
    }
}

pub const CHAT_TABS: [&str; 5] = ["entire", "market", "party", "pledge", "ally"];

/// Ordem das chaves dentro de cada seção do `chatfilter.ini`, a mesma em que o
/// `LoadChatFilter @ 0x100558d0` as lê e a mesma do arquivo que vem com o cliente.
pub const CHAT_FILTER_KEYS: [&str; 12] = [
    "system", "damage", "chat", "normal", "shout", "pledge", "party", "market", "tell", "ally", "useitems", "hero",
];

/// Ordem das seções no arquivo do cliente.
pub const CHAT_FILTER_SECTIONS: [&str; 5] = ["entire", "pledge", "ally", "party", "market"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatTab {
    pub name: &'static str,
    pub filters: HashMap<String, bool>,
}

impl ChatTab {
    pub fn shows(&self, filter: &str) -> bool {
        self.filters.get(&filter.to_lowercase()).copied().unwrap_or(false)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatFilter {
    pub tabs: [ChatTab; 5],
}

impl ChatFilter {
    pub fn from_ini(ini: &IniFile) -> ChatFilter {
        ChatFilter {
            tabs: CHAT_TABS.map(|name| {
                let section = format!("{name}_tab");
                let (normal, pledge, party, market, ally) = match name {
                    "entire" => (true, true, true, false, false),
                    "pledge" => (false, true, false, false, false),
                    "party" => (false, false, true, false, false),
                    "market" => (false, false, false, true, false),
                    _ => (false, false, false, false, true),
                };
                let defaults = [
                    ("system", true),
                    ("chat", true),
                    ("normal", normal),
                    ("shout", true),
                    ("pledge", pledge),
                    ("party", party),
                    ("market", market),
                    ("tell", true),
                    ("damage", true),
                    ("ally", ally),
                    ("useitems", false),
                    ("hero", false),
                ];
                let filters = defaults
                    .into_iter()
                    .map(|(key, default)| (key.to_string(), ini.bool(&section, key).unwrap_or(default)))
                    .collect();
                ChatTab { name, filters }
            }),
        }
    }

    /// Texto do `chatfilter.ini` como o cliente o escreve: seções na ordem do arquivo original,
    /// as doze chaves na ordem do `LoadChatFilter`, `True`/`False` e fim de linha do Windows.
    pub fn to_ini(&self) -> String {
        let mut text = String::new();
        for section in CHAT_FILTER_SECTIONS {
            let Some(tab) = self.tabs.iter().find(|tab| tab.name == section) else { continue };
            if !text.is_empty() {
                text.push_str("\r\n");
            }
            text.push_str(&format!("[{section}_tab]\r\n"));
            for key in CHAT_FILTER_KEYS {
                let Some(on) = tab.filters.get(key) else { continue };
                let value = if *on { "True" } else { "False" };
                text.push_str(&format!("{key}={value}\r\n"));
            }
        }
        text
    }
}

pub const TRANSPARENT_FRAME_ALPHA: f32 = 128.0 / 255.0;

pub fn frame_alpha(_options: &Options) -> f32 {
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    const WINDOWS_INFO: &str = include_str!("../tests/fixtures/windowsinfo.ini");
    const OPTION: &str = include_str!("../tests/fixtures/option.ini");
    const CHAT_FILTER: &str = include_str!("../tests/fixtures/chatfilter.ini");

    #[test]
    fn parses_sections_and_keys_case_insensitively() {
        let ini = parse_ini(
            "; comentário\r\n[Game]\r\nTransparencyMode = True\r\n\r\n[44]\r\nposX=0\r\nposY=746\r\nHeight=187\r\n",
        );
        assert_eq!(ini.get("game", "transparencymode"), Some("True"));
        assert_eq!(ini.get("GAME", "TRANSPARENCYMODE"), Some("True"));
        assert_eq!(ini.int("44", "height"), Some(187));
        assert_eq!(ini.get("44", "missing"), None);
        assert_eq!(ini.get("nope", "posX"), None);
    }

    #[test]
    fn windows_info_reads_positions_chat_height_and_rotate() {
        let info = WindowsInfo::from_ini(&parse_ini(WINDOWS_INFO));
        assert_eq!(info.position(WindowId::CHAT), Some((0, 746)));
        assert_eq!(info.chat_height, Some(187));
        assert_eq!(info.position(WindowId::SHORTCUT), Some((1478, 168)));
        assert_eq!(info.shortcut_rotate, Some(false));
        assert_eq!(info.positions.get(&3), Some(&(635, 310)));
        assert_eq!(info.width(WindowId(12)), Some(172));
        assert_eq!(info.width(WindowId::CHAT), None);
        assert!(info.positions.len() > 80);
    }

    #[test]
    fn windows_info_without_the_special_sections_leaves_them_empty() {
        let info = WindowsInfo::from_ini(&parse_ini("[1]\nposX=5\nposY=6\n[2]\nposX=1\n"));
        assert_eq!(info.positions.len(), 1);
        assert_eq!(info.chat_height, None);
        assert_eq!(info.shortcut_rotate, None);
    }

    #[test]
    fn options_read_transparency_and_viewport() {
        let options = Options::from_ini(&parse_ini(OPTION));
        assert!(options.transparency);
        assert_eq!(options.viewport, Some((1558, 933)));
        assert!(options.system_msg.shown && options.system_msg.damage && options.system_msg.expendable_items);
        assert!(options.show_zone_title);
        assert_eq!(frame_alpha(&options), 1.0, "alpha legado não representa o estado dinâmico de NCGDevice");
    }

    #[test]
    fn chat_filter_reads_the_five_tabs_in_order() {
        let filter = ChatFilter::from_ini(&parse_ini(CHAT_FILTER));
        let names: Vec<&str> = filter.tabs.iter().map(|t| t.name).collect();
        assert_eq!(names, ["entire", "market", "party", "pledge", "ally"]);
        assert!(filter.tabs.iter().all(|t| t.filters.len() == 12));
        assert!(filter.tabs[0].shows("damage"));
        assert!(!filter.tabs[3].shows("normal"));
        assert!(filter.tabs[3].shows("pledge"));
        assert!(filter.tabs[2].shows("party") && !filter.tabs[2].shows("market"));
        assert!(filter.tabs[1].shows("market") && !filter.tabs[1].shows("chat"));
        assert!(filter.tabs[1].shows("TELL"));
        assert!(!filter.tabs[1].shows("unknown"));
        let text = filter.to_ini();
        assert_eq!(text.lines().next(), Some("[entire_tab]"));
        assert_eq!(text.lines().nth(1), Some("system=True"));
        assert_eq!(text.lines().nth(2), Some("damage=True"));
        assert_eq!(text.lines().nth(13), Some(""));
        assert_eq!(text.lines().nth(14), Some("[pledge_tab]"));
        assert_eq!(text.lines().filter(|l| l.starts_with('[')).count(), 5);
        assert_eq!(text.matches("=True").count() + text.matches("=False").count(), 60);
        assert_eq!(ChatFilter::from_ini(&parse_ini(&text)), filter);
        assert!(text.contains("\r\n"));
    }

    #[test]
    fn chat_filter_without_sections_takes_the_defaults_of_load_filters() {
        let filter = ChatFilter::from_ini(&parse_ini(""));
        assert_eq!(filter.tabs.len(), 5);
        let on = |tab: usize| CHAT_FILTER_KEYS.iter().filter(|key| filter.tabs[tab].shows(key)).copied().collect::<Vec<_>>();
        assert_eq!(on(0), ["system", "damage", "chat", "normal", "shout", "pledge", "party", "tell"]);
        assert_eq!(on(1), ["system", "damage", "chat", "shout", "market", "tell"]);
        assert_eq!(on(2), ["system", "damage", "chat", "shout", "party", "tell"]);
        assert_eq!(on(3), ["system", "damage", "chat", "shout", "pledge", "tell"]);
        assert_eq!(on(4), ["system", "damage", "chat", "shout", "tell", "ally"]);
        assert!(!filter.tabs[0].shows("hero"));
    }

    #[test]
    fn legacy_frame_alpha_does_not_apply_device_transparency() {
        let options = Options::from_ini(&parse_ini("[Game]\nTransparencyMode=False\n"));
        assert!(!options.transparency);
        assert_eq!(options.viewport, None);
        assert_eq!(frame_alpha(&options), 1.0);
        assert_eq!(frame_alpha(&Options::from_ini(&parse_ini(""))), 1.0);
        let on = Options::from_ini(&parse_ini("[Game]\nTransparencyMode=True\n"));
        assert!(on.transparency);
        assert_eq!(frame_alpha(&on), 1.0, "estado dinâmico de transparência pertence ao dispositivo");
    }
}
