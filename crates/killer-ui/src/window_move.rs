use crate::positioning_rect::PositioningRect;
use crate::window_restore::x87_to_int as ftol;

pub trait WindowMoveBackend {
    type ChildNode: Copy;
    fn anchor(&mut self) -> u32;
    fn parent_rect(&mut self) -> Option<PositioningRect>;
    fn rect(&mut self) -> PositioningRect;
    fn anchor_offset(&mut self) -> (i32, i32);
    fn set_position(&mut self, x: i32, y: i32);
    fn first_child(&mut self) -> Option<Self::ChildNode>;
    fn move_child(&mut self, node: Self::ChildNode, dx: i32, dy: i32);
    fn next_child(&mut self, node: Self::ChildNode) -> Option<Self::ChildNode>;
}

fn anchor_axis(mode: u32, parent: f32, own: f32) -> i32 {
    match mode {
        0 => 0,
        1 => ftol(f64::from(parent) * 0.5 - f64::from(own) * 0.5),
        _ => ftol(f64::from(parent) - f64::from(own)),
    }
}

pub fn move_window(backend: &mut impl WindowMoveBackend, dx: i32, dy: i32) {
    let anchor = backend.anchor();
    let mut delta = (dx, dy);
    if anchor != 0
        && let Some(parent) = backend.parent_rect()
    {
        let rect = backend.rect();
        let position = if (1..=9).contains(&anchor) {
            (
                anchor_axis((anchor - 1) % 3, parent.width, rect.width),
                anchor_axis((anchor - 1) / 3, parent.height, rect.height),
            )
        } else {
            (dy, dy)
        };
        let offset = backend.anchor_offset();
        delta = (
            position.0.wrapping_add(offset.0).wrapping_sub(rect.x.wrapping_sub(parent.x)),
            position.1.wrapping_add(offset.1).wrapping_sub(rect.y.wrapping_sub(parent.y)),
        );
    }
    let rect = backend.rect();
    backend.set_position(rect.x.wrapping_add(delta.0), rect.y.wrapping_add(delta.1));
    let mut node = backend.first_child();
    while let Some(child) = node {
        backend.move_child(child, delta.0, delta.1);
        node = backend.next_child(child);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Backend {
        anchor: u32,
        parent: Option<PositioningRect>,
        rect: PositioningRect,
        offset: (i32, i32),
        children: usize,
        calls: Vec<(usize, i32, i32)>,
    }

    impl WindowMoveBackend for Backend {
        type ChildNode = usize;
        fn anchor(&mut self) -> u32 {
            self.anchor
        }
        fn parent_rect(&mut self) -> Option<PositioningRect> {
            assert_ne!(self.anchor, 0);
            self.parent
        }
        fn rect(&mut self) -> PositioningRect {
            self.rect
        }
        fn anchor_offset(&mut self) -> (i32, i32) {
            self.offset
        }
        fn set_position(&mut self, x: i32, y: i32) {
            self.rect.x = x;
            self.rect.y = y;
            self.calls.push((usize::MAX, x, y));
        }
        fn first_child(&mut self) -> Option<usize> {
            (self.children > 0).then_some(0)
        }
        fn move_child(&mut self, node: usize, dx: i32, dy: i32) {
            self.calls.push((node, dx, dy));
            self.children = 3;
        }
        fn next_child(&mut self, node: usize) -> Option<usize> {
            (node + 1 < self.children).then_some(node + 1)
        }
    }

    fn backend(anchor: u32) -> Backend {
        Backend {
            anchor,
            parent: Some(PositioningRect { x: 10, y: 20, width: 101.5, height: 81.5 }),
            rect: PositioningRect { x: 17, y: 29, width: 20.25, height: 10.25 },
            offset: (3, -4),
            children: 1,
            calls: Vec::new(),
        }
    }

    #[test]
    fn all_anchors_move_owner_before_dynamic_children() {
        for anchor in 1..=9 {
            let mut b = backend(anchor);
            move_window(&mut b, 400, 500);
            let x = 13 + [0, 40, 81][((anchor - 1) % 3) as usize];
            let y = 16 + [0, 35, 71][((anchor - 1) / 3) as usize];
            assert_eq!(
                b.calls,
                vec![(usize::MAX, x, y), (0, x - 17, y - 29), (1, x - 17, y - 29), (2, x - 17, y - 29)]
            );
            assert_eq!(b.rect.width, 20.25);
            assert_eq!(b.rect.height, 10.25);
        }
    }

    #[test]
    fn missing_parent_no_anchor_and_unknown_anchor_preserve_native_branches() {
        for anchor in [0, 1, 10, u32::MAX] {
            let mut b = backend(anchor);
            b.parent = None;
            move_window(&mut b, i32::MAX, i32::MIN);
            assert_eq!((b.rect.x, b.rect.y), (17_i32.wrapping_add(i32::MAX), 29_i32.wrapping_add(i32::MIN)));
        }
        for anchor in [10, u32::MAX] {
            let mut b = backend(anchor);
            move_window(&mut b, 100, -25);
            assert_eq!((b.rect.x, b.rect.y), (-12, -9));
        }
    }

    #[test]
    fn conversions_do_not_round_to_f32_or_saturate_i32() {
        assert_eq!(anchor_axis(2, 16777216.0, -1.0), 16777217);
        assert_eq!(anchor_axis(1, 16777216.0, -3.0), 8388609);
        assert_eq!(anchor_axis(2, 4294967296.0, 0.0), 0);
        assert_eq!(anchor_axis(2, f32::NAN, 0.0), 0);
        assert_eq!(anchor_axis(2, f32::INFINITY, 0.0), 0);
    }
}
