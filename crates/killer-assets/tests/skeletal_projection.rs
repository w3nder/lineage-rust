use killer_assets::skeletal_lod::{lod_world_point, max_draw_scale, multiply_matrices, projection_reference_point, scene_project};
use serde_json::json;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
#[ignore]
fn project_10613b10_and_get_max_10117fd0_match_original_functions() {
    let mut cases = Vec::new();
    let mut expected = Vec::<Vec<f32>>::new();
    let values = [0, 0x80000000, 0x3f800000, 0xbf800000, 0x7f800000, 0xff800000, 0x7fc00000, 0x7f800001, 1, 0x7f7fffff];
    for component in 0..3 {
        for x in values {
            for y in values {
                for controller in [false, true] {
                    for has_pawn in [false, true] {
                        let mut point = [0; 3];
                        let mut viewer = [0; 3];
                        point[component] = x;
                        viewer[component] = y;
                        let viewer = has_pawn.then_some(viewer);
                        cases.push(json!({"reference_point": point, "viewer": viewer, "controller": controller}));
                        expected.push(projection_reference_point(
                            point.map(f32::from_bits),
                            viewer.filter(|_| controller).map(|v| v.map(f32::from_bits)),
                        ).to_vec());
                    }
                }
            }
        }
    }
    for x in values {
        for y in values {
            for z in values {
                let vector = [x, y, z];
                cases.push(json!({"vector": vector}));
                expected.push(vec![max_draw_scale(vector.map(f32::from_bits))]);
            }
        }
    }
    let mut seed = 0x12345678u32;
    for case in 0..4096 {
        let mut next = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            if case & 1 == 0 { f32::from_bits(seed) } else { (seed as i32) as f32 / 65536.0 }
        };
        let matrix: [[f32; 4]; 4] = std::array::from_fn(|_| std::array::from_fn(|_| next()));
        let right: [[f32; 4]; 4] = std::array::from_fn(|_| std::array::from_fn(|_| next()));
        cases.push(json!({
            "left": matrix.into_iter().flatten().map(f32::to_bits).collect::<Vec<_>>(),
            "right": right.into_iter().flatten().map(f32::to_bits).collect::<Vec<_>>(),
        }));
        expected.push(multiply_matrices(matrix, right).into_iter().flatten().collect());
        let point = std::array::from_fn(|_| next());
        cases.push(json!({
            "matrix": matrix.into_iter().flatten().map(f32::to_bits).collect::<Vec<_>>(),
            "point": point.map(f32::to_bits),
        }));
        expected.push(scene_project(matrix, point).to_vec());
        let location = [11.0, -23.0, 42.0];
        let soft = case & 2 != 0;
        let root = (case & 4 != 0).then_some(point);
        cases.push(json!({
            "matrix": matrix.into_iter().flatten().map(f32::to_bits).collect::<Vec<_>>(),
            "world_location": location.map(f32::to_bits),
            "root": root.map(|v| v.map(f32::to_bits)), "soft": soft,
        }));
        expected.push(lod_world_point(location, root.filter(|_| soft), matrix).to_vec());
    }
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").expect("KILLER_ORACLE_PYTHON"))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/skeletal_projection_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&serde_json::to_vec(&cases).unwrap()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let native: Vec<Vec<u32>> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(native.len(), expected.len());
    for (index, (native, expected)) in native.iter().zip(&expected).enumerate() {
        assert_eq!(native.len(), expected.len());
        for (&native, &expected) in native.iter().zip(expected) {
            if f32::from_bits(native).is_nan() && cases[index].get("reference_point").is_none() {
                assert!(expected.is_nan(), "case {index}: {}", cases[index]);
            } else {
                assert_eq!(native, expected.to_bits(), "case {index}: {}", cases[index]);
            }
        }
    }
    eprintln!("{} casos comparados com funções originais", cases.len());
}
