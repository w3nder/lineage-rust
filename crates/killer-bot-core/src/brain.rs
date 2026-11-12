use crate::alerts::{Alert, Alerts, logout};
use crate::dead::{Dead, DeadAction};
use crate::env::{Env, Timers};
use crate::fight::Fight;
use crate::options::BotOptions;
use crate::targeting::Blacklist;
use crate::{party, rest, tables};
use killer_world::NetCommand;
use killer_world::chat::ChatLine;

#[derive(Debug, Clone, PartialEq)]
pub enum BrainEvent {
    StopBotting,
    Alert(Alert),
    LoggedOut(String),
}

#[derive(Debug, Default)]
pub struct Decision {
    pub command: Option<NetCommand>,
    pub events: Vec<BrainEvent>,
}

#[derive(Default)]
pub struct Brain {
    pub options: BotOptions,
    timers: Timers,
    blacklist: Blacklist,
    fight: Fight,
    dead: Dead,
    alerts: Alerts,
    shots: Option<i32>,
}

impl Brain {
    pub fn new(options: BotOptions) -> Brain {
        Brain { options, ..Brain::default() }
    }

    pub fn set_options(&mut self, options: BotOptions) {
        self.options = options;
        self.timers.clear();
        self.shots = None;
    }

    pub fn blacklisted(&self) -> u32 {
        self.blacklist.added
    }

    pub fn blacklist_target(&mut self, object_id: i32, now: std::time::Duration) {
        self.blacklist.add(object_id, now);
    }

    pub fn decide(&mut self, env: &Env<'_>, new_chat: &[ChatLine]) -> Decision {
        let mut decision = Decision::default();
        for alert in self.alerts.step(env, &self.options.alerts, new_chat) {
            decision.events.push(BrainEvent::Alert(alert));
        }
        if let Some((command, reason)) = logout(env, &self.options.logout) {
            decision.events.push(BrainEvent::LoggedOut(reason));
            decision.command = Some(command);
            return decision;
        }
        if let Some(action) = self.dead.step(env, &self.options.dead) {
            match action {
                DeadAction::StopBotting => decision.events.push(BrainEvent::StopBotting),
                DeadAction::Restart(point) => decision.command = Some(NetCommand::RestartPoint { point }),
                DeadAction::Logout => decision.command = Some(NetCommand::Logout),
            }
            if decision.command.is_some() {
                return decision;
            }
        }
        if env.me().is_some_and(|me| me.dead) {
            decision.command = party::answer_dialogs(env, &self.options);
            return decision;
        }
        decision.command = self.pick(env);
        decision
    }

