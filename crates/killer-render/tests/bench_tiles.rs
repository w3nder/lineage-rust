use glam::{Vec3, vec3};
use killer_assets::client::Client;
use killer_assets::options::ClientOptions;
use killer_assets::scene::{Neighbours, build_scene_with, sky};
use killer_assets::textures::{BankSource, TextureBank};
use killer_assets::timeenv::TimeEnv;
use killer_render::camera::Camera;
use killer_render::environment::Environment;
use killer_render::headless::Headless;
use std::path::PathBuf;
use std::time::Instant;

fn client() -> Option<Client> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    Client::open(&root).ok()
}

const FRAMES: usize = 20;

fn bench(client: &mut Client, tile: &str) -> (usize, usize, f64) {
    let loaded = client.load_level(tile).unwrap();
    let options = ClientOptions::load(client).unwrap();
    let scene = build_scene_with(client, &loaded.level, &options, Neighbours::default());
    let sky = sky::load_shared(client, &options).unwrap();
    let timeenv = TimeEnv::load(client, 0).unwrap();
    let start = loaded.level.actors.iter().find(|actor| actor.class.eq_ignore_ascii_case("PlayerStart")).unwrap();
    let target = Vec3::from(start.placement.location);
    let yaw = start.placement.rotation[1] as f32 * std::f32::consts::TAU / 65536.0;
    let back = vec3(yaw.cos(), yaw.sin(), 0.0) * -230.0 + vec3(0.0, 0.0, 120.0);
    let camera = Camera { eye: target + back, target: target + vec3(0.0, 0.0, 60.0), fov_x_deg: 60.0, near: 10.0, far: 60000.0 };
    let mut bank = TextureBank::new();
    for draw in scene.draws.iter().chain(&sky.draws) {
        for path in draw.material.texture_paths() {
            bank.texture(client.catalog(), path);
        }
        if let Some(mask) = &draw.mask {
            bank.alpha_map(client.catalog(), mask);
        }
    }
    let mut headless = Headless::new(1280, 720).unwrap();
    let mut source = BankSource { bank: &mut bank, catalog: client.catalog() };
    headless.upload(&scene, &mut source);
    headless.renderer_mut().add_tile("sky", &sky, &mut source);
    headless.renderer_mut().set_view_target(Some(target));
    let env = Environment::from_time(&timeenv, 12.0);
    headless.render(&camera, &env).unwrap();
    let started = Instant::now();
    for _ in 0..FRAMES {
        headless.render(&camera, &env).unwrap();
    }
    let ms = started.elapsed().as_secs_f64() * 1000.0 / FRAMES as f64;
    let (draws, triangles) = headless.renderer_mut().visible_stats(&camera, 1280.0 / 720.0);
    (draws, triangles, ms)
}

#[test]
#[ignore]
fn giran_and_heine_frame_cost_from_the_player_start() {
    let Some(mut client) = client() else { return };
    for tile in ["22_22", "23_24"] {
        let (draws, triangles, ms) = bench(&mut client, tile);
        eprintln!("{tile}: {draws} draws, {triangles} triângulos visíveis, {ms:.1} ms/quadro (headless 1280×720, debug)");
        assert!(draws > 0);
    }
}
