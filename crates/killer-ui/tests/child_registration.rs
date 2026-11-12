use killer_ui::window::WindowId;
use killer_ui::window_registration::{ChildRegistrationBackend, register_child};
use std::io::Write;
use std::process::{Command, Stdio};

struct Backend {
    parent: Option<u32>,
    next_parent: Option<u32>,
    events: Vec<u32>,
}

impl ChildRegistrationBackend for Backend {
    type Record = u32;
    type Window = WindowId;
    fn parent_record(&mut self) -> Option<u32> {
        self.parent
    }
    fn register_in_manager(&mut self, child: WindowId, flags: u32, parent: u32) -> Option<u32> {
        assert_eq!(child, WindowId(12));
        self.events.extend([1, parent, flags]);
        self.parent = self.next_parent;
        Some(7)
    }
    fn set_record_parent(&mut self, record: u32, parent: Option<u32>) {
        assert_eq!(record, 7);
        assert_eq!(parent, self.next_parent);
    }
    fn append_child(&mut self, record: u32) {
        assert_eq!(record, 7);
        self.events.extend([2, self.parent.unwrap_or(0)]);
    }
}

#[test]
#[ignore]
fn child_registration_10035820_matches_native_parent_reload_and_order() {
    let mut cases = Vec::new();
    let mut seed = 0x123401ff_u32;
    for initial in 0..4 {
        for next_parent in 0..4 {
            for _ in 0..64 {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                cases.push((initial, next_parent, seed));
            }
        }
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/child_registration_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    for (initial, next, flags) in &cases {
        writeln!(input, "{initial} {next} {flags}").unwrap();
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    for ((initial, next, flags), line) in cases.into_iter().zip(output.lines()) {
        let mut b = Backend {
            parent: (initial != 0).then_some(initial),
            next_parent: (next != 0).then_some(next),
            events: Vec::new(),
        };
        let record = register_child(&mut b, WindowId(12), flags).unwrap();
        let mut actual = vec![u32::from(record.is_some())];
        actual.extend(b.events);
        let expected: Vec<u32> = line.split_whitespace().map(|v| v.parse().unwrap()).collect();
        assert_eq!(actual, expected, "{initial} {next} {flags}");
    }
}
