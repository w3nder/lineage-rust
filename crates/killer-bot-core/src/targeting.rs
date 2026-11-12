use crate::env::Env;
use crate::options::{AliveFilter, AttackableFilter, BotOptions, BoxFilter, KillSteal, KindFilter};
use killer_world::{Entity, EntityKind};
use std::collections::HashMap;
use std::time::Duration;

pub const SEARCH_RANGE: f32 = 1500.0;
pub const BLACKLIST_TIME: Duration = Duration::from_secs(60);

#[derive(Debug, Default)]
pub struct Blacklist {
    until: HashMap<i32, Duration>,
    pub added: u32,
}

impl Blacklist {
    pub fn add(&mut self, object_id: i32, now: Duration) {
        self.added += 1;
        self.until.insert(object_id, now + BLACKLIST_TIME);
    }

    pub fn contains(&self, object_id: i32, now: Duration) -> bool {
        self.until.get(&object_id).is_some_and(|until| *until > now)
    }
}

fn is_chest(env: &Env<'_>, entity: &Entity) -> bool {
    let EntityKind::Npc { npc_id, .. } = entity.kind else { return false };
    entity.name.to_lowercase().contains("chest") || env.names.npc(npc_id).to_lowercase().contains("chest")
}

fn is_summon(env: &Env<'_>, entity: &Entity) -> bool {
    if !matches!(entity.kind, EntityKind::Npc { .. }) || entity.title.is_empty() {
        return false;
    }
    env.me().is_some_and(|me| me.name.eq_ignore_ascii_case(&entity.title))
        || env.world.entities.values().any(|other| matches!(other.kind, EntityKind::Player { .. }) && other.name.eq_ignore_ascii_case(&entity.title))
}

fn steals(env: &Env<'_>, entity: &Entity) -> bool {
    let Some(me) = env.me() else { return false };
    entity.target.is_some_and(|target| {
        target != me.object_id
            && !env.world.party.members.iter().any(|member| member.object_id == target)
            && env.world.pet.as_ref().is_none_or(|pet| pet.info.object_id != target)
    })
}

pub fn accepts(env: &Env<'_>, options: &BotOptions, entity: &Entity) -> bool {
    let targeting = &options.targeting;
    let Some(me) = env.me() else { return false };
    if entity.object_id == me.object_id {
        return false;
    }
    let (is_npc, attackable) = match entity.kind {
        EntityKind::Npc { attackable, .. } => (true, attackable),
        EntityKind::Player { .. } => (false, entity.pvp_flag != 0 || entity.karma > 0),
        EntityKind::Item { .. } => return false,
    };
    let kind_ok = match targeting.kind {
        KindFilter::Npcs => is_npc,
        KindFilter::Players => !is_npc,
        KindFilter::Both => true,
    };
    let attackable_ok = match targeting.attackable {
        AttackableFilter::Attackable => attackable || !is_npc,
        AttackableFilter::Invincible => !attackable,
        AttackableFilter::Both => true,
    };
    let alive_ok = match targeting.alive {
        AliveFilter::Alive => !entity.dead,
        AliveFilter::Dead => entity.dead,
        AliveFilter::Both => true,
    };
    let inside = options.polygon.contains(entity.position.x, entity.position.y, entity.position.z, me.position.z);
    let box_ok = match targeting.in_box {
        BoxFilter::In => inside,
        BoxFilter::NotIn => !inside,
        BoxFilter::Both => true,
    };
    let steal_ok = match targeting.kill_steal {
        KillSteal::Dont => !steals(env, entity),
        KillSteal::Only => steals(env, entity),
        KillSteal::Both => true,
    };
    if !(kind_ok && attackable_ok && alive_ok && box_ok && steal_ok) {
        return false;
    }
    if is_npc {
        let EntityKind::Npc { npc_id, .. } = entity.kind else { return false };
        let listed = options.do_not.npcs.contains(&npc_id);
        if options.do_not.attack_only_listed != listed {
            return false;
        }
        if (options.do_not.ignore_summons && is_summon(env, entity)) || (options.do_not.ignore_chests && is_chest(env, entity)) {
            return false;
        }
    }
    true
}

