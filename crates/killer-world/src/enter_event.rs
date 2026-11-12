use l2_dat::entereventgrp::EnterEvent;

pub const SPAWN_ANIMATION_ENTER_EVENT: u8 = 2;
pub const RISE_SPEED: f32 = f32::from_bits(0x3f0e_38e4);
pub const EFFECT_SCALE_PER_RADIUS: f32 = f32::from_bits(0x3dcc_cccd);
pub const NO_SOUND: &str = "none";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rise {
    pub start: f32,
    pub target: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnterEffect {
    pub object_id: i32,
    pub class: String,
    pub location: [f32; 3],
    pub yaw: i32,
    pub scale: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnterSound {
    pub object_id: i32,
    pub name: String,
    pub position: [f32; 3],
    pub volume: f32,
    pub radius: f32,
}

pub fn start_height(is_rise: u32, z: f32, collision_height: f32) -> f32 {
    match is_rise {
        1 => z - (collision_height + collision_height),
        2 => (collision_height + collision_height) + z,
        _ => z,
    }
}

pub fn begin(is_rise: u32, z: f32, collision_height: f32) -> Option<Rise> {
    (is_rise != 0).then(|| Rise { start: start_height(is_rise, z, collision_height), target: z })
}

pub fn step(rise: &Rise, z: f32, dt: f32) -> (Option<f32>, bool) {
    if z == rise.target {
        return (None, true);
    }
    let advanced = ((f64::from(rise.target) - f64::from(rise.start)) * f64::from(dt) * f64::from(RISE_SPEED) + f64::from(z)) as f32;
    let crossing = (f64::from(rise.target) - f64::from(advanced)) * (f64::from(rise.target) - f64::from(z));
    if crossing < 0.0 { (Some(rise.target), true) } else { (Some(advanced), false) }
}

pub fn sound_of(row: &EnterEvent) -> Option<&str> {
    let name = row.sound_name.as_str();
    (!name.is_empty() && !name.eq_ignore_ascii_case(NO_SOUND)).then_some(name)
}

pub fn effect_of(row: &EnterEvent) -> Option<&str> {
    (!row.effect_name.is_empty()).then_some(row.effect_name.as_str())
}

pub fn anim_of(row: &EnterEvent) -> Option<&str> {
    (!row.anim_name.is_empty()).then_some(row.anim_name.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(is_rise: u32) -> EnterEvent {
        EnterEvent {
            id: 212,
            reserved: String::new(),
            sound_name: "SkillSound4.summon_shot".into(),
            sound_vol: 250.0,
            sound_rad: 100.0,
            is_rise,
            spawn_type: 1,
            effect_name: "LineageEffect.e_u011_b".into(),
            anim_name: "atkwait".into(),
        }
    }

    #[test]
    fn the_start_height_sinks_or_lifts_the_pawn_by_twice_its_collision_height() {
        assert_eq!(start_height(1, 100.0, 10.0), 80.0);
        assert_eq!(start_height(2, 100.0, 10.0), 120.0);
        assert_eq!(start_height(0, 100.0, 10.0), 100.0);
        assert_eq!(start_height(3, 100.0, 10.0), 100.0);
        assert_eq!(begin(0, 100.0, 10.0), None);
        assert_eq!(begin(1, 100.0, 10.0), Some(Rise { start: 80.0, target: 100.0 }));
        assert_eq!(begin(3, 100.0, 10.0), Some(Rise { start: 100.0, target: 100.0 }));
    }

    #[test]
    fn the_rise_moves_by_the_distance_times_dt_times_the_engine_constant_and_stops_on_the_target() {
        let rise = Rise { start: 80.0, target: 100.0 };
        let (z, done) = step(&rise, 80.0, 0.1);
        assert!(!done);
        assert!((z.unwrap() - (80.0 + 20.0 * 0.1 * RISE_SPEED)).abs() < 1e-4);
        assert_eq!(step(&rise, 99.5, 1.0), (Some(100.0), true));
        assert_eq!(step(&rise, 100.0, 0.1), (None, true));
        let sinking = Rise { start: 120.0, target: 100.0 };
        assert_eq!(step(&sinking, 100.4, 1.0), (Some(100.0), true));
        let (z, done) = step(&sinking, 120.0, 0.1);
        assert!(!done && z.unwrap() < 120.0);
    }

    #[test]
    fn a_rising_pawn_reaches_its_height_in_finitely_many_ticks() {
        let rise = begin(1, 500.0, 25.0).unwrap();
        let mut z = rise.start;
        let mut ticks = 0;
        loop {
            ticks += 1;
            let (next, done) = step(&rise, z, 1.0 / 60.0);
            z = next.unwrap_or(z);
            if done {
                break;
            }
            assert!(ticks < 1000);
        }
        assert_eq!(z, 500.0);
    }

    #[test]
    fn empty_and_none_values_select_nothing() {
        let mut event = row(1);
        assert_eq!(sound_of(&event), Some("SkillSound4.summon_shot"));
        assert_eq!(effect_of(&event), Some("LineageEffect.e_u011_b"));
        assert_eq!(anim_of(&event), Some("atkwait"));
        event.sound_name = "None".into();
        event.effect_name.clear();
        event.anim_name.clear();
        assert_eq!((sound_of(&event), effect_of(&event), anim_of(&event)), (None, None, None));
        event.sound_name.clear();
        assert_eq!(sound_of(&event), None);
    }
}
