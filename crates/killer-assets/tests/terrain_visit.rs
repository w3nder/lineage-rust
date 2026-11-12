use killer_assets::collision::terrain::QuadTrace;
use serde_json::{Value, json};
use std::{io::Write, path::PathBuf, process::{Command, Stdio}};
use ue2_level::Coords;

#[test]
#[ignore]
fn terrain_grid_visit_matches_native_order_and_contact_stop() {
    let coords = [[0.0;3],[1.0,0.0,0.0],[0.0,1.0,0.0],[0.0,0.0,1.0]];
    let mut cases = Vec::new();
    for start in [[-1.0,3.0,100.0],[5.2,5.7,100.0],[20.8,20.1,100.0]] {
        for end in [[-1.0,20.0,0.0],[5.2,5.7,0.0],[20.8,20.1,0.0],[5.2,20.0,0.0],[40.0,40.0,0.0]] {
            for radius in [0.2,1.0,5.0] {
                for contact in [[-1,-1],[5,5],[10,10],[20,20]] {
                    for alternate in [None,Some([[1.0,2.0,0.0],[0.5,0.0,0.0],[0.0,0.5,0.0],[0.0,0.0,1.0]])] {
                        cases.push(json!({"coords":coords,"alternate":alternate,"size":[32,32],"start":start,"end":end,"extent":[radius,radius,1.0],"contact":contact}));
                    }
                }
            }
        }
    }
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/terrain_visit_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script).arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", json!(cases)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let expected: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(expected.len(),cases.len());
    let coords = |data: Value| {
        let [origin,x_axis,y_axis,z_axis]: [[f32;3];4] = serde_json::from_value(data).unwrap();
        Coords { origin,x_axis,y_axis,z_axis }
    };
    for (index,(case,native)) in cases.iter().zip(expected).enumerate() {
        let trace = QuadTrace { start:serde_json::from_value(case["start"].clone()).unwrap(),
            end:serde_json::from_value(case["end"].clone()).unwrap(),extent:serde_json::from_value(case["extent"].clone()).unwrap(),
            size:[32,32],to_heightmap:coords(case["coords"].clone()),alternate:(!case["alternate"].is_null()).then(|| coords(case["alternate"].clone())) };
        let contact: [i32;2] = serde_json::from_value(case["contact"].clone()).unwrap();
        let mut visited = Vec::new();
        let hit = trace.visit(|x,y| { visited.push([x,y]); [x as i32,y as i32] == contact });
        assert_eq!(hit,native["hit"].as_bool().unwrap(),"case {index}: {case}");
        let expected: Vec<[usize;2]> = serde_json::from_value(native["visited"].clone()).unwrap();
        assert_eq!(visited,expected,"case {index}: {case}");
    }
    eprintln!("{} travessias de terreno comparadas com engine.dll",cases.len());
}
