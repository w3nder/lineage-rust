use glam::Vec3;
use killer_assets::scene::actor_lighting::{Bounds, LightMap, Query, ambient_color, box_leaves};
use killer_assets::client::Client;
use killer_assets::scene::lights::{box_leaf_candidates, leaf_candidates, point_lights, priority, select};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[test]
#[ignore]
fn update_render_data_104ffae0_all_zones_and_lobby_spots() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR obrigatório"));
    let mut client = Client::open(&root).unwrap();
    let mut maps = vec!["lobby".to_owned()];
    for x in 15..=26 {
        for y in 10..=26 {
            let name = format!("{x}_{y}");
            if client.level_path(&name).is_ok() { maps.push(name); }
        }
    }
    assert_eq!(maps.len(), 144);
    let mut checked_zones = 0;
    let mut checked_positions = 0;
    let mut checked_meshes = 0;
    let mut missing_meshes = std::collections::BTreeMap::new();
    for name in &maps {
        let loaded = client.load_level(name).unwrap();
        let level = &loaded.level;
        let zones = killer_assets::scene::zone_lighting(level);
        let map = LightMap::new(level, &point_lights(&level.lights));
        let mut mesh_bounds = HashMap::new();
        for actor in level.actors.iter().filter(|a| a.static_mesh.is_some()) {
            let path = actor.static_mesh.as_ref().unwrap();
            let bbox = *mesh_bounds.entry(path).or_insert_with(|| client.catalog().static_mesh(path).map(|mesh| mesh.mesh.bounding_box));
            let Some(bbox) = bbox else {
                let package = path.split('.').next().unwrap();
                missing_meshes.insert(format!("{name}: {path}"), format!("{:?}", client.catalog().package_status(package)));
                continue;
            };
            let world = killer_assets::scene::transform::placement_matrix(actor.placement.location, actor.placement.rotation, actor.scale);
            let bounds = Bounds::transformed(bbox, world);
            assert!(bounds.min.is_finite() && bounds.max.is_finite(), "{name}: {path}");
            let (vector, sun) = map.zone_ambient(bounds);
            let glow = killer_assets::scene::actor_lighting::actor_glow(level, actor);
            let actual = map.sample(Query { position: Vec3::from(actor.placement.location), center: bounds.center(), radius: bounds.extent().length(), bounds }, glow, 0.0);
            assert_eq!(actual.ambient, ambient_color(vector, glow, 0.0));
            assert_eq!(actual.sun_affect, sun);
            checked_meshes += 1;
        }
        for zone in &zones {
            let vector = zone.ambient_vector.unwrap_or([0.0; 3]);
            assert!(vector.iter().all(|c| c.is_finite()));
            for glow in 0..255u8 {
                let reference = vector.map(|v| (((f64::from(v.max(0.0)) + f64::from(glow) / 255.0) as f32 * 255.0).floor() as i32).clamp(0, 255) as u8);
                assert_eq!(ambient_color(Vec3::from(vector).max(Vec3::ZERO), glow, 0.0), reference, "{name}");
            }
            checked_zones += 1;
        }
        let mut positions: Vec<Vec3> = level.actors.iter().map(|a| Vec3::from(a.placement.location)).collect();
        if name == "lobby" {
            let logon = client.logongrp().unwrap();
            positions.extend(logon.slots.iter().chain(&logon.stage).chain(std::iter::once(&logon.center)).map(|spot| Vec3::new(spot.x, spot.y, spot.z)));
            let lobby_zone = level.zone_infos.get("ZoneInfo2").unwrap();
            assert!(!lobby_zone.sun_affect);
            assert_eq!(ambient_color(Vec3::from(lobby_zone.ambient_vector.unwrap()), 0, 0.0), [70; 3]);
        }
        for position in positions {
            let bounds = Bounds { min: position - Vec3::ONE, max: position + Vec3::ONE };
            let leaves = box_leaves(&level.bsp_tree, bounds);
            let mut vector = Vec3::ZERO;
            let mut sun = false;
            for leaf in leaves {
                let leaf = level.bsp_render.leaves.get(leaf).expect("índice de folha válido");
                let zone = &zones[leaf.zone as usize];
                vector = vector.max(Vec3::from(zone.ambient_vector.unwrap_or([0.0; 3])));
                sun |= zone.sun_affect;
            }
            let sample = map.sample(Query { position, center: position, radius: 1.0, bounds }, 0, 0.0);
            assert_eq!(sample.ambient, ambient_color(vector, 0, 0.0), "{name} {position}");
            assert_eq!(sample.sun_affect, sun, "{name} {position}");
            checked_positions += 1;
        }
        client.catalog().release_files();
    }
    eprintln!("{} mapas; {checked_zones} zonas × 255 valores constantes de AmbientGlow; {checked_positions} posições incluindo os 26 spots do lobby; {checked_meshes} static meshes com FBox serializada", maps.len());
    for (mesh, reason) in &missing_meshes { eprintln!("malha não verificada: {mesh} ({reason})"); }
}

