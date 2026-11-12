use killer_ui::positioning_rect::PositioningRect;
use killer_ui::window_clip::clip_to_parent;
use std::io::Write;
use std::process::{Command, Stdio};

fn rect(v: &[u32]) -> PositioningRect {
    PositioningRect { x: v[0] as i32, y: v[1] as i32, width: f32::from_bits(v[2]), height: f32::from_bits(v[3]) }
}

fn bits(r: PositioningRect) -> [u32; 4] {
    [r.x as u32, r.y as u32, r.width.to_bits(), r.height.to_bits()]
}

#[test]
#[ignore]
fn clipping_matches_original_geometry_offsets_output_and_hide() {
    let mut cases = Vec::new();
    for chain in 0..4 {
        for x in [-101i32, -100, -1, 0, 1, 99, 100, 101] {
            for y in [-101i32, -100, -1, 0, 1, 99, 100, 101] {
                for size in [0u32, 0x80000000, 0x3f000000, 0x42c90000, 0x7f800000, 0x7fc00123, 0x7f800123, 0xff800000] {
                    cases.push([chain, x as u32, y as u32, size, size, 0, 0, 100f32.to_bits(), 100f32.to_bits(), 23, 37, 7, 9, 81f32.to_bits(), 91f32.to_bits()]);
                }
            }
        }
    }
    let mut seed = 0xc1435821u32;
    for i in 0..4096 {
        let mut v = [0; 15];
        for n in &mut v { seed = seed.wrapping_mul(1664525).wrapping_add(1013904223); *n = seed; }
        v[0] = if i % 8 == 0 { i % 4 } else { 3 };
        if i & 1 != 0 {
            for n in [1, 2, 5, 6] { v[n] = ((v[n] % 301) as i32 - 150) as u32; }
            for n in [3, 4, 7, 8] { v[n] = ((v[n] % 401) as f32 * 0.5).to_bits(); }
        }
        cases.push(v);
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/window_clip_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    for v in &cases { writeln!(input, "{}", v.iter().map(u32::to_string).collect::<Vec<_>>().join(" ")).unwrap(); }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    for (v, line) in cases.iter().zip(output.lines()) {
        let mut window = rect(&v[1..5]);
        let mut offsets = (v[9] as i32, v[10] as i32);
        let mut out = rect(&v[11..15]);
        let hidden = clip_to_parent(&mut window, &mut offsets, &mut out, (v[0] == 3).then(|| rect(&v[5..9])));
        let expected: Vec<_> = [u32::from(hidden)].into_iter().chain(bits(window)).chain([offsets.0 as u32, offsets.1 as u32]).chain(bits(out)).collect();
        let actual: Vec<u32> = line.split_whitespace().map(|value| value.parse().unwrap()).collect();
        assert_eq!(actual, expected, "case={v:?}");
    }
}
