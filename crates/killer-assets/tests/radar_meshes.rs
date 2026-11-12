use killer_assets::characters::{Part, load_part};
use killer_assets::client::Client;
use std::path::PathBuf;

const MESHES: [(&str, Option<&str>); 9] = [
    ("LineageDecos.radar_back", Some("L2UI_CH3.Radar.radar_back")),
    ("LineageDecos.radar_arrow", Some("L2UI_CH3.Radar.radar_arrow")),
    ("LineageDecos.radar_dot_party", Some("L2UI_CH3.Radar.radar_party")),
    ("LineageDecos.radar_dot_target", Some("L2UI_CH3.Radar.radar_target")),
    ("LineageDecos.radar_dot_tutorial1", Some("L2UI_CH3.Radar.radar_tutorial1")),
    ("LineageDecos.radar_dot_tutorial2", Some("L2UI_CH3.Radar.radar_tutorial2")),
    ("LineageDecos.radar_center", Some("L2UI_CH3.Radar.radar_myself")),
    ("LineageDecos.radar_n", Some("L2UI_CH3.Radar.radar_n")),
    ("LineageDecos.radar_time", None),
];

#[test]
#[ignore]
fn the_nine_radar_skeletal_meshes_load_as_flat_pieces_without_animation() {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from).expect("KILLER_CLIENT_DIR");
    let mut client = Client::open(&root).unwrap();
    for (name, texture) in MESHES {
        assert_eq!(client.catalog().object_class(name).as_deref(), Some("SkeletalMesh"), "{name}");
        let mesh = load_part(&mut client, &Part::new(name, Vec::new())).unwrap_or_else(|| panic!("{name}"));
        assert_eq!(mesh.bones.len(), 1, "{name}");
        assert_eq!(mesh.animation, None, "{name}");
        assert_eq!((mesh.origin, mesh.rotation, mesh.scale), (glam::Vec3::ZERO, [0; 3], glam::Vec3::ONE), "{name}");
        assert_eq!(mesh.sections.len(), 1, "{name}");
        let lod = mesh.lod.as_ref().unwrap();
        assert_eq!(lod.textures, vec![texture.map(str::to_owned)], "{name}");
        if let Some(texture) = texture {
            assert_eq!((mesh.vertices.len(), mesh.indices.len()), (4, 6), "{name}");
            assert_eq!(mesh.sections[0].texture(), texture, "{name}");
            assert!(mesh.vertices.iter().all(|v| v.position[2] == 0.0), "{name}");
            assert!(client.catalog().texture_raw(texture).is_some(), "{texture}");
        }
        eprintln!("{name}: {} vértices, caixa {:?}, uv {:?}, {}", mesh.vertices.len(), mesh.bounding_box, mesh.vertices.iter().map(|v| (v.position, v.uv)).collect::<Vec<_>>(), mesh.sections[0].texture());
    }
}
