use glam::{Mat4, Vec3, vec3};
use killer_assets::characters::{Part, load_part};
use killer_assets::client::Client;
use killer_assets::scene::{Scene, ZoneLighting};
use killer_assets::textures::{BankSource, TextureBank, write_png};
use killer_assets::timeenv::TimeEnv;
use killer_render::camera::Camera;
use killer_render::environment::Environment;
use killer_render::headless::{CLEAR_RGBA, Headless};
use killer_render::skinned::CharacterInstance;
use std::path::PathBuf;

const SIZE: u32 = 512;

fn client() -> Option<Client> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    Client::open(&root).ok()
}

fn dwarf_parts(client: &mut Client) -> Vec<Part> {
    let tables = client.tables().unwrap();
    let mut parts = Vec::new();
    for armor in tables.armorgrp.values() {
        for row in &armor.rows {
            for (index, texture) in row.textures.iter().enumerate() {
                if texture.to_lowercase().starts_with("c5mdwarf.mdwarf_m004_t1005_")
                    && let Some(mesh) = row.meshes.get(index).filter(|mesh| !mesh.is_empty())
                {
                    parts.push(Part::new(mesh.clone(), vec![texture.clone()]));
                }
            }
        }
    }
    parts.sort_by(|a, b| a.mesh.cmp(&b.mesh));
    parts.dedup_by(|a, b| a.mesh == b.mesh);
    parts
}

#[test]
#[ignore]
fn renders_the_c5_dwarf_set_solid_with_its_shine_pass() {
    let Some(mut client) = client() else { return };
    let parts = dwarf_parts(&mut client);
    assert!(parts.len() >= 4, "{parts:?}");
    let timeenv = TimeEnv::load(&mut client, 0).unwrap();
    let Ok(mut headless) = Headless::new(SIZE, SIZE) else { return };
    let scene = Scene { zones: vec![ZoneLighting::default()], ..Default::default() };
    let mut bank = TextureBank::new();
    headless.upload(&scene, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
    let mut instances = Vec::new();
    let mut lowest = f32::MAX;
    let mut highest = f32::MIN;
    for part in &parts {
        let mesh = load_part(&mut client, part).unwrap();
        for vertex in &mesh.vertices {
            let z = mesh.local.transform_point3(Vec3::from(vertex.position)).z;
            lowest = lowest.min(z);
            highest = highest.max(z);
        }
        let mut source = BankSource { bank: &mut bank, catalog: client.catalog() };
        let id = headless.renderer_mut().upload_character(&mesh, &mut source);
        let (center, radius) = killer_assets::scene::transform::sphere(mesh.bounding_sphere, mesh.local);
        instances.push(CharacterInstance { overlay: [0.0; 4], actor_id: 1, actor_position: glam::Vec3::ZERO, collision_height: 22.0, shadow_caster: true, light_sphere: center.extend(radius).to_array(), mesh: id, world: mesh.local, palette: vec![Mat4::IDENTITY; mesh.bones.len()], zone: 0, ambient: [0; 3], sun_affect: true, lights: Vec::new(), pawn_lights: Vec::new() });
    }
    headless.renderer_mut().set_instances(&instances);
    let middle = (lowest + highest) * 0.5;
    let camera = Camera { eye: vec3(0.0, -(highest - lowest) * 1.6, middle), target: vec3(0.0, 0.0, middle), fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
    let rgba = headless.render(&camera, &Environment::from_time(&timeenv, 12.0)).unwrap();
    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/dwarf-c5-set.png");
    write_png(&out, SIZE, SIZE, &rgba).unwrap();
    let covered = rgba.as_chunks::<4>().0.iter().filter(|px| px[..3] != CLEAR_RGBA[..3]).count();
    assert!(covered > (SIZE * SIZE / 20) as usize, "{covered}");
}

#[test]
#[ignore]
fn dxt3_textures_sample_their_color_on_the_gpu() {
    use killer_assets::scene::{DrawKind, MeshDraw, Pass, Vertex, texture_material};
    let Some(mut client) = client() else { return };
    let Ok(mut headless) = Headless::new(64, 64) else { return };
    let timeenv = TimeEnv::load(&mut client, 0).unwrap();
    let mut bank = TextureBank::new();
    for path in ["c5MDwarf.Glove.MDwarf_m004_t1005_g_sp", "c5MDwarf.UpBody.MDwarf_m004_t1005_u_ori"] {
        let v = |x: f32, z: f32, u: f32, w: f32| Vertex { position: [x, 0.0, z], normal: [0.0, -1.0, 0.0], uv: [u, w], mask_uv: [0.0; 2], color: [0.5, 0.5, 0.5, 1.0] };
        let vertices = vec![v(-100.0, -100.0, 0.0, 0.0), v(100.0, -100.0, 1.0, 0.0), v(100.0, 100.0, 1.0, 1.0), v(-100.0, 100.0, 0.0, 1.0)];
        let draw = MeshDraw { instances: Vec::new(), vertices, indices: vec![0, 2, 1, 0, 3, 2], material: texture_material(path), mask: None, kind: DrawKind::StaticMesh, sun_affect: false, pass: Pass::Opaque, zone: 0, lightmap: None, modulate_2x: true, sector: None, fade: None, tint: None, actor: None };
        let scene = Scene { zones: vec![ZoneLighting { sun_affect: false, ..ZoneLighting::default() }], draws: vec![draw], ..Default::default() };
        headless.upload(&scene, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        let camera = Camera { eye: vec3(0.0, -300.0, 0.0), target: vec3(0.0, 0.0, 0.0), fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
        let rgba = headless.render(&camera, &Environment::from_time(&timeenv, 12.0)).unwrap();
        let mean: Vec<f32> = (0..3).map(|k| rgba.as_chunks::<4>().0.iter().skip(20 * 64 + 20).take(24).map(|px| f32::from(px[k])).sum::<f32>() / 24.0).collect();
        eprintln!("{path}: {mean:?}");
        assert!(mean[0] < 200.0, "{path}: {mean:?}");
    }
}