#[test]
#[ignore]
fn inspect_character_lights_in_giran_and_lobby() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR obrigatório"));
    let mut client = Client::open(&root).unwrap();
    let table = client.logongrp().unwrap();
    let center = Vec3::new(table.center.x, table.center.y, table.center.z);
    for (tile, position) in [("22_22", Vec3::new(83386.0, 145353.0, -3394.0)), ("lobby", center)] {
        let loaded = client.load_level(tile).unwrap();
        let level = &loaded.level;
        let lights = point_lights(&level.lights);
        let candidates = leaf_candidates(level, &lights, position, 64.0);
        let selected = select(&candidates, position, 64.0, false, 4, false);
        let unfiltered = select(&lights, position, 64.0, false, 4, false);
        eprintln!("{tile} {position:?} zona {}: {} candidatas, {} selecionadas, {} sem folhas", level.bsp_tree.zone_at(position.to_array()), candidates.len(), selected.len(), unfiltered.len());
        for light in &unfiltered {
            let source = &level.lights[light.source];
            eprintln!("{}: tipo {} efeito {} special {} brilho {} posição {:?} R {} dir {:?} cor {:?} folha {}", source.name, source.light_type, source.effect, source.special_lit, source.brightness, source.location, source.radius, source.direction, light.color, candidates.contains(light));
        }
        assert!(!unfiltered.is_empty());
    }
}

#[test]
#[ignore]
fn fun_10649da0_and_fun_10648630_all_client_tiles() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR obrigatório"));
    let mut client = Client::open(&root).unwrap();
    let (mut tiles, mut lights, mut coronas, mut meshes, mut changed) = (0, 0, 0, 0, 0);
    let mut missing = 0;
    for x in 15..=26 {
        for y in 10..=26 {
            let tile = format!("{x}_{y}");
            if client.level_path(&tile).is_err() { continue; }
            let loaded = client.load_level(&tile).unwrap();
            let level = &loaded.level;
            assert!(level.bsp_render.complete, "{tile}: Model incompleto");
            let point = point_lights(&level.lights);
            for leaf in &level.bsp_render.leaves {
                if let Ok(start) = usize::try_from(leaf.permeating) {
                    assert!(start < level.bsp_render.lights.len(), "{tile}: iPermeating");
                    assert!(level.bsp_render.lights[start..].iter().any(Option::is_none), "{tile}: run sem terminador");
                }
            }
            let mut spheres = HashMap::new();
            for actor in level.actors.iter().filter(|a| a.static_mesh.is_some() && a.lighting.is_empty()) {
                meshes += 1;
                let path = actor.static_mesh.as_ref().unwrap();
                let sphere = spheres.entry(path).or_insert_with(|| client.catalog().static_mesh(path).map(|mesh| mesh.mesh.bounding_sphere));
                let Some(sphere) = sphere else { missing += 1; continue };
                let world = killer_assets::scene::transform::placement_matrix(actor.placement.location, actor.placement.rotation, actor.scale);
                let (center, radius) = killer_assets::scene::transform::sphere(*sphere, world);
                assert!(radius.is_finite() && radius >= 0.0, "{tile}: {path}");
                let candidates = leaf_candidates(level, &point, center, radius);
                let zone = level.bsp_tree.zone_at(actor.placement.location);
                let sunlight = level.zones.get(usize::from(zone)).and_then(|zone| zone.actor.as_ref()).and_then(|name| level.zone_infos.get(name)).is_none_or(|info| info.sun_affect);
                let limit = usize::from(actor.max_lights).min(8).saturating_sub(usize::from(sunlight));
                let selected = select(&candidates, center, radius, actor.special_lit, usize::from(actor.max_lights), sunlight);
                assert!(selected.len() <= limit);
                assert!(selected.iter().all(|l| candidates.contains(l)));
                assert!(selected.windows(2).all(|p| priority(&p[0], center, radius) >= priority(&p[1], center, radius)));
                if point.iter().filter(|l| l.position.distance(center) < l.radius).count() != selected.len() { changed += 1; }
            }
            if tile == "22_22" {
                let named: HashSet<_> = level.bsp_render.lights.iter().flatten().collect();
                assert_eq!(named.len(), 87);
                let solar = level.bsp_render.leaves.iter().filter(|leaf| {
                    usize::try_from(leaf.permeating).ok().is_some_and(|start| level.bsp_render.lights[start..].iter().take_while(|n| n.is_some()).flatten().any(|n| n == "NMovableSunLight0"))
                }).count();
                assert_eq!(solar, 1054);
            }
            lights += level.lights.len();
            coronas += level.lights.iter().filter(|l| l.corona.is_some()).count();
            tiles += 1;
            client.catalog().release_files();
        }
    }
    eprintln!("{tiles} tiles; {lights} luzes; {coronas} coronas; {meshes} meshes sem ColorStream; {missing} malhas não resolvidas; {changed} conjuntos diferentes da seleção antiga usando esfera serializada");
    assert_eq!(tiles, 143);
    assert!(coronas > 0);
    assert!(changed > 0);
}

#[test]
#[ignore]
fn a_box_collects_only_leaf_lights_its_circumscribed_sphere_also_collects() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR obrigatório"));
    let mut client = Client::open(&root).unwrap();
    let loaded = client.load_level("22_22").unwrap();
    let level = &loaded.level;
    let lights = point_lights(&level.lights);
    let (mut box_total, mut sphere_total) = (0, 0);
    for position in [Vec3::new(83386.0, 145353.0, -3394.0), Vec3::new(83400.0, 148000.0, -3400.0), Vec3::new(80000.0, 146000.0, -3300.0)] {
        for half in [64.0f32, 300.0, 1500.0] {
            let extents = Vec3::splat(half);
            let boxed = box_leaf_candidates(level, &lights, position, extents);
            let sphere = leaf_candidates(level, &lights, position, extents.length());
            assert!(boxed.iter().all(|light| sphere.contains(light)));
            box_total += boxed.len();
            sphere_total += sphere.len();
        }
    }
    assert!(box_total <= sphere_total);
    eprintln!("caixa {box_total} esfera {sphere_total}");
}
