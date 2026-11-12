use crate::color::{Argb, TEXT};
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::window::{Rect, WindowId};

pub const SIZE: i32 = 32;
pub const PARTY_MATCH_TEXTURE: &str = "L2UI_CH3.PartyMatchWnd.PartyMatchIcon";
pub const TOOLTIP_BACKGROUND: &str = crate::party_match::ICON.texture_str(0x218);
pub const BLINK_TIMER_ID: u32 = 0xfff001;
pub const BLINK_TIMER_MS: u32 = 500;
pub const BLINK_COLOR: Argb = Argb(0xff388080);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Restore(WindowId),
}

#[derive(Clone, Debug, PartialEq)]
pub struct IconView {
    pub owner: WindowId,
    pub rect: Rect,
    pub texture: String,
    pub title: String,
    pub visible: bool,
    pub tooltip_above: bool,
    pub hovered: bool,
    pub dragging: bool,
    pub blinking: bool,
    pub blink_phase: bool,
    pub timers: Vec<f32>,
    cursor: (i32, i32),
    grab_offset: (i32, i32),
}

impl IconView {
    pub fn new(owner: WindowId, texture: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            owner,
            rect: Rect::new(0, 0, SIZE, SIZE),
            texture: texture.into(),
            title: title.into(),
            visible: false,
            tooltip_above: false,
            hovered: false,
            dragging: false,
            blinking: false,
            blink_phase: false,
            timers: Vec::new(),
            cursor: (0, 0),
            grab_offset: (0, 0),
        }
    }

    pub fn show_at(&mut self, x: i32, y: i32, tooltip_above: bool) {
        self.rect.x = x;
        self.rect.y = y;
        self.tooltip_above = tooltip_above;
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.tooltip_above = false;
    }

    pub fn hit(&self, x: i32, y: i32) -> bool {
        self.visible && contains(self.rect, x, y)
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) {
        self.cursor = packed_cursor(x, y);
        self.grab_offset = (self.cursor.0.wrapping_sub(self.rect.x), self.cursor.1.wrapping_sub(self.rect.y));
    }

    pub fn mouse_move(&mut self, x: i32, y: i32, left_button: bool) {
        let (x, y) = packed_cursor(x, y);
        if left_button {
            self.move_by(x.wrapping_sub(self.cursor.0), y.wrapping_sub(self.cursor.1));
            self.cursor = (x, y);
            self.dragging = true;
        } else {
            self.hovered = contains(self.rect, x, y);
        }
    }

    pub fn update_drag(&mut self, x: i32, y: i32) {
        if self.dragging {
            self.move_by(
                x.wrapping_sub(self.grab_offset.0).wrapping_sub(self.rect.x),
                y.wrapping_sub(self.grab_offset.1).wrapping_sub(self.rect.y),
            );
        }
    }

    fn move_by(&mut self, x: i32, y: i32) {
        let (x, y) = packed_cursor(x, y);
        self.rect.x = self.rect.x.wrapping_add(x);
        self.rect.y = self.rect.y.wrapping_add(y);
    }

    pub fn mouse_up(&mut self) -> Option<Action> {
        if self.dragging {
            self.dragging = false;
            None
        } else {
            self.blinking = false;
            if !self.timers.is_empty() {
                self.timers.remove(0);
            }
            Some(Action::Restore(self.owner))
        }
    }

    pub fn start_blink(&mut self) -> bool {
        if !self.visible {
            return false;
        }
        self.blinking = true;
        self.timers.push(0.0);
        true
    }

    pub fn tick(&mut self, delta_seconds: f32) {
        let period = BLINK_TIMER_MS as f32 * 0.001;
        let mut fired = 0;
        for accumulated in &mut self.timers {
            *accumulated += delta_seconds;
            if period < *accumulated {
                *accumulated -= period;
                fired += 1;
            }
        }
        for _ in 0..fired {
            self.on_timer(BLINK_TIMER_ID);
        }
    }

    pub fn on_timer(&mut self, id: u32) {
        if id == BLINK_TIMER_ID {
            self.blink_phase = !self.blink_phase;
        }
    }

    pub fn tooltip_rect(&self, text_size: (i32, i32), root_width: i32) -> Option<Rect> {
        if !self.visible || !self.hovered || self.title.is_empty() {
            return None;
        }
        let (w, h) = text_size;
        let x = if self.rect.x.wrapping_add(w) > root_width { root_width.wrapping_sub(w) } else { self.rect.x };
        let y = if self.tooltip_above { self.rect.y.wrapping_sub(h) } else { self.rect.y.wrapping_add(self.rect.h) };
        Some(Rect::new(x, y, w, h))
    }

    pub fn paint(&self, list: &mut DrawList) {
        if !self.visible || self.texture.is_empty() {
            return;
        }
        let start = list.quads.len();
        list.draw_texture(self.rect.x, self.rect.y, SIZE, SIZE, 0, 0, SIZE, SIZE, &self.texture, 1.0);
        if self.blinking && self.blink_phase {
            list.draw_brighten_texture(self.rect.x, self.rect.y, SIZE, SIZE, &self.texture, BLINK_COLOR);
        }
        list.clip_from(start, self.rect);
    }

    pub fn paint_tooltip(&self, list: &mut DrawList, font: &BitmapFont, root_width: i32) {
        let Some(rect) = self.tooltip_rect((font.measure(&self.title) as i32, font.line_height as i32), root_width)
        else {
            return;
        };
        let mut body = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
        body.draw_texture(rect.x, rect.y, rect.w, rect.h, 0, 0, 32, 32, TOOLTIP_BACKGROUND, 1.0);
        body.draw_text(rect.x, rect.y, TEXT, &self.title, font);
        list.append_clipped(body, rect);
    }
}