    fn pick(&mut self, env: &Env<'_>) -> Option<NetCommand> {
        let options = &self.options;
        party::answer_dialogs(env, options)
            .or_else(|| tables::auto_shots(env, options, &mut self.shots))
            .or_else(|| tables::items(env, options, &mut self.timers))
            .or_else(|| tables::buffs_heals(env, options, &mut self.timers))
            .or_else(|| tables::toggles(env, options, &mut self.timers))
            .or_else(|| rest::rest(env, options))
            .or_else(|| party::invites(env, options, &mut self.timers))
            .or_else(|| party::follow(env, options, &mut self.timers))
            .or_else(|| {
                let fighting = env.world.targeted().is_some_and(|target| !target.dead);
                if fighting { tables::combat(env, options, &mut self.timers) } else { None }
            })
            .or_else(|| self.fight.step(env, options, &mut self.blacklist, &mut self.timers))
            .or_else(|| self.fight.pickup(env, options, &mut self.timers))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::names::IdNames;
    use crate::options::{BuffHeal, Trait};
    use crate::testkit::{Vitals, npc_info, user_info, world_of};
    use killer_world::World;
    use l2_net::game::SkillEntry;
    use std::collections::HashSet;
    use std::time::Duration;

    fn env<'a>(world: &'a World, sets: &'a HashSet<i32>, now: u64) -> Env<'a> {
        Env { world, names: &IdNames, now: Duration::from_secs(now), sweepable: sets, spoiled: sets }
    }

    fn world(hp: i32) -> World {
        let mut world = world_of(&[user_info(1, "Wender", &Vitals { hp, max_hp: 1000, ..Vitals::full() }, [0, 0, 0])]);
        world.skills.push(SkillEntry { passive: false, level: 1, id: 1204 });
        world
    }

    #[test]
    fn heals_come_before_fighting_and_fighting_before_picking_up() {
        let none = HashSet::new();
        let mut options = BotOptions::default();
        options.attack.active_target = true;
        options.attack.active_attack = true;
        options.pickup.pickup = true;
        options.buffs_heals.push(BuffHeal { skill_id: 1204, trait_: Trait::Hp, below_percent: 50, delay_seconds: 60, mp_above: 10, ..Default::default() });
        let mut brain = Brain::new(options);
        let mut hurt = world(200);
        hurt.apply(&npc_info(9, 1, "Wolf", true, [100, 0, 0]));
        let first = brain.decide(&env(&hurt, &none, 0), &[]);
        assert_eq!(first.command, Some(NetCommand::UseSkill { skill_id: 1204, ctrl: false, shift: false }));
        let second = brain.decide(&env(&hurt, &none, 1), &[]);
        assert_eq!(second.command, Some(NetCommand::Action { object_id: 9, origin: [0, 0, 0], shift: false }));
    }

    #[test]
    fn nothing_happens_with_default_options() {
        let none = HashSet::new();
        let mut brain = Brain::new(BotOptions::default());
        let mut world = world(900);
        world.apply(&npc_info(9, 1, "Wolf", true, [100, 0, 0]));
        assert_eq!(brain.decide(&env(&world, &none, 0), &[]).command, None);
    }

    #[test]
    fn logout_conditions_beat_everything_else_and_report_the_reason() {
        let none = HashSet::new();
        let mut options = BotOptions::default();
        options.logout.hp_below = Some(30);
        options.attack.active_target = true;
        let mut brain = Brain::new(options);
        let mut hurt = world(100);
        hurt.apply(&npc_info(9, 1, "Wolf", true, [100, 0, 0]));
        let decision = brain.decide(&env(&hurt, &none, 0), &[]);
        assert_eq!(decision.command, Some(NetCommand::Logout));
        assert_eq!(decision.events, vec![BrainEvent::LoggedOut("HP".into())]);
    }

    #[test]
    fn death_stops_botting_once_then_returns_when_allowed_and_ignores_the_rest() {
        let none = HashSet::new();
        let mut options = BotOptions::default();
        options.dead.toggle_botting = true;
        options.dead.return_enabled = true;
        options.dead.return_delay_seconds = 5;
        options.attack.active_target = true;
        let mut brain = Brain::new(options);
        let mut dead = world(0);
        dead.me.as_mut().unwrap().dead = true;
        dead.death = Some(l2_net::game::Die { object_id: 1, to_village: true, to_hideout: false, to_castle: false, to_siege_hq: false, sweepable: false, to_fixed_spot: false });
        dead.apply(&npc_info(9, 1, "Wolf", true, [100, 0, 0]));
        assert_eq!(brain.decide(&env(&dead, &none, 0), &[]).events, vec![BrainEvent::StopBotting]);
        assert_eq!(brain.decide(&env(&dead, &none, 2), &[]).command, None);
        assert_eq!(brain.decide(&env(&dead, &none, 6), &[]).command, Some(NetCommand::RestartPoint { point: 0 }));
    }

    #[test]
    fn changing_the_options_resets_the_delays() {
        let none = HashSet::new();
        let mut options = BotOptions::default();
        options.buffs_heals.push(BuffHeal { skill_id: 1204, trait_: Trait::Always, delay_seconds: 600, mp_above: 10, ..Default::default() });
        let mut brain = Brain::new(options.clone());
        let world = world(900);
        assert!(brain.decide(&env(&world, &none, 0), &[]).command.is_some());
        assert!(brain.decide(&env(&world, &none, 1), &[]).command.is_none());
        brain.set_options(options);
        assert!(brain.decide(&env(&world, &none, 2), &[]).command.is_some());
    }
}
