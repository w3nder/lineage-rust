use crate::window_restore::float_size_to_int;

pub const PROPAGATING_CLASSES: [u32; 10] = [
    0x102dbfe8, 0x102e1858, 0x102e08c0, 0x102f12a0, 0x102df3e8, 0x102d5d10, 0x102d5280, 0x10304a88, 0x10303af0,
    0x10304020,
];

pub trait WindowResizeBackend {
    type ChildNode: Copy;
    fn clear_relations(&mut self);
    fn has_frame_flag(&mut self) -> bool;
    fn has_frame(&mut self) -> bool;
    fn frame_height(&mut self) -> f32;
    fn resize_frame(&mut self, width: i32, height: i32);
    fn size(&mut self) -> (f32, f32);
    fn set_size(&mut self, size: (f32, f32));
    fn has_anchor(&mut self) -> bool;
    fn update_anchor(&mut self, x: i32, y: i32);
    fn inherits(&mut self, class: u32) -> bool;
    fn first_child(&mut self) -> Option<Self::ChildNode>;
    fn next_child(&mut self, node: Self::ChildNode) -> Option<Self::ChildNode>;
    fn prepare_child(&mut self, node: Self::ChildNode, mode: i32);
    fn rearrange_child(&mut self, node: Self::ChildNode, old: (f32, f32), new: (f32, f32));
}

pub fn resize_window(backend: &mut impl WindowResizeBackend, width: f32, height: f32) {
    let size = (if width < 0.0 { 0.0 } else { width }, if height < 0.0 { 0.0 } else { height });
    backend.clear_relations();
    if backend.has_frame_flag() && backend.has_frame() {
        let frame_height = float_size_to_int(backend.frame_height());
        backend.resize_frame(float_size_to_int(size.0), frame_height);
    }
    let old = backend.size();
    backend.set_size(size);
    if backend.has_anchor() {
        backend.update_anchor(0, 0);
    }
    if !PROPAGATING_CLASSES.into_iter().any(|class| backend.inherits(class)) {
        return;
    }
    let mut node = backend.first_child();
    while let Some(child) = node {
        backend.prepare_child(child, 3);
        node = backend.next_child(child);
    }
    node = backend.first_child();
    while let Some(child) = node {
        backend.rearrange_child(child, old, size);
        node = backend.next_child(child);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Backend {
        frame_flag: bool,
        frame: bool,
        anchor: bool,
        class: u32,
        size: (f32, f32),
        nodes: usize,
        grow: bool,
        calls: Vec<String>,
    }

    impl Default for Backend {
        fn default() -> Self {
            Self {
                frame_flag: false,
                frame: false,
                anchor: false,
                class: 0,
                size: (800.0, 600.0),
                nodes: 2,
                grow: false,
                calls: Vec::new(),
            }
        }
    }

    impl WindowResizeBackend for Backend {
        type ChildNode = usize;
        fn clear_relations(&mut self) {
            self.calls.push("clear".into());
        }
        fn has_frame_flag(&mut self) -> bool {
            self.frame_flag
        }
        fn has_frame(&mut self) -> bool {
            self.calls.push("frame?".into());
            self.frame
        }
        fn frame_height(&mut self) -> f32 {
            self.calls.push("frame_height".into());
            31.75
        }
        fn resize_frame(&mut self, width: i32, height: i32) {
            self.calls.push(format!("frame:{width}:{height}"));
            self.size = (900.0, 700.0);
        }
        fn size(&mut self) -> (f32, f32) {
            self.calls.push("old".into());
            self.size
        }
        fn set_size(&mut self, size: (f32, f32)) {
            self.calls.push("write".into());
            self.size = size;
        }
        fn has_anchor(&mut self) -> bool {
            self.anchor
        }
        fn update_anchor(&mut self, x: i32, y: i32) {
            assert_eq!((x, y), (0, 0));
            self.calls.push("anchor".into());
        }
        fn inherits(&mut self, class: u32) -> bool {
            self.calls.push(format!("class:{class:x}"));
            class == self.class
        }
        fn first_child(&mut self) -> Option<usize> {
            self.calls.push("first".into());
            (self.nodes > 0).then_some(0)
        }
        fn next_child(&mut self, node: usize) -> Option<usize> {
            (node + 1 < self.nodes).then_some(node + 1)
        }
        fn prepare_child(&mut self, node: usize, mode: i32) {
            assert_eq!(mode, 3);
            self.calls.push(format!("prepare:{node}"));
            if self.grow {
                self.nodes += 1;
                self.grow = false;
            }
        }
        fn rearrange_child(&mut self, node: usize, old: (f32, f32), new: (f32, f32)) {
            assert_eq!(new, self.size);
            assert_eq!(old, if self.frame_flag && self.frame { (900.0, 700.0) } else { (800.0, 600.0) });
            self.calls.push(format!("rearrange:{node}"));
        }
    }

    #[test]
    fn resize_10036de0_preserves_frame_anchor_class_and_two_pass_order() {
        for frame_flag in [false, true] {
            for frame in [false, true] {
                for anchor in [false, true] {
                    for class in PROPAGATING_CLASSES.into_iter().chain([0, 0x102eb440]) {
                        let mut backend = Backend { frame_flag, frame, anchor, class, ..Backend::default() };
                        resize_window(&mut backend, 100.75, 52.0);
                        let mut expected = vec!["clear".to_owned()];
                        if frame_flag {
                            expected.push("frame?".into());
                        }
                        if frame_flag && frame {
                            expected.extend(["frame_height", "frame:100:31"].map(str::to_owned));
                        }
                        expected.extend(["old", "write"].map(str::to_owned));
                        if anchor {
                            expected.push("anchor".into());
                        }
                        for candidate in PROPAGATING_CLASSES {
                            expected.push(format!("class:{candidate:x}"));
                            if candidate == class {
                                break;
                            }
                        }
                        if PROPAGATING_CLASSES.contains(&class) {
                            expected.extend(
                                ["first", "prepare:0", "prepare:1", "first", "rearrange:0", "rearrange:1"]
                                    .map(str::to_owned),
                            );
                        }
                        assert_eq!(backend.calls, expected);
                    }
                }
            }
        }
    }

    #[test]
    fn resize_10036de0_preserves_nan_negative_zero_and_nonnegative_bits() {
        for bits in [0, 0x80000000, 0x7fc01234, 0xffc01234, 1, 0x7f800000, 0xff800000, 0xbf800000] {
            let input = f32::from_bits(bits);
            let mut backend = Backend::default();
            resize_window(&mut backend, input, input);
            let expected = if input < 0.0 { 0 } else { bits };
            assert_eq!(backend.size.0.to_bits(), expected);
            assert_eq!(backend.size.1.to_bits(), expected);
        }
    }

    #[test]
    fn resize_10036f8a_reloads_children_and_reads_next_after_effect() {
        let mut backend = Backend { class: PROPAGATING_CLASSES[0], grow: true, ..Backend::default() };
        resize_window(&mut backend, 1.0, 2.0);
        assert_eq!(
            &backend.calls[backend.calls.len() - 8..],
            ["first", "prepare:0", "prepare:1", "prepare:2", "first", "rearrange:0", "rearrange:1", "rearrange:2"]
        );
        backend.calls.clear();
        backend.nodes = 0;
        resize_window(&mut backend, 3.0, 4.0);
        assert_eq!(&backend.calls[backend.calls.len() - 2..], ["first", "first"]);
    }
}
