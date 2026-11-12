use killer_assets::bone_scale::big_head_section;
use killer_assets::client::Client;
use std::path::PathBuf;

#[test]
#[ignore]
fn bighead_int_has_a_max_size_for_every_player_master_mesh() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let ini = killer_ui::ini::parse_ini(&client.read_system_text("bighead.int").unwrap());
    assert_eq!(ini.get("MFIGHTER", "MaxSize"), Some("2.3"));
    let tables = client.tables().unwrap();
    let mut sections = Vec::new();
    for row in tables.chargrp.iter().filter(|row| !row.face_meshes.is_empty()) {
        let section = big_head_section(&row.face_meshes[0]).to_owned();
        eprintln!("{} -> {section} {:?}", row.face_meshes[0], ini.get(&section, "MaxSize"));
        sections.push((section.clone(), ini.has_section(&section)));
    }
    assert!(sections.iter().filter(|(_, found)| *found).count() >= 14, "{sections:?}");
}
