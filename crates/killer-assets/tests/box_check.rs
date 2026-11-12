use killer_assets::collision::BoxCheck;
use serde_json::{Value, json};
use std::{io::Write, path::PathBuf, process::{Command, Stdio}};

#[test]
#[ignore]
fn box_check_matches_original_state_and_short_circuit_for_planes_and_bounds() {
    let mut cases = Vec::new();
    for x in [-5000.0, -20.0, -1.0, -0.00002, 0.0, 0.00002, 1.0, 20.0, 5000.0] {
        for end in [-20.0, 0.0, 20.0] {
            for extent in [0.0, 0.2, 5.0, 50.0] {
                for limit in [0.0, 0.25, 1.0] {
                    cases.push(json!({"start":[x,2.0,-3.0],"end":[end,-4.0,8.0],"extent":[extent,extent,extent],"limit":limit,
                        "operations":[{"plane":[1.0,0.0,0.0,0.0]}, {"plane":[0.3,-0.7,0.2,0.0]},
                        {"box":[[-10.0,-10.0,-10.0],[10.0,10.0,10.0]]}]}));
                }
            }
        }
    }
    for axis in [[0.0;3], [1.0,0.0,0.0], [0.0,1.0,0.0], [0.0,0.0,1.0], [0.3,-0.7,0.2]] {
        for edge in [[0.0;3], axis, [1.0,2.0,3.0]] {
            for orientation in [[1.0,1.0,1.0], [-1.0,-1.0,-1.0]] {
                cases.push(json!({"start":[20.0,0.0,0.0],"end":[-20.0,0.0,0.0],"extent":[5.0,5.0,5.0],"limit":1.0,
                    "operations":[{"edge":[[0.0,0.0,0.0],edge,orientation,axis]}]}));
            }
        }
    }
    let mut seed = 19432_u32;
    let mut number = || { seed = seed.wrapping_mul(1664525).wrapping_add(1013904223); (seed as i32) as f32 / 1048576.0 };
    for _ in 0..2048 {
        let start = [number(), number(), number()];
        let end = [number(), number(), number()];
        let plane = [number()/2048.0, number()/2048.0, number()/2048.0, number()];
        cases.push(json!({"start":start,"end":end,"extent":[5.0,5.0,20.0],"limit":1.0,
            "operations":[{"edge":[start,end,[1.0,0.0,0.0],[0.0,1.0,0.0]]},{"plane":plane},{"box":[[-100.0,-200.0,-300.0],[100.0,200.0,300.0]]}]}));
    }
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/box_check_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script).arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", json!(cases)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let expected: Vec<Vec<Value>> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(expected.len(), cases.len());
    for (index, (case, expected)) in cases.iter().zip(expected).enumerate() {
        let mut state = BoxCheck::new(serde_json::from_value(case["start"].clone()).unwrap(),
            serde_json::from_value(case["end"].clone()).unwrap(), serde_json::from_value(case["extent"].clone()).unwrap(),
            case["limit"].as_f64().unwrap() as f32);
        for (operation, native) in case["operations"].as_array().unwrap().iter().zip(expected) {
            let accept = if operation.get("plane").is_some() {
                state.clip_plane(serde_json::from_value(operation["plane"].clone()).unwrap())
            } else if operation.get("edge").is_some() {
                let [point, edge, orientation, axis]: [[f32; 3]; 4] = serde_json::from_value(operation["edge"].clone()).unwrap();
                state.clip_edge(point, edge, orientation, axis)
            } else { state.clip_box(serde_json::from_value(operation["box"].clone()).unwrap()) };
            assert_eq!(accept, native["accept"].as_bool().unwrap(), "case {index}: {case}");
            assert_eq!(state.contact, native["contact"].as_bool().unwrap(), "case {index}");
            let normal: [f32; 3] = serde_json::from_value(native["normal"].clone()).unwrap();
            assert_eq!(state.normal.map(f32::to_bits), normal.map(f32::to_bits), "case {index}");
            for (value, key) in [(state.enter, "enter"), (state.exit, "exit")] {
                let native = native[key].as_f64().unwrap() as f32;
                assert_eq!(value.to_bits(), native.to_bits(), "case {index}, {key}: {case}");
            }
        }
    }
    eprintln!("{} estados de BoxCheck comparados com engine.dll", cases.len());
}
