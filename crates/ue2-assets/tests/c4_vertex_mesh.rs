use std::path::PathBuf;
use ue2_package::Package;

#[test]
#[ignore]
fn serialize_107093d0_reads_every_vertex_mesh_in_effect_package() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let path = root.join("Animations/LineageEffectMeshes.ukx");
    let encoded = std::fs::read(&path).unwrap();
    let file = l2_crypto::decrypt(&encoded, &path).unwrap().into_owned();
    let package = Package::parse(&file).unwrap();
    let mut count = 0;
    for export in package.exports().iter().filter(|e| package.class_name(e).eq_ignore_ascii_case("VertMesh")) {
        let mesh = ue2_assets::read_vertex_mesh(&package, &file, export)
            .unwrap_or_else(|e| panic!("{}: {e}", package.name(export.name)));
        eprintln!("{}: vertices={} frames={} sequences={} faces={}", package.name(export.name), mesh.vertex_count, mesh.positions.len() / mesh.vertex_count, mesh.sequences.len(), mesh.faces.len());
        assert!(!mesh.sequences.is_empty());
        count += 1;
    }
    assert_eq!(count, 15);
}

#[test]
#[ignore]
fn get_frame_107069c0_matches_native_for_every_effect_vertex_mesh() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    use ue2_assets::VertexPose;

    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let path = root.join("Animations/LineageEffectMeshes.ukx");
    let encoded = std::fs::read(&path).unwrap();
    let file = l2_crypto::decrypt(&encoded, &path).unwrap().into_owned();
    let package = Package::parse(&file).unwrap();
    let mut input = String::new();
    let mut expected = Vec::new();
    let mut case_count = 0;
    for export in package.exports().iter().filter(|e| package.class_name(e).eq_ignore_ascii_case("VertMesh")) {
        let mesh = ue2_assets::read_vertex_mesh(&package, &file, export).unwrap();
        let mut cases = Vec::new();
        for (index, seq) in mesh.sequences.iter().enumerate() {
            for i in 0..=seq.frames * 2 {
                cases.push((true, Some(index), i as f32 / (seq.frames * 2) as f32, mesh.vertex_count));
            }
            let start = -1.0 / seq.frames as f32;
            cases.extend([
                (true, Some(index), start, mesh.vertex_count),
                (false, Some(index), 0.5, mesh.vertex_count / 2),
                (false, Some(index), start, mesh.vertex_count),
                (false, Some(index), start * 0.5, mesh.vertex_count),
                (false, Some(index), start * 0.01, mesh.vertex_count),
                (false, Some(index), 0.0, mesh.vertex_count),
                (false, None, start, mesh.vertex_count),
                (false, None, start * 0.5, mesh.vertex_count),
                (false, None, 0.0, mesh.vertex_count),
                (true, None, -0.1, mesh.vertex_count),
                (true, None, 0.5, 0),
                (false, Some(index), 0.5, mesh.vertex_count + 1),
            ]);
        }
        let mut pose = VertexPose::new(&mesh);
        let mut outputs = Vec::new();
        for &(reset, sequence, frame, limit) in &cases {
            if reset { pose = VertexPose::new(&mesh); }
            outputs.push(pose.sample(sequence, frame, limit).unwrap().to_vec());
        }
        case_count += cases.len();
        let value = serde_json::json!({
            "vertices": mesh.vertex_count, "positions": mesh.positions, "normals": mesh.normals,
            "sequences": mesh.sequences.iter().map(|s| serde_json::json!({"start":s.start,"frames":s.frames})).collect::<Vec<_>>(),
            "cases": cases.iter().map(|(reset, sequence, frame, limit)| serde_json::json!({"reset":reset,"sequence":sequence,"frame":frame,"limit":limit})).collect::<Vec<_>>()
        });
        input.push_str(&value.to_string());
        input.push('\n');
        expected.push((package.name(export.name).to_owned(), outputs));
    }
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/vertex_frame_reference.py");
    let mut child = Command::new(std::env::var_os("KILLER_ORACLE_PYTHON").unwrap_or_else(|| "python3".into()))
        .arg(script).arg(&root).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || stdin.write_all(input.as_bytes()));
    let result = child.wait_with_output().unwrap();
    writer.join().unwrap().unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let output = String::from_utf8(result.stdout).unwrap();
    assert_eq!(output.lines().count(), expected.len());
    for (line, (name, cases)) in output.lines().zip(expected) {
        let native: Vec<[Vec<u32>; 2]> = serde_json::from_str(line).unwrap();
        assert_eq!(native.len(), cases.len());
        for (case, (native, points)) in native.iter().zip(cases).enumerate() {
            assert_eq!(native[0].len(), points.len() * 3, "{name} case {case}");
            for (i, point) in points.iter().enumerate() {
                for (component, values) in [point.position, point.normal].iter().enumerate() {
                    for (axis, &value) in values.iter().enumerate() {
                        let original = f32::from_bits(native[component][i * 3 + axis]);
                        assert!((value - original).abs() <= 0.0001, "{name} case={case} vertex={i} component={component} axis={axis}: {value} != {original}");
                    }
                }
            }
        }
    }
    eprintln!("15 malhas, {case_count} chamadas GetFrame nativas, todos os vértices e normais");
}

