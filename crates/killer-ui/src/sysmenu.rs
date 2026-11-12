use crate::color::TEXT;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{FrameCtrl, TITLE_HEIGHT};
use crate::window::Rect;

pub const SYSMENU_SIZE: (i32, i32) = (172, 295);
pub const SYSMENU_BACK: &str = "L2UI_CH3.SystemMenuWnd.systemmenu_back";
pub const TITLE_SYSSTRING: u32 = 446;
pub const BUTTON_X: i32 = 13;
pub const BUTTON_Y: i32 = 29;
pub const BUTTON_STEP: i32 = 37;
pub const BUTTON_SIZE: i32 = 36;
pub const LABEL_X: i32 = 51;
pub const LABEL_Y: i32 = 43;
pub const BUTTONS: usize = 7;
pub const ALT_B: &str = "(Alt+B)";
pub const ALT_R: &str = "(Alt+R)";
pub const SHORTCUT_BAR_WIDTH: i32 = 46;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SysMenuButton {
    Community,
    Macro,
    Help,
    Petition,
    Options,
    Restart,
    ExitGame,
}

impl SysMenuButton {
    pub const ALL: [SysMenuButton; BUTTONS] = [
        SysMenuButton::Community,
        SysMenuButton::Macro,
        SysMenuButton::Help,
        SysMenuButton::Petition,
        SysMenuButton::Options,
        SysMenuButton::Restart,
        SysMenuButton::ExitGame,
    ];

    pub fn index(self) -> usize {
        SysMenuButton::ALL.iter().position(|b| *b == self).unwrap_or(0)
    }

    pub fn sysstring(self) -> u32 {
        match self {
            SysMenuButton::Community => 387,
            SysMenuButton::Macro => 711,
            SysMenuButton::Help => 145,
            SysMenuButton::Petition => 470,
            SysMenuButton::Options => 146,
            SysMenuButton::Restart => 147,
            SysMenuButton::ExitGame => 148,
        }
    }

    pub fn control_id(self) -> u32 {
        match self {
            SysMenuButton::Community => 0xf0f024,
            SysMenuButton::Macro => 0xf0f022,
            SysMenuButton::Help => 0xf0f006,
            SysMenuButton::Petition => 0xf0f023,
            SysMenuButton::Options => 0xf0f007,
            SysMenuButton::Restart => 0xf0f008,
            SysMenuButton::ExitGame => 0xf0f009,
        }
    }

    pub fn textures(self) -> (String, String) {
        let n = self.index() + 1;
        (format!("L2UI_CH3.SystemMenuWnd.systemicon{n}"), format!("L2UI_CH3.SystemMenuWnd.systemicon{n}_down"))
    }

    pub fn rect(self) -> Rect {
        Rect::new(BUTTON_X, BUTTON_Y + BUTTON_STEP * self.index() as i32, BUTTON_SIZE, BUTTON_SIZE)
    }

    pub fn label(self, sysstring: &str) -> String {
        match self {
            SysMenuButton::Community => format!("{sysstring} {ALT_B}"),
            SysMenuButton::Macro => format!("{sysstring} {ALT_R}"),
            _ => sysstring.to_string(),
        }
    }
}

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    (
        crate::window_restore::x87_to_int(f64::from(viewport.0 as f32) - f64::from(SYSMENU_SIZE.0)),
        crate::window_restore::x87_to_int(f64::from(viewport.1 as f32) - f64::from(SYSMENU_SIZE.1) - f64::from(SHORTCUT_BAR_WIDTH)),
    )
}

pub fn initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    let (x, y) = default_position(viewport);
    crate::initial_position::InitialPosition {
        enabled: true, horizontal_mode: 1, vertical_mode: 4,
        x: viewport.0.wrapping_sub(x), y: viewport.1.wrapping_sub(y),
    }
}

pub fn grip() -> Rect {
    Rect::new(0, 0, SYSMENU_SIZE.0, TITLE_HEIGHT)
}

