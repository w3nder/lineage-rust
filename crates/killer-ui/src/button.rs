use crate::color::{BUTTON_TEXT, BUTTON_TEXT_DISABLED, CHECK_TEXT, CHECK_TEXT_DISABLED};
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::window::Rect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextButton {
    pub rect: Rect,
    pub normal: &'static str,
    pub down: &'static str,
    pub label: String,
    pub enabled: bool,
    pub text_offset: (i32, i32),
}

impl TextButton {
    pub fn new(rect: Rect, normal: &'static str, down: &'static str, label: &str) -> TextButton {
        TextButton { rect, normal, down, label: label.to_string(), enabled: true, text_offset: (0, 0) }
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        self.rect.contains(x, y)
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, pressed: bool, font: &BitmapFont) {
        let Rect { x: bx, y: by, w, h } = self.rect;
        let start = list.quads.len();
        let texture = if pressed { self.down } else { self.normal };
        list.draw_button_texture(x + bx, y + by, w, h, 0, 0, w, h, texture, 1.0, self.enabled);
        if !self.label.is_empty() {
            let tw = font.measure(&self.label) as i32;
            let th = font.line_height as i32;
            let tx = (self.text_offset.0 as f64 + w as f64 * 0.5 - (tw / 2) as f64) as i32;
            let ty = (self.text_offset.1 as f64 + h as f64 * 0.5 - (th / 2) as f64) as i32;
            let color = if self.enabled { BUTTON_TEXT } else { BUTTON_TEXT_DISABLED };
            list.draw_text(x + bx + tx, y + by + ty, color, &self.label, font);
        }
        list.clip_from(start, Rect::new(x + bx, y + by, w, h));
    }
}

pub const CHECK_BOX: &str = "L2UI.Control.CheckBox";
pub const CHECK_BOX_CHECKED: &str = "L2UI.Control.CheckBox_Checked";
pub const CHECK_BOX_UNABLE: &str = "L2UI.Control.CheckBox_Unable";
pub const CHECK_BOX_CHECKED_UNABLE: &str = "L2UI.Control.CheckBox_Checked_unable";
pub const CHECK_ART: i32 = 12;
pub const CHECK_LABEL_GAP: i32 = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckBox {
    pub rect: Rect,
    pub label: String,
    pub checked: bool,
    pub enabled: bool,
    pub label_left: bool,
}

impl CheckBox {
    pub fn new(rect: Rect, label: &str) -> CheckBox {
        CheckBox { rect, label: label.to_string(), checked: false, enabled: true, label_left: false }
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        self.rect.contains(x, y)
    }

    pub fn mouse_down(&mut self) -> Option<bool> {
        if !self.enabled {
            return None;
        }
        self.checked = !self.checked;
        Some(self.checked)
    }

    pub fn texture(&self) -> &'static str {
        match (self.checked, self.enabled) {
            (true, true) => CHECK_BOX_CHECKED,
            (false, true) => CHECK_BOX,
            (true, false) => CHECK_BOX_CHECKED_UNABLE,
            (false, false) => CHECK_BOX_UNABLE,
        }
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        let Rect { x: bx, y: by, w, h } = self.rect;
        let start = list.quads.len();
        let color = if self.enabled { CHECK_TEXT } else { CHECK_TEXT_DISABLED };
        let (box_x, text_x) = if self.label_left {
            (x + bx + font.measure(&self.label) as i32 + CHECK_LABEL_GAP, x + bx)
        } else {
            (x + bx, x + bx + CHECK_ART + CHECK_LABEL_GAP)
        };
        list.draw_texture(box_x, y + by, CHECK_ART, CHECK_ART, 0, 0, CHECK_ART, CHECK_ART, self.texture(), 1.0);
        list.draw_text(text_x, y + by, color, &self.label, font);
        list.clip_from(start, Rect::new(x + bx, y + by, w, h));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::Quad;
    use crate::font::{Glyphs, gly};

