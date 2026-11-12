use glam::Vec3;
use killer_assets::scene::{DrawKind, MeshDraw, Pass, Scene, Vertex};
use killer_assets::scene::emitters::{EmitterAsset, EmitterPart, particle_material};
use killer_assets::textures::{TextureData, TextureFormatKind, TextureSource};
use killer_render::{camera::Camera, environment::Environment, headless::Headless};
use std::sync::Arc;
use ue2_level::{DrawStyle, Emitter, SpriteEmitter};

struct Textures;
impl TextureSource for Textures {
    fn texture(&mut self, path: &str) -> Arc<TextureData> {
        if path == "atlas" { return Arc::new(TextureData { width: 2, height: 1, format: TextureFormatKind::Rgba8, mips: vec![vec![255, 0, 0, 255, 0, 0, 255, 255]] }); }
        let pixel = match path { "gray" => [128, 128, 128, 255], "alpha" => [255, 255, 255, 128], "green" => [0, 255, 0, 255], _ => [255; 4] };
        Arc::new(TextureData { width: 1, height: 1, format: TextureFormatKind::Rgba8, mips: vec![pixel.to_vec()] })
    }
}

fn camera() -> Camera { Camera { eye: Vec3::new(0.0, -1000.0, 0.0), target: Vec3::ZERO, fov_x_deg: 60.0, near: 1.0, far: 10000.0 } }

fn draw(e: &SpriteEmitter, path: &str) -> MeshDraw {
    MeshDraw { instances: Vec::new(), vertices: Vec::new(), indices: Vec::new(), material: particle_material(e, path, true, true), mask: None, kind: DrawKind::Particle, sun_affect: false, pass: Pass::Translucent, zone: 0, lightmap: None, modulate_2x: false, sector: None, fade: None, tint: None, actor: None }
}

fn emitter(style: DrawStyle, path: &str, y: f32, size: f32, z_write: bool) -> EmitterAsset {
    let e = SpriteEmitter { coordinate_system: 0, automatic_initial_spawning: false, initial_particles_per_second: 30.0, max_particles: 1, lifetime: [100.0; 2], start_size: [[size; 2]; 3], draw_style: style, z_write, ..Default::default() };
    let part = EmitterPart { draws: vec![draw(&e, path)], ..Default::default() };
    EmitterAsset { definition: Emitter { location: [0.0, y, 0.0], sprites: vec![e], ..Default::default() }, parts: vec![part], zone: 0, range: 12000.0, lighting: None }
}

fn background() -> MeshDraw {
    let e = SpriteEmitter { draw_style: DrawStyle::Regular, z_write: true, ..Default::default() };
    let mut d = draw(&e, "gray");
    d.pass = Pass::Opaque;
    d.vertices = [[-1000.0, 100.0, -1000.0], [1000.0, 100.0, -1000.0], [-1000.0, 100.0, 1000.0], [1000.0, 100.0, 1000.0]].map(|position| Vertex { position, normal: [0.0, -1.0, 0.0], uv: [0.5; 2], mask_uv: [0.5; 2], color: [1.0; 4] }).to_vec();
    d.indices = vec![0, 1, 2, 1, 3, 2];
    d
}

fn render(headless: &mut Headless, scene: &Scene) -> Vec<u8> {
    headless.upload(scene, &mut Textures);
    headless.renderer_mut().set_particle_delta(Some(1.0 / 30.0));
    headless.render(&camera(), &Environment::default()).unwrap()
}

#[test]
fn sprite_106aa500_half_extent_and_d3d_1000a380_seven_blends() {
    let mut gpu = Headless::new(256, 256).unwrap();
    for (style, expected) in [(DrawStyle::Regular, 255), (DrawStyle::AlphaBlend, 192), (DrawStyle::Modulated, 255), (DrawStyle::Translucent, 255), (DrawStyle::AlphaModulate, 255), (DrawStyle::Darken, 0), (DrawStyle::Brighten, 255)] {
        let scene = Scene { draws: vec![background()], emitters: vec![emitter(style, "alpha", 0.0, 50.0, false)], ..Default::default() };
        let image = render(&mut gpu, &scene);
        let center = &image[(128 * 256 + 128) * 4..][..3];
        for channel in center { assert!((i32::from(*channel) - expected).abs() <= 2, "{style:?}: {center:?}"); }
        let width = (0..256).filter(|x| image[(128 * 256 + x) * 4] != 128).count() as f32;
        let projected = 100.0 / 1000.0 * 128.0 / 30.0_f32.to_radians().tan();
        assert!((width - projected).abs() <= 1.0, "{style:?}: {width} vs {projected}");
    }
}

