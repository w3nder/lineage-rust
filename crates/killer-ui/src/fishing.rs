use crate::color::Argb;
use crate::draw::{DrawList, XLARGE_NUMBER_STRIP};
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::window::Rect;

pub const SIZE: (i32, i32) = (256, 276);
pub const POSITION: (i32, i32) = (100, 100);
pub const FISHING_BACK: &str = "L2UI_CH3.FishingWnd.fishing_back";
pub const FISHING_CLOCKICON: &str = "L2UI_CH3.FishingWnd.fishing_clockicon";
pub const FISHING_BAR1: &str = "L2UI_CH3.FishingWnd.fishing_bar1";
pub const FISHING_BAR2: &str = "L2UI_CH3.FishingWnd.fishing_bar2";
pub const FISHING_EFFECT: &str = "L2UI_CH3.FishingWnd.fishing_effect";
pub const EXHAUSTED_ICON: &str = "Icon.skill_i.skill1313";
pub const RESISTING_ICON: &str = "Icon.skill_i.skill1314";
pub const VIEW_RECT: Rect = Rect::new(2, 20, 250, 234);
pub const VIEWPORT_TEXTURE: &str = "fishing:viewport";
pub const VIEWPORT: crate::viewport_wnd::ViewportWnd = crate::viewport_wnd::ViewportWnd::new((2, 20), (252.0, 254.0));
pub const VIEWPORT_SOURCE: i32 = 256;
const MESSAGE_POSITIVE: u32 = 0xff0000;
const MESSAGE_NEUTRAL: u32 = 0xdcdcdc;
const DIGIT_COLOR: u32 = 0xffdcdcdc;
const HINT_COLOR: u32 = 0xffe7dcb6;
const BAR_FULL: i32 = 230;
const BAR_START: i32 = 115;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FishMessage {
    pub text: String,
    pub value: i32,
    pub alpha: u8,
    pub positive: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FishPaint {
    pub started: bool,
    pub time: i32,
    pub bar_width: Option<i32>,
    pub effect_alpha: Option<u8>,
    pub message: Option<FishMessage>,
    pub hint: Option<(bool, String)>,
    pub viewport: bool,
}

pub fn paint(state: &FishPaint, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) {
    let (x, y) = origin;
    FrameCtrl::titled("", false).paint(list, x, y, SIZE.0, SIZE.1, font, alpha);
    list.draw_texture(x, y + 20, 256, 256, 0, 0, 256, 256, FISHING_BACK, alpha);
    if state.viewport {
        VIEWPORT.draw(list, origin, VIEWPORT_TEXTURE, VIEWPORT_SOURCE, alpha);
    }
    if let Some(message) = &state.message {
        let a = u32::from(message.alpha) << 24;
        let color = a | if message.positive { MESSAGE_POSITIVE } else { MESSAGE_NEUTRAL };
        list.draw_text(x + 13, y + 256, Argb(color), &message.text, font);
        if message.value != 0 {
            let value = message.value.to_string();
            let width = font.measure(&value) as i32;
            let color = a | if message.value > 0 { MESSAGE_POSITIVE } else { MESSAGE_NEUTRAL };
            list.draw_text(x + 242 - width, y + 256, Argb(color), &value, font);
        }
    }
    if state.started {
        list.draw_texture(x + 110, y + 215, 12, 16, 0, 0, 12, 16, FISHING_CLOCKICON, alpha);
        list.draw_digits_with(XLARGE_NUMBER_STRIP, x + 125, y + 216, &state.time.to_string(), Argb(DIGIT_COLOR));
        if let Some(width) = state.bar_width {
            list.draw_texture(x + 13, y + 236, width, 11, 0, 0, 8, 11, FISHING_BAR1, alpha);
            list.draw_texture(x + 13 + width, y + 236, BAR_FULL - width, 11, 0, 0, 8, 11, FISHING_BAR2, alpha);
        }
        if let Some(effect) = state.effect_alpha {
            list.draw_texture(x - 22, y + 221, 297, 41, 0, 0, 297, 41, FISHING_EFFECT, f32::from(effect) / 255.0);
        }
    }
    if let Some((exhausted, text)) = &state.hint {
        let icon = if *exhausted { EXHAUSTED_ICON } else { RESISTING_ICON };
        list.draw_texture(x + 13, y + 33, 32, 32, 0, 0, 32, 32, icon, alpha);
        list.draw_text_clipped(x + 53, y + 33, SIZE.0 - 63, Argb(HINT_COLOR), text, font);
    }
}

pub fn bar_width(hp: i32, start_hp: i32) -> Option<i32> {
    (start_hp != 0).then(|| hp * BAR_START / start_hp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};

    #[test]
    fn the_viewport_quad_is_drawn_only_with_a_render_and_covers_the_region_of_0x100135f0() {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        let font = BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap());
        let mut list = DrawList::default();
        paint(&FishPaint::default(), &mut list, (100, 100), &font, 1.0);
        assert!(list.quads.iter().all(|quad| &*quad.texture != VIEWPORT_TEXTURE));
        let mut list = DrawList::default();
        paint(&FishPaint { viewport: true, ..FishPaint::default() }, &mut list, (100, 100), &font, 1.0);
        let quad = list.quads.iter().find(|quad| &*quad.texture == VIEWPORT_TEXTURE).unwrap();
        assert_eq!((quad.x, quad.y, quad.w, quad.h), (102.0, 120.0, 252.0, 254.0));
        assert_eq!((quad.uw, quad.vh), (256.0, 256.0));
    }

    #[test]
    fn the_initial_hp_fills_half_of_the_bar() {
        assert_eq!(bar_width(100, 100), Some(115));
        assert_eq!(bar_width(50, 100), Some(57));
        assert_eq!(bar_width(1, 0), None);
    }
}
