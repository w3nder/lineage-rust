use crate::ui2d::GameUi;
use crate::ui_option_config::ConfigEdit;
use killer_ui::window::WindowId;

const SAVED_ID_LIMIT: u32 = 0x10000;

pub fn edits(ui: &GameUi) -> Vec<ConfigEdit> {
    let mut edits = Vec::new();
    for window in &ui.desktop.windows {
        let id = window.id.0;
        if id == 0 || id >= SAVED_ID_LIMIT {
            continue;
        }
        let section = id.to_string();
        edits.push(ConfigEdit::new(&section, "posX", window.rect.x.to_string()));
        edits.push(ConfigEdit::new(&section, "posY", window.rect.y.to_string()));
    }
    for id in [WindowId::TARGET, WindowId::STATUS] {
        if let Some(window) = ui.desktop.window(id) {
            edits.push(ConfigEdit::new(&id.0.to_string(), "width", window.rect.w.to_string()));
        }
    }
    if let Some(window) = ui.desktop.window(WindowId::CHAT) {
        edits.push(ConfigEdit::new(&WindowId::CHAT.0.to_string(), "Height", window.rect.h.to_string()));
    }
    let rotate = if ui.shortcut.vertical { "False" } else { "True" };
    edits.push(ConfigEdit::new(&WindowId::SHORTCUT.0.to_string(), "rotate", rotate));
    edits
}

pub fn save(path: &std::path::Path, ui: &GameUi) -> std::io::Result<()> {
    crate::ui_option_config::save_or_create(path, &edits(ui))
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::ini::{WindowsInfo, parse_ini};

    #[test]
    fn windows_info_save_10073370_round_trips_through_the_load() {
        let mut ui = crate::ui2d::tests::game_ui((1558, 933));
        {
            let status = ui.desktop.window_mut(WindowId::STATUS).unwrap();
            status.rect.x = 41;
            status.rect.y = 42;
            status.rect.w = 250;
        }
        {
            let target = ui.desktop.window_mut(WindowId::TARGET).unwrap();
            target.rect.w = 374;
            target.rect.x = 600;
        }
        let ini = parse_ini(&crate::ui_option_config::edit_ini("", &edits(&ui)));
        let info = WindowsInfo::from_ini(&ini);
        assert_eq!(info.position(WindowId::STATUS), Some((41, 42)));
        assert_eq!(info.width(WindowId::STATUS), Some(250));
        assert_eq!(info.width(WindowId::TARGET), Some(374));
        assert_eq!(info.position(WindowId::TARGET).map(|p| p.0), Some(600));
        assert_eq!(info.width(WindowId::PARTY_STATUS), None);
        assert_eq!(info.chat_height, Some(187));
        assert_eq!(info.shortcut_rotate, Some(false));
        assert!(info.positions.keys().all(|id| *id < SAVED_ID_LIMIT));
    }

    #[test]
    fn saving_creates_the_file_when_missing_and_keeps_other_sections() {
        let ui = crate::ui2d::tests::game_ui((1558, 933));
        let dir = std::env::temp_dir().join(format!("killer-wininfo-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("WindowsInfo.ini");
        save(&path, &ui).unwrap();
        let first = std::fs::read_to_string(&path).unwrap();
        assert_eq!(WindowsInfo::from_ini(&parse_ini(&first)).width(WindowId::STATUS), Some(172));
        std::fs::write(&path, format!("[999]\nposX=1\n{first}")).unwrap();
        save(&path, &ui).unwrap();
        assert!(std::fs::read_to_string(&path).unwrap().contains("[999]\nposX=1"));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