#[test]
fn particle_zwrite_off_preserves_later_geometry_behind_and_culling_uses_actor_range() {
    let mut gpu = Headless::new(256, 256).unwrap();
    for write in [false, true] {
        let front = emitter(DrawStyle::Regular, "white", 0.0, 50.0, write);
        let mut back = emitter(DrawStyle::Regular, "green", 0.0, 50.0, false);
        back.definition.sprites[0].start_location[1] = [50.0; 2];
        back.parts[0].draws[0].material.sorting_level = 255;
        let image = render(&mut gpu, &Scene { emitters: vec![front, back], ..Default::default() });
        assert_eq!(&image[(128 * 256 + 128) * 4..][..3], if write { &[255, 255, 255] } else { &[0, 255, 0] });
    }
    let mut far = emitter(DrawStyle::Regular, "white", 0.0, 50.0, false);
    far.range = 999.0;
    let image = render(&mut gpu, &Scene { emitters: vec![far], ..Default::default() });
    assert_eq!(&image[(128 * 256 + 128) * 4..][..4], &killer_render::headless::CLEAR_RGBA);
}

#[test]
fn sprite_106ac0d0_blends_second_uv_with_diffuse_alpha_without_losing_texture_alpha() {
    let mut gpu = Headless::new(256, 256).unwrap();
    let mut actor = emitter(DrawStyle::AlphaBlend, "atlas", 0.0, 300.0, false);
    let e = &mut actor.definition.sprites[0];
    e.lifetime = [1.0; 2];
    e.initial_time = [0.25; 2];
    e.subdivisions = [2, 1];
    e.blend_subdivisions = true;
    actor.parts[0].draws[0].material = particle_material(e, "atlas", true, true);
    let image = render(&mut gpu, &Scene { emitters: vec![actor], ..Default::default() });
    let center = &image[(128 * 256 + 128) * 4..][..3];
    for (a, b) in center.iter().zip([128i32, 0, 127]) { assert!((i32::from(*a) - b).abs() <= 3, "{center:?}"); }
}

#[test]
fn mesh_105b0700_set_global_color_10019020_is_separate_for_each_particle() {
    let mut gpu = Headless::new(256, 256).unwrap();
    for use_particle_color in [false, true] {
        let mut actor = emitter(DrawStyle::AlphaBlend, "white", 0.0, 10.0, false);
        let e = &mut actor.definition.sprites[0];
        e.class = "MeshEmitter".into();
        e.mesh = Some(ue2_level::MeshShape { mesh: None, own_blend: true, two_sided: true, use_particle_color });
        e.max_particles = 2;
        e.initial_particles_per_second = 60.0;
        e.start_velocity[0] = [3600.0; 2];
        e.lifetime = [1.0; 2];
        e.use_color_scale = true;
        e.color_scale = vec![(0.0, [255, 0, 0, 255]), (1.0 / 60.0, [0, 0, 255, 255])];
        let mut mesh = background();
        mesh.kind = DrawKind::StaticMesh;
        for v in &mut mesh.vertices { v.position[0] /= 1000.0; v.position[1] = 0.0; v.position[2] /= 1000.0; }
        mesh.material = killer_assets::scene::texture_material("");
        mesh.material.stages[0].source = l2_catalog::StageSource::Factor;
        mesh.material.passes[0].lit = false;
        mesh.material.passes[0].two_sided = true;
        actor.parts[0].draws = vec![mesh];
        let image = render(&mut gpu, &Scene { emitters: vec![actor], ..Default::default() });
        let center = &image[(128 * 256 + 128) * 4..][..3];
        let projected = camera().view_proj(1.0).project_point3(Vec3::new(60.0, 0.0, 0.0));
        let x = ((projected.x + 1.0) * 128.0) as usize;
        let moved = &image[(128 * 256 + x) * 4..][..3];
        assert!(center[0] >= 253 && center[2] <= 2, "{center:?}");
        assert!(moved[0] <= 2 && moved[2] >= 253, "{moved:?}");
    }
}

#[test]
fn particle_material_slots_survive_tile_removal_growth_and_reload() {
    let mut gpu = Headless::new(256, 256).unwrap();
    let white = Scene { emitters: vec![emitter(DrawStyle::Regular, "white", 0.0, 50.0, false)], ..Default::default() };
    let green = Scene { emitters: vec![emitter(DrawStyle::Regular, "green", 0.0, 50.0, false)], ..Default::default() };
    render(&mut gpu, &white);
    gpu.renderer_mut().add_tile("next", &green, &mut Textures);
    gpu.renderer_mut().remove_tile("");
    let image = gpu.render(&camera(), &Environment::default()).unwrap();
    assert_eq!(&image[(128 * 256 + 128) * 4..][..3], &[0, 255, 0]);
    gpu.renderer_mut().set_tile_visible("next", false);
    let image = gpu.render(&camera(), &Environment::default()).unwrap();
    assert_eq!(&image[(128 * 256 + 128) * 4..][..4], &killer_render::headless::CLEAR_RGBA);
    gpu.renderer_mut().remove_tile("next");
    for i in 0..1030 { gpu.renderer_mut().add_tile(&i.to_string(), &white, &mut Textures); }
    let image = gpu.render(&camera(), &Environment::default()).unwrap();
    assert_eq!(&image[(128 * 256 + 128) * 4..][..3], &[255; 3]);
    let image = render(&mut gpu, &green);
    assert_eq!(&image[(128 * 256 + 128) * 4..][..3], &[0, 255, 0]);
}
