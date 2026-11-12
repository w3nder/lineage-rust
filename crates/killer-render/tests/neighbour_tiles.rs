use glam::vec3;
use killer_assets::client::Client;
use killer_assets::options::ClientOptions;
use killer_assets::scene::{DrawKind, Neighbours, build_scene_with};
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
fn heine_and_its_west_neighbour_render_together_stitched_at_the_border() {
    let Some(mut client) = client() else { return };
    let heine = client.load_level("23_24").unwrap();
    let west = client.load_level("22_24").unwrap();
    let options = ClientOptions::load(&mut client).unwrap();
    let heine_scene = build_scene_with(&mut client, &heine.level, &options, Neighbours::default());
    let west_scene = build_scene_with(&mut client, &west.level, &options, Neighbours { right: Some(&heine.level), top: None, corner: None });
    let border_x = 98304.0;
    let seam: Vec<[f32; 3]> = west_scene
        .draws
        .iter()
        .filter(|draw| draw.kind == DrawKind::Terrain && draw.sector.is_some())
        .flat_map(|draw| draw.vertices.iter().map(|v| v.position))
        .filter(|p| (p[0] - border_x).abs() < 0.5)
        .collect();
    let first_column: Vec<[f32; 3]> = heine_scene
        .draws
        .iter()
        .filter(|draw| draw.kind == DrawKind::Terrain && draw.sector.is_some())
        .flat_map(|draw| draw.vertices.iter().map(|v| v.position))
        .filter(|p| (p[0] - border_x).abs() < 0.5)
        .collect();
    assert!(!seam.is_empty() && !first_column.is_empty());
    for p in &seam {
        let same = first_column.iter().any(|q| (q[1] - p[1]).abs() < 0.5 && (q[2] - p[2]).abs() < 0.01);
        assert!(same, "vértice de costura {p:?} sem par na coluna 0 de 23_24");
    }
    let timeenv = TimeEnv::load(&mut client, 0).unwrap();
    let mut bank = TextureBank::new();
    for draw in heine_scene.draws.iter().chain(&west_scene.draws) {
        for path in draw.material.texture_paths() {
            bank.texture(client.catalog(), path);
        }
        if let Some(mask) = &draw.mask {
            bank.alpha_map(client.catalog(), mask);
        }
    }
    let Ok(mut headless) = Headless::new(1280, 720) else { return };
    let mut source = BankSource { bank: &mut bank, catalog: client.catalog() };
    headless.renderer_mut().clear();
    headless.renderer_mut().add_tile("23_24", &heine_scene, &mut source);
    headless.renderer_mut().add_tile("22_24", &west_scene, &mut source);
    let camera = Camera { eye: vec3(101_000.0, 221_500.0, -2800.0), target: vec3(90_000.0, 219_000.0, -3500.0), fov_x_deg: 70.0, near: 10.0, far: 60000.0 };
    headless.renderer_mut().set_view_target(Some(vec3(border_x + 4000.0, 221_500.0, -3500.0)));
    let env = Environment::from_time(&timeenv, 12.0);
    let both = headless.render(&camera, &env).unwrap();
    headless.renderer_mut().set_tile_visible("22_24", false);
    let alone = headless.render(&camera, &env).unwrap();
    let changed = both.as_chunks::<4>().0.iter().zip(alone.as_chunks::<4>().0.iter()).filter(|(a, b)| a != b).count();
    eprintln!("{changed} pixels vêm do vizinho 22_24");
    assert!(changed > 100_000, "{changed}");
    headless.renderer_mut().set_tile_visible("22_24", true);
    let with_neighbour = headless.renderer_mut().visible_draw_count(&camera, 1280.0 / 720.0);
    headless.renderer_mut().set_tile_visible("22_24", false);
    let without = headless.renderer_mut().visible_draw_count(&camera, 1280.0 / 720.0);
    eprintln!("draws visíveis: {with_neighbour} com o vizinho, {without} com ele fora de alcance");
    assert!(without < with_neighbour, "o tile escondido pela regra dos 28672 u não pode emitir draws");
    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/heine-vizinho.png");
    write_png(&out, 1280, 720, &both).unwrap();
}
