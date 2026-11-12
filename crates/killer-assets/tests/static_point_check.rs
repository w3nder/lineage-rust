use glam::Mat4;
use killer_assets::collision::{point_check::PointHit, static_mesh::Mesh};
use serde_json::{Value, json};
use std::{io::Write, path::PathBuf, process::{Command, Stdio}};
use ue2_assets::{CollisionNode, CollisionTriangle};

fn cross(a: [f32;3], b: [f32;3]) -> [f32;3] {
    [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]
}

fn solid(vertices: &[[f32;3]], faces: &[[i32;3]]) -> (Vec<CollisionTriangle>, Vec<CollisionNode>) {
    let triangles: Vec<CollisionTriangle> = faces.iter().map(|face| {
        let [a,b,c] = face.map(|i| vertices[i as usize]);
        let u = [b[0]-a[0], b[1]-a[1], b[2]-a[2]];
        let v = [c[0]-a[0], c[1]-a[1], c[2]-a[2]];
        let n = cross(u, v);
        let length = (n[0]*n[0]+n[1]*n[1]+n[2]*n[2]).sqrt();
        let n = n.map(|x| x/length);
        let w = a[0]*n[0]+a[1]*n[1]+a[2]*n[2];
        CollisionTriangle { planes: [[n[0],n[1],n[2],w],[0.0;4],[0.0;4],[0.0;4]], indices: [face[0], face[1], face[2], 0] }
    }).collect();
    let nodes = faces.iter().enumerate().map(|(i,face)| {
        let points = face.map(|k| vertices[k as usize]);
        let min = [0,1,2].map(|k| points.iter().map(|p| p[k]).fold(f32::INFINITY, f32::min));
        let max = [0,1,2].map(|k| points.iter().map(|p| p[k]).fold(f32::NEG_INFINITY, f32::max));
        let child = |c: usize| if c < faces.len() { c as i32 } else { -1 };
        CollisionNode { indices: [i as i32, -1, child(2*i+1), child(2*i+2)], bounds: [min, max], valid: 1 }
    }).collect();
    (triangles, nodes)
}

fn cube() -> (Vec<[f32;3]>, Vec<[i32;3]>) {
    let vertices = vec![[0.,0.,0.],[40.,0.,0.],[40.,30.,0.],[0.,30.,0.],[0.,0.,20.],[40.,0.,20.],[40.,30.,20.],[0.,30.,20.]];
    let faces = vec![[0,2,1],[0,3,2],[4,5,6],[4,6,7],[0,1,5],[0,5,4],[1,2,6],[1,6,5],[2,3,7],[2,7,6],[3,0,4],[3,4,7]];
    (vertices, faces)
}

fn slab() -> (Vec<[f32;3]>, Vec<[i32;3]>) {
    (vec![[-60.,-60.,0.],[60.,-60.,0.],[60.,60.,0.],[-60.,60.,0.],[0.,0.,35.]], vec![[0,2,1],[0,3,2],[0,1,4],[1,2,4],[2,3,4],[3,0,4]])
}

fn matrix(angle: f32, scale: [f32;3], shift: [f32;3]) -> [[f32;4];4] {
    let (s, c) = angle.sin_cos();
    [[c*scale[0], s*scale[0], 0.0, 0.0], [-s*scale[1], c*scale[1], 0.0, 0.0], [0.0, 0.0, scale[2], 0.0], [shift[0], shift[1], shift[2], 1.0]]
}

#[test]
#[ignore]
fn static_mesh_point_check_matches_native_ustaticmesh_pointcheck() {
    let transforms = [
        matrix(0.0,[1.0;3],[0.0;3]), matrix(0.6,[1.0;3],[10.0,-5.0,3.0]), matrix(2.1,[1.5,1.5,1.5],[-20.0,8.0,0.0]),
        matrix(0.3,[-1.0,1.0,1.0],[5.0,5.0,-2.0]), matrix(1.0,[0.7,1.3,0.9],[0.0,0.0,0.0]),
    ];
    let shapes = [cube(), slab()];
    let mut worlds = Vec::new();
    let mut cases = Vec::new();
    let mut owner = Vec::new();
    for (s,(vertices,faces)) in shapes.iter().enumerate() {
        let (triangles, nodes) = solid(vertices, faces);
        for (t,rows) in transforms.iter().enumerate() {
            let to_world = Mat4::from_cols_array_2d(rows);
            let to_local = to_world.inverse();
            let local_rows = to_local.to_cols_array_2d();
            let world = worlds.len();
            worlds.push((vertices.clone(), triangles.clone(), nodes.clone(), to_world, to_local));
            for x in [-70.0_f32,-25.0,-5.0,0.0,12.0,20.0,38.0,47.0,70.0] {
                for y in [-65.0_f32,-12.0,0.0,14.0,29.0,42.0] {
                    for z in [-30.0_f32,-6.0,-1.0,0.0,2.5,10.0,19.0,24.0,40.0] {
                        for extent in [[0.5,0.5,0.5],[6.0,6.0,10.0],[20.0,3.0,2.0],[11.0,11.0,30.0]] {
                            owner.push(world);
                            cases.push(json!({
                                "vertices":vertices,
                                "triangles":triangles.iter().map(|tr| json!({"planes":tr.planes,"indices":tr.indices})).collect::<Vec<_>>(),
                                "nodes":nodes.iter().map(|n| json!({"indices":n.indices,"bounds":[n.bounds[0][0],n.bounds[0][1],n.bounds[0][2],n.bounds[1][0],n.bounds[1][1],n.bounds[1][2]]})).collect::<Vec<_>>(),
                                "l2w":rows.iter().flatten().collect::<Vec<_>>(),"w2l":local_rows.iter().flatten().collect::<Vec<_>>(),
                                "point":[x,y,z],"extent":extent,"shape":s,"transform":t,
                            }));
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
        let (vertices, triangles, nodes, to_world, to_local) = &worlds[owner[i]];
        let mesh = Mesh { vertices, triangles, nodes, transform: *to_world, world_to_local: *to_local };
        let point = serde_json::from_value(case["point"].clone()).unwrap();
        let extent = serde_json::from_value(case["extent"].clone()).unwrap();
        let mut hit = PointHit { location: [0.0;3], normal: [0.0;3], item: 0 };
        let encroaches = mesh.point_check(point, extent, &mut hit);
        assert_eq!(!encroaches, expected["fits"].as_bool().unwrap(), "{i}: shape {} transform {} point {point:?} extent {extent:?} expected {expected} got {hit:?}", case["shape"], case["transform"]);
        let location: [f32;3] = serde_json::from_value(expected["location"].clone()).unwrap();
        let normal: [f32;3] = serde_json::from_value(expected["normal"].clone()).unwrap();
        assert_eq!(hit.location.map(f32::to_bits), location.map(f32::to_bits), "{i}: shape {} transform {} {point:?} {extent:?} {expected} {hit:?}", case["shape"], case["transform"]);
        assert_eq!(hit.normal.map(f32::to_bits), normal.map(f32::to_bits), "{i}: {expected} {hit:?}");
        assert_eq!(hit.item, expected["item"].as_i64().unwrap() as i32, "{i}: {expected} {hit:?}");
        if encroaches { blocked += 1 } else { free += 1 }
    }
    assert!(blocked > 200 && free > 200, "{blocked} {free}");
}
