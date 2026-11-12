#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationButton {
    Tutorial,
    Quest,
    Mail,
}

pub trait NotificationLayoutBackend {
    type RootRect: Clone;
    fn root_rect(&mut self) -> Self::RootRect;
    fn chat_y(&mut self) -> i32;
    fn chat_child_visible(&mut self) -> bool;
    fn set_button_position(&mut self, button: NotificationButton, x: i32, y: i32, root: Self::RootRect);
}

pub fn reposition_notifications(backend: &mut impl NotificationLayoutBackend) {
    let root = backend.root_rect();
    let chat_y = backend.chat_y();
    let offset = if backend.chat_child_visible() { 167 } else { 37 };
    let y = chat_y.wrapping_sub(offset);
    for (button, x) in [(NotificationButton::Tutorial, 5), (NotificationButton::Quest, 42), (NotificationButton::Mail, 79)] {
        backend.set_button_position(button, x, y, root.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Backend {
        y: i32,
        visible: bool,
        calls: Vec<String>,
    }

    impl NotificationLayoutBackend for Backend {
        type RootRect = [i32; 4];
        fn root_rect(&mut self) -> Self::RootRect {
            self.calls.push("root".into());
            [11, 12, 1558, 933]
        }
        fn chat_y(&mut self) -> i32 {
            self.calls.push("chat".into());
            self.y
        }
        fn chat_child_visible(&mut self) -> bool {
            self.calls.push("visible".into());
            self.visible
        }
        fn set_button_position(&mut self, button: NotificationButton, x: i32, y: i32, root: Self::RootRect) {
            assert_eq!(root, [11, 12, 1558, 933]);
            self.calls.push(format!("{button:?} {x} {y}"));
            self.y = 9999;
        }
    }

    #[test]
    fn notification_layout_1005cfe0_reads_once_and_moves_in_original_order() {
        for (visible, expected_y) in [(false, 709), (true, 579)] {
            let mut backend = Backend { y: 746, visible, calls: Vec::new() };
            reposition_notifications(&mut backend);
            assert_eq!(backend.calls, ["root".into(), "chat".into(), "visible".into(),
                format!("Tutorial 5 {expected_y}"), format!("Quest 42 {expected_y}"), format!("Mail 79 {expected_y}")]);
        }
    }

    #[test]
    fn notification_layout_1005cfe0_keeps_negative_positions_and_wraps() {
        for (y, expected) in [(0, -37), (-10, -47), (i32::MIN, i32::MAX - 36)] {
            let mut backend = Backend { y, visible: false, calls: Vec::new() };
            reposition_notifications(&mut backend);
            assert_eq!(backend.calls[3], format!("Tutorial 5 {expected}"));
        }
    }
}
