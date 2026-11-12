use killer_ui::window::Rect;
use killer_ui::zone_title::{ZoneTitleTickBackend, tick_zone_title, title_line_index};
use std::io::Write;
use std::process::{Command, Stdio};

struct Backend {
    values: [u32; 14],
    counts: [u32; 3],
    events: Vec<i64>,
}

impl ZoneTitleTickBackend for Backend {
    fn first_line_empty(&mut self) -> bool {
        self.counts[0] <= 1
    }
    fn visible(&mut self) -> bool {
        self.events.extend([0, i64::from(self.values[9])]);
        self.values[9] != 0
    }
    fn first_line_height(&mut self) -> i32 {
        self.events.extend([5, 0]);
        self.values[6] as i32
    }
    fn title_width(&mut self) -> i32 {
        let index = title_line_index(self.counts.map(|n| n.saturating_sub(1) as usize));
        self.events.extend([5, index as i64]);
        self.values[3 + index] as i32
    }
    fn screen_height(&mut self) -> i32 {
        self.values[8] as i32
    }
    fn screen_width(&mut self) -> i32 {
        self.values[7] as i32
    }
    fn set_rect(&mut self, rect: Rect) {
        self.events.extend([1, i64::from(rect.x), i64::from(rect.y), i64::from(rect.w), i64::from(rect.h), 0, 0, 3]);
    }
    fn show(&mut self) {
        self.values[9] = 1;
        self.events.push(2);
    }
    fn hide(&mut self) {
        self.values[9] = 0;
        self.events.push(3);
    }
    fn elapsed_seconds(&mut self) -> f32 {
        f32::from_bits(self.values[10])
    }
    fn set_elapsed_seconds(&mut self, seconds: f32) {
        self.values[10] = seconds.to_bits();
    }
    fn clear_lines(&mut self) {
        self.counts = [0; 3];
        self.events.extend([4, 0, 4, 1, 4, 2]);
    }
    fn set_alpha(&mut self, alpha: u8) {
        self.values[12] = u32::from(alpha);
    }
    fn set_shadow_alpha(&mut self, alpha: u8) {
        self.values[13] = u32::from(alpha);
    }
}

#[test]
#[ignore]
fn zone_title_tick_10121370_and_width_10121260_match_original() {
    let mut cases = Vec::new();
    let times = [0.0_f32, 0.5, 1.999, 2.0, 2.001, 4.0, 4.001, 5.0, 5.5, 6.0, 8.0, 8.001, -1.0];
    for lengths in
        [[u32::MAX, 3, 1], [3, u32::MAX, 2], [0, 0, 0], [0, 3, 1], [3, 3, 3], [3, 4, 4], [3, 2, 5], [5, 2, 1]]
    {
        for screen in [[1558, 933], [1921, 1081], [0, 0], [u32::MAX, u32::MAX], [0x80000000, 0x7fffffff]] {
            for visible in [0, 1] {
                for time in times {
                    cases.push([
                        lengths[0],
                        lengths[1],
                        lengths[2],
                        71,
                        25,
                        13,
                        20,
                        screen[0],
                        screen[1],
                        visible,
                        time.to_bits(),
                        0,
                        211,
                        97,
                    ]);
                }
            }
        }
    }
    let floats = [
        0, 1, 0x80000000, 0x7f800000, 0xff800000, 0x7fc00123, 0x7f800123, 0xffc00123, 0x7fc00456, 0xff800456,
        0x3fffffff, 0x40000000, 0x40800000, 0x41000000, 0x41000001,
    ];
    for elapsed in floats {
        for delta in floats {
            cases.push([2, 1, 1, 11, 300, 5, 17, 1279, 721, 1, elapsed, delta, 31, 251]);
        }
    }
    let mut seed = 0x7abe0132_u32;
    for _ in 0..1024 {
        let mut v = [0; 14];
        for n in &mut v {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            *n = seed;
        }
        for n in &mut v[..3] {
            *n %= 64;
        }
        v[9] &= 1;
        v[12] &= 255;
        v[13] &= 255;
        cases.push(v);
    }
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/zone_title_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script)
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let input_cases = cases.clone();
    let writer = std::thread::spawn(move || {
        for v in input_cases {
            writeln!(input, "{}", v.map(|n| n.to_string()).join(" ")).unwrap();
        }
    });
    let output = child.wait_with_output().unwrap();
    writer.join().unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    for (v, line) in cases.iter().zip(output.lines()) {
        let mut backend = Backend {
            values: *v,
            counts: [v[0].wrapping_add(1), v[1].wrapping_add(1), v[2].wrapping_add(1)],
            events: Vec::new(),
        };
        tick_zone_title(&mut backend, f32::from_bits(v[11]));
        let mut actual = vec![
            i64::from(backend.values[10]),
            i64::from(backend.values[12]),
            i64::from(backend.values[13]),
            i64::from(backend.values[9]),
        ];
        actual.extend(backend.counts.map(i64::from));
        actual.extend(backend.events);
        let expected: Vec<i64> = line.split_whitespace().map(|n| n.parse().unwrap()).collect();
        assert_eq!(actual, expected, "{v:?}");
    }
    eprintln!("{} cenários comparados com a DLL", cases.len());
}
