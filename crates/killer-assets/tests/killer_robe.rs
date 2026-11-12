use killer_assets::{characters::load_part, client::Client, pawn::{Part, Placement}};
use std::{collections::BTreeSet, path::PathBuf};

#[test]
#[ignore]
fn killer_robe_all_rows_resolve_meshes_and_materials() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let path=root.join("system/Engine.u");
    let raw=std::fs::read(&path).unwrap();
    let file=l2_crypto::decrypt(&raw,&path).unwrap().into_owned();
    let package=ue2_package::Package::parse(&file).unwrap();
    for class in ["Modifier","TexModifier","TexEnvMap"] {
        let export=package.exports().iter().find(|e|package.class_name(e)=="Class" && package.name(e.name)==class).unwrap();
        let properties=ue2_assets::class_default_properties(&package,&file,export).unwrap();
        eprintln!("Engine.{class}: {properties:?}");
    }
    let mut client = Client::open(&root).unwrap();
    let tables = client.tables().unwrap();
    let ids: BTreeSet<_> = tables.itemname.values().filter(|n| n.name.starts_with("Killer ") && n.add_name == "Robe").map(|n| n.id).collect();
    assert_eq!(ids.len(), 5);
    let mut report = Vec::new();
    let mut failures = Vec::new();
    let mut unavailable = BTreeSet::new();
    for id in ids {
        let armor = &tables.armorgrp[&id];
        for (row, resources) in armor.rows.iter().enumerate() {
            report.push(format!("{id} row={row} {resources:?}"));
            for path in resources.textures.iter().chain(&resources.ex_textures).filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("None")) {
                match client.catalog().material(path) {
                    Some(material) => for texture in material.texture_paths() {
                        if let Err(reason) = client.catalog().try_texture_raw(texture) {unavailable.insert(format!("{path}: {texture}: {reason:?}"));}
                    },
                    None => {unavailable.insert(format!("{path}: material não resolvido"));}
                }
            }
            for path in resources.meshes.iter().filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("None")) {
                let slot = match armor.body_part {6=>6,9=>3,10=>2,11=>1,12=>0,_=>panic!("body part")};
                let mut part = Part::new(path, resources.textures.clone());
                part.slot = slot;
                part.placement = Placement::Skinned;
                match load_part(&mut client, &part) {
                    Some(mesh) => {
                        report.push(format!("  mesh={} vertices={} sections={}",path,mesh.vertices.len(),mesh.sections.len()));
                        for section in mesh.sections {
                            report.push(format!("  material {:?}", section.material));
                        }
                    }
                    None => failures.push(format!("{id} row={row} {path}")),
                }
            }
            client.catalog().release_files();
        }
    }
    let dir=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/preparation-validation");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("killer-robe.txt"),report.join("\n")).unwrap();
    assert!(unavailable.is_empty(),"Recursos indisponíveis: {unavailable:?}");
    assert!(failures.is_empty(),"{failures:?}");
}
