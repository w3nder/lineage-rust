use crate::env::Env;
use crate::options::{AlertOptions, LogoutOptions, names_list};
use crate::rest::is_listed;
use killer_world::chat::{CHAT_SYSTEM, ChatLine};
use killer_world::{EntityKind, NetCommand};
use l2_net::game::packets::{CHAT_ALL, CHAT_FRIEND, CHAT_TELL};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Alert {
    HpLow,
    MpLow,
    CpLow,
    Player(String),
    Clan(String),
    PrivateMessage,
    WhiteChat,
    FriendChat,
}

impl Alert {
    pub fn text(&self, english: bool) -> String {
        match (self, english) {
            (Alert::HpLow, false) => "HP baixo".into(),
            (Alert::HpLow, true) => "Low HP".into(),
            (Alert::MpLow, false) => "MP baixo".into(),
            (Alert::MpLow, true) => "Low MP".into(),
            (Alert::CpLow, false) => "CP baixo".into(),
            (Alert::CpLow, true) => "Low CP".into(),
            (Alert::Player(name), false) => format!("jogador perto: {name}"),
            (Alert::Player(name), true) => format!("player nearby: {name}"),
            (Alert::Clan(name), false) => format!("clã perto: {name}"),
            (Alert::Clan(name), true) => format!("clan nearby: {name}"),
            (Alert::PrivateMessage, false) => "mensagem privada".into(),
            (Alert::PrivateMessage, true) => "private message".into(),
            (Alert::WhiteChat, false) => "chat branco".into(),
            (Alert::WhiteChat, true) => "white chat".into(),
            (Alert::FriendChat, false) => "chat de amigo".into(),
            (Alert::FriendChat, true) => "friend chat".into(),
        }
    }
}

fn stat_below(value: Option<(i32, i32)>, limit: Option<u8>) -> bool {
    limit.zip(crate::condition::percent(value)).is_some_and(|(limit, now)| now < limit)
}

fn nearby(env: &Env<'_>, names: &str, ignore_party: bool, by_clan: bool) -> Vec<String> {
    let listed = names_list(names);
    env.world
        .entities
        .values()
        .filter(|entity| matches!(entity.kind, EntityKind::Player { .. }))
        .filter(|entity| !(ignore_party && env.world.party.members.iter().any(|member| member.object_id == entity.object_id)))
        .filter_map(|entity| {
            if by_clan {
                let name = env.world.pledges.get(&entity.clan_id).map(|record| record.name.clone()).filter(|name| !name.is_empty())?;
                (listed.is_empty() || is_listed(names, &name)).then_some(name)
            } else {
                (listed.is_empty() || is_listed(names, &entity.name)).then(|| entity.name.clone())
            }
        })
        .collect()
}

#[derive(Debug, Default)]
pub struct Alerts {
    active: HashSet<Alert>,
}

impl Alerts {
    pub fn step(&mut self, env: &Env<'_>, options: &AlertOptions, chat: &[ChatLine]) -> Vec<Alert> {
        let Some(me) = env.me() else { return Vec::new() };
        let mut current: HashSet<Alert> = HashSet::new();
        if stat_below(me.hp, options.hp_below) {
            current.insert(Alert::HpLow);
        }
        if stat_below(me.mp, options.mp_below) {
            current.insert(Alert::MpLow);
        }
        if stat_below(me.cp, options.cp_below) {
            current.insert(Alert::CpLow);
        }
        if options.player {
            current.extend(nearby(env, &options.player_names, options.player_ignore_party, false).into_iter().map(Alert::Player));
        }
        if options.clan {
            current.extend(nearby(env, &options.clan_names, options.clan_ignore_party, true).into_iter().map(Alert::Clan));
        }
        let fresh: Vec<Alert> = current.iter().filter(|alert| !self.active.contains(alert)).cloned().collect();
        self.active = current;
        let mut out = fresh;
        for line in chat.iter().filter(|line| line.kind != CHAT_SYSTEM) {
            let alert = match line.kind {
                CHAT_TELL if options.private_message => Alert::PrivateMessage,
                CHAT_ALL if options.white_chat && !line.sender.eq_ignore_ascii_case(&me.name) => Alert::WhiteChat,
                CHAT_FRIEND if options.friend_chat => Alert::FriendChat,
                _ => continue,
            };
            out.push(alert);
        }
        out.sort_by_key(|alert| alert.text(true));
        out
    }
}

