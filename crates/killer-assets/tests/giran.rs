use killer_assets::client::Client;
use std::path::PathBuf;

fn test_client() -> Option<Client> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    Client::open(&root).ok()
}

#[test]
#[ignore]
fn giran_level_counts() {
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    assert_eq!(loaded.name, "22_22");
    assert_eq!(loaded.level.actors.len(), 2395);
    assert_eq!(loaded.level.bsp.len(), 2113);
    let meshes: std::collections::BTreeSet<_> =
        loaded.level.actors.iter().filter_map(|a| a.static_mesh.as_deref()).collect();
    assert_eq!(meshes.len(), 288);
    assert_eq!(loaded.level.terrains.len(), 1);
    let terrain = &loaded.level.terrains[0];
    let heightmap = client.catalog().heightmap(&terrain.heightmap).unwrap();
    assert_eq!((heightmap.width, heightmap.height), (256, 256));
}

#[test]
#[ignore]
fn giran_scene_has_terrain_bsp_and_props() {
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let scene = killer_assets::scene::build_scene(&mut client, &loaded.level);
    assert!(scene.draws.len() > 300, "{}", scene.draws.len());
    let triangles: usize = scene.draws.iter().map(|d| d.indices.len() / 3).sum();
    assert!(triangles > 100_000, "{triangles}");
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for terrain in &loaded.level.terrains {
        let zone = killer_assets::scene::terrain_zone(&loaded.level, terrain);
        for draw in killer_assets::scene::terrain::build(&mut client, terrain, zone, killer_assets::scene::sun_affect(&scene.zones, zone), None, &[], &killer_assets::scene::terrain::TerrainSeam::NONE) {
            for v in &draw.vertices {
                for axis in 0..3 {
                    min[axis] = min[axis].min(v.position[axis]);
                    max[axis] = max[axis].max(v.position[axis]);
                }
            }
        }
    }
    for draw in killer_assets::scene::props::build(&mut client, &loaded.level, &killer_assets::options::ClientOptions::default().clipping) {
        for v in &draw.vertices {
            for axis in 0..3 {
                min[axis] = min[axis].min(v.position[axis]);
                max[axis] = max[axis].max(v.position[axis]);
            }
        }
    }
    assert!(min[0] >= 65536.0 - 1.0 && max[0] <= 98304.0 + 1.0, "{min:?} {max:?}");
}

#[test]
#[ignore]
fn giran_textures_have_mip_chains() {
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let terrain = &loaded.level.terrains[0];
    let layer = &terrain.layers[0];
    let texture_path = killer_assets::scene::first_texture_path(&killer_assets::scene::resolve_material(&mut client, &layer.material));
    let raw = client.catalog().texture_raw(&texture_path).unwrap();
    assert!(raw.mips.len() > 1, "{}", raw.mips.len());
    assert!(
        matches!(
            raw.format,
            ue2_assets::TextureFormat::Dxt1
                | ue2_assets::TextureFormat::Dxt3
                | ue2_assets::TextureFormat::Dxt5
                | ue2_assets::TextureFormat::Rgba8
        ),
        "{:?}",
        raw.format
    );
    for pair in raw.mips.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        assert_eq!(b.width, (a.width / 2).max(1), "{} -> {}", a.width, b.width);
        assert_eq!(b.height, (a.height / 2).max(1), "{} -> {}", a.height, b.height);
    }
}

#[test]
#[ignore]
fn giran_lighting_has_sun_and_ambient() {
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    println!(
        "luz: sol {:.4} dir {:?} ambiente {:.4}",
        loaded.lighting.sun_brightness, loaded.lighting.sun_dir, loaded.lighting.ambient
    );
    assert!((loaded.lighting.ambient - 50.0 / 255.0).abs() < 1e-3, "{}", loaded.lighting.ambient);
    assert!((loaded.lighting.sun_brightness - 70.0 / 255.0).abs() < 1e-3);
    assert!(loaded.lighting.sun_dir.z < 0.0, "{:?}", loaded.lighting.sun_dir);
}

#[test]
fn missing_tile_is_not_found() {
    let dir = std::env::temp_dir().join("killer-assets-empty");
    std::fs::create_dir_all(dir.join("maps")).unwrap();
    let mut client = Client::open(&dir).unwrap();
    assert!(matches!(client.load_level("99_99"), Err(killer_assets::client::AssetError::NotFound(_))));
}

