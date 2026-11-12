use crate::condition::percent;
use crate::session::BotState;
use killer_world::state::Dialog;
use killer_world::{Entity, EntityKind, World};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Gauge {
    pub current: i32,
    pub max: i32,
    pub percent: Option<u8>,
}

impl Gauge {
    pub fn of(value: Option<(i32, i32)>) -> Gauge {
        let (current, max) = value.unwrap_or((0, 0));
        Gauge { current, max, percent: percent(value) }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TargetView {
    pub object_id: i32,
    pub name: String,
    pub hp: Gauge,
    pub dead: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemberView {
    pub object_id: i32,
    pub name: String,
    pub hp: Gauge,
    pub mp: Gauge,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuffView {
    pub skill_id: i32,
    pub level: u16,
    pub seconds: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillView {
    pub id: i32,
    pub level: i32,
    pub passive: bool,
    pub ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemView {
    pub object_id: i32,
    pub item_id: i32,
    pub count: i32,
    pub equipped: bool,
    pub body_part: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NearbyKind {
    Monster,
    Npc,
    Player,
    Item,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NearbyView {
    pub object_id: i32,
    pub name: String,
    pub title: String,
    pub kind: NearbyKind,
    pub distance: f32,
    pub hp: Option<Gauge>,
    pub position: [f32; 3],
    pub npc_id: i32,
    pub item_id: i32,
    pub count: i32,
    pub class_id: i32,
    pub clan_id: i32,
    pub ally_id: i32,
    pub pvp_flag: i32,
    pub dead: bool,
    pub dest: Option<[f32; 3]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StatsView {
    pub str_: i32,
    pub dex: i32,
    pub con: i32,
    pub int_: i32,
    pub wit: i32,
    pub men: i32,
    pub p_atk: i32,
    pub p_def: i32,
    pub accuracy: i32,
    pub critical: i32,
    pub atk_spd: i32,
    pub m_atk: i32,
    pub m_def: i32,
    pub evasion: i32,
    pub cast_spd: i32,
    pub sp: i32,
    pub load: i32,
    pub max_load: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DeathView {
    pub town: bool,
    pub hideout: bool,
    pub castle: bool,
    pub siege_hq: bool,
    pub fortress: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestionKind {
    Party,
    Pledge,
    Alliance,
    Trade,
    Friend,
    Confirm,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    pub index: usize,
    pub kind: QuestionKind,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerView {
    pub id: u8,
    pub flags: u32,
    pub address: String,
    pub players: u16,
    pub max_players: u16,
    pub online: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterView {
    pub slot: usize,
    pub name: String,
    pub clan_id: i32,
    pub sex: i32,
    pub race: i32,
    pub base_class_id: i32,
    pub active: i32,
    pub position: [i32; 3],
    pub hp: i32,
    pub mp: i32,
    pub sp: i32,
    pub exp: i32,
    pub level: i32,
    pub karma: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Snapshot {
    pub state: BotState,
    pub name: String,
    pub level: i32,
    pub hp: Gauge,
    pub mp: Gauge,
    pub cp: Gauge,
    pub position: [i32; 3],
    pub target: Option<TargetView>,
    pub party: Vec<MemberView>,
    pub buffs: Vec<BuffView>,
    pub skills: Vec<SkillView>,
    pub inventory: Vec<ItemView>,
    pub nearby: Vec<NearbyView>,
    pub adena: i32,
    pub exp: i32,
    pub karma: i32,
    pub stats: StatsView,
    pub death: Option<DeathView>,
    pub questions: Vec<Question>,
    pub clan_id: i32,
    pub ally_id: i32,
    pub title: String,
    pub class_id: i32,
    pub run_speed: i32,
    pub pvp_kills: i32,
    pub pk_kills: i32,
    pub recommend_have: i32,
    pub recommend_left: i32,
    pub servers: Vec<ServerView>,
    pub characters: Vec<CharacterView>,
    pub clan: Option<ClanView>,
    pub gains: crate::stats::Gains,
    pub loot: Vec<crate::stats::LootRow>,
    pub bad_mobs: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClanMember {
    pub name: String,
    pub level: i32,
    pub class_id: i32,
    pub online: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClanView {
    pub name: String,
    pub leader: String,
    pub level: i32,
    pub castle_id: i32,
    pub hideout_id: i32,
    pub ally: String,
    pub at_war: bool,
    pub members: Vec<ClanMember>,
}

fn nearby_kind(entity: &Entity) -> NearbyKind {
    match entity.kind {
        EntityKind::Player { .. } => NearbyKind::Player,
        EntityKind::Npc { attackable: true, .. } => NearbyKind::Monster,
        EntityKind::Npc { .. } => NearbyKind::Npc,
        EntityKind::Item { .. } => NearbyKind::Item,
    }
}

fn question_of(index: usize, dialog: &Dialog) -> Option<Question> {
    let (kind, text) = match dialog {
        Dialog::JoinParty(ask) => (QuestionKind::Party, format!("{} convida para o grupo", ask.requestor_name)),
        Dialog::JoinPledge(ask) => (QuestionKind::Pledge, format!("convite para o clã {}", ask.pledge_name)),
        Dialog::JoinAlliance(ask) => (QuestionKind::Alliance, format!("{} convida para a aliança {}", ask.requestor_name, ask.requestor_ally_name)),
        Dialog::TradeRequest { requestor_id } => (QuestionKind::Trade, format!("pedido de troca de #{requestor_id}")),
        Dialog::FriendRequest { requestor_name } => (QuestionKind::Friend, format!("{requestor_name} quer ser seu amigo")),
        Dialog::Confirm(message) => (QuestionKind::Confirm, format!("confirmação #{}", message.id)),
        _ => return None,
    };
    Some(Question { index, kind, text })
}

impl Snapshot {
    pub fn of(state: BotState, world: &World) -> Snapshot {
        Self::with_login(state, world, &[], &[])
    }

    pub fn with_login(state: BotState, world: &World, servers: &[l2_net::login::GameServer], characters: &[l2_net::game::CharSelectEntry]) -> Snapshot {
        let me = world.me.as_ref();
        let mut nearby: Vec<NearbyView> = match me {
            Some(me) => world
                .entities
                .values()
                .map(|entity| NearbyView {
                    object_id: entity.object_id,
                    name: entity.name.clone(),
                    title: entity.title.clone(),
                    kind: nearby_kind(entity),
                    npc_id: match entity.kind {
                        EntityKind::Npc { npc_id, .. } => npc_id,
                        _ => 0,
                    },
                    item_id: match entity.kind {
                        EntityKind::Item { item_id, .. } => item_id,
                        _ => 0,
                    },
                    count: match entity.kind {
                        EntityKind::Item { count, .. } => count,
                        _ => 0,
                    },
                    class_id: match entity.kind {
                        EntityKind::Player { class_id, .. } => class_id,
                        _ => 0,
                    },
                    clan_id: entity.clan_id,
                    ally_id: entity.ally_id,
                    pvp_flag: entity.pvp_flag,
                    dead: entity.dead,
                    dest: entity.dest.map(|dest| dest.to_array()),
                    distance: entity.position.distance(me.position),
                    hp: entity.hp.map(|hp| Gauge::of(Some(hp))),
                    position: entity.position.to_array(),
                })
                .collect(),
            None => Vec::new(),
        };
        nearby.sort_by(|a, b| a.distance.total_cmp(&b.distance).then(a.object_id.cmp(&b.object_id)));
        Snapshot {
            state,
            name: me.map(|me| me.name.clone()).unwrap_or_default(),
            level: me.map_or(0, |me| me.level),
            hp: Gauge::of(me.and_then(|me| me.hp)),
            mp: Gauge::of(me.and_then(|me| me.mp)),
            cp: Gauge::of(me.and_then(|me| me.cp)),
            position: me.map_or([0; 3], |me| me.position.to_array().map(|value| value as i32)),
            target: world.targeted().map(|target| TargetView { object_id: target.object_id, name: target.name.clone(), hp: Gauge::of(target.hp), dead: target.dead }),
            party: world
                .party
                .members
                .iter()
                .map(|member| MemberView {
                    object_id: member.object_id,
                    name: member.name.clone(),
                    hp: Gauge::of(Some((member.hp, member.max_hp))),
                    mp: Gauge::of(Some((member.mp, member.max_mp))),
                })
                .collect(),
            buffs: world.buffs.iter().map(|buff| BuffView { skill_id: buff.skill_id, level: buff.level, seconds: buff.duration }).collect(),
            skills: world
                .skills
                .iter()
                .map(|skill| SkillView {
                    id: skill.id,
                    level: skill.level,
                    passive: skill.passive,
                    ready: !world.cooldowns.iter().any(|reuse| reuse.skill_id == skill.id && reuse.remaining > 0),
                })
                .collect(),
            inventory: world
                .inventory
                .items
                .iter()
                .map(|item| ItemView { object_id: item.object_id, item_id: item.item_id, count: item.count, equipped: item.equipped, body_part: item.body_part })
                .collect(),
            nearby,
            adena: world.inventory.adena(),
            exp: me.map_or(0, |me| me.exp),
            karma: me.map_or(0, |me| me.karma),
            stats: StatsView {
                str_: world.stats.str_,
                dex: world.stats.dex,
                con: world.stats.con,
                int_: world.stats.int_,
                wit: world.stats.wit,
                men: world.stats.men,
                p_atk: world.stats.p_atk,
                p_def: world.stats.p_def,
                accuracy: world.stats.accuracy,
                critical: world.stats.critical,
                atk_spd: world.stats.atk_spd,
                m_atk: world.stats.m_atk,
                m_def: world.stats.m_def,
                evasion: world.stats.evasion,
                cast_spd: world.stats.cast_spd,
                sp: world.stats.sp,
                load: world.stats.load,
                max_load: world.stats.max_load,
            },
            death: world.death.as_ref().map(|die| DeathView { town: die.to_village, hideout: die.to_hideout, castle: die.to_castle, siege_hq: die.to_siege_hq, fortress: die.to_fixed_spot }),
            questions: world.dialogs.pending.iter().enumerate().filter_map(|(index, dialog)| question_of(index, dialog)).collect(),
            clan_id: me.map_or(0, |me| me.clan_id),
            ally_id: me.map_or(0, |me| me.ally_id),
            title: me.map_or_else(String::new, |me| me.title.clone()),
            class_id: match me.map(|me| &me.kind) {
                Some(EntityKind::Player { class_id, .. }) => *class_id,
                _ => 0,
            },
            run_speed: me.map_or(0, |me| me.run_speed),
            pvp_kills: world.social.pvp_kills,
            pk_kills: world.social.pk_kills,
            recommend_have: i32::from(world.social.recom_have),
            recommend_left: i32::from(world.social.recom_left),
            servers: servers
                .iter()
                .map(|server| ServerView {
                    id: server.id,
                    flags: server.extra.flags,
                    address: format!("{}.{}.{}.{}:{}", server.ip[0], server.ip[1], server.ip[2], server.ip[3], server.port),
                    players: server.players,
                    max_players: server.max_players,
                    online: server.online,
                })
                .collect(),
            characters: characters
                .iter()
                .enumerate()
                .map(|(slot, entry)| CharacterView {
                    slot,
                    name: entry.name.clone(),
                    clan_id: entry.clan_id,
                    sex: entry.sex,
                    race: entry.race,
                    base_class_id: entry.base_class_id,
                    active: entry.active,
                    position: [entry.x, entry.y, entry.z],
                    hp: entry.hp as i32,
                    mp: entry.mp as i32,
                    sp: entry.sp,
                    exp: entry.exp,
                    level: entry.level,
                    karma: entry.karma,
                })
                .collect(),
            clan: world.pledge_window.info.as_ref().map(|info| ClanView {
                name: info.name.clone(),
                leader: info.leader_name.clone(),
                level: info.level,
                castle_id: info.castle_id,
                hideout_id: info.hideout_id,
                ally: info.ally_name.clone(),
                at_war: info.at_war,
                members: world.pledge_window.members.iter().map(|member| ClanMember { name: member.name.clone(), level: member.level, class_id: member.class_id, online: member.online_object_id != 0 }).collect(),
            }),
            gains: crate::stats::Gains::default(),
            loot: Vec::new(),
            bad_mobs: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{Vitals, character, ground_item, npc_info, user_info, world_of};

    #[test]
    fn snapshot_reads_vitals_and_percent_without_dividing_by_zero() {
        let world = world_of(&[user_info(1, "Wender", &Vitals { hp: 250, max_hp: 1000, mp: 0, max_mp: 0, cp: 10, max_cp: 10 }, [1, 2, 3])]);
        let snapshot = Snapshot::of(BotState::InWorld, &world);
        assert_eq!(snapshot.name, "Wender");
        assert_eq!(snapshot.hp, Gauge { current: 250, max: 1000, percent: Some(25) });
        assert_eq!(snapshot.mp.percent, None);
        assert_eq!(snapshot.position, [1, 2, 3]);
    }

    #[test]
    fn snapshot_lists_nearby_sorted_by_distance_and_marks_kind() {
        let mut world = world_of(&[user_info(1, "Wender", &Vitals::full(), [0, 0, 0])]);
        world.apply(&npc_info(10, 1, "Wolf", true, [300, 0, 0]));
        world.apply(&npc_info(11, 2, "Guard", false, [100, 0, 0]));
        world.apply(&character(12, "Rival", [200, 0, 0]));
        world.apply(&ground_item(13, 57, [50, 0, 0]));
        let kinds: Vec<_> = Snapshot::of(BotState::InWorld, &world).nearby.iter().map(|view| (view.object_id, view.kind)).collect();
        assert_eq!(kinds, vec![(13, NearbyKind::Item), (11, NearbyKind::Npc), (12, NearbyKind::Player), (10, NearbyKind::Monster)]);
    }

    #[test]
    fn snapshot_exposes_death_options_and_pending_questions() {
        let mut world = world_of(&[user_info(1, "Wender", &Vitals::full(), [0, 0, 0])]);
        assert!(Snapshot::of(BotState::InWorld, &world).death.is_none());
        world.death = Some(l2_net::game::Die { object_id: 1, to_village: true, to_hideout: false, to_castle: true, to_siege_hq: false, sweepable: false, to_fixed_spot: false });
        world.dialogs.push(Dialog::JoinParty(l2_net::game::AskJoinParty { requestor_name: "Tank".into(), item_distribution: 0 }));
        world.dialogs.push(Dialog::PetName);
        world.dialogs.push(Dialog::TradeRequest { requestor_id: 7 });
        let snapshot = Snapshot::of(BotState::InWorld, &world);
        assert_eq!(snapshot.death, Some(DeathView { town: true, hideout: false, castle: true, siege_hq: false, fortress: false }));
        assert_eq!(snapshot.questions.iter().map(|q| (q.index, q.kind)).collect::<Vec<_>>(), vec![(0, QuestionKind::Party), (2, QuestionKind::Trade)]);
        assert!(snapshot.questions[0].text.contains("Tank"));
    }

    #[test]
    fn snapshot_of_a_session_not_in_world_is_empty_but_valid() {
        let snapshot = Snapshot::of(BotState::Connecting, &World::default());
        assert_eq!(snapshot.state, BotState::Connecting);
        assert!(snapshot.name.is_empty() && snapshot.nearby.is_empty() && snapshot.target.is_none());
        assert_eq!(snapshot.hp, Gauge::default());
    }
}
