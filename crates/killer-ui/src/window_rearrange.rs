use crate::positioning_rect::{PositioningGraph, get_positioning_rect};
use crate::window::WindowId;
use crate::window_restore::float_size_to_int;
use crate::window_stuck::{MissingStuckWindow, WindowStuckState, collect_stuck_windows};

pub trait RearrangeBackend: PositioningGraph {
    fn stuck_state_mut(&mut self, window: WindowId) -> Option<&mut WindowStuckState>;
    fn ratios_mut(&mut self, window: WindowId) -> Option<&mut [f32; 2]>;
    fn move_window(&mut self, window: WindowId, delta: (i32, i32));
}

pub fn rearrange_position(
    backend: &mut impl RearrangeBackend,
    window: WindowId,
    old: (f32, f32),
    new: (f32, f32),
) -> Result<(), MissingStuckWindow> {
    if backend.stuck_state(window).ok_or(MissingStuckWindow(window))?.flags() & 1 != 0 {
        return Ok(());
    }
    let mut bounds = backend.positioning_rect(window).ok_or(MissingStuckWindow(window))?;
    let mut owners = [window; 4];
    get_positioning_rect(backend, window, &mut bounds, &mut owners)?;
    let x = backend.positioning_rect(owners[0]).ok_or(MissingStuckWindow(owners[0]))?.x;
    let y = backend.positioning_rect(owners[2]).ok_or(MissingStuckWindow(owners[2]))?.y;
    let precise_x = f64::from(x);
    let rounded_y = f64::from(y as f32);
    let mut dx = x.wrapping_neg() as f32;
    let mut dy = y.wrapping_neg() as f32;
    let ratios = backend.ratios_mut(window).ok_or(MissingStuckWindow(window))?;
    if f64::from(bounds.width) + 200.0 < f64::from(old.0) {
        ratios[0] = (precise_x / (f64::from(old.0) - f64::from(bounds.width))) as f32;
    }
    if f64::from(bounds.height) + 200.0 < f64::from(old.1) {
        ratios[1] = (rounded_y / (f64::from(old.1) - f64::from(bounds.height))) as f32;
    }
    if new.0 > bounds.width {
        dx = ((f64::from(new.0) - f64::from(bounds.width)) * f64::from(ratios[0]) - precise_x) as f32;
    }
    if new.1 > bounds.height {
        dy = ((f64::from(new.1) - f64::from(bounds.height)) * f64::from(ratios[1]) - rounded_y) as f32;
    }
    let mut group = Vec::new();
    collect_stuck_windows(backend, window, &mut group)?;
    for member in group {
        backend.stuck_state_mut(member).ok_or(MissingStuckWindow(member))?.set_flags(1);
        backend.move_window(member, (float_size_to_int(dx), float_size_to_int(dy)));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::positioning_rect::PositioningRect;
    use crate::window_stuck::StuckWindowGraph;

    struct Backend {
        states: [WindowStuckState; 2],
        rects: [PositioningRect; 2],
        ratios: [[f32; 2]; 2],
        moves: Vec<(WindowId, (i32, i32))>,
    }

    impl StuckWindowGraph for Backend {
        fn stuck_state(&self, window: WindowId) -> Option<&WindowStuckState> {
            self.states.get(window.0 as usize)
        }
    }
    impl PositioningGraph for Backend {
        fn positioning_rect(&self, window: WindowId) -> Option<PositioningRect> {
            self.rects.get(window.0 as usize).copied()
        }
    }
    impl RearrangeBackend for Backend {
        fn stuck_state_mut(&mut self, window: WindowId) -> Option<&mut WindowStuckState> {
            self.states.get_mut(window.0 as usize)
        }
        fn ratios_mut(&mut self, window: WindowId) -> Option<&mut [f32; 2]> {
            self.ratios.get_mut(window.0 as usize)
        }
        fn move_window(&mut self, window: WindowId, delta: (i32, i32)) {
            assert_eq!(self.states[window.0 as usize].flags() & 1, 1);
            self.moves.push((window, delta));
            let rect = &mut self.rects[window.0 as usize];
            rect.x = rect.x.wrapping_add(delta.0);
            rect.y = rect.y.wrapping_add(delta.1);
            self.states[0].links[0].clear();
        }
    }
    fn backend() -> Backend {
        let mut states = [WindowStuckState::new(0xa0), WindowStuckState::new(0xb0)];
        states[0].links[0].push(Some(WindowId(1)));
        Backend {
            states,
            rects: [
                PositioningRect { x: 100, y: 100, width: 100.0, height: 50.0 },
                PositioningRect { x: 200, y: 100, width: 100.0, height: 50.0 },
            ],
            ratios: [[0.5, 0.25], [0.75, 0.875]],
            moves: Vec::new(),
        }
    }

    #[test]
    fn rearrange_10036a30_moves_collected_group_once_and_marks_before_each_callback() {
        let mut backend = backend();
        rearrange_position(&mut backend, WindowId(0), (1000.0, 800.0), (1800.0, 1550.0)).unwrap();
        assert_eq!(backend.moves, [(WindowId(0), (100, 100)), (WindowId(1), (100, 100))]);
        assert_eq!(backend.ratios[0], [0.125, (100.0_f64 / 750.0) as f32]);
        assert_eq!(backend.ratios[1], [0.75, 0.875]);
        assert_eq!((backend.states[0].flags(), backend.states[1].flags()), (0xa1, 0xb1));
        rearrange_position(&mut backend, WindowId(1), (1.0, 1.0), (2.0, 2.0)).unwrap();
        assert_eq!(backend.moves.len(), 2);
    }

    #[test]
    fn rearrange_10036a30_keeps_saved_ratios_at_threshold_and_uses_them_without_clamping() {
        let mut backend = backend();
        rearrange_position(&mut backend, WindowId(0), (400.0, 250.0), (1000.0, 1050.0)).unwrap();
        assert_eq!(backend.ratios[0], [0.5, 0.25]);
        assert_eq!(backend.moves, [(WindowId(0), (300, 150)), (WindowId(1), (300, 150))]);
    }

    #[test]
    fn rearrange_10036a30_no_space_keeps_wrapped_negation_and_nan_does_not_update_ratios() {
        let mut backend = backend();
        backend.states[0].links[0].clear();
        backend.rects[0].x = i32::MIN;
        backend.rects[0].y = -73;
        rearrange_position(&mut backend, WindowId(0), (f32::NAN, f32::NAN), (100.0, 50.0)).unwrap();
        assert_eq!(backend.ratios[0], [0.5, 0.25]);
        assert_eq!(backend.moves, [(WindowId(0), (i32::MIN, 73))]);
    }

    #[test]
    fn rearrange_10036abb_rounds_y_to_f32_but_keeps_x_in_the_register() {
        let mut backend = backend();
        backend.states[0].links[0].clear();
        backend.rects[0] = PositioningRect { x: 16_777_217, y: 16_777_217, width: 0.0, height: 0.0 };
        backend.ratios[0] = [1.0, 1.0];
        rearrange_position(&mut backend, WindowId(0), (0.0, 0.0), (16_777_216.0, 16_777_216.0)).unwrap();
        assert_eq!(backend.moves, [(WindowId(0), (-1, 0))]);
    }
}
