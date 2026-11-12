use crate::env::{Env, Timers};
use crate::options::BotOptions;
use crate::targeting::{self, Blacklist};
use killer_world::{EntityKind, ItemOrigin, NetCommand};
use std::collections::HashSet;
use std::time::Duration;

const MOVE_PAUSE: Duration = Duration::from_secs(1);
const SELECT_PAUSE: Duration = Duration::from_millis(500);
const ATTACK_PAUSE: Duration = Duration::from_secs(2);
const STUCK_AFTER: Duration = Duration::from_secs(6);
const STANDING_STILL: f32 = 5.0;

#[derive(Debug, Default)]
pub struct Fight {
    attempts: u32,
    tracked: Option<i32>,
    last_hp: Option<i32>,
    spoil_cast: HashSet<i32>,
    swept: HashSet<i32>,
    pub mine: HashSet<i32>,
    moved_at: Option<(glam::Vec3, Duration)>,
}

fn toward(env: &Env<'_>, to: glam::Vec3, stop: f32) -> Option<NetCommand> {
    let me = env.me()?;
    let distance = me.position.distance(to);
    let target = if distance > stop && distance > 0.0 { me.position + (to - me.position) * ((distance - stop * 0.5) / distance) } else { to };
    Some(NetCommand::MoveTo { target: target.to_array().map(|value| value as i32), origin: env.origin()? })
}

impl Fight {
    pub fn reset(&mut self) {
        *self = Fight::default();
    }

    fn unstuck(&mut self, env: &Env<'_>, options: &BotOptions, blacklist: &mut Blacklist) -> Option<NetCommand> {
        if !options.attack.auto_unstuck {
            return None;
        }
        let me = env.me()?;
        let moving = me.dest.is_some();
        match self.moved_at {
            Some((position, since)) if moving && position.distance(me.position) < STANDING_STILL => {
                if env.now.saturating_sub(since) >= STUCK_AFTER {
                    self.moved_at = None;
                    if let Some(target) = env.world.target {
                        blacklist.add(target, env.now);
                    }
                    return Some(NetCommand::TargetCancel);
                }
            }
            _ => self.moved_at = Some((me.position, env.now)),
        }
        None
    }

    pub fn step(&mut self, env: &Env<'_>, options: &BotOptions, blacklist: &mut Blacklist, timers: &mut Timers) -> Option<NetCommand> {
        let me = env.me()?;
        let attack = &options.attack;
        if me.dead || me.sitting || !(attack.active_target || attack.active_attack) {
            return None;
        }
        if let Some(command) = self.unstuck(env, options, blacklist) {
            return Some(command);
        }
        match env.world.targeted() {
            Some(target) if !target.dead => self.engage(env, options, target, blacklist, timers),
            Some(target) => self.after_kill(env, options, target.object_id, timers),
            None => self.acquire(env, options, blacklist, timers),
        }
    }

    fn engage(&mut self, env: &Env<'_>, options: &BotOptions, target: &killer_world::Entity, blacklist: &mut Blacklist, timers: &mut Timers) -> Option<NetCommand> {
        let attack = &options.attack;
        let me = env.me()?;
        if self.tracked != Some(target.object_id) {
            self.tracked = Some(target.object_id);
            self.attempts = 0;
            self.last_hp = target.hp.map(|(current, _)| current);
        }
        if !attack.active_attack {
            return None;
        }
        self.mine.insert(target.object_id);
        if let Some(command) = self.spoil(env, options, target, timers) {
            return Some(command);
        }
        let distance = me.position.distance(target.position);
        if (attack.move_before_attack || attack.smart_move_before_attack) && distance > attack.move_range && timers.ready("move", env.now, MOVE_PAUSE) {
            timers.mark("move", env.now);
            return toward(env, target.position, attack.move_range);
        }
        let already = env.world.combat.auto_attack_target == Some(target.object_id) || me.auto_attack;
        if already || !timers.ready("attack", env.now, ATTACK_PAUSE) {
            return None;
        }
        let hp = target.hp.map(|(current, _)| current);
        if self.attempts > 0 && hp == self.last_hp && self.attempts >= options.advanced.blacklist_tries && options.attack.auto_blacklist {
            blacklist.add(target.object_id, env.now);
            self.attempts = 0;
            return Some(NetCommand::TargetCancel);
        }
        self.attempts += 1;
        timers.mark("attack", env.now);
        Some(NetCommand::AttackRequest { object_id: target.object_id, origin: env.origin()?, shift: false })
    }

