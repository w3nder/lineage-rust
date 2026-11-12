use crate::world_sound::{SoundPlan, VOLUME_SCALE};
use glam::Vec3;
use killer_assets::client::DatTables;
use killer_world::{Entity, EntityKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Cast,
    Shot,
    Explosion,
}

pub fn voice_row(entity: &Entity) -> Option<usize> {
    match &entity.kind {
        EntityKind::Player { race, sex, class_id, .. } => l2_dat::chargrp::index_for(*race, *sex, *class_id),
        _ => None,
    }
}

pub fn transient_plans(entity: &Entity, tables: &DatTables, skill_id: i32) -> Vec<SoundPlan> {
    let Some(record) = l2_dat::skillsoundgrp::find(&tables.skillsound, skill_id, 1) else { return Vec::new() };
    record
        .slots
        .iter()
        .filter(|slot| !slot.spell.name.is_empty())
        .map(|slot| SoundPlan {
            actor: entity.object_id,
            name: slot.spell.name.clone(),
            position: entity.position,
            volume: slot.spell.volume * VOLUME_SCALE,
            radius: slot.spell.radius,
            skip_if_playing: false,
        })
        .collect()
}

pub fn plans(caster: &Entity, tables: &DatTables, mode: Mode, skill_id: i32, level: i32) -> Vec<SoundPlan> {
    cast_plans(caster.object_id, caster.position, voice_row(caster), &tables.skillsound, mode, skill_id, level)
}

pub fn cast_plans(
    actor: i32,
    position: Vec3,
    row: Option<usize>,
    records: &[l2_dat::skillsoundgrp::SkillSound],
    mode: Mode,
    skill_id: i32,
    level: i32,
) -> Vec<SoundPlan> {
    let Some(record) = l2_dat::skillsoundgrp::find(records, skill_id, level) else { return Vec::new() };
    let plan = |name: &str, volume: f32, radius: f32| {
        (!name.is_empty()).then(|| SoundPlan { actor, name: name.to_owned(), position, volume: volume * VOLUME_SCALE, radius, skip_if_playing: false })
    };
    let mut plans: Vec<SoundPlan> = record
        .slots
        .iter()
        .filter_map(|slot| {
            let sound = match mode {
                Mode::Cast => &slot.spell,
                Mode::Shot => &slot.shot,
                Mode::Explosion => &slot.explosion,
            };
            plan(&sound.name, sound.volume, sound.radius)
        })
        .collect();
    let voices = match mode {
        Mode::Cast => &record.cast,
        Mode::Shot => &record.magic,
        Mode::Explosion => return plans,
    };
    if let Some(name) = row.and_then(|row| voices.get(row)) {
        plans.extend(plan(name, record.cast_volume, record.cast_radius));
    }
    plans
}

#[cfg(test)]
mod tests {
    use super::*;
    use l2_dat::skillsoundgrp::{EffectSlot, Sound, SkillSound};

    fn sound(name: &str, volume: f32, radius: f32) -> Sound {
        Sound { name: name.into(), volume, radius }
    }

    fn record() -> SkillSound {
        let empty = || sound("", 0.0, 0.0);
        let mut cast: [String; 15] = std::array::from_fn(|_| String::new());
        let mut magic: [String; 15] = std::array::from_fn(|_| String::new());
        cast[0] = "ChrSound.m_hfighter_sub".into();
        magic[2] = "ChrSound.m_darkelf_magic".into();
        SkillSound {
            id: 1,
            level: 1,
            slots: [
                EffectSlot { spell: sound("SkillSound4.a_cast", 250.0, 50.0), shot: sound("SkillSound4.a_shot", 250.0, 100.0), explosion: empty() },
                EffectSlot { spell: empty(), shot: sound("SkillSound4.a_shot", 250.0, 100.0), explosion: empty() },
                EffectSlot { spell: empty(), shot: empty(), explosion: sound("SkillSound4.a_exp", 250.0, 200.0) },
            ],
            cast,
            magic,
            cast_volume: 250.0,
            cast_radius: 50.0,
        }
    }

    #[test]
    fn cast_plays_every_named_spell_slot_and_the_race_voice_at_the_casters_position() {
        let position = Vec3::new(1.0, 2.0, 3.0);
        let plans = cast_plans(7, position, Some(0), &[record()], Mode::Cast, 1, 4);
        assert_eq!(plans.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), ["SkillSound4.a_cast", "ChrSound.m_hfighter_sub"]);
        assert!(plans.iter().all(|p| p.actor == 7 && p.position == position && !p.skip_if_playing));
        assert!((plans[0].volume - 250.0 * VOLUME_SCALE).abs() < 1e-6);
        assert_eq!((plans[0].radius, plans[1].radius), (50.0, 50.0));
    }

    #[test]
    fn shot_plays_each_shot_slot_even_when_two_repeat_the_same_name_and_uses_the_magic_voice() {
        let plans = cast_plans(7, Vec3::ZERO, Some(2), &[record()], Mode::Shot, 1, 1);
        assert_eq!(
            plans.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
            ["SkillSound4.a_shot", "SkillSound4.a_shot", "ChrSound.m_darkelf_magic"]
        );
        assert_eq!(plans[0].radius, 100.0);
    }

    #[test]
    fn explosion_mode_3_plays_only_the_expeffect_slots_without_voice() {
        let plans = cast_plans(7, Vec3::ZERO, Some(0), &[record()], Mode::Explosion, 1, 1);
        assert_eq!(plans.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), ["SkillSound4.a_exp"]);
        assert_eq!(plans[0].radius, 200.0);
    }

    #[test]
    fn an_empty_voice_position_or_a_non_player_is_silent_without_falling_back_to_the_spell() {
        let none = cast_plans(7, Vec3::ZERO, Some(5), &[record()], Mode::Cast, 1, 1);
        assert_eq!(none.len(), 1);
        let npc = cast_plans(7, Vec3::ZERO, None, &[record()], Mode::Cast, 1, 1);
        assert_eq!(npc.len(), 1);
        assert!(cast_plans(7, Vec3::ZERO, Some(0), &[record()], Mode::Cast, 999, 1).is_empty());
    }
}
