use std::path::PathBuf;

#[test]
#[ignore]
fn npc_records_carry_the_target_window_skill_lists_and_the_bars_flag() {
    let mut client = killer_assets::client::Client::open(&PathBuf::from(std::env::var("KILLER_CLIENT_DIR").unwrap())).unwrap();
    let tables = client.tables().unwrap();
    let with_skills = tables.npcgrp.iter().filter(|npc| !npc.skills.is_empty()).count();
    let flagged = tables.npcgrp.iter().filter(|npc| npc.shows_bars != 0).count();
    let flags: std::collections::BTreeSet<u32> = tables.npcgrp.iter().map(|npc| npc.shows_bars).collect();
    println!("npcs {} com skills {with_skills} com barras {flagged} valores {flags:?}", tables.npcgrp.len());
    assert!(!tables.npcgrp.is_empty());
}