    fn spoil(&mut self, env: &Env<'_>, options: &BotOptions, target: &killer_world::Entity, timers: &mut Timers) -> Option<NetCommand> {
        let spoil = &options.spoil;
        if !spoil.auto_spoil || !matches!(target.kind, EntityKind::Npc { .. }) || env.spoiled.contains(&target.object_id) {
            return None;
        }
        if !spoil.until_success && self.spoil_cast.contains(&target.object_id) {
            return None;
        }
        if env.my_mp() <= spoil.mp_above as i32 || !timers.ready("spoil", env.now, Duration::from_secs(2)) {
            return None;
        }
        let name = if spoil.spoil_crush { "Spoil Crush" } else if spoil.plunder { "Plunder" } else { "Spoil" };
        let skill_id = env.skill_by_name(name).filter(|id| env.ready(*id))?;
        timers.mark("spoil", env.now);
        self.spoil_cast.insert(target.object_id);
        Some(NetCommand::UseSkill { skill_id, ctrl: false, shift: false })
    }

    fn after_kill(&mut self, env: &Env<'_>, options: &BotOptions, target: i32, timers: &mut Timers) -> Option<NetCommand> {
        if options.spoil.auto_sweep
            && env.sweepable.contains(&target)
            && !self.swept.contains(&target)
            && let Some(skill_id) = env.skill_by_name("Sweeper").filter(|id| env.ready(*id))
        {
            self.swept.insert(target);
            return Some(NetCommand::UseSkill { skill_id, ctrl: false, shift: false });
        }
        if options.attack.cancel_target_on_dead {
            return Some(NetCommand::TargetCancel);
        }
        let _ = timers;
        None
    }

    fn acquire(&mut self, env: &Env<'_>, options: &BotOptions, blacklist: &Blacklist, timers: &mut Timers) -> Option<NetCommand> {
        let attack = &options.attack;
        if attack.active_target
            && let Some(id) = targeting::pick(env, options, blacklist)
        {
            let entity = env.world.entity(id)?;
            let me = env.me()?;
            if attack.move_before_targeting && me.position.distance(entity.position) > attack.move_range && timers.ready("move", env.now, MOVE_PAUSE) {
                timers.mark("move", env.now);
                return toward(env, entity.position, attack.move_range);
            }
            if timers.ready("select", env.now, SELECT_PAUSE) {
                timers.mark("select", env.now);
                return env.select(id);
            }
            return None;
        }
        self.no_mobs(env, options, timers)
    }

    fn no_mobs(&mut self, env: &Env<'_>, options: &BotOptions, timers: &mut Timers) -> Option<NetCommand> {
        let rule = &options.no_mobs;
        let me = env.me()?;
        if !rule.move_to_loc || (rule.out_of_combat && me.in_combat) {
            return None;
        }
        let home = glam::Vec3::new(rule.x as f32, rule.y as f32, rule.z as f32);
        if me.position.distance(home) > rule.leash && timers.ready("home", env.now, Duration::from_secs(2)) {
            timers.mark("home", env.now);
            return Some(NetCommand::MoveTo { target: [rule.x, rule.y, rule.z], origin: env.origin()? });
        }
        None
    }

