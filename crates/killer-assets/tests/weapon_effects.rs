use killer_assets::client::Client;
use std::collections::BTreeSet;
use std::path::PathBuf;

fn test_client() -> Client {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from).expect("KILLER_CLIENT_DIR obrigatório");
    Client::open(&root).unwrap()
}

#[test]
#[ignore]
fn every_weapongrp_effect_class_reads_its_defaultproperties() {
    let mut client = test_client();
    let tables = client.tables().unwrap();
    let names: BTreeSet<String> = tables.weapongrp.values().flat_map(|w| &w.effects).map(|e| e.name.clone()).filter(|n| !n.is_empty() && !n.eq_ignore_ascii_case("None")).collect();
    assert!(names.len() >= 20, "{names:?}");
    for name in &names {
        let emitter = client.emitter_class(name).unwrap_or_else(|| panic!("{name}"));
        assert!(!emitter.sprites.is_empty(), "{name}");
        assert!(emitter.sprites.iter().all(|s| !s.class.is_empty()), "{name}");
    }
    eprintln!("{} classes de efeito de arma lidas", names.len());
}

#[test]
#[ignore]
fn c_u007_matches_the_class_defaults_in_lineageeffect_u() {
    let mut client = test_client();
    let emitter = client.emitter_class("LineageEffect.c_u007").unwrap();
    assert!(emitter.auto_reset);
    assert_eq!(emitter.sprites.len(), 1);
    let s = &emitter.sprites[0];
    assert_eq!(s.class, "SpriteEmitter");
    assert_eq!(s.max_particles, 8);
    assert_eq!(s.opacity, 0.5);
    assert_eq!(s.initial_particles_per_second, 4.0);
    assert!(!s.automatic_initial_spawning);
    assert_eq!(s.start_offset, [25.0, 0.0, 0.0]);
    assert_eq!(s.start_size, [[3.0, 4.0], [3.0, 4.0], [1.0, 1.34]]);
    assert_eq!(s.start_velocity, [[4.0, 4.0], [0.0, 0.0], [0.0, 0.0]]);
    assert_eq!(s.lifetime, [2.0, 2.0]);
    assert_eq!(s.coordinate_system, 1);
    assert_eq!(s.texture.as_deref().map(str::to_ascii_lowercase).as_deref(), Some("lineageeffectstextures.etc.fx_m_t0144"));
    assert_eq!(s.subdivisions, [4, 1]);
    assert_eq!(s.color_scale.len(), 3);
}

#[test]
#[ignore]
fn soulshot_data_load_10433f10_reads_the_client_int_and_every_class_loads() {
    let mut client = test_client();
    let tables = client.tables().unwrap();
    let data = &tables.soulshot;
    assert_eq!(data.hands, [[0, 0], [-1, 1], [-1, 1], [1, 1], [-1, 1], [0, -1], data.hands[6], data.hands[7]]);
    let mut classes = 0;
    for grade in &data.grades {
        for class in [&grade.on_sticks, &grade.on_books, &grade.atk, &grade.atk_dark_cr, &grade.atk_cr, &grade.spirit_on_sticks, &grade.spirit_on_books] {
            assert!(class.starts_with("LineageEffect."), "{class}");
            let emitter = client.emitter_class(class).unwrap_or_else(|| panic!("{class}"));
            assert!(!emitter.sprites.is_empty(), "{class}");
            classes += 1;
        }
    }
    assert_eq!(classes, 42);
    assert_eq!(data.grades[0].on_sticks, "LineageEffect.e_u505_a");
    assert_eq!(data.grades[5].spirit_on_books, "LineageEffect.e_u517_b");
}