    #[test]
    fn check_box_10005420_toggles_before_notification_and_preserves_disabled_state() {
        for enabled in [false, true] {
            for checked in [false, true] {
                let mut check = CheckBox::new(Rect::new(25, 199, 100, 17), "label");
                check.checked = checked;
                check.enabled = enabled;
                let before = check.clone();
                assert_eq!(check.mouse_down(), enabled.then_some(!checked));
                assert_eq!(check.checked, if enabled { !checked } else { checked });
                assert_eq!(check.rect, before.rect);
                assert_eq!(check.label, before.label);
                assert_eq!(check.enabled, before.enabled);
                assert_eq!(check.mouse_down(), enabled.then_some(checked));
                assert_eq!(check, before);
            }
        }
    }

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn with_texture<'a>(quads: &'a [Quad], texture: &str) -> Vec<&'a Quad> {
        quads.iter().filter(|q| q.texture == texture).collect()
    }

    #[test]
    fn the_state_texture_fills_the_rect_and_the_label_is_centered() {
        let button = TextButton::new(Rect::new(51, 75, 76, 23), "n", "d", "OK");
        let mut list = DrawList::default();
        button.paint(&mut list, 10, 20, false, &font());
        let back = with_texture(&list.quads, "n");
        assert_eq!(
            (back[0].x, back[0].y, back[0].w, back[0].h, back[0].uw, back[0].vh),
            (61.0, 95.0, 76.0, 23.0, 76.0, 23.0)
        );
        let text = with_texture(&list.quads, "f");
        assert_eq!(text.len(), 2);
        assert_eq!((text[0].x, text[0].y), (61.0 + 38.0 - 6.0, 95.0 + 11.0 - 6.0));
        assert_eq!(text[0].color, BUTTON_TEXT.rgba());
        let mut pressed = DrawList::default();
        button.paint(&mut pressed, 0, 0, true, &font());
        assert_eq!(with_texture(&pressed.quads, "d").len(), 1);
        assert!(button.contains(60, 80));
        assert!(!button.contains(50, 80));
    }

    #[test]
    fn a_disabled_button_shows_the_state_face_and_greys_the_label() {
        let button = TextButton { enabled: false, ..TextButton::new(Rect::new(0, 0, 96, 23), "n", "d", "To Village") };
        let mut list = DrawList::default();
        button.paint(&mut list, 0, 0, true, &font());
        assert!(with_texture(&list.quads, "n").is_empty());
        assert_eq!(with_texture(&list.quads, "d").len(), 1);
        assert!(with_texture(&list.quads, "f").iter().all(|q| q.color == BUTTON_TEXT_DISABLED.rgba()));
        let mut list = DrawList::default();
        button.paint(&mut list, 0, 0, false, &font());
        assert_eq!(with_texture(&list.quads, "n").len(), 1);
    }

    #[test]
    fn the_label_is_moved_by_the_text_offset_and_clipped_to_the_button() {
        let mut button = TextButton::new(Rect::new(0, 0, 20, 23), "n", "d", "WWWWWWWWWW");
        button.text_offset = (2, 1);
        let mut list = DrawList::default();
        button.paint(&mut list, 0, 0, false, &font());
        let text = with_texture(&list.quads, "f");
        assert!(!text.is_empty());
        assert!(text.iter().all(|q| q.x >= 0.0 && q.x + q.w <= 20.0 && q.y >= 0.0 && q.y + q.h <= 23.0));
    }
    /// `NCCheckBox::OnPaint @ 0x10005450`: arte de 12x12 em `(0, 0)` e o rótulo quatro pixels
    /// depois dela, em `0xFFDCDCDC` quando ligada e `0xFF808080` quando não.
    #[test]
    fn the_check_box_art_matches_the_state_and_the_label_follows_it() {
        let font = font();
        let mut check = CheckBox::new(Rect::new(17, 61, 100, 12), "Chat");
        let art = |check: &CheckBox| {
            let mut list = DrawList::default();
            check.paint(&mut list, 10, 20, &font);
            list.quads
        };
        let quads = art(&check);
        let box_art = with_texture(&quads, CHECK_BOX);
        assert_eq!((box_art[0].x, box_art[0].y, box_art[0].w, box_art[0].h), (27.0, 81.0, 12.0, 12.0));
        assert_eq!((box_art[0].uw, box_art[0].vh, box_art[0].color[3]), (12.0, 12.0, 1.0));
        let label = with_texture(&quads, "f");
        assert_eq!(label.len(), 4);
        assert_eq!((label[0].x, label[0].y), (27.0 + 16.0, 81.0));
        assert!(label.iter().all(|q| q.color == CHECK_TEXT.rgba()));
        check.checked = true;
        assert_eq!(with_texture(&art(&check), CHECK_BOX_CHECKED).len(), 1);
        check.enabled = false;
        let quads = art(&check);
        assert_eq!(with_texture(&quads, CHECK_BOX_CHECKED_UNABLE).len(), 1);
        assert!(with_texture(&quads, "f").iter().all(|q| q.color == CHECK_TEXT_DISABLED.rgba()));
        check.checked = false;
        assert_eq!(with_texture(&art(&check), CHECK_BOX_UNABLE).len(), 1);
        assert!(check.contains(20, 65) && !check.contains(16, 65));
    }
}
