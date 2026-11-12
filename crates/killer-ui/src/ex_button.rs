pub const DISABLE_TEXTURE: &str = "NWindow.Disable";
pub const NOTIFY_MESSAGE: i32 = 0x501;
pub const SOUND_ENABLED: u16 = 0xac;
pub const SOUND_CLICK: usize = 0;
pub const SOUND_DENIED: usize = 2;
pub const KEY_ENTER: u32 = 0xd;
pub const KEY_SPACE: u32 = 0x20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Notify {
    pub message: i32,
    pub id: i32,
    pub param: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExButtonEvents {
    pub sound: Option<usize>,
    pub notify: Option<Notify>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TooltipKind {
    Compact,
    Full,
}

pub fn post_paint_tooltip(inside: bool, has_item: bool, full: bool) -> Option<TooltipKind> {
    if !inside || !has_item {
        return None;
    }
    Some(if full { TooltipKind::Full } else { TooltipKind::Compact })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExButton {
    pub id: i32,
    pub param: i32,
    pub state: i32,
    pub key_state: i32,
    pub enabled: bool,
    pub sound: bool,
    pub disable_texture: Option<String>,
    pub mouse_over: bool,
}

impl ExButton {
    pub fn new(id: i32) -> ExButton {
        ExButton { id, param: -1, state: 0, key_state: 0, enabled: true, sound: true, disable_texture: None, mouse_over: false }
    }

    pub fn on_create(&mut self) {
        if self.disable_texture.is_none() {
            self.disable_texture = Some(DISABLE_TEXTURE.to_string());
        }
    }

    fn click_sound(&self) -> usize {
        if self.enabled { SOUND_CLICK } else { SOUND_DENIED }
    }

    fn notify(&self, param: i32) -> Notify {
        Notify { message: NOTIFY_MESSAGE, id: self.id, param }
    }

    pub fn on_mouse_move(&mut self, top_window_at_pointer: bool, left_down: bool) {
        if top_window_at_pointer {
            self.mouse_over = true;
            self.state = if left_down { 1 } else { 2 };
        } else {
            self.state = 0;
            self.mouse_over = false;
        }
    }

    pub fn on_lbutton_down(&mut self, hit: bool) -> ExButtonEvents {
        let mut events = ExButtonEvents::default();
        if hit {
            self.state = 1;
            if self.sound {
                events.sound = Some(self.click_sound());
            }
        }
        events
    }

    pub fn on_lbutton_up(&mut self, hit: bool, hover: bool) -> ExButtonEvents {
        let mut events = ExButtonEvents::default();
        if hit && self.enabled {
            let param = if self.param == -1 { self.id } else { self.param };
            events.notify = Some(self.notify(param));
        }
        self.state = if hover { 2 } else { 0 };
        events
    }

    pub fn on_key_down(&mut self, key: u32) -> ExButtonEvents {
        let mut events = ExButtonEvents::default();
        if (key == KEY_ENTER || key == KEY_SPACE) && self.key_state == 1 {
            if self.sound {
                events.sound = Some(self.click_sound());
            }
            events.notify = Some(self.notify(0));
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ex_button_10006e10_loads_the_disable_texture_only_once() {
        let mut button = ExButton::new(7);
        button.on_create();
        assert_eq!(button.disable_texture.as_deref(), Some("NWindow.Disable"));
        button.disable_texture = Some("custom".into());
        button.on_create();
        assert_eq!(button.disable_texture.as_deref(), Some("custom"));
    }

    #[test]
    fn ex_button_100012e0_shows_over_or_pressed_under_the_pointer_and_normal_elsewhere() {
        let mut button = ExButton::new(7);
        button.on_mouse_move(true, false);
        assert_eq!((button.state, button.mouse_over), (2, true));
        button.on_mouse_move(true, true);
        assert_eq!(button.state, 1);
        button.on_mouse_move(false, true);
        assert_eq!((button.state, button.mouse_over), (0, false));
    }

    #[test]
    fn ex_button_10001200_presses_only_on_a_hit_and_plays_the_sound_for_its_enabled_state() {
        let mut button = ExButton::new(7);
        assert_eq!(button.on_lbutton_down(false), ExButtonEvents::default());
        assert_eq!(button.state, 0);
        assert_eq!(button.on_lbutton_down(true).sound, Some(0));
        assert_eq!(button.state, 1);
        button.enabled = false;
        assert_eq!(button.on_lbutton_down(true).sound, Some(2));
        button.sound = false;
        assert_eq!(button.on_lbutton_down(true).sound, None);
        assert_eq!(button.state, 1);
    }

    #[test]
    fn ex_button_10001270_notifies_the_parent_with_the_param_or_the_id_and_settles_on_hover() {
        let mut button = ExButton::new(7);
        button.state = 1;
        let events = button.on_lbutton_up(true, true);
        assert_eq!(events.notify, Some(Notify { message: 0x501, id: 7, param: 7 }));
        assert_eq!(button.state, 2);
        button.param = 33;
        assert_eq!(button.on_lbutton_up(true, false).notify.map(|n| n.param), Some(33));
        assert_eq!(button.state, 0);
        assert_eq!(button.on_lbutton_up(false, true).notify, None);
        button.enabled = false;
        assert_eq!(button.on_lbutton_up(true, true).notify, None);
        assert_eq!(button.state, 2);
    }

    #[test]
    fn ex_button_10001440_activates_on_enter_or_space_only_while_pressed() {
        let mut button = ExButton::new(9);
        button.state = 1;
        assert_eq!(button.on_key_down(KEY_ENTER), ExButtonEvents::default());
        button.key_state = 1;
        let events = button.on_key_down(KEY_SPACE);
        assert_eq!(events.sound, Some(0));
        assert_eq!(events.notify, Some(Notify { message: 0x501, id: 9, param: 0 }));
        assert_eq!(button.on_key_down(0x41), ExButtonEvents::default());
        button.enabled = false;
        assert_eq!(button.on_key_down(KEY_ENTER).sound, Some(2));
    }

    #[test]
    fn ex_button_10003640_picks_the_compact_tooltip_unless_the_full_flag_is_set() {
        assert_eq!(post_paint_tooltip(true, true, false), Some(TooltipKind::Compact));
        assert_eq!(post_paint_tooltip(true, true, true), Some(TooltipKind::Full));
        assert_eq!(post_paint_tooltip(false, true, true), None);
        assert_eq!(post_paint_tooltip(true, false, false), None);
    }
}
