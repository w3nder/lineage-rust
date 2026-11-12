use glam::Vec3;
use killer_assets::pickup::{Shape,TraceHit,drop::{self,Request}};
use serde_json::{Value,json};
use std::{io::Write,path::PathBuf,process::{Command,Stdio}};

#[test]
#[ignore]
fn every_drop_plan_branch_matches_original_spawn_and_mover_configuration() {
    let mut cases = Vec::new();
    for style in 0..=5 {
        for kind in 0..=5 {
            for index in 0..3 {
                for height in [0.2,13.74] {
                    for (position,normal,time) in [([1.25,-2.75,3.0],[0.0,0.0,1.0],0.0),([83400.0,148000.0,-3400.0],[0.3,0.4,0.866],0.02),([83400.25,148000.75,-3400.5],[-0.4,0.3,0.866],0.9),([83400.5,148000.25,-3400.75],[0.0,0.0,1.0],0.158_333_33),([-120.5,77.25,1.0e-3],[0.0,0.6,0.8],1.0e-7)] {
                        for source in [None,Some([83310.25,148010.75,-3380.5])] {
                            for non_level_actor in [false,true] {
                                cases.push(json!({"style":style,"kind":kind,"index":index,"height":height,"radius":5.0,"position":position,"normal":normal,"time":time,"source":source,"rotation":[100,16384,300],"non_level_actor":non_level_actor}));
                            }
                        }
                    }
                }
            }
        }
    }
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/pickup_plan_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script).arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    writeln!(child.stdin.take().unwrap(),"{}",json!(cases)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let expected: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(expected.len(),cases.len());
    for (index,(case,native)) in cases.iter().zip(expected).enumerate() {
        let vector = |key: &str| Vec3::from(serde_json::from_value::<[f32;3]>(case[key].clone()).unwrap());
        let request = Request { shape:Shape { radius:5.0,height:case["height"].as_f64().unwrap() as f32 },style:case["style"].as_u64().unwrap() as u32,kind:case["kind"].as_u64().unwrap() as u32,index:case["index"].as_u64().unwrap() as usize,
            packet_rotation:serde_json::from_value(case["rotation"].clone()).unwrap(),source:serde_json::from_value::<Option<[f32;3]>>(case["source"].clone()).unwrap().map(Vec3::from) };
        let plan = drop::plan(request,TraceHit { position:vector("position"),normal:vector("normal"),time:case["time"].as_f64().unwrap() as f32,non_level_actor:case["non_level_actor"].as_bool().unwrap() }).unwrap();
        let compare = |label: &str,actual: Vec3,expected: &Value| {
            let expected: [f32;3] = serde_json::from_value(expected.clone()).unwrap();
            assert_eq!(actual.to_array().map(f32::to_bits),expected.map(f32::to_bits),"case {index} {label}: {actual:?} vs {expected:?}; {case}");
        };
        compare("position",plan.placement.position,&native["position"]);
        compare("effect",plan.effect_position,&native["effect"]);
        for (label,actual) in [("rotation",plan.placement.rotation),("target",plan.spin.target),("rate",plan.spin.rate)] {
            assert_eq!(actual,serde_json::from_value::<[i32;3]>(native[label].clone()).unwrap(),"case {index} {label}: {case}");
        }
        let movers = native["movers"].as_array().unwrap();
        if let Some(trajectory) = plan.trajectory {
            assert_eq!(movers.len(),if trajectory.intermediate.is_some() { 2 } else { 1 });
            compare("destination",trajectory.destination,&movers[0]["destination"]);
            compare("initial",trajectory.initial,&movers.last().unwrap()["initial"]);
            if let Some(intermediate) = trajectory.intermediate { compare("intermediate",intermediate,&movers[0]["initial"]); }
            for mover in movers {
                assert_eq!(trajectory.speed.to_bits(),(mover["speed"].as_f64().unwrap() as f32).to_bits());
                assert_eq!(trajectory.acceleration.to_bits(),(mover["acceleration"].as_f64().unwrap() as f32).to_bits());
            }
        } else { assert!(movers.is_empty()); }
    }
    eprintln!("{} planos OnDropItem comparados com engine.dll",cases.len());
}
