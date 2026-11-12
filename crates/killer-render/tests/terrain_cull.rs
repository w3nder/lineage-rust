use glam::{Vec3, vec3};
use killer_assets::client::Client;
use killer_assets::scene::{DrawKind, Scene, build_scene};
use killer_assets::textures::{BankSource, TextureBank};
use killer_render::camera::Camera;
use killer_render::headless::Headless;
use std::path::PathBuf;

fn client() -> Option<Client> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    Client::open(&root).ok()
}

#[test]
#[ignore]
fn giran_terrain_draws_only_the_sectors_within_terrain_range_of_the_player_start() {
    let Some(mut client) = client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let start = loaded.level.actors.iter().find(|actor| actor.class.eq_ignore_ascii_case("PlayerStart")).unwrap();
    let target = Vec3::from(start.placement.location);
    let full = build_scene(&mut client, &loaded.level);
    assert_eq!(full.terrain_range, 16384.0);
    let mut scene = Scene { zones: full.zones.clone(), lightmaps: full.lightmaps.clone(), terrain_range: full.terrain_range, ..Default::default() };
    scene.draws = full.draws.iter().filter(|draw| draw.kind == DrawKind::Terrain && draw.sector.is_some()).cloned().collect();
    let Ok(mut headless) = Headless::new(320, 180) else { return };
    let mut bank = TextureBank::new();
    let mut source = BankSource { bank: &mut bank, catalog: client.catalog() };
    headless.upload(&scene, &mut source);
    let camera = Camera { eye: target + vec3(0.0, -400.0, 300.0), target, fov_x_deg: 90.0, near: 10.0, far: 60000.0 };
    let frustum = camera.frustum(320.0 / 180.0);
    let expected = |reference: Vec3| {
        scene
            .draws
            .iter()
            .filter(|draw| {
                let sector = draw.sector.unwrap();
                let location = Vec3::from(sector.location);
                (location.truncate() - reference.truncate()).length() <= 16384.0 && frustum.contains_box(Vec3::from(sector.min), Vec3::from(sector.max))
            })
            .count()
    };
    let all_in_frustum = scene.draws.iter().filter(|draw| draw.sector.is_some_and(|s| frustum.contains_box(Vec3::from(s.min), Vec3::from(s.max)))).count();
    headless.renderer_mut().set_view_target(Some(target));
    let visible = headless.renderer_mut().visible_draw_count(&camera, 320.0 / 180.0);
    eprintln!("{} draws de terreno, {all_in_frustum} no frustum, {visible} desenhados com o ViewTarget no PlayerStart", scene.draws.len());
    assert_eq!(visible, expected(target));
    let far = target + vec3(20000.0, 0.0, 0.0);
    headless.renderer_mut().set_view_target(Some(far));
    let culled = headless.renderer_mut().visible_draw_count(&camera, 320.0 / 180.0);
    eprintln!("{culled} desenhados com o ViewTarget 20000 u a leste");
    assert_eq!(culled, expected(far));
    assert!(culled < all_in_frustum, "{culled} vs {all_in_frustum}");
}