pub fn pick(env: &Env<'_>, options: &BotOptions, blacklist: &Blacklist) -> Option<i32> {
    let me = env.me()?;
    let priority = |entity: &Entity| match entity.kind {
        EntityKind::Player { class_id, .. } => options.player_sorting.priorities.iter().find(|row| row.class_id == class_id).map_or(0, |row| row.priority),
        _ => 0,
    };
    let threshold = options.player_sorting.equidistant_threshold.max(1) as f32;
    env.world
        .entities
        .values()
        .filter(|entity| !blacklist.contains(entity.object_id, env.now) && accepts(env, options, entity))
        .map(|entity| (entity.position.distance(me.position), entity))
        .filter(|(distance, entity)| *distance <= SEARCH_RANGE && (matches!(entity.kind, EntityKind::Npc { .. }) || (entity.position.z - me.position.z).abs() as i32 <= options.player_sorting.max_z_difference))
        .min_by(|a, b| {
            let band = |distance: f32| (distance / threshold).floor() as i64;
            band(a.0).cmp(&band(b.0)).then(priority(b.1).cmp(&priority(a.1))).then(a.0.total_cmp(&b.0)).then(a.1.object_id.cmp(&b.1.object_id))
        })
        .map(|(_, entity)| entity.object_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::names::IdNames;
    use crate::options::{ClassPriority, Point, Polygon};
    use crate::testkit::{Vitals, character, npc_info, user_info, world_of};
    use killer_world::World;
    use std::collections::HashSet;

    fn base() -> World {
        world_of(&[user_info(1, "Wender", &Vitals::full(), [0, 0, 0])])
    }

    fn env<'a>(world: &'a World, sets: &'a HashSet<i32>) -> Env<'a> {
        Env { world, names: &IdNames, now: Duration::ZERO, sweepable: sets, spoiled: sets }
    }

    #[test]
    fn picks_the_nearest_living_attackable_npc_and_skips_peaceful_ones() {
        let mut world = base();
        world.apply(&npc_info(10, 1, "Far", true, [800, 0, 0]));
        world.apply(&npc_info(11, 2, "Near", true, [200, 0, 0]));
        world.apply(&npc_info(12, 3, "Peaceful", false, [50, 0, 0]));
        let none = HashSet::new();
        assert_eq!(pick(&env(&world, &none), &BotOptions::default(), &Blacklist::default()), Some(11));
    }

    #[test]
    fn the_do_not_list_excludes_unless_attack_only_turns_it_into_an_allow_list() {
        let mut world = base();
        world.apply(&npc_info(10, 100, "Wolf", true, [100, 0, 0]));
        world.apply(&npc_info(11, 200, "Bear", true, [300, 0, 0]));
        let none = HashSet::new();
        let mut options = BotOptions::default();
        options.do_not.npcs = vec![100];
        assert_eq!(pick(&env(&world, &none), &options, &Blacklist::default()), Some(11));
        options.do_not.attack_only_listed = true;
        assert_eq!(pick(&env(&world, &none), &options, &Blacklist::default()), Some(10));
    }

    #[test]
    fn chests_summons_and_the_blacklist_are_ignored_when_asked() {
        let mut world = base();
        world.apply(&npc_info(10, 1, "Treasure Chest", true, [100, 0, 0]));
        world.apply(&npc_info(11, 2, "Pet", true, [150, 0, 0]));
        world.entities.get_mut(&11).unwrap().title = "Wender".into();
        world.apply(&npc_info(12, 3, "Wolf", true, [400, 0, 0]));
        let none = HashSet::new();
        let mut options = BotOptions::default();
        assert_eq!(pick(&env(&world, &none), &options, &Blacklist::default()), Some(10));
        options.do_not.ignore_chests = true;
        options.do_not.ignore_summons = true;
        assert_eq!(pick(&env(&world, &none), &options, &Blacklist::default()), Some(12));
        let mut blacklist = Blacklist::default();
        blacklist.add(12, Duration::ZERO);
        assert_eq!(pick(&env(&world, &none), &options, &blacklist), None);
        assert!(!blacklist.contains(12, BLACKLIST_TIME + Duration::from_secs(1)));
    }

    #[test]
    fn the_polygon_decides_which_mobs_are_in_the_box() {
        let mut world = base();
        world.apply(&npc_info(10, 1, "Inside", true, [100, 0, 0]));
        world.apply(&npc_info(11, 1, "Outside", true, [1200, 0, 0]));
        let none = HashSet::new();
        let mut options = BotOptions { polygon: Polygon { points: Polygon::regular((0, 0), 4, 500.0, 45.0), z_range: 1000 }, ..Default::default() };
        assert_eq!(pick(&env(&world, &none), &options, &Blacklist::default()), Some(10));
        options.targeting.in_box = BoxFilter::NotIn;
        assert_eq!(pick(&env(&world, &none), &options, &Blacklist::default()), Some(11));
        options.targeting.in_box = BoxFilter::Both;
        assert_eq!(pick(&env(&world, &none), &options, &Blacklist::default()), Some(10));
        let _ = Point::default();
    }

    #[test]
    fn kill_steal_filters_use_who_the_mob_is_attacking() {
        let mut world = base();
        world.apply(&npc_info(10, 1, "Busy", true, [100, 0, 0]));
        world.entities.get_mut(&10).unwrap().target = Some(77);
        world.apply(&npc_info(11, 1, "Free", true, [300, 0, 0]));
        let none = HashSet::new();
        let mut options = BotOptions::default();
        assert_eq!(pick(&env(&world, &none), &options, &Blacklist::default()), Some(11));
        options.targeting.kill_steal = KillSteal::Only;
        assert_eq!(pick(&env(&world, &none), &options, &Blacklist::default()), Some(10));
        options.targeting.kill_steal = KillSteal::Both;
        assert_eq!(pick(&env(&world, &none), &options, &Blacklist::default()), Some(10));
        world.entities.get_mut(&10).unwrap().target = Some(1);
        options.targeting.kill_steal = KillSteal::Dont;
        assert_eq!(pick(&env(&world, &none), &options, &Blacklist::default()), Some(10));
    }

    #[test]
    fn players_can_be_targeted_and_class_priority_breaks_ties_inside_the_equidistant_band() {
        let mut world = base();
        world.apply(&character(20, "Mage", [200, 0, 0]));
        world.apply(&character(21, "Tank", [210, 0, 0]));
        world.entities.get_mut(&20).unwrap().pvp_flag = 1;
        world.entities.get_mut(&21).unwrap().pvp_flag = 1;
        if let EntityKind::Player { class_id, .. } = &mut world.entities.get_mut(&21).unwrap().kind {
            *class_id = 88;
        }
        let none = HashSet::new();
        let mut options = BotOptions::default();
        options.targeting.kind = KindFilter::Players;
        assert_eq!(pick(&env(&world, &none), &options, &Blacklist::default()), Some(20));
        options.player_sorting.priorities.push(ClassPriority { class_id: 88, priority: 5 });
        assert_eq!(pick(&env(&world, &none), &options, &Blacklist::default()), Some(21));
    }

    #[test]
    fn alive_and_dead_filters_pick_corpses_when_asked() {
        let mut world = base();
        world.apply(&npc_info(10, 1, "Corpse", true, [100, 0, 0]));
        world.entities.get_mut(&10).unwrap().dead = true;
        let none = HashSet::new();
        let mut options = BotOptions::default();
        assert_eq!(pick(&env(&world, &none), &options, &Blacklist::default()), None);
        options.targeting.alive = AliveFilter::Dead;
        assert_eq!(pick(&env(&world, &none), &options, &Blacklist::default()), Some(10));
    }
}
