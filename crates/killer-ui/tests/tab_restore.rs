use killer_ui::tab_control::{TabBackend, TabControl, TabEntry, TabRestoreBackend};
use std::io::Write;
use std::process::{Command, Stdio};

struct Backend {
    delta: (i32, i32),
    calls: Vec<String>,
}

impl TabBackend for Backend {
    fn sound(&mut self, _: i32) { panic!("unexpected sound"); }
    fn show_page(&mut self, _: usize, _: bool) { panic!("unexpected show"); }
    fn focus_page(&mut self, _: usize) { panic!("unexpected page focus"); }
    fn focus_button(&mut self, _: usize) { panic!("unexpected button focus"); }
    fn raise_page_in_parent(&mut self, _: usize) { panic!("unexpected raise"); }
    fn notify_parent(&mut self, index: usize) { self.calls.push(format!("notify:{index}")); }
}

impl TabRestoreBackend for Backend {
    fn restore_button_position(&mut self, index: usize) -> (i32, i32) {
        self.calls.push(format!("button:{index}:{}:{}", self.delta.0, self.delta.1));
        self.delta
    }
    fn move_restored_page(&mut self, page: usize, delta: (i32, i32)) {
        self.calls.push(format!("page:{page}:{}:{}", delta.0, delta.1));
    }
}

#[test]
#[ignore]
fn restore_detached_1002d980_matches_original_buttons_pages_and_notifications() {
    let mut cases = Vec::new();
    for count in 0..=5 {
        for mask in 0..(1 << count) {
            for positions in [[0, 0, 0, 0], [12, 25, 112, 225], [-12, -25, 112, 225],
                [i32::MIN, i32::MAX, i32::MAX, i32::MIN]] {
                cases.push((count, mask, positions));
            }
        }
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/tab_restore_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    for (count, mask, positions) in &cases {
        writeln!(input, "{count} {mask} {} {} {} {}", positions[0], positions[1], positions[2], positions[3]).unwrap();
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{:?}", output.status);
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    for ((count, mask, [sx, sy, x, y]), native) in cases.into_iter().zip(output.lines()) {
        let mut control = TabControl {
            entries: (0..count).map(|index| TabEntry { page: Some(index), enabled: index % 2 == 0,
                detached: mask & (1 << index) != 0, pressed: index == 0, org: (0, 0) }).collect(),
            current: Some(0), flags: 1,
        };
        let mut backend = Backend { delta: (sx.wrapping_sub(x), sy.wrapping_sub(y)), calls: Vec::new() };
        control.restore_detached(&mut backend).unwrap();
        let actual = if backend.calls.is_empty() { "none".into() } else { backend.calls.join("|") };
        assert_eq!(actual, native, "count={count} mask={mask} pos={sx},{sy},{x},{y}");
        assert!(control.entries.iter().all(|entry| !entry.detached));
        assert_eq!(control.current, Some(0));
    }
}
