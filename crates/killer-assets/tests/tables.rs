use killer_assets::client::Client;
use l2_dat::AnimKind;
use std::path::PathBuf;

fn test_client() -> Option<Client> {
    let root = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    Client::open(&root).ok()
}

#[test]
#[ignore]
fn client_tables_come_from_the_system_folder() {
    let Some(mut client) = test_client() else { return };
    let tables = client.tables().unwrap();
    assert_eq!(tables.npcgrp.len(), 6340);
    assert_eq!(tables.npcgrp.get(7).unwrap().mesh, "LineageMonsters.walking_fungus_m00");
    assert_eq!(tables.npcgrp.get(579).unwrap().mesh, "lineageMonsters_L2Killer.viRUS_Hunchback_m00");
    assert_eq!(tables.npcgrp.get(7001).unwrap().mesh, "LineageNPCs.a_traderA_MHuman_m00");
    assert_eq!(tables.chargrp.len(), 15);
    assert_eq!(tables.chargrp[0].face_meshes, ["Fighter.MFighter_m000_f"]);
    assert_eq!(tables.hairgrp.len(), 15);
    assert_eq!(tables.hairgrp[0].bh[9], 9);
    assert_eq!(tables.npcname.len(), tables.npcgrp.len());
    assert!(tables.npcgrp.iter().all(|npc| tables.npcname.contains_key(&npc.id)));
    assert_eq!(tables.npcname[&28086].name, "Almir Revenger");
    assert_eq!(tables.npcname[&7006].title, "Gatekeeper");
    assert_eq!(tables.systemmsg.len(), 1689);
    assert!(tables.systemmsg[&1].text.starts_with("The server will be disconnected"));
    assert!(tables.anim_names.len() >= 528 + 14 + 100 + 100, "{}", tables.anim_names.len());
    assert_eq!(tables.anim_names["anakim"].name(AnimKind::Wait, 0), Some("wait"));
    assert_eq!(tables.anim_names["lilim_knight"].name(AnimKind::Death, 0), Some("death"));
    assert_eq!(tables.anim_names["mfighter"].name(AnimKind::Wait, 0), Some("wait_Hand_MFighter"));
    assert_eq!(tables.anim_names["gremlin"].name(AnimKind::Run, 0), Some("run"));
    assert_eq!(tables.anim_names["a_tradera_mhuman"].name(AnimKind::Walk, 0), Some("walk_Hand"));
}

#[test]
#[ignore]
fn read_system_file_decrypts() {
    let Some(mut client) = test_client() else { return };
    let body = client.read_system_file("npcname-e.dat").unwrap();
    assert_eq!(body[..4], [0xd9, 0x18, 0, 0]);
    assert!(matches!(client.read_system_file("nao-existe.dat"), Err(killer_assets::client::AssetError::NotFound(_))));
}

#[test]
#[ignore]
fn language_10433990_loads_every_localized_row_from_the_selected_client_files() {
    use l2_dat::{actionname, by_id, commandname, hennagrp, itemname, npcname, questname,
        skillname, symbolname, sysstring, systemmsg, zonename};
    let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
    let mut client = Client::open(&PathBuf::from(root)).unwrap();
    let mut language = client.client_language();
    for native in [false, true] {
        language.is_native = native;
        let tables = client.tables_for_language(&language).unwrap();
        let mut read = |base: &str| client.read_system_file(&language.table_name(base)).unwrap();
        assert_eq!(tables.npcname, npcname::npc_records(tables.npcgrp.iter().map(|npc| npc.id), npcname::read(&read("npcname")).unwrap()));
        assert_eq!(tables.itemname, by_id(itemname::read(&read("itemname")).unwrap(), |row| row.id));
        assert_eq!(tables.systemmsg, by_id(systemmsg::read(&read("systemmsg")).unwrap(), |row| row.id));
        assert_eq!(tables.questname, questname::by_id_state(questname::read(&read("questname")).unwrap()));
        assert_eq!(tables.sysstring, sysstring::read(&read("sysstring")).unwrap().into_iter().map(|row| (row.id, row.text)).collect());
        assert_eq!(tables.symbolname, symbolname::read(&read("symbolname")).unwrap().into_iter().collect());
        assert_eq!(tables.hennagrp, by_id(hennagrp::read(&read("hennagrp")).unwrap(), |row| row.symbol_id));
        assert_eq!(tables.skillname, skillname::by_id_level(skillname::read(&read("skillname")).unwrap()));
        assert_eq!(tables.zonename, zonename::read(&read("zonename")).unwrap());
        assert_eq!(tables.actions, actionname::read(&read("actionname")).unwrap());
        assert_eq!(tables.commandname, commandname::by_name(commandname::read(&read("commandname")).unwrap()));
        assert_eq!(tables.language_type, language.language_type);
    }
}

#[test]
#[ignore]
fn lobby_failure_messages_are_the_ones_the_console_slots_read() {
    let Some(mut client) = test_client() else { return };
    let tables = client.tables().unwrap();
    for id in [128u32, 840, 1776, 1780] {
        eprintln!("{id}: {:?}", tables.systemmsg.get(&id).map(|m| m.text.clone()));
    }
}
