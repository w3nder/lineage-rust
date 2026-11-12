use killer_assets::collision::transform;
use serde_json::json;
use std::{io::Write,path::PathBuf,process::{Command,Stdio}};
use ue2_level::Placement;

#[test]
#[ignore]
fn collision_actor_matrix_preserves_native_trigonometry_and_pivot_rounding() {
    let mut cases = Vec::new();
    for location in [[0.0;3],[83400.25,148600.75,-3400.0]] {
        for rotation in [[0;3],[12345,-23001,71500],[16383,16385,-4000],[65536,131072,-65536],[-2147483648,2147483647,65539]] {
            for scale in [[1.0;3],[0.7,2.35,-1.25],[-3.7,0.15,4.2]] {
                for pivot in [[0.0;3],[13.74,-7.29,192.18]] {
                    cases.push(json!({"location":location,"rotation":rotation,"scale":scale,"pivot":pivot}));
                }
            }
        }
    }
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/collision_transform_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script).arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    writeln!(child.stdin.take().unwrap(),"{}",json!(cases)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let expected: Vec<[f32;16]> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(expected.len(),cases.len());
    for (index,(case,native)) in cases.iter().zip(expected).enumerate() {
        let vector = |name: &str| serde_json::from_value::<[f32;3]>(case[name].clone()).unwrap();
        let rotation = serde_json::from_value(case["rotation"].clone()).unwrap();
        let actual = transform::actor(Placement { location:vector("location"),rotation },vector("scale"),vector("pivot")).to_cols_array();
        for (entry,(actual,native)) in actual.into_iter().zip(native).enumerate() {
            assert_eq!(actual.to_bits(),native.to_bits(),"case {index}, entry {entry}, {actual} != {native}, {case}");
        }
    }
    eprintln!("{} matrizes comparadas com engine.dll",cases.len());
}
