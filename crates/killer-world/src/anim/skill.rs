use l2_dat::animnames::{AnimKind, AnimSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillAnim {
    pub stages: Vec<Option<String>>,
    pub hold: Option<usize>,
}

fn chain(kinds: &[AnimKind], hold: Option<usize>, set: &AnimSet, weapon: usize) -> SkillAnim {
    SkillAnim { stages: kinds.iter().map(|kind| set.name(*kind, weapon).map(str::to_string)).collect(), hold }
}

pub fn skill_anim(animation: &str, set: &AnimSet, weapon: usize, npc_seq: Option<&str>) -> Option<SkillAnim> {
    use AnimKind::*;
    let letter = animation.to_ascii_uppercase();
    let cast = |intro: AnimKind, release: AnimKind| chain(&[intro, CastEnd, release], Some(1), set, weapon);
    let instant = |release: AnimKind| chain(&[CastEnd, release], Some(0), set, weapon);
    let single = |kind: AnimKind| chain(&[kind], None, set, weapon);
    let mix = |a: AnimKind, b: AnimKind, c: AnimKind| chain(&[a, b, c], None, set, weapon);
    let mut anim = match letter.as_str() {
        "A" => cast(CastShort, MagicNoTarget),
        "B" => cast(CastShort, MagicShot),
        "C" => cast(CastShort, MagicThrow),
        "D" => cast(CastMid, MagicNoTarget),
        "E" => cast(CastMid, MagicShot),
        "F" => cast(CastMid, MagicThrow),
        "G" => cast(CastLong, MagicNoTarget),
        "H" => cast(CastLong, MagicShot),
        "I" => cast(CastLong, MagicThrow),
        "J" => instant(MagicNoTarget),
        "K" => instant(MagicShot),
        "L" => instant(MagicThrow),
        "M" => single(PicItem),
        "N" => single(SpAtk27),
        "S" => single(SpAtk01),
        "T" => single(SpAtk02),
        "U" => single(SpAtk03),
        "V" => single(SpAtk04),
        "W" => single(SpAtk05),
        "X" => single(SpAtk06),
        "Y" => single(ShieldAtk),
        "MIX01" => mix(SpAtk09, SpAtk17, SpAtk24),
        "MIX02" => mix(SpAtk07, SpAtk16, SpAtk25),
        "MIX03" => mix(SpAtk07, SpAtk13, SpAtk20),
        "MIX04" => mix(SpAtk07, SpAtk12, SpAtk21),
        "MIX05" => mix(SpAtk08, SpAtk13, SpAtk22),
        "MIX06" => mix(SpAtk10, SpAtk11, SpAtk18),
        "MIX07" => mix(SpAtk10, SpAtk11, SpAtk19),
        "MIX08" => mix(SpAtk07, SpAtk14, SpAtk23),
        "MIX09" => mix(SpAtk09, SpAtk15, SpAtk26),
        _ => SkillAnim { stages: Vec::new(), hold: None },
    };
    if let Some(seq) = npc_seq {
        match anim.stages.last_mut() {
            Some(last) => *last = Some(seq.to_string()),
            None => anim.stages.push(Some(seq.to_string())),
        }
    }
    (!anim.stages.is_empty()).then_some(anim)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StageSeq {
    pub seconds: f32,
    pub attack_shot: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkillTimeline {
    pub ends: Vec<f32>,
    pub rate: f32,
    pub first_tween: f32,
    pub shot_time: f32,
    pub hold: Option<usize>,
}

pub const CAST_LEAD_STYLES: [i32; 4] = [2, 10, 5, 8];

pub fn cast_lead(cast_style: i32, visual_effect_lead: Option<f32>) -> f32 {
    match visual_effect_lead {
        Some(lead) if lead >= 0.15 => lead,
        _ if CAST_LEAD_STYLES.contains(&cast_style) => 0.4,
        _ if cast_style == 3 => 0.0,
        _ => 0.15,
    }
}

pub fn skill_timeline(
    stages: &[Option<StageSeq>],
    hold: Option<usize>,
    hit_time: f32,
    skill_speed_rate: f32,
    lead: f32,
) -> Option<SkillTimeline> {
    if stages.is_empty() || skill_speed_rate == 0.0 {
        return None;
    }
    let count = stages.len();
    let mut durations = vec![0.0f32; count];
    let mut total = 0.0;
    let mut shot_offset = 0.0;
    let mut shot_stage = None;
    for index in (0..count).rev() {
        let Some(seq) = stages[index].filter(|seq| seq.seconds > 0.0 && hold != Some(index)) else { continue };
        durations[index] = seq.seconds;
        if shot_offset == 0.0 {
            shot_offset = seq.attack_shot.unwrap_or(0.0) * seq.seconds;
            shot_stage = Some(index);
            total = shot_offset;
        } else {
            total += seq.seconds;
        }
    }
    if total == 0.0 {
        shot_offset = durations[count - 1];
        shot_stage = Some(count - 1);
        total = durations.iter().sum();
    }
    let available = hit_time - lead;
    let (first_tween, rate) = match hold {
        None => (0.2, total / (available - 0.2)),
        Some(hold) => {
            let first_tween = 0.2 / skill_speed_rate;
            durations[hold] = available - (total + 0.2) / skill_speed_rate;
            if durations[hold] >= 0.0 {
                (first_tween, skill_speed_rate)
            } else {
                durations[hold] = 0.0;
                (first_tween, total / (available - first_tween))
            }
        }
    };
    let mut ends = Vec::with_capacity(count);
    let mut at = first_tween;
    let mut shot_time = first_tween;
    for (index, duration) in durations.iter().enumerate() {
        if shot_stage == Some(index) {
            shot_time = if rate <= 0.0 { at } else { shot_offset / rate + at };
        }
        let mut end = if hold == Some(index) || rate <= 0.0 { at + duration } else { duration / rate + at };
        if index == 0 && hold != Some(0) && rate > 0.0 && lead >= 0.15 {
            end -= end / available * 0.15;
        }
        ends.push(end);
        at = end;
    }
    Some(SkillTimeline { ends, rate, first_tween, shot_time, hold })
}

impl SkillTimeline {
    pub fn next_stage(&self, elapsed: f32, current: Option<usize>) -> Option<usize> {
        if let Some(current) = current
            && elapsed != 0.0
            && elapsed <= self.ends[current]
        {
            return Some(current);
        }
        let mut stage = current.map_or(0, |current| current + 1);
        while stage < self.ends.len() && self.ends[stage] <= 0.0 {
            stage += 1;
        }
        (stage < self.ends.len()).then_some(stage)
    }

    pub fn is_hold(&self, stage: usize) -> bool {
        self.hold == Some(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l2_dat::animnames::parse;

    const WARRIOR: &str = include_str!("../../../l2-dat/tests/fixtures/lineagewarrior.int.txt");

    fn fighter() -> AnimSet {
        parse(WARRIOR).remove("mfighter").unwrap()
    }

    #[test]
    fn letters_map_to_the_cast_hold_release_chains() {
        let set = fighter();
        let a = skill_anim("A", &set, 0, None).unwrap();
        assert_eq!(
            a.stages,
            [
                Some("CastShort_MFighter".to_string()),
                Some("CastEnd_MFighter".into()),
                Some("MagicNoTarget_MFighter".into())
            ]
        );
        assert_eq!(a.hold, Some(1));
        let h = skill_anim("h", &set, 1, None).unwrap();
        assert_eq!(h.stages[0].as_deref(), Some("CastLong_MFighter"));
        assert_eq!(h.stages[2].as_deref(), Some("MagicShot_MFighter"));
        let l = skill_anim("L", &set, 0, None).unwrap();
        assert_eq!(l.stages, [Some("CastEnd_MFighter".to_string()), Some("MagicThrow_MFighter".into())]);
        assert_eq!(l.hold, Some(0));
        let s = skill_anim("S", &set, 1, None).unwrap();
        assert_eq!(s.stages, [Some("SpAtk01_1HS_MFighter".to_string())]);
        assert_eq!(s.hold, None);
        assert_eq!(skill_anim("M", &set, 0, None).unwrap().stages[0].as_deref(), Some("PicItem_MFighter"));
        assert_eq!(skill_anim("Y", &set, 0, None).unwrap().stages[0].as_deref(), Some("shieldatk_1HS_MFighter"));
        assert_eq!(skill_anim("N", &set, 3, None).unwrap().stages[0].as_deref(), Some("Social_dance_MFighter"));
        let mix = skill_anim("Mix07", &set, 3, None).unwrap();
        assert_eq!(
            mix.stages,
            [
                Some("spatk07_Dual_MFighter".to_string()),
                Some("spatk11_Dual_MFighter".into()),
                Some("spatk19_Dual_MFighter".into())
            ]
        );
        assert_eq!(mix.hold, None);
        assert_eq!(skill_anim("S", &set, 0, None).unwrap().stages, [None]);
        assert_eq!(skill_anim("Z", &set, 0, None), None);
        assert_eq!(skill_anim("", &set, 0, None), None);
    }

    #[test]
    fn npc_sequence_replaces_the_last_stage_or_becomes_the_only_one() {
        let set = fighter();
        let b = skill_anim("B", &set, 0, Some("spatk02")).unwrap();
        assert_eq!(b.stages[2].as_deref(), Some("spatk02"));
        assert_eq!(b.stages[0].as_deref(), Some("CastShort_MFighter"));
        let z = skill_anim("Z", &set, 0, Some("atk01")).unwrap();
        assert_eq!(z.stages, [Some("atk01".to_string())]);
        assert_eq!(z.hold, None);
    }

    #[test]
    fn cast_lead_depends_on_style_and_visual_effect() {
        assert_eq!(cast_lead(2, None), 0.4);
        assert_eq!(cast_lead(8, None), 0.4);
        assert_eq!(cast_lead(3, None), 0.0);
        assert_eq!(cast_lead(1, None), 0.15);
        assert_eq!(cast_lead(1, Some(0.5)), 0.5);
        assert_eq!(cast_lead(2, Some(0.1)), 0.4);
    }

    #[test]
    fn timeline_stretches_the_hold_to_fill_the_hit_time() {
        let stages = [
            Some(StageSeq { seconds: 1.0, attack_shot: None }),
            Some(StageSeq { seconds: 0.5, attack_shot: None }),
            Some(StageSeq { seconds: 2.0, attack_shot: Some(0.25) }),
        ];
        let t = skill_timeline(&stages, Some(1), 6.0, 1.0, 0.0).unwrap();
        assert_eq!(t.rate, 1.0);
        assert!((t.first_tween - 0.2).abs() < 1e-6);
        assert!((t.ends[0] - 1.2).abs() < 1e-5);
        assert!((t.ends[1] - 5.5).abs() < 1e-5);
        assert!((t.ends[2] - 7.5).abs() < 1e-5);
        assert!((t.shot_time - 6.0).abs() < 1e-5);
        assert_eq!(t.next_stage(0.0, None), Some(0));
        assert_eq!(t.next_stage(0.5, Some(0)), Some(0));
        assert_eq!(t.next_stage(1.3, Some(0)), Some(1));
        assert_eq!(t.next_stage(5.9, Some(1)), Some(2));
        assert_eq!(t.next_stage(7.6, Some(2)), None);
        assert!(t.is_hold(1) && !t.is_hold(0));
    }

    #[test]
    fn timeline_compresses_the_rate_when_the_hit_time_is_too_short() {
        let stages = [
            Some(StageSeq { seconds: 1.0, attack_shot: None }),
            Some(StageSeq { seconds: 0.5, attack_shot: None }),
            Some(StageSeq { seconds: 1.0, attack_shot: None }),
        ];
        let t = skill_timeline(&stages, Some(1), 1.0, 2.0, 0.0).unwrap();
        assert!((t.first_tween - 0.1).abs() < 1e-6);
        assert!((t.rate - 2.0 / 0.9).abs() < 1e-5, "{}", t.rate);
        assert!((t.ends[0] - (0.1 + 1.0 / t.rate)).abs() < 1e-5);
        assert!((t.ends[1] - t.ends[0]).abs() < 1e-6);
        assert!((t.ends[2] - 1.0).abs() < 1e-4, "{}", t.ends[2]);
        assert!((t.shot_time - 1.0).abs() < 1e-4);
        assert_eq!(t.next_stage(t.ends[0] + 1e-4, Some(0)), Some(1));
        assert_eq!(t.next_stage(t.ends[0] + 1e-4, Some(1)), Some(2));
    }

    #[test]
    fn timeline_without_hold_scales_everything_into_the_window() {
        let stages = [Some(StageSeq { seconds: 1.0, attack_shot: Some(0.5) })];
        let t = skill_timeline(&stages, None, 2.2, 1.0, 0.0).unwrap();
        assert!((t.rate - 0.25).abs() < 1e-6);
        assert!((t.ends[0] - 4.2).abs() < 1e-5);
        assert!((t.shot_time - 2.2).abs() < 1e-5);
        assert_eq!(skill_timeline(&[], None, 1.0, 1.0, 0.0), None);
        assert_eq!(skill_timeline(&stages, None, 1.0, 0.0, 0.0), None);
        let missing = skill_timeline(&[None], None, 1.0, 1.0, 0.0).unwrap();
        assert_eq!(missing.ends, [0.2]);
        let led = skill_timeline(&stages, None, 2.2, 1.0, 0.2).unwrap();
        let end = 1.0 / led.rate + 0.2;
        assert!((led.ends[0] - (end - end / 2.0 * 0.15)).abs() < 1e-5);
    }
}
