use killer_ui::positioning_rect::PositioningRect;
use killer_ui::window_move::{WindowMoveBackend, move_window};
use killer_ui::window_position::{RelativePositionBackend, set_relative_position};
use std::io::Write;
use std::process::{Command, Stdio};

struct Backend {
    values: [u32; 15],
    rect: PositioningRect,
    count: usize,
    events: Vec<i64>,
}

impl WindowMoveBackend for Backend {
    type ChildNode = usize;
    fn anchor(&mut self) -> u32 {
        self.values[0]
    }
    fn parent_rect(&mut self) -> Option<PositioningRect> {
        (self.values[1] == 3).then(|| rect(&self.values[7..11]))
    }
    fn rect(&mut self) -> PositioningRect {
        self.rect
    }
    fn anchor_offset(&mut self) -> (i32, i32) {
        (self.values[11] as i32, self.values[12] as i32)
    }
    fn set_position(&mut self, x: i32, y: i32) {
        self.rect.x = x;
        self.rect.y = y;
    }
    fn first_child(&mut self) -> Option<usize> {
        (self.count > 0).then_some(0)
    }
    fn move_child(&mut self, _: usize, dx: i32, dy: i32) {
        self.events.extend([self.rect.x, self.rect.y, dx, dy].map(i64::from));
        self.count = 3;
    }
    fn next_child(&mut self, node: usize) -> Option<usize> {
        (node + 1 < self.count).then_some(node + 1)
    }
}

fn rect(v: &[u32]) -> PositioningRect {
    PositioningRect { x: v[0] as i32, y: v[1] as i32, width: f32::from_bits(v[2]), height: f32::from_bits(v[3]) }
}

impl RelativePositionBackend for Backend {
    fn clear_relations(&mut self) {}
    fn position(&mut self) -> (i32, i32) {
        (self.rect.x, self.rect.y)
    }
    fn dispatch_move(&mut self, dx: i32, dy: i32) {
        move_window(self, dx, dy);
    }
}

#[test]
#[ignore]
fn movement_100351c0_matches_original() {
    compare(false);
}

#[test]
#[ignore]
fn relative_position_10035e30_matches_original_composed_moves() {
    compare(true);
}

fn compare(relative: bool) {
    let mut cases = Vec::new();
    for anchor in [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, u32::MAX] {
        for chain in 0..4 {
            for children in 0..2 {
                for dimension in [
                    0,
                    1.0_f32.to_bits(),
                    101.5_f32.to_bits(),
                    16777216.0_f32.to_bits(),
                    0x7f800000,
                    0x7fc00001,
                    0x7f800001,
                    0xff800000,
                ] {
                    cases.push([
                        anchor,
                        chain,
                        children,
                        17,
                        29,
                        dimension,
                        dimension ^ 0x80000000,
                        10,
                        20,
                        1.0_f32.to_bits(),
                        81.5_f32.to_bits(),
                        3,
                        (-4_i32) as u32,
                        400,
                        500,
                    ]);
                }
            }
        }
    }
    let mut seed = 0xead01234_u32;
    for index in 0..1024 {
        let mut values = [0; 15];
        for value in &mut values {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            *value = seed;
        }
        values[0] = index % 12;
        values[1] = 3;
        values[2] = index % 2;
        cases.push(values);
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/window_move_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .args(if relative { vec!["--relative"] } else { vec![] })
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    for values in &cases {
        writeln!(input, "{}", values.iter().map(u32::to_string).collect::<Vec<_>>().join(" ")).unwrap();
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    for (values, line) in cases.into_iter().zip(output.lines()) {
        let mut b = Backend { values, rect: rect(&values[3..7]), count: values[2] as usize, events: Vec::new() };
        if relative {
            set_relative_position(&mut b, values[13] as i32, values[14] as i32, rect(&values[7..11]));
        } else {
            move_window(&mut b, values[13] as i32, values[14] as i32);
        }
        let mut actual = vec![
            i64::from(b.rect.x),
            i64::from(b.rect.y),
            i64::from(b.rect.width.to_bits()),
            i64::from(b.rect.height.to_bits()),
        ];
        actual.extend(b.events);
        let expected: Vec<i64> = line.split_whitespace().map(|v| v.parse().unwrap()).collect();
        assert_eq!(actual, expected, "{values:?}");
    }
}
