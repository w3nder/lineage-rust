use killer_ui::window_focus_dispatch::{FocusDispatchBackend, FocusHandler, focus_with_handler};
use std::convert::Infallible;
use std::io::Write;
use std::process::{Command, Stdio};

struct Backend {
    values: [u32; 6],
    mode: u32,
    child: u32,
    changed_vtable: bool,
    events: Vec<u32>,
}

impl FocusDispatchBackend for Backend {
    type Error = Infallible;
    fn focus_base(&mut self) -> Result<(), Infallible> {
        self.events.push(0);
        if self.values[5] & 1 != 0 { self.mode = self.values[2]; }
        if self.values[5] & 2 != 0 { self.child = self.values[4]; }
        Ok(())
    }
    fn dialog_input_mode(&mut self) -> u32 { self.mode }
    fn focus_child(&mut self, slot: u32) -> Result<(), Infallible> {
        let child = match slot { 0x240 => self.child, 0x134 => self.values[3], _ => panic!("unexpected slot") };
        assert_ne!(child, 0);
        self.events.push(child);
        if self.values[5] & 4 != 0 { self.changed_vtable = true; }
        Ok(())
    }
    fn kill_own_focus_virtual(&mut self) -> Result<(), Infallible> {
        self.events.push(if self.changed_vtable { 4 } else { 3 });
        Ok(())
    }
}

#[test]
#[ignore]
fn focus_overrides_match_original_mode_child_and_vtable_reloads() {
    let mut cases = Vec::new();
    for kind in 0..2 {
        for mode in [0, 1, u32::MAX] {
            for next_mode in [0, 1, u32::MAX] {
                for child in 1..3 {
                    for next_child in 0..3 {
                        for mutation in 0..8 {
                            let final_mode = if mutation & 1 != 0 { next_mode } else { mode };
                            let final_child = if mutation & 2 != 0 { next_child } else { child };
                            if kind == 0 && final_mode != 0 && final_child == 0 { continue; }
                            cases.push([kind, mode, next_mode, child, next_child, mutation]);
                        }
                    }
                }
            }
        }
    }
    assert_eq!(cases.len(), 816);
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/window_focus_dispatch_reference.py"))
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
        let mut backend = Backend { mode: values[1], child: values[3], values, changed_vtable: false, events: Vec::new() };
        focus_with_handler(&mut backend, if values[0] == 0 { FocusHandler::Dialog } else { FocusHandler::Chat }).unwrap();
        let mut actual = vec![backend.mode, backend.child, u32::from(backend.changed_vtable)];
        actual.extend(backend.events);
        let expected: Vec<u32> = line.split_whitespace().map(|word| word.parse().unwrap()).collect();
        assert_eq!(actual, expected, "{values:?}");
    }
}
