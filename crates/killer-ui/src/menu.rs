use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{FrameCtrl, FrameMode};
use crate::window::Rect;

pub const MENU_SIZE: (i32, i32) = (173, 46);

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    (
        crate::window_restore::x87_to_int(f64::from(viewport.0 as f32) - f64::from(MENU_SIZE.0)),
        crate::window_restore::x87_to_int(f64::from(viewport.1 as f32) - f64::from(MENU_SIZE.1)),
    )
}

pub fn initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    let (x, y) = default_position(viewport);
    crate::initial_position::InitialPosition {
        enabled: true, horizontal_mode: 1, vertical_mode: 4,
        x: viewport.0.wrapping_sub(x), y: viewport.1.wrapping_sub(y),
    }
}

pub const MENU_BACK1: &str = "L2UI_ch3.SmallWnd.Smallwindow_back1";
pub const MENU_BACK2: &str = "L2UI_ch3.SmallWnd.Smallwindow_back2";
pub const MENU_BACK3: &str = "L2UI_ch3.SmallWnd.Smallwindow_back3";
pub const BODY_X: i32 = 12;
pub const BODY_END_WIDTH: i32 = 16;
pub const BODY_UW: i32 = 16;
pub const BODY_MID_INSET: i32 = 44;
pub const BUTTON_SIZE: i32 = 34;
pub const BUTTON_Y: i32 = 6;
pub const BUTTON_XS: [i32; 4] = [19, 56, 93, 130];
pub const BUTTON_TEXTURES: [&str; 4] = [
    "L2UI_ch3.MenuIcon.MenuButton1",
    "L2UI_ch3.MenuIcon.MenuButton2",
    "L2UI_ch3.MenuIcon.MenuButton3",
    "L2UI_ch3.MenuIcon.MenuButton4",
];
pub const BUTTON_DOWN_SUFFIX: &str = "_down";
pub const BUTTON_TOOLTIP_SYSSTRINGS: [u32; 4] = [434, 195, 925, 193];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuButton {
    CharacterStatus,
    Inventory,
    Map,
    System,
}

impl MenuButton {
    pub const ALL: [MenuButton; 4] =
        [MenuButton::CharacterStatus, MenuButton::Inventory, MenuButton::Map, MenuButton::System];

    pub fn index(self) -> usize {
        match self {
            MenuButton::CharacterStatus => 0,
            MenuButton::Inventory => 1,
            MenuButton::Map => 2,
            MenuButton::System => 3,
        }
    }

    pub fn rect(self) -> Rect {
        Rect::new(BUTTON_XS[self.index()], BUTTON_Y, BUTTON_SIZE, BUTTON_SIZE)
    }

    pub fn tooltip_sysstring(self) -> u32 {
        BUTTON_TOOLTIP_SYSSTRINGS[self.index()]
    }
}

pub fn button_at(x: i32, y: i32) -> Option<MenuButton> {
    MenuButton::ALL.into_iter().find(|b| b.rect().contains(x, y))
}

pub fn grip() -> Rect {
    FrameCtrl::new(FrameMode::VerticalGrip).grip(MENU_SIZE.0, MENU_SIZE.1)
}

