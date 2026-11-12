use crate::env::{Env, Timers};
use crate::options::{BotOptions, FollowStyle, names_list};
use crate::rest::is_listed;
use killer_world::state::Dialog;
use killer_world::{EntityKind, NetCommand};
use l2_net::game::SystemMessageArg;
use std::time::Duration;

fn related(env: &Env<'_>, name: &str, party: bool, clan: bool, alliance: bool) -> bool {
    if party && env.world.party.members.iter().any(|member| member.name.eq_ignore_ascii_case(name)) {
        return true;
    }
    let Some(me) = env.me() else { return false };
    let Some(other) = env.world.entities.values().find(|entity| entity.name.eq_ignore_ascii_case(name)) else { return false };
    (clan && me.clan_id != 0 && other.clan_id == me.clan_id) || (alliance && me.ally_id != 0 && other.ally_id == me.ally_id)
}

pub fn answer_dialogs(env: &Env<'_>, options: &BotOptions) -> Option<NetCommand> {
    let party = &options.party;
    for (index, dialog) in env.world.dialogs.pending.iter().enumerate() {
        let _ = index;
        match dialog {
            Dialog::JoinParty(ask) if party.accept_party => {
                let listed = party.accept_party_names.trim().is_empty() || is_listed(&party.accept_party_names, &ask.requestor_name);
                if listed || related(env, &ask.requestor_name, false, party.accept_party_clan, party.accept_party_alliance) {
                    return Some(NetCommand::AnswerJoinParty { accept: true });
                }
            }
            Dialog::Confirm(message) if message.id == 1510 && party.accept_rez => {
                let requester = message.args.iter().find_map(|arg| match arg {
                    SystemMessageArg::Text(text) => Some(text.as_str()),
                    _ => None,
                });
                let allowed = match requester {
                    Some(name) => {
                        party.accept_rez_names.trim().is_empty() && !(party.accept_rez_party || party.accept_rez_alliance || party.accept_rez_clan)
                            || is_listed(&party.accept_rez_names, name)
                            || related(env, name, party.accept_rez_party, party.accept_rez_clan, party.accept_rez_alliance)
                    }
                    None => true,
                };
                if allowed {
                    return Some(NetCommand::DlgAnswer { message_id: message.id, answer: 1 });
                }
            }
            _ => {}
        }
    }
    None
}

pub fn invites(env: &Env<'_>, options: &BotOptions, timers: &mut Timers) -> Option<NetCommand> {
    let party = &options.party;
    if party.drop_party_if_leader && env.me().is_some_and(|me| env.world.party.leader_id == me.object_id) && !env.world.party.members.is_empty() && timers.ready("drop", env.now, Duration::from_secs(5)) {
        timers.mark("drop", env.now);
        return Some(NetCommand::DismissParty);
    }
    if !party.auto_invite || !timers.ready("invite", env.now, Duration::from_secs(10)) {
        return None;
    }
    let name = names_list(&party.auto_invite_names).into_iter().find(|name| {
        env.world.entities.values().any(|entity| matches!(entity.kind, EntityKind::Player { .. }) && entity.name.eq_ignore_ascii_case(name))
            && !env.world.party.members.iter().any(|member| member.name.eq_ignore_ascii_case(name))
    })?;
    timers.mark("invite", env.now);
    Some(NetCommand::JoinParty { name, item_distribution: party.loot_type.distribution() })
}

