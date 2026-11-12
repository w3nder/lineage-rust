use killer_assets::collision::model::Model;
use serde_json::{Value, json};
use std::{io::Write, path::PathBuf, process::{Command, Stdio}};
use ue2_level::{BspNode, BspTree, BspRender};

#[test]
#[ignore]
fn model_box_check_matches_native_solid_leaf_hulls() {
    let planes = [[0.0,0.0,1.0,0.0], [0.0,0.0,-1.0,100.0], [1.0,0.0,0.0,10.0], [-1.0,0.0,0.0,10.0], [0.0,1.0,0.0,10.0], [0.0,-1.0,0.0,10.0]];
    let tree = BspTree { nodes: planes.into_iter().enumerate().map(|(i,plane)| BspNode { plane, front:-1, back:if i==5 {-1} else {i as i32+1}, flags:0,vertices:3, coplanar:-1,collision_bound:0,render_bound:-1,surface:0,zone_front:0,zone_back:0,leaf_front:-1,leaf_back:-1 }).collect() };
    let mut hulls = vec![0,1,2,3,4,5,-1];
    hulls.extend([-10.0_f32,-10.0,-100.0,10.0,10.0,0.0].map(|v|v.to_bits() as i32));
    let data = BspRender { leaf_hulls:hulls.clone(),root_outside:true,..Default::default() };
    let nodes:Vec<_> = tree.nodes.iter().map(|n|json!({"plane":n.plane,"back":n.back,"front":n.front,"flags":n.flags,"vertices":n.vertices,"bound":n.collision_bound})).collect();
    let mut cases=Vec::new();
    for x in [-20.0,-15.0,-10.0,-9.9,0.0,9.9,10.0,15.0,20.0] {
        for height in [-120.0,-100.0,-50.0,-0.2,0.0,0.2,10.0,100.0] {
            for extent in [0.2,5.0,10.0] {
                cases.push(json!({"nodes":nodes,"hulls":hulls,"outside":true,"start":[x,0.0,height],"end":[x,0.0,-150.0],"extent":[5.0,5.0,extent]}));
            }
        }
    }
    for x in [-20.0,-9.9,0.0,3.7,9.9,20.0] {
        for (start,end) in [(10.0,-150.0),(37.25,-3.5),(100.0,-150.0),(0.5,-0.5),(250.0,-400.0),(13.74,1.0)] {
            for extent in [[5.0,5.0,0.2],[5.0,5.0,1.0],[5.0,5.0,5.75],[5.0,5.0,13.74]] {
                cases.push(json!({"nodes":nodes,"hulls":hulls,"outside":true,"start":[x,0.31,start],"end":[x*0.5,-0.7,end],"extent":extent,"wrapper":true}));
            }
        }
    }
    for x in [-20.0,-10.001,-10.0,-9.999,0.0,9.999,10.0,10.001,20.0] {
        for (start,end) in [(100.0,-150.0),(-150.0,100.0),(-50.0,100.0),(-50.0,-50.0)] {
            for flags in [0,0x10,0x20] {
                cases.push(json!({"nodes":nodes,"hulls":hulls,"outside":true,"start":[x,0.0,start],"end":[x,0.0,end],"line_flags":flags}));
            }
        }
    }
    let script=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/model_collision_reference.py");
    let mut child=Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_|"python3".into())).arg(script).arg(std::env::var("KILLER_CLIENT_DIR").unwrap()).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    writeln!(child.stdin.take().unwrap(),"{}",json!(cases)).unwrap();
    let output=child.wait_with_output().unwrap();assert!(output.status.success());
    let expected:Vec<Value>=serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(expected.len(),cases.len());
    let mut wrapper_hits = 0;
    for (i,(case,expected)) in cases.iter().zip(expected).enumerate() {
        let model = Model { tree:&tree,data:&data,transform:None,unscaled:true };
        let start = serde_json::from_value(case["start"].clone()).unwrap();
        let end = serde_json::from_value(case["end"].clone()).unwrap();
        if case["wrapper"].as_bool() == Some(true) {
            let result = model.trace(start,end,serde_json::from_value(case["extent"].clone()).unwrap(),0);
            assert_eq!(result.is_some(),expected["hit"].as_bool().unwrap(),"{i}: {case} {expected} {result:?}");
            if let Some(hit) = result {
                wrapper_hits += 1;
                assert_eq!(hit.time.to_bits(),(expected["time"].as_f64().unwrap() as f32).to_bits(),"{i}: {hit:?} {expected}");
                let position:[f32;3]=serde_json::from_value(expected["position"].clone()).unwrap();
                assert_eq!(hit.position.map(f32::to_bits),position.map(f32::to_bits),"{i}: {hit:?} {expected}");
                let normal:[f32;3]=serde_json::from_value(expected["normal"].clone()).unwrap();
                assert_eq!(hit.normal.map(f32::to_bits),normal.map(f32::to_bits),"{i}: {hit:?} {expected}");
            }
            continue;
        }
        let result = if let Some(flags) = case["line_flags"].as_u64() { model.trace_line(start,end,flags as u32) }
            else { model.trace_raw(start,end,serde_json::from_value(case["extent"].clone()).unwrap()) };
        assert_eq!(result.is_some(),expected["hit"].as_bool().unwrap(),"{i}: {case} {expected}");
        if let Some(hit)=result {
            let time=expected["time"].as_f64().unwrap() as f32;
            assert!((hit.time-time).abs()<1e-6,"{i}: {hit:?} {expected}");
            let normal:[f32;3]=serde_json::from_value(expected["normal"].clone()).unwrap();
            for (a,b) in hit.normal.into_iter().zip(normal) { assert!((a-b).abs()<1e-6,"{i}: {hit:?} {expected}"); }
        }
    }
    assert!(wrapper_hits > 40,"{wrapper_hits}");
    eprintln!("{} traços BSP comparados com engine.dll, {wrapper_hits} acertos com extensão no LineCheck inteiro",cases.len());
}
