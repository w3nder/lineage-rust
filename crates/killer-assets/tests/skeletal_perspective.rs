use killer_assets::skeletal_lod::{CameraProjectionInput, camera_projection_matrix, perspective_matrix};
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
#[ignore]
fn camera_projection_106162ff_matches_all_mode_branches() {
    let mut cases = Vec::new();
    let mut matrices = Vec::new();
    for mode in -1..=40 {
        for size in [[1558, 933], [0, 0], [1, 1], [16777217, 16777219], [-1, 10]] {
            for unscaled in [false, true] {
                for zoom in [0.0f32, 10000.0] {
                    let input = CameraProjectionInput {
                        render_mode: mode, fov: 60.0, target_size: size, viewport_width: size[0],
                        unscaled_ortho: unscaled, ortho_zoom: zoom, near: 10.0, far: 65536.0,
                    };
                    cases.push(serde_json::json!({
                        "mode": mode, "target_size": size, "viewport_width": input.viewport_width,
                        "unscaled_ortho": unscaled, "ortho_zoom": zoom.to_bits(),
                        "fov": input.fov.to_bits(), "near": input.near.to_bits(), "far": input.far.to_bits(),
                    }));
                    matrices.push(camera_projection_matrix(input));
                }
            }
        }
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").expect("KILLER_ORACLE_PYTHON"))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/skeletal_perspective_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(&serde_json::to_vec(&cases).unwrap()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let native: Vec<[u32; 16]> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(native.len(), matrices.len());
    for (index, (actual, expected)) in matrices.iter().zip(native).enumerate() {
        for (actual, expected) in actual.iter().flatten().zip(expected) {
            if actual.is_nan() {
                assert!(f32::from_bits(expected).is_nan(), "{}", cases[index]);
            } else {
                assert_eq!(actual.to_bits(), expected, "{}", cases[index]);
            }
        }
    }
    eprintln!("{} projeções comparadas em todos os ramos de modo", cases.len());
}

#[test]
#[ignore]
fn perspective_10614ff0_matches_native_constructor() {
    let mut inputs = Vec::new();
    for fov in [1.0f32, 30.0, 50.0, 60.0, 90.0, 120.0, 179.0] {
        let half = (f64::from(fov) * f64::from_bits(0x3f81_df46_a252_9d39)) as f32;
        for other in [half, 0.2, 0.7] {
            for multiplier in [[1.0, 1.0], [1.0, 16.0 / 9.0], [0.75, 1.25]] {
                for near in [0.0, 1.0, 10.0, 20.0] {
                    for far in [1.0, 10.0, 65536.0, 1000000.0] {
                        inputs.push([half, other, multiplier[0], multiplier[1], near, far]);
                    }
                }
            }
        }
    }
    let bits: Vec<_> = inputs.iter().map(|case| case.map(f32::to_bits)).collect();
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").expect("KILLER_ORACLE_PYTHON"))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/skeletal_perspective_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(&serde_json::to_vec(&bits).unwrap()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let native: Vec<[u32; 16]> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(native.len(), inputs.len());
    for (case, expected) in inputs.iter().zip(native) {
        let actual = perspective_matrix([case[0], case[1]], [case[2], case[3]], case[4], case[5]);
        for (index, (actual, expected)) in actual.into_iter().flatten().zip(expected).enumerate() {
            if actual.is_nan() {
                assert!(f32::from_bits(expected).is_nan(), "{case:?} [{index}]");
            } else {
                assert_eq!(actual.to_bits(), expected, "{case:?} [{index}]");
            }
        }
    }
    eprintln!("{} matrizes comparadas com o construtor original", inputs.len());
}