pub fn button_at(x: i32, y: i32) -> Option<SysMenuButton> {
    SysMenuButton::ALL.into_iter().find(|b| b.rect().contains(x, y))
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SysMenuView {
    pub title: String,
    pub labels: [String; BUTTONS],
    pub pressed: Option<SysMenuButton>,
    pub restart_locked: bool,
}

impl SysMenuView {
    pub fn with_sysstrings(lookup: impl Fn(u32) -> Option<String>) -> SysMenuView {
        let text = |id: u32| lookup(id).unwrap_or_else(|| format!("sysstring {id}"));
        SysMenuView {
            title: text(TITLE_SYSSTRING),
            labels: SysMenuButton::ALL.map(|b| b.label(&text(b.sysstring()))),
            pressed: None,
            restart_locked: false,
        }
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        let (w, h) = SYSMENU_SIZE;
        FrameCtrl::titled(&self.title, false).paint(list, x, y, w, h, font, alpha);
        list.draw_texture(x, y + TITLE_HEIGHT, w, h - TITLE_HEIGHT, 0, 0, w, h - TITLE_HEIGHT, SYSMENU_BACK, alpha);
        for (button, label) in SysMenuButton::ALL.into_iter().zip(&self.labels) {
            let rect = button.rect();
            let (normal, down) = button.textures();
            let texture = if self.pressed == Some(button) { down } else { normal };
            list.draw_texture(x + rect.x, y + rect.y, rect.w, rect.h, 0, 0, rect.w, rect.h, &texture, 1.0);
            list.draw_text(x + LABEL_X, y + LABEL_Y + BUTTON_STEP * button.index() as i32, TEXT, label, font);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view() -> SysMenuView {
        SysMenuView::with_sysstrings(|id| {
            Some(
                match id {
                    446 => "System Menu",
                    387 => "Community",
                    711 => "Macro",
                    145 => "Help",
                    470 => "Petition",
                    146 => "Options",
                    147 => "Restart",
                    148 => "Exit Game",
                    _ => return None,
                }
                .to_string(),
            )
        })
    }

    #[test]
    fn seven_buttons_of_36_pixels_every_37_from_y_29() {
        let rects: Vec<Rect> = SysMenuButton::ALL.iter().map(|b| b.rect()).collect();
        assert_eq!(rects[0], Rect::new(13, 29, 36, 36));
        assert_eq!(rects[1], Rect::new(13, 66, 36, 36));
        assert_eq!(rects[6], Rect::new(13, 251, 36, 36));
        assert_eq!(button_at(20, 40), Some(SysMenuButton::Community));
        assert_eq!(button_at(48, 286), Some(SysMenuButton::ExitGame));
        assert_eq!(button_at(13, 65), None);
        assert_eq!(button_at(60, 40), None);
        assert_eq!(
            SysMenuButton::ALL.map(|b| b.control_id()),
            [0xf0f024, 0xf0f022, 0xf0f006, 0xf0f023, 0xf0f007, 0xf0f008, 0xf0f009]
        );
        assert_eq!(SysMenuButton::ALL.map(|b| b.sysstring()), [387, 711, 145, 470, 146, 147, 148]);
    }

    #[test]
    fn labels_add_the_alt_keys_to_community_and_macro() {
        let view = view();
        assert_eq!(view.title, "System Menu");
        assert_eq!(view.labels[0], "Community (Alt+B)");
        assert_eq!(view.labels[1], "Macro (Alt+R)");
        assert_eq!(view.labels[2], "Help");
        assert_eq!(view.labels[6], "Exit Game");
        assert_eq!(SysMenuView::with_sysstrings(|_| None).labels[3], "sysstring 470");
    }

    #[test]
    fn the_window_sits_above_the_shortcut_bar_in_the_bottom_right_corner() {
        assert_eq!(SYSMENU_SIZE, (172, 295));
        assert_eq!(default_position((1558, 933)), (1386, 592));
        assert_eq!(grip(), Rect::new(0, 0, 172, 20));
    }
}
