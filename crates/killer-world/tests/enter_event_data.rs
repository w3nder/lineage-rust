use l2_net::game::NpcInfo;
use killer_world::{World, WorldEvent};
use std::path::PathBuf;

fn npc(object_id: i32, npc_id: u32) -> NpcInfo {
    NpcInfo {
        object_id,
        npc_id: npc_id as i32,
        attackable: true,
        x: 1000,
        y: 2000,
        z: 3000,
        heading: 16384,
        m_atk_speed: 333,
        p_atk_speed: 253,
        run_speed: 120,
        walk_speed: 60,
        speed_multiplier: 1.0,
        attack_speed_multiplier: 1.0,
        collision_radius: 9.0,
        collision_height: 23.0,
        rhand: 0,
        chest: 0,
        lhand: 0,
        name_above: true,
        running: false,
        in_combat: false,
        alike_dead: false,
        spawn_animation: 2,
        name: String::new(),
        title: String::new(),
        playable_flag: 0,
        pvp_flag: 0,
        karma: 0,
        abnormal_effect: 0,
        clan_id: 0,
        clan_crest_id: 0,
        ally_id: 0,
        ally_crest_id: 0,
        team: 0,
        base_collision_radius: 9.0,
        base_collision_height: 23.0,
    }
}

#[test]
#[ignore]
fn every_row_of_the_client_entereventgrp_drives_the_spawn_enter_event() {
    let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
    let mut client = killer_assets::client::Client::open(&root).unwrap();
    let tables = client.tables().unwrap();
    assert_eq!(tables.entereventgrp.len(), 1120);
    let mut rises = [0usize; 3];
    let mut classes = std::collections::BTreeSet::new();
    let mut sounds = std::collections::BTreeSet::new();
    for (id, row) in &tables.entereventgrp {
        assert!(row.is_enter() && !row.is_exit(), "{id}");
        assert!(l2_dat::entereventgrp::exit_of(&tables.entereventgrp, *id).is_none());
        let mut world = World { enter_events: tables.entereventgrp.clone(), ..World::default() };
        world.apply_world(&WorldEvent::NpcInfo(npc(5, *id)));
        let effects = world.effects.take_enter_effects();
        let played = world.effects.take_enter_sounds();
        assert_eq!(effects.len(), usize::from(!row.effect_name.is_empty()), "{id}");
        assert_eq!(played.len(), usize::from(killer_world::enter_event::sound_of(row).is_some()), "{id}");
        assert_eq!(world.rising.contains_key(&5), row.is_rise != 0, "{id}");
        rises[row.is_rise as usize] += 1;
        let entity = &world.entities[&5];
        match row.is_rise {
            1 => assert_eq!(entity.position.z, 3000.0 - 46.0),
            2 => assert_eq!(entity.position.z, 3000.0 + 46.0),
            _ => assert_eq!(entity.position.z, 3000.0),
        }
        for _ in 0..240 {
            world.tick(1.0 / 60.0);
        }
        assert_eq!(world.entities[&5].position.z, 3000.0, "{id}");
        assert!(world.rising.is_empty(), "{id}");
        classes.extend(effects.into_iter().map(|effect| effect.class));
        sounds.extend(played.into_iter().map(|sound| sound.name));
    }
    assert_eq!(rises, [36, 1060, 24]);
    assert_eq!(classes.len(), 3);
    for class in &classes {
        assert!(client.emitter_class(class).is_some(), "{class}");
    }
    assert_eq!(sounds.len(), 2);
    for name in &sounds {
        assert!(client.sound(name).is_some(), "{name}");
    }
}
