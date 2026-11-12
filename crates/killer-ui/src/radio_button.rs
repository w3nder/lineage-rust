use crate::color::TEXT;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::window::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RadioEvents {
    pub select_in_parent: bool,
    pub release_focus: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MoveEvents {
    pub enter: bool,
    pub leave: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RadioButton {
    pub enabled: bool,
    pub focused: bool,
    pub rect: Rect,
    pub pressed: bool,
    pub state: i32,
    pub inside: bool,
    pub tooltip: Option<String>,
    pub tooltip_texture: Option<(String, (i32, i32))>,
    pub size_memo: (i32, i32),
    pub group: i32,
    pub field_2d8: i32,
}

impl RadioButton {
    pub fn on_create(&mut self) {
        self.size_memo = (self.rect.w, self.rect.h);
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        self.rect.x <= x && x as f32 <= self.rect.x as f32 + self.rect.w as f32 && self.rect.y <= y && y as f32 <= self.rect.y as f32 + self.rect.h as f32
    }

    pub fn on_mouse_move(&mut self, x: i32, y: i32, over: bool) -> MoveEvents {
        let mut events = MoveEvents::default();
        if self.enabled {
            if over {
                self.state = if self.pressed { 1 } else { 2 };
                events.enter = true;
            } else {
                self.state = i32::from(self.pressed);
                events.leave = true;
            }
            self.inside = self.contains(x, y);
        }
        events
    }

    pub fn post_paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont) {
        let Some(text) = self.tooltip.as_deref().filter(|text| !text.is_empty() && self.inside) else {
            return;
        };
        let x = origin.0 + self.rect.x;
        let y = origin.1 + self.rect.y;
        let (tw, th) = (font.measure(text) as i32, font.line_height as i32);
        let mut body = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
        if let Some((texture, (uw, vh))) = &self.tooltip_texture {
            body.draw_texture(x, y, tw, th, 0, 0, *uw, *vh, texture, 1.0);
        }
        body.draw_text(x, y, TEXT, text, font);
        list.append_clipped(body, Rect::new(x, y, tw, th));
    }

    pub fn on_lbutton_down(&mut self) -> bool {
        true
    }

    pub fn on_lbutton_up(&mut self) -> (bool, RadioEvents) {
        let mut events = RadioEvents::default();
        if self.enabled {
            events.select_in_parent = true;
            events.release_focus = true;
            self.focused = false;
        }
        (true, events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radio_button_10001aa0_consumes_the_press_without_touching_state() {
        let mut radio = RadioButton { enabled: true, focused: true, ..Default::default() };
        let before = radio.clone();
        assert!(radio.on_lbutton_down());
        assert_eq!(radio, before);
    }

    #[test]
    fn radio_button_10001b20_selects_through_the_parent_and_drops_focus_only_when_enabled() {
        let mut radio = RadioButton { enabled: false, focused: true, ..Default::default() };
        assert_eq!(radio.on_lbutton_up(), (true, RadioEvents::default()));
        assert!(radio.focused);
        radio.enabled = true;
        assert_eq!(radio.on_lbutton_up(), (true, RadioEvents { select_in_parent: true, release_focus: true }));
        assert!(!radio.focused);
    }

    fn font() -> BitmapFont {
        use crate::font::{Glyphs, gly};
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    #[test]
    fn radio_button_10001ed0_remembers_the_created_size() {
        let mut radio = RadioButton { rect: Rect::new(3, 4, 20, 21), ..Default::default() };
        radio.on_create();
        assert_eq!(radio.size_memo, (20, 21));
    }

    #[test]
    fn radio_button_10001bb0_tracks_hover_and_press_states_and_the_inside_flag() {
        let mut radio = RadioButton { enabled: true, rect: Rect::new(10, 10, 20, 20), ..Default::default() };
        let events = radio.on_mouse_move(15, 15, true);
        assert_eq!((radio.state, events.enter, radio.inside), (2, true, true));
        radio.pressed = true;
        radio.on_mouse_move(15, 15, true);
        assert_eq!(radio.state, 1);
        let events = radio.on_mouse_move(100, 15, false);
        assert_eq!((radio.state, events.leave, radio.inside), (1, true, false));
        radio.pressed = false;
        radio.on_mouse_move(100, 15, false);
        assert_eq!(radio.state, 0);
        radio.enabled = false;
        radio.state = 7;
        assert_eq!(radio.on_mouse_move(15, 15, true), MoveEvents::default());
        assert_eq!(radio.state, 7);
    }

    #[test]
    fn radio_button_10002510_draws_the_tooltip_only_while_the_cursor_is_inside() {
        let font = font();
        let mut radio = RadioButton {
            rect: Rect::new(10, 10, 20, 20),
            tooltip: Some("Tip".into()),
            tooltip_texture: Some(("tip".into(), (64, 16))),
            ..Default::default()
        };
        let mut list = DrawList::default();
        radio.post_paint(&mut list, (0, 0), &font);
        assert!(list.quads.is_empty());
        radio.inside = true;
        radio.post_paint(&mut list, (5, 5), &font);
        let back = list.quads.iter().find(|q| q.texture == "tip").unwrap();
        assert_eq!((back.x, back.y, back.w, back.h, back.uw, back.vh), (15.0, 15.0, font.measure("Tip") as f32, 13.0, 64.0, 16.0));
        assert!(list.quads.iter().any(|q| q.texture == "f" && q.color == TEXT.rgba()));
        radio.tooltip = Some(String::new());
        let mut list = DrawList::default();
        radio.post_paint(&mut list, (0, 0), &font);
        assert!(list.quads.is_empty());
    }
}
