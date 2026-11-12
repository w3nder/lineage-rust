use crate::button::TextButton;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::window::Rect;

pub const FUNCTION_SIZE: (i32, i32) = (120, 138);
pub const FUNCTION_MARGIN: (i32, i32) = (154, 236);
pub const FUNCTION_BACK: &str = "L2UI_CH3.LobbyWnd.lobby_menuback";
pub const BIG_BUTTON: &str = "L2UI_CH3.Button.BigButton";
pub const BIG_BUTTON_DOWN: &str = "L2UI_CH3.Button.BigButton_down";
pub const MENU_TITLE: &str = "L2UI_CH3.BenchMarkWnd.BenchMark_Title";
pub const MENU_TITLE_SOURCE: (i32, i32) = (0x400, 0x300);
pub const BUTTON_IDS: [u32; 4] = [0xaff22011, 0xaff22012, 0xaff22013, 0xaff22014];
pub const BUTTON_RECTS: [Rect; 4] = [Rect::new(13, 20, 96, 23), Rect::new(13, 45, 96, 23), Rect::new(13, 70, 96, 23), Rect::new(13, 95, 96, 23)];
pub const SYSSTRINGS: [u32; 4] = [1023, 1022, 156, 153];
pub const MESSAGE_NOTIFY: u32 = 0x501;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionAction {
    Resume,
    Benchmark,
    Options,
    Exit,
}