#[test]
#[ignore]
fn play_107053f0_and_update_10704f00_match_original_branches() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    use ue2_assets::{VertexAnimEvent, VertexAnimation, VertexSequence};
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let sequences = vec![
        VertexSequence { name: "many".into(), groups: vec![], start: 0, frames: 31, rate: 30.0, notifies: vec![] },
        VertexSequence { name: "notifies".into(), groups: vec![], start: 0, frames: 30, rate: 30.0, notifies: [0.8, 0.2, 0.5, 0.5].map(|time| (time, String::new(), ue2_package::ObjectRef::Export(0))).to_vec() },
        VertexSequence { name: "one".into(), groups: vec![], start: 0, frames: 1, rate: 15.0, notifies: vec![] },
    ];
    let mut commands = Vec::new();
    let mut expected = Vec::new();
    let mut animation = VertexAnimation::default();
    for looping in [false, true] {
        for rate in [-1.0, 0.0, 0.5, 1.0, 2.0] {
            for tween in [-1.0, 0.0, 0.2] {
                for index in [0, 0, 1, 2, 3] {
                    let speed = if rate < 0.0 { 4.0 } else { 0.0 };
                    commands.push(serde_json::json!({"play":[index,rate,tween,looping],"speed":speed}));
                    animation.play(&sequences, sequences.get(index).map_or("missing", |s| s.name.as_str()), rate, tween, looping, speed);
                    expected.push((animation.clone(), Vec::<i32>::new()));
                    for (delta, enabled) in [(0.0, true), (0.016, false), (0.1, true), (1.0, true), (5.0, true)] {
                        let mut events = Vec::new();
                        animation.update(&sequences, delta, speed, enabled, |_, event| { events.push(match event { VertexAnimEvent::End => -1, VertexAnimEvent::Notify(i) => i as i32 }); true });
                        commands.push(serde_json::json!({"update":delta,"notifies":enabled,"speed":speed}));
                        expected.push((animation.clone(), events));
                    }
                }
            }
        }
    }
    let input = serde_json::json!({
        "sequences":sequences.iter().map(|s| serde_json::json!({"frames":s.frames,"rate":s.rate,"notifies":s.notifies.iter().map(|n| n.0).collect::<Vec<_>>()})).collect::<Vec<_>>(),
        "commands":commands
    }).to_string() + "\n";
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/vertex_animation_reference.py");
    let mut child = Command::new(std::env::var_os("KILLER_ORACLE_PYTHON").unwrap_or_else(|| "python3".into()))
        .arg(script).arg(root).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let native: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    let native = native.as_array().unwrap();
    assert_eq!(native.len(), expected.len());
    for (index, (native, (animation, events))) in native.iter().zip(expected).enumerate() {
        let values = native["state"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u32).collect::<Vec<_>>();
        assert_eq!(values[0], animation.sequence.map_or(0, |i| i as u32 + 1), "case {index}");
        for (field, value) in [animation.rate, animation.frame, animation.last, animation.tween_rate, animation.old_rate, animation.original_rate].into_iter().enumerate() {
            let original = f32::from_bits(values[field + 1]);
            assert!((value - original).abs() <= 0.00001, "case {index} field {field}: {value} != {original}; command={}", commands[index]);
        }
        assert_eq!(values[7], u32::from(animation.looping), "case {index}");
        assert_eq!(values[8], u32::from(animation.notify), "case {index}");
        assert_eq!(native["events"], serde_json::json!(events), "case {index}");
    }
    eprintln!("{} operações PlayAnim/UpdateAnimation comparadas com a DLL", commands.len());
}
