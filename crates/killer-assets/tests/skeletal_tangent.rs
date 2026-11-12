use std::io::Write;
use std::process::{Command, Stdio};

#[test]
#[ignore]
fn render_1069a860_host_tangent_agrees_on_camera_samples() {
    let mut inputs: Vec<f32> = (0..=7200).map(|index| index as f32 / 20.0).collect();
    for center in [50.0f32, 60.0, 90.0, 180.0] {
        for offset in -32i64..=32 {
            inputs.push(f32::from_bits((i64::from(center.to_bits()) + offset) as u32));
        }
    }
    let bits: Vec<_> = inputs.iter().map(|value| value.to_bits()).collect();
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").expect("KILLER_ORACLE_PYTHON"))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/skeletal_tangent_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&serde_json::to_vec(&bits).unwrap()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let native: Vec<u32> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(native.len(), inputs.len());
    for (fov, native) in inputs.iter().zip(native) {
        let candidate = killer_assets::skeletal_lod::half_fov_tangent(*fov);
        assert_eq!(candidate.to_bits(), native, "FOV {fov:?}");
    }
    eprintln!("{} amostras de FOV comparadas, não todos os f32", inputs.len());
}
