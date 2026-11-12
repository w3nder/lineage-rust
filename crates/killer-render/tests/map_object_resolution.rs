use glam::Vec3;
use killer_assets::{
    client::Client,
    options::ClientOptions,
    scene::{self, Scene},
    textures::{BankSource, TextureBank, write_png},
    timeenv::TimeEnv,
};
use killer_render::{camera::Camera, environment::Environment, headless::Headless};
use std::path::PathBuf;

#[test]
#[ignore]
fn original_homonymous_map_meshes_reach_gpu_with_their_materials() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let options = ClientOptions::load(&mut client).unwrap();
    let environment = Environment::from_time(&TimeEnv::load(&mut client, 0).unwrap(), 12.0);
    let mut gpu = Headless::new(512, 512).unwrap();
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/map-object-resolution");
    std::fs::create_dir_all(&output).unwrap();
    for (tile, name) in [
        ("17_25", "sp_lighthouse.sp_lighthouse001"),
        ("17_25", "sp_lighthouse.sp_lighthouse002"),
        ("18_23", "gludio_dungeon_enterance_s.gludio_dungeon_enterance01"),
        ("18_23", "gludio_dungeon_enterance_s.gludio_dungeon_enterance03"),
    ] {
        let mut level = client.load_level(tile).unwrap().level;
        level.actors.retain(|actor| {
            actor.class.eq_ignore_ascii_case("LevelInfo")
                || actor.static_mesh.as_deref().is_some_and(|path| path.eq_ignore_ascii_case(name))
        });
        let mut scene = Scene { zones: scene::zone_lighting(&level), ..Default::default() };
        scene.draws = scene::props::build(&mut client, &level, &options.clipping);
        let mut low = Vec3::splat(f32::MAX);
        let mut high = Vec3::splat(f32::MIN);
        for vertex in scene.draws.iter().flat_map(|draw| draw.world_vertices()) {
            low = low.min(Vec3::from(vertex.position));
            high = high.max(Vec3::from(vertex.position));
        }
        scene.bounds_min = low.to_array();
        scene.bounds_max = high.to_array();
        assert!(!scene.draws.is_empty(), "{name}");
        let center = (Vec3::from(scene.bounds_min) + Vec3::from(scene.bounds_max)) * 0.5;
        let radius = Vec3::from(scene.bounds_max).distance(center).max(1.0);
        let camera = Camera {
            eye: center + Vec3::new(1.0, -1.5, 0.65).normalize() * radius * 2.6,
            target: center,
            fov_x_deg: 60.0,
            near: 1.0,
            far: radius * 10.0,
        };
        let mut bank = TextureBank::new();
        gpu.upload(&Scene::default(), &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        let empty = gpu.render(&camera, &environment).unwrap();
        gpu.upload(&scene, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        let image = gpu.render(&camera, &environment).unwrap();
        let changed = image.as_chunks::<4>().0.iter().zip(empty.as_chunks::<4>().0.iter()).filter(|(a, b)| a != b).count();
        assert!(changed > 0, "{name}: nada visível");
        write_png(&output.join(format!("{name}.png")), 512, 512, &image).unwrap();
        eprintln!("{tile} {name}: {} draws, {changed} pixels distintos", scene.draws.len());
        client.catalog().release_files();
    }
}
