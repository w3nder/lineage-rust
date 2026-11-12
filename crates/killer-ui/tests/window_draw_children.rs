use killer_ui::window_draw_children::{DrawChildrenBackend, draw_children};
use std::convert::Infallible;
use std::io::Write;
use std::process::{Command, Stdio};

struct Backend {
    values: [u32; 5], special: Option<usize>, records: [usize; 4], previous: [Option<usize>; 4],
    device: usize, states: [u32; 2], events: Vec<u32>,
}

impl DrawChildrenBackend for Backend {
    type Record = usize;
    type Window = usize;
    type Node = usize;
    type Device = usize;
    type Error = Infallible;
    fn window(&mut self, record: usize) -> Result<usize, Infallible> { Ok(record) }
    fn special_root_record(&mut self) -> Option<usize> { self.special }
    fn last_child(&mut self, window: &usize) -> Option<usize> { match *window { 0 => Some(1), 2 => Some(3), _ => None } }
    fn first_child(&mut self, window: &usize) -> Option<usize> { match *window { 0 => Some(0), 2 => Some(2), _ => None } }
    fn node_record(&mut self, node: usize) -> Result<usize, Infallible> { Ok(self.records[node]) }
    fn previous(&mut self, node: usize) -> Result<Option<usize>, Infallible> { Ok(self.previous[node]) }
    fn flags(&mut self, window: usize) -> Result<u32, Infallible> {
        let flags = if self.values[1] & (1 << (window - 1)) != 0 { 0x80000 } else { 0 };
        self.events.extend([8, window as u32, flags]); Ok(flags)
    }
    fn special_state_enabled(&mut self) -> bool { self.values[2] != 0 }
    fn device(&mut self) -> usize { self.device }
    fn set_device_state(&mut self, device: usize, state: u32) -> Result<(), Infallible> {
        self.states[device] = state; self.events.extend([9, device as u32, state]); Ok(())
    }
    fn window_message(&mut self, window: usize, message: u32, device: usize, argument: u32) -> Result<(), Infallible> {
        assert_eq!((message, argument), (15, 0));
        self.events.extend([window as u32, device as u32, self.states[device]]);
        if window == 2 {
            if self.values[3] & 1 != 0 { self.records[1] = 5; }
            if self.values[3] & 2 != 0 { self.previous[1] = None; }
            if self.values[3] & 4 != 0 { self.device = 1; }
            if self.values[3] & 8 != 0 { self.special = Some(2); }
        }
        Ok(())
    }
}

#[test]
#[ignore]
fn draw_traversal_matches_native_recursion_and_live_transparency_state() {
    let mut cases = Vec::new();
    for special in 0..5 {
        for mask in 0..16 {
            for mode in [0, 1, u32::MAX] {
                for mutation in 0..16 {
                    for duplicate in 0..2 { cases.push([special, mask, mode, mutation, duplicate]); }
                }
            }
        }
    }
    assert_eq!(cases.len(), 7680);
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/window_draw_children_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    for values in &cases { writeln!(input, "{}", values.iter().map(u32::to_string).collect::<Vec<_>>().join(" ")).unwrap(); }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    for (values, line) in cases.into_iter().zip(output.lines()) {
        let mut backend = Backend {
            special: [None, None, Some(0), Some(2), Some(5)][values[0] as usize],
            records: [1, if values[4] == 0 { 2 } else { 1 }, 3, 4], previous: [None, Some(0), None, Some(2)],
            values, device: 0, states: [7, 8], events: Vec::new(),
        };
        draw_children(&mut backend, Some(0)).unwrap();
        let mut actual = backend.states.to_vec(); actual.extend(backend.events);
        let expected: Vec<u32> = line.split_whitespace().map(|word| word.parse().unwrap()).collect();
        assert_eq!(actual, expected, "{values:?}");
    }
}
