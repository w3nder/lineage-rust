use killer_assets::collision::cylinder;
use serde_json::{Value,json};
use std::{io::Write,path::PathBuf,process::{Command,Stdio}};

#[test]
#[ignore]
fn cylinder_caps_sides_and_interior_match_original_primitive() {
    let mut cases = Vec::new();
    for start in [[0.0,0.0,100.0],[0.0,0.0,0.0],[2.0,3.0,0.0],[30.0,0.0,0.0],[22.0,0.0,0.0],[30.0,40.0,100.0],[-50.0,-50.0,-100.0]] {
        for end in [[0.0,0.0,-100.0],[0.0,0.0,0.0],[2.0,3.0,0.0],[100.0,0.0,0.0],[22.0,0.0,100.0],[30.0,40.0,0.0],[-50.0,-50.0,100.0]] {
            for extent in [[0.0;3],[5.0,5.0,10.0],[5.0,20.0,0.2],[1.0;3]] {
                for radius in [0.0,1.0,22.0] {
                    for center in [[0.0_f32;3],[83400.125,148000.75,-3431.5],[-100123.25,72345.125,8123.5]] {
                        let shifted = |point: [f32;3]| std::array::from_fn::<_,3,_>(|axis| point[axis]+center[axis]);
                        cases.push(json!({"center":center,"radius":radius,"height":22.0,"start":shifted(start),"end":shifted(end),"extent":extent}));
                    }
                }
            }
        }
    }
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/cylinder_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script).arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    writeln!(child.stdin.take().unwrap(),"{}",json!(cases)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let expected: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(expected.len(),cases.len());
    for (index,(case,native)) in cases.iter().zip(expected).enumerate() {
        let vector = |name: &str| serde_json::from_value(case[name].clone()).unwrap();
        let hit = cylinder::trace(vector("center"),case["radius"].as_f64().unwrap() as f32,22.0,vector("start"),vector("end"),vector("extent"));
        assert_eq!(hit.is_some(),!native.is_null(),"case {index}: {case}, {hit:?}, {native}");
        if let Some(hit) = hit {
            let position: [f32;3] = serde_json::from_value(native["position"].clone()).unwrap();
            let normal: [f32;3] = serde_json::from_value(native["normal"].clone()).unwrap();
            assert_eq!(hit.time.to_bits(),(native["time"].as_f64().unwrap() as f32).to_bits(),"time case {index}: {case}, {hit:?}, {native}");
            assert_eq!(hit.position.map(f32::to_bits),position.map(f32::to_bits),"position case {index}: {case}");
            assert_eq!(hit.normal.map(f32::to_bits),normal.map(f32::to_bits),"normal case {index}: {case}, {hit:?}, {native}");
        }
    }
    eprintln!("{} colisões de cilindro comparadas com engine.dll",cases.len());
}
