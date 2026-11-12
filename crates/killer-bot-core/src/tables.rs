use crate::condition::percent;
use crate::env::{Env, Timers};
use crate::options::{BotOptions, Comparison, ToggleTrait, Trait, names_list};
use crate::traits::{Subject, holds, subject};
use killer_world::{EntityKind, NetCommand};
use killer_world::state::Dialog;
use std::time::Duration;

fn my_percent(env: &Env<'_>, which: Trait) -> Option<u8> {
    let me = env.me()?;
    match which {
        Trait::Hp => percent(me.hp),
        Trait::Mp => percent(me.mp),
        Trait::Cp => percent(me.cp),
        _ => None,
    }
}

fn find_subject(env: &Env<'_>, name: &str) -> Option<Subject> {
    let by_entity = env.world.entities.values().find(|entity| matches!(entity.kind, EntityKind::Player { .. }) && entity.name.eq_ignore_ascii_case(name)).map(|entity| entity.object_id);
    let by_party = env.world.party.members.iter().find(|member| member.name.eq_ignore_ascii_case(name)).map(|member| member.object_id);
    let me = env.me().filter(|me| me.name.eq_ignore_ascii_case(name)).map(|me| me.object_id);
    subject(env.world, me.or(by_entity).or(by_party)?)
}

pub fn buffs_heals(env: &Env<'_>, options: &BotOptions, timers: &mut Timers) -> Option<NetCommand> {
    let me = env.me()?;
    for (index, rule) in options.buffs_heals.iter().enumerate() {
        if !rule.on || rule.skill_id <= 0 || !env.ready(rule.skill_id) || env.my_mp() <= i64::from(rule.mp_above) as i32 {
            continue;
        }
        let key = format!("bh{index}");
        if !timers.ready(&key, env.now, Duration::from_secs(u64::from(rule.delay_seconds))) {
            continue;
        }
        let mut names = names_list(&rule.names);
        if names.is_empty() {
            names.push(me.name.clone());
        }
        let subjects: Vec<Subject> = names.iter().filter_map(|name| find_subject(env, name)).collect();
        let target = subjects.into_iter().find(|candidate| {
            let in_range = env.world.entity(candidate.object_id).is_none_or(|entity| entity.position.distance(me.position) <= options.party.buff_range) || candidate.object_id == me.object_id;
            in_range && holds(rule.trait_, rule.below_percent, candidate, env.world, env.names)
        });
        let Some(target) = target else { continue };
        let on_self = target.object_id == me.object_id;
        if (!on_self || rule.need_target) && env.world.target != Some(target.object_id) {
            return env.select(target.object_id);
        }
        timers.mark(&key, env.now);
        return Some(NetCommand::UseSkill { skill_id: rule.skill_id, ctrl: options.party.buff_with_control, shift: options.party.buff_with_shift });
    }
    None
}

pub fn toggles(env: &Env<'_>, options: &BotOptions, timers: &mut Timers) -> Option<NetCommand> {
    for (index, rule) in options.toggles.iter().enumerate() {
        if !rule.on || rule.skill_id <= 0 || !env.knows(rule.skill_id) {
            continue;
        }
        let which = match rule.trait_ {
            ToggleTrait::Hp => Trait::Hp,
            ToggleTrait::Mp => Trait::Mp,
        };
        let Some(current) = my_percent(env, which) else { continue };
        let active = env.world.buffs.iter().any(|buff| buff.skill_id == rule.skill_id);
        let wants_on = current < rule.below_percent;
        let wants_off = current > rule.above_percent;
        let key = format!("tg{index}");
        if ((wants_on && !active) || (wants_off && active)) && timers.ready(&key, env.now, Duration::from_secs(1)) {
            timers.mark(&key, env.now);
            return Some(NetCommand::UseSkill { skill_id: rule.skill_id, ctrl: false, shift: false });
        }
    }
    None
}

