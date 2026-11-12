use crate::window::WindowId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowStuckState {
    pub links: [Vec<Option<WindowId>>; 4],
    flags: u32,
}

impl WindowStuckState {
    pub fn new(flags: u32) -> Self {
        Self { links: std::array::from_fn(|_| Vec::new()), flags }
    }

    pub fn init_stuck_state(&mut self) {
        for links in &mut self.links {
            *links = Vec::new();
        }
    }

    pub fn flags(&self) -> u32 {
        self.flags
    }

    pub fn set_flags(&mut self, mask: u32) {
        self.flags |= mask;
    }

    pub fn clear_flags(&mut self, mask: u32) {
        self.flags &= !mask;
    }
}

pub trait StuckWindowGraph {
    fn stuck_state(&self, window: WindowId) -> Option<&WindowStuckState>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MissingStuckWindow(pub WindowId);

pub fn collect_stuck_windows(
    graph: &impl StuckWindowGraph,
    window: WindowId,
    output: &mut Vec<WindowId>,
) -> Result<(), MissingStuckWindow> {
    if output.contains(&window) {
        return Ok(());
    }
    output.push(window);
    let mut stack = vec![(window, 0, 0)];
    while let Some(frame) = stack.last_mut() {
        let (current, side, index) = *frame;
        if side == 4 {
            stack.pop();
            continue;
        }
        let state = graph.stuck_state(current).ok_or(MissingStuckWindow(current))?;
        if index >= state.links[side].len() {
            *frame = (current, side + 1, 0);
            continue;
        }
        *frame = (current, side, index + 1);
        if let Some(next) = state.links[side][index]
            && !output.contains(&next)
        {
            output.push(next);
            stack.push((next, 0, 0));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Graph(Vec<WindowStuckState>);

    impl StuckWindowGraph for Graph {
        fn stuck_state(&self, window: WindowId) -> Option<&WindowStuckState> {
            self.0.get(window.0 as usize)
        }
    }

    #[test]
    fn get_stuck_list_100364b0_preserves_depth_first_order_cycles_and_null_links() {
        let mut graph = Graph((0..7).map(WindowStuckState::new).collect());
        graph.0[0].links = [
            vec![None, Some(WindowId(1)), Some(WindowId(2))],
            vec![Some(WindowId(3))],
            vec![Some(WindowId(4))],
            vec![Some(WindowId(5))],
        ];
        graph.0[1].links[3] = vec![Some(WindowId(6)), Some(WindowId(0))];
        graph.0[6].links[0] = vec![Some(WindowId(6)), Some(WindowId(3))];
        graph.0[3].links[1] = vec![Some(WindowId(2))];
        let before: Vec<_> = graph.0.clone();
        let mut output = Vec::new();
        collect_stuck_windows(&graph, WindowId(0), &mut output).unwrap();
        assert_eq!(output, [0, 1, 6, 3, 2, 4, 5].map(WindowId));
        assert_eq!(graph.0, before);
    }

    #[test]
    fn get_stuck_list_100364b0_retains_existing_duplicates_and_does_not_expand_seen_nodes() {
        let mut graph = Graph((0..4).map(WindowStuckState::new).collect());
        graph.0[0].links[0] = vec![Some(WindowId(1)), Some(WindowId(2))];
        graph.0[1].links[0] = vec![Some(WindowId(3))];
        let mut output = vec![WindowId(1), WindowId(1)];
        collect_stuck_windows(&graph, WindowId(0), &mut output).unwrap();
        assert_eq!(output, [1, 1, 0, 2].map(WindowId));
        collect_stuck_windows(&graph, WindowId(0), &mut output).unwrap();
        assert_eq!(output, [1, 1, 0, 2].map(WindowId));
        let mut missing = vec![WindowId(99)];
        collect_stuck_windows(&graph, WindowId(99), &mut missing).unwrap();
        graph.0[2].links[0] = vec![Some(WindowId(99))];
        let mut output = Vec::new();
        assert_eq!(collect_stuck_windows(&graph, WindowId(0), &mut output), Err(MissingStuckWindow(WindowId(99))));
        assert_eq!(output, [0, 1, 3, 2, 99].map(WindowId));
    }

    #[test]
    fn get_stuck_list_100364b0_handles_a_deep_chain_without_recursive_host_calls() {
        let mut graph = Graph((0..4096).map(WindowStuckState::new).collect());
        for i in 0..4095 {
            graph.0[i].links[0].push(Some(WindowId(i as u32 + 1)));
        }
        graph.0[4095].links[3].push(Some(WindowId(0)));
        let mut output = Vec::new();
        collect_stuck_windows(&graph, WindowId(0), &mut output).unwrap();
        assert_eq!(output, (0..4096).map(WindowId).collect::<Vec<_>>());
    }

    #[test]
    fn init_stuck_state_10035ba0_releases_all_four_lists_but_preserves_flags() {
        let mut state = WindowStuckState::new(0xdeadbeef);
        for (i, links) in state.links.iter_mut().enumerate() {
            links.reserve(8);
            links.extend(std::iter::repeat_n(Some(WindowId::OPTION), i));
            links.push(None);
        }
        for _ in 0..2 {
            state.init_stuck_state();
            assert!(state.links.iter().all(|links| links.is_empty() && links.capacity() == 0));
            assert_eq!(state.flags(), 0xdeadbeef);
        }
    }

    #[test]
    fn flag_operations_10034bc0_10034bd0_preserve_unmasked_bits_and_links() {
        for flags in [0, 1, 2, 3, 0x80000000, 0xabcdef03, u32::MAX] {
            for mask in [0, 1, 2, 3, 0x80000000, 0xabcdef03, u32::MAX] {
                let mut state = WindowStuckState::new(flags);
                state.links[0].push(Some(WindowId::CHAT));
                state.clear_flags(mask);
                assert_eq!(state.flags(), flags & !mask);
                state.set_flags(mask);
                assert_eq!(state.flags(), flags | mask);
                assert_eq!(state.links[0], [Some(WindowId::CHAT)]);
            }
        }
    }
}
