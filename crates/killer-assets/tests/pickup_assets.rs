use killer_assets::{client::Client, pickup::{self, Shape}};
use std::{collections::HashSet, path::PathBuf};

#[test]
#[ignore]
fn every_item_drop_loads_original_mesh_slots_or_reports_unavailable_client_data() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let tables = client.tables().unwrap();
    let mut items: Vec<_> = tables.item_icons.items.values().collect();
    items.sort_by_key(|item| item.id);
    let mut checked = HashSet::new();
    let mut missing = Vec::new();
    let mut unavailable = Vec::new();
    let mut pieces = 0;
    for item in &items {
        if !checked.insert(format!("{:?}",item.drop)) { continue; }
        let expected = pickup::appearance(&item.drop);
        let asset = pickup::assets::load(&mut client,&item.drop);
        for path in &asset.missing {
            let class = client.catalog().object_class(path);
            let error = client.catalog().try_skeletal_mesh_lod_with_info(path,0).err();
            let message = format!("{}: {path}, class={class:?}, error={error:?}",item.id);
            match error {
                Some(l2_catalog::MissingAsset::Encrypted | l2_catalog::MissingAsset::WrongClass(_)) => unavailable.push(message),
                _ => missing.push(message),
            }
        }
        if asset.missing.is_empty() { assert_eq!(asset.pieces.len(),expected.parts.len(),"item {}",item.id); }
        let mut shape = Shape::from_data(&item.drop);
        for piece in &asset.pieces {
            let source = expected.parts.iter().find(|part| part.slot == piece.mesh.slot).unwrap();
            assert_eq!(piece.mesh.lod.as_ref().unwrap().source.mesh,source.mesh);
            shape.mesh(item.drop.style,&piece.mesh.floats_150);
            assert_eq!(piece.shape,shape);
            assert!(!piece.mesh.vertices.is_empty(),"item {} slot {}",item.id,piece.mesh.slot);
            assert!(!piece.mesh.indices.is_empty(),"item {} slot {}",item.id,piece.mesh.slot);
            assert!(!piece.mesh.sections.is_empty(),"item {} slot {}",item.id,piece.mesh.slot);
            pieces += 1;
        }
        drop(asset);
        client.catalog().release_files();
        assert_eq!(client.catalog().cached_bytes(),0);
    }
    eprintln!("{} itens, {} configurações de drop, {pieces} peças",items.len(),checked.len());
    eprintln!("dados indisponíveis no cliente ({}):\n{}",unavailable.len(),unavailable.join("\n"));
    assert!(missing.is_empty(),"recursos de drop ausentes:\n{}",missing.join("\n"));
}

#[test]
#[ignore]
fn every_drop_sound_loads_or_reports_absent_original_data() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let tables = client.tables().unwrap();
    let mut names: Vec<_> = tables.weapongrp.values().map(|item| item.drop_sound.as_str())
        .chain(tables.armorgrp.values().map(|item| item.drop_sound.as_str()))
        .chain(tables.etcitemgrp.values().map(|item| item.item_sound.as_str()))
        .chain(std::iter::once(killer_assets::pickup::drop::THROW_SOUND))
        .filter(|name| !name.is_empty() && !name.eq_ignore_ascii_case("None")).collect();
    names.sort_unstable();
    names.dedup();
    let mut missing = Vec::new();
    let mut unavailable = Vec::new();
    for name in &names {
        match client.sound(name) {
            Some((format,bytes)) => {
                assert_eq!(format.to_lowercase(),"wav","{name}");
                assert!(bytes.starts_with(b"RIFF"),"{name}");
            }
            None if client.catalog().object_class(name).is_none() => unavailable.push(*name),
            None => missing.push(*name),
        }
        client.catalog().release_files();
    }
    eprintln!("{} sons de queda/lançamento; dados ausentes: {unavailable:?}",names.len());
    assert!(missing.is_empty(),"sons ausentes: {missing:?}");
}
