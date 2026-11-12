use killer_ui::ui_device::{CanvasTexture, CanvasUv, TextureInput, TextureResources, UiDevice};
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
#[ignore]
fn texture_device_wrappers_match_original_arguments_alpha_and_bounds() {
    let mut cases = Vec::new();
    for kind in 0..2 {
        for mask in 0..5 {
            for alpha in (0..256).chain([0x100, 0x1ff, u32::MAX]) {
                for state in [0, 1, u32::MAX] {
                    for mutation in 0..2 {
                        cases.push([kind, mask, alpha, state, alpha % 4, mutation]);
                    }
                }
            }
        }
    }
    assert_eq!(cases.len(), 15540);
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/ui_device_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    for values in &cases { writeln!(input, "{}", values.iter().map(u32::to_string).collect::<Vec<_>>().join(" ")).unwrap(); }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap();
    assert_eq!(output.lines().count(), cases.len());
    let edges = [(0, 0), (1, -1), (-1, i32::MAX), (i32::MIN, i32::MIN)];
    for (values, line) in cases.into_iter().zip(output.lines()) {
        let [kind, mask, alpha, state, edge, mutation] = values;
        let (y, h) = edges[edge as usize];
        let mut device = UiDevice { origin: [i32::MAX; 2], minimum_y: 0, maximum_y: 0, transparency_state: state };
        let input = TextureInput { rect: [1, y, -1, h], uv: [i32::MIN, i32::MAX, 16777217, -16777217], texture: 7, alpha };
        let resources = (mask != 0).then_some(TextureResources { primary: mask & 1 != 0, secondary: mask & 2 != 0 });
        let mut events = Vec::new();
        let callback = |device: &mut UiDevice, call: CanvasTexture<u32>| {
            events.extend(call.rect.map(|v| v as u32));
            events.extend(match call.uv { CanvasUv::Texture(uv) => uv.map(f32::to_bits), CanvasUv::Stretched(uv) => uv.map(|v| v as u32) });
            events.extend([call.texture, call.alpha]);
            if let Some(clip) = call.clip { events.push(clip); }
            if mutation != 0 { device.origin[1] = -100; device.minimum_y = 100; device.maximum_y = -100; }
            Ok::<_, ()>(())
        };
        if kind == 0 { device.draw_texture(input, resources, 123, callback).unwrap(); }
        else { device.draw_stretched_texture(input, resources, callback).unwrap(); }
        let mut actual = vec![device.origin[1] as u32, device.minimum_y as u32, device.maximum_y as u32];
        actual.extend(events);
        let expected: Vec<u32> = line.split_whitespace().map(|word| word.parse().unwrap()).collect();
        assert_eq!(actual, expected, "{values:?}");
    }
}
