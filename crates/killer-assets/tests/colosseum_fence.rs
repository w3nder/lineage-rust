use killer_assets::client::Client;
use killer_assets::options::ClippingRange;
use killer_assets::scene::DrawKind;
use killer_assets::scene::props::{SpawnedMesh, build_spawned};
use std::path::PathBuf;

#[test]
#[ignore]
fn make_column_and_make_wall_meshes_build_as_spawnable_static_mesh_actors() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let meshes = ["Aden_Colosseum.aden_colosseum_fence01", "Aden_Colosseum.aden_colosseum_fence02"];
    for mesh in meshes {
        assert_eq!(client.catalog().object_class(mesh).as_deref(), Some("StaticMesh"), "{mesh}");
    }
    let loaded = client.load_level("24_18").unwrap();
    let location = loaded.level.actors.iter().find(|actor| actor.static_mesh.is_some()).unwrap().placement.location;
    let spawned: Vec<_> = meshes.iter().enumerate().map(|(index, mesh)| SpawnedMesh {
        mesh: (*mesh).into(),
        placement: ue2_level::Placement { location, rotation: [0, 0x4000 * index as i32, 0] },
        actor_id: 0x8000_0000 + index as u32,
    }).collect();
    let pieces = build_spawned(&mut client, &loaded.level, &ClippingRange { actor: 5.5, static_mesh: 3.5, static_mesh_lod: 5.5, pawn: 1.5, terrain: 5.5, projector: 1.5 }, &spawned);
    assert_eq!(pieces.len(), 2);
    for (index, draws) in pieces.iter().enumerate() {
        assert!(draws.iter().map(|draw| draw.indices.len()).sum::<usize>() > 0, "{}", meshes[index]);
        let mut textures: Vec<_> = draws.iter().flat_map(|draw| draw.material.texture_paths()).collect();
        textures.sort_unstable();
        assert_eq!(textures, ["Aden_Colosseum_T.colo_add_wall003", "Aden_Colosseum_T.colo_add_wall007", "Aden_Colosseum_T.colo_add_wall009"]);
        for draw in draws {
            assert_eq!(draw.kind, DrawKind::StaticMesh);
            assert_eq!(draw.actor.map(|actor| actor.id), Some(0x8000_0000 + index as u32));
            assert_eq!(draw.indices.is_empty(), draw.vertices.is_empty());
        }
    }
}

#[test]
#[ignore]
fn spawned_fence_pieces_block_sweeps_as_immediately_stop_static_mesh_actors() {
    use glam::Vec3;
    use killer_assets::collision::{resources::Resources, world::{Spawned, Surface, View, trace_selection, trace_spawned, trace_targets}};
    use killer_assets::pickup::TraceQuery;
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let mut resources = Resources::default();
    let shape = resources.class_collision(&mut client, "Engine.SpawnableStaticMeshActor").unwrap().expect("SpawnableStaticMeshActor fora do hash");
    assert_eq!(shape.trace_flags, 0x80);
    assert_eq!(shape.cylinder, None);
    assert!(shape.static_mesh_actor && shape.immediately_stop && shape.unscaled());
    assert_eq!(shape.pushable_mass, None);
    assert_eq!(shape.pre_pivot, [0.0; 3]);
    let loaded = client.load_level("24_18").unwrap();
    let base = Vec3::from(loaded.level.actors.iter().find(|actor| actor.static_mesh.is_some()).unwrap().placement.location);
    let views = [View { name: "24_18".into(), level: std::sync::Arc::new(loaded.level), poses: Default::default() }];
    for (mesh, rotation, across) in [("Aden_Colosseum.aden_colosseum_fence01", [0, 0, 0], Vec3::Y), ("Aden_Colosseum.aden_colosseum_fence02", [0, 0, 0], Vec3::Y), ("Aden_Colosseum.aden_colosseum_fence02", [0, 0x4000, 0], Vec3::X)] {
        let location = base + Vec3::Z * 1000.0;
        let bounds = resources.bounds(&mut client, mesh).unwrap().bounds;
        let center = location + (Vec3::from(bounds[0]) + Vec3::from(bounds[1])) / 2.0;
        let center = Vec3::new(location.x, location.y, center.z);
        let query = TraceQuery { start: center - across * 300.0, end: center + across * 300.0, extent: Vec3::new(10.0, 10.0, 10.0), flags: 0x86 };
        let level = trace_selection(&mut client, &mut resources, &views, query).unwrap().map(|selection| selection.hit.time);
        let piece = Spawned { class: "Engine.SpawnableStaticMeshActor".into(), mesh: mesh.into(), placement: ue2_level::Placement { location: location.to_array(), rotation } };
        let hit = trace_spawned(&mut client, &mut resources, &views, query, &[], std::slice::from_ref(&piece)).unwrap().unwrap_or_else(|| panic!("{mesh} {rotation:?} não bloqueia"));
        assert!(hit.immediately_stop, "{mesh}");
        assert_eq!(hit.surface, Some(Surface::StaticMeshActor));
        assert!(hit.hit.time > 0.0 && hit.hit.time < 0.5 && hit.hit.time < level.unwrap_or(1.0), "{mesh}: {:?} nível {level:?}", hit.hit);
        let click = TraceQuery { extent: Vec3::splat(0.1), flags: 0x200bf, ..query };
        let picked = trace_targets(&mut client, &mut resources, &views, click, &[], std::slice::from_ref(&piece)).unwrap().unwrap_or_else(|| panic!("{mesh} fora do clique"));
        assert!(picked.target.is_none() && picked.surface == Some(Surface::StaticMeshActor), "{mesh}: {picked:?}");
        let camera = TraceQuery { extent: Vec3::splat(0.1), ..query };
        let viewed = trace_spawned(&mut client, &mut resources, &views, camera, &[], std::slice::from_ref(&piece)).unwrap().unwrap_or_else(|| panic!("{mesh} fora da câmera"));
        assert_eq!(viewed.surface, Some(Surface::StaticMeshActor), "{mesh}");
        let elsewhere = Spawned { placement: ue2_level::Placement { location: (location + Vec3::Z * 2000.0).to_array(), rotation }, ..piece };
        assert_eq!(trace_spawned(&mut client, &mut resources, &views, query, &[], &[elsewhere]).unwrap().map(|selection| selection.hit.time), level, "{mesh}");
    }
}
