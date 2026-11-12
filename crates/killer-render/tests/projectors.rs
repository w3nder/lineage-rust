use glam::Vec3;
use killer_assets::{
    cache::{CachedScene, SourceStamp, decode, encode_to_vec},
    client::Client,
    options::ClientOptions,
    scene::{DrawKind, Neighbours, Pass, build_scene_with},
    textures::{BankSource, TextureBank, write_png},
    timeenv::TimeEnv,
};
use killer_render::{camera::Camera, environment::Environment, headless::Headless};
use std::path::PathBuf;

#[test]
#[ignore]
fn c4_bsp_projector_pixels_and_cache_in_every_populated_map() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").unwrap());
    let mut client = Client::open(&root).unwrap();
    let options = ClientOptions::load(&mut client).unwrap();
    let environment = Environment::from_time(&TimeEnv::load(&mut client, 0).unwrap(), 12.0);
    let mut gpu = Headless::new(640, 360).unwrap();
    for tile in ["16_10", "20_17", "24_20"] {
        let loaded = client.load_level(tile).unwrap();
        let mut scene = build_scene_with(&mut client, &loaded.level, &options, Neighbours::default());
        scene.retain_draws(|d| d.kind == DrawKind::Bsp || d.material.root_class == "projectormaterial");
        let projected: Vec<_> = scene.draws.iter().filter(|d| d.pass == Pass::Projector).collect();
        if projected.is_empty() {
            let mut rejected = 0;
            for p in &loaded.level.projectors {
                let material = client.catalog().material(p.texture.as_deref().unwrap()).unwrap();
                let projection = killer_assets::scene::projectors::calc_matrix(p, material.first_texture().unwrap().size.map(|n| n as u32), None);
                for polygon in &loaded.level.bsp {
                    let points: Vec<_> = polygon.corners.iter().copied().map(Vec3::from).collect();
                    if killer_assets::scene::projectors::clip_bsp_polygon(&points, &projection.planes, true).len() < 3 { continue; }
                    let node = loaded.level.bsp_render.node_draws[polygon.node_index];
                    let section = &loaded.level.bsp_render.sections[usize::try_from(node.section).unwrap()];
                    assert!(polygon.flags & 0x8000 != 0 || (section.poly_flags & 0x400000 != 0 && p.flags & ue2_level::projectors::ON_UNLIT == 0), "{tile}: receiver omitted without a native filter");
                    rejected += 1;
                }
            }
            assert!(rejected > 0);
            eprintln!("{tile}: all BSP receivers rejected by surface/unlit filters");
            continue;
        }
        let cameras: Vec<_> = projected.iter().filter(|d| d.vertices.iter().any(|v| v.color[3] > 0.0)).map(|draw| {
            let [a, b, c] = [0, 1, 2].map(|i| Vec3::from(draw.vertices[draw.indices[i] as usize].position));
            let center = (a + b + c) / 3.0;
            let normal = (b - a).cross(c - a).normalize();
            let offset = -normal * 100.0 + if normal.z.abs() > 0.9 { Vec3::Y * -50.0 } else { Vec3::ZERO };
            Camera { eye: center + offset, target: center, fov_x_deg: 60.0, near: 1.0, far: 10000.0 }
        }).collect();
        eprintln!("{tile}: {} BSP projector draws; {} receiver views", projected.len(), cameras.len());
        scene.emitters.clear();
        scene.moving.clear();
        scene.sky_bodies.clear();
        scene.flares.clear();
        scene.coronas.clear();
        let mut bank = TextureBank::new();
        gpu.upload(&scene, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        let (camera, before, after, changed) = cameras.into_iter().filter_map(|camera| {
            gpu.renderer_mut().set_projectors(false);
            let before = gpu.render(&camera, &environment).unwrap();
            gpu.renderer_mut().set_projectors(true);
            let after = gpu.render(&camera, &environment).unwrap();
            let changed = before.as_chunks::<4>().0.iter().zip(after.as_chunks::<4>().0.iter()).filter(|(a, b)| a[..3] != b[..3]).count();
            (changed > 0).then_some((camera, before, after, changed))
        }).max_by_key(|(_, _, _, changed)| *changed).unwrap_or_else(|| panic!("{tile}: no receiver view has a visible contribution"));
        let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders");
        write_png(&output.join(format!("task12-{tile}-bsp-com.png")), 640, 360, &after).unwrap();
        write_png(&output.join(format!("task12-{tile}-bsp-sem.png")), 640, 360, &before).unwrap();
        assert!(changed > 0, "{tile}: BSP projector has no visible contribution");
        let stamp = SourceStamp::of(tile.as_bytes());
        let bytes = encode_to_vec(&stamp, &CachedScene::from_bank(scene, &bank));
        let cached = decode(bytes.as_slice(), &stamp).unwrap();
        gpu.upload(&cached.scene, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        assert_eq!(gpu.render(&camera, &environment).unwrap(), after, "{tile}: BSP cache changed pixels");
        eprintln!("{tile}: {changed} BSP pixels changed; cache identical");
    }
}

#[test]
#[ignore]
fn c4_projector_terrain_pixels_and_cache_in_both_populated_maps() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").unwrap());
    let mut client = Client::open(&root).unwrap();
    let options = ClientOptions::load(&mut client).unwrap();
    let environment = Environment::from_time(&TimeEnv::load(&mut client, 0).unwrap(), 12.0);
    let mut gpu = Headless::new(640, 360).unwrap();
    for tile in ["20_17", "24_20"] {
        let loaded = client.load_level(tile).unwrap();
        let mut scene = build_scene_with(&mut client, &loaded.level, &options, Neighbours::default());
        let first = scene.draws.iter().find(|d| d.pass == Pass::Projector).unwrap();
        let center = first.indices.iter().map(|i| Vec3::from(first.vertices[*i as usize].position)).sum::<Vec3>() / first.indices.len() as f32;
        scene.retain_draws(|d| matches!(d.kind, DrawKind::Terrain | DrawKind::Projector)
            && d.material.root_class != "projectormaterial");
        scene.emitters.clear();
        scene.moving.clear();
        scene.sky_bodies.clear();
        scene.flares.clear();
        scene.coronas.clear();
        let camera = Camera {
            eye: center + Vec3::new(0.0, -1000.0, 1200.0), target: center,
            fov_x_deg: 60.0, near: 10.0, far: 10000.0,
        };
        let mut bank = TextureBank::new();
        gpu.upload(&scene, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        gpu.renderer_mut().set_projectors(false);
        let before = gpu.render(&camera, &environment).unwrap();
        gpu.renderer_mut().set_projectors(true);
        let after = gpu.render(&camera, &environment).unwrap();
        let changed = before.as_chunks::<4>().0.iter().zip(after.as_chunks::<4>().0.iter()).filter(|(a, b)| a[..3] != b[..3]).count();
        assert!(changed > 0, "{tile}: projetor sem contribuição nos pixels");
        gpu.renderer_mut().set_projector_view(None, false, true);
        assert_eq!(gpu.render(&camera, &environment).unwrap(), before, "{tile}: filtro wireframe");
        gpu.renderer_mut().set_projector_view(None, false, false);
        assert_eq!(gpu.render(&camera, &environment).unwrap(), after, "{tile}: restauração do filtro");
        let stamp = SourceStamp::of(tile.as_bytes());
        let cached = CachedScene::from_bank(scene, &bank);
        let bytes = encode_to_vec(&stamp, &cached);
        let cached = decode(bytes.as_slice(), &stamp).unwrap();
        gpu.upload(&cached.scene, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        assert_eq!(gpu.render(&camera, &environment).unwrap(), after, "{tile}: cache mudou pixels");
        let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders");
        write_png(&output.join(format!("task12-{tile}-terreno-com.png")), 640, 360, &after).unwrap();
        write_png(&output.join(format!("task12-{tile}-terreno-sem.png")), 640, 360, &before).unwrap();
        eprintln!("{tile}: {changed} pixels alterados; câmera {:?} -> {:?}", camera.eye, camera.target);
    }
}
