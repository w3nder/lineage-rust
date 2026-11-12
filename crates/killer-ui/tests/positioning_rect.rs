use killer_ui::positioning_rect::{PositioningGraph, PositioningRect, get_positioning_rect};
use killer_ui::window::WindowId;
use killer_ui::window_stuck::{StuckWindowGraph, WindowStuckState};
use std::io::Write;
use std::process::{Command, Stdio};

struct Graph([(WindowStuckState, PositioningRect); 3]);

impl StuckWindowGraph for Graph {
    fn stuck_state(&self, window: WindowId) -> Option<&WindowStuckState> {
        self.0.get(window.0 as usize).map(|item| &item.0)
    }
}

impl PositioningGraph for Graph {
    fn positioning_rect(&self, window: WindowId) -> Option<PositioningRect> {
        self.0.get(window.0 as usize).map(|item| item.1)
    }
}

fn row(x: i32, y: i32, width: u32, height: u32) -> [i64; 4] {
    [i64::from(x), i64::from(y), i64::from(width), i64::from(height)]
}

fn rect(row: [i64; 4]) -> PositioningRect {
    PositioningRect {
        x: row[0] as i32,
        y: row[1] as i32,
        width: f32::from_bits(row[2] as u32),
        height: f32::from_bits(row[3] as u32),
    }
}

#[test]
#[ignore]
fn positioning_10036890_matches_original_pc53_bounds_and_owners() {
    let default = row(100, 100, 20.0_f32.to_bits(), 20.0_f32.to_bits());
    let mut cases = Vec::new();
    for x in [i32::MIN, -100, 0, 50, 100, 120, 200, i32::MAX] {
        for y in [i32::MIN, -100, 0, 50, 100, 120, 200, i32::MAX] {
            for size in [0.0_f32, 10.5, 20.0, 200.0, -10.5] {
                let other = row(x, y, size.to_bits(), size.to_bits());
                cases.push([default, default, other, other]);
            }
        }
    }
    for bits in [
        0, 1, 0x80000000, 0x7f800000, 0xff800000, 0x7fc01234, 0x7f800001, 0xffc01234, 0xff800001, 0x4f000000,
        0x5effffff, 0x5f000000, 0x7f7fffff, 0xff7fffff,
    ] {
        let other = row(-50, -70, bits, bits);
        cases.push([default, default, other, other]);
        cases.push([other, default, default, default]);
    }
    let mut seed = 123456789_u32;
    for _ in 0..1024 {
        let mut next = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            seed
        };
        let rows = std::array::from_fn(|_| row(next() as i32, next() as i32, next(), next()));
        cases.push(rows);
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/positioning_rect_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    for case in &cases {
        writeln!(input, "{}", case.iter().flatten().map(ToString::to_string).collect::<Vec<_>>().join(" ")).unwrap();
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{:?}", output.status);
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    for (case, native) in cases.into_iter().zip(output.lines()) {
        let mut graph = Graph(std::array::from_fn(|i| (WindowStuckState::new(0), rect(case[i + 1]))));
        graph.0[0].0.links[0] = vec![Some(WindowId(1)), Some(WindowId(2))];
        let mut bounds = rect(case[0]);
        let mut owners = [WindowId(99); 4];
        get_positioning_rect(&graph, WindowId(0), &mut bounds, &mut owners).unwrap();
        let mut result = row(bounds.x, bounds.y, bounds.width.to_bits(), bounds.height.to_bits()).to_vec();
        result.extend(owners.map(|id| i64::from(id.0)));
        assert_eq!(result.iter().map(ToString::to_string).collect::<Vec<_>>().join(" "), native, "{case:?}");
    }
}
