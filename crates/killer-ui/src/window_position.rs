use crate::positioning_rect::PositioningRect;

pub trait AnchorBackend {
    fn store_anchor(&mut self, mode: u32, offset: (i32, i32));
    fn move_window(&mut self, dx: i32, dy: i32);
}

pub fn set_anchor(backend: &mut impl AnchorBackend, mode: u32, offset: (i32, i32)) {
    backend.store_anchor(mode, offset);
    if mode != 0 {
        backend.move_window(0, 0);
    }
}

pub trait RelativePositionBackend {
    fn clear_relations(&mut self);
    fn position(&mut self) -> (i32, i32);
    fn dispatch_move(&mut self, dx: i32, dy: i32);
}

pub fn set_relative_position(backend: &mut impl RelativePositionBackend, x: i32, y: i32, root: PositioningRect) {
    backend.clear_relations();
    let old = backend.position();
    let dx = root.x.wrapping_sub(old.0).wrapping_add(x);
    let dy = y.wrapping_sub(old.1.wrapping_sub(root.y));
    backend.dispatch_move(dx, 0);
    backend.dispatch_move(0, dy);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Backend {
        anchor: (u32, i32, i32),
        position: (i32, i32),
        calls: Vec<(i32, i32)>,
    }

    impl AnchorBackend for Backend {
        fn store_anchor(&mut self, mode: u32, offset: (i32, i32)) {
            self.anchor = (mode, offset.0, offset.1);
        }
        fn move_window(&mut self, dx: i32, dy: i32) {
            assert_ne!(self.anchor.0, 0);
            self.calls.push((dx, dy));
        }
    }

    impl RelativePositionBackend for Backend {
        fn clear_relations(&mut self) {
            self.position = (17, 29);
        }
        fn position(&mut self) -> (i32, i32) {
            self.position
        }
        fn dispatch_move(&mut self, dx: i32, dy: i32) {
            self.calls.push((dx, dy));
            self.position = (123456, -987654);
        }
    }

    #[test]
    fn anchor_stores_even_disabled_and_dispatches_all_nonzero_modes() {
        for mode in [0, 1, 9, 10, u32::MAX] {
            let mut b = Backend::default();
            set_anchor(&mut b, mode, (i32::MIN, i32::MAX));
            assert_eq!(b.anchor, (mode, i32::MIN, i32::MAX));
            assert_eq!(b.calls, if mode == 0 { vec![] } else { vec![(0, 0)] });
        }
    }

    #[test]
    fn relative_position_captures_after_clear_and_before_both_moves() {
        let mut b = Backend::default();
        set_relative_position(&mut b, 7, 9, PositioningRect { x: 100, y: 200, width: f32::NAN, height: f32::INFINITY });
        assert_eq!(b.calls, [(90, 0), (0, 180)]);
        b.calls.clear();
        set_relative_position(
            &mut b,
            i32::MAX,
            i32::MIN,
            PositioningRect { x: i32::MIN, y: i32::MAX, width: 0.0, height: 0.0 },
        );
        assert_eq!(b.calls, [(-18, 0), (0, -30)]);
    }
}
