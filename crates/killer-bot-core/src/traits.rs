use crate::condition::percent;
use crate::names::Names;
use crate::options::Trait;
use killer_world::{Entity, World};

pub const BLEEDING: i32 = 0x0001;
pub const POISON: i32 = 0x0002;
pub const ICE: i32 = 0x0008;
pub const FEAR: i32 = 0x0020;
pub const STUN: i32 = 0x0040;
pub const SLEEP: i32 = 0x0080;
pub const MUTED: i32 = 0x0100;
pub const ROOT: i32 = 0x0200;
pub const HOLD_PARALYZE: i32 = 0x0400;
pub const HOLD_PETRIFY: i32 = 0x0800;
pub const FLOATING_ROOT: i32 = 0x20000;

#[derive(Debug, Clone, PartialEq)]
pub struct Subject {
    pub object_id: i32,
    pub name: String,
    pub hp: Option<(i32, i32)>,
    pub mp: Option<(i32, i32)>,
    pub cp: Option<(i32, i32)>,
    pub dead: bool,
    pub abnormal: i32,
}

impl Subject {
    pub fn of_entity(entity: &Entity) -> Subject {
        Subject { object_id: entity.object_id, name: entity.name.clone(), hp: entity.hp, mp: entity.mp, cp: entity.cp, dead: entity.dead, abnormal: entity.abnormal_effect }
    }
}

pub fn subject(world: &World, object_id: i32) -> Option<Subject> {
    if let Some(entity) = world.entity(object_id) {
        let mut subject = Subject::of_entity(entity);
        if let Some(member) = world.party.members.iter().find(|member| member.object_id == object_id) {
            subject.hp = subject.hp.or(Some((member.hp, member.max_hp)));
            subject.mp = subject.mp.or(Some((member.mp, member.max_mp)));
            subject.cp = subject.cp.or(Some((member.cp, member.max_cp)));
        }
        return Some(subject);
    }
    let member = world.party.members.iter().find(|member| member.object_id == object_id)?;
    Some(Subject {
        object_id,
        name: member.name.clone(),
        hp: Some((member.hp, member.max_hp)),
        mp: Some((member.mp, member.max_mp)),
        cp: Some((member.cp, member.max_cp)),
        dead: member.hp <= 0 && member.max_hp > 0,
        abnormal: 0,
    })
}

pub fn status_bit(trait_: Trait) -> Option<i32> {
    Some(match trait_ {
        Trait::Bleeding => BLEEDING,
        Trait::Poisoned => POISON,
        Trait::Iced => ICE,
        Trait::Shackled => ROOT,
        Trait::Feared => FEAR,
        Trait::Stunned => STUN,
        Trait::Slept => SLEEP,
        Trait::Muted => MUTED,
        Trait::Rooted => FLOATING_ROOT,
        Trait::Paralyzed => HOLD_PARALYZE,
        Trait::Petrified => HOLD_PETRIFY,
        _ => return None,
    })
}

fn has_named_buff(world: &World, names: &dyn Names, skill: &str) -> bool {
    names.find_skill(skill).is_some_and(|id| world.buffs.iter().any(|buff| buff.skill_id == id))
}

pub fn holds(trait_: Trait, below_percent: u8, subject: &Subject, world: &World, names: &dyn Names) -> bool {
    let low = |value: Option<(i32, i32)>| percent(value).is_some_and(|current| current < below_percent);
    let is_me = world.me.as_ref().is_some_and(|me| me.object_id == subject.object_id);
    match trait_ {
        Trait::Always => true,
        Trait::Hp => low(subject.hp),
        Trait::Mp => low(subject.mp),
        Trait::Cp => low(subject.cp),
        Trait::Dead => subject.dead,
        Trait::DeathPenalty => is_me && has_named_buff(world, names, "Death Penalty"),
        Trait::UltimateDefense => is_me && has_named_buff(world, names, "Ultimate Defense"),
        Trait::Charges | Trait::Souls => false,
        other => status_bit(other).is_some_and(|bit| subject.abnormal & bit != 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::names::IdNames;
    use crate::testkit::{Vitals, character, user_info, world_of};
    use l2_net::game::PartyMember;

    struct Named;

    impl Names for Named {
        fn item(&self, id: i32) -> String {
            id.to_string()
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
                "Death Penalty" => Some(5076),
                "Ultimate Defense" => Some(110),
                _ => None,
            }
        }
    }

    fn me(hp: i32) -> World {
        world_of(&[user_info(1, "Wender", &Vitals { hp, max_hp: 1000, ..Vitals::full() }, [0, 0, 0])])
    }

    #[test]
    fn percent_traits_compare_the_subject_stat_with_the_threshold() {
        let world = me(300);
        let subject = subject(&world, 1).unwrap();
        assert!(holds(Trait::Hp, 60, &subject, &world, &IdNames));
        assert!(!holds(Trait::Hp, 20, &subject, &world, &IdNames));
        assert!(!holds(Trait::Mp, 60, &subject, &world, &IdNames));
        assert!(holds(Trait::Always, 0, &subject, &world, &IdNames));
    }

    #[test]
    fn a_far_party_member_is_known_by_the_party_packet_even_without_an_entity() {
        let mut world = me(900);
        world.party.members.push(PartyMember { object_id: 7, name: "Healer".into(), cp: 0, max_cp: 0, hp: 100, max_hp: 1000, mp: 0, max_mp: 0, level: 40, class_id: 0 });
        let far = subject(&world, 7).unwrap();
        assert_eq!(far.name, "Healer");
        assert!(holds(Trait::Hp, 60, &far, &world, &IdNames));
        assert!(subject(&world, 99).is_none());
    }

    #[test]
    fn status_traits_read_the_abnormal_mask_of_the_subject() {
        let mut world = me(900);
        world.apply(&character(2, "Rival", [10, 0, 0]));
        world.entities.get_mut(&2).unwrap().abnormal_effect = STUN | POISON;
        let rival = subject(&world, 2).unwrap();
        assert!(holds(Trait::Stunned, 0, &rival, &world, &IdNames));
        assert!(holds(Trait::Poisoned, 0, &rival, &world, &IdNames));
        assert!(!holds(Trait::Slept, 0, &rival, &world, &IdNames));
        assert_eq!(status_bit(Trait::Hp), None);
    }

    #[test]
    fn death_penalty_and_ultimate_defense_come_from_my_buffs_by_skill_name() {
        let mut world = me(900);
        world.buffs.push(l2_net::game::Buff { skill_id: 5076, level: 2, duration: 600 });
        let mine = subject(&world, 1).unwrap();
        assert!(holds(Trait::DeathPenalty, 0, &mine, &world, &Named));
        assert!(!holds(Trait::UltimateDefense, 0, &mine, &world, &Named));
        assert!(!holds(Trait::DeathPenalty, 0, &mine, &world, &IdNames));
    }

    #[test]
    fn charges_and_souls_are_open_without_a_c4_field() {
        let world = me(900);
        let mine = subject(&world, 1).unwrap();
        assert!(!holds(Trait::Charges, 0, &mine, &world, &IdNames));
        assert!(!holds(Trait::Souls, 0, &mine, &world, &IdNames));
    }
}
