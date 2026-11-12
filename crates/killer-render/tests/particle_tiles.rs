use killer_assets::{client::Client, options::ClientOptions, scene::emitters};
use killer_render::particles::{EmptyWorld, ParticleSystem};
use std::{collections::BTreeSet, path::PathBuf};

#[test]
#[ignore]
fn all_143_tiles_resolve_particle_meshes_textures_and_simulate() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR obrigatório"));
    let mut client = Client::open(&root).unwrap();
    let options = ClientOptions::load(&mut client).unwrap();
    let mut textures = BTreeSet::new();
    let mut missing = BTreeSet::new();
    let mut tiles = 0;
    let mut parts = 0;
    let mut alive = 0;
    let mut no_texture = 0;
    for x in 15..=26 {
        for y in 10..=26 {
            let tile = format!("{x}_{y}");
            if client.level_path(&tile).is_err() { continue; }
            let loaded = client.load_level(&tile).unwrap();
            let assets = emitters::build(&mut client, &loaded.level, &options);
            for asset in &assets {
                for (definition, part) in asset.definition.sprites.iter().zip(&asset.parts) {
                    if definition.disabled { continue; }
                    let textured = definition.class == "SpriteEmitter" || definition.beam.is_some();
                    no_texture += usize::from(textured && definition.texture.is_none());
                    if (textured && definition.texture.is_some()) || definition.mesh.as_ref().is_some_and(|m| m.mesh.is_some()) {
                        assert!(!part.draws.is_empty(), "{tile}: {} {:?}", definition.class, definition.mesh);
                        parts += 1;
                    }
                    for draw in &part.draws {
                        for path in draw.material.texture_paths() {
                            if !path.is_empty() && textures.insert(path.to_owned()) && client.catalog().texture_raw(path).is_none() {
                                assert_eq!(client.catalog().object_class(path).as_deref(), Some("WetTexture"), "asset não resolvido: {path}");
                                missing.insert(path.to_owned());
                            }
                        }
                    }
                }
                let mut system = ParticleSystem::new(&asset.definition, 1);
                for _ in 0..150 { system.tick(1.0 / 30.0, true, asset, &mut EmptyWorld); }
                for state in &system.states {
                    for p in state.particles[..state.active].iter().filter(|p| p.alive()) {
                        assert!(p.location.is_finite() && p.velocity.is_finite() && p.size.is_finite(), "{tile}: {p:?}");
                        assert!(p.beam.iter().all(|v| v.is_finite()), "{tile}");
                        alive += 1;
                    }
                }
            }
            tiles += 1;
            client.catalog().release_files();
        }
    }
    eprintln!("tiles={tiles}, drawable_parts={parts}, no_texture={no_texture}, textures={}, alive_at_5s={alive}, missing={missing:?}", textures.len());
    assert_eq!(tiles, 143);
    assert_eq!(parts + no_texture, 5588 + 411 + 70);
    assert!(missing.is_empty(), "texturas sem resolução: {missing:?}");
}
