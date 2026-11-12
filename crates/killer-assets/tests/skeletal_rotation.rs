use killer_assets::skeletal_lod::{camera_view_matrix, inverse_rotation_matrix, sine_table};
use serde_json::json;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
#[ignore]
fn inverse_rotation_10327a50_matches_native_with_supplied_table() {
    compare_rotation(false);
}

#[test]
#[ignore]
fn sine_table_1014f1f0_and_inverse_rotation_match_original() {
    compare_rotation(true);
}

fn compare_rotation(native_table: bool) {
    let table: [f32; 16384] = if native_table { sine_table() } else {
        std::array::from_fn(|i| (i as f32 - 8192.0) / 8192.0)
    };
    let mut rotations = Vec::new();
    for axis in 0..3 {
        for angle in [i32::MIN, i32::MIN + 1, -65537, -65536, -16385, -16384, -5, -4, -3, -2, -1,
            0, 1, 2, 3, 4, 5, 16383, 16384, 16385, 65535, 65536, 65537, i32::MAX] {
            let mut rotation = [0; 3];
            rotation[axis] = angle;
            rotations.push(rotation);
        }
    }
    let mut seed = 0xabcdef01u32;
    for _ in 0..4096 {
        rotations.push(std::array::from_fn(|_| {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            seed as i32
        }));
    }
    let supplied = (!native_table).then(|| table.map(f32::to_bits).to_vec());
    let mut views = Vec::new();
    let mut expected_views = Vec::new();
    for mode in -1..=40 {
        for rotation in rotations.iter().step_by(64) {
            let location = [111429.0f32, 219384.0, -3544.0];
            for scale in [[1.0f32, 1.0], [-1.0, 0.75]] {
                views.push(json!({"mode": mode, "location": location.map(f32::to_bits), "rotation": rotation, "scale": scale.map(f32::to_bits)}));
                expected_views.push(camera_view_matrix(location, *rotation, mode, scale, &table));
            }
        }
    }
    let data = json!({"table": supplied, "rotations": rotations, "views": views});
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").expect("KILLER_ORACLE_PYTHON"))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/skeletal_rotation_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(&serde_json::to_vec(&data).unwrap()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let mut native: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let native_table: Vec<u32> = serde_json::from_value(native["table"].take()).unwrap();
    assert_eq!(native_table, table.map(f32::to_bits).to_vec(), "tabela inteira");
    let native_views: Vec<[u32; 16]> = serde_json::from_value(native["views"].take()).unwrap();
    assert_eq!(native_views.len(), expected_views.len());
    for (index, (expected, native)) in expected_views.iter().zip(native_views).enumerate() {
        let bits: Vec<_> = expected.iter().flatten().map(|value| value.to_bits()).collect();
        assert_eq!(bits, native, "view {}", views[index]);
    }
    let native: Vec<[u32; 16]> = serde_json::from_value(native["rotations"].take()).unwrap();
    assert_eq!(native.len(), rotations.len());
    for (rotation, expected) in rotations.iter().zip(native) {
        let actual = inverse_rotation_matrix(*rotation, &table);
        let bits: Vec<_> = actual.into_iter().flatten().map(f32::to_bits).collect();
        assert_eq!(bits, expected, "{rotation:?}");
    }
    eprintln!("{} rotações e {} entradas da tabela comparadas", rotations.len(), native_table.len());
}
