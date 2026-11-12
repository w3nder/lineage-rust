use glam::vec3;
use killer_assets::client::Client;
use killer_assets::options::ClientOptions;
use killer_assets::scene::{Neighbours, build_scene_with};
use killer_assets::textures::{BankSource, TextureBank, write_png};
use killer_assets::timeenv::TimeEnv;
use killer_render::camera::Camera;
use killer_render::environment::Environment;
use killer_render::headless::Headless;
use std::path::PathBuf;

fn client() -> Option<Client> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    Client::open(&root).ok()
}

#[test]
#[ignore]
fn giran_plaza_stair_is_solid_from_a_low_camera() {
    let Some(mut client) = client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let options = ClientOptions::load(&mut client).unwrap();
    let scene = build_scene_with(&mut client, &loaded.level, &options, Neighbours::default());
    let mirrored = loaded.level.actors.iter().filter(|a| a.static_mesh.is_some() && a.scale.iter().product::<f32>() < 0.0).count();
    eprintln!("{mirrored} actors espelhados (DrawScale3D com sinal negativo) em 22_22");
    let timeenv = TimeEnv::load(&mut client, 0).unwrap();
    let mut bank = TextureBank::new();
    for draw in &scene.draws {
        for path in draw.material.texture_paths() {
            bank.texture(client.catalog(), path);
        }
        if let Some(mask) = &draw.mask {
            bank.alpha_map(client.catalog(), mask);
        }
    }
    let camera = Camera { eye: vec3(83172.0, 149000.0, -3380.0), target: vec3(83172.0, 148618.0, -3420.0), fov_x_deg: 80.0, near: 10.0, far: 60000.0 };
    let env = Environment::from_time(&timeenv, 12.0);
    let Ok(mut headless) = Headless::new(1280, 720) else { return };
    let mut source = BankSource { bank: &mut bank, catalog: client.catalog() };
    headless.upload(&scene, &mut source);
    headless.renderer_mut().set_view_target(Some(camera.eye));
    let rgba = headless.render(&camera, &env).unwrap();
    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/giran-escada.png");
    write_png(&out, 1280, 720, &rgba).unwrap();
    assert!(mirrored > 0);
}
