use killer_ui::window_focus_selection::{FocusSelectionBackend, select_focus_after_hide};
use std::io::Write;
use std::process::{Command, Stdio};

struct Backend { v: [u32; 7], after_root: bool, replacement: usize, count: usize, events: Vec<u32> }

impl FocusSelectionBackend for Backend {
    type Record = usize;
    type Window = usize;
    type ChildNode = usize;
    fn root(&mut self) -> Option<usize> {
        if !self.after_root { return Some(0); }
        match self.v[2] { 1 | 2 => None, 4 => Some(8), _ => Some(0) }
    }
    fn parent_record(&mut self, record: usize) -> Option<usize> {
        match record { 9 if self.v[0] == 2 => Some(10), 10 => Some(0), _ => None }
    }
    fn record_window(&mut self, record: usize) -> usize { if record >= 9 { 5 } else { record } }
    fn is_visible(&mut self, window: usize) -> bool {
        self.events.extend([1, window as u32]);
        if window == 0 { self.after_root = true; return self.v[1] != 0; }
        if self.v[6] & 1 != 0 && window == 6 { self.replacement = 7; }
        if self.v[6] & 2 != 0 && matches!(window, 2 | 6) { self.count = 4; }
        self.v[4] & (1 << window) != 0
    }
    fn window_record(&mut self, window: usize) -> Option<usize> { (self.v[2] != 3).then_some(window) }
    fn first_child(&mut self, _: usize) -> Option<usize> { (self.count != 0).then_some(0) }
    fn next_child(&mut self, node: usize) -> Option<usize> { (node + 1 < self.count).then_some(node + 1) }
    fn child_window(&mut self, node: usize) -> usize { if node == 1 { self.replacement } else { node + 1 } }
    fn flags(&mut self, window: usize) -> u32 {
        self.events.extend([2, window as u32]);
        if self.v[6] & 1 != 0 && window == 2 { self.replacement = 6; }
        if self.v[5] & (1 << window) != 0 { 0x80000 } else { 0 }
    }
    fn set_focus(&mut self, window: usize) { self.events.extend([3, window as u32]); }
}

#[test]
#[ignore]
fn focus_selection_matches_native_traversal_and_callback_mutations() {
    let mut cases = Vec::new();
    for record in 0..3 {
        for visible in 0..2 {
            for change in 0..5 {
                for count in 0..5 {
                    for mutation in 0..4 { cases.push([record, visible, change, count, 0xff, 4, mutation]); }
                }
            }
        }
    }
    for visible in 0..16 {
        for excluded in 0..16 {
            for mutation in 0..4 { cases.push([2, 1, 0, 4, (visible << 1) | 0xc0, excluded << 1, mutation]); }
        }
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/window_focus_selection_reference.py"))
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
        let mut b = Backend { v: *v, after_root: false, replacement: 2, count: v[3] as usize, events: Vec::new() };
        select_focus_after_hide(&mut b, (v[0] != 0).then_some(9)).unwrap();
        let expected: Vec<_> = [b.events.len() as u32].into_iter().chain(b.events).collect();
        let actual: Vec<u32> = line.split_whitespace().map(|value| value.parse().unwrap()).collect();
        assert_eq!(actual, expected, "case={v:?}");
    }
}
