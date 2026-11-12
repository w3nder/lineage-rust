use killer_ui::positioning_rect::PositioningRect;
use killer_ui::secondary_layout::{SecondaryLayoutBackend, default_position};
use killer_ui::window::WindowId;
use std::io::Write;
use std::process::{Command, Stdio};

struct Backend {
    rect: PositioningRect,
    captured: PositioningRect,
    settings: bool,
    message: Option<WindowId>,
    mutation: u32,
    events: Vec<i64>,
}

impl SecondaryLayoutBackend for Backend {
    type Window = WindowId;
    fn root_rect(&mut self) -> PositioningRect {
        self.rect
    }
    fn settings_window(&mut self) -> Option<WindowId> {
        self.settings.then_some(WindowId(1))
    }
    fn message_window(&mut self) -> Option<WindowId> {
        self.message
    }
    fn set_position(&mut self, window: WindowId, x: i32, y: i32, root: PositioningRect) {
        assert_eq!(
            (root.x, root.y, root.width.to_bits(), root.height.to_bits()),
            (self.captured.x, self.captured.y, self.captured.width.to_bits(), self.captured.height.to_bits())
        );
        self.events.extend([i64::from(window.0), i64::from(x), i64::from(y)]);
        if window.0 == 1 && self.mutation & 4 != 0 {
            self.message = None;
        }
    }
    fn resize(&mut self, window: WindowId, width: f32, height: f32) {
        assert_eq!(window, WindowId(3));
        self.events.extend([2, i64::from(width.to_bits()), i64::from(height.to_bits())]);
        if self.mutation & 1 != 0 {
            self.rect = PositioningRect { x: 99, y: 100, width: 1.0, height: 2.0 };
        }
        if self.mutation & 2 != 0 {
            self.message = Some(WindowId(4));
        }
    }
}

#[test]
#[ignore]
fn secondary_layout_10090f50_matches_native_pc53_bits_and_reloaded_message() {
    let mut cases = Vec::new();
    for width in [
        0,
        1,
        0x80000000,
        0x3f800000,
        0x7f800000,
        0xff800000,
        0x7fc00001,
        0x7f800001,
        0x5f000000,
        0xdf000000,
        0x4b800001,
        0xcb800001,
        1558.0_f32.to_bits(),
        800.5_f32.to_bits(),
    ] {
        for mask in 0..4 {
            for mutation in 0..8 {
                cases.push((width, width ^ 0x80000000, mask, mutation));
            }
        }
    }
    let mut seed = 0x0184abcd_u32;
    for index in 0..512 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let width = seed;
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        cases.push((width, seed, index % 4, index % 8));
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/secondary_layout_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .arg("--bits")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    for (width, height, mask, mutation) in &cases {
        writeln!(input, "{width} {height} {mask} {mutation}").unwrap();
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    for ((width, height, mask, mutation), line) in cases.into_iter().zip(output.lines()) {
        let rect = PositioningRect { x: 11, y: 12, width: f32::from_bits(width), height: f32::from_bits(height) };
        let mut b = Backend {
            rect,
            captured: rect,
            settings: mask & 1 != 0,
            message: (mask & 2 != 0).then_some(WindowId(3)),
            mutation,
            events: Vec::new(),
        };
        default_position(&mut b).unwrap();
        let expected: Vec<i64> = line.split_whitespace().map(|v| v.parse().unwrap()).collect();
        assert_eq!(b.events, expected, "{width} {height} {mask} {mutation}");
    }
}
