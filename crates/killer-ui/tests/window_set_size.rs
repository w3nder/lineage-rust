use killer_ui::initial_position::InitialPosition;
use killer_ui::window_set_size::{SetWindowSize, SetWindowSizeBackend, set_window_size};
use std::io::Write;
use std::process::{Command, Stdio};

struct Backend {
    v: [u32; 24],
    initial: InitialPosition,
    position: (i32, i32),
    size: (f32, f32),
    children: usize,
    events: Vec<u32>,
}

impl Backend {
    fn descriptor(&self) -> [u32; 5] {
        [u32::from(self.initial.enabled), self.initial.horizontal_mode as u32, self.initial.vertical_mode as u32, self.initial.x as u32, self.initial.y as u32]
    }
    fn rect(&self) -> [u32; 4] {
        [self.position.0 as u32, self.position.1 as u32, self.size.0.to_bits(), self.size.1.to_bits()]
    }
}

impl SetWindowSizeBackend for Backend {
    type ChildNode = usize;
    fn initial_position(&mut self) -> InitialPosition { self.initial }
    fn set_initial_position(&mut self, value: InitialPosition) { self.initial = value; }
    fn screen_width(&mut self) -> i32 { self.v[7] as i32 }
    fn screen_height(&mut self) -> i32 { self.v[8] as i32 }
    fn clear_relations(&mut self) {
        self.events.push(0);
        self.events.extend(self.descriptor());
        if self.v[20] != 0 { self.position = (123, 456); }
    }
    fn position(&mut self) -> (i32, i32) { self.position }
    fn set_position(&mut self, value: (i32, i32)) { self.position = value; }
    fn set_size(&mut self, value: (f32, f32)) { self.size = value; }
    fn first_child(&mut self) -> Option<usize> { (self.children != 0).then_some(0) }
    fn move_child(&mut self, node: usize, delta: (i32, i32)) {
        self.events.extend([1, self.position.0 as u32, self.position.1 as u32, delta.0 as u32, delta.1 as u32]);
        if self.v[20] != 0 {
            self.children = 2;
            self.position = (i32::MAX, -3);
            self.v[18] = 2;
            if node == 0 { self.v[21] ^= 0x100000; }
        }
    }
    fn next_child(&mut self, node: usize) -> Option<usize> { (node + 1 < self.children).then_some(node + 1) }
    fn parent_position(&mut self) -> Option<(i32, i32)> { (self.v[18] == 2).then_some((self.v[22] as i32, self.v[23] as i32)) }
    fn flags(&mut self) -> u32 { self.v[21] }
    fn apply_zero_rect(&mut self) { self.events.push(2); self.events.extend(self.rect()); }
}

#[test]
#[ignore]
fn set_window_size_matches_native_state_and_callback_order() {
    let mut cases = Vec::new();
    for h in [u32::MAX, 0, 1, 2, 3, 4, 5, 6] {
        for v in [u32::MAX, 0, 1, 2, 3, 4, 5, 6] {
            for mask in 0..32 {
                let mut c = [0; 24];
                c[..9].copy_from_slice(&[mask & 1, h, v, 0x80000000, 503, if mask & 2 == 0 { 0 } else { u32::MAX }, if mask & 4 == 0 { 0 } else { 125 }, 0x80000000, (-503i32) as u32]);
                c[9..18].copy_from_slice(&[17, 29, 0x7fc00123, 0x80000000, 1, 91, 92, 93, 94]);
                c[18..].copy_from_slice(&[mask % 3, (mask >> 3) & 1, (mask >> 4) & 1, if mask & 2 == 0 { 0 } else { 0x100000 }, 101, 0x7fffffff]);
                cases.push(c);
            }
        }
    }
    let mut seed = 0xeda13578u32;
    for i in 0..1024 {
        let mut c = [0; 24];
        for value in &mut c { seed = seed.wrapping_mul(1664525).wrapping_add(1013904223); *value = seed; }
        c[0] = i & 1;
        c[1] = i % 7;
        c[2] = (i / 7) % 7;
        c[13] &= 1;
        c[18] = i % 3;
        c[19] = (i >> 1) & 1;
        c[20] = (i >> 2) & 1;
        cases.push(c);
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/window_set_size_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    for c in &cases { writeln!(input, "{}", c.iter().map(u32::to_string).collect::<Vec<_>>().join(" ")).unwrap(); }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    for (c, line) in cases.iter().zip(output.lines()) {
        let mut backend = Backend {
            v: *c, initial: InitialPosition { enabled: c[13] != 0, horizontal_mode: c[14] as i32, vertical_mode: c[15] as i32, x: c[16] as i32, y: c[17] as i32 },
            position: (c[9] as i32, c[10] as i32), size: (f32::from_bits(c[11]), f32::from_bits(c[12])), children: c[19] as usize, events: Vec::new(),
        };
        set_window_size(&mut backend, SetWindowSize { position: (c[3] as i32, c[4] as i32), size: (c[5] as i32, c[6] as i32), remember: c[0] != 0, modes: (c[1] as i32, c[2] as i32) });
        let expected: Vec<_> = backend.descriptor().into_iter().chain(backend.rect()).chain(backend.events).collect();
        let actual: Vec<u32> = line.split_whitespace().map(|value| value.parse().unwrap()).collect();
        assert_eq!(actual, expected, "case={c:?}");
    }
}