pub fn items(env: &Env<'_>, options: &BotOptions, timers: &mut Timers) -> Option<NetCommand> {
    let me = subject(env.world, env.me()?.object_id)?;
    for (index, rule) in options.items.iter().enumerate() {
        if !rule.on || rule.item_id <= 0 {
            continue;
        }
        let key = format!("it{index}");
        if !timers.ready(&key, env.now, Duration::from_millis(u64::from(rule.delay_ms))) || !holds(rule.trait_, rule.below_percent, &me, env.world, env.names) {
            continue;
        }
        if let Some(object_id) = env.item_object(rule.item_id) {
            timers.mark(&key, env.now);
            return Some(NetCommand::UseItem { object_id, ctrl: false });
        }
    }
    None
}

pub fn auto_shots(env: &Env<'_>, options: &BotOptions, enabled: &mut Option<i32>) -> Option<NetCommand> {
    let wanted = (options.auto_shots.active && options.auto_shots.item_id > 0 && env.item_object(options.auto_shots.item_id).is_some()).then_some(options.auto_shots.item_id);
    match (wanted, *enabled) {
        (Some(item_id), None) => {
            *enabled = Some(item_id);
            Some(NetCommand::AutoSoulShot { item_id, enable: true })
        }
        (None, Some(item_id)) if !options.auto_shots.active => {
            *enabled = None;
            Some(NetCommand::AutoSoulShot { item_id, enable: false })
        }
        _ => None,
    }
}

pub fn shortcut_command(env: &Env<'_>, page: u8, slot: u8) -> Option<NetCommand> {
    let index = i32::from(page.saturating_sub(1)) * l2_net::game::SHORTCUT_SLOTS_PER_PAGE + i32::from(slot.saturating_sub(1));
    let shortcut = env.world.shortcuts.iter().find(|shortcut| shortcut.slot == index)?;
    match shortcut.kind {
        l2_net::game::ShortcutKind::Skill { skill_id, .. } => env.ready(skill_id).then_some(NetCommand::UseSkill { skill_id, ctrl: false, shift: false }),
        l2_net::game::ShortcutKind::Item { object_id } => env.world.inventory.find(object_id).map(|_| NetCommand::UseItem { object_id, ctrl: false }),
        l2_net::game::ShortcutKind::Action { id } => Some(NetCommand::UseAction { id, ctrl: false, shift: false }),
        _ => None,
    }
}

pub fn combat(env: &Env<'_>, options: &BotOptions, timers: &mut Timers) -> Option<NetCommand> {
    for (index, rule) in options.combat.iter().enumerate() {
        if !rule.on || env.my_mp() <= i64::from(rule.mp_above) as i32 {
            continue;
        }
        let key = format!("cb{index}");
        if !timers.ready(&key, env.now, Duration::from_millis(u64::from(rule.delay_ms))) {
            continue;
        }
        let passes = match rule.trait_ {
            Trait::Always => true,
            Trait::Hp | Trait::Mp | Trait::Cp => my_percent(env, rule.trait_).is_some_and(|current| match rule.comparison {
                Comparison::AtLeast => current >= rule.percent,
                Comparison::AtMost => current <= rule.percent,
            }),
            other => subject(env.world, env.me()?.object_id).is_some_and(|me| holds(other, 0, &me, env.world, env.names)),
        };
        if !passes {
            continue;
        }
        if let Some(command) = shortcut_command(env, rule.page, rule.slot) {
            timers.mark(&key, env.now);
            return Some(command);
        }
    }
    None
}

