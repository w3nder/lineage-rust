pub mod preferences;
pub mod calendar;

pub use web_time as time;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static HUD: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

#[cfg(target_arch = "wasm32")]
pub fn hud_preferences() -> String { HUD.with(|text| text.borrow().clone()) }

#[cfg(target_arch = "wasm32")]
pub fn set_hud_preferences(text: String) { HUD.with(|value| *value.borrow_mut() = text); }

#[cfg(not(target_arch = "wasm32"))]
pub use std::fs;

#[cfg(target_arch = "wasm32")]
pub mod fs;

pub fn is_dir(path: &std::path::Path) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    { path.is_dir() }
    #[cfg(target_arch = "wasm32")]
    { fs::is_dir(path) }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static SETTINGS: std::cell::RefCell<preferences::Preferences> = std::cell::RefCell::new(preferences::Preferences::default());
}

#[cfg(target_arch = "wasm32")]
pub fn setting(name: &str) -> Option<String> {SETTINGS.with(|settings|settings.borrow().get(name).map(str::to_owned))}

#[cfg(target_arch = "wasm32")]
pub fn set_setting(name: &str,text: String) -> Result<(),&'static str> {SETTINGS.with(|settings|settings.borrow_mut().set(name,text))}

#[cfg(target_arch = "wasm32")]
pub fn settings() -> std::collections::BTreeMap<String,String> {SETTINGS.with(|settings|settings.borrow().snapshot())}

#[cfg(target_arch = "wasm32")]
pub fn restore_settings(files: std::collections::BTreeMap<String,String>) -> Result<(),&'static str> {SETTINGS.with(|settings|settings.borrow_mut().restore(files))}

#[cfg(target_arch = "wasm32")]
pub fn settings_revision() -> u32 {SETTINGS.with(|settings|settings.borrow().revision())}

#[cfg(not(target_arch = "wasm32"))]
static ORDER_ROOT: std::sync::Mutex<Option<std::path::PathBuf>> = std::sync::Mutex::new(None);

#[cfg(not(target_arch = "wasm32"))]
pub fn set_item_order_root(root: std::path::PathBuf) {
    if let Ok(mut guard) = ORDER_ROOT.lock() {
        *guard = Some(root);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn read_item_order(file: &str) -> Option<Vec<u8>> {
    let root = ORDER_ROOT.lock().ok()?.clone()?;
    std::fs::read(root.join("iwo").join(file)).ok()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn write_item_order(file: &str, bytes: &[u8]) -> Result<(), String> {
    let root = ORDER_ROOT.lock().map_err(|e| e.to_string())?.clone().ok_or("sem pasta do cliente")?;
    let dir = root.join("iwo");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(file), bytes).map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn read_item_order(file: &str) -> Option<Vec<u8>> {
    let text = setting(&format!("iwo/{file}"))?;
    (0..text.len() / 2).map(|i| u8::from_str_radix(text.get(i * 2..i * 2 + 2)?, 16).ok()).collect()
}

#[cfg(target_arch = "wasm32")]
pub fn write_item_order(file: &str, bytes: &[u8]) -> Result<(), String> {
    let text: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    set_setting(&format!("iwo/{file}"), text).map_err(str::to_owned)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod order_tests {
    #[test]
    fn the_item_order_file_goes_to_the_iwo_folder_of_the_client_and_comes_back_byte_for_byte() {
        let root = std::env::temp_dir().join(format!("killer-iwo-{}", std::process::id()));
        super::set_item_order_root(root.clone());
        assert_eq!(super::read_item_order("a_1.dat"), None);
        super::write_item_order("a_1.dat", &[1, 0, 0, 0, 0xff, 0xff, 0xff, 0xff]).unwrap();
        assert_eq!(super::read_item_order("a_1.dat"), Some(vec![1, 0, 0, 0, 0xff, 0xff, 0xff, 0xff]));
        assert!(root.join("iwo").join("a_1.dat").is_file());
        let _ = std::fs::remove_dir_all(root);
    }
}
