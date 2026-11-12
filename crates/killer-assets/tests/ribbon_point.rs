use killer_assets::ribbon_point::{Actor, Bone, Emitter, Host, Warning, get_new_ribbon_point};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

const START_NAME: u32 = 77;
const END_NAME: u32 = 78;

fn coords(index: usize) -> [f32; 12] {
    let i = index as f32;
    [110.5 + i * 3.25, -190.25 + i, 320.75 - i * 7.5, 0.6, 0.8, 0.0, -0.8, 0.6, 0.0, 0.0, 0.0, 1.0]
}

fn rotation(index: usize) -> [i32; 3] {
    [index as i32 * 1700 - 2000, 16384 - index as i32 * 4100, index as i32 * 900]
}

#[test]
#[ignore]
fn get_new_ribbon_point_10655150_matches_native_for_every_branch() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/ribbon_point_reference.py");
    let python = std::env::var_os("KILLER_ORACLE_PYTHON").unwrap_or_else(|| "python3".into());
    let mut child = Command::new(python).arg(script).arg(&root).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut output = BufReader::new(child.stdout.take().unwrap());
    let mut cases = 0;
    let rotations228 = [[0, 0, 0], [1000, -3000, 500], [65536, 0, -65536]];
    let bone_sets: [(Option<usize>, Option<usize>); 4] = [(Some(2), Some(4)), (None, Some(4)), (Some(3), None), (Some(1), Some(1))];
    let presence = [(true, true, true, true, true), (false, true, true, true, true), (true, false, true, true, true), (true, true, false, true, true), (true, true, true, false, true), (true, true, true, true, false)];
    for coordinate_system in 0..=4u8 {
        for axis in 0..=5u8 {
            for flags in [0x5fu32, 0x1f, 0xdf] {
                for origin_offset in [0.0f32, 0.5, 1.0] {
                    for rotation228 in rotations228 {
                        for (start, end) in bone_sets {
                            for (host, owner, mesh, instance, named) in presence {
                                if (start, end) != bone_sets[0] && (host, owner, mesh, instance, named) != presence[0] {
                                    continue;
                                }
                                if origin_offset != 0.5 && (axis != 3 || coordinate_system != 1) {
                                    continue;
                                }
                                let width = 21.2f32;
                                let offsets = [[5.3f32, 1.25, -0.5], [10.6f32, -2.0, 0.75]];
                                let host_actor = Actor { location: [1.0, 2.0, 3.0], rotation: [100, 2000, -300] };
                                let owner_actor = Actor { location: [100.0, 200.0, 300.0], rotation: [-1200, 16384 + 333, 700] };
                                let bones: serde_json::Map<String, Value> = [(START_NAME, start), (END_NAME, end)]
                                    .into_iter()
                                    .filter_map(|(name, bone)| Some((name.to_string(), json!(bone?))))
                                    .collect();
                                let request = json!({
                                    "coordinate_system": coordinate_system, "rot228": rotation228, "host": host, "width": width, "axis": axis,
                                    "origin_offset": origin_offset, "offsets": offsets, "names": [if named { START_NAME } else { 0 }, END_NAME],
                                    "flags": flags, "owner": owner, "host_location": host_actor.location, "host_rotation": host_actor.rotation,
                                    "mesh": mesh, "owner_location": owner_actor.location, "owner_rotation": owner_actor.rotation, "instance": instance,
                                    "bones": bones, "coords": (0..6).map(coords).collect::<Vec<_>>(), "rotations": (0..6).map(rotation).collect::<Vec<_>>(),
                                });
                                writeln!(input, "{request}").unwrap();
                                input.flush().unwrap();
                                let mut line = String::new();
                                output.read_line(&mut line).unwrap();
                                let native: Value = serde_json::from_str(&line).unwrap();
                                let bone = |index: Option<usize>| index.map(|i| Bone { origin: [coords(i)[0], coords(i)[1], coords(i)[2]], rotation: rotation(i) });
                                let mut emitter = Emitter {
                                    coordinate_system,
                                    axis,
                                    flags,
                                    width,
                                    origin_offset,
                                    start_offset: offsets[0],
                                    end_offset: offsets[1],
                                    rotation: rotation228,
                                    has_start_name: named,
                                };
                                let host_value = Host { actor: host_actor, owner: owner.then_some(owner_actor), owner_mesh: mesh };
                                let mut warnings = Vec::new();
                                let sample = get_new_ribbon_point(&mut emitter, host.then_some(&host_value), instance, bone(start), bone(end), &mut warnings, width);
                                let context = format!("{request}");
                                let bits = |v: [f32; 3]| v.map(f32::to_bits).to_vec();
                                let expected = |key: &str| native[key].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u32).collect::<Vec<_>>();
                                assert_eq!(bits(sample.a), expected("a"), "a {context} {sample:?}");
                                assert_eq!(bits(sample.b), expected("b"), "b {context} {sample:?}");
                                assert_eq!(sample.width.to_bits(), native["width"].as_u64().unwrap() as u32, "width {context}");
                                assert_eq!(u64::from(emitter.axis), native["axis"].as_u64().unwrap(), "axis {context}");
                                assert_eq!(u64::from(emitter.flags), native["flags"].as_u64().unwrap(), "flags {context}");
                                let logged: Vec<Warning> = native["log"].as_array().unwrap().iter().map(|name| if name.as_u64() == Some(u64::from(START_NAME)) { Warning::MissingStartBone } else { Warning::MissingEndBone }).collect();
                                assert_eq!(warnings, logged, "log {context}");
                                cases += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    drop(input);
    assert!(child.wait().unwrap().success());
    eprintln!("{cases} casos");
    assert!(cases > 500);
}
