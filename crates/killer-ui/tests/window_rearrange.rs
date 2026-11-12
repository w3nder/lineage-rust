use killer_ui::positioning_rect::{PositioningGraph, PositioningRect};
use killer_ui::window::WindowId;
use killer_ui::window_rearrange::{RearrangeBackend, rearrange_position};
use killer_ui::window_stuck::{StuckWindowGraph, WindowStuckState};
use std::io::Write;
use std::process::{Command, Stdio};

struct Backend {
    states: [WindowStuckState; 3],
    rects: [PositioningRect; 3],
    ratios: [[f32; 2]; 3],
    moves: Vec<String>,
}
impl StuckWindowGraph for Backend {
    fn stuck_state(&self, window: WindowId) -> Option<&WindowStuckState> {
        self.states.get(window.0 as usize)
    }
}
impl PositioningGraph for Backend {
    fn positioning_rect(&self, window: WindowId) -> Option<PositioningRect> {
        self.rects.get(window.0 as usize).copied()
    }
}
impl RearrangeBackend for Backend {
    fn stuck_state_mut(&mut self, window: WindowId) -> Option<&mut WindowStuckState> {
        self.states.get_mut(window.0 as usize)
    }
    fn ratios_mut(&mut self, window: WindowId) -> Option<&mut [f32; 2]> {
        self.ratios.get_mut(window.0 as usize)
    }
    fn move_window(&mut self, window: WindowId, delta: (i32, i32)) {
        self.moves.push(format!("m:{}:{}:{}:{}", window.0, delta.0, delta.1, self.states[window.0 as usize].flags()));
        self.states[0].links[0].clear();
    }
}

fn bits(value: f32) -> i64 {
    i64::from(value.to_bits())
}

#[test]
#[ignore]
fn rearrange_10036a30_matches_original_ratios_flags_and_move_order() {
    let base = [
        100,
        100,
        bits(100.0),
        bits(50.0),
        200,
        100,
        bits(100.0),
        bits(50.0),
        50,
        80,
        bits(20.0),
        bits(30.0),
        bits(0.5),
        bits(0.25),
        bits(1000.0),
        bits(800.0),
        bits(1800.0),
        bits(1550.0),
        0xa0,
        0xb1,
        0xc0,
    ];
    let mut cases = Vec::new();
    for old_width in [0.0, 450.0, 450.5, 1000.0, f32::NAN] {
        for old_height in [0.0, 270.0, 270.5, 800.0, f32::NAN] {
            for new in [0.0, 70.0, 250.0, 500.5, 1800.0, f32::NAN] {
                for flag in [0xa0, 0xa1] {
                    let mut case = base;
                    case[14..18].copy_from_slice(&[bits(old_width), bits(old_height), bits(new), bits(new)]);
                    case[18] = flag;
                    cases.push(case);
                }
            }
        }
    }
    for xy in [i32::MIN, -16_777_217, -1, 0, 16_777_217, i32::MAX] {
        for ratio in [0_u32, 1, 0x80000000, 0x3f800000, 0x7f800000, 0xff800000, 0x7fc01234, 0x7f800001] {
            let mut case = base;
            for i in 0..3 {
                case[i * 4] = i64::from(xy);
                case[i * 4 + 1] = i64::from(xy);
            }
            case[12] = i64::from(ratio);
            case[13] = i64::from(ratio);
            case[14] = bits(0.0);
            case[15] = bits(0.0);
            cases.push(case);
        }
    }
    let mut seed = 123456789_u32;
    for _ in 0..512 {
        let mut case = base;
        for (i, value) in case[..18].iter_mut().enumerate() {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            *value = if i < 12 && i % 4 < 2 { i64::from(seed as i32) } else { i64::from(seed) };
        }
        cases.push(case);
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/positioning_rect_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .arg("--rearrange")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    for case in &cases {
        writeln!(input, "{}", case.iter().map(ToString::to_string).collect::<Vec<_>>().join(" ")).unwrap();
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{:?}", output.status);
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    for (case, native) in cases.into_iter().zip(output.lines()) {
        let float = |i: usize| f32::from_bits(case[i] as u32);
        let mut backend = Backend {
            states: std::array::from_fn(|i| WindowStuckState::new(case[18 + i] as u32)),
            rects: std::array::from_fn(|i| PositioningRect {
                x: case[i * 4] as i32,
                y: case[i * 4 + 1] as i32,
                width: float(i * 4 + 2),
                height: float(i * 4 + 3),
            }),
            ratios: [[float(12), float(13)], [0.75, 0.875], [0.75, 0.875]],
            moves: Vec::new(),
        };
        backend.states[0].links[0] = vec![Some(WindowId(1)), Some(WindowId(2))];
        rearrange_position(&mut backend, WindowId(0), (float(14), float(15)), (float(16), float(17))).unwrap();
        let mut result = vec![
            format!(
                "r:{}",
                backend.ratios.iter().flatten().map(|value| value.to_bits().to_string()).collect::<Vec<_>>().join(":")
            ),
            format!("f:{}", backend.states.iter().map(|state| state.flags().to_string()).collect::<Vec<_>>().join(":")),
        ];
        result.extend(backend.moves);
        assert_eq!(result.join("|"), native, "{case:?}");
    }
}
