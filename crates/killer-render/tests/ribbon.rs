use glam::Vec3;
use killer_assets::client::Client;
use killer_render::particles::ribbon::{RibbonPoint, RibbonState};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

#[test]
#[ignore]
fn ribbon_update_10655f30_matches_native_sampling_decay_and_interpolation() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let actor = client.emitter_class("LineageEffect.RibbonSet").unwrap();
    let original = actor.sprites.iter().find_map(|e| e.ribbon.as_ref()).unwrap();
    assert_eq!(actor.sprites[0].max_particles, 1);
    assert_eq!(original.num_points, 80);
    assert_eq!(original.axis, 3);
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/ribbon_reference.py");
    let python = std::env::var_os("KILLER_ORACLE_PYTHON").unwrap_or_else(|| "python3".into());
    let mut child =
        Command::new(python).arg(script).arg(&root).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    let mut calls = 0;
    for flags in [0, 1, 2, 3, 5, 11, 15, 95] {
        for acc_drop in 0..=3 {
            for interpolate in [false, true] {
                let mut shape = original.clone();
                shape.flags = flags;
                shape.sheets = if interpolate { 3 } else { 1 };
                shape.acc_drop = acc_drop;
                shape.interpolate = interpolate;
                shape.num_points = 20;
                shape.drop_rate = 2;
                shape.min_points = 3;
                shape.threshold = 1.5;
                shape.threshold2 = 2.0;
                shape.sample_rate = 0.05;
                let commands: Vec<Value> = (0..45).map(|i| {
                    let move_time = if i < 22 { i as f32 } else { 21.0 };
                    json!({"edges": [[move_time * 1.7 + 1.0, move_time.sin() * 2.0, 3.0], [move_time + 1.0, 4.0, move_time.cos()]],
                        "delta": 0.1, "age": i as f32 * 0.1, "lifetime": 6.0, "stopped": i > 30})
                }).collect();
                let request = json!({"flags": flags, "acc_drop": acc_drop, "interpolate": interpolate, "limit": shape.num_points,
                    "drop_rate": shape.drop_rate, "min_points": shape.min_points, "threshold": shape.threshold, "threshold2": shape.threshold2,
                    "min_dist": shape.min_sample_dist, "sample_rate": shape.sample_rate, "control_points": shape.control_points, "scale_ratio": shape.scale_ratio, "sheets": shape.sheets, "uv_scale": shape.uv_scale, "commands": commands});
                writeln!(input, "{request}").unwrap();
                input.flush().unwrap();
                let mut line = String::new();
                output.read_line(&mut line).unwrap();
                let native: Vec<Value> = serde_json::from_str(&line).unwrap();
                let mut state = RibbonState::new(&shape);
                for (step, (command, expected)) in commands.iter().zip(native).enumerate() {
                    let edges: [[f32; 3]; 2] = serde_json::from_value(command["edges"].clone()).unwrap();
                    state.update(
                        &shape,
                        0.1,
                        step > 30,
                        step as f32 * 0.1,
                        6.0,
                        RibbonPoint { edges: edges.map(Vec3::from), width: 20.0, ..Default::default() },
                    );
                    let context = format!("flags={flags} acc={acc_drop} interpolate={interpolate} step={step}");
                    assert_eq!(state.limit, expected["limit"].as_i64().unwrap() as i32, "{context}");
                    assert_eq!(
                        u32::from(state.killed) | (u32::from(state.decaying) << 1),
                        expected["flags"].as_u64().unwrap() as u32,
                        "{context}"
                    );
                    near(state.timer, f32::from_bits(expected["timer"].as_u64().unwrap() as u32), &context);
                    near(state.sample_rate, f32::from_bits(expected["rate"].as_u64().unwrap() as u32), &context);
                    let points = expected["points"].as_array().unwrap();
                    assert_eq!(state.points.len(), points.len(), "{context}");
                    for (point, native) in state.points.iter().zip(points) {
                        let floats = point
                            .edges
                            .into_iter()
                            .chain(point.tangents)
                            .flat_map(|v| v.to_array())
                            .chain([point.width]);
                        for (got, want) in floats.zip(native.as_array().unwrap()) {
                            near(got, f32::from_bits(want.as_u64().unwrap() as u32), &context);
                        }
                    }
                    if let Some(expected_vertices) = expected.get("vertices") {
                        let (vertices, indices) = state.vertices(&shape, step > 30, [255; 4]);
                        assert_eq!(
                            indices,
                            serde_json::from_value::<Vec<u32>>(expected["indices"].clone()).unwrap(),
                            "{context}"
                        );
                        assert_eq!(vertices.len(), expected_vertices.as_array().unwrap().len(), "{context}");
                        for (v, native) in vertices.iter().zip(expected_vertices.as_array().unwrap()) {
                            for (value, slot) in v.position.into_iter().chain(v.uv).zip([0, 1, 2, 4, 5]) {
                                near(
                                    value,
                                    f32::from_bits(native[slot].as_u64().unwrap() as u32),
                                    &format!("geometry {context}"),
                                );
                            }
                        }
                    }
                    calls += 1;
                }
            }
        }
    }
    drop(input);
    assert!(child.wait().unwrap().success());
    eprintln!("Ribbon_Update 0x10655f30: {calls} passos nativos aprovados");
}

