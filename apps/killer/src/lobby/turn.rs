use killer_assets::pawn_rotation::{self, ControllerBody, PHYS_WALKING, PawnBody, PawnTurnState, ROTATE_TO_DESIRED, RotationDefaults, TurnGate};

pub const WANT_CHANGE_YAW: u32 = 1;
pub const TURNING: u32 = 2;
pub const YAW_PERIOD: f64 = 65536.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PawnTurn {
    pub flags: u32,
    pub wanted_yaw: i32,
    pub turning_dir: i32,
    pub desired: [i32; 3],
}

pub fn ftol(value: f64) -> i32 {
    (value.trunc() as i64) as i32
}

pub fn default_character_turn(pawn: Option<&mut PawnTurn>, rate: f32) {
    if let Some(pawn) = pawn {
        pawn.flags |= TURNING;
        pawn.turning_dir = ftol(f64::from(rate));
    }
}

pub fn default_character_stop(pawn: Option<&mut PawnTurn>) {
    if let Some(pawn) = pawn {
        pawn.flags &= !TURNING;
    }
}

pub fn reset_default_character_yaw(pawn: Option<&mut PawnTurn>, table_yaw: &mut f32, rotation_yaw: i32) {
    let Some(pawn) = pawn else { return };
    if f64::from(*table_yaw) > YAW_PERIOD {
        let mut yaw = f64::from(*table_yaw);
        loop {
            yaw -= YAW_PERIOD;
            if yaw <= YAW_PERIOD {
                break;
            }
        }
        *table_yaw = yaw as f32;
    }
    if *table_yaw < 0.0 {
        let mut yaw = f64::from(*table_yaw);
        loop {
            yaw += YAW_PERIOD;
            if yaw >= 0.0 {
                break;
            }
        }
        *table_yaw = yaw as f32;
    }
    if f64::from(rotation_yaw) != f64::from(*table_yaw) {
        pawn.flags |= WANT_CHANGE_YAW;
        pawn.wanted_yaw = ftol(f64::from(*table_yaw));
    }
}

