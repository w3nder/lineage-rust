use glam::Vec3;

use crate::movement::planar_distance;
use crate::transient_effect_data::{SCANNED, SPAWNED};

pub const CULL_RANGE_UNITS: f64 = 32768.0 * 0.0625;
pub const REUSE_SECONDS_PER_MS: f32 = 0.001;
pub const MACRO_WAIT_STEP_MS: i32 = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastTarget {
    Missing,
    Pawn { is_me: bool },
    Mover,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CastContext {
    pub keep_min_frame_rate: bool,
    pub pawn_clipping: f32,
    pub me_position: Option<Vec3>,
    pub caster_position: Vec3,
    pub caster_rendered: bool,
    pub caster_is_me: bool,
    pub caster_dead: bool,
    pub target: CastTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastVariant {
    Pawn,
    Mover,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcceptedCast {
    pub caster_id: i32,
    pub target_id: i32,
    pub skill_id: i32,
    pub skill_level: i32,
    pub hit_time: i32,
    pub reuse_delay: i32,
    pub caster_is_me: bool,
    pub variant: CastVariant,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MagicInfo {
    pub effect_actor: Vec<i32>,
    pub ground_targets: Vec<Vec3>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MagicState {
    pub current: MagicInfo,
    pub pending: Option<MagicInfo>,
}

impl MagicState {
    pub fn set_magic_info(&mut self) {
        if let Some(pending) = self.pending.take() {
            self.current = pending;
        } else {
            self.current = MagicInfo::default();
        }
    }
}

impl MagicInfo {
    pub fn effect_list_head_empty(&self) -> bool {
        self.effect_actor.first().is_some_and(|head| *head == 0)
    }
}

pub fn cull_range(pawn_clipping: f32) -> f32 {
    (f64::from(pawn_clipping) * CULL_RANGE_UNITS) as f32
}

pub fn variant_of(target: CastTarget) -> Option<CastVariant> {
    match target {
        CastTarget::Pawn { .. } => Some(CastVariant::Pawn),
        CastTarget::Mover => Some(CastVariant::Mover),
        CastTarget::Missing => None,
    }
}

pub fn admitted(context: &CastContext) -> bool {
    let Some(variant) = variant_of(context.target) else { return false };
    if context.keep_min_frame_rate {
        let Some(me) = context.me_position else { return false };
        let target_is_me = matches!(context.target, CastTarget::Pawn { is_me: true });
        let away = !context.caster_rendered && !context.caster_is_me;
        let hidden = match variant {
            CastVariant::Pawn => away && !target_is_me,
            CastVariant::Mover => away,
        };
        if hidden {
            return false;
        }
        if cull_range(context.pawn_clipping) < planar_distance(me, context.caster_position) {
            return false;
        }
    }
    !context.caster_dead
}

pub fn spawn_n_transient_effect(skill_id: i32) -> bool {
    SCANNED.contains(&skill_id) && SPAWNED.binary_search(&skill_id).is_ok()
}

pub fn reuse_seconds(reuse_delay_ms: i32) -> i32 {
    (reuse_delay_ms as f32 * REUSE_SECONDS_PER_MS) as i32
}

pub fn cast_seconds(hit_time_ms: i32) -> f32 {
    hit_time_ms as f32 * REUSE_SECONDS_PER_MS
}

pub fn macro_wait_seconds(hit_time_ms: i32) -> f32 {
    (hit_time_ms / MACRO_WAIT_STEP_MS + 1) as f32
}

pub fn reuse_started(reuse_delay_ms: i32) -> Option<i32> {
    Some(reuse_seconds(reuse_delay_ms)).filter(|seconds| *seconds > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> CastContext {
        CastContext {
            keep_min_frame_rate: false,
            pawn_clipping: 0.7,
            me_position: Some(Vec3::ZERO),
            caster_position: Vec3::new(100.0, 0.0, 0.0),
            caster_rendered: true,
            caster_is_me: false,
            caster_dead: false,
            target: CastTarget::Pawn { is_me: false },
        }
    }

    #[test]
    fn without_a_target_pawn_or_mover_nothing_happens() {
        assert!(admitted(&context()));
        assert!(!admitted(&CastContext { target: CastTarget::Missing, ..context() }));
        assert!(admitted(&CastContext { target: CastTarget::Mover, ..context() }));
    }

    #[test]
    fn a_dead_controller_returns_before_anything_else() {
        assert!(!admitted(&CastContext { caster_dead: true, ..context() }));
        assert!(!admitted(&CastContext { caster_dead: true, target: CastTarget::Mover, ..context() }));
    }

    #[test]
    fn the_frame_rate_cull_needs_the_local_pawn_and_the_range() {
        let low = CastContext { keep_min_frame_rate: true, ..context() };
        assert!(admitted(&low));
        assert!(!admitted(&CastContext { me_position: None, ..low }));
        assert!((cull_range(0.7) - 1433.6).abs() < 0.01);
        assert!(!admitted(&CastContext { caster_position: Vec3::new(1433.7, 0.0, 99.0), ..low }));
        assert!(admitted(&CastContext { caster_position: Vec3::new(1433.5, 0.0, 99.0), ..low }));
    }

    #[test]
    fn an_unrendered_stranger_only_casts_when_aimed_at_the_local_pawn() {
        let low = CastContext { keep_min_frame_rate: true, caster_rendered: false, ..context() };
        assert!(!admitted(&low));
        assert!(admitted(&CastContext { target: CastTarget::Pawn { is_me: true }, ..low }));
        assert!(admitted(&CastContext { caster_is_me: true, ..low }));
        assert!(!admitted(&CastContext { target: CastTarget::Mover, ..low }));
        assert!(admitted(&CastContext { target: CastTarget::Mover, caster_is_me: true, ..low }));
    }

    #[test]
    fn times_follow_the_integer_and_float_conversions_of_the_engine() {
        assert_eq!(reuse_seconds(4999), 4);
        assert_eq!(reuse_started(999), None);
        assert_eq!(reuse_started(1000), Some(1));
        assert_eq!(reuse_started(-5000), None);
        assert!((cast_seconds(1500) - 1.5).abs() < 1e-6);
        assert_eq!(macro_wait_seconds(0), 1.0);
        assert_eq!(macro_wait_seconds(999), 1.0);
        assert_eq!(macro_wait_seconds(1000), 2.0);
        assert_eq!(macro_wait_seconds(2500), 3.0);
    }

    #[test]
    fn spawn_n_transient_effect_returns_what_the_engine_returned_for_every_skill_id() {
        assert!(SPAWNED.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(SPAWNED.iter().all(|id| SCANNED.contains(id)));
        for shot in [2039, 2047, 2061, 2150, 2154, 2155, 2164] {
            assert!(spawn_n_transient_effect(shot), "{shot}");
        }
        assert!(!spawn_n_transient_effect(1));
        assert!(!spawn_n_transient_effect(4552));
        assert!(!spawn_n_transient_effect(-1));
        assert!(!spawn_n_transient_effect(0x10000));
    }

    #[test]
    fn the_effect_actor_head_is_empty_only_for_a_listed_null() {
        assert!(!MagicInfo::default().effect_list_head_empty());
        assert!(MagicInfo { effect_actor: vec![0, 3], ..Default::default() }.effect_list_head_empty());
        assert!(!MagicInfo { effect_actor: vec![5], ..Default::default() }.effect_list_head_empty());
    }
}
