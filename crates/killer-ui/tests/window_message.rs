use killer_ui::window_message::{WindowMessageBackend, command_target_message, window_message};
use std::convert::Infallible;
use std::io::Write;
use std::process::{Command, Stdio};

struct Backend { flags: u32, value: u32, events: Vec<u32> }
impl WindowMessageBackend for Backend {
    type Error = Infallible;
    fn flags(&mut self) -> u32 { self.flags }
    fn call_virtual(&mut self, slot: u32, arguments: &[u32]) -> Result<u32, Infallible> {
        self.events.extend([slot, arguments.len() as u32]); self.events.extend(arguments); Ok(self.value)
    }
}

#[test]
#[ignore]
fn command_and_window_dispatch_match_native_slots_arguments_and_return_values() {
    let mut cases = Vec::new();
    for kind in 0..2 {
        for message in (0..=0x506).chain([u32::MAX, 0xf0000001]) {
            for flags in [0, 2, u32::MAX] {
                for value in [0, 1, 0x80000000, u32::MAX] {
                    cases.push([kind, message, flags, value, message.wrapping_mul(0x7654321), !message]);
                }
            }
        }
    }
    assert_eq!(cases.len(), 30936);
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/window_message_reference.py"))
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
        let [kind, message, flags, value, wparam, lparam] = values;
        let mut backend = Backend { flags, value, events: Vec::new() };
        let result = if kind == 0 { command_target_message(&mut backend, message, wparam) }
            else { window_message(&mut backend, message, wparam, lparam) }.unwrap();
        let mut actual = vec![u32::from(result)]; actual.extend(backend.events);
        let expected: Vec<u32> = line.split_whitespace().map(|word| word.parse().unwrap()).collect();
        assert_eq!(actual, expected, "{values:?}");
    }
}