pub fn tick_pawn(pawn: &mut PawnTurn, heading: &mut i32, latent: bool, defaults: &RotationDefaults, delta: f32) {
    let rotation = [0, *heading, 0];
    let mut body = PawnBody {
        physics: PHYS_WALKING,
        flags: if defaults.rotate_to_desired { ROTATE_TO_DESIRED } else { 0 },
        rotation,
        desired: pawn.desired,
        rate: defaults.rate,
        velocity: [0.0; 3],
        accel_rate: defaults.accel_rate,
        bits_1400: 0,
        bits_1404: 0,
        floor_normal: [0.0, 0.0, 1.0],
        controller: Some(ControllerBody { flags: 0, rotation, desired: [0; 3], player_turn_rate: Some(0), spider_axis: [0.0; 3] }),
    };
    if let Some(next) = pawn_rotation::pawn_physics_rotation(&mut body, delta, [0.0; 3]) {
        pawn_rotation::move_actor_rotation(&mut body.rotation, false, true, true, || false, next);
    }
    let mut rotation = body.rotation;
    let mut state = PawnTurnState { flags: pawn.flags, turning_dir: pawn.turning_dir, wanted_yaw: pawn.wanted_yaw, yaw_rate: defaults.keyboard_rate[1], yaw_override: 0 };
    let gate = TurnGate { net_login: true, controller: true, latent, busy: false };
    pawn_rotation::pawn_tick_turn(&mut state, &mut rotation, &mut pawn.desired, None, gate, delta);
    pawn.flags = state.flags;
    pawn.wanted_yaw = state.wanted_yaw;
    *heading = rotation[1];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_character_turn_1042d700_sets_turning_and_truncates_the_rate() {
        let mut pawn = PawnTurn { flags: WANT_CHANGE_YAW, ..PawnTurn::default() };
        default_character_turn(Some(&mut pawn), 2.0);
        assert_eq!(pawn, PawnTurn { flags: WANT_CHANGE_YAW | TURNING, turning_dir: 2, ..PawnTurn::default() });
        default_character_turn(Some(&mut pawn), -2.9);
        assert_eq!(pawn.turning_dir, -2);
        default_character_stop(Some(&mut pawn));
        assert_eq!(pawn.flags, WANT_CHANGE_YAW);
        default_character_turn(None, 2.0);
        default_character_stop(None);
    }

    #[test]
    fn reset_default_character_yaw_1042d7a0_wraps_the_table_and_wants_the_yaw_only_when_it_differs() {
        let mut pawn = PawnTurn::default();
        let mut yaw = 65536.0 * 2.0 + 100.5;
        reset_default_character_yaw(Some(&mut pawn), &mut yaw, 100);
        assert_eq!(yaw, 100.5);
        assert_eq!(pawn, PawnTurn { flags: WANT_CHANGE_YAW, wanted_yaw: 100, ..PawnTurn::default() });
        let mut pawn = PawnTurn::default();
        let mut yaw = 65536.0;
        reset_default_character_yaw(Some(&mut pawn), &mut yaw, 65536);
        assert_eq!(yaw, 65536.0);
        assert_eq!(pawn, PawnTurn::default());
        let mut yaw = -15360.0;
        reset_default_character_yaw(Some(&mut pawn), &mut yaw, 0);
        assert_eq!(yaw, 50176.0);
        assert_eq!(pawn, PawnTurn { flags: WANT_CHANGE_YAW, wanted_yaw: 50176, ..PawnTurn::default() });
        let mut pawn = PawnTurn::default();
        let mut yaw = -65536.0;
        reset_default_character_yaw(Some(&mut pawn), &mut yaw, 0);
        assert_eq!(yaw, 0.0);
        assert_eq!(pawn, PawnTurn::default());
        let mut yaw = -26624.0;
        reset_default_character_yaw(None, &mut yaw, 0);
        assert_eq!(yaw, -26624.0);
        let mut pawn = PawnTurn::default();
        let mut yaw = f32::NAN;
        reset_default_character_yaw(Some(&mut pawn), &mut yaw, 0);
        assert!(yaw.is_nan());
        assert_eq!(pawn.flags, WANT_CHANGE_YAW);
    }

    const DEFAULTS: RotationDefaults = RotationDefaults { rate: [0, 65000, 0], keyboard_rate: [0, 10000, 0], rotate_to_desired: true, accel_rate: 2048.0 };

    #[test]
    fn tick_pawn_105fa168_follows_the_desired_yaw_by_the_rotation_rate_and_the_keyboard_rate_sets_the_desired() {
        let mut pawn = PawnTurn { desired: [0, 1000, 0], ..PawnTurn::default() };
        let mut heading = 1000;
        default_character_turn(Some(&mut pawn), 2.0);
        tick_pawn(&mut pawn, &mut heading, false, &DEFAULTS, 0.05);
        assert_eq!((heading, pawn.desired[1]), (1000, 2000));
        tick_pawn(&mut pawn, &mut heading, false, &DEFAULTS, 0.05);
        assert_eq!((heading, pawn.desired[1]), (2000, 3000));
        default_character_stop(Some(&mut pawn));
        pawn.desired = [0, heading, 0];
        tick_pawn(&mut pawn, &mut heading, false, &DEFAULTS, 0.05);
        assert_eq!((heading, pawn.desired[1]), (2000, 2000));
    }

    #[test]
    fn tick_pawn_10595e30_walks_the_yaw_back_to_the_wanted_one_and_clears_the_flag() {
        let mut pawn = PawnTurn { desired: [0, 60000, 0], ..PawnTurn::default() };
        let mut heading = 60000;
        let mut table_yaw = 2000.0;
        reset_default_character_yaw(Some(&mut pawn), &mut table_yaw, heading);
        assert_eq!(pawn.flags, WANT_CHANGE_YAW);
        let mut frames = 0;
        while pawn.flags & WANT_CHANGE_YAW != 0 && frames < 1000 {
            tick_pawn(&mut pawn, &mut heading, false, &DEFAULTS, 0.016);
            frames += 1;
        }
        for _ in 0..4 {
            tick_pawn(&mut pawn, &mut heading, false, &DEFAULTS, 0.016);
        }
        assert!(frames > 5 && frames < 1000, "{frames}");
        assert_eq!(heading, 2000);
        assert_eq!(pawn.desired[1], 2000);
    }
}
