use killer_assets::client::Client;
use std::{collections::BTreeMap, path::PathBuf};

const PACKAGES: [&str; 2] = ["l2killer_headgear.", "l2killer_assassin_light."];

fn references(debug: &str) -> Vec<String> {
    let lower = debug.to_lowercase();
    let mut found = Vec::new();
    for package in PACKAGES {
        for (at, _) in lower.match_indices(package) {
            let end = lower[at..].find(['"', ' ', ',', ')']).map_or(lower.len(), |n| at + n);
            found.push(lower[at..end].to_owned());
        }
    }
    found
}

#[test]
#[ignore]
fn items_meshes_and_icons_that_depend_on_the_cafef00d_packages_are_listed_and_skipped_alike() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let tables = client.tables().unwrap();
    let mut report = BTreeMap::<(String, u32), Vec<String>>::new();
    let mut add = |kind: &str, id: u32, debug: String| {
        let found = references(&debug);
        if !found.is_empty() {
            report.entry((kind.to_owned(), id)).or_default().extend(found);
        }
    };
    for (id, armor) in &tables.armorgrp {
        add("armorgrp", *id, format!("{armor:?}"));
    }
    for (id, weapon) in &tables.weapongrp {
        add("weapongrp", *id, format!("{weapon:?}"));
    }
    for (id, item) in &tables.etcitemgrp {
        add("etcitemgrp", *id, format!("{item:?}"));
    }
    for (id, item) in &tables.item_icons.items {
        add("icones", *id, format!("{item:?}"));
    }
    for (index, template) in tables.chargrp.iter().enumerate() {
        add("chargrp", index as u32, format!("{template:?}"));
    }
    for (id, armor) in &tables.armorgrp {
        for row in &armor.rows {
            let textures = row.textures.iter().chain(&row.ex_textures).any(|t| !references(t).is_empty());
            let meshes_outside = row
                .meshes
                .iter()
                .chain(&row.ex_meshes)
                .any(|m| !m.is_empty() && !m.eq_ignore_ascii_case("None") && references(m).is_empty());
            assert!(!(textures && meshes_outside), "item {id}: textura do pacote protegido em malha de pacote legível");
        }
    }
    let mut lines = Vec::new();
    for ((kind, id), paths) in &mut report {
        paths.sort();
        paths.dedup();
        let name = tables.itemname.get(id).map(|n| format!("{} {}", n.name, n.add_name)).unwrap_or_default();
        lines.push(format!("{kind}\t{id}\t{}\t{}", name.trim(), paths.join(",")));
    }
    let every: std::collections::BTreeSet<_> = report.values().flatten().cloned().collect();
    let mut skipped = 0;
    for path in &every {
        let package = path.split('.').next().unwrap();
        match client.catalog().package_status(package) {
            l2_catalog::PackageStatus::Encrypted => skipped += 1,
            l2_catalog::PackageStatus::Loaded => {}
            other => panic!("{path}: {other:?}"),
        }
    }
    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/preparation-validation/cafef00d-dependents.tsv");
    std::fs::create_dir_all(out.parent().unwrap()).unwrap();
    std::fs::write(&out, lines.join("\n")).unwrap();
    eprintln!("{} entradas, {} referências únicas, {skipped} puladas; {}", lines.len(), every.len(), out.display());
}

#[test]
#[ignore]
fn item_icons_of_the_decoded_cafef00d_packages_resolve_to_loadable_textures() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let tables = client.tables().unwrap();
    for id in 13048..=13066 {
        let texture = tables.item_icons.texture(id).unwrap_or_else(|| panic!("{id}"));
        assert_ne!(texture, killer_assets::client::ITEM_ICON_LOAD_FAILED, "{id}");
        assert_eq!(client.catalog().object_class(&texture).as_deref(), Some("Texture"), "{id}: {texture}");
    }
    let loaded = tables.item_icons.items.values().filter(|i| i.texture(0).is_some_and(|t| t != killer_assets::client::ITEM_ICON_LOAD_FAILED)).count();
    assert!(loaded > 1000);
}
