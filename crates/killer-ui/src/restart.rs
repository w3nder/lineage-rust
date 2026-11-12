use crate::button::TextButton;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::window::Rect;

pub const RESTART_SIZE: (i32, i32) = (128, 128);
pub const RESTART_INITIAL_POSITION: crate::initial_position::InitialPosition = crate::initial_position::InitialPosition {
    enabled: true, horizontal_mode: 2, vertical_mode: 2, x: RESTART_SIZE.0, y: RESTART_SIZE.1,
};
pub const RESTART_BACK: &str = "L2UI_CH3.SystemMenuWnd.system2_back";
pub const RESTART_BACK_SIZE: (i32, i32) = (141, 131);
pub const BIG_BUTTON: &str = "L2UI_CH3.button.bigbutton";
pub const BIG_BUTTON_DOWN: &str = "L2UI_CH3.button.bigbutton_down";
pub const BUTTON_X: i32 = 16;
pub const BUTTON_SIZE: (i32, i32) = (96, 23);
pub const BUTTON_STEP: i32 = 24;
pub const FIRST_BUTTON_Y: i32 = 6;
pub const BUTTON_SYSSTRINGS: [u32; 5] = [372, 374, 375, 373, 465];
pub const BUTTON_DEFAULT_LABELS: [&str; 5] = ["To Village", "To Hideaway", "To Castle", "To Siege HQ", "Fixed"];
pub const RESTART_POINTS: [i32; 5] = [0, 1, 2, 3, 4];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestartView {
    pub labels: [String; 5],
    pub enabled: [bool; 5],
}

impl RestartView {
    pub fn new(enabled: [bool; 5]) -> RestartView {
        RestartView { labels: BUTTON_DEFAULT_LABELS.map(String::from), enabled }
    }

    pub fn buttons(&self) -> Vec<TextButton> {
        let (w, h) = BUTTON_SIZE;
        (0..5)
            .map(|i| TextButton {
                enabled: self.enabled[i],
                ..TextButton::new(
                    Rect::new(BUTTON_X, FIRST_BUTTON_Y + i as i32 * BUTTON_STEP, w, h),
                    BIG_BUTTON,
                    BIG_BUTTON_DOWN,
                    &self.labels[i],
                )
            })
            .collect()
    }

    pub fn button_at(&self, x: i32, y: i32) -> Option<usize> {
        self.buttons().iter().position(|b| b.enabled && b.contains(x, y))
    }

    pub fn restart_point(&self, index: usize) -> Option<i32> {
        (self.enabled.get(index) == Some(&true)).then(|| RESTART_POINTS[index])
    }
}

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    (
        crate::window_restore::x87_to_int(f64::from(viewport.0 as f32) * 0.5 - 64.0),
        crate::window_restore::x87_to_int(f64::from(viewport.1 as f32) * 0.5 - 64.0),
    )
}

pub fn paint_restart(
    list: &mut DrawList,
    x: i32,
    y: i32,
    view: &RestartView,
    pressed: Option<usize>,
    font: &BitmapFont,
) {
    let (bw, bh) = RESTART_BACK_SIZE;
    list.draw_texture(x, y, bw, bh, 0, 0, bw, bh, RESTART_BACK, 1.0);
    for (index, button) in view.buttons().iter().enumerate().filter(|(_, b)| b.enabled) {
        button.paint(list, x, y, pressed == Some(index), font);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::{BUTTON_TEXT, BUTTON_TEXT_DISABLED};
    use crate::draw::Quad;
    use crate::font::{Glyphs, gly};

    const TEXTURES: &str = include_str!("../tests/fixtures/nwindow-texturas-restart.txt");

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn with_texture<'a>(quads: &'a [Quad], texture: &str) -> Vec<&'a Quad> {
        quads.iter().filter(|q| q.texture == texture).collect()
    }

    fn view() -> RestartView {
        RestartView::new([true, false, true, false, false])
    }

    fn painted(view: &RestartView, pressed: Option<usize>) -> Vec<Quad> {
        let mut list = DrawList::default();
        paint_restart(&mut list, 0, 0, view, pressed, &font());
        list.quads
    }

    #[test]
    fn the_back_is_141_by_131_with_five_big_buttons_every_24_pixels_from_six() {
        let quads = painted(&view(), None);
        let back = with_texture(&quads, RESTART_BACK);
        assert_eq!(
            (back[0].x, back[0].y, back[0].w, back[0].h, back[0].uw, back[0].vh),
            (0.0, 0.0, 141.0, 131.0, 141.0, 131.0)
        );
        let buttons = with_texture(&quads, BIG_BUTTON);
        assert_eq!(buttons.len(), 2);
        assert_eq!((buttons[0].x, buttons[0].y, buttons[0].w, buttons[0].h), (16.0, 6.0, 96.0, 23.0));
        assert_eq!((buttons[1].x, buttons[1].y), (16.0, 6.0 + 24.0 * 2.0));
        let all = painted(&RestartView::new([true; 5]), None);
        for (i, b) in with_texture(&all, BIG_BUTTON).iter().enumerate() {
            assert_eq!((b.x, b.y, b.w, b.h), (16.0, 6.0 + 24.0 * i as f32, 96.0, 23.0));
        }
        assert!(with_texture(&quads, BIG_BUTTON_DOWN).is_empty());
        let labels: Vec<&Quad> = quads.iter().filter(|q| q.texture == "f").collect();
        let village: Vec<&&Quad> = labels.iter().filter(|q| q.y == 6.0 + 11.0 - 6.0).collect();
        assert_eq!(village.len(), "To Village".replace(' ', "").len());
        assert_eq!(village[0].x, (16 + 48 - 30) as f32);
        assert!(village.iter().all(|q| q.color == BUTTON_TEXT.rgba()));
        assert!(labels.iter().all(|q| q.y != 30.0 + 11.0 - 6.0));
        assert!(labels.iter().all(|q| q.color != BUTTON_TEXT_DISABLED.rgba()));
        assert_eq!(default_position((1558, 933)), (715, 402));
    }

    #[test]
    fn pressed_enabled_buttons_use_the_down_texture_and_map_to_restart_points() {
        let view = view();
        let quads = painted(&view, Some(2));
        assert_eq!(with_texture(&quads, BIG_BUTTON_DOWN)[0].y, 54.0);
        assert_eq!(with_texture(&painted(&view, Some(1)), BIG_BUTTON_DOWN).len(), 0);
        assert_eq!(view.button_at(20, 10), Some(0));
        assert_eq!(view.button_at(20, 30), None);
        assert_eq!(RestartView::new([true; 5]).button_at(20, 30), Some(1));
        assert_eq!(RestartView::new([true; 5]).button_at(111, 102 + 22), Some(4));
        assert_eq!(view.button_at(111, 102 + 22), None);
        assert_eq!(view.button_at(112, 110), None);
        assert_eq!(view.button_at(20, 29), None);
        assert_eq!(view.restart_point(0), Some(0));
        assert_eq!(view.restart_point(1), None);
        assert_eq!(view.restart_point(2), Some(2));
        assert_eq!(view.restart_point(9), None);
        assert_eq!(BUTTON_SYSSTRINGS, [372, 374, 375, 373, 465]);
    }

    #[test]
    fn every_texture_exists_in_the_client_list() {
        let known: Vec<String> = TEXTURES.lines().map(|l| l.trim().to_lowercase()).collect();
        for texture in painted(&view(), Some(0)).iter().map(|q| q.texture.as_str()).filter(|t| *t != "f") {
            assert!(known.contains(&texture.to_lowercase()), "textura desconhecida {texture}");
        }
    }
}