#[test]
#[ignore]
fn giran_zones() {
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let level = &loaded.level;
    assert!(level.zones.len() >= 18, "{}", level.zones.len());
    assert!(level.bsp_tree.nodes.len() >= level.bsp.len());
    assert!(level.bsp.iter().all(|p| usize::from(p.zone) < level.zones.len()));
    let terrain = level.zone_infos.get("ZoneInfo3").unwrap();
    assert_eq!(terrain.ambient_brightness, 50);
    assert!(terrain.sun_affect);
    assert!(!terrain.is_sky);
    assert_eq!(terrain.distance_fog.map(|f| f.end), Some(15000.0));
    assert!(level.zone_infos.values().any(|z| !z.sun_affect));
    assert!(level.zone_infos.values().any(|z| z.is_sky && z.location != [0.0; 3]));
    let start = level.actors.iter().find(|a| a.name == "PlayerStart5").unwrap();
    let hall = level.bsp_tree.zone_at(start.placement.location);
    let plaza = level.bsp_tree.zone_at([83400.0, 148000.0, -3400.0]);
    println!(
        "PlayerStart5 {:?} -> zona {hall} {:?}; praça -> zona {plaza} {:?}",
        start.placement.location,
        level.zones[usize::from(hall)].actor,
        level.zones[usize::from(plaza)].actor
    );
    assert_ne!(hall, plaza);
    assert_eq!(level.zones[usize::from(plaza)].actor.as_deref(), Some("ZoneInfo3"));
    let hall_info = &level.zone_infos[level.zones[usize::from(hall)].actor.as_deref().unwrap()];
    assert!(!hall_info.sun_affect);
    assert!(hall_info.ambient_brightness > 0);
}

#[test]
#[ignore]
fn giran_scene_zones() {
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let scene = killer_assets::scene::build_scene(&mut client, &loaded.level);
    assert_eq!(scene.zones.len(), 20);
    let sky: Vec<usize> = scene.zones.iter().enumerate().filter(|(_, z)| z.is_sky).map(|(i, _)| i).collect();
    assert_eq!(sky.len(), 1, "{sky:?}");
    let sky = sky[0] as u8;
    assert_ne!(scene.zones[usize::from(sky)].sky_location, [0.0; 3]);
    let terrain = &scene.zones[1];
    assert_eq!(terrain.fog.map(|f| (f.start, f.end)), Some((3000.0, 15000.0)));
    assert!(terrain.sun_affect);
    assert_eq!(terrain.ambient_brightness, 50);
    assert!(!scene.zones[0].is_sky && scene.zones[0].fog.is_none());
    assert!(scene.draws.iter().all(|d| d.zone != sky), "com SharedSky o céu do tile não é desenhado");
    let own_sky = killer_assets::scene::build_scene_with(&mut client, &loaded.level, &killer_assets::options::ClientOptions { shared_sky: false, ..killer_assets::options::ClientOptions::default() }, killer_assets::scene::Neighbours::default());
    let sky_draws: Vec<_> = own_sky.draws.iter().filter(|d| d.zone == sky).collect();
    assert!(!sky_draws.is_empty());
    assert_eq!(own_sky.sky_bodies.len(), 2);
    let vertices: usize = sky_draws.iter().map(|d| d.vertices.len()).sum();
    let outside = sky_draws.iter().flat_map(|d| &d.vertices).filter(|v| v.position[0] < 65536.0 || v.position[0] > 98304.0).count();
    println!("céu: zona {sky}, {} draws, {vertices} vértices, {outside} fora do tile em X", sky_draws.len());
    assert_eq!(outside, vertices);
    let world_zones: std::collections::BTreeSet<u8> = scene.draws.iter().filter(|d| d.zone != sky).map(|d| d.zone).collect();
    println!("zonas com draws: {world_zones:?}");
    assert!(world_zones.len() > 1, "{world_zones:?}");
    let terrain_draws = scene.draws.iter().filter(|d| d.mask.is_some() || matches!(d.pass, killer_assets::scene::Pass::Layer(_))).count();
    assert!(terrain_draws > 0);
    assert!(scene.draws.iter().filter(|d| matches!(d.pass, killer_assets::scene::Pass::Layer(_))).all(|d| d.zone == 1));
}