fn near(got: f32, want: f32, context: &str) {
    assert!(
        got == want || (got.is_nan() && want.is_nan()) || (got - want).abs() <= 0.0001,
        "{context}: Rust {got}, DLL {want}"
    );
}

#[test]
#[ignore]
fn ribbon_set_defaults_feed_get_new_ribbon_point_bone_to_bone_relative() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let actor = client.emitter_class("LineageEffect.RibbonSet").unwrap();
    let emitter = actor.sprites.iter().find(|e| e.ribbon.is_some()).unwrap();
    let ribbon = emitter.ribbon.as_ref().unwrap();
    eprintln!("coordinate_system {} flags {:#x} axis {} bones {:?} offsets {:?} po {} width {} points {} threshold {} {} max {} min_dist {} uv {:?} sheets {}",
        emitter.coordinate_system, ribbon.flags, ribbon.axis, ribbon.bone_names, ribbon.bone_offsets, ribbon.origin_offset, ribbon.width,
        ribbon.num_points, ribbon.threshold, ribbon.threshold2, emitter.max_particles, ribbon.min_sample_dist, ribbon.uv_scale, ribbon.sheets);
    assert_eq!(emitter.coordinate_system, 1);
    assert_eq!(ribbon.axis, 3);
}

#[test]
fn relative_ribbon_points_return_to_the_bone_through_the_render_transform() {
    use killer_assets::ribbon_point::{Actor, Bone, Emitter, Host, get_new_ribbon_point};
    use killer_render::particles::rotator_matrix;
    for (rotation, bone_rotation) in [([0, 0, 0], [0, 0, 0]), ([0, 16384, 0], [1200, -7000, 300]), ([-1500, 40000, 900], [9000, 3000, -2000])] {
        let owner = Actor { location: [120.0, -40.0, 15.0], rotation };
        let bone = Bone { origin: [150.0, -20.0, 60.0], rotation: bone_rotation };
        let mut emitter = Emitter {
            coordinate_system: 1,
            axis: 3,
            flags: 0x5f,
            width: 30.0,
            origin_offset: 0.5,
            start_offset: [15.0, 0.0, 0.0],
            end_offset: [30.0, 0.0, 0.0],
            rotation: [0; 3],
            has_start_name: true,
        };
        let host = Host { actor: Actor { location: [0.0; 3], rotation: [0; 3] }, owner: Some(owner), owner_mesh: true };
        let sample = get_new_ribbon_point(&mut emitter, Some(&host), true, Some(bone), Some(bone), &mut Vec::new(), 30.0);
        let world = |p: [f32; 3]| rotator_matrix(owner.rotation) * Vec3::from(p) + Vec3::from(owner.location);
        let axis = rotator_matrix(bone_rotation) * Vec3::X;
        let base = Vec3::from(bone.origin) + axis * 15.0;
        assert!((world(sample.a) - (base + axis * 15.0)).length() < 1e-2, "{rotation:?} {:?}", world(sample.a));
        assert!((world(sample.b) - (base - axis * 15.0)).length() < 1e-2, "{rotation:?} {:?}", world(sample.b));
    }
}