pub fn follow(env: &Env<'_>, options: &BotOptions, timers: &mut Timers) -> Option<NetCommand> {
    let rule = &options.follow;
    if !rule.active_follow {
        return None;
    }
    let me = env.me()?;
    let leader = env.world.entities.values().find(|entity| matches!(entity.kind, EntityKind::Player { .. }) && entity.name.eq_ignore_ascii_case(rule.name.trim()))?;
    if (rule.target || rule.attack)
        && let Some(target) = leader.target
        && env.world.target != Some(target)
        && env.world.entity(target).is_some_and(|entity| !entity.dead)
        && timers.ready("assist", env.now, Duration::from_millis(500))
    {
        timers.mark("assist", env.now);
        return env.select(target);
    }
    if rule.attack
        && let Some(target) = env.world.targeted()
        && !target.dead
        && Some(target.object_id) == leader.target
        && rule.instant_attack
        && me.position.distance(target.position) <= rule.distance.max(150.0)
        && !me.auto_attack
        && timers.ready("assist-attack", env.now, Duration::from_secs(2))
    {
        timers.mark("assist-attack", env.now);
        return Some(NetCommand::AttackRequest { object_id: target.object_id, origin: env.origin()?, shift: false });
    }
    let distance = me.position.distance(leader.position);
    if distance <= rule.distance || !timers.ready("follow", env.now, Duration::from_millis(u64::from(options.advanced.autofollow_delay).max(700))) {
        return None;
    }
    timers.mark("follow", env.now);
    let target = match rule.style {
        FollowStyle::L2Net => leader.position,
        FollowStyle::Walker => me.position + (leader.position - me.position) * ((distance - rule.distance * 0.5) / distance),
    };
    Some(NetCommand::MoveTo { target: target.to_array().map(|value| value as i32), origin: env.origin()? })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::names::IdNames;
    use crate::testkit::{Vitals, character, npc_info, user_info, world_of};
    use killer_world::World;
    use l2_net::game::{AskJoinParty, PartyMember, SystemMessage};
    use std::collections::HashSet;

    fn env<'a>(world: &'a World, sets: &'a HashSet<i32>, now: u64) -> Env<'a> {
        Env { world, names: &IdNames, now: Duration::from_secs(now), sweepable: sets, spoiled: sets }
    }

    fn base() -> World {
        world_of(&[user_info(1, "Wender", &Vitals::full(), [0, 0, 0])])
    }

    fn member(id: i32, name: &str) -> PartyMember {
        PartyMember { object_id: id, name: name.into(), cp: 0, max_cp: 0, hp: 900, max_hp: 1000, mp: 0, max_mp: 0, level: 40, class_id: 0 }
    }

    #[test]
    fn party_invites_are_accepted_only_when_enabled_and_from_listed_or_related_players() {
        let none = HashSet::new();
        let mut world = base();
        world.dialogs.push(Dialog::JoinParty(AskJoinParty { requestor_name: "Tank".into(), item_distribution: 0 }));
        let mut options = BotOptions::default();
        assert_eq!(answer_dialogs(&env(&world, &none, 0), &options), None);
        options.party.accept_party = true;
        assert_eq!(answer_dialogs(&env(&world, &none, 0), &options), Some(NetCommand::AnswerJoinParty { accept: true }));
        options.party.accept_party_names = "Healer".into();
        assert_eq!(answer_dialogs(&env(&world, &none, 0), &options), None);
        options.party.accept_party_clan = true;
        world.me.as_mut().unwrap().clan_id = 5;
        world.apply(&character(2, "Tank", [10, 0, 0]));
        world.entities.get_mut(&2).unwrap().clan_id = 5;
        assert_eq!(answer_dialogs(&env(&world, &none, 0), &options), Some(NetCommand::AnswerJoinParty { accept: true }));
    }

    #[test]
    fn a_revive_request_is_accepted_by_name_or_relation() {
        let none = HashSet::new();
        let mut world = base();
        world.dialogs.push(Dialog::Confirm(SystemMessage { id: 1510, args: vec![SystemMessageArg::Text("Healer".into())] }));
        let mut options = BotOptions::default();
        assert_eq!(answer_dialogs(&env(&world, &none, 0), &options), None);
        options.party.accept_rez = true;
        assert_eq!(answer_dialogs(&env(&world, &none, 0), &options), Some(NetCommand::DlgAnswer { message_id: 1510, answer: 1 }));
        options.party.accept_rez_party = true;
        assert_eq!(answer_dialogs(&env(&world, &none, 0), &options), None);
        world.party.members.push(member(3, "Healer"));
        assert_eq!(answer_dialogs(&env(&world, &none, 0), &options), Some(NetCommand::DlgAnswer { message_id: 1510, answer: 1 }));
    }

    #[test]
    fn auto_invite_asks_listed_nearby_players_not_yet_in_the_party_with_the_loot_type() {
        let none = HashSet::new();
        let mut world = base();
        world.apply(&character(2, "Tank", [10, 0, 0]));
        world.apply(&character(3, "Healer", [20, 0, 0]));
        world.party.members.push(member(3, "Healer"));
        let mut options = BotOptions::default();
        options.party.auto_invite = true;
        options.party.auto_invite_names = "Healer, Tank, Ausente".into();
        options.party.loot_type = crate::options::LootType::ByTurn;
        let mut timers = Timers::default();
        assert_eq!(invites(&env(&world, &none, 0), &options, &mut timers), Some(NetCommand::JoinParty { name: "Tank".into(), item_distribution: 3 }));
        assert_eq!(invites(&env(&world, &none, 1), &options, &mut timers), None);
    }

    #[test]
    fn drop_party_if_leader_dismisses_when_i_lead() {
        let none = HashSet::new();
        let mut world = base();
        world.party.leader_id = 1;
        world.party.members.push(member(3, "Healer"));
        let mut options = BotOptions::default();
        options.party.drop_party_if_leader = true;
        assert_eq!(invites(&env(&world, &none, 0), &options, &mut Timers::default()), Some(NetCommand::DismissParty));
        world.party.leader_id = 3;
        assert_eq!(invites(&env(&world, &none, 0), &options, &mut Timers::default()), None);
    }

    #[test]
    fn active_follow_keeps_the_distance_in_walker_style_and_goes_to_the_leader_in_l2net_style() {
        let none = HashSet::new();
        let mut world = base();
        world.apply(&character(2, "Tank", [1000, 0, 0]));
        let mut options = BotOptions::default();
        options.follow.active_follow = true;
        options.follow.name = "tank".into();
        options.follow.distance = 100.0;
        assert_eq!(follow(&env(&world, &none, 0), &options, &mut Timers::default()), Some(NetCommand::MoveTo { target: [950, 0, 0], origin: [0, 0, 0] }));
        options.follow.style = FollowStyle::L2Net;
        assert_eq!(follow(&env(&world, &none, 0), &options, &mut Timers::default()), Some(NetCommand::MoveTo { target: [1000, 0, 0], origin: [0, 0, 0] }));
        world.me.as_mut().unwrap().position = glam::Vec3::new(950.0, 0.0, 0.0);
        assert_eq!(follow(&env(&world, &none, 0), &options, &mut Timers::default()), None);
        options.follow.active_follow = false;
        world.me.as_mut().unwrap().position = glam::Vec3::ZERO;
        assert_eq!(follow(&env(&world, &none, 0), &options, &mut Timers::default()), None);
    }

    #[test]
    fn follow_target_and_attack_copy_the_leader_target_and_attack_it_instantly() {
        let none = HashSet::new();
        let mut world = base();
        world.apply(&character(2, "Tank", [50, 0, 0]));
        world.apply(&npc_info(9, 1, "Wolf", true, [100, 0, 0]));
        world.entities.get_mut(&2).unwrap().target = Some(9);
        let mut options = BotOptions::default();
        options.follow.active_follow = true;
        options.follow.name = "Tank".into();
        options.follow.attack = true;
        options.follow.instant_attack = true;
        let mut timers = Timers::default();
        assert_eq!(follow(&env(&world, &none, 0), &options, &mut timers), Some(NetCommand::Action { object_id: 9, origin: [0, 0, 0], shift: false }));
        world.target = Some(9);
        assert_eq!(follow(&env(&world, &none, 1), &options, &mut timers), Some(NetCommand::AttackRequest { object_id: 9, origin: [0, 0, 0], shift: false }));
    }
}
