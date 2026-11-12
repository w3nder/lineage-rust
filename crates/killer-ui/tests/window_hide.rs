use killer_ui::window_hide::{WindowHideBackend, hide_window};
use std::io::Write;
use std::process::{Command, Stdio};

struct Backend { values: [u32; 6], events: Vec<u32> }

impl WindowHideBackend for Backend {
    type ChildNode = usize;
    type Record = u32;
    fn flags(&mut self) -> u32 { self.values[0] }
    fn first_child(&mut self) -> Option<usize> { (self.values[4] != 0).then_some(0) }
    fn set_flags(&mut self, value: u32) { self.values[0] = value; }
    fn hide_child(&mut self, node: usize) {
        self.events.push(1);
        self.events.extend(&self.values[..4]);
        if node == 0 {
            if self.values[5] & 1 != 0 { self.values[4] = 2; }
            if self.values[5] & 2 != 0 { self.values[1] ^= 1; }
            if self.values[5] & 4 != 0 { self.values[2] = 17; self.values[3] = 17; }
        }
    }
    fn next_child(&mut self, node: usize) -> Option<usize> { (node + 1 < self.values[4] as usize).then_some(node + 1) }
    fn has_focus(&mut self) -> bool { self.values[1] != 0 }
    fn focused_record(&mut self) -> Option<u32> { (self.values[3] != 0).then_some(self.values[3]) }
    fn record(&mut self) -> Option<u32> { (self.values[2] != 0).then_some(self.values[2]) }
    fn clear_focused_record(&mut self) { self.values[3] = 0; }
    fn clear_own_focus(&mut self) { self.values[1] = 0; }
    fn select_focus_after_hide(&mut self, record: Option<u32>) { self.events.extend([2, record.unwrap_or(0), self.values[1], self.values[3]]); }
}

#[test]
fn hide_propagates_and_uses_focus_after_child_callbacks() {
    let mut backend = Backend { values: [0xff, 0, 3, 4, 1, 7], events: Vec::new() };
    hide_window(&mut backend);
    assert_eq!(&backend.values[..4], &[0xfd, 0, 17, 0]);
    assert_eq!(backend.events, [1, 0xfd, 0, 3, 4, 1, 0xfd, 1, 17, 17, 2, 17, 0, 0]);
}

#[test]
#[ignore]
fn hide_matches_native_child_order_and_focus_state() {
    let mut cases = Vec::new();
    for flags in [0, 2, 0xffffffff] {
        for focus in [0, 1, 7] {
            for record in [0, 3] {
                for focused in [0, 3, 4] {
                    for children in 0..2 {
                        for mutation in 0..8 { cases.push([flags, focus, record, focused, children, mutation]); }
                    }
                }
            }
        }
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/window_hide_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    for v in &cases { writeln!(input, "{}", v.iter().map(u32::to_string).collect::<Vec<_>>().join(" ")).unwrap(); }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    for (v, line) in cases.iter().zip(output.lines()) {
        let mut backend = Backend { values: *v, events: Vec::new() };
        hide_window(&mut backend);
        let expected: Vec<_> = backend.values[..4].iter().copied().chain(backend.events).collect();
        let actual: Vec<u32> = line.split_whitespace().map(|value| value.parse().unwrap()).collect();
        assert_eq!(actual, expected, "case={v:?}");
    }
}
