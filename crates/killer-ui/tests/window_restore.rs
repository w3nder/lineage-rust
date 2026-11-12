use killer_ui::window::{Rect, WindowId};
use killer_ui::window_restore::{WindowRestoreBackend, restore_from_minimized};
use std::io::Write;
use std::process::{Command, Stdio};

struct Backend {
    case: [i64; 6],
    calls: Vec<String>,
}

impl WindowRestoreBackend for Backend {
    fn play_sound(&mut self, id: u32) { self.calls.push(format!("sound:{id}")); }
    fn owner_is_inventory(&mut self) -> bool { matches!(self.case[0], 1 | 2) }
    fn visible(&mut self, window: WindowId) -> bool {
        self.calls.push(format!("visible:{}", window.0));
        let bit = [21, 18, 24, 19].iter().position(|id| *id == window.0).unwrap();
        self.case[1] & (1 << bit) != 0
    }
    fn shop_mode(&mut self, window: WindowId) -> i32 {
        self.calls.push(format!("mode:{}", window.0));
        self.case[if window == WindowId::SHOP { 2 } else { 3 }] as i32
    }
    fn minimized_position(&mut self) -> (i32, i32) { (-5, 173) }
    fn window_size(&mut self) -> (f32, f32) { (f32::from_bits(self.case[4] as u32), f32::from_bits(self.case[5] as u32)) }
    fn set_window_size(&mut self, minimized: bool, rect: Rect, modes: (i32, i32, i32)) {
        self.calls.push(format!("size:{}:{}:{}:{}:{}:{}:{}:{}", u8::from(minimized), rect.x, rect.y, rect.w, rect.h, modes.0, modes.1, modes.2));
    }
    fn show_window(&mut self) { self.calls.push("show".into()); }
    fn focus_window(&mut self) { self.calls.push("focus".into()); }
    fn hide_minimized(&mut self) { self.calls.push("hide".into()); }
}

#[test]
#[ignore]
fn restore_100356c0_guards_order_and_float_conversion_match_original() {
    let mut cases = Vec::new();
    for kind in 0..4 {
        for mask in 0..16 {
            for shop in -1..=2 {
                for store in -1..=2 {
                    cases.push([kind, mask, shop, store, i64::from(256.9_f32.to_bits()), i64::from(401.9_f32.to_bits())]);
                }
            }
        }
    }
    for bits in [0_u32, 1, 0x80000000, 0x80000001, 0x3fffffff, 0xbfffffff, 0x4f000000, 0xcf000000,
        0x4f800000, 0x4f800001, 0x5effffff, 0x5f000000, 0xdeffffff, 0xdf000000, 0xdf000001,
        0x7f7fffff, 0xff7fffff, 0x7f800000, 0xff800000, 0x7fc00000, 0x7f800001] {
        cases.push([0, 0, 0, 0, i64::from(bits), i64::from(bits)]);
    }
    let mut seed = 123456789_u32;
    for _ in 0..512 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        cases.push([3, 15, 1, 0, i64::from(seed), i64::from(seed ^ 0x80000000)]);
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/window_restore_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    for case in &cases { writeln!(input, "{}", case.iter().map(ToString::to_string).collect::<Vec<_>>().join(" ")).unwrap(); }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{:?}", output.status);
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    for (case, native) in cases.into_iter().zip(output.lines()) {
        let mut backend = Backend { case, calls: Vec::new() };
        restore_from_minimized(&mut backend);
        assert_eq!(backend.calls.join("|"), native, "{case:?}");
    }
}
