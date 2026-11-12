use killer_assets::client::Client;
use killer_ui::zone_title::{ZoneIndex, ZoneTexture, map_cell, zone_at, zone_texture_name};
use std::path::PathBuf;

#[test]
#[ignore]
fn the_installed_zone_textures_resolve_the_town_names_at_known_positions() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let tables = client.tables().unwrap();
    let index = ZoneIndex::new(&tables.zonename);
    let mut found = Vec::new();
    for (name, position) in [("Giran", (82157.0f32, 147057.0f32, -134.0f32)), ("Talking Island", (-84000.0, 244000.0, -3600.0)), ("Aden", (147500.0, 25000.0, -2600.0))] {
        let cell = map_cell(position.0 as i32, position.1 as i32);
        let texture = client.catalog().texture_raw(&zone_texture_name(cell)).and_then(|raw| {
            let mip = raw.mips.into_iter().next()?;
            Some(ZoneTexture { width: mip.width as i32, height: mip.height as i32, pixels: mip.data })
        });
        let zone = zone_at(&index, position, |_| texture.as_ref()).map(|zone| zone.name.clone());
        eprintln!("{name} {cell:?} texture={} -> {zone:?}", texture.is_some());
        found.push((name, texture.is_some(), zone));
    }
    let names: Vec<_> = found.iter().map(|(_, loaded, zone)| (*loaded, zone.as_deref())).collect();
    assert_eq!(
        names,
        [(true, Some("Giran Castle Town")), (true, Some("Talking Island Village")), (true, Some("Town of Aden"))]
    );
}

#[test]
#[ignore]
fn the_town_map_textures_have_the_sizes_of_the_100b8c20_tables() {
    use killer_ui::minimap::{MAP_HEIGHTS, MAP_WIDTHS, TOWN_MAP_NAMES, TOWN_MAP_PREFIX};
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    for (index, name) in TOWN_MAP_NAMES.iter().enumerate() {
        let raw = client.catalog().texture_raw(&format!("{TOWN_MAP_PREFIX}{name}"));
        let mip = raw.as_ref().and_then(|raw| raw.mips.first());
        eprintln!("{name}: {:?}", mip.map(|mip| (mip.width, mip.height)));
        let (width, height) = mip.map(|mip| (mip.width as i32, mip.height as i32)).unwrap_or((0, 0));
        assert!(width >= MAP_WIDTHS[index + 1] && height >= MAP_HEIGHTS[index + 1], "{name}");
    }
}
