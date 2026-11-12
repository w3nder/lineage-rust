pub const CLICK: usize = 0;
pub const CLICK_FAILED: usize = 2;
pub const PICKUP: usize = 3;
pub const TRASH_BASKET: usize = 4;
pub const WINDOW_OPEN: usize = 5;
pub const WINDOW_CLOSE: usize = 6;
pub const CHARSTAT_OPEN: usize = 11;
pub const CHARSTAT_CLOSE: usize = 12;
pub const INVENTORY_OPEN: usize = 13;
pub const INVENTORY_CLOSE: usize = 14;
pub const MAP_OPEN: usize = 15;
pub const MAP_CLOSE: usize = 16;
pub const SYSTEM_OPEN: usize = 17;
pub const SYSTEM_CLOSE: usize = 18;

pub const TABLE: [&str; 19] = [
    "InterfaceSound.click_01",
    "itemsound.click_2",
    "itemsound.click_failed",
    "itemsound.pickup",
    "itemsound.trash_basket",
    "itemsound.window_open",
    "itemsound.window_close",
    "itemsound.quest_tutorial",
    "itemSound.itemdrop_etc_paper",
    "itemSound.cooltime_end",
    "itemSound.petition",
    "InterfaceSound.charstat_open_01",
    "InterfaceSound.charstat_close_01",
    "InterfaceSound.inventory_open_01",
    "InterfaceSound.inventory_close_01",
    "InterfaceSound.map_open_01",
    "InterfaceSound.map_close_01",
    "InterfaceSound.system_open_01",
    "InterfaceSound.system_close_01",
];

pub fn name(index: usize) -> Option<&'static str> {
    TABLE.get(index).copied()
}

pub fn button_click(enabled: bool) -> &'static str {
    TABLE[if enabled { CLICK } else { CLICK_FAILED }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_indices_name_the_sounds_of_the_nwindow_table() {
        assert_eq!(name(CLICK), Some("InterfaceSound.click_01"));
        assert_eq!(name(WINDOW_CLOSE), Some("itemsound.window_close"));
        assert_eq!(name(CHARSTAT_OPEN), Some("InterfaceSound.charstat_open_01"));
        assert_eq!(name(INVENTORY_CLOSE), Some("InterfaceSound.inventory_close_01"));
        assert_eq!(name(SYSTEM_CLOSE), Some("InterfaceSound.system_close_01"));
        assert_eq!(name(19), None);
        assert_eq!(button_click(true), "InterfaceSound.click_01");
        assert_eq!(button_click(false), "itemsound.click_failed");
    }

    #[test]
    #[ignore]
    fn the_table_matches_0x10244378_of_the_installed_nwindow_dll() {
        let root = std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let data = std::fs::read(std::path::Path::new(&root).join("system/nwindow.dll")).expect("nwindow.dll");
        let pe = data.windows(4).position(|w| w == b"PE\0\0").expect("PE");
        let sections = u16::from_le_bytes([data[pe + 6], data[pe + 7]]) as usize;
        let optional = u16::from_le_bytes([data[pe + 20], data[pe + 21]]) as usize;
        let base = u32::from_le_bytes(data[pe + 52..pe + 56].try_into().expect("base"));
        let table = pe + 24 + optional;
        let offset = |va: u32| {
            (0..sections).find_map(|i| {
                let s = table + i * 40;
                let word = |at: usize| u32::from_le_bytes(data[s + at..s + at + 4].try_into().expect("word"));
                let (size, rva, raw) = (word(8), word(12), word(20));
                let rel = va.checked_sub(base)?;
                (rel >= rva && rel < rva + size).then(|| (rel - rva + raw) as usize)
            })
        };
        for (index, expected) in TABLE.iter().enumerate() {
            let slot = offset(0x1024_4378 + index as u32 * 4).expect("slot");
            let pointer = u32::from_le_bytes(data[slot..slot + 4].try_into().expect("pointer"));
            let at = offset(pointer).expect("string");
            let units: Vec<u16> = data[at..].chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).take_while(|u| *u != 0).collect();
            assert_eq!(String::from_utf16(&units).expect("utf16"), *expected, "índice {index}");
        }
    }
}
