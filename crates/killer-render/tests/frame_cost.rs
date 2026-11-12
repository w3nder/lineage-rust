use glam::{Vec3, vec3};
use killer_assets::client::Client;
use killer_assets::options::ClientOptions;
use killer_assets::scene::{Neighbours, build_scene_with};
use killer_assets::textures::{BankSource, TextureBank};
use killer_assets::timeenv::TimeEnv;
use killer_render::camera::Camera;
use killer_render::environment::Environment;
use killer_render::headless::Headless;
use std::path::PathBuf;
use std::time::Instant;

#[test]
#[ignore]
fn frame_cost_of_a_big_tile_from_its_player_start() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let tile = std::env::var("KILLER_TILE").unwrap_or_else(|_| "18_20".into());
    let mut client = Client::open(&root).unwrap();
    let loaded = client.load_level(&tile).unwrap();
    let options = ClientOptions::load(&mut client).unwrap();
    let scene = build_scene_with(&mut client, &loaded.level, &options, Neighbours::default());
    let start = loaded.level.actors.iter().find(|a| a.class.eq_ignore_ascii_case("PlayerStart")).map(|a| Vec3::from(a.placement.location)).unwrap();
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
    let (width, height) = std::env::var("KILLER_SIZE").ok().and_then(|value| value.split_once('x').and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)))).unwrap_or((1280, 720));
    let Ok(mut headless) = Headless::new(width, height) else { return };
    let mut source = BankSource { bank: &mut bank, catalog: client.catalog() };
    headless.renderer_mut().clear();
    headless.renderer_mut().add_tile(&tile, &scene, &mut source);
    headless.renderer_mut().set_view_target(Some(start));
    let env = Environment::from_time(&timeenv, 12.0);
    for (distance, height) in [(400.0, 250.0), (3000.0, 1500.0)] {
    let mut total = 0.0;
    let mut draws = 0;
    for turn in 0..8 {
        let angle = turn as f32 * std::f32::consts::TAU / 8.0;
        let eye = start + vec3(-angle.cos() * distance, -angle.sin() * distance, height);
        let camera = Camera { eye, target: start + vec3(0.0, 0.0, 60.0), fov_x_deg: 70.0, near: 10.0, far: 60000.0 };
        headless.render(&camera, &env).unwrap();
        let begin = Instant::now();
        for _ in 0..10 {
            headless.render(&camera, &env).unwrap();
        }
        total += begin.elapsed().as_secs_f64() * 1000.0 / 10.0;
        draws += headless.renderer_mut().visible_draw_count(&camera, width as f32 / height);
    }
    eprintln!("{tile} a {distance} u: {:.2} ms por quadro (render + leitura), {} draws visíveis em média", total / 8.0, draws / 8);
    }
}