pub fn paint_menu(list: &mut DrawList, x: i32, y: i32, pressed: Option<MenuButton>, font: &BitmapFont, alpha: f32) {
    let (w, h) = MENU_SIZE;
    FrameCtrl::new(FrameMode::VerticalGrip).paint(list, x, y, w, h, font, alpha);
    list.draw_texture(x + BODY_X, y, BODY_END_WIDTH, h, 0, 0, BODY_UW, h, MENU_BACK1, alpha);
    list.draw_texture(x + BODY_X + BODY_END_WIDTH, y, w - BODY_MID_INSET, h, 0, 0, BODY_UW, h, MENU_BACK2, alpha);
    list.draw_texture(x + w - BODY_END_WIDTH, y, BODY_END_WIDTH, h, 0, 0, BODY_UW, h, MENU_BACK3, alpha);
    for button in MenuButton::ALL {
        let rect = button.rect();
        let texture = BUTTON_TEXTURES[button.index()];
        let state =
            if pressed == Some(button) { format!("{texture}{BUTTON_DOWN_SUFFIX}") } else { texture.to_string() };
        list.draw_texture(x + rect.x, y + rect.y, rect.w, rect.h, 0, 0, rect.w, rect.h, &state, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::Quad;
    use crate::font::{Glyphs, gly};

    const TEXTURES: &str = include_str!("../tests/fixtures/nwindow-texturas-menu.txt");

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn painted(pressed: Option<MenuButton>) -> Vec<Quad> {
        let mut list = DrawList::default();
        paint_menu(&mut list, 0, 0, pressed, &font(), 1.0);
        list.quads
    }

    fn with_texture<'a>(quads: &'a [Quad], texture: &str) -> Vec<&'a Quad> {
        quads.iter().filter(|q| q.texture == texture).collect()
    }

    #[test]
    fn body_is_the_grip_and_three_small_window_slices_over_173_by_46() {
        let quads = painted(None);
        let grip = with_texture(&quads, crate::frame::SMALLBAR_2);
        assert_eq!((grip[0].x, grip[0].y, grip[0].w, grip[0].h), (0.0, 8.0, 12.0, 30.0));
        let back1 = with_texture(&quads, MENU_BACK1);
        assert_eq!(
            (back1[0].x, back1[0].y, back1[0].w, back1[0].h, back1[0].uw, back1[0].vh),
            (12.0, 0.0, 16.0, 46.0, 16.0, 46.0)
        );
        let back2 = with_texture(&quads, MENU_BACK2);
        assert_eq!((back2[0].x, back2[0].w, back2[0].uw, back2[0].vh), (28.0, 129.0, 16.0, 46.0));
        let back3 = with_texture(&quads, MENU_BACK3);
        assert_eq!((back3[0].x, back3[0].w, back3[0].uw), (157.0, 16.0, 16.0));
        assert_eq!(super::grip(), Rect::new(0, 0, 12, 46));
    }

    #[test]
    fn four_buttons_of_34_pixels_sit_at_19_56_93_and_130() {
        let quads = painted(None);
        for (button, x) in MenuButton::ALL.into_iter().zip([19.0, 56.0, 93.0, 130.0]) {
            let q = with_texture(&quads, BUTTON_TEXTURES[button.index()]);
            assert_eq!((q[0].x, q[0].y, q[0].w, q[0].h, q[0].uw, q[0].vh), (x, 6.0, 34.0, 34.0, 34.0, 34.0));
            assert_eq!(q[0].color[3], 1.0);
        }
        assert!(with_texture(&quads, "L2UI_ch3.MenuIcon.MenuButton2_down").is_empty());
        let pressed = painted(Some(MenuButton::Inventory));
        assert_eq!(with_texture(&pressed, "L2UI_ch3.MenuIcon.MenuButton2_down")[0].x, 56.0);
        assert!(with_texture(&pressed, "L2UI_ch3.MenuIcon.MenuButton2").is_empty());
        assert_eq!(MenuButton::Map.tooltip_sysstring(), 925);
        assert_eq!(MenuButton::System.tooltip_sysstring(), 193);
    }

    #[test]
    fn button_hit_test_follows_the_button_rects() {
        assert_eq!(button_at(20, 7), Some(MenuButton::CharacterStatus));
        assert_eq!(button_at(54, 20), None);
        assert_eq!(button_at(56 + 33, 6 + 33), Some(MenuButton::Inventory));
        assert_eq!(button_at(100, 30), Some(MenuButton::Map));
        assert_eq!(button_at(163, 39), Some(MenuButton::System));
        assert_eq!(button_at(164, 39), None);
        assert_eq!(button_at(5, 20), None);
        assert_eq!(button_at(20, 41), None);
    }

    #[test]
    fn frame_and_body_carry_the_window_alpha() {
        let mut list = DrawList::default();
        paint_menu(&mut list, 10, 20, None, &font(), 0.5);
        assert_eq!(with_texture(&list.quads, MENU_BACK2)[0].color[3], 0.5);
        assert_eq!(
            (with_texture(&list.quads, MENU_BACK1)[0].x, with_texture(&list.quads, MENU_BACK1)[0].y),
            (22.0, 20.0)
        );
        assert_eq!(with_texture(&list.quads, crate::frame::SMALLBAR_1)[0].color[3], 0.5);
        assert_eq!(with_texture(&list.quads, BUTTON_TEXTURES[0])[0].x, 29.0);
    }

    #[test]
    fn every_texture_exists_in_the_client_list() {
        let known: Vec<String> = TEXTURES.lines().map(|l| l.trim().to_lowercase()).collect();
        let mut quads = painted(Some(MenuButton::CharacterStatus));
        quads.extend(painted(Some(MenuButton::Inventory)));
        quads.extend(painted(Some(MenuButton::Map)));
        quads.extend(painted(Some(MenuButton::System)));
        for texture in quads.iter().map(|q| q.texture.as_str()) {
            assert!(known.contains(&texture.to_lowercase()), "textura desconhecida {texture}");
        }
    }
}
