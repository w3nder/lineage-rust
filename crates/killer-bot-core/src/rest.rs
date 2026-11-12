use crate::condition::percent;
use crate::env::Env;
use crate::options::{BotOptions, names_list};
use killer_world::{EntityKind, NetCommand};

pub fn rest(env: &Env<'_>, options: &BotOptions) -> Option<NetCommand> {
    let rest = &options.rest;
    let me = env.me()?;
    if me.dead {
        return None;
    }
    if rest.follow_rest {
        let leader = env.world.entities.values().find(|entity| matches!(entity.kind, EntityKind::Player { .. }) && entity.name.eq_ignore_ascii_case(rest.follow_rest_name.trim()));
        if let Some(leader) = leader {
            if leader.sitting && !me.sitting {
                return Some(NetCommand::ChangeWaitType { standing: false });
            }
            if !leader.sitting && me.sitting {
                return Some(NetCommand::ChangeWaitType { standing: true });
            }
            return None;
        }
    }
    let hp = percent(me.hp);
    let mp = percent(me.mp);
    let busy = me.in_combat || env.world.targeted().is_some_and(|target| !target.dead);
    if !me.sitting {
        let low = rest.below_hp.zip(hp).is_some_and(|(below, now)| now < below) || rest.below_mp.zip(mp).is_some_and(|(below, now)| now < below);
        return (low && !busy).then_some(NetCommand::ChangeWaitType { standing: false });
    }
    let hp_done = rest.until_hp.is_none_or(|until| hp.is_some_and(|now| now >= until));
    let mp_done = rest.until_mp.is_none_or(|until| mp.is_some_and(|now| now >= until));
    let has_goal = rest.until_hp.is_some() || rest.until_mp.is_some();
    let attacked = me.in_combat;
    ((has_goal && hp_done && mp_done) || attacked).then_some(NetCommand::ChangeWaitType { standing: true })
}

pub fn is_listed(names: &str, name: &str) -> bool {
    names_list(names).iter().any(|listed| listed.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::names::IdNames;
    use crate::testkit::{Vitals, character, user_info, world_of};
    use killer_world::World;
    use std::collections::HashSet;
    use std::time::Duration;

    fn env<'a>(world: &'a World, sets: &'a HashSet<i32>) -> Env<'a> {
        Env { world, names: &IdNames, now: Duration::ZERO, sweepable: sets, spoiled: sets }
    }

    fn world(hp: i32, mp: i32) -> World {
        world_of(&[user_info(1, "Wender", &Vitals { hp, max_hp: 1000, mp, max_mp: 1000, ..Vitals::full() }, [0, 0, 0])])
    }

    #[test]
    fn it_sits_below_hp_or_mp_when_idle_and_stands_when_both_goals_are_met() {
        let none = HashSet::new();
        let mut options = BotOptions::default();
        options.rest.below_hp = Some(30);
        options.rest.until_hp = Some(80);
        assert_eq!(rest(&env(&world(200, 900), &none), &options), Some(NetCommand::ChangeWaitType { standing: false }));
        assert_eq!(rest(&env(&world(500, 900), &none), &options), None);
        let mut sitting = world(500, 900);
        sitting.me.as_mut().unwrap().sitting = true;
        assert_eq!(rest(&env(&sitting, &none), &options), None);
        sitting.me.as_mut().unwrap().hp = Some((850, 1000));
        assert_eq!(rest(&env(&sitting, &none), &options), Some(NetCommand::ChangeWaitType { standing: true }));
    }

    #[test]
    fn mp_rest_and_the_in_combat_guard_work_like_hp() {
        let none = HashSet::new();
        let mut options = BotOptions::default();
        options.rest.below_mp = Some(20);
        options.rest.until_mp = Some(70);
        let mut world = world(900, 100);
        assert_eq!(rest(&env(&world, &none), &options), Some(NetCommand::ChangeWaitType { standing: false }));
        world.me.as_mut().unwrap().in_combat = true;
        assert_eq!(rest(&env(&world, &none), &options), None);
        world.me.as_mut().unwrap().sitting = true;
        assert_eq!(rest(&env(&world, &none), &options), Some(NetCommand::ChangeWaitType { standing: true }));
    }

    #[test]
    fn follow_rest_copies_the_leader_sitting_and_standing() {
        let none = HashSet::new();
        let mut options = BotOptions::default();
        options.rest.follow_rest = true;
        options.rest.follow_rest_name = "Tank".into();
        let mut world = world(900, 900);
        world.apply(&character(2, "Tank", [50, 0, 0]));
        assert_eq!(rest(&env(&world, &none), &options), None);
        world.entities.get_mut(&2).unwrap().sitting = true;
        assert_eq!(rest(&env(&world, &none), &options), Some(NetCommand::ChangeWaitType { standing: false }));
        world.me.as_mut().unwrap().sitting = true;
        assert_eq!(rest(&env(&world, &none), &options), None);
        world.entities.get_mut(&2).unwrap().sitting = false;
        assert_eq!(rest(&env(&world, &none), &options), Some(NetCommand::ChangeWaitType { standing: true }));
    }

    #[test]
    fn name_lists_match_ignoring_case() {
        assert!(is_listed("Tank, Healer", "healer"));
        assert!(!is_listed("Tank", "Healer"));
        assert!(!is_listed("", "Tank"));
    }
}
