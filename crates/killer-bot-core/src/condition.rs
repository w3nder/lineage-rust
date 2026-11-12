use killer_world::World;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ConditionSpec {
    HpBelow(u8),
    HpAbove(u8),
    MpBelow(u8),
    MpAbove(u8),
    CpBelow(u8),
    TargetHpBelow(u8),
    HasTarget,
    TargetDead,
    InCombat,
    HasBuff(i32),
    LacksBuff(i32),
    SkillReady(i32),
    ItemCountAtLeast { item_id: i32, count: i32 },
    TargetInRange(f32),
}

pub fn percent(value: Option<(i32, i32)>) -> Option<u8> {
    let (current, max) = value?;
    if max <= 0 {
        return None;
    }
    Some((i64::from(current.clamp(0, max)) * 100 / i64::from(max)) as u8)
}

pub fn holds(spec: &ConditionSpec, world: &World, _now: Duration) -> bool {
    let me = world.me.as_ref();
    let target = world.targeted();
    match spec {
        ConditionSpec::HpBelow(limit) => me.and_then(|me| percent(me.hp)).is_some_and(|value| value < *limit),
        ConditionSpec::HpAbove(limit) => me.and_then(|me| percent(me.hp)).is_some_and(|value| value > *limit),
        ConditionSpec::MpBelow(limit) => me.and_then(|me| percent(me.mp)).is_some_and(|value| value < *limit),
        ConditionSpec::MpAbove(limit) => me.and_then(|me| percent(me.mp)).is_some_and(|value| value > *limit),
        ConditionSpec::CpBelow(limit) => me.and_then(|me| percent(me.cp)).is_some_and(|value| value < *limit),
        ConditionSpec::TargetHpBelow(limit) => target.and_then(|t| percent(t.hp)).is_some_and(|value| value < *limit),
        ConditionSpec::HasTarget => target.is_some(),
        ConditionSpec::TargetDead => target.is_some_and(|t| t.dead),
        ConditionSpec::InCombat => me.is_some_and(|me| me.in_combat),
        ConditionSpec::HasBuff(id) => world.buffs.iter().any(|buff| buff.skill_id == *id),
        ConditionSpec::LacksBuff(id) => !world.buffs.iter().any(|buff| buff.skill_id == *id),
        ConditionSpec::SkillReady(id) => {
            world.skills.iter().any(|skill| skill.id == *id && !skill.passive)
                && !world.cooldowns.iter().any(|reuse| reuse.skill_id == *id && reuse.remaining > 0)
        }
        ConditionSpec::ItemCountAtLeast { item_id, count } => {
            world.inventory.items.iter().filter(|item| item.item_id == *item_id).map(|item| i64::from(item.count)).sum::<i64>() >= i64::from(*count)
        }
        ConditionSpec::TargetInRange(range) => match (me, target) {
            (Some(me), Some(target)) => me.position.distance(target.position) <= *range,
            _ => false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{Vitals, npc_info, user_info, world_of};
    use l2_net::game::{Buff, SkillEntry, SkillReuse};

    fn hurt() -> World {
        world_of(&[user_info(1, "Wender", &Vitals { hp: 300, max_hp: 1000, ..Vitals::full() }, [0, 0, 0])])
    }

    #[test]
    fn percent_is_none_for_zero_max_or_missing() {
        assert_eq!(percent(None), None);
        assert_eq!(percent(Some((5, 0))), None);
        assert_eq!(percent(Some((250, 1000))), Some(25));
    }

    #[test]
    fn hp_below_is_false_when_hp_is_unknown() {
        let world = World::default();
        assert!(!holds(&ConditionSpec::HpBelow(50), &world, Duration::ZERO));
        assert!(holds(&ConditionSpec::HpBelow(50), &hurt(), Duration::ZERO));
        assert!(!holds(&ConditionSpec::HpAbove(50), &hurt(), Duration::ZERO));
    }

    #[test]
    fn has_buff_reads_world_buffs() {
        let mut world = hurt();
        world.buffs.push(Buff { skill_id: 1204, level: 2, duration: 600 });
        assert!(holds(&ConditionSpec::HasBuff(1204), &world, Duration::ZERO));
        assert!(!holds(&ConditionSpec::LacksBuff(1204), &world, Duration::ZERO));
        assert!(holds(&ConditionSpec::LacksBuff(1205), &world, Duration::ZERO));
    }

    #[test]
    fn skill_ready_uses_cooldown_remaining() {
        let mut world = hurt();
        world.skills.push(SkillEntry { passive: false, level: 1, id: 56 });
        assert!(holds(&ConditionSpec::SkillReady(56), &world, Duration::ZERO));
        world.cooldowns.push(SkillReuse { skill_id: 56, level: 1, reuse: 10, remaining: 4 });
        assert!(!holds(&ConditionSpec::SkillReady(56), &world, Duration::ZERO));
        assert!(!holds(&ConditionSpec::SkillReady(99), &world, Duration::ZERO));
    }

    #[test]
    fn target_in_range_uses_entity_positions() {
        let mut world = hurt();
        world.apply(&npc_info(7, 20001, "Wolf", true, [100, 0, 0]));
        assert!(world.select(7));
        assert!(holds(&ConditionSpec::HasTarget, &world, Duration::ZERO));
        assert!(holds(&ConditionSpec::TargetInRange(150.0), &world, Duration::ZERO));
        assert!(!holds(&ConditionSpec::TargetInRange(50.0), &world, Duration::ZERO));
    }
}
