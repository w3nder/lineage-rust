pub const STATE_LOGON: i32 = 2;
pub const STATE_CHAR_CREATE: i32 = 3;
pub const STATE_CHAR_SELECT: i32 = 4;

const RACES: [&str; 5] = ["Human", "Elf", "DarkElf", "Orc", "Dwarf"];
const KNIGHT_CLASSES: [i32; 5] = [0, 18, 31, 44, 53];
const MAGE_CLASSES: [i32; 4] = [10, 25, 38, 49];
const CLASS_TABLE_LAST: i32 = 0x35;

pub fn move_camera_by_state(state: i32) -> Option<&'static str> {
    match state {
        STATE_LOGON => Some("LogOn_Warp"),
        STATE_CHAR_CREATE => Some("Char_Create_Warp"),
        STATE_CHAR_SELECT => Some("Char_Select_Warp"),
        _ => None,
    }
}

pub fn move_camera_by_character(race: i32, class_id: i32, gender: i32) -> Option<String> {
    if race == -1 {
        return None;
    }
    let mut tag = String::new();
    if let Some(name) = usize::try_from(race).ok().and_then(|index| RACES.get(index)) {
        tag.push_str(name);
    }
    if (0..=CLASS_TABLE_LAST).contains(&class_id) {
        if KNIGHT_CLASSES.contains(&class_id) {
            tag.push_str("_Knight");
        } else if MAGE_CLASSES.contains(&class_id) {
            tag.push_str("_Mage");
        }
    }
    match gender {
        0 => tag.push_str("_M"),
        1 => tag.push_str("_F"),
        _ => {}
    }
    tag.push_str("_Warp");
    Some(tag)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_camera_by_state_1042e860_triggers_the_three_lobby_warps() {
        assert_eq!(move_camera_by_state(2), Some("LogOn_Warp"));
        assert_eq!(move_camera_by_state(3), Some("Char_Create_Warp"));
        assert_eq!(move_camera_by_state(4), Some("Char_Select_Warp"));
        for state in [-1, 0, 1, 5, 6, 7] {
            assert_eq!(move_camera_by_state(state), None);
        }
    }

    #[test]
    fn move_camera_by_character_1042e920_joins_race_class_gender_and_warp() {
        assert_eq!(move_camera_by_character(-1, 0, 0), None);
        assert_eq!(move_camera_by_character(0, 0, 0).as_deref(), Some("Human_Knight_M_Warp"));
        assert_eq!(move_camera_by_character(1, 25, 1).as_deref(), Some("Elf_Mage_F_Warp"));
        assert_eq!(move_camera_by_character(2, 31, 0).as_deref(), Some("DarkElf_Knight_M_Warp"));
        assert_eq!(move_camera_by_character(3, 49, 1).as_deref(), Some("Orc_Mage_F_Warp"));
        assert_eq!(move_camera_by_character(4, 53, 0).as_deref(), Some("Dwarf_Knight_M_Warp"));
        assert_eq!(move_camera_by_character(0, 1, 0).as_deref(), Some("Human_M_Warp"));
        assert_eq!(move_camera_by_character(0, 0x36, 2).as_deref(), Some("Human_Warp"));
        assert_eq!(move_camera_by_character(5, 10, 1).as_deref(), Some("_Mage_F_Warp"));
        assert_eq!(move_camera_by_character(-2, -1, -1).as_deref(), Some("_Warp"));
        for class_id in 0..=CLASS_TABLE_LAST {
            let knight = [0, 18, 31, 44, 53].contains(&class_id);
            let mage = [10, 25, 38, 49].contains(&class_id);
            let tag = move_camera_by_character(0, class_id, 0).unwrap_or_default();
            assert_eq!(tag.contains("_Knight"), knight, "{class_id}");
            assert_eq!(tag.contains("_Mage"), mage, "{class_id}");
        }
    }
}
