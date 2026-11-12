use killer_assets::client::Client;
use l2_dat::{ItemKind, armorgrp, etcitemgrp, itemicons, weapongrp};
use std::{collections::BTreeMap, path::PathBuf};

#[test]
#[ignore]
fn item_index_resolves_every_id_like_the_original_set_chain() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let etc_body = client.read_system_file("etcitemgrp.dat").unwrap();
    let armor_body = client.read_system_file("armorgrp.dat").unwrap();
    let weapon_body = client.read_system_file("weapongrp.dat").unwrap();
    let mut winner: BTreeMap<u32, (ItemKind, usize)> = BTreeMap::new();
    let mut seen: BTreeMap<u32, Vec<ItemKind>> = BTreeMap::new();
    for (kind, ids) in [
        (ItemKind::Etc, etcitemgrp::read(&etc_body).unwrap().iter().map(|r| r.id).collect::<Vec<_>>()),
        (ItemKind::Armor, armorgrp::read(&armor_body).unwrap().iter().map(|r| r.id).collect()),
        (ItemKind::Weapon, weapongrp::read(&weapon_body).unwrap().iter().map(|r| r.id).collect()),
    ] {
        for (row, id) in ids.into_iter().enumerate() {
            winner.insert(id, (kind, row));
            seen.entry(id).or_default().push(kind);
        }
    }
    let collisions: Vec<_> = seen.iter().filter(|(_, kinds)| kinds.len() > 1).collect();
    eprintln!("{} ids com mais de um registro", collisions.len());
    for (id, kinds) in &collisions {
        eprintln!("{id}: {kinds:?} -> {:?}", winner[id]);
    }
    assert_eq!(collisions.len(), 96);

    let tables = client.tables().unwrap();
    let etc_icons = itemicons::read(&etc_body, ItemKind::Etc).unwrap();
    let armor_icons = itemicons::read(&armor_body, ItemKind::Armor).unwrap();
    let weapon_icons = itemicons::read(&weapon_body, ItemKind::Weapon).unwrap();
    assert_eq!(tables.item_icons.len(), winner.len());
    assert_eq!(tables.weapongrp.len() + tables.armorgrp.len() + tables.etcitemgrp.len(), winner.len());
    for (&id, &(kind, row)) in &winner {
        let expected = match kind {
            ItemKind::Etc => &etc_icons[row],
            ItemKind::Armor => &armor_icons[row],
            ItemKind::Weapon => &weapon_icons[row],
        };
        let mut expected = expected.clone();
        if let Some(name) = expected.texture(0)
            && client.catalog().object_class(&name).is_none()
        {
            expected.icons[0] = killer_assets::client::ITEM_ICON_LOAD_FAILED.to_owned();
        }
        assert_eq!(tables.item_icons.get(id), Some(&expected), "ícone do id {id}");
        assert_eq!(tables.weapongrp.contains_key(&id), kind == ItemKind::Weapon, "weapongrp {id}");
        assert_eq!(tables.armorgrp.contains_key(&id), kind == ItemKind::Armor, "armorgrp {id}");
        assert_eq!(tables.etcitemgrp.contains_key(&id), kind == ItemKind::Etc, "etcitemgrp {id}");
        assert_eq!(tables.items().find(id as i32).is_some(), kind != ItemKind::Etc, "Items {id}");
    }
    assert!(tables.weapongrp.contains_key(&9133));
    assert!(tables.item_icons.texture(9133).unwrap().to_lowercase().contains("draconic_bow"));
}
