use glam::Vec3;
use killer_assets::{
    client::Client,
    options::ClientOptions,
    scene::{Neighbours, build_scene_with},
    textures::{BankSource, TextureBank, write_png},
    timeenv::TimeEnv,
};
use killer_render::{camera::Camera, environment::Environment, headless::Headless};
use std::path::PathBuf;

#[test]
#[ignore]
fn original_movers_render_closed_and_open_in_three_tiles() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("cliente"));
    let mut client = Client::open(&root).unwrap();
    let options = ClientOptions::load(&mut client).unwrap();
    let time = TimeEnv::load(&mut client, 0).unwrap();
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders");
    for tile in ["22_22", "23_24", "24_18"] {
        let loaded = client.load_level(tile).unwrap();
        let scene = build_scene_with(&mut client, &loaded.level, &options, Neighbours::default());
        let actor = scene
            .moving
            .iter()
            .find(|a| {
                if tile == "23_24" {
                    a.mesh.as_ref().is_some_and(|p| p.ends_with("Innadrile_gondola"))
                } else if tile == "24_18" {
                    a.name == "Mover33"
                } else {
                    a.mover.as_ref().is_some_and(|m| m.server_id.is_some() && m.key_rot[1] != [0; 3])
                }
            })
            .unwrap();
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for vertex in actor.draws.iter().flat_map(|d| &d.vertices) {
            min = min.min(Vec3::from(vertex.position));
            max = max.max(Vec3::from(vertex.position));
        }
        let center = (min + max) * 0.5;
        let span = max - min;
        let distance = span.max_element().max(100.0) * 1.8;
        let direction = if tile == "23_24" {
            Vec3::new(0.15, 0.15, 1.0)
        } else if span.x < span.y {
            Vec3::new(1.0, 0.15, 0.35)
        } else {
            Vec3::new(0.15, 1.0, 0.35)
        };
        let camera =
            Camera { eye: center + direction * distance, target: center, fov_x_deg: 65.0, near: 1.0, far: 60000.0 };
        let mut bank = TextureBank::new();
        let mut source = BankSource { bank: &mut bank, catalog: client.catalog() };
        let mut headless = Headless::new(960, 540).unwrap();
        headless.upload(&scene, &mut source);
        headless.renderer_mut().set_movers_auto_tick(false);
        headless.renderer_mut().set_view_target(Some(center));
        let env = Environment::from_time(&time, 12.0);
        let before = headless.render(&camera, &env).unwrap();
        write_png(&output.join(format!("task10-{tile}-fechado.png")), 960, 540, &before).unwrap();
        if let Some(id) = actor.mover.as_ref().and_then(|m| m.server_id) {
            headless.renderer_mut().door_info(100, id);
            headless.renderer_mut().door_status(100, false, 0, false);
        } else {
            headless.renderer_mut().tick_movers(0.0);
        }
        headless.renderer_mut().tick_movers(1.0);
        let after = headless.render(&camera, &env).unwrap();
        assert!(before != after, "{tile}: movimento precisa aparecer na GPU");
        write_png(&output.join(format!("task10-{tile}-movimento.png")), 960, 540, &after).unwrap();
        assert_eq!(headless.renderer_mut().moving_actors().count(), scene.moving.len());
        client.catalog().release_files();
    }
}
