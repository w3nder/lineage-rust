use killer_ui::window_show::{WindowShowBackend, show_window};
use std::convert::Infallible;
use std::io::Write;
use std::process::{Command, Stdio};

struct Backend {
    values: [u32; 4],
    head: Option<usize>,
    next: [Option<usize>; 4],
    first: bool,
    events: Vec<u32>,
}

impl WindowShowBackend for Backend {
    type ChildNode = usize;
    type Error = Infallible;
    fn flags(&mut self) -> u32 { self.values[0] }
    fn hide_minimized(&mut self) -> Result<(), Infallible> {
        self.events.extend([10, self.values[0]]);
        if self.values[3] & 1 != 0 { self.values[0] = self.values[1]; }
        if self.values[3] & 2 != 0 { self.head = Some(2); }
        Ok(())
    }
    fn set_flags(&mut self, flags: u32) { self.values[0] = flags; }
    fn first_child(&mut self) -> Option<usize> { self.head }
    fn show_child(&mut self, node: usize) -> Result<(), Infallible> {
        self.events.extend([node as u32, self.values[0]]);
        if self.first {
            if self.values[3] & 4 != 0 { self.next[node] = Some(3); }
            if self.values[3] & 8 != 0 { self.values[0] &= !2; }
            if self.values[3] & 16 != 0 { self.head = None; }
            self.first = false;
        }
        Ok(())
    }
    fn next_child(&mut self, node: usize) -> Option<usize> { self.next[node] }
}

#[test]
#[ignore]
fn show_matches_native_minimized_hide_flag_reload_and_live_child_traversal() {
    let mut cases = Vec::new();
    let mut seed = 0x87ab90cd_u32;
    for flags in [0, 2, 0x2000, 0x2002, !0x2000, u32::MAX, 0x80000, 0x80000000] {
        for children in 0..5 {
            for mutation in 0..32 {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                cases.push([flags, seed, children, mutation]);
            }
        }
    }
    for _ in 0..1024 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let flags = seed;
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        cases.push([flags, seed, seed % 5, (seed >> 8) % 32]);
    }
    assert_eq!(cases.len(), 2304);
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/window_show_reference.py"))
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
            values, head: (values[2] != 0).then_some(0),
            next: std::array::from_fn(|index| (index + 1 < values[2] as usize).then_some(index + 1)),
            first: true, events: Vec::new(),
        };
        show_window(&mut backend).unwrap();
        let mut actual = vec![backend.values[0], backend.head.map_or(0, |head| head as u32 + 1)];
        actual.extend(backend.events);
        let expected: Vec<u32> = line.split_whitespace().map(|word| word.parse().unwrap()).collect();
        assert_eq!(actual, expected, "{values:?}");
    }
}
