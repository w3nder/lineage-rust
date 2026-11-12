use killer_ui::window::WindowId;
use killer_ui::window_destroy::{OwnedWindow, WindowDestroyBackend, destroy_window};
use std::io::Write;
use std::process::{Command, Stdio};

struct Backend {
    windows: [Option<WindowId>; 2],
    auxiliary: Option<u32>,
    modes: [u32; 3],
    events: Vec<u32>,
}

impl WindowDestroyBackend for Backend {
    type Auxiliary = u32;
    fn owned_window(&mut self, kind: OwnedWindow) -> Option<WindowId> {
        self.windows[kind as usize]
    }
    fn unregister_owned(&mut self, window: WindowId) {
        self.events.extend([1, window.0]);
        let index = if window.0 == 1 { 0 } else { 1 };
        if self.modes[index] != 0 {
            self.windows[index] = (self.modes[index] == 2).then_some(WindowId(window.0 + 10));
        }
    }
    fn delete_window(&mut self, window: WindowId, mode: u32) {
        self.events.extend([2, window.0, mode]);
    }
    fn clear_owned(&mut self, kind: OwnedWindow) {
        self.windows[kind as usize] = None;
    }
    fn auxiliary(&mut self) -> Option<u32> {
        self.auxiliary
    }
    fn destroy_auxiliary(&mut self, auxiliary: u32) {
        assert_eq!(auxiliary, 3);
        self.events.push(3);
        if self.modes[2] != 0 {
            self.auxiliary = Some(99);
        }
    }
    fn free_auxiliary(&mut self, auxiliary: u32) {
        assert_eq!(auxiliary, 3);
        self.events.push(4);
    }
    fn clear_auxiliary(&mut self) {
        self.auxiliary = None;
    }
}

#[test]
#[ignore]
fn on_destroy_10036140_matches_original_owned_reloads_and_auxiliary_capture() {
    let mut cases = Vec::new();
    for mask in 0..8 {
        for frame in 0..3 {
            for minimized in 0..3 {
                for aux in 0..2 {
                    cases.push((mask, [frame, minimized, aux]));
                }
            }
        }
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/window_destroy_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    for (mask, modes) in &cases {
        writeln!(input, "{mask} {} {} {}", modes[0], modes[1], modes[2]).unwrap();
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    for ((mask, modes), line) in cases.into_iter().zip(output.lines()) {
        let mut b = Backend {
            windows: [(mask & 1 != 0).then_some(WindowId(1)), (mask & 4 != 0).then_some(WindowId(2))],
            auxiliary: (mask & 2 != 0).then_some(3),
            modes,
            events: Vec::new(),
        };
        let result = destroy_window(&mut b);
        let mut actual = vec![
            u32::from(result),
            b.windows[0].map_or(0, |v| v.0),
            b.auxiliary.unwrap_or(0),
            b.windows[1].map_or(0, |v| v.0),
        ];
        actual.extend(b.events);
        let expected: Vec<u32> = line.split_whitespace().map(|v| v.parse().unwrap()).collect();
        assert_eq!(actual, expected, "{mask} {modes:?}");
    }
}
