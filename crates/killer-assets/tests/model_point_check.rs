use killer_assets::collision::model::Model;
use serde_json::{Value, json};
use std::{io::Write, path::PathBuf, process::{Command, Stdio}};
use ue2_level::{BspNode, BspRender, BspTree};

fn node(plane: [f32; 4], front: i32, back: i32, bound: i32) -> BspNode {
    BspNode { plane, front, back, flags: 0, vertices: 3, coplanar: -1, collision_bound: bound, render_bound: -1, surface: 0, zone_front: 0, zone_back: 0, leaf_front: -1, leaf_back: -1 }
}

fn chain(offset: f32) -> Vec<BspNode> {
    [[0.0,0.0,1.0,0.0], [0.0,0.0,-1.0,100.0], [1.0,0.0,0.0,10.0+offset], [-1.0,0.0,0.0,10.0-offset], [0.0,1.0,0.0,10.0], [0.0,-1.0,0.0,10.0]]
        .into_iter().enumerate().map(|(i,plane)| node(plane, -1, if i == 5 { -1 } else { i as i32+1 }, 0)).collect()
}

fn hulls() -> Vec<i32> {
    let mut hulls = vec![0,1,2,3,4,5,-1];
    hulls.extend([-10.0_f32,-10.0,-100.0,10.0,10.0,0.0].map(|v| v.to_bits() as i32));
    hulls
}

fn skewed() -> Vec<BspNode> {
    let planes = [[0.6_f32,0.0,0.8,0.0], [-0.6,0.0,-0.8,50.0], [0.0,1.0,0.0,20.0], [0.0,-1.0,0.0,20.0], [0.8,0.0,-0.6,30.0], [-0.8,0.0,0.6,30.0]];
    planes.into_iter().enumerate().map(|(i,plane)| node(plane, -1, if i == 5 { -1 } else { i as i32+1 }, 0)).collect()
}

fn split() -> (Vec<BspNode>, Vec<i32>) {
    let mut nodes = vec![node([1.0,0.0,0.0,0.0], 1, -1, -1)];
    nodes.extend(chain(0.0).into_iter().map(|mut n| { n.front = -1; if n.back >= 0 { n.back += 1; } n }));
    nodes[1].collision_bound = 0;
    let hulls = hulls().into_iter().enumerate().map(|(i,v)| if i < 6 { v+1 } else { v }).collect();
    (nodes, hulls)
}

fn json_nodes(nodes: &[BspNode]) -> Vec<Value> {
    nodes.iter().map(|n| json!({"plane":n.plane,"back":n.back,"front":n.front,"flags":n.flags,"vertices":n.vertices,"bound":n.collision_bound})).collect()
}

#[test]
#[ignore]
fn model_point_check_matches_native_unmodel_pointcheck() {
    let (split_nodes, split_hulls) = split();
    let worlds: Vec<(Vec<BspNode>, Vec<i32>, bool)> = vec![
        (chain(0.0), hulls(), true), (chain(3.0), hulls(), true), (skewed(), hulls(), true), (chain(0.0), hulls(), false), (split_nodes, split_hulls, true),
    ];
    let mut cases = Vec::new();
    let mut owners = Vec::new();
    for (w,(nodes,hulls,outside)) in worlds.iter().enumerate() {
        let encoded = json_nodes(nodes);
        for x in [-30.0_f32,-12.0,-10.0,-9.5,-4.0,0.0,0.3,5.0,9.5,10.0,12.0,30.0] {
            for y in [-25.0_f32,-9.0,0.0,7.5,21.0] {
                for z in [-120.0_f32,-101.0,-99.0,-60.0,-20.0,-0.5,0.0,0.5,3.0,20.0] {
                    for extent in [[0.0,0.0,0.0],[0.5,0.5,0.5],[5.0,5.0,5.75],[13.0,2.0,30.0],[1.0,9.0,0.1]] {
                        for flags in [0_u32,0x10] {
                            owners.push(w);
                            cases.push(json!({"nodes":encoded,"hulls":hulls,"outside":outside,"point":[x,y,z],"extent":extent,"flags":flags}));
                        }
                    }
                }
            }
        }
    }
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/model_collision_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into())).arg(script).arg(std::env::var("KILLER_CLIENT_DIR").unwrap()).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", json!(cases)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let expected: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(expected.len(), cases.len());
    let (mut blocked, mut free) = (0, 0);
    for (i,(case,expected)) in cases.iter().zip(expected).enumerate() {
        let (nodes, hulls, outside) = &worlds[owners[i]];
        let tree = BspTree { nodes: nodes.clone() };
        let data = BspRender { leaf_hulls: hulls.clone(), root_outside: *outside, ..Default::default() };
        let model = Model { tree: &tree, data: &data, transform: None, unscaled: true };
        let point = serde_json::from_value(case["point"].clone()).unwrap();
        let extent = serde_json::from_value(case["extent"].clone()).unwrap();
        let (fits, hit) = model.point_check(point, extent, case["flags"].as_u64().unwrap() as u32);
        assert_eq!(fits, expected["fits"].as_bool().unwrap(), "{i}: {case} {expected} {hit:?}");
        let location: [f32;3] = serde_json::from_value(expected["location"].clone()).unwrap();
        let normal: [f32;3] = serde_json::from_value(expected["normal"].clone()).unwrap();
        assert_eq!(hit.location.map(f32::to_bits), location.map(f32::to_bits), "{i}: {case} {expected} {hit:?}");
        assert_eq!(hit.normal.map(f32::to_bits), normal.map(f32::to_bits), "{i}: {case} {expected} {hit:?}");
        assert_eq!(hit.item, expected["item"].as_i64().unwrap() as i32, "{i}: {case} {expected} {hit:?}");
        if fits { free += 1 } else { blocked += 1 }
    }
    assert!(blocked > 100 && free > 100, "{blocked} {free}");
}
