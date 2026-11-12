use glam::Vec3;
use killer_assets::{characters::mesh_local,collision::transform};
use serde_json::json;
use std::{io::Write,path::PathBuf,process::{Command,Stdio}};
use ue2_level::Placement;

#[test]
#[ignore]
fn pickup_world_matrix_matches_native_mesh_to_world() {
    let mut cases = Vec::new();
    for (origin,rot_origin,scale) in [([0.0,0.0,0.0],[0,0,0],[1.0,1.0,1.0]),([0.0,0.0,5.5],[0,49152,0],[1.0,1.0,1.0]),([0.0,0.0,0.7],[0,49152,0],[3.0,3.0,3.0]),([1.25,-2.0,6.0],[1200,-7000,300],[1.0,1.5,0.75])] {
        for rotation in [[0,0,0],[46422,32768,16384],[51782,45322,16384],[13653,32768,16384],[0,36909,0],[4000,0,0],[0,0,4000],[-32768,1000,16384]] {
            for location in [[0.0,0.0,0.0],[83400.0,148065.0,-3427.0]] {
                cases.push(json!({"mesh_scale":scale,"origin":origin,"rot_origin":rot_origin,"location":location,"rotation":rotation,"draw_scale":1.0,"draw_scale_3d":[1.0,1.0,1.0],"pre_pivot":[0.0,0.0,0.0],"factor":1.0}));
            }
        }
    }
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/pickup_mesh_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script).arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    writeln!(child.stdin.take().unwrap(),"{}",json!(cases)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let expected: Vec<[f32;16]> = serde_json::from_slice(&output.stdout).unwrap();
    let mut worst = 0.0_f32;
    for (index,(case,native)) in cases.iter().zip(expected).enumerate() {
        let v3 = |key: &str| serde_json::from_value::<[f32;3]>(case[key].clone()).unwrap();
        let i3 = |key: &str| serde_json::from_value::<[i32;3]>(case[key].clone()).unwrap();
        let world = transform::actor(Placement { location:v3("location"),rotation:i3("rotation") },[1.0;3],[0.0;3])*mesh_local(v3("origin"),i3("rot_origin"),v3("mesh_scale"));
        for point in [Vec3::ZERO,Vec3::X*30.0,Vec3::Y*7.5,Vec3::Z*-5.0,Vec3::new(-17.1,5.3,0.6)] {
            let ours = world.transform_point3(point);
            let theirs = Vec3::new(point.x*native[0]+point.y*native[4]+point.z*native[8]+native[12],
                point.x*native[1]+point.y*native[5]+point.z*native[9]+native[13],
                point.x*native[2]+point.y*native[6]+point.z*native[10]+native[14]);
            let error = (ours-theirs).abs().max_element();
            worst = worst.max(error);
            assert!(error < 1e-2,"caso {index} ponto {point:?}: nosso {ours:?} nativo {theirs:?}; {case}");
        }
    }
    eprintln!("{} matrizes MeshToWorld comparadas, maior diferença {worst}",cases.len());
}
