#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Victim {
    Caster,
    Target,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attack {
    Notify { victim: Victim },
    Strike { victim: Victim, damage: i32, critical: bool, notify: bool },
}

type Variants = &'static [(u8, &'static [Attack])];

include!("skill_attack_data.rs");

pub const EFFECT_LIST_HEAD_EMPTY: u8 = 32;

pub fn attacks(skill_id: i32, env: u8) -> &'static [Attack] {
    let Ok(index) = TABLE.binary_search_by_key(&skill_id, |(id, _)| *id) else { return &[] };
    TABLE[index].1.iter().find(|(bits, _)| *bits == env).map_or(&[], |(_, attacks)| *attacks)
}

pub fn skills() -> impl Iterator<Item = i32> {
    TABLE.iter().map(|(id, _)| *id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill_effects::{NPC, SELF_TARGET, SHOT_STAGE_TWO, TARGET_LIST};

    #[test]
    fn skill_1_strikes_the_target_critically_in_every_environment_and_skill_3_notifies_it() {
        for env in 0..32 {
            let victim = if env & SELF_TARGET != 0 { Victim::Caster } else { Victim::Target };
            assert_eq!(attacks(1, env), &[Attack::Strike { victim, damage: 1, critical: true, notify: true }], "{env}");
        }
        assert_eq!(attacks(3, SHOT_STAGE_TWO), &[Attack::Notify { victim: Victim::Target }]);
        assert_eq!(attacks(3, SHOT_STAGE_TWO | SELF_TARGET | NPC), &[Attack::Notify { victim: Victim::Caster }]);
        assert!(attacks(3, TARGET_LIST).iter().all(|attack| matches!(attack, Attack::Notify { .. })));
        assert!(attacks(2, SHOT_STAGE_TWO).is_empty());
        assert!(attacks(0x7fff, 0).is_empty());
    }

    #[test]
    fn the_table_is_sorted_and_covers_the_skills_found_by_the_emulator() {
        let ids: Vec<i32> = skills().collect();
        assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(ids.len(), 48);
        assert!(ids.contains(&0x100f) && ids.contains(&0x16a));
    }
}
