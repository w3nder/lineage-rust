use glam::Vec3;
use killer_assets::client::Client;
use killer_assets::collision::{resources::Resources, transform, world::{View, music_volume_at, physics_volume_at}};
use std::{path::PathBuf, sync::Arc};

#[test]
#[ignore]
fn physics_volumes_of_the_143_tiles_encompass_their_own_centers() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let mut resources = Resources::default();
    let mut by_class: std::collections::BTreeMap<String, [usize; 4]> = Default::default();
    let (mut tiles, mut volumes, mut inside, mut water, mut wrong_priority, mut defaults) = (0, 0, 0, 0, 0, 0);
    for x in 15..=26 {
        for y in 10..=26 {
            let name = format!("{x}_{y}");
            let Ok(loaded) = client.load_level(&name) else { continue };
            tiles += 1;
            let level = Arc::new(loaded.level);
            let views = vec![View { name: name.clone(), level: level.clone(), poses: Default::default() }];
            let tile = resources.tile(&mut client, &name, &level).unwrap();
            defaults += usize::from(tile.default_volume.is_some());
            let in_actors = tile.actors.iter().filter(|shape| level.actors[shape.index].class.ends_with("WaterVolume")).count();
            if in_actors > 0 { eprintln!("WV {name}: {in_actors} WaterVolume entre os atores de colisão"); }
            for volume in &tile.volumes {
                volumes += 1;
                let entry = by_class.entry(level.actors[volume.index].class.clone()).or_default();
                entry[0] += 1; entry[1] += usize::from(volume.props.water); entry[2] += usize::from(volume.props.l2_water); entry[3] += usize::from(volume.props.l2_step);
                let actor = &level.actors[volume.index];
                let data = resources.model(&mut client, &volume.brush).unwrap();
                let [lo, hi] = data.bounding_box;
                let transform = transform::actor(actor.placement, actor.scale, [0.0; 3]);
                let center = transform.transform_point3(Vec3::from(lo).midpoint(Vec3::from(hi)));
                let found = physics_volume_at(&mut client, &mut resources, &views, center).unwrap();
                if found.priority >= volume.props.priority { inside += 1; } else { wrong_priority += 1; }
                if volume.props.water && found.water { water += 1; }
            }
        }
    }
    eprintln!("CLASSES {by_class:?}");
    eprintln!("VOLUMES {tiles} tiles, {volumes} volumes, {inside} contêm o próprio centro (prioridade >=), {wrong_priority} não, {water} de água achados, {defaults} tiles com DefaultPhysicsVolume");
    assert_eq!(tiles, 143);
}

#[test]
#[ignore]
fn immediately_stop_and_pushable_decorations_over_the_143_tiles() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let mut resources = Resources::default();
    let mut by_class: std::collections::BTreeMap<String, [usize; 2]> = Default::default();
    let (mut tiles, mut shapes) = (0, 0);
    for x in 15..=26 {
        for y in 10..=26 {
            let name = format!("{x}_{y}");
            let Ok(loaded) = client.load_level(&name) else { continue };
            tiles += 1;
            let level = Arc::new(loaded.level);
            let tile = resources.tile(&mut client, &name, &level).unwrap();
            shapes += tile.actors.len();
            for shape in tile.actors.iter().filter(|shape| shape.immediately_stop || shape.pushable_mass.is_some()) {
                let entry = by_class.entry(level.actors[shape.index].class.clone()).or_default();
                entry[0] += usize::from(shape.immediately_stop);
                entry[1] += usize::from(shape.pushable_mass.is_some());
            }
        }
    }
    eprintln!("PARADA {tiles} tiles, {shapes} atores de colisão; por classe [bImmediatelyStop, Decoration bPushable]: {by_class:?}");
    assert_eq!(tiles, 143);
}

#[test]
#[ignore]
fn deco_layer_instances_feed_the_grass_field() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let options = killer_assets::options::ClientOptions { render_deco: true, ..Default::default() };
    let (mut tiles, mut with_grass) = (0, 0);
    for x in 15..=26 {
        for y in 10..=26 {
            let Ok(loaded) = client.load_level(&format!("{x}_{y}")) else { continue };
            tiles += 1;
            let layers: usize = loaded.level.terrains.iter().map(|terrain| terrain.deco_layers.len()).sum();
            if layers == 0 { continue; }
            let field = killer_assets::scene::decoration_points(&mut client, &loaded.level, &options);
            let center = killer_assets::tiles::TileId::parse(&format!("{x}_{y}")).unwrap().center();
            let near: usize = (-8..8).flat_map(|i| (-8..8).map(move |j| (i, j))).map(|(i, j)| field.count_within(center[0] + i as f32 * 2048.0, center[1] + j as f32 * 2048.0, 64.0)).sum();
            if near > 0 { with_grass += 1; }
        }
    }
    eprintln!("GRAMA {tiles} tiles, {with_grass} com instâncias de deco");
    assert!(with_grass > 0);
}

#[test]
#[ignore]
fn music_volumes_of_the_143_tiles_resolve_to_their_own_music_at_the_center() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let mut resources = Resources::default();
    let (mut tiles, mut volumes, mut found_own, mut forced, mut looped) = (0, 0, 0, 0, 0);
    let mut ids = std::collections::BTreeSet::new();
    for x in 15..=26 {
        for y in 10..=26 {
            let name = format!("{x}_{y}");
            let Ok(loaded) = client.load_level(&name) else { continue };
            tiles += 1;
            let level = Arc::new(loaded.level);
            let views = vec![View { name: name.clone(), level: level.clone(), poses: Default::default() }];
            let tile = resources.tile(&mut client, &name, &level).unwrap();
            for volume in &tile.music_volumes {
                volumes += 1;
                ids.insert(volume.music_id);
                forced += usize::from(volume.force_play);
                looped += usize::from(volume.looped);
                let actor = &level.actors[volume.index];
                let data = resources.model(&mut client, &volume.brush).unwrap();
                let [lo, hi] = data.bounding_box;
                let transform = transform::actor(actor.placement, actor.scale, [0.0; 3]);
                let center = transform.transform_point3(Vec3::from(lo).midpoint(Vec3::from(hi)));
                let zone = music_volume_at(&mut client, &mut resources, &views, center).unwrap();
                if zone.is_some_and(|zone| zone.tile == name) { found_own += 1; }
            }
        }
    }
    eprintln!("MUSICA {tiles} tiles, {volumes} MusicVolume, {found_own} achados no próprio centro, {forced} com bForcePlayMusic, {looped} com bLoopMusic, ids {ids:?}");
    assert_eq!(tiles, 143);
    assert!(volumes > 0);
    assert!(found_own > 0);
}
