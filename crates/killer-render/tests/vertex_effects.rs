use glam::Vec3;
use killer_assets::{client::Client, options::ClientOptions, scene::{Scene, emitters}, textures::{BankSource, TextureBank, write_png}};
use killer_render::{camera::Camera, environment::Environment, headless::Headless, particles::{EmptyWorld, ParticleSystem}};
use std::path::PathBuf;

#[test]
#[ignore]
fn original_vertex_effects_animate_and_reach_gpu() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let options = ClientOptions::load(&mut client).unwrap();
    let file = client.read_system_file("LineageEffect.u").unwrap();
    let package = ue2_package::Package::parse(&file).unwrap();
    let mut gpu = Headless::new(512, 512).unwrap();
    let camera = Camera { eye: Vec3::new(220.0, -350.0, 200.0), target: Vec3::new(0.0, 0.0, 25.0), fov_x_deg: 60.0, near: 1.0, far: 10000.0 };
    let environment = Environment::default();
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/vertex-effects");
    std::fs::create_dir_all(&output).unwrap();
    let mut count = 0;
    for export in package.exports().iter().filter(|e| package.class_name(e) == "Class") {
        let name = package.name(export.name);
        let Some(actor) = client.emitter_class(&format!("LineageEffect.{name}")) else { continue };
        if !actor.sprites.iter().any(|e| e.vertex_mesh.is_some()) { continue; }
        let mut asset = emitters::effect_asset(&mut client, &actor, &options);
        for (definition, part) in actor.sprites.iter().zip(&mut asset.parts) {
            if definition.vertex_mesh.is_none() { part.draws.clear(); }
            else {
                assert!(part.vertex_mesh.is_some(), "{name}");
                assert!(!part.draws.is_empty(), "{name}");
                assert!(part.draws.iter().all(|d| !d.indices.is_empty()), "{name}");
                assert!(part.draws.iter().all(|d| d.material.texture_paths().iter().any(|p| !p.is_empty())), "{name}: material original descartado");
            }
        }
        let mut system = ParticleSystem::new(&actor, 1);
        let mut animated = false;
        for _ in 0..20 {
            system.tick(1.0 / 30.0, true, &asset, &mut EmptyWorld);
            for (state, part) in system.states.iter().zip(&asset.parts) {
                if part.vertex_mesh.is_none() { continue; }
                animated |= state.particles[..state.active].iter().any(|p| p.alive() && p.vertex_frame > 0.0);
            }
        }
        assert!(animated, "{name}: quadros não avançam");
        let mut bank = TextureBank::new();
        gpu.upload(&Scene::default(), &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        let empty = gpu.render(&camera, &environment).unwrap();
        gpu.upload(&Scene { emitters: vec![asset], ..Default::default() }, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        gpu.renderer_mut().set_particle_delta(Some(0.0));
        for _ in 0..12 { gpu.renderer_mut().tick_particles(1.0 / 30.0, camera.eye); }
        let first = gpu.render(&camera, &environment).unwrap();
        for _ in 0..8 { gpu.renderer_mut().tick_particles(1.0 / 30.0, camera.eye); }
        let second = gpu.render(&camera, &environment).unwrap();
        let changed = [&first, &second].into_iter().map(|image| image.as_chunks::<4>().0.iter().zip(empty.as_chunks::<4>().0.iter()).filter(|(a,b)| a != b).count()).max().unwrap();
        assert!(changed > 0, "{name}: nenhum pixel de VertMesh");
        assert_ne!(first, second, "{name}: imagem congelada");
        write_png(&output.join(format!("{name}-040.png")), 512, 512, &first).unwrap();
        write_png(&output.join(format!("{name}-067.png")), 512, 512, &second).unwrap();
        eprintln!("{name}: {changed} pixels, animação e GPU aprovadas");
        count += 1;
        client.catalog().release_files();
    }
    assert_eq!(count, 7);
}
