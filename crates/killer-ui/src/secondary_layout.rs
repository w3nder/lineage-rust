use crate::positioning_rect::PositioningRect;
use crate::window_restore::x87_to_int;

pub trait SecondaryLayoutBackend {
    type Window;
    fn root_rect(&mut self) -> PositioningRect;
    fn settings_window(&mut self) -> Option<Self::Window>;
    fn message_window(&mut self) -> Option<Self::Window>;
    fn set_position(&mut self, window: Self::Window, x: i32, y: i32, root: PositioningRect);
    fn resize(&mut self, window: Self::Window, width: f32, height: f32);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MissingMessageAfterResize;

pub fn default_position(backend: &mut impl SecondaryLayoutBackend) -> Result<(), MissingMessageAfterResize> {
    let root = backend.root_rect();
    if let Some(window) = backend.settings_window() {
        let y = x87_to_int(f64::from(root.height) * 0.5 - 200.0);
        let x = x87_to_int(f64::from(root.width) * 0.5 - 128.0);
        backend.set_position(window, x, y, root);
    }
    if let Some(window) = backend.message_window() {
        backend.resize(window, root.width, 52.0);
        let y = x87_to_int(f64::from(root.height) - 52.0);
        let window = backend.message_window().ok_or(MissingMessageAfterResize)?;
        backend.set_position(window, 0, y, root);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::window::WindowId;

    struct Backend {
        rect: PositioningRect,
        message: Option<WindowId>,
        remove: bool,
        calls: Vec<String>,
    }

    impl SecondaryLayoutBackend for Backend {
        type Window = WindowId;
        fn root_rect(&mut self) -> PositioningRect {
            self.calls.push("root".into());
            self.rect
        }
        fn settings_window(&mut self) -> Option<WindowId> {
            Some(WindowId(1))
        }
        fn message_window(&mut self) -> Option<WindowId> {
            self.message
        }
        fn set_position(&mut self, window: WindowId, x: i32, y: i32, root: PositioningRect) {
            assert_eq!(root.width, 1558.0);
            assert_eq!(root.height, 933.0);
            self.calls.push(format!("position {} {x} {y}", window.0));
        }
        fn resize(&mut self, window: WindowId, width: f32, height: f32) {
            assert_eq!((window, width, height), (WindowId(2), 1558.0, 52.0));
            self.calls.push("resize".into());
            self.message = (!self.remove).then_some(WindowId(3));
            self.rect.width = 1.0;
            self.rect.height = 2.0;
        }
    }

    #[test]
    fn root_is_captured_once_but_message_is_reloaded_after_resize() {
        for remove in [false, true] {
            let mut b = Backend {
                rect: PositioningRect { x: 11, y: 12, width: 1558.0, height: 933.0 },
                message: Some(WindowId(2)),
                remove,
                calls: Vec::new(),
            };
            let result = default_position(&mut b);
            if remove {
                assert_eq!(result, Err(MissingMessageAfterResize));
                assert_eq!(b.calls, ["root", "position 1 651 266", "resize"]);
            } else {
                assert_eq!(result, Ok(()));
                assert_eq!(b.calls, ["root", "position 1 651 266", "resize", "position 3 0 881"]);
            }
        }
    }
}