fn contains(rect: Rect, x: i32, y: i32) -> bool {
    x >= rect.x && x <= rect.x.wrapping_add(rect.w) && y >= rect.y && y <= rect.y.wrapping_add(rect.h)
}

fn packed_cursor(x: i32, y: i32) -> (i32, i32) {
    (i32::from(x as i16), i32::from(y as i16))
}

pub trait MinimizeBackend {
    fn has_icon(&self) -> bool;
    fn set_icon(&mut self, texture: &str, title_sysstring: u32, tooltip_above: bool);
    fn owner_x(&self) -> i32;
    fn move_icon(&mut self, dx: i32, dy: i32);
    fn hide_owner(&mut self);
    fn show_icon(&mut self);
    fn focus_icon(&mut self);
}

pub const GM_EVENT_TEXTURE: &str = "L2UI_CH3.MenuIcon.Menuicon4";
pub const GM_EVENT_TIP_SYSSTRING: u32 = 1104;

pub fn minimize_party_match(backend: &mut impl MinimizeBackend) {
    minimize_with_icon(backend, PARTY_MATCH_TEXTURE, 389, true);
}

pub fn minimize_gm_event(backend: &mut impl MinimizeBackend) {
    minimize_with_icon(backend, GM_EVENT_TEXTURE, GM_EVENT_TIP_SYSSTRING, true);
}