pub fn pending_revive(_env: &Env<'_>, dialog: &Dialog) -> bool {
    matches!(dialog, Dialog::Confirm(message) if message.id == 1510)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::names::IdNames;
    use crate::options::{BuffHeal, CombatRule, ItemRule, Toggle};
    use crate::testkit::{Vitals, user_info, world_of};
    use killer_world::World;
    use l2_net::game::{InventoryItem, PartyMember, Shortcut, ShortcutKind, SkillEntry};
    use std::collections::HashSet;

    fn make(hp: i32, mp: i32) -> World {
        let mut world = world_of(&[user_info(1, "Wender", &Vitals { hp, max_hp: 1000, mp, max_mp: 1000, ..Vitals::full() }, [0, 0, 0])]);
        world.skills = vec![SkillEntry { passive: false, level: 1, id: 1204 }, SkillEntry { passive: false, level: 1, id: 1011 }, SkillEntry { passive: false, level: 1, id: 1035 }];
        world
    }

    fn env<'a>(world: &'a World, sets: &'a HashSet<i32>, now: u64) -> Env<'a> {
        Env { world, names: &IdNames, now: Duration::from_secs(now), sweepable: sets, spoiled: sets }
    }

    fn stock(object_id: i32, item_id: i32) -> InventoryItem {
        InventoryItem { type1: 4, object_id, item_id, count: 5, type2: 4, custom_type1: 0, equipped: false, body_part: 0, enchant: 0, custom_type2: 0 }
    }

    #[test]
    fn a_self_buff_row_casts_on_me_when_the_trait_holds_and_waits_for_its_delay() {
        let world = make(300, 900);
        let none = HashSet::new();
        let mut options = BotOptions::default();
        options.buffs_heals.push(BuffHeal { skill_id: 1204, trait_: Trait::Hp, below_percent: 60, delay_seconds: 5, mp_above: 100, ..Default::default() });
        let mut timers = Timers::default();
        assert_eq!(buffs_heals(&env(&world, &none, 0), &options, &mut timers), Some(NetCommand::UseSkill { skill_id: 1204, ctrl: false, shift: false }));
        assert_eq!(buffs_heals(&env(&world, &none, 2), &options, &mut timers), None);
        assert!(buffs_heals(&env(&world, &none, 6), &options, &mut timers).is_some());
        assert_eq!(buffs_heals(&env(&make(900, 900), &none, 20), &options, &mut Timers::default()), None);
    }

    #[test]
    fn a_heal_row_with_names_selects_the_party_member_first_then_casts() {
        let mut world = make(900, 900);
        world.party.members.push(PartyMember { object_id: 7, name: "Tank".into(), cp: 0, max_cp: 0, hp: 200, max_hp: 1000, mp: 0, max_mp: 0, level: 40, class_id: 0 });
        world.apply(&crate::testkit::character(7, "Tank", [100, 0, 0]));
        let none = HashSet::new();
        let mut options = BotOptions::default();
        options.buffs_heals.push(BuffHeal { skill_id: 1011, trait_: Trait::Hp, names: "tank".into(), ..Default::default() });
        let mut timers = Timers::default();
        assert_eq!(buffs_heals(&env(&world, &none, 0), &options, &mut timers), Some(NetCommand::Action { object_id: 7, origin: [0, 0, 0], shift: false }));
        world.target = Some(7);
        assert_eq!(buffs_heals(&env(&world, &none, 0), &options, &mut timers), Some(NetCommand::UseSkill { skill_id: 1011, ctrl: false, shift: false }));
        options.party.buff_range = 50.0;
        assert_eq!(buffs_heals(&env(&world, &none, 9), &options, &mut Timers::default()), None);
    }

    #[test]
    fn rows_need_the_skill_ready_and_enough_mp() {
        let mut world = make(300, 50);
        let none = HashSet::new();
        let mut options = BotOptions::default();
        options.buffs_heals.push(BuffHeal { skill_id: 1204, trait_: Trait::Hp, mp_above: 100, ..Default::default() });
        assert_eq!(buffs_heals(&env(&world, &none, 0), &options, &mut Timers::default()), None);
        world.me.as_mut().unwrap().mp = Some((500, 1000));
        world.cooldowns.push(l2_net::game::SkillReuse { skill_id: 1204, level: 1, reuse: 10, remaining: 4 });
        assert_eq!(buffs_heals(&env(&world, &none, 0), &options, &mut Timers::default()), None);
        world.cooldowns.clear();
        assert!(buffs_heals(&env(&world, &none, 0), &options, &mut Timers::default()).is_some());
    }

    #[test]
    fn toggle_rows_turn_on_below_and_off_above_the_thresholds() {
        let mut world = make(900, 200);
        let none = HashSet::new();
        let mut options = BotOptions::default();
        options.toggles.push(Toggle { skill_id: 1035, trait_: ToggleTrait::Mp, below_percent: 30, above_percent: 80, ..Default::default() });
        let use_it = Some(NetCommand::UseSkill { skill_id: 1035, ctrl: false, shift: false });
        assert_eq!(toggles(&env(&world, &none, 0), &options, &mut Timers::default()), use_it);
        world.buffs.push(l2_net::game::Buff { skill_id: 1035, level: 1, duration: -1 });
        assert_eq!(toggles(&env(&world, &none, 0), &options, &mut Timers::default()), None);
        world.me.as_mut().unwrap().mp = Some((900, 1000));
        assert_eq!(toggles(&env(&world, &none, 0), &options, &mut Timers::default()), use_it);
        world.buffs.clear();
        assert_eq!(toggles(&env(&world, &none, 0), &options, &mut Timers::default()), None);
    }

    #[test]
    fn item_rows_use_the_item_when_the_trait_holds_with_stock_and_delay() {
        let mut world = make(300, 900);
        let none = HashSet::new();
        let mut options = BotOptions::default();
        options.items.push(ItemRule { item_id: 1060, trait_: Trait::Hp, below_percent: 40, delay_ms: 2000, ..Default::default() });
        assert_eq!(items(&env(&world, &none, 0), &options, &mut Timers::default()), None);
        world.inventory.items.push(stock(77, 1060));
        let mut timers = Timers::default();
        assert_eq!(items(&env(&world, &none, 0), &options, &mut timers), Some(NetCommand::UseItem { object_id: 77, ctrl: false }));
        assert_eq!(items(&env(&world, &none, 1), &options, &mut timers), None);
        assert!(items(&env(&world, &none, 3), &options, &mut timers).is_some());
    }

    #[test]
    fn auto_shots_toggle_once_per_activation_and_off_when_deactivated() {
        let mut world = make(900, 900);
        world.inventory.items.push(stock(88, 1463));
        let none = HashSet::new();
        let mut options = BotOptions::default();
        options.auto_shots.item_id = 1463;
        let mut enabled = None;
        assert_eq!(auto_shots(&env(&world, &none, 0), &options, &mut enabled), None);
        options.auto_shots.active = true;
        assert_eq!(auto_shots(&env(&world, &none, 0), &options, &mut enabled), Some(NetCommand::AutoSoulShot { item_id: 1463, enable: true }));
        assert_eq!(auto_shots(&env(&world, &none, 0), &options, &mut enabled), None);
        options.auto_shots.active = false;
        assert_eq!(auto_shots(&env(&world, &none, 0), &options, &mut enabled), Some(NetCommand::AutoSoulShot { item_id: 1463, enable: false }));
    }

    #[test]
    fn combat_rows_fire_the_shortcut_slot_content_with_the_comparison_and_delay() {
        let mut world = make(900, 900);
        world.shortcuts.push(Shortcut { slot: 12 + 2, kind: ShortcutKind::Skill { skill_id: 1204, level: 1 }, character_type: 1 });
        let none = HashSet::new();
        let mut options = BotOptions::default();
        options.combat.push(CombatRule { page: 2, slot: 3, trait_: Trait::Hp, comparison: Comparison::AtLeast, percent: 50, delay_ms: 3000, mp_above: 10, ..Default::default() });
        let mut timers = Timers::default();
        assert_eq!(combat(&env(&world, &none, 0), &options, &mut timers), Some(NetCommand::UseSkill { skill_id: 1204, ctrl: false, shift: false }));
        assert_eq!(combat(&env(&world, &none, 1), &options, &mut timers), None);
        options.combat[0].comparison = Comparison::AtMost;
        assert_eq!(combat(&env(&world, &none, 10), &options, &mut Timers::default()), None);
        options.combat[0].slot = 9;
        options.combat[0].comparison = Comparison::AtLeast;
        assert_eq!(combat(&env(&world, &none, 10), &options, &mut Timers::default()), None);
    }

    #[test]
    fn the_revive_confirmation_is_recognised() {
        let message = l2_net::game::SystemMessage { id: 1510, args: Vec::new() };
        let world = make(900, 900);
        let none = HashSet::new();
        assert!(pending_revive(&env(&world, &none, 0), &Dialog::Confirm(message)));
        assert!(!pending_revive(&env(&world, &none, 0), &Dialog::PetName));
    }
}
