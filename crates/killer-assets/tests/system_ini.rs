use killer_assets::client::Client;
use killer_ui::ini::{Options, WindowsInfo, parse_ini};
use killer_ui::window::WindowId;
use std::path::PathBuf;

fn test_client() -> Option<Client> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    Client::open(&root).ok()
}

#[test]
#[ignore]
fn windows_info_ini_has_the_chat_section() {
    let Some(mut client) = test_client() else { return };
    let ini = parse_ini(&client.read_system_text("WindowsInfo.ini").unwrap());
    assert!(ini.0.contains_key("44"));
    let info = WindowsInfo::from_ini(&ini);
    assert!(info.chat_height.is_some());
    assert!(info.position(WindowId::CHAT).is_some());
}

#[test]
#[ignore]
fn option_ini_has_a_viewport() {
    let Some(mut client) = test_client() else { return };
    let options = Options::from_ini(&parse_ini(&client.read_system_text("option.ini").unwrap()));
    assert!(options.viewport.is_some_and(|(w, h)| w > 0 && h > 0));
}
