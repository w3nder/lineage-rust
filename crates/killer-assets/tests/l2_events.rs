use killer_assets::client::Client;
use std::collections::BTreeMap;
use std::path::PathBuf;
use ue2_assets::{find, object_properties};

#[test]
#[ignore]
fn l2_game_event_of_every_actor_of_the_143_tiles() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR obrigatório"));
    let mut client = Client::open(&root).unwrap();
    let mut tiles = 0;
    let mut raw_actors = 0;
    let mut actors = 0;
    let mut entries = 0;
    let mut classes = BTreeMap::<String, usize>::new();
    let mut commands = BTreeMap::<u8, usize>::new();
    let mut hidden = BTreeMap::<(String, bool), usize>::new();
    let mut ids = BTreeMap::<i32, usize>::new();
    let mut emitters = 0;
    let mut smoke = BTreeMap::<(String, String), usize>::new();
    let mut smoke_sprites = BTreeMap::<(bool, bool), usize>::new();
    for x in 15..=26 {
        for y in 10..=26 {
            let tile = format!("{x}_{y}");
            if client.level_path(&tile).is_err() {
                continue;
            }
            let loaded = client.load_level(&tile).unwrap();
            let path = client.level_path(&tile).unwrap();
            let raw = std::fs::read(&path).unwrap();
            let file = l2_crypto::decrypt(&raw, &path).unwrap();
            let package = ue2_package::Package::parse(&file).unwrap();
            let mut tile_raw = 0;
            for export in package.exports() {
                if export.serial_size == 0 || matches!(export.class, ue2_package::ObjectRef::Null) {
                    continue;
                }
                let Ok(properties) = object_properties(&package, &file[..], export) else { continue };
                if find(&properties, "Level").is_some() && find(&properties, "L2GameEvent").is_some() {
                    tile_raw += 1;
                }
            }
            raw_actors += tile_raw;
            tiles += 1;
            let mut tile_actors = 0;
            for emitter in loaded.level.emitters.iter().filter(|e| e.tag.as_deref().is_some_and(|t| t.eq_ignore_ascii_case("antarascave_smoke"))) {
                for sprite in emitter.sprites.iter().filter(|s| !s.class.is_empty()) {
                    *smoke_sprites.entry((sprite.disabled, sprite.trigger_disabled)).or_default() += 1;
                }
            }
            for actor in loaded.level.actors.iter().filter(|a| a.tag.as_deref().is_some_and(|t| t.eq_ignore_ascii_case("antarascave_smoke"))) {
                *smoke.entry((tile.clone(), actor.class.clone())).or_default() += 1;
            }
            for actor in &loaded.level.actors {
                if actor.l2_events.is_empty() {
                    continue;
                }
                tile_actors += 1;
                *classes.entry(actor.class.clone()).or_default() += 1;
                *hidden.entry((actor.class.clone(), actor.hidden)).or_default() += 1;
                for event in &actor.l2_events {
                    entries += 1;
                    *commands.entry(event.cmd).or_default() += 1;
                    *ids.entry(event.id).or_default() += 1;
                }
            }
            let emitter_actors: Vec<_> = loaded.level.actors.iter().filter(|a| a.class.eq_ignore_ascii_case("Emitter")).collect();
            assert_eq!(emitter_actors.len(), loaded.level.emitters.len(), "{tile}");
            for (actor, emitter) in emitter_actors.iter().zip(&loaded.level.emitters) {
                assert_eq!(actor.l2_events, emitter.l2_events, "{tile}");
                emitters += usize::from(!emitter.l2_events.is_empty());
            }
            assert_eq!(tile_actors, tile_raw, "{tile}");
            actors += tile_actors;
            client.catalog().release_files();
        }
    }
    eprintln!("tiles={tiles}; actors={actors}; entries={entries}; emitters={emitters}; classes={classes:?}; commands={commands:?}; hidden={hidden:?}; ids={ids:?}; smoke={smoke:?}; smoke_sprites={smoke_sprites:?}");
    assert_eq!(tiles, 143);
    assert_eq!(actors, raw_actors);
    assert_eq!((actors, entries, emitters), (30, 30, 30));
    assert_eq!(commands, BTreeMap::from([(ue2_level::LEC_SHOW, 30)]));
    assert_eq!(smoke, BTreeMap::from([(("25_21".to_owned(), "Emitter".to_owned()), 21)]));
    assert_eq!(smoke_sprites, BTreeMap::from([((true, true), 92)]));
}
