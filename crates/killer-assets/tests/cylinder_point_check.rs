use killer_assets::collision::{cylinder, point_check::PointHit};
use serde_json::{Value, json};
use std::{io::Write, path::PathBuf, process::{Command, Stdio}};

#[test]
#[ignore]
fn cylinder_point_check_matches_native_uprimitive_pointcheck() {
    let mut cases = Vec::new();
    for center in [[0.0_f32,0.0,0.0],[137.5,-42.25,311.0]] {
        for (radius,height) in [(20.0_f32,40.0_f32),(7.5,90.0),(35.0,12.5)] {
            for dx in [-60.0_f32,-26.0,-20.0,-12.0,0.0,0.5,9.0,20.0,26.0,41.0] {
                for dy in [-45.0_f32,-8.0,0.0,13.0,33.0] {
                    for dz in [-140.0_f32,-91.0,-52.0,-40.0,-10.0,0.0,5.0,38.0,40.0,44.0,90.0,100.0] {
                        for extent in [[0.0_f32,0.0,0.0],[6.0,6.0,10.0],[12.0,3.0,2.0],[0.25,0.25,30.0]] {
                            cases.push(json!({"center":center,"radius":radius,"height":height,"point":[center[0]+dx,center[1]+dy,center[2]+dz],"extent":extent}));
                        }
                    }
                }
            }
        }
    }
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/static_point_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into())).arg(script).arg(std::env::var("KILLER_CLIENT_DIR").unwrap()).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", json!(cases)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let expected: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(expected.len(), cases.len());
    let (mut blocked, mut free) = (0, 0);
    for (i,(case,expected)) in cases.iter().zip(expected).enumerate() {
        let center = serde_json::from_value(case["center"].clone()).unwrap();
        let point = serde_json::from_value(case["point"].clone()).unwrap();
        let extent = serde_json::from_value(case["extent"].clone()).unwrap();
        let mut hit = PointHit { location: [0.0;3], normal: [0.0;3], item: 0 };
        let encroaches = cylinder::point_check(center, case["radius"].as_f64().unwrap() as f32, case["height"].as_f64().unwrap() as f32, point, extent, &mut hit);
        assert_eq!(!encroaches, expected["fits"].as_bool().unwrap(), "{i}: {case} {expected} {hit:?}");
        let location: [f32;3] = serde_json::from_value(expected["location"].clone()).unwrap();
        let normal: [f32;3] = serde_json::from_value(expected["normal"].clone()).unwrap();
        assert_eq!(hit.location.map(f32::to_bits), location.map(f32::to_bits), "{i}: {case} {expected} {hit:?}");
        assert_eq!(hit.normal.map(f32::to_bits), normal.map(f32::to_bits), "{i}: {case} {expected} {hit:?}");
        if encroaches { blocked += 1 } else { free += 1 }
    }
    assert!(blocked > 500 && free > 500, "{blocked} {free}");
}
