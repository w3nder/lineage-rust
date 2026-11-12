use killer_world::{SessionEvent, World, WorldEvent};
use l2_net::game::{CharInfo, CharSelectEntry, NpcInfo, VISIBLE_EQUIP_SLOTS, PAPERDOLL_SLOTS, PaperdollSlot, SpawnItem, UserInfoHead, UserSocial, UserStats};

pub struct Vitals {
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub cp: i32,
    pub max_cp: i32,
}

impl Vitals {
    pub fn full() -> Self {
        Self { hp: 1000, max_hp: 1000, mp: 500, max_mp: 500, cp: 300, max_cp: 300 }
    }
}

pub fn user_info(object_id: i32, name: &str, vitals: &Vitals, at: [i32; 3]) -> SessionEvent {
    SessionEvent::UserInfo(Box::new(UserInfoHead {
        abnormal_effect: 0,
        x: at[0],
        y: at[1],
        z: at[2],
        heading: 0,
        object_id,
        name: name.to_owned(),
        race: 0,
        sex: 0,
        class_id: 0,
        level: 40,
        exp: 0,
        max_hp: vitals.max_hp,
        hp: vitals.hp,
        max_mp: vitals.max_mp,
        mp: vitals.mp,
        pvp_flag: 0,
        karma: 0,
        run_speed: 120,
        walk_speed: 80,
        speed_multiplier: 1.0,
        attack_speed_multiplier: 1.0,
        collision_radius: 8.0,
        collision_height: 23.0,
        hair_style: 0,
        hair_color: 0,
        face: 0,
        builder_level: 0,
        title: String::new(),
        max_cp: vitals.max_cp,
        cp: vitals.cp,
        stats: UserStats::default(),
        paperdoll: [PaperdollSlot { object_id: 0, item_id: 0 }; PAPERDOLL_SLOTS],
        inventory_limit: 80,
        name_color: 0xffffff,
        social: UserSocial::default(),
        enchant_effect: 0,
    }))
}

pub fn npc_info(object_id: i32, npc_id: i32, name: &str, attackable: bool, at: [i32; 3]) -> SessionEvent {
    SessionEvent::World(WorldEvent::NpcInfo(NpcInfo {
        object_id,
        npc_id,
        attackable,
        x: at[0],
        y: at[1],
        z: at[2],
        heading: 0,
        m_atk_speed: 333,
        p_atk_speed: 300,
        run_speed: 120,
        walk_speed: 80,
        speed_multiplier: 1.0,
        attack_speed_multiplier: 1.0,
        collision_radius: 10.0,
        collision_height: 20.0,
        rhand: 0,
        chest: 0,
        lhand: 0,
        name_above: false,
        running: false,
        in_combat: false,
        alike_dead: false,
        spawn_animation: 0,
        name: name.to_owned(),
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
        base_collision_radius: 10.0,
        base_collision_height: 20.0,
    }))
}

pub fn ground_item(object_id: i32, item_id: i32, at: [i32; 3]) -> SessionEvent {
    SessionEvent::World(WorldEvent::SpawnItem(SpawnItem { object_id, item_id, x: at[0], y: at[1], z: at[2], stackable: false, count: 1, heading: 0 }))
}

pub fn world_of(events: &[SessionEvent]) -> World {
    let mut world = World::default();
    for event in events {
        world.apply(event);
    }
    world
}

pub fn character(object_id: i32, name: &str, at: [i32; 3]) -> SessionEvent {
    SessionEvent::World(WorldEvent::CharInfo(Box::new(CharInfo {
        x: at[0],
        y: at[1],
        z: at[2],
        heading: 0,
        object_id,
        name: name.to_owned(),
        race: 0,
        sex: 0,
        class_id: 0,
        equipped: [0; VISIBLE_EQUIP_SLOTS],
        pvp_flag: 0,
        karma: 0,
        m_atk_speed: 333,
        p_atk_speed: 300,
        run_speed: 120,
        walk_speed: 80,
        swim_run_speed: 50,
        swim_walk_speed: 50,
        ground_run_speed: 120,
        ground_walk_speed: 80,
        fly_run_speed: 0,
        fly_walk_speed: 0,
        speed_multiplier: 1.0,
        attack_speed_multiplier: 1.0,
        collision_radius: 8.0,
        collision_height: 23.0,
        hair_style: 0,
        hair_color: 0,
        face: 0,
        title: String::new(),
        clan_id: 0,
        clan_crest_id: 0,
        ally_id: 0,
        ally_crest_id: 0,
        sitting: false,
        running: true,
        in_combat: false,
        alike_dead: false,
        invisible: false,
        mount_type: 0,
        store_type: 0,
        cubics: Vec::new(),
        finding_party: 0,
        abnormal_effect: 0,
        recommendations_left: 0,
        recommendations_received: 0,
        mount_npc_id: 0,
        active_class_id: 0,
        enchant_effect: 0,
        team: 0,
        large_crest_id: 0,
        noble: 0,
        hero: 0,
        fishing: 0,
        fish_x: 0,
        fish_y: 0,
        fish_z: 0,
        name_color: 0xffffff,
    })))
}

pub fn char_entry(name: &str) -> CharSelectEntry {
    CharSelectEntry {
        name: name.to_owned(),
        char_id: 1,
        login: String::new(),
        clan_id: 0,
        sex: 0,
        race: 0,
        base_class_id: 0,
        active: 0,
        x: 0,
        y: 0,
        z: 0,
        hp: 100.0,
        mp: 50.0,
        sp: 0,
        exp: 0,
        level: 40,
        karma: 0,
        paperdoll_objects: [0; 16],
        paperdoll_items: [0; 16],
        hair_style: 0,
        hair_color: 0,
        face: 0,
        max_hp: 100.0,
        max_mp: 50.0,
        delete_days: 0,
        class_id: 0,
        last_used: 0,
        enchant_effect: 0,
    }
}