impl FunctionAction {
    pub fn from_id(id: u32) -> Option<FunctionAction> {
        match id {
            0xaff22011 => Some(FunctionAction::Resume),
            0xaff22012 => Some(FunctionAction::Benchmark),
            0xaff22013 => Some(FunctionAction::Options),
            0xaff22014 => Some(FunctionAction::Exit),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayFunctionWnd {
    pub screen: (i32, i32),
    pub labels: [String; 4],
    pub pressed: Option<usize>,
    pub visible: bool,
}

impl ReplayFunctionWnd {
    pub fn new(screen: (i32, i32), lookup: impl Fn(u32) -> Option<String>) -> ReplayFunctionWnd {
        let text = |id: u32| lookup(id).unwrap_or_default();
        ReplayFunctionWnd {
            screen,
            labels: [text(SYSSTRINGS[0]), text(SYSSTRINGS[1]), text(SYSSTRINGS[2]), text(SYSSTRINGS[3])],
            pressed: None,
            visible: true,
        }
    }

    pub fn origin(&self) -> (i32, i32) {
        (self.screen.0 - FUNCTION_MARGIN.0, self.screen.1 - FUNCTION_MARGIN.1)
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<usize> {
        let (ox, oy) = self.origin();
        BUTTON_RECTS.iter().position(|rect| rect.contains(x - ox, y - oy))
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) {
        self.pressed = if self.visible { self.hit(x, y) } else { None };
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<FunctionAction> {
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        FunctionAction::from_id(BUTTON_IDS[pressed])
    }

    pub fn paint(&self, list: &mut DrawList, font: &BitmapFont) {
        if !self.visible {
            return;
        }
        let (ox, oy) = self.origin();
        list.draw_texture(ox, oy, FUNCTION_SIZE.0, FUNCTION_SIZE.1, 0, 0, FUNCTION_SIZE.0, FUNCTION_SIZE.1, FUNCTION_BACK, 1.0);
        for (index, rect) in BUTTON_RECTS.iter().enumerate() {
            TextButton::new(*rect, BIG_BUTTON, BIG_BUTTON_DOWN, &self.labels[index]).paint(list, ox, oy, self.pressed == Some(index), font);
        }
    }
}

pub const LOGO_FALLBACK: &str = "L2Font.replay_logo-k";
pub const MINI_LOGO_FALLBACK: &str = "L2Font.start_minilogo-k";
pub const LOGO_SOURCE: (i32, i32) = (0x100, 0x40);
pub const MINI_LOGO_SOURCE: (i32, i32) = (0xE8, 0x78);
pub const LOGO_SCALE: f64 = 0.7;
pub const MINI_LOGO_SCALE: f64 = 0.8;
pub const LOGO_ORIGIN: f32 = 30.0;
pub const MINI_LOGO_RIGHT: f32 = 256.0;
pub const REFERENCE_SCREEN: (f64, f64) = (1024.0, 768.0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayLogoWnd {
    pub screen: (i32, i32),
    pub logo: String,
    pub mini_logo: String,
    pub visible: bool,
}

impl ReplayLogoWnd {
    pub fn new(screen: (i32, i32), logo: &str, mini_logo: &str) -> ReplayLogoWnd {
        ReplayLogoWnd { screen, logo: logo.to_string(), mini_logo: mini_logo.to_string(), visible: false }
    }

    pub fn from_localization(screen: (i32, i32), country: &str, ini: &crate::ini::IniFile) -> ReplayLogoWnd {
        let pick = |key: &str, fallback: &str| ini.get(country, &format!("{country}_{key}")).unwrap_or(fallback).to_string();
        ReplayLogoWnd {
            screen,
            logo: pick("ReplayLogoTexture", LOGO_FALLBACK),
            mini_logo: pick("MiniLogoTexture", MINI_LOGO_FALLBACK),
            visible: false,
        }
    }

    pub fn scale(&self) -> f64 {
        (f64::from(self.screen.0) / REFERENCE_SCREEN.0).max(f64::from(self.screen.1) / REFERENCE_SCREEN.1)
    }

    pub fn paint(&self, list: &mut DrawList) {
        if !self.visible {
            return;
        }
        let (sw, sh) = (self.screen.0 as f32, self.screen.1 as f32);
        let mini = (self.scale() * MINI_LOGO_SCALE) as f32;
        let logo = (self.scale() * LOGO_SCALE) as f32;
        let (mw, mh) = MINI_LOGO_SOURCE;
        list.draw_texture(
            (sw - mini * MINI_LOGO_RIGHT) as i32,
            (sh - mini * mh as f32) as i32,
            (mini * mw as f32) as i32,
            (mini * mh as f32) as i32,
            0,
            0,
            mw,
            mh,
            &self.mini_logo,
            1.0,
        );
        let (lw, lh) = LOGO_SOURCE;
        list.draw_texture(
            (logo * LOGO_ORIGIN) as i32,
            (logo * LOGO_ORIGIN) as i32,
            (logo * lw as f32) as i32,
            (logo * lh as f32) as i32,
            0,
            0,
            lw,
            lh,
            &self.logo,
            1.0,
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayPlayWnd {
    pub screen: (i32, i32),
    pub logo: String,
    pub mini_logo: String,
    pub visible: bool,
}

impl ReplayPlayWnd {
    pub fn new(screen: (i32, i32), logo: &str, mini_logo: &str) -> ReplayPlayWnd {
        ReplayPlayWnd { screen, logo: logo.to_string(), mini_logo: mini_logo.to_string(), visible: true }
    }

    pub fn paint(&self, _list: &mut DrawList) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayMenuWnd {
    pub screen: (i32, i32),
    pub function: ReplayFunctionWnd,
}

impl ReplayMenuWnd {
    pub fn new(screen: (i32, i32), lookup: impl Fn(u32) -> Option<String>) -> ReplayMenuWnd {
        ReplayMenuWnd { screen, function: ReplayFunctionWnd::new(screen, lookup) }
    }

    pub fn resize(&mut self, screen: (i32, i32)) {
        self.screen = screen;
        self.function.screen = screen;
    }

    pub fn paint(&self, list: &mut DrawList, font: &BitmapFont) {
        let (w, h) = self.screen;
        list.draw_texture(0, 0, w, h, 0, 0, MENU_TITLE_SOURCE.0, MENU_TITLE_SOURCE.1, MENU_TITLE, 1.0);
        self.function.paint(list, font);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn menu() -> ReplayMenuWnd {
        ReplayMenuWnd::new((1024, 768), |id| Some(format!("s{id}")))
    }

    #[test]
    fn layout_equals_the_executed_oncreate_0x100f9a00() {
        assert_eq!(BUTTON_RECTS.map(|rect| (rect.x, rect.y, rect.w, rect.h)), [(13, 20, 96, 23), (13, 45, 96, 23), (13, 70, 96, 23), (13, 95, 96, 23)]);
        assert_eq!(BUTTON_IDS, [0xaff22011, 0xaff22012, 0xaff22013, 0xaff22014]);
        assert_eq!(SYSSTRINGS, [1023, 1022, 156, 153]);
        assert_eq!(FUNCTION_SIZE, (120, 138));
    }

    #[test]
    fn function_window_is_anchored_to_the_right_and_bottom_margins_of_0x100fa0f0() {
        assert_eq!(menu().function.origin(), (1024 - 154, 768 - 236));
    }

    #[test]
    fn buttons_map_to_the_handlers_of_the_wndproc_0x100f8d80() {
        let mut menu = menu();
        let (ox, oy) = menu.function.origin();
        for (index, expected) in [FunctionAction::Resume, FunctionAction::Benchmark, FunctionAction::Options, FunctionAction::Exit].into_iter().enumerate() {
            let (x, y) = (ox + BUTTON_RECTS[index].x + 1, oy + BUTTON_RECTS[index].y + 1);
            menu.function.mouse_down(x, y);
            assert_eq!(menu.function.mouse_up(x, y), Some(expected));
        }
        let (x, y) = (ox + BUTTON_RECTS[0].x + 1, oy + BUTTON_RECTS[0].y + 1);
        menu.function.mouse_down(x, y);
        assert_eq!(menu.function.mouse_up(x + 500, y), None);
        assert_eq!(FunctionAction::from_id(0xaff22015), None);
    }

    #[test]
    fn menu_stretches_the_title_over_the_whole_window_and_draws_four_buttons() {
        let menu = menu();
        let mut draws = DrawList::default();
        menu.paint(&mut draws, &font());
        let first = &draws.quads[0];
        assert_eq!((first.texture.as_str(), first.w, first.h), (MENU_TITLE, 1024.0, 768.0));
        assert_eq!(draws.quads.iter().filter(|quad| quad.texture.as_str() == BIG_BUTTON).count(), 4);
        assert!(draws.quads.iter().any(|quad| quad.texture.as_str() == FUNCTION_BACK));
    }

    #[test]
    fn hidden_function_window_neither_paints_nor_presses() {
        let mut menu = menu();
        menu.function.visible = false;
        let (ox, oy) = menu.function.origin();
        menu.function.mouse_down(ox + 14, oy + 21);
        assert_eq!(menu.function.pressed, None);
        let mut draws = DrawList::default();
        menu.paint(&mut draws, &font());
        assert_eq!(draws.quads.len(), 1);
    }

    #[test]
    fn the_logo_window_scales_by_the_larger_ratio_and_anchors_the_small_logo_bottom_right() {
        let mut logo = ReplayLogoWnd::new((1024, 768), "logo", "mini");
        let mut list = DrawList::default();
        logo.paint(&mut list);
        assert!(list.quads.is_empty());
        logo.visible = true;
        logo.paint(&mut list);
        let quad = |name: &str| list.quads.iter().find(|q| q.texture == name).map(|q| (q.x, q.y, q.w, q.h, q.uw, q.vh, q.color[3]));
        assert_eq!(quad("mini"), Some((819.0, 672.0, 185.0, 96.0, 232.0, 120.0, 1.0)));
        assert_eq!(quad("logo"), Some((21.0, 21.0, 179.0, 44.0, 256.0, 64.0, 1.0)));
        logo.screen = (1280, 800);
        assert!((logo.scale() - 1.25).abs() < 1e-9);
    }

    #[test]
    fn the_logo_textures_come_from_the_localization_of_the_country_with_the_original_fallback() {
        let ini = crate::ini::parse_ini(include_str!("../tests/fixtures/localization.ini"));
        let english = ReplayLogoWnd::from_localization((1024, 768), "English", &ini);
        assert_eq!((english.logo.as_str(), english.mini_logo.as_str()), ("L2Font-e.Replay_logo-e", "L2Font-e.mini_logo-e"));
        let unknown = ReplayLogoWnd::from_localization((1024, 768), "Nowhere", &ini);
        assert_eq!((unknown.logo.as_str(), unknown.mini_logo.as_str()), (LOGO_FALLBACK, MINI_LOGO_FALLBACK));
    }

    #[test]
    fn the_play_window_keeps_the_two_logo_textures_and_paints_nothing_in_this_client() {
        let play = ReplayPlayWnd::new((1024, 768), "logo", "mini");
        let mut list = DrawList::default();
        play.paint(&mut list);
        assert!(list.quads.is_empty());
        assert_eq!((play.logo.as_str(), play.mini_logo.as_str()), ("logo", "mini"));
    }
}
