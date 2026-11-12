use killer_world::magic::spawn_n_transient_effect;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[test]
#[ignore]
fn spawn_n_transient_effect_covers_every_row_of_the_client_skillgrp() {
    let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
    let path = root.join("system").join("skillgrp.dat");
    let raw = std::fs::read(&path).unwrap();
    let body = l2_crypto::decrypt(&raw, &path).unwrap().into_owned();
    let skills = l2_dat::skillgrp::read(&body).unwrap();
    assert_eq!(skills.len(), 26351);
    let mut instant = BTreeMap::new();
    for skill in &skills {
        let id = i32::try_from(skill.id).unwrap();
        assert!((0..0x10000).contains(&id), "o oráculo varreu 0..0x10000");
        if skill.cast_style == 0 {
            *instant.entry(spawn_n_transient_effect(id)).or_insert(0usize) += 1;
        }
    }
    eprintln!("cast_style 0: {instant:?}");
    assert_eq!(instant.get(&false), Some(&4502));
    assert_eq!(instant.get(&true), Some(&165));
    assert_eq!(instant.values().sum::<usize>(), skills.iter().filter(|skill| skill.cast_style == 0).count());
}
