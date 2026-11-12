use killer_assets::client::Client;
use killer_assets::options::ClientOptions;
use killer_assets::scene::sky::{SkyTint, load_shared};
use std::path::PathBuf;

fn test_client() -> Option<Client> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    Client::open(&root).ok()
}

#[test]
#[ignore]
fn the_shared_sky_level_has_the_sun_the_five_moons_and_nine_flares() {
    let Some(mut client) = test_client() else { return };
    let loaded = client.load_level("skylevel").unwrap();
    let sky = &loaded.level.sky;
    let zone = sky.zone.as_ref().unwrap();
    assert!((zone.location[0] - 324.2).abs() < 0.1 && (zone.location[1] - 264_435.16).abs() < 0.1);
    let flares: Vec<_> = zone.flares().collect();
    assert_eq!(flares.len(), 9);
    assert_eq!(flares[0].0, "L2_Skies.FlareSet.Flare02");
    assert_eq!(flares.iter().map(|f| f.1).collect::<Vec<_>>(), [-0.12, -0.1, 0.1, 0.12, 0.17, 0.2, 0.4, 0.5, 0.6]);
    assert_eq!(flares.iter().map(|f| f.2).collect::<Vec<_>>(), [0.5, 0.38, 0.3, 0.4, 1.0, 1.0, 1.0, 2.0, 3.0]);
    let sun = sky.sun.as_ref().unwrap();
    assert_eq!((sun.radius, sun.scale, sun.skin.as_deref()), (350.0, 1.0, Some("L2_Skies.FlareSet.Sun01")));
    assert_eq!(sky.moons.len(), 5);
    assert!(sky.moons.iter().all(|moon| moon.radius == 245.0));
    let mut env_types: Vec<u8> = sky.moons.iter().map(|moon| moon.env_type).collect();
    env_types.sort_unstable();
    assert_eq!(env_types, [0, 1, 1, 1, 2]);
    let options = ClientOptions::load(&mut client).unwrap();
    assert!(options.shared_sky);
    let scene = load_shared(&mut client, &options).unwrap();
    assert!(scene.draws.iter().all(|draw| scene.zone(draw.zone).is_sky));
    let tints: Vec<SkyTint> = scene.draws.iter().filter_map(|draw| draw.tint).collect();
    assert!(tints.contains(&SkyTint::SkyBox) && tints.contains(&SkyTint::Hazering) && tints.contains(&SkyTint::Cloud(0)), "{tints:?}");
    assert_eq!(scene.sky_bodies.len(), 6);
    assert_eq!(scene.flares.len(), 9);
    for body in &scene.sky_bodies {
        let paths = body.material.texture_paths();
        assert!(!paths.is_empty());
        for path in paths {
            let class = client.catalog().object_class(path);
            assert!(class.as_deref().is_some_and(|class| class.eq_ignore_ascii_case("Texture")), "{path}: {class:?}");
            assert!(client.catalog().try_texture_raw(path).is_ok(), "{path}");
        }
    }
}

#[test]
#[ignore]
fn the_moon_skins_that_are_shaders_draw_the_diffuse_texture_with_the_brighten_blend() {
    let Some(mut client) = test_client() else { return };
    for skin in ["L2_Skies.moon.0", "L2_Skies.moon.1", "L2_Skies.moon.2", "L2_Skies.moon.3", "L2_Skies.moon.Default_Moon"] {
        let class = client.catalog().object_class(skin).unwrap_or_default();
        let material = killer_assets::scene::resolve_material(&mut client, skin);
        let paths = material.texture_paths();
        assert!(!paths.is_empty(), "{skin}");
        assert!(paths.iter().all(|path| !path.eq_ignore_ascii_case(skin) || class.eq_ignore_ascii_case("Texture")), "{skin} ({class}) ficou como textura");
        let body = killer_assets::scene::sky::body_material_over(material);
        let pass = &body.passes[0];
        assert_eq!((pass.src, pass.dst, pass.z_write, pass.two_sided), (l2_catalog::BlendFactor::SrcAlpha, l2_catalog::BlendFactor::One, false, true), "{skin}");
    }
}

#[test]
#[ignore]
fn every_tile_sky_zone_carries_the_same_flare_table_as_the_shared_sky() {
    let Some(mut client) = test_client() else { return };
    let shared = client.load_level("skylevel").unwrap();
    let reference: Vec<(String, f32, f32)> = shared.level.sky.zone.as_ref().unwrap().flares().map(|(t, o, s)| (t.to_owned(), o, s)).collect();
    let (mut tiles, mut same, mut without_sky, mut deviations) = (0, 0, 0, Vec::new());
    for x in 15..=26 {
        for y in 10..=26 {
            let tile = format!("{x}_{y}");
            let Ok(loaded) = client.load_level(&tile) else { continue };
            tiles += 1;
            let Some(zone) = loaded.level.sky.zone.as_ref() else {
                without_sky += 1;
                continue;
            };
            let flares: Vec<(String, f32, f32)> = zone.flares().map(|(t, o, s)| (t.to_owned(), o, s)).collect();
            if flares == reference {
                same += 1;
            } else {
                deviations.push(format!("{tile}: {} flares", flares.len()));
            }
            if let Some(sun) = &loaded.level.sky.sun
                && sun.radius != 350.0
            {
                deviations.push(format!("{tile}: NSun Radius {}", sun.radius));
            }
        }
    }
    eprintln!("{tiles} tiles, {same} com a tabela de flares do skylevel, {without_sky} sem SkyZoneInfo, desvios: {deviations:?}");
    assert_eq!(tiles, 143);
    assert!(deviations.len() <= 5, "{deviations:?}");
}
