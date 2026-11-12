use crate::color::TEXT;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::window::Rect;

pub const TEXT_GAP: i32 = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountSpin {
    pub rect: Rect,
    pub background: Option<(String, (i32, i32))>,
    pub label: String,
    pub value_text: String,
    pub text_origin: (i32, i32),
    pub min: i32,
    pub max: i32,
    pub value: i32,
    pub cancel_step: bool,
}

pub const UP_ID: u32 = 0x100a;
pub const DOWN_ID: u32 = 0x100b;
pub const NOTIFY_UP: i32 = 1;
pub const NOTIFY_DOWN: i32 = 2;
pub const REPEAT_DELAY: u32 = 180;
pub const BACKGROUND: &str = "NWindow.EditBack";
pub const BUTTON_NORMAL: &str = "NWindow.WhiteTexture";
pub const BUTTON_PUSHED: &str = "NWindow.BlackTexture";

impl CountSpin {
    pub fn new(rect: Rect, min: i32, max: i32, label: Option<&str>) -> CountSpin {
        CountSpin {
            rect,
            background: None,
            label: label.unwrap_or("Default").to_string(),
            value_text: min.to_string(),
            text_origin: (5, 5),
            min,
            max,
            value: min,
            cancel_step: false,
        }
    }

    pub fn up_rect(&self) -> Rect {
        let (w, h) = (self.rect.w as f32, self.rect.h as f32);
        Rect::new((w - h + 5.0) as i32, 3, (h - 10.0) as i32, (0.5 * (h - 10.0)) as i32)
    }

    pub fn down_rect(&self) -> Rect {
        let (w, h) = (self.rect.w as f32, self.rect.h as f32);
        let half = 0.5 * (h - 10.0);
        Rect::new((w - h + 5.0) as i32, (h - (half + 5.0) + 2.0) as i32, (h - 10.0) as i32, half as i32)
    }

    pub fn on_up(&mut self, notify: impl FnOnce(i32)) {
        if self.value < self.max {
            self.value += 1;
            notify(NOTIFY_UP);
            if self.cancel_step {
                self.value -= 1;
                self.cancel_step = false;
            }
            self.value_text = self.value.to_string();
        }
    }

    pub fn on_down(&mut self, notify: impl FnOnce(i32)) {
        if self.value > self.min {
            self.value -= 1;
            notify(NOTIFY_DOWN);
            if self.cancel_step {
                self.value += 1;
                self.cancel_step = false;
            }
            self.value_text = self.value.to_string();
        }
    }

    pub fn command(&mut self, id: u32, notify: impl FnOnce(i32)) -> bool {
        match id {
            UP_ID => self.on_up(notify),
            DOWN_ID => self.on_down(notify),
            _ => return false,
        }
        true
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont) {
        let x = origin.0 + self.rect.x;
        let y = origin.1 + self.rect.y;
        let mut body = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
        if let Some((texture, (tw, th))) = &self.background {
            body.draw_texture(x, y, self.rect.w, self.rect.h, 0, 0, *tw, *th, texture, 1.0);
        }
        let start = x + self.text_origin.0;
        let width = body.draw_text(start, y + self.text_origin.1, TEXT, &self.label, font) - start;
        body.draw_text(start + width + TEXT_GAP, y + self.text_origin.1, TEXT, &self.value_text, font);
        list.append_clipped(body, Rect::new(x, y, self.rect.w, self.rect.h));
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

    #[test]
    fn count_spin_100070c0_draws_the_back_then_label_and_number_five_pixels_apart() {
        let font = font();
        let mut spin = CountSpin::new(Rect::new(10, 20, 80, 17), 12, 20, Some("ea"));
        spin.background = Some(("back".into(), (64, 16)));
        spin.text_origin = (3, 2);
        let mut list = DrawList::default();
        spin.paint(&mut list, (100, 200), &font);
        let back: Vec<_> = list.quads.iter().filter(|q| q.texture == "back").collect();
        assert_eq!(back.len(), 1);
        assert_eq!((back[0].x, back[0].y, back[0].w, back[0].h, back[0].uw, back[0].vh), (110.0, 220.0, 80.0, 17.0, 64.0, 16.0));
        let text: Vec<_> = list.quads.iter().filter(|q| q.texture == "f").collect();
        assert_eq!(text.len(), 4);
        assert_eq!((text[0].x, text[0].y), (113.0, 222.0));
        assert_eq!(text[0].color, TEXT.rgba());
        let label_width = font.measure("ea") as f32;
        assert_eq!((text[2].x, text[2].y), (113.0 + label_width + 5.0, 222.0));
        spin.background = None;
        let mut list = DrawList::default();
        spin.paint(&mut list, (0, 0), &font);
        assert!(list.quads.iter().all(|q| q.texture == "f"));
    }

    #[test]
    fn count_spin_10007560_starts_at_the_minimum_with_the_default_label() {
        let spin = CountSpin::new(Rect::new(0, 0, 80, 20), 3, 9, None);
        assert_eq!((spin.value, spin.value_text.as_str(), spin.label.as_str(), spin.text_origin), (3, "3", "Default", (5, 5)));
    }

    #[test]
    fn count_spin_10006dd0_and_10006e30_step_notify_and_honour_the_cancel() {
        let mut spin = CountSpin::new(Rect::new(0, 0, 80, 20), 0, 2, None);
        let mut sent = Vec::new();
        assert!(spin.command(UP_ID, |code| sent.push(code)));
        assert!(spin.command(UP_ID, |code| sent.push(code)));
        assert!(spin.command(UP_ID, |code| sent.push(code)));
        assert_eq!((spin.value, spin.value_text.as_str(), sent.as_slice()), (2, "2", [1, 1].as_slice()));
        spin.cancel_step = true;
        spin.on_down(|code| sent.push(code));
        assert_eq!((spin.value, spin.value_text.as_str(), spin.cancel_step), (2, "2", false));
        assert!(!spin.command(0x100c, |_| {}));
    }

    #[test]
    fn count_spin_10007300_places_the_two_buttons_at_the_right_end() {
        let spin = CountSpin::new(Rect::new(0, 0, 80, 20), 0, 9, None);
        assert_eq!(spin.up_rect(), Rect::new(65, 3, 10, 5));
        assert_eq!(spin.down_rect(), Rect::new(65, 12, 10, 5));
    }
}
