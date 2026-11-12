use glam::{Mat4, Vec3, vec3};
use killer_assets::characters::{fallback_npc_appearance, load_part};
use killer_assets::client::Client;
use killer_assets::scene::build_scene;
use killer_assets::textures::{BankSource, TextureBank};
use killer_assets::timeenv::TimeEnv;
use killer_render::camera::Camera;
use killer_render::environment::Environment;
use killer_render::headless::Headless;
use killer_render::skinned::CharacterInstance;
use killer_ui::draw::{DrawList, Quad};
use std::path::PathBuf;

fn client() -> Option<Client> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    Client::open(&root).ok()
}

fn brightness(rgba: &[u8], x: usize, y: usize) -> u32 {
    let p = &rgba[(y * 1280 + x) * 4..][..3];
    u32::from(p[0]) + u32::from(p[1]) + u32::from(p[2])
}

#[test]
#[ignore]
fn hall_pennants_stay_lit_with_characters_in_the_frame() {
    let Some(mut client) = client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let timeenv = TimeEnv::load(&mut client, 0).unwrap();
    let mut bank = TextureBank::new();
    let scene = build_scene(&mut client, &loaded.level);
    for draw in &scene.draws {
        for path in draw.material.texture_paths() {
            bank.texture(client.catalog(), path);
        }
        if let Some(mask) = &draw.mask {
            bank.texture(client.catalog(), mask);
        }
    }
    let Ok(mut headless) = Headless::new(1280, 720) else { return };
    let mut source = BankSource { bank: &mut bank, catalog: client.catalog() };
    headless.upload(&scene, &mut source);
    let camera = Camera {
        eye: vec3(83226.0, 145353.0, -3350.0),
        target: vec3(83386.0, 145353.0, -3393.0),
        fov_x_deg: 60.0,
        near: 10.0,
        far: 60000.0,
    };
    let env = Environment::from_time(&timeenv, 12.0);
    let alone = headless.render(&camera, &env).unwrap();
    let tips = [(57usize, 4usize), (247, 4), (873, 4), (1047, 4), (1220, 4)];
    for (x, y) in tips {
        assert!(brightness(&alone, x, y) > 60, "sem personagens: ({x},{y}) = {}", brightness(&alone, x, y));
    }
    let part = &fallback_npc_appearance().parts[0];
    let mesh = load_part(&mut client, part).unwrap();
    let mut source = BankSource { bank: &mut bank, catalog: client.catalog() };
    let id = headless.renderer_mut().upload_character(&mesh, &mut source);
    let world = Mat4::from_translation(Vec3::new(83506.0, 145413.0, -3417.0))
        * Mat4::from_scale(Vec3::splat(10.0))
        * mesh.local;
    let (center, radius) = killer_assets::scene::transform::sphere(mesh.bounding_sphere, world);
    let bounds = killer_assets::scene::actor_lighting::Bounds::transformed(mesh.bounding_box, world);
    let lighting = killer_assets::scene::actor_lighting::LightMap::new(&loaded.level, &scene.lights);
    let sample = lighting.sample(killer_assets::scene::actor_lighting::Query { position: world.w_axis.truncate(), center, radius, bounds }, 0, 0.0);
    let instance = CharacterInstance { overlay: [0.0; 4], actor_id: 1, actor_position: world.w_axis.truncate(), collision_height: 22.0, shadow_caster: true, light_sphere: center.extend(radius).to_array(), mesh: id, world, palette: mesh.bind.clone(), zone: 4, ambient: sample.ambient, sun_affect: sample.sun_affect, lights: sample.lights, pawn_lights: Vec::new() };
    headless.renderer_mut().set_instances(&[instance]);
    let with = headless.render(&camera, &env).unwrap();
    for (x, y) in tips {
        assert!(brightness(&with, x, y) > 60, "com personagem: ({x},{y}) = {}", brightness(&with, x, y));
    }
    let mut list = DrawList::default();
    list.quads.push(Quad {
        x: 600.0,
        y: 170.0,
        w: 40.0,
        h: 13.0,
        u: 0.0,
        v: 0.0,
        uw: 40.0,
        vh: 13.0,
        texture: "L2Font-e.SmallFont-e".into(),
        color: [1.0; 4],
        rotation: 0.0, depth: 0.0, additive: false,
    });
    let mut source = BankSource { bank: &mut bank, catalog: client.catalog() };
    headless.renderer_mut().set_ui(&list, &mut source);
    let with_ui = headless.render(&camera, &env).unwrap();
    for (x, y) in tips {
        assert!(brightness(&with_ui, x, y) > 60, "com interface: ({x},{y}) = {}", brightness(&with_ui, x, y));
    }
}
