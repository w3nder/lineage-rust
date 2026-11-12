use crate::button::TextButton;
use crate::color::TEXT;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::window::Rect;

pub const SIZE: (i32, i32) = (205, 87);
pub const BACK: &str = "L2UI_ch3.dialog.system_back";
pub const BACK_SOURCE: (i32, i32) = (205, 87);
pub const TEXT_POS: (i32, i32) = (10, 10);
pub const TEXT_MARGIN: i32 = 20;
pub const OK_RECT: Rect = Rect::new(65, 60, 74, 21);
pub const OK_ID: u32 = 0x1004;
pub const BUTTON_NORMAL: &str = "L2UI_ch3.button.btn1_normal";
pub const BUTTON_DOWN: &str = "L2UI_ch3.button.btn1_normalon";
pub const SYSMSG_TEXT: u32 = 1239;
pub const SYSSTRING_OK: u32 = 140;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyCryptWarning {
    pub screen: (i32, i32),
    pub text: String,
    pub ok: String,
    pub visible: bool,
    pub pressed: bool,
}

impl KeyCryptWarning {
    pub fn new(screen: (i32, i32), text: String, ok: String) -> KeyCryptWarning {
        KeyCryptWarning { screen, text, ok, visible: false, pressed: false }
    }

    pub fn origin(&self) -> (i32, i32) {
        ((self.screen.0 as f32 / 2.0 - 101.0) as i32, (self.screen.1 as f32 / 2.0 - 43.0) as i32)
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.pressed = false;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.pressed = false;
    }

    fn on_button(&self, x: i32, y: i32) -> bool {
        let (ox, oy) = self.origin();
        OK_RECT.contains(x - ox, y - oy)
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        let (ox, oy) = self.origin();
        self.visible && Rect::new(ox, oy, SIZE.0, SIZE.1).contains(x, y)
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) {
        self.pressed = self.visible && self.on_button(x, y);
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> bool {
        let pressed = std::mem::take(&mut self.pressed);
        if pressed && self.on_button(x, y) {
            self.hide();
            return true;
        }
        false
    }

    pub fn paint(&self, list: &mut DrawList, font: &BitmapFont) {
        if !self.visible {
            return;
        }
        let (ox, oy) = self.origin();
        list.draw_texture(ox, oy, BACK_SOURCE.0, BACK_SOURCE.1, 0, 0, BACK_SOURCE.0, BACK_SOURCE.1, BACK, 1.0);
        let width = SIZE.0 - TEXT_MARGIN;
        let lines = crate::text_align::bitmap_lines(&self.text, width, font, 1, false);
        crate::text_align::paint(list, (ox + TEXT_POS.0, oy + TEXT_POS.1), &lines, TEXT, font, true);
        TextButton::new(OK_RECT, BUTTON_NORMAL, BUTTON_DOWN, &self.ok).paint(list, ox, oy, self.pressed, font);
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

    fn dialog() -> KeyCryptWarning {
        KeyCryptWarning::new((1024, 768), "aviso do modulo de teclado".into(), "OK".into())
    }

    #[test]
    fn it_is_centered_on_the_whole_screen_and_starts_hidden() {
        let dialog = dialog();
        assert_eq!(dialog.origin(), (1024 / 2 - 101, 768 / 2 - 43));
        let mut list = DrawList::default();
        dialog.paint(&mut list, &font());
        assert!(list.quads.is_empty());
    }

    #[test]
    fn ok_hides_it_only_when_released_over_the_button() {
        let mut dialog = dialog();
        dialog.show();
        let (ox, oy) = dialog.origin();
        let (x, y) = (ox + OK_RECT.x + 1, oy + OK_RECT.y + 1);
        dialog.mouse_down(x, y);
        assert!(!dialog.mouse_up(ox, oy));
        assert!(dialog.visible);
        dialog.mouse_down(x, y);
        assert!(dialog.mouse_up(x, y));
        assert!(!dialog.visible);
    }

    #[test]
    fn it_paints_the_back_at_its_own_size_the_wrapped_text_and_the_button() {
        let mut dialog = dialog();
        dialog.show();
        let mut list = DrawList { capture_text: true, ..Default::default() };
        dialog.paint(&mut list, &font());
        assert!(list.quads.iter().any(|quad| quad.texture == BACK && quad.w == 205.0 && quad.h == 87.0));
        assert!(list.quads.iter().any(|quad| quad.texture == BUTTON_NORMAL));
    }
}
