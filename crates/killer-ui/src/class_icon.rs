pub const CLASS_CATEGORY: [u8; 119] = [
    7, 7, 0, 0, 7, 2, 2, 7, 0, 1, 8, 8, 4, 4, 6, 8,
    5, 5, 7, 7, 2, 3, 7, 0, 1, 8, 8, 4, 6, 8, 5, 7,
    7, 2, 3, 7, 0, 1, 8, 8, 4, 6, 8, 5, 7, 7, 0, 7,
    0, 8, 8, 5, 5, 7, 7, 0, 7, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 9, 9, 11, 11, 10, 9, 13, 13,
    15, 14, 14, 11, 12, 9, 10, 13, 15, 14, 11, 12, 9, 10, 13, 15,
    14, 9, 9, 14, 14, 9, 9,
];

pub const ICON_TEXTURES: [&str; 16] = [
    "L2UI_CH3.PartyWnd.party_styleicon1",
    "L2UI_CH3.PartyWnd.party_styleicon2",
    "L2UI_CH3.PartyWnd.party_styleicon3",
    "L2UI_CH3.PartyWnd.party_styleicon4",
    "L2UI_CH3.PartyWnd.party_styleicon5",
    "L2UI_CH3.PartyWnd.party_styleicon6",
    "L2UI_CH3.PartyWnd.party_styleicon7",
    "L2UI_CH3.PartyWnd.party_styleicon1_1",
    "L2UI_CH3.PartyWnd.party_styleicon1_2",
    "L2UI_CH3.PartyWnd.party_styleicon1_3",
    "L2UI_CH3.PartyWnd.party_styleicon2_3",
    "L2UI_CH3.PartyWnd.party_styleicon3_3",
    "L2UI_CH3.PartyWnd.party_styleicon4_3",
    "L2UI_CH3.PartyWnd.party_styleicon5_3",
    "L2UI_CH3.PartyWnd.party_styleicon6_3",
    "L2UI_CH3.PartyWnd.party_styleicon7_3",
];

pub fn category(class_id: i32) -> usize {
    usize::try_from(class_id).ok().and_then(|id| CLASS_CATEGORY.get(id)).map_or(0, |&category| usize::from(category))
}

pub fn texture(class_id: i32) -> Option<&'static str> {
    (class_id >= 0).then(|| ICON_TEXTURES[category(class_id)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categories_follow_the_compare_chain_of_1006bac0() {
        assert_eq!((category(0), category(9), category(96), category(111), category(118)), (7, 1, 15, 15, 9));
        assert_eq!((category(2), category(57), category(58), category(87), category(500), category(-1)), (0, 0, 0, 0, 0, 0));
    }

    #[test]
    fn textures_follow_the_jump_table_of_10074d20() {
        assert_eq!(texture(9), Some("L2UI_CH3.PartyWnd.party_styleicon2"));
        assert_eq!(texture(2), Some("L2UI_CH3.PartyWnd.party_styleicon1"));
        assert_eq!(texture(96), Some("L2UI_CH3.PartyWnd.party_styleicon7_3"));
        assert_eq!(texture(-1), None);
    }
}
