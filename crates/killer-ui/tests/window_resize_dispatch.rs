use killer_ui::window_resize_dispatch::{ResizeDispatchBackend, ResizeHandler, resize_with_handler};
use std::convert::Infallible;
use std::io::Write;
use std::process::{Command, Stdio};

struct Backend {
    values: [u32; 7],
    size: (f32, f32),
    message: Option<u32>,
    second: Option<u32>,
    events: Vec<u32>,
}

impl ResizeDispatchBackend for Backend {
    type Window = u32;
    type Error = Infallible;
    fn resize_base(&mut self, width: f32, height: f32) -> Result<(), Infallible> {
        self.events.extend([0, width.to_bits(), height.to_bits()]);
        self.size = (f32::from_bits(self.values[3]), f32::from_bits(self.values[4]));
        self.message = (self.values[5] & 1 != 0).then_some(1);
        self.second = (self.values[5] & 2 != 0).then_some(2);
        Ok(())
    }
    fn child(&mut self, slot: u32) -> Option<u32> {
        match slot { 0x1e0 | 0xf0 => self.message, 0xec => self.second, _ => panic!("unexpected slot") }
    }
    fn size(&mut self) -> (f32, f32) { self.size }
    fn resize_child(&mut self, child: u32, width: f32, height: f32) -> Result<(), Infallible> {
        self.events.extend([child, width.to_bits(), height.to_bits()]);
        if child == 1 {
            if self.values[6] & 1 != 0 { self.size = (1.0, 2.0); }
            if self.values[6] & 2 != 0 { self.second = Some(3); }
            if self.values[6] & 4 != 0 { self.second = None; }
        }
        Ok(())
    }
}

#[test]
#[ignore]
fn resize_overrides_match_native_float_bits_presence_and_callback_reloads() {
    let edges = [0, 0x80000000, 1, 0x80000001, 0x3f800000, 0x424fffff, 0x42500000, 0x42500001, 0x7f800000, 0xff800000, 0x7fc12345, 0x7f812345];
    let mut cases = Vec::new();
    for kind in 0..4 {
        for mask in 0..4 {
            for mutation in 0..8 {
                for index in 0..edges.len() {
                    cases.push([kind, edges[index], edges[(index + 3) % edges.len()], edges[(index + 5) % edges.len()], edges[(index + 7) % edges.len()], mask, mutation]);
                }
            }
        }
    }
    let mut seed = 0xe300b4cd_u32;
    for index in 0..1024 {
        let mut values = [0; 7];
        for value in &mut values[1..5] {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            *value = seed;
        }
        values[0] = index % 4;
        values[5] = (index / 4) % 4;
        values[6] = (index / 16) % 8;
        cases.push(values);
    }
    assert_eq!(cases.len(), 2560);
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/window_resize_dispatch_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    for values in &cases { writeln!(input, "{}", values.iter().map(u32::to_string).collect::<Vec<_>>().join(" ")).unwrap(); }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    let handlers = [ResizeHandler::Base, ResizeHandler::SystemMessage, ResizeHandler::PrimaryRoot, ResizeHandler::SecondaryRoot];
    for (values, line) in cases.into_iter().zip(output.lines()) {
        let mut backend = Backend { values, size: (0.0, 0.0), message: None, second: None, events: Vec::new() };
        resize_with_handler(&mut backend, handlers[values[0] as usize], f32::from_bits(values[1]), f32::from_bits(values[2])).unwrap();
        let mut actual = vec![backend.size.0.to_bits(), backend.size.1.to_bits(), backend.second.unwrap_or(0)];
        actual.extend(backend.events);
        let expected: Vec<u32> = line.split_whitespace().map(|word| word.parse().unwrap()).collect();
        assert_eq!(actual, expected, "{values:?}");
    }
}
