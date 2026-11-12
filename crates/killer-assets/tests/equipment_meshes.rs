use killer_assets::client::Client;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

#[test]
#[ignore]
fn every_equipment_and_enchant_mesh_reports_its_actual_load_result() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let tables = client.tables().unwrap();
    let mut paths = BTreeMap::<String, BTreeSet<u32>>::new();
    for armor in tables.armorgrp.values() {
        for path in armor.rows.iter().flat_map(|r| r.meshes.iter().chain(&r.ex_meshes)).chain(&armor.npc_meshes) {
            if !path.is_empty() && !path.eq_ignore_ascii_case("None") {
                paths.entry(path.to_lowercase()).or_default().insert(armor.id);
            }
        }
    }
    for weapon in tables.weapongrp.values() {
        for path in weapon.meshes.iter().chain(weapon.enchant_meshes.iter().map(|e| &e.name)) {
            if !path.is_empty() && !path.eq_ignore_ascii_case("None") {
                paths.entry(path.to_lowercase()).or_default().insert(weapon.id);
            }
        }
    }
    let mut package = String::new();
    let mut loaded = 0;
    let mut failed = Vec::new();
    let mut undecodable = Vec::new();
    for (path, ids) in &paths {
        let next = path.split('.').next().unwrap();
        if next != package {
            client.catalog().release_files();
            package = next.to_owned();
        }
        match client.catalog().try_skeletal_mesh(path) {
            Ok(mesh) => {
                assert!(mesh.mesh.indices.len().is_multiple_of(3), "{path}");
                loaded += 1;
            }
            Err(reason) => {
                let row = format!("{path}\t{ids:?}\t{reason:?}");
                if matches!(reason, l2_catalog::MissingAsset::Undecodable { .. }) {
                    undecodable.push(row.clone());
                }
                failed.push(row);
            }
        }
    }
    let report =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/preparation-validation/equipment-meshes.tsv");
    std::fs::create_dir_all(report.parent().unwrap()).unwrap();
    std::fs::write(&report, failed.join("\n")).unwrap();
    eprintln!(
        "{} referências distintas: {loaded} lidas, {} sem malha; relatório {}",
        paths.len(),
        failed.len(),
        report.display()
    );
    assert!(undecodable.is_empty(), "falha de leitor em recurso existente: {undecodable:?}");
}