    pub fn pickup(&self, env: &Env<'_>, options: &BotOptions, timers: &mut Timers) -> Option<NetCommand> {
        let pickup = &options.pickup;
        if !pickup.pickup || !timers.ready("pickup", env.now, Duration::from_millis(600)) {
            return None;
        }
        if pickup.after_attack && env.world.targeted().is_some_and(|target| !target.dead) {
            return None;
        }
        let me = env.me()?;
        let best = env
            .world
            .entities
            .values()
            .filter_map(|entity| {
                let EntityKind::Item { item_id, .. } = entity.kind else { return None };
                if options.do_not.items.contains(&item_id) != options.do_not.pick_only_listed {
                    return None;
                }
                if options.do_not.ignore_unknown_items && env.names.item(item_id).starts_with('#') {
                    return None;
                }
                if pickup.only_mine && !matches!(entity.item_origin, Some(ItemOrigin::Drop { source_id, .. }) if self.mine.contains(&source_id)) {
                    return None;
                }
                let distance = entity.position.distance(me.position);
                (distance <= pickup.range).then_some((distance, entity.object_id))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        let (_, object_id) = best?;
        timers.mark("pickup", env.now);
        env.select(object_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::names::IdNames;
    use crate::testkit::{Vitals, ground_item, npc_info, user_info, world_of};
    use killer_world::World;
    use l2_net::game::SkillEntry;

    struct Spoilers;

    impl crate::names::Names for Spoilers {
        fn item(&self, id: i32) -> String {
            format!("Item {id}")
        }
        fn skill(&self, id: i32, _level: i32) -> String {
            id.to_string()
        }
        fn npc(&self, id: i32) -> String {
            id.to_string()
        }
        fn class(&self, id: i32) -> String {
            id.to_string()
        }
        fn find_skill(&self, name: &str) -> Option<i32> {
            match name {
                "Spoil" => Some(254),
                "Sweeper" => Some(42),
                _ => None,
            }
        }
    }

    fn base() -> World {
        let mut world = world_of(&[user_info(1, "Wender", &Vitals::full(), [0, 0, 0])]);
        world.skills = vec![SkillEntry { passive: false, level: 1, id: 254 }, SkillEntry { passive: false, level: 1, id: 42 }];
        world
    }

    fn run(world: &World, options: &BotOptions, fight: &mut Fight, now: u64) -> Option<NetCommand> {
        let none = HashSet::new();
        let env = Env { world, names: &IdNames, now: Duration::from_secs(now), sweepable: &none, spoiled: &none };
        fight.step(&env, options, &mut Blacklist::default(), &mut Timers::default())
    }

    fn make_env<'a>(world: &'a World, none: &'a HashSet<i32>) -> Env<'a> {
        Env { world, names: &Spoilers, now: Duration::ZERO, sweepable: none, spoiled: none }
    }

    fn active() -> BotOptions {
        let mut options = BotOptions::default();
        options.attack.active_target = true;
        options.attack.active_attack = true;
        options
    }

    #[test]
    fn with_both_active_flags_off_the_autofighter_does_nothing() {
        let mut world = base();
        world.apply(&npc_info(9, 1, "Wolf", true, [100, 0, 0]));
        assert_eq!(run(&world, &BotOptions::default(), &mut Fight::default(), 0), None);
    }

    #[test]
    fn it_selects_a_target_then_attacks_it() {
        let mut world = base();
        world.apply(&npc_info(9, 1, "Wolf", true, [100, 0, 0]));
        let options = active();
        let mut fight = Fight::default();
        assert_eq!(run(&world, &options, &mut fight, 0), Some(NetCommand::Action { object_id: 9, origin: [0, 0, 0], shift: false }));
        world.target = Some(9);
        assert_eq!(run(&world, &options, &mut fight, 1), Some(NetCommand::AttackRequest { object_id: 9, origin: [0, 0, 0], shift: false }));
        world.combat.auto_attack_target = Some(9);
        assert_eq!(run(&world, &options, &mut fight, 5), None);
    }

    #[test]
    fn active_target_without_active_attack_only_selects() {
        let mut world = base();
        world.apply(&npc_info(9, 1, "Wolf", true, [100, 0, 0]));
        let mut options = BotOptions::default();
        options.attack.active_target = true;
        let mut fight = Fight::default();
        assert!(matches!(run(&world, &options, &mut fight, 0), Some(NetCommand::Action { .. })));
        world.target = Some(9);
        assert_eq!(run(&world, &options, &mut fight, 1), None);
    }

    #[test]
    fn move_before_attack_walks_toward_a_far_target_first() {
        let mut world = base();
        world.apply(&npc_info(9, 1, "Wolf", true, [600, 0, 0]));
        world.target = Some(9);
        let mut options = active();
        options.attack.move_before_attack = true;
        options.attack.move_range = 150.0;
        let command = run(&world, &options, &mut Fight::default(), 0);
        assert_eq!(command, Some(NetCommand::MoveTo { target: [525, 0, 0], origin: [0, 0, 0] }));
        options.attack.move_before_attack = false;
        assert!(matches!(run(&world, &options, &mut Fight::default(), 0), Some(NetCommand::AttackRequest { .. })));
    }

    #[test]
    fn auto_spoil_casts_the_named_skill_once_per_target_unless_until_success() {
        let mut world = base();
        world.apply(&npc_info(9, 1, "Wolf", true, [100, 0, 0]));
        world.target = Some(9);
        let mut options = active();
        options.spoil.auto_spoil = true;
        options.spoil.mp_above = 50;
        let none = HashSet::new();
        let spoiler_env = |now: u64| Env { world: &world, names: &Spoilers, now: Duration::from_secs(now), sweepable: &none, spoiled: &none };
        let mut fight = Fight::default();
        let first = fight.step(&spoiler_env(0), &options, &mut Blacklist::default(), &mut Timers::default());
        assert_eq!(first, Some(NetCommand::UseSkill { skill_id: 254, ctrl: false, shift: false }));
        let second = fight.step(&spoiler_env(5), &options, &mut Blacklist::default(), &mut Timers::default());
        assert!(matches!(second, Some(NetCommand::AttackRequest { .. })));
        options.spoil.until_success = true;
        let third = fight.step(&spoiler_env(9), &options, &mut Blacklist::default(), &mut Timers::default());
        assert_eq!(third, Some(NetCommand::UseSkill { skill_id: 254, ctrl: false, shift: false }));
    }

    #[test]
    fn auto_sweep_uses_the_sweeper_on_a_sweepable_corpse_then_cancels_the_target() {
        let mut world = base();
        world.apply(&npc_info(9, 1, "Wolf", true, [100, 0, 0]));
        world.entities.get_mut(&9).unwrap().dead = true;
        world.target = Some(9);
        let mut options = active();
        options.spoil.auto_sweep = true;
        options.attack.cancel_target_on_dead = true;
        let sweepable: HashSet<i32> = [9].into_iter().collect();
        let none = HashSet::new();
        let env = |now: u64| Env { world: &world, names: &Spoilers, now: Duration::from_secs(now), sweepable: &sweepable, spoiled: &none };
        let mut fight = Fight::default();
        assert_eq!(fight.step(&env(0), &options, &mut Blacklist::default(), &mut Timers::default()), Some(NetCommand::UseSkill { skill_id: 42, ctrl: false, shift: false }));
        assert_eq!(fight.step(&env(1), &options, &mut Blacklist::default(), &mut Timers::default()), Some(NetCommand::TargetCancel));
    }

    #[test]
    fn with_no_mobs_it_walks_back_home_beyond_the_leash_only_out_of_combat_when_asked() {
        let mut world = base();
        world.me.as_mut().unwrap().position = glam::Vec3::new(1000.0, 0.0, 0.0);
        let mut options = active();
        options.no_mobs.move_to_loc = true;
        options.no_mobs.leash = 100.0;
        assert_eq!(run(&world, &options, &mut Fight::default(), 0), Some(NetCommand::MoveTo { target: [0, 0, 0], origin: [1000, 0, 0] }));
        options.no_mobs.out_of_combat = true;
        world.me.as_mut().unwrap().in_combat = true;
        assert_eq!(run(&world, &options, &mut Fight::default(), 0), None);
        world.me.as_mut().unwrap().position = glam::Vec3::new(50.0, 0.0, 0.0);
        world.me.as_mut().unwrap().in_combat = false;
        assert_eq!(run(&world, &options, &mut Fight::default(), 0), None);
    }

    #[test]
    fn a_stuck_walker_blacklists_its_target_after_standing_still() {
        let mut world = base();
        world.apply(&npc_info(9, 1, "Wolf", true, [600, 0, 0]));
        world.target = Some(9);
        world.me.as_mut().unwrap().dest = Some(glam::Vec3::new(500.0, 0.0, 0.0));
        let mut options = active();
        options.attack.auto_unstuck = true;
        let none = HashSet::new();
        let mut fight = Fight::default();
        let mut blacklist = Blacklist::default();
        let mut timers = Timers::default();
        let mut step = |now: u64| fight.step(&Env { world: &world, names: &IdNames, now: Duration::from_secs(now), sweepable: &none, spoiled: &none }, &options, &mut blacklist, &mut timers);
        assert!(step(0).is_some());
        assert_ne!(step(3), Some(NetCommand::TargetCancel));
        assert_eq!(step(7), Some(NetCommand::TargetCancel));
        assert!(blacklist.contains(9, Duration::from_secs(8)));
    }

    #[test]
    fn pickup_obeys_range_lists_after_attack_and_only_mine() {
        let mut world = base();
        world.apply(&ground_item(20, 57, [100, 0, 0]));
        world.apply(&ground_item(21, 1060, [200, 0, 0]));
        let none = HashSet::new();
        let mut options = BotOptions::default();
        assert_eq!(Fight::default().pickup(&make_env(&world, &none), &options, &mut Timers::default()), None);
        options.pickup.pickup = true;
        assert_eq!(Fight::default().pickup(&make_env(&world, &none), &options, &mut Timers::default()), Some(NetCommand::Action { object_id: 20, origin: [0, 0, 0], shift: false }));
        options.do_not.items = vec![57];
        assert_eq!(Fight::default().pickup(&make_env(&world, &none), &options, &mut Timers::default()), Some(NetCommand::Action { object_id: 21, origin: [0, 0, 0], shift: false }));
        options.do_not.pick_only_listed = true;
        assert_eq!(Fight::default().pickup(&make_env(&world, &none), &options, &mut Timers::default()), Some(NetCommand::Action { object_id: 20, origin: [0, 0, 0], shift: false }));
        options.do_not = Default::default();
        options.pickup.range = 150.0;
        assert_eq!(Fight::default().pickup(&make_env(&world, &none), &options, &mut Timers::default()), Some(NetCommand::Action { object_id: 20, origin: [0, 0, 0], shift: false }));
        options.pickup.only_mine = true;
        assert_eq!(Fight::default().pickup(&make_env(&world, &none), &options, &mut Timers::default()), None);
        world.entities.get_mut(&20).unwrap().item_origin = Some(ItemOrigin::Drop { source_id: 9, source_position: Some(glam::Vec3::ZERO) });
        let mut fight = Fight::default();
        fight.mine.insert(9);
        assert!(fight.pickup(&make_env(&world, &none), &options, &mut Timers::default()).is_some());
        options.pickup.only_mine = false;
        options.pickup.after_attack = true;
        world.apply(&npc_info(9, 1, "Wolf", true, [100, 0, 0]));
        world.target = Some(9);
        assert_eq!(Fight::default().pickup(&make_env(&world, &none), &options, &mut Timers::default()), None);
    }
}
