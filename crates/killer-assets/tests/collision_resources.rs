use killer_assets::client::Client;
use killer_assets::collision::resources::{Primitive, Resources};
use std::{collections::BTreeMap, path::PathBuf};

#[test]
#[ignore]
fn every_collision_primitive_of_every_tile_resolves_its_resources() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let only = std::env::var("KILLER_TILES").ok();
    let mut client = Client::open(&root).unwrap();
    let mut failures: BTreeMap<String, (String, usize)> = BTreeMap::new();
    let mut checked = 0;
    for x in 15..=26 {
        for y in 10..=26 {
            let tile = format!("{x}_{y}");
            if only.as_deref().is_some_and(|only| !only.split(',').any(|t| t == tile)) {
                continue;
            }
            let Ok(loaded) = client.load_level(&tile) else { continue };
            let level = std::sync::Arc::new(loaded.level);
            let mut resources = Resources::default();
            let built = match resources.tile(&mut client, &tile, &level) {
                Ok(built) => built,
                Err(error) => {
                    failures.entry(error).or_insert((tile.clone(), 0)).1 += 1;
                    continue;
                }
            };
            let mut seen = std::collections::BTreeSet::new();
            for shape in &built.actors {
                let result = match &shape.primitive {
                    Primitive::Mesh(path) if seen.insert(path.to_lowercase()) => resources.bounds(&mut client, path).and_then(|_| resources.mesh(&mut client, path).map(|_| ())),
                    Primitive::Model(path) if seen.insert(path.to_lowercase()) => resources.model(&mut client, path).map(|_| ()),
                    _ => continue,
                };
                checked += 1;
                if let Err(error) = result {
                    failures.entry(error).or_insert((tile.clone(), 0)).1 += 1;
                }
            }
        }
    }
    for (error, (tile, count)) in &failures {
        eprintln!("{count}x (primeiro em {tile}): {error}");
    }
    assert!(failures.is_empty(), "{} recursos de colisão falham de {checked}", failures.len());
}