pub fn minimize_with_icon(backend: &mut impl MinimizeBackend, texture: &str, title_sysstring: u32, tooltip_above: bool) {
    if backend.has_icon() {
        backend.set_icon(texture, title_sysstring, tooltip_above);
    }
    let x = backend.owner_x();
    if x < 0 && backend.has_icon() {
        backend.move_icon(x.wrapping_neg(), 0);
    }
    backend.hide_owner();
    if backend.has_icon() {
        backend.show_icon();
        backend.focus_icon();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_blink_highlight_is_the_additive_brighten_pass_of_100de40() {
        let mut icon = IconView::new(WindowId(34), PARTY_MATCH_TEXTURE, "x");
        icon.show_at(10, 20, false);
        icon.blinking = true;
        let mut list = DrawList::default();
        icon.paint(&mut list);
        assert_eq!(list.quads.len(), 1);
        icon.blink_phase = true;
        let mut list = DrawList::default();
        icon.paint(&mut list);
        assert_eq!(list.quads.len(), 2);
        assert!(!list.quads[0].additive && list.quads[1].additive);
        assert_eq!(list.quads[1].color, BLINK_COLOR.rgba());
    }

    #[test]
    fn icon_texture_and_size_come_from_the_native_child_contract() {
        let icon = crate::party_match::ICON;
        assert_eq!(icon.kind, crate::child_contract::ChildKind::Icon);
        assert_eq!(TOOLTIP_BACKGROUND, "NWindow.ChatBack");
        assert_eq!((icon.rect.w, icon.rect.h), (32, 32));
        assert_eq!(icon.registration, (0x81022, 0, 0x3000_0000));
    }

    #[derive(Default)]
    struct Backend {
        icon: bool,
        x: i32,
        calls: Vec<String>,
    }

    impl MinimizeBackend for Backend {
        fn has_icon(&self) -> bool {
            self.icon
        }
        fn set_icon(&mut self, texture: &str, id: u32, above: bool) {
            assert_eq!((texture, id, above), (PARTY_MATCH_TEXTURE, 389, true));
            self.calls.push("set".into());
        }
        fn owner_x(&self) -> i32 {
            self.x
        }
        fn move_icon(&mut self, x: i32, y: i32) {
            self.calls.push(format!("move:{x}:{y}"));
        }
        fn hide_owner(&mut self) {
            self.calls.push("0x20010060".into());
        }
        fn show_icon(&mut self) {
            self.calls.push("0x20010020".into());
        }
        fn focus_icon(&mut self) {
            self.calls.push("0x20010030".into());
        }
    }

    fn icon() -> IconView {
        let mut icon = IconView::new(WindowId(34), PARTY_MATCH_TEXTURE, "Party Matching");
        icon.show_at(10, 76, true);
        icon
    }

    #[test]
    fn minimize_100b1640_preserves_sequence_and_absent_icon_branch() {
        let mut b = Backend { icon: true, x: -600, ..Default::default() };
        minimize_party_match(&mut b);
        assert_eq!(b.calls, ["set", "move:600:0", "0x20010060", "0x20010020", "0x20010030"]);
        let mut b = Backend::default();
        minimize_party_match(&mut b);
        assert_eq!(b.calls, ["0x20010060"]);
    }

    #[test]
    fn icon_click_restores_but_zero_delta_mousemove_prevents_restore() {
        let mut v = icon();
        v.mouse_down(12, 80);
        assert_eq!(v.mouse_up(), Some(Action::Restore(WindowId(34))));
        v.mouse_down(12, 80);
        v.mouse_move(12, 80, true);
        assert!(v.dragging);
        assert_eq!(v.mouse_up(), None);
        assert!(!v.dragging);
        v.mouse_down(12, 80);
        v.mouse_move(112, 180, true);
        v.update_drag(212, 280);
        assert_eq!(v.rect, Rect::new(210, 276, 32, 32));
        assert_eq!(v.mouse_up(), None);
    }

    #[test]
    fn tooltip_1001e180_uses_above_flag_and_only_horizontal_fit() {
        let mut v = icon();
        v.hovered = true;
        assert_eq!(v.tooltip_rect((80, 13), 800), Some(Rect::new(10, 63, 80, 13)));
        v.show_at(780, 2, true);
        assert_eq!(v.tooltip_rect((80, 13), 800), Some(Rect::new(720, -11, 80, 13)));
        assert_eq!(v.tooltip_rect((900, 13), 800), Some(Rect::new(-100, -11, 900, 13)));
        v.tooltip_above = false;
        assert_eq!(v.tooltip_rect((80, 13), 800), Some(Rect::new(720, 34, 80, 13)));
        v.hide();
        assert_eq!(v.tooltip_rect((80, 13), 800), None);
    }

    #[test]
    fn blink_timers_are_periodic_stack_and_restore_kills_one() {
        let mut v = icon();
        assert!(v.start_blink());
        assert!(v.start_blink());
        v.tick(0.4);
        assert!(!v.blink_phase);
        v.tick(0.2);
        assert!(!v.blink_phase);
        v.tick(0.5);
        assert!(v.timers.iter().all(|t| *t < 0.5));
        v.mouse_down(12, 80);
        assert_eq!(v.mouse_up(), Some(Action::Restore(WindowId(34))));
        assert_eq!(v.timers.len(), 1);
    }

    #[test]
    fn icon_blink_hide_and_paint_follow_native_flags() {
        let mut v = icon();
        assert!(v.start_blink());
        v.on_timer(BLINK_TIMER_ID + 1);
        assert!(!v.blink_phase);
        v.on_timer(BLINK_TIMER_ID);
        let mut list = DrawList::default();
        v.paint(&mut list);
        assert_eq!(list.quads.len(), 2);
        assert_eq!(list.quads[1].color, BLINK_COLOR.rgba());
        assert_eq!((list.quads[0].x, list.quads[0].y, list.quads[0].uw, list.quads[0].vh), (10.0, 76.0, 32.0, 32.0));
        v.hide();
        assert!(!v.tooltip_above);
        assert!(v.blinking);
        assert!(!v.start_blink());
        v.on_timer(BLINK_TIMER_ID);
        assert!(!v.blink_phase);
        let mut list = DrawList::default();
        v.paint(&mut list);
        assert!(list.quads.is_empty());
    }

    #[test]
    #[ignore]
    fn icon_and_party_match_minimize_original_handlers_unicorn() {
        let python = std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into());
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let output = std::process::Command::new(python)
            .arg(root.join("docs/evidence/minimized-extraction/oracle.py"))
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let text = String::from_utf8(output.stdout).unwrap();
        let mut count = 0;
        for line in text.lines() {
            count += 1;
            let c: Vec<_> = line.split('\t').collect();
            let n = |i: usize| c[i].parse::<i32>().unwrap();
            let mut v = icon();
            match c[0] {
                "minimize" => {
                    let mut b = Backend { icon: n(1) != 0, x: n(2), ..Default::default() };
                    minimize_party_match(&mut b);
                    assert_eq!(b.calls.join(","), c[3], "{line}");
                }
                "mouse" => {
                    v.blinking = true;
                    v.mouse_down(12, 80);
                    v.mouse_move(n(2), n(3), n(1) != 0);
                    assert_eq!((v.rect.x, v.rect.y, v.dragging), (n(4), n(5), n(6) != 0), "{line}");
                    assert_eq!(v.mouse_up().is_some(), n(7) != 0, "{line}");
                    assert_eq!(v.blinking, n(8) != 0, "{line}");
                }
                "tooltip" => {
                    v.tooltip_above = n(1) != 0;
                    v.hovered = n(2) != 0;
                    v.rect.x = n(3);
                    if n(4) == 0 {
                        v.title.clear();
                    }
                    let rect = v.tooltip_rect((n(4), 13), 800);
                    if c[5] == "none" {
                        assert_eq!(rect, None, "{line}");
                    } else {
                        assert_eq!(rect, Some(Rect::new(n(5), n(6), n(4), 13)), "{line}");
                    }
                }
                "blink" => {
                    v.visible = n(1) != 0;
                    v.blink_phase = n(2) != 0;
                    v.start_blink();
                    v.on_timer(n(3) as u32);
                    assert_eq!((v.blinking, v.blink_phase), (n(4) != 0, n(5) != 0), "{line}");
                }
                "paint" => {
                    if n(1) == 0 {
                        v.texture.clear();
                    }
                    v.blinking = n(2) != 0;
                    v.blink_phase = n(3) != 0;
                    let mut list = DrawList::default();
                    v.paint(&mut list);
                    assert_eq!(list.quads.len(), n(4) as usize, "{line}");
                }
                "hover" => {
                    v.mouse_move(n(1), n(2), false);
                    assert_eq!(v.hovered, n(3) != 0, "{line}");
                    assert_eq!(v.hit(n(1), n(2)), n(3) != 0, "{line}");
                }
                "dragpaint" => {
                    v.mouse_down(12, 80);
                    v.dragging = true;
                    v.update_drag(n(1), n(2));
                    assert_eq!((v.rect.x, v.rect.y), (n(3), n(4)), "{line}");
                }
                _ => panic!("{line}"),
            }
        }
        assert_eq!(count, 131);
    }
}
