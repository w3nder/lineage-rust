use crate::window::{Rect, WindowId};

pub trait WindowRestoreBackend {
    fn play_sound(&mut self, id: u32);
    fn owner_is_inventory(&mut self) -> bool;
    fn visible(&mut self, window: WindowId) -> bool;
    fn shop_mode(&mut self, window: WindowId) -> i32;
    fn minimized_position(&mut self) -> (i32, i32);
    fn window_size(&mut self) -> (f32, f32);
    fn set_window_size(&mut self, minimized: bool, rect: Rect, position_modes: (i32, i32, i32));
    fn show_window(&mut self);
    fn focus_window(&mut self);
    fn hide_minimized(&mut self);
}

pub fn float_size_to_int(value: f32) -> i32 {
    x87_to_int(f64::from(value))
}

pub fn x87_to_int(value: f64) -> i32 {
    if !(-9223372036854775808.0..9223372036854775808.0).contains(&value) {
        0
    } else {
        value as i64 as i32
    }
}

pub fn restore_from_minimized(backend: &mut impl WindowRestoreBackend) {
    backend.play_sound(5);
    if backend.owner_is_inventory() {
        if backend.visible(WindowId::SHOP) && backend.shop_mode(WindowId::SHOP) == 1 {
            return;
        }
        if backend.visible(WindowId::WAREHOUSE) && backend.shop_mode(WindowId::WAREHOUSE) == 0 {
            return;
        }
        if backend.visible(WindowId::TRADE) || backend.visible(WindowId(19)) {
            return;
        }
    }
    let (x, y) = backend.minimized_position();
    let (width, height) = backend.window_size();
    let height = float_size_to_int(height);
    let width = float_size_to_int(width);
    backend.set_window_size(false, Rect::new(x, y, width, height), (0, 0, 3));
    backend.set_window_size(true, Rect::default(), (0, 0, 3));
    backend.show_window();
    backend.focus_window();
    backend.hide_minimized();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Backend {
        inventory: bool,
        visible: Vec<WindowId>,
        mode: i32,
        calls: Vec<String>,
    }

    impl WindowRestoreBackend for Backend {
        fn play_sound(&mut self, id: u32) { self.calls.push(format!("sound {id}")); }
        fn owner_is_inventory(&mut self) -> bool { self.inventory }
        fn visible(&mut self, window: WindowId) -> bool {
            self.calls.push(format!("visible {}", window.0));
            self.visible.contains(&window)
        }
        fn shop_mode(&mut self, window: WindowId) -> i32 {
            self.calls.push(format!("mode {}", window.0));
            self.mode
        }
        fn minimized_position(&mut self) -> (i32, i32) { (-5, 173) }
        fn window_size(&mut self) -> (f32, f32) { (256.9, 401.9) }
        fn set_window_size(&mut self, minimized: bool, rect: Rect, position_modes: (i32, i32, i32)) {
            assert_eq!(position_modes, (0, 0, 3));
            self.calls.push(format!("size {minimized} {} {} {} {}", rect.x, rect.y, rect.w, rect.h));
        }
        fn show_window(&mut self) { self.calls.push("show".into()); }
        fn focus_window(&mut self) { self.calls.push("focus".into()); }
        fn hide_minimized(&mut self) { self.calls.push("hide minimized".into()); }
    }

    #[test]
    fn restore_100356c0_non_inventory_ignores_guards_and_preserves_effect_order() {
        let mut backend = Backend { visible: vec![WindowId::SHOP, WindowId::TRADE], mode: 1, ..Backend::default() };
        restore_from_minimized(&mut backend);
        assert_eq!(backend.calls, ["sound 5", "size false -5 173 256 401", "size true 0 0 0 0", "show", "focus", "hide minimized"]);
    }

    #[test]
    fn restore_100356c0_inventory_guards_short_circuit_after_the_sound() {
        for (visible, mode, expected) in [
            (WindowId::SHOP, 1, vec!["sound 5", "visible 21", "mode 21"]),
            (WindowId::WAREHOUSE, 0, vec!["sound 5", "visible 21", "visible 18", "mode 18"]),
            (WindowId::TRADE, 7, vec!["sound 5", "visible 21", "visible 18", "visible 24"]),
            (WindowId(19), 7, vec!["sound 5", "visible 21", "visible 18", "visible 24", "visible 19"]),
        ] {
            let mut backend = Backend { inventory: true, visible: vec![visible], mode, ..Backend::default() };
            restore_from_minimized(&mut backend);
            assert_eq!(backend.calls, expected);
        }
        let mut backend = Backend { inventory: true, visible: vec![WindowId::SHOP], mode: 2, ..Backend::default() };
        restore_from_minimized(&mut backend);
        assert_eq!(backend.calls.last().unwrap(), "hide minimized");
    }

    #[test]
    fn ftol_10171a14_dimensions_truncate_to_i64_and_keep_low_bits() {
        for (value, expected) in [(1.9, 1), (-1.9, -1), (-0.0, 0), (2147483648.0, i32::MIN),
            (4294967296.0, 0), (4294967808.0, 512), (f32::INFINITY, 0), (f32::NEG_INFINITY, 0), (f32::NAN, 0)] {
            assert_eq!(float_size_to_int(value), expected, "{value:?}");
        }
    }
}