pub fn logout(env: &Env<'_>, options: &LogoutOptions) -> Option<(NetCommand, String)> {
    let me = env.me()?;
    let reason = if stat_below(me.hp, options.hp_below) {
        "HP"
    } else if stat_below(me.mp, options.mp_below) {
        "MP"
    } else if stat_below(me.cp, options.cp_below) {
        "CP"
    } else if options.player && !nearby(env, &options.player_names, false, false).is_empty() {
        "player"
    } else if options.clan && !nearby(env, &options.clan_names, false, true).is_empty() {
        "clan"
    } else {
        return None;
    };
    Some((NetCommand::Logout, reason.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::names::IdNames;
    use crate::testkit::{Vitals, character, user_info, world_of};
    use killer_world::World;
    use std::time::Duration;

    fn env<'a>(world: &'a World, sets: &'a HashSet<i32>) -> Env<'a> {
        Env { world, names: &IdNames, now: Duration::ZERO, sweepable: sets, spoiled: sets }
    }

    fn world(hp: i32) -> World {
        world_of(&[user_info(1, "Wender", &Vitals { hp, max_hp: 1000, ..Vitals::full() }, [0, 0, 0])])
    }

    #[test]
    fn a_stat_alert_fires_once_when_it_crosses_and_again_after_recovering() {
        let none = HashSet::new();
        let options = AlertOptions { hp_below: Some(50), ..Default::default() };
        let mut alerts = Alerts::default();
        assert!(alerts.step(&env(&world(900), &none), &options, &[]).is_empty());
        assert_eq!(alerts.step(&env(&world(300), &none), &options, &[]), vec![Alert::HpLow]);
        assert!(alerts.step(&env(&world(200), &none), &options, &[]).is_empty());
        assert!(alerts.step(&env(&world(900), &none), &options, &[]).is_empty());
        assert_eq!(alerts.step(&env(&world(100), &none), &options, &[]), vec![Alert::HpLow]);
    }

    #[test]
    fn player_alerts_respect_the_name_list_and_the_party_ignore() {
        let none = HashSet::new();
        let mut world = world(900);
        world.apply(&character(2, "Rival", [10, 0, 0]));
        world.apply(&character(3, "Friend", [20, 0, 0]));
        world.party.members.push(l2_net::game::PartyMember { object_id: 3, name: "Friend".into(), cp: 0, max_cp: 0, hp: 1, max_hp: 1, mp: 0, max_mp: 0, level: 1, class_id: 0 });
        let mut options = AlertOptions { player: true, player_ignore_party: true, ..Default::default() };
        assert_eq!(Alerts::default().step(&env(&world, &none), &options, &[]), vec![Alert::Player("Rival".into())]);
        options.player_names = "Friend".into();
        assert!(Alerts::default().step(&env(&world, &none), &options, &[]).is_empty());
        options.player_ignore_party = false;
        assert_eq!(Alerts::default().step(&env(&world, &none), &options, &[]), vec![Alert::Player("Friend".into())]);
    }

    #[test]
    fn chat_alerts_cover_private_white_and_friend_messages_but_not_my_own_lines() {
        let none = HashSet::new();
        let world = world(900);
        let options = AlertOptions { private_message: true, white_chat: true, friend_chat: true, ..Default::default() };
        let lines = [
            ChatLine::new(CHAT_TELL, "Rival", "oi"),
            ChatLine::new(CHAT_ALL, "Rival", "alô"),
            ChatLine::new(CHAT_ALL, "Wender", "eu mesmo"),
            ChatLine::new(CHAT_FRIEND, "Amigo", "ei"),
            ChatLine::new(CHAT_SYSTEM, "", "#12"),
        ];
        let found = Alerts::default().step(&env(&world, &none), &options, &lines);
        assert_eq!(found.len(), 3);
        assert!(found.contains(&Alert::PrivateMessage) && found.contains(&Alert::WhiteChat) && found.contains(&Alert::FriendChat));
        assert!(Alerts::default().step(&env(&world, &none), &AlertOptions::default(), &lines).is_empty());
    }

    #[test]
    fn logout_triggers_on_low_stats_or_a_listed_player_and_says_why() {
        let none = HashSet::new();
        let options = LogoutOptions { hp_below: Some(30), ..Default::default() };
        assert_eq!(logout(&env(&world(900), &none), &options), None);
        assert_eq!(logout(&env(&world(200), &none), &options), Some((NetCommand::Logout, "HP".to_owned())));
        let mut crowded = world(900);
        crowded.apply(&character(2, "Rival", [10, 0, 0]));
        let by_player = LogoutOptions { player: true, player_names: "rival".into(), ..Default::default() };
        assert_eq!(logout(&env(&crowded, &none), &by_player), Some((NetCommand::Logout, "player".to_owned())));
        let other = LogoutOptions { player: true, player_names: "alguem".into(), ..Default::default() };
        assert_eq!(logout(&env(&crowded, &none), &other), None);
    }

    #[test]
    fn alert_texts_exist_in_both_languages() {
        for alert in [Alert::HpLow, Alert::MpLow, Alert::CpLow, Alert::Player("X".into()), Alert::Clan("Y".into()), Alert::PrivateMessage, Alert::WhiteChat, Alert::FriendChat] {
            assert!(!alert.text(false).is_empty() && !alert.text(true).is_empty());
            assert_ne!(alert.text(false), alert.text(true));
        }
    }
}
