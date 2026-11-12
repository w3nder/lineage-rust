use killer_assets::collision::static_mesh::line_box_exact;
use serde_json::{Value, json};
use std::{io::Write, path::PathBuf, process::{Command, Stdio}};

#[test]
#[ignore]
fn static_line_box_matches_native_for_inside_outside_parallel_and_edges() {
    let mut cases = Vec::new();
    for start in [-100.0,-10.1,-10.0,0.0,10.0,10.1,100.0] {
        for delta in [-200.0,-10.0,0.0,1e-7,10.0,200.0] {
            for axis in 0..3 {
                let mut point = [0.0_f32;3];
                point[axis] = start;
                let mut direction = [0.0_f32;3];
                direction[axis] = delta;
                cases.push(json!({"bounds":[[-10.0,-10.0,-10.0],[10.0,10.0,10.0]],"start":point,"delta":direction}));
            }
        }
    }
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/static_triangle_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script).arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", json!(cases)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let expected: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(cases.len(), expected.len());
    for (index, (case, native)) in cases.iter().zip(expected).enumerate() {
        let delta: [f32;3] = serde_json::from_value(case["delta"].clone()).unwrap();
        let inverse = delta.map(|value| (1.0/f64::from(value)) as f32);
        let actual = line_box_exact(serde_json::from_value(case["bounds"].clone()).unwrap(),
            serde_json::from_value(case["start"].clone()).unwrap(), delta, inverse);
        assert_eq!(actual, native.as_bool().unwrap(), "case {index}: {case}");
    }
    eprintln!("{} limites comparados com engine.dll", cases.len());
}
