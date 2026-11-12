use killer_ui::canvas_clip::CanvasClip;
use killer_ui::canvas_texture::{CanvasTextureInput, texture_tile};
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
#[ignore]
fn canvas_texture_matches_original_clipping_and_draw_tile_arguments() {
    let mut cases = Vec::new();
    let mut seed = 0x53829312_u32;
    for index in 0..4096 {
        let mut values = [0u32; 15];
        for value in &mut values[..4] {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            *value = ((seed % 400) as i32 - 100) as u32;
        }
        for value in &mut values[4..12] {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            *value = (((seed % 1000) as i32 - 100) as f32 / 3.0).to_bits();
        }
        values[12] = index;
        values[13] = [0, 1, 0x100, u32::MAX][index as usize % 4];
        values[14] = 0.5f32.to_bits();
        cases.push(values);
    }
    let edges = [0, 0x80000000, 1, 0x80000001, 0x7fffffff, 0xffffffff, 16777217, 0x4f000000, 0xcf000000, 0x7f800000, 0xff800000, 0x7fc12345, 0x7f812345, 0xffc23456, 0x3f800000, 0xbf800000];
    for field in 0..15 {
        for edge in edges {
            for clip in [0, 1, 0x100, u32::MAX] {
                let mut values = [0, 0, 100, 100, 0, 0, 100.0f32.to_bits(), 100.0f32.to_bits(), 10.0f32.to_bits(), 20.0f32.to_bits(), 30.0f32.to_bits(), 40.0f32.to_bits(), 255, clip, 0.5f32.to_bits()];
                values[field] = edge;
                cases.push(values);
            }
        }
    }
    for _ in 0..1024 {
        let mut values = [0; 15];
        for value in &mut values { seed = seed.wrapping_mul(1664525).wrapping_add(1013904223); *value = seed; }
        cases.push(values);
    }
    assert_eq!(cases.len(), 6080);
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/canvas_texture_reference.py"))
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
        let canvas = CanvasClip { flags: 0, origin: [f32::from_bits(values[8]), f32::from_bits(values[9])], size: [f32::from_bits(values[10]), f32::from_bits(values[11])], cursor: [0.0; 2], saved_cursor: [0.0; 2], saved_origin: [0.0; 2], saved_size: [0.0; 2] };
        let input = CanvasTextureInput { texture: (), rect: [values[0] as i32, values[1] as i32, values[2] as i32, values[3] as i32], uv: [f32::from_bits(values[4]), f32::from_bits(values[5]), f32::from_bits(values[6]), f32::from_bits(values[7])], alpha: values[12], clip: values[13] };
        let mut actual = vec![0];
        if let Some(tile) = texture_tile(&canvas, f32::from_bits(values[14]), input) {
            actual[0] = 1;
            actual.extend(tile.rect.map(f32::to_bits)); actual.extend(tile.uv.map(f32::to_bits)); actual.push(tile.depth.to_bits());
            actual.extend(tile.color.map(f32::to_bits)); actual.extend(tile.secondary_color.map(f32::to_bits));
        }
        let expected: Vec<u32> = line.split_whitespace().map(|word| word.parse().unwrap()).collect();
        assert_eq!(actual, expected, "{values:?}");
    }
}
