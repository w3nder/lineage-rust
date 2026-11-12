use killer_assets::client::{Client, set_base_root};
use std::path::PathBuf;

#[test]
#[ignore]
fn interlude_assets_with_the_c4_server_data_load_every_table_and_find_custom_items() {
    let c4 = PathBuf::from(std::env::var_os("KILLER_C4_DIR").expect("KILLER_C4_DIR"));
    let interlude = PathBuf::from(std::env::var_os("KILLER_INTERLUDE_DIR").expect("KILLER_INTERLUDE_DIR"));
    set_base_root(c4);
    let mut client = Client::open(&interlude).unwrap();
    assert_eq!(client.chronicle(), l2_dat::Chronicle::Interlude);
    let tables = client.tables().unwrap();
    let name = |id: u32| tables.itemname.get(&id).map(|item| item.name.clone());
    eprintln!("item 9416: {:?}, item 234: {:?}, armaduras {}, armas {}, npcs {}", name(9416), name(234), tables.armorgrp.len(), tables.weapongrp.len(), tables.npcname.len());
    assert_eq!(name(9416).as_deref(), Some("Maxkiller Breastplate"));
    let texture = client.catalog().texture("LineageWeaponsTex.ItemCom_Sword_Drow_00");
    eprintln!("textura de arma do C4/Interlude: {}", texture.is_some());
}

#[test]
#[ignore]
fn item_icons_resolve_to_the_right_images_in_the_hybrid_client() {
    let c4 = PathBuf::from(std::env::var_os("KILLER_C4_DIR").expect("KILLER_C4_DIR"));
    let interlude = PathBuf::from(std::env::var_os("KILLER_INTERLUDE_DIR").expect("KILLER_INTERLUDE_DIR"));
    let out = PathBuf::from(std::env::var_os("KILLER_ICON_OUT").expect("KILLER_ICON_OUT"));
    std::fs::create_dir_all(&out).unwrap();
    set_base_root(c4.clone());
    let mut hybrid = Client::open(&interlude).unwrap();
    let tables = hybrid.tables().unwrap();
    let mut pure = Client::open(&c4).unwrap();
    for id in [57u32, 1, 234, 9416, 6577, 1060, 17, 736] {
        let Some(icon) = tables.item_icons.items.get(&id) else {
            eprintln!("item {id}: sem ícone na tabela");
            continue;
        };
        let name = icon.texture(0).unwrap_or_default();
        let h = hybrid.catalog().texture(&name);
        let c = pure.catalog().texture(&name);
        eprintln!("item {id} {:?}: textura {name:?} híbrido {:?} C4 puro {:?}", tables.itemname.get(&id).map(|n| n.name.clone()), h.as_ref().map(|i| (i.width, i.height)), c.as_ref().map(|i| (i.width, i.height)));
        for (label, image) in [("hib", h), ("c4", c)] {
            if let Some(image) = image {
                let mut ppm = format!("P6\n{} {}\n255\n", image.width, image.height).into_bytes();
                for px in image.rgba.chunks(4) {
                    ppm.extend_from_slice(&px[..3]);
                }
                std::fs::write(out.join(format!("{id}_{label}.ppm")), ppm).unwrap();
            }
        }
    }
}
