use killer_ui::canvas_clip::CanvasClip;
use std::io::Write;
use std::process::{Command, Stdio};

fn state(values: &[u32]) -> CanvasClip {
    let pair = |index| [f32::from_bits(values[index]), f32::from_bits(values[index + 1])];
    CanvasClip { flags: values[0], origin: pair(1), size: pair(3), cursor: pair(5), saved_cursor: pair(7), saved_origin: pair(9), saved_size: pair(11) }
}

#[test]
#[ignore]
fn canvas_clip_matches_native_bit_copies_conversion_and_repeated_state() {
    let edges = [0, 0x80000000, 0x7fffffff, 0xffffffff, 16777217, 0x7f800000, 0x7fc12345, 0x7f812345];
    let mut cases = Vec::new();
    for kind in 0..3 {
        for flags in [0, 8, 0xfffffff7, u32::MAX] {
            for shift in 0..edges.len() {
                let mut values = vec![kind, flags];
                values.extend((0..16).map(|i| edges[(i + shift) % edges.len()])); cases.push(values);
            }
        }
    }
    let mut seed = 0x139c9ab1_u32;
    for index in 0..1024 {
        let mut values = vec![index % 3];
        for _ in 0..17 { seed = seed.wrapping_mul(1664525).wrapping_add(1013904223); values.push(seed); }
        cases.push(values);
    }
    assert_eq!(cases.len(), 1120);
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/canvas_clip_reference.py"))
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
        let before = state(&values[1..14]);
        let mut clip = before;
        if values[0] != 1 { clip.set_clip([values[14] as i32, values[15] as i32, values[16] as i32, values[17] as i32]); }
        if values[0] == 2 { clip.origin = before.origin; clip.size = before.size; clip.cursor = before.cursor; }
        if values[0] != 0 { clip.release_clip(); }
        let mut actual = vec![clip.flags];
        for pair in [clip.origin, clip.size, clip.cursor, clip.saved_cursor, clip.saved_origin, clip.saved_size] { actual.extend(pair.map(f32::to_bits)); }
        let expected: Vec<u32> = line.split_whitespace().map(|word| word.parse().unwrap()).collect();
        assert_eq!(actual, expected, "{values:?}");
    }
}
