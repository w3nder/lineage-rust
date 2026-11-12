use crate::window::WindowId;
use crate::window_stuck::{MissingStuckWindow, StuckWindowGraph, collect_stuck_windows};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PositioningRect {
    pub x: i32,
    pub y: i32,
    pub width: f32,
    pub height: f32,
}

pub trait PositioningGraph: StuckWindowGraph {
    fn positioning_rect(&self, window: WindowId) -> Option<PositioningRect>;
}

pub fn get_positioning_rect(
    graph: &impl PositioningGraph,
    window: WindowId,
    bounds: &mut PositioningRect,
    owners: &mut [WindowId; 4],
) -> Result<(), MissingStuckWindow> {
    let mut group = Vec::new();
    collect_stuck_windows(graph, window, &mut group)?;
    for member in group {
        let rect = graph.positioning_rect(member).ok_or(MissingStuckWindow(member))?;
        if rect.x < bounds.x {
            let delta = bounds.x.wrapping_sub(rect.x);
            bounds.x = rect.x;
            bounds.width = (f64::from(delta) + f64::from(bounds.width)) as f32;
            owners[0] = member;
        }
        if rect.y < bounds.y {
            let delta = bounds.y.wrapping_sub(rect.y);
            bounds.y = rect.y;
            bounds.height = (f64::from(delta) + f64::from(bounds.height)) as f32;
            owners[2] = member;
        }
        let right = f64::from(rect.width) + f64::from(rect.x);
        if right > f64::from(bounds.x) + f64::from(bounds.width) {
            bounds.width = (right - f64::from(bounds.x)) as f32;
            owners[1] = member;
        }
        let bottom = f64::from(rect.height) + f64::from(rect.y);
        if bottom > f64::from(bounds.y) + f64::from(bounds.height) {
            bounds.height = (bottom - f64::from(bounds.y)) as f32;
            owners[3] = member;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::window_stuck::WindowStuckState;

    struct Graph(Vec<(WindowStuckState, PositioningRect)>);

    impl StuckWindowGraph for Graph {
        fn stuck_state(&self, window: WindowId) -> Option<&WindowStuckState> {
            self.0.get(window.0 as usize).map(|value| &value.0)
        }
    }

    impl PositioningGraph for Graph {
        fn positioning_rect(&self, window: WindowId) -> Option<PositioningRect> {
            self.0.get(window.0 as usize).map(|value| value.1)
        }
    }

    #[test]
    fn positioning_10036890_tracks_four_owners_and_preserves_ties() {
        let rects = [
            PositioningRect { x: 100, y: 100, width: 20.0, height: 20.0 },
            PositioningRect { x: 50, y: 80, width: 10.0, height: 10.0 },
            PositioningRect { x: 110, y: 110, width: 40.0, height: 60.0 },
            PositioningRect { x: 50, y: 80, width: 100.0, height: 90.0 },
        ];
        let mut graph = Graph(rects.into_iter().map(|rect| (WindowStuckState::new(0), rect)).collect());
        graph.0[0].0.links[0] = vec![Some(WindowId(1)), Some(WindowId(2)), Some(WindowId(3))];
        let mut bounds = rects[0];
        let mut owners = [WindowId(0); 4];
        get_positioning_rect(&graph, WindowId(0), &mut bounds, &mut owners).unwrap();
        assert_eq!(bounds, PositioningRect { x: 50, y: 80, width: 100.0, height: 90.0 });
        assert_eq!(owners, [1, 2, 1, 2].map(WindowId));
    }

    #[test]
    fn positioning_10036890_keeps_caller_bounds_when_no_edge_expands() {
        let rect = PositioningRect { x: 10, y: 10, width: f32::NAN, height: 5.0 };
        let graph = Graph(vec![(WindowStuckState::new(0), rect)]);
        let initial = PositioningRect { x: 0, y: 0, width: 100.0, height: 100.0 };
        let mut bounds = initial;
        let mut owners = [WindowId(99); 4];
        get_positioning_rect(&graph, WindowId(0), &mut bounds, &mut owners).unwrap();
        assert_eq!(bounds, initial);
        assert_eq!(owners, [WindowId(99); 4]);
    }
}