#[test]
#[ignore]
fn giran_hall_pennants_blend_by_alpha_and_are_lit_by_the_hall_lights() {
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let level = &loaded.level;
    let hall = level.bsp_tree.zone_at([83386.0, 145353.0, -3333.0]);
    let zones = killer_assets::scene::zone_lighting(level);
    assert!(!killer_assets::scene::sun_affect(&zones, hall));
    let in_hall = |a: &&ue2_level::Actor| level.bsp_tree.zone_at(a.placement.location) == hall;
    let pennants: Vec<_> = level.actors.iter().filter(|a| a.static_mesh.as_deref() == Some("Agit_B_s.9_0")).filter(in_hall).collect();
    assert_eq!(pennants.len(), 8);
    assert!(pennants.iter().all(|a| !a.sun_affect && a.lighting.is_empty()));
    let stored: Vec<_> = level.actors.iter().filter(|a| a.static_mesh.is_some()).collect();
    let lit = stored.iter().filter(|a| a.sun_affect).count();
    assert_eq!((stored.len(), lit), (1942, 1534));

    let lights = killer_assets::scene::lights::point_lights(&level.lights);
    assert_eq!(level.lights.len(), 87);
    assert_eq!(lights.len(), 87);
    assert_eq!(lights.iter().filter(|light| light.color != [0.0; 3]).count(), 84);
    let reaching = |a: &ue2_level::Actor| killer_assets::scene::lights::nearest(&lights, glam::Vec3::from(a.placement.location)).len();
    assert!(pennants.iter().all(|a| reaching(a) >= 2), "{:?}", pennants.iter().map(|a| reaching(a)).collect::<Vec<_>>());

    let material = client.catalog().material("Agit_B_t.agit_b_flag008a_a").unwrap();
    let pass = &material.passes[0];
    assert_eq!((pass.src, pass.dst), (l2_catalog::BlendFactor::SrcAlpha, l2_catalog::BlendFactor::InvSrcAlpha));
    assert!(material.requires_sorting());
    let texture = killer_assets::scene::first_texture_path(&material);
    assert!(texture.eq_ignore_ascii_case("Agit_B_t.Agit_B_flag008a"), "{texture}");
    let raw = client.catalog().texture_raw(&texture).unwrap();
    assert_eq!(raw.format, ue2_assets::TextureFormat::Dxt3);

    let draws = killer_assets::scene::props::build(&mut client, level, &killer_assets::options::ClientOptions::default().clipping);
    let flags: Vec<_> = draws.iter().filter(|d| killer_assets::scene::first_texture_path(&d.material).eq_ignore_ascii_case("Agit_B_t.Agit_B_flag008a") && d.zone == hall).collect();
    assert_eq!(flags.len(), 8);
    for draw in flags {
        assert_eq!(draw.material.passes[0].src, l2_catalog::BlendFactor::SrcAlpha);
        assert_eq!(draw.pass, killer_assets::scene::Pass::Translucent);
        assert!(!draw.sun_affect);
        let brightest = draw.vertices.iter().map(|v| v.color[0].max(v.color[1]).max(v.color[2])).fold(0.0f32, f32::max);
        assert!(brightest > 0.05, "{brightest}");
        assert!(draw.vertices.iter().all(|v| v.color[3] == 1.0 && v.color[0] >= v.color[1] && v.color[1] >= v.color[2]));
    }
}

#[test]
#[ignore]
fn giran_static_mesh_triangles_wind_against_their_stored_normals() {
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let meshes: std::collections::BTreeSet<String> =
        loaded.level.actors.iter().filter_map(|a| a.static_mesh.clone()).collect();
    let (mut along, mut against, mut flat) = (0usize, 0usize, 0usize);
    for path in &meshes {
        let Some(mesh) = client.catalog().static_mesh(path) else { continue };
        let mesh = mesh.mesh;
        for tri in mesh.indices.as_chunks::<3>().0 {
            let p = |i: u16| glam::Vec3::from(mesh.positions[usize::from(i)]);
            let n = |i: u16| glam::Vec3::from(mesh.normals.get(usize::from(i)).copied().unwrap_or([0.0; 3]));
            let (a, b, c) = (p(tri[0]), p(tri[1]), p(tri[2]));
            let winding = (b - a).cross(c - a);
            let stored = n(tri[0]) + n(tri[1]) + n(tri[2]);
            let dot = winding.dot(stored);
            if winding.length_squared() == 0.0 || stored.length_squared() == 0.0 || dot.abs() < 1e-6 {
                flat += 1;
            } else if dot > 0.0 {
                along += 1;
            } else {
                against += 1;
            }
        }
    }
    eprintln!("giran static meshes: {along} along, {against} against, {flat} undecided");
    assert!(against > 100 * along, "{along} along vs {against} against");
}

#[test]
#[ignore]
fn giran_bsp_polygons_wind_along_their_plane_normals() {
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("22_22").unwrap();
    let (mut along, mut against) = (0usize, 0usize);
    for polygon in &loaded.level.bsp {
        let p = |i: usize| glam::Vec3::from(polygon.corners[i]);
        let winding = (p(1) - p(0)).cross(p(2) - p(0));
        let dot = winding.dot(glam::Vec3::from(polygon.normal));
        if dot > 1e-6 {
            along += 1;
        } else if dot < -1e-6 {
            against += 1;
        }
    }
    eprintln!("giran bsp: {along} along, {against} against");
    assert!(along > 50 * against, "{along} along vs {against} against");
}
