use crate::initial_position::InitialPosition;

#[derive(Clone, Copy, Debug)]
pub struct SetWindowSize {
    pub position: (i32, i32),
    pub size: (i32, i32),
    pub remember: bool,
    pub modes: (i32, i32),
}

pub trait SetWindowSizeBackend {
    type ChildNode: Copy;
    fn initial_position(&mut self) -> InitialPosition;
    fn set_initial_position(&mut self, position: InitialPosition);
    fn screen_width(&mut self) -> i32;
    fn screen_height(&mut self) -> i32;
    fn clear_relations(&mut self);
    fn position(&mut self) -> (i32, i32);
    fn set_position(&mut self, position: (i32, i32));
    fn set_size(&mut self, size: (f32, f32));
    fn first_child(&mut self) -> Option<Self::ChildNode>;
    fn move_child(&mut self, node: Self::ChildNode, delta: (i32, i32));
    fn next_child(&mut self, node: Self::ChildNode) -> Option<Self::ChildNode>;
    fn parent_position(&mut self) -> Option<(i32, i32)>;
    fn flags(&mut self) -> u32;
    fn apply_zero_rect(&mut self);
}

pub fn set_window_size(backend: &mut impl SetWindowSizeBackend, request: SetWindowSize) {
    let (x, y) = request.position;
    let (width, height) = request.size;
    if request.remember {
        let mut initial = backend.initial_position();
        initial.enabled = true;
        initial.horizontal_mode = request.modes.0;
        initial.vertical_mode = request.modes.1;
        initial.x = match request.modes.0 {
            0 => x,
            1 => backend.screen_width().wrapping_sub(x),
            2 => width,
            5 => (backend.screen_width() / 2).wrapping_sub(x),
            _ => initial.x,
        };
        initial.y = match request.modes.1 {
            3 => y,
            4 => backend.screen_height().wrapping_sub(y),
            2 => height,
            5 => (backend.screen_height() / 2).wrapping_sub(y),
            _ => initial.y,
        };
        backend.set_initial_position(initial);
    }
    backend.clear_relations();
    let old = backend.position();
    let delta = (x.wrapping_sub(old.0), y.wrapping_sub(old.1));
    backend.set_position((x, y));
    if width != 0 && height != 0 {
        backend.set_size((width as u32 as f32, height as u32 as f32));
    }
    let mut node = backend.first_child();
    while let Some(child) = node {
        backend.move_child(child, delta);
        node = backend.next_child(child);
    }
    if let Some(parent) = backend.parent_position() {
        let current = backend.position();
        backend.set_position((current.0.wrapping_add(parent.0), current.1.wrapping_add(parent.1)));
    }
    if backend.flags() & 0x100000 != 0 {
        backend.apply_zero_rect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Backend {
        initial: InitialPosition,
        position: (i32, i32),
        size: (f32, f32),
        mutate: bool,
        children: usize,
        calls: Vec<String>,
    }

    impl Backend {
        fn new() -> Self {
            Self {
                initial: InitialPosition { enabled: false, horizontal_mode: 7, vertical_mode: 8, x: 91, y: 92 },
                position: (10, 20), size: (30.0, 40.0), mutate: false, children: 0, calls: Vec::new(),
            }
        }
    }

    impl SetWindowSizeBackend for Backend {
        type ChildNode = usize;
        fn initial_position(&mut self) -> InitialPosition { self.initial }
        fn set_initial_position(&mut self, value: InitialPosition) { self.initial = value; }
        fn screen_width(&mut self) -> i32 { -503 }
        fn screen_height(&mut self) -> i32 { i32::MIN }
        fn clear_relations(&mut self) { self.calls.push("clear".into()); }
        fn position(&mut self) -> (i32, i32) { self.position }
        fn set_position(&mut self, value: (i32, i32)) { self.position = value; self.calls.push(format!("position:{value:?}")); }
        fn set_size(&mut self, value: (f32, f32)) { self.size = value; self.calls.push("size".into()); }
        fn first_child(&mut self) -> Option<usize> { (self.children != 0).then_some(0) }
        fn move_child(&mut self, node: usize, delta: (i32, i32)) {
            self.calls.push(format!("child:{node}:{delta:?}"));
            if self.mutate && node == 0 { self.children += 1; self.position = (i32::MAX, -3); }
        }
        fn next_child(&mut self, node: usize) -> Option<usize> { (node + 1 < self.children).then_some(node + 1) }
        fn parent_position(&mut self) -> Option<(i32, i32)> { self.mutate.then_some((2, 7)) }
        fn flags(&mut self) -> u32 { if self.mutate { 0x100000 } else { 0 } }
        fn apply_zero_rect(&mut self) { self.calls.push("zero_rect".into()); }
    }

    #[test]
    fn live_children_preserve_delta_then_parent_uses_current_position() {
        let mut backend = Backend::new();
        backend.mutate = true;
        backend.children = 1;
        set_window_size(&mut backend, SetWindowSize { position: (50, 70), size: (-1, 125), remember: false, modes: (0, 3) });
        assert_eq!(backend.calls, ["clear", "position:(50, 70)", "size", "child:0:(40, 50)", "child:1:(40, 50)", "position:(-2147483647, 4)", "zero_rect"]);
        assert_eq!(backend.size, (u32::MAX as f32, 125.0));
        assert_eq!(backend.initial, Backend::new().initial);
    }

    #[test]
    fn all_mode_branches_and_zero_dimension_pairs() {
        for horizontal in [-1, 0, 1, 2, 3, 4, 5, 6] {
            for vertical in [-1, 0, 1, 2, 3, 4, 5, 6] {
                for size in [(0, 0), (0, 5), (5, 0), (-1, -2)] {
                    let mut backend = Backend::new();
                    set_window_size(&mut backend, SetWindowSize { position: (7, 11), size, remember: true, modes: (horizontal, vertical) });
                    assert_eq!(backend.initial, InitialPosition {
                        enabled: true, horizontal_mode: horizontal, vertical_mode: vertical,
                        x: match horizontal { 0 => 7, 1 => -510, 2 => size.0, 5 => -258, _ => 91 },
                        y: match vertical { 3 => 11, 4 => i32::MIN.wrapping_sub(11), 2 => size.1, 5 => -1073741835, _ => 92 },
                    });
                    assert_eq!(backend.size, if size.0 == 0 || size.1 == 0 { (30.0, 40.0) } else { (size.0 as u32 as f32, size.1 as u32 as f32) });
                }
            }
        }
    }
}
