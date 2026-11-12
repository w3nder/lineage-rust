use killer_assets::client::Client;
use std::collections::HashMap;
use std::path::PathBuf;

fn dir(var: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(var).unwrap_or_else(|| panic!("{var}")))
}

#[test]
#[ignore]
fn item_ids_of_the_c4_server_client_against_the_interlude_client() {
    let mut c4 = Client::open(&dir("KILLER_C4_DIR")).unwrap();
    let mut il = Client::open(&dir("KILLER_INTERLUDE_DIR")).unwrap();
    let names_c4: HashMap<u32, String> = l2_dat::itemname::read_as(&c4.read_system_file("itemname-e.dat").unwrap(), c4.chronicle()).unwrap().into_iter().map(|row| (row.id, row.name)).collect();
    let names_il: HashMap<u32, String> = l2_dat::itemname::read_as(&il.read_system_file("itemname-e.dat").unwrap(), il.chronicle()).unwrap().into_iter().map(|row| (row.id, row.name)).collect();
    let only_c4 = names_c4.keys().filter(|id| !names_il.contains_key(id)).count();
    let same = names_c4.iter().filter(|(id, name)| names_il.get(id) == Some(name)).count();
    let different: Vec<_> = names_c4.iter().filter(|(id, name)| names_il.get(id).is_some_and(|other| other != *name)).collect();
    eprintln!("itens C4 {} Interlude {}: só no C4 {only_c4}, mesmo nome {same}, mesmo id com nome diferente {}", names_c4.len(), names_il.len(), different.len());
    let mut sample: Vec<_> = different.into_iter().collect();
    sample.sort();
    for (id, name) in sample.iter().take(8) {
        eprintln!("  id {id}: C4 {name:?} / Interlude {:?}", names_il[id]);
    }
    let mut missing: Vec<_> = names_c4.iter().filter(|(id, _)| !names_il.contains_key(id)).collect();
    missing.sort();
    for (id, name) in missing.iter().take(8) {
        eprintln!("  só no C4: {id} {name:?}");
    }
}
