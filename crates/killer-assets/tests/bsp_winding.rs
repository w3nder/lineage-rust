use killer_assets::client::Client;
use killer_assets::scene::{DrawKind, build_scene};
use std::path::PathBuf;

fn sign(draw: &killer_assets::scene::MeshDraw) -> (usize, usize) {
    let (mut positive, mut negative) = (0, 0);
    for tri in draw.indices.as_chunks::<3>().0 {
        let v = |i: u32| glam::Vec3::from(draw.vertices[i as usize].position);
        let n = glam::Vec3::from(draw.vertices[tri[0] as usize].normal);
        let geometric = (v(tri[1]) - v(tri[0])).cross(v(tri[2]) - v(tri[1]));
        let dot = geometric.dot(n);
        if dot > 1e-3 { positive += 1 } else if dot < -1e-3 { negative += 1 }
    }
    (positive, negative)
}

#[test]
#[ignore]
fn bsp_and_static_mesh_winding_signs_against_their_normals() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    for tile in ["22_22", "23_24", "20_17", "16_10"] {
        let Ok(loaded) = client.load_level(tile) else { continue };
        let scene = build_scene(&mut client, &loaded.level);
        let (mut bsp, mut mesh) = ((0, 0), (0, 0));
        for draw in &scene.draws {
            let s = match draw.kind {
                DrawKind::Bsp => &mut bsp,
                DrawKind::StaticMesh => &mut mesh,
                _ => continue,
            };
            let (p, n) = sign(draw);
            s.0 += p;
            s.1 += n;
        }
        assert!(bsp.0 > 10 * bsp.1, "{tile}: bsp {bsp:?}");
        assert!(mesh.1 > 10 * mesh.0, "{tile}: static mesh {mesh:?}");
    }
}
