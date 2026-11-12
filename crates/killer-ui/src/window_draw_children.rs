pub trait DrawChildrenBackend {
    type Record: Copy + Eq;
    type Window: Clone;
    type Node: Copy;
    type Device;
    type Error;
    fn window(&mut self, record: Self::Record) -> Result<Self::Window, Self::Error>;
    fn special_root_record(&mut self) -> Option<Self::Record>;
    fn last_child(&mut self, window: &Self::Window) -> Option<Self::Node>;
    fn first_child(&mut self, window: &Self::Window) -> Option<Self::Node>;
    fn node_record(&mut self, node: Self::Node) -> Result<Self::Record, Self::Error>;
    fn previous(&mut self, node: Self::Node) -> Result<Option<Self::Node>, Self::Error>;
    fn flags(&mut self, window: Self::Window) -> Result<u32, Self::Error>;
    fn special_state_enabled(&mut self) -> bool;
    fn device(&mut self) -> Self::Device;
    fn set_device_state(&mut self, device: Self::Device, state: u32) -> Result<(), Self::Error>;
    fn window_message(&mut self, window: Self::Window, message: u32, device: Self::Device, argument: u32) -> Result<(), Self::Error>;
}

enum Step<R, W, N> {
    Enter(Option<R>),
    Visit { owner: W, special: bool, node: N },
    After { owner: W, special: bool, node: N, reset: bool },
}

pub fn draw_children<B: DrawChildrenBackend>(backend: &mut B, record: Option<B::Record>) -> Result<(), B::Error> {
    let mut steps = vec![Step::Enter(record)];
    while let Some(step) = steps.pop() {
        match step {
            Step::Enter(None) => {}
            Step::Enter(Some(record)) => {
                let special_record = backend.special_root_record();
                let owner = backend.window(record)?;
                let special = special_record == Some(record);
                if let Some(node) = backend.last_child(&owner) { steps.push(Step::Visit { owner, special, node }); }
            }
            Step::Visit { owner, special, node } => {
                let mut reset = false;
                if special {
                    let is_head = if let Some(head) = backend.first_child(&owner) {
                        backend.node_record(node)? == backend.node_record(head)?
                    } else { false };
                    if !is_head {
                        let record = backend.node_record(node)?;
                        let window = backend.window(record)?;
                        if backend.flags(window)? & 0x80000 == 0 && backend.special_state_enabled() {
                            let device = backend.device();
                            backend.set_device_state(device, 1)?;
                            reset = true;
                        }
                    }
                }
                let record = backend.node_record(node)?;
                let window = backend.window(record)?;
                let device = backend.device();
                backend.window_message(window, 15, device, 0)?;
                let record = backend.node_record(node)?;
                steps.push(Step::After { owner, special, node, reset });
                steps.push(Step::Enter(Some(record)));
            }
            Step::After { owner, special, node, reset } => {
                if reset {
                    let device = backend.device();
                    backend.set_device_state(device, 0)?;
                }
                if let Some(node) = backend.previous(node)? { steps.push(Step::Visit { owner, special, node }); }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::Infallible;

    struct Backend { special: Option<usize>, enabled: bool, flags: u32, records: [usize; 3], previous: [Option<usize>; 3], mutate: bool, device: u32, states: [u32; 2], events: Vec<(usize, u32, u32)> }
    impl DrawChildrenBackend for Backend {
        type Record = usize;
        type Window = usize;
        type Node = usize;
        type Device = u32;
        type Error = Infallible;
        fn window(&mut self, record: usize) -> Result<usize, Infallible> { Ok(record) }
        fn special_root_record(&mut self) -> Option<usize> { self.special }
        fn last_child(&mut self, window: &usize) -> Option<usize> { match *window { 0 => Some(1), 2 => Some(2), _ => None } }
        fn first_child(&mut self, window: &usize) -> Option<usize> { (*window == 0).then_some(0) }
        fn node_record(&mut self, node: usize) -> Result<usize, Infallible> { Ok(self.records[node]) }
        fn previous(&mut self, node: usize) -> Result<Option<usize>, Infallible> { Ok(self.previous[node]) }
        fn flags(&mut self, _: usize) -> Result<u32, Infallible> { Ok(self.flags) }
        fn special_state_enabled(&mut self) -> bool { self.enabled }
        fn device(&mut self) -> u32 { self.device }
        fn set_device_state(&mut self, device: u32, state: u32) -> Result<(), Infallible> { self.states[device as usize] = state; self.events.push((9, device, state)); Ok(()) }
        fn window_message(&mut self, window: usize, message: u32, device: u32, argument: u32) -> Result<(), Infallible> {
            assert_eq!((message, argument), (15, 0));
            self.events.push((window, device, self.states[device as usize]));
            if self.mutate && window == 2 { self.records[1] = 4; self.previous[1] = None; self.device = 1; }
            Ok(())
        }
    }
    fn backend() -> Backend { Backend { special: None, enabled: true, flags: 0, records: [1, 2, 3], previous: [None, Some(0), None], mutate: false, device: 0, states: [7, 8], events: Vec::new() } }

    #[test]
    fn paints_reverse_children_then_descendants_and_special_state_is_not_restored() {
        for special in [None, Some(0), Some(2)] {
            for flags in [0, 0x80000] {
                for enabled in [false, true] {
                    let mut b = backend(); b.special = special; b.flags = flags; b.enabled = enabled;
                    draw_children(&mut b, Some(0)).unwrap();
                    let active = special.is_some() && flags == 0 && enabled;
                    let expected = if active && special == Some(0) { vec![(9, 0, 1), (2, 0, 1), (3, 0, 1), (9, 0, 0), (1, 0, 0)] }
                        else if active { vec![(2, 0, 7), (9, 0, 1), (3, 0, 1), (9, 0, 0), (1, 0, 0)] }
                        else { vec![(2, 0, 7), (3, 0, 7), (1, 0, 7)] };
                    assert_eq!(b.events, expected);
                }
            }
        }
    }

    #[test]
    fn rereads_record_previous_and_device_after_callback_and_null_is_noop() {
        let mut b = backend(); b.special = Some(0); b.mutate = true;
        draw_children(&mut b, None).unwrap(); assert!(b.events.is_empty());
        draw_children(&mut b, Some(0)).unwrap();
        assert_eq!(b.events, [(9, 0, 1), (2, 0, 1), (9, 1, 0)]);
        assert_eq!(b.states, [1, 0]);
    }
}
