use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{FrameCtrl, TITLE_HEIGHT};
use crate::keys::MainTab;
use crate::window::Rect;

pub const MAIN_SIZE: (i32, i32) = (256, 401);
pub const MAIN_DEFAULT_X: i32 = 0;
pub const MAIN_DEFAULT_Y_RISE: i32 = 252;
pub const TAB_BACK: &str = "L2UI_CH3.TabButton.MainWndTabBack";
pub const TAB_BACK_RECT: Rect = Rect::new(0, 20, 256, 46);
pub const TAB_CTRL_RECT: Rect = Rect::new(12, 25, 224, 41);
pub const TAB_BUTTON_SIZE: (i32, i32) = (44, 41);
pub const TAB_GAP: i32 = 1;
pub const PAGE_RECT: Rect = Rect::new(0, 66, 256, 335);
pub const TAB_CTRL_ID: u32 = 0xf2a7;
pub const FRAME_ID: u32 = 0xf2a6;

pub fn tab_texture(tab: MainTab, active: bool) -> String {
    let n = tab.index() + 1;
    if active {
        format!("L2UI_CH3.TabButton.MainWndTabBtn{n}On")
    } else {
        format!("L2UI_CH3.TabButton.MainWndTabBtn{n}")
    }
}

pub fn tab_rect(tab: MainTab) -> Rect {
    let (w, h) = TAB_BUTTON_SIZE;
    Rect::new(TAB_CTRL_RECT.x + (w + TAB_GAP) * tab.index() as i32, TAB_CTRL_RECT.y, w, h)
}

pub fn tab_at(x: i32, y: i32) -> Option<MainTab> {
    MainTab::ALL.into_iter().find(|tab| tab_rect(*tab).contains(x, y))
}

pub fn grip() -> Rect {
    Rect::new(0, 0, MAIN_SIZE.0, TITLE_HEIGHT)
}

pub fn close_button() -> Rect {
    Rect::new(
        MAIN_SIZE.0 - crate::frame::CLOSE_INSET,
        crate::frame::BUTTON_Y,
        crate::frame::BUTTON_SIZE,
        crate::frame::BUTTON_SIZE,
    )
}

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    (MAIN_DEFAULT_X, crate::window_restore::x87_to_int(f64::from(viewport.1 as f32) * 0.5 - f64::from(MAIN_DEFAULT_Y_RISE)))
}

pub fn initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    let (x, y) = default_position(viewport);
    crate::initial_position::InitialPosition {
        enabled: true,
        horizontal_mode: 0,
        vertical_mode: 5,
        x,
        y: (viewport.1 / 2).wrapping_sub(y),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MainWndView {
    pub tab: MainTab,
    pub titles: [String; 5],
}

impl MainWndView {
    pub fn with_sysstrings(lookup: impl Fn(u32) -> Option<String>) -> MainWndView {
        let titles = MainTab::ALL
            .map(|tab| lookup(tab.title_sysstring()).unwrap_or_else(|| format!("sysstring {}", tab.title_sysstring())));
        MainWndView { tab: MainTab::Status, titles }
    }

    pub fn title(&self) -> &str {
        &self.titles[self.tab.index()]
    }

    pub fn paint_chrome(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        let (w, h) = MAIN_SIZE;
        FrameCtrl::titled(self.title(), true).paint(list, x, y, w, h, font, alpha);
        let back = TAB_BACK_RECT;
        list.draw_texture(x + back.x, y + back.y, back.w, back.h, 0, 0, back.w, back.h, TAB_BACK, alpha);
        for tab in MainTab::ALL {
            let rect = tab_rect(tab);
            let texture = tab_texture(tab, tab == self.tab);
            list.draw_texture(x + rect.x, y + rect.y, rect.w, rect.h, 0, 0, rect.w, rect.h, &texture, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_tabs_of_44_by_41_fill_the_224_pixel_tab_control() {
        assert_eq!(tab_rect(MainTab::Status), Rect::new(12, 25, 44, 41));
        assert_eq!(tab_rect(MainTab::Skills), Rect::new(57, 25, 44, 41));
        assert_eq!(tab_rect(MainTab::Quests), Rect::new(192, 25, 44, 41));
        assert_eq!(tab_rect(MainTab::Quests).x + 44, TAB_CTRL_RECT.x + TAB_CTRL_RECT.w);
        assert_eq!(tab_at(13, 30), Some(MainTab::Status));
        assert_eq!(tab_at(56, 30), None);
        assert_eq!(tab_at(100, 60), Some(MainTab::Skills));
        assert_eq!(tab_at(100, 66), None);
        assert_eq!(tab_at(235, 40), Some(MainTab::Quests));
        assert_eq!(PAGE_RECT, Rect::new(0, 66, 256, 335));
        assert_eq!(grip(), Rect::new(0, 0, 256, 20));
        assert_eq!(close_button(), Rect::new(233, 3, 15, 15));
        assert_eq!(default_position((1558, 933)), (0, 214));
    }
}
