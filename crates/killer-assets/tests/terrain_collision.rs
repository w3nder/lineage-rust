use killer_assets::collision::terrain::Quad;
use serde_json::{Value, json};
use std::{io::Write, path::PathBuf, process::{Command, Stdio}};

#[test]
#[ignore]
fn terrain_quad_matches_original_for_diagonals_holes_inversion_and_extents() {
    let mut cases = Vec::new();
    for heights in [[0.0;4],[0.0,10.0,25.0,-20.0]] {
        let vertices = [[0.0,0.0,heights[0]],[100.0,0.0,heights[1]],[100.0,100.0,heights[2]],[0.0,100.0,heights[3]]];
        for turned in [false,true] {
            for inverted in [false,true] {
                for (visible,deleted,skip_visibility) in [(true,false,false),(false,false,false),(false,false,true),(true,true,true)] {
                    for x in [-0.1,0.0,0.01,30.0,50.0,100.0,100.1] {
                        for y in [0.0,50.0,100.0] {
                            for extent in [[0.2;3],[5.0,100.0,20.0]] {
                                let (start,end) = if inverted { (-100.0,100.0) } else { (100.0,-100.0) };
                                cases.push(json!({"vertices":vertices,"turned":turned,"inverted":inverted,"visible":visible,"deleted":deleted,
                                    "skip_visibility":skip_visibility,"start":[x,y,start],"end":[x,y,end],"extent":extent}));
                            }
                        }
                    }
                }
            }
        }
    }
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/terrain_quad_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script).arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", json!(cases)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let expected: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(expected.len(),cases.len());
    for (index,(case,native)) in cases.iter().zip(expected).enumerate() {
        let quad = Quad { vertices:serde_json::from_value(case["vertices"].clone()).unwrap(),
            turned:case["turned"].as_bool().unwrap(),inverted:case["inverted"].as_bool().unwrap(),
            visible:case["visible"].as_bool().unwrap(),deleted:case["deleted"].as_bool().unwrap() };
        let mut hit = None;
        let accepted = quad.trace(serde_json::from_value(case["start"].clone()).unwrap(),
            serde_json::from_value(case["end"].clone()).unwrap(),serde_json::from_value(case["extent"].clone()).unwrap(),
            case["skip_visibility"].as_bool().unwrap(),&mut hit);
        assert_eq!(!accepted,native.is_null(),"case {index}: {case}, {hit:?}, {native}");
        if let Some(hit) = hit {
            assert!((hit.time-native["time"].as_f64().unwrap() as f32).abs()<1e-6,"case {index}: {hit:?}, {native}");
            for (actual,key) in [(hit.normal,"normal"),(hit.position,"position")] {
                let expected: [f32;3] = serde_json::from_value(native[key].clone()).unwrap();
                for (a,b) in actual.into_iter().zip(expected) { assert!((a-b).abs()<0.0001,"case {index}, {key}: {hit:?}, {native}"); }
            }
        }
    }
    eprintln!("{} colisões de quad comparadas com engine.dll",cases.len());
}
