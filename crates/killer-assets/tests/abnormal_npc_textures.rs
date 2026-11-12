use killer_assets::abnormal_state::{NpcTextureLists, abnormal_texture_slots};
use killer_assets::client::Client;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[test]
#[ignore]
fn material_sequence_init_10494875_reads_textures_and_textures2_of_every_npcgrp_row() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let tables = client.tables().unwrap();
    let mut rows = 0;
    let mut swapping = 0;
    let mut slots = 0;
    let mut classes: BTreeMap<String, usize> = BTreeMap::new();
    let mut names: BTreeMap<String, Option<String>> = BTreeMap::new();
    for npc in tables.npcgrp.iter() {
        rows += 1;
        let lists = NpcTextureLists { normal: npc.textures.iter().collect::<Vec<_>>(), abnormal: npc.textures2.iter().collect::<Vec<_>>() };
        let used = abnormal_texture_slots(&lists);
        assert_eq!(used.len(), npc.textures.len().min(npc.textures2.len()), "npc {}", npc.id);
        for (index, name) in &used {
            assert!(std::ptr::eq(*name, &npc.textures2[*index]), "npc {}", npc.id);
        }
        if !used.is_empty() {
            swapping += 1;
            eprintln!("npc {} {}: {:?} -> {:?}", npc.id, npc.mesh, npc.textures, npc.textures2);
        }
        slots += used.len();
        for (_, name) in used {
            let class = names.entry(name.clone()).or_insert_with(|| client.catalog().object_class(name)).clone();
            *classes.entry(class.unwrap_or_else(|| "ausente".into())).or_default() += 1;
        }
    }
    assert_eq!(rows, tables.npcgrp.len());
    eprintln!("{rows} linhas do npcgrp, {swapping} trocam texturas no 0x8000 ({slots} índices, {} nomes distintos)", names.len());
    eprintln!("classes dos nomes de textures2 usados: {classes:?}");
}
