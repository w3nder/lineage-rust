#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Argb(pub u32);

impl Argb {
    pub fn rgba(self) -> [f32; 4] {
        let channel = |shift: u32| ((self.0 >> shift) & 0xFF) as f32 / 255.0;
        [channel(16), channel(8), channel(0), channel(24)]
    }
}

pub const TEXT: Argb = Argb(0xFFDCDCDC);
pub const GOLD: Argb = Argb(0xFFB09B79);
pub const TITLE: Argb = Argb(0xFFC8D2DC);
pub const TARGET_LABEL: Argb = Argb(0xFFA3A3A3);
pub const TARGET_VALUE: Argb = Argb(0xFF808080);
pub const BUTTON_TEXT: Argb = Argb(0xFFE6DCBE);
pub const BUTTON_TEXT_DISABLED: Argb = Argb(0xFFA0A0A0);
pub const CHECK_TEXT: Argb = Argb(0xFFDCDCDC);
pub const CHECK_TEXT_DISABLED: Argb = Argb(0xFF808080);
pub const TARGET_NAME_COLORS: [(i32, Argb); 7] = [
    (-8, Argb(0xFFFF0000)),
    (-5, Argb(0xFFFF9191)),
    (-2, Argb(0xFFFAFE91)),
    (3, Argb(0xFFDCDCDC)),
    (6, Argb(0xFFA2FFAB)),
    (9, Argb(0xFFA2A8FC)),
    (i32::MAX, Argb(0xFF0000FF)),
];
pub const CHAT_SYSTEM: i32 = -1;

pub fn target_name_color(level_difference: i32) -> Argb {
    TARGET_NAME_COLORS.iter().find(|(limit, _)| level_difference < *limit).map_or(TEXT, |(_, color)| *color)
}

pub fn chat_color(kind: i32) -> Argb {
    match kind {
        1 | 11 => Argb(0xFFFF7200),
        2 => Argb(0xFFFF00FF),
        3 => Argb(0xFF00FF00),
        4 => Argb(0xFF7D77FF),
        6 | 7 | 10 => Argb(0xFF80FFFF),
        8 => Argb(0xFFEAA5F5),
        9 => Argb(0xFF77FF99),
        15 => Argb(0xFFFFE87B),
        16 => Argb(0xFF7B7DF2),
        17 => Argb(0xFF408CFF),
        18 => Argb(0xFF00FFFF),
        _ => TEXT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_colors_follow_the_client_table() {
        assert_eq!(chat_color(1).0, 0xFFFF7200);
        assert_eq!(chat_color(11).0, 0xFFFF7200);
        assert_eq!(chat_color(2).0, 0xFFFF00FF);
        assert_eq!(chat_color(99).0, 0xFFDCDCDC);
        assert_eq!(chat_color(CHAT_SYSTEM), TEXT);
        assert_eq!(Argb(0x80FF0000).rgba(), [1.0, 0.0, 0.0, 128.0 / 255.0]);
    }

    #[test]
    fn target_name_color_follows_the_level_difference_bands() {
        assert_eq!(target_name_color(-9).0, 0xFFFF0000);
        assert_eq!(target_name_color(-8).0, 0xFFFF9191);
        assert_eq!(target_name_color(-6).0, 0xFFFF9191);
        assert_eq!(target_name_color(-5).0, 0xFFFAFE91);
        assert_eq!(target_name_color(-2).0, 0xFFDCDCDC);
        assert_eq!(target_name_color(0).0, 0xFFDCDCDC);
        assert_eq!(target_name_color(2).0, 0xFFDCDCDC);
        assert_eq!(target_name_color(3).0, 0xFFA2FFAB);
        assert_eq!(target_name_color(6).0, 0xFFA2A8FC);
        assert_eq!(target_name_color(8).0, 0xFFA2A8FC);
        assert_eq!(target_name_color(9).0, 0xFF0000FF);
    }
}
