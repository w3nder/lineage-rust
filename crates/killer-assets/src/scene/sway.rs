use super::transform::rotator_matrix;
use glam::Vec3;
use std::f32::consts::PI;

const DEGREES_TO_ROTATOR: f32 = 182.0;
const TIME_STEP: f32 = 0.05;
const AMPLITUDE_SCALE: f32 = 0.5;

/// The per-actor state `FUN_10554cc0` keeps in `Acceleration`: sign, time and the random yaw amplitude.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SwayState {
    pub sign: f32,
    pub time: f32,
    pub amplitude: f32,
}

/// One entry of `Winds` (`Level+0x74 → +0x2cc`): strength and direction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wind {
    pub strength: f32,
    pub direction: Vec3,
}

fn rotation_of(direction: Vec3) -> [i32; 3] {
    let yaw = direction.y.atan2(direction.x);
    let pitch = direction.z.atan2(direction.truncate().length());
    [(pitch / (2.0 * PI) * 65536.0) as i32, (yaw / (2.0 * PI) * 65536.0) as i32, 0]
}

fn add(a: [i32; 3], b: [i32; 3]) -> [i32; 3] {
    [a[0].wrapping_add(b[0]), a[1].wrapping_add(b[1]), a[2].wrapping_add(b[2])]
}

/// `FUN_10554cc0`: one render call of the sway for one wind; returns the actor's new rotation.
pub fn sway(wind: &Wind, original: [i32; 3], state: &mut SwayState, frequency: f32, max_angle: f32, random: &mut impl FnMut() -> f32) -> [i32; 3] {
    if state.sign == 0.0 {
        state.sign = if random() < 0.5 { 1.0 } else { -1.0 };
        state.time = 0.0;
        state.amplitude = 0.0;
    }
    let combined = add(rotation_of(wind.direction), original);
    let dir = (rotator_matrix(combined) * Vec3::X).normalize_or(Vec3::X) * state.sign;
    let delta = [
        (dir.x * max_angle * DEGREES_TO_ROTATOR) as i32,
        (max_angle * state.amplitude * DEGREES_TO_ROTATOR + original[1] as f32) as i32,
        (dir.y * max_angle * DEGREES_TO_ROTATOR) as i32,
    ];
    let s = ((1.0 / frequency) * state.time * PI).sin();
    let mut rotation = [0; 3];
    for axis in 0..3 {
        rotation[axis] = original[axis] + ((delta[axis] - original[axis]) as f32 * s) as i32;
    }
    state.time += TIME_STEP;
    if state.time > frequency {
        state.time = 0.0;
        state.sign = -state.sign;
        state.amplitude = (random() - 0.5) * max_angle * AMPLITUDE_SCALE;
    }
    if wind.strength == 0.0 {
        state.sign = 0.0;
        return original;
    }
    rotation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frequency_of_fifteen_swings_over_six_hundred_frames_to_the_max_angle() {
        let wind = Wind { strength: 1.0, direction: Vec3::X };
        let mut state = SwayState::default();
        let mut random = || 0.9;
        let original = [0, 0, 0];
        let mut peak = 0i32;
        let mut sign_changes = 0;
        let mut last_sign = 0.0;
        for _ in 0..600 {
            let rotation = sway(&wind, original, &mut state, 15.0, 10.0, &mut random);
            peak = peak.max(rotation[0].abs());
            if state.sign != last_sign {
                sign_changes += 1;
                last_sign = state.sign;
            }
        }
        assert_eq!(sign_changes, 3);
        assert!((peak - 1820).abs() <= 2, "{peak}");
        assert!(state.time < 0.2, "{}", state.time);
    }

    #[test]
    fn no_wind_keeps_the_original_pose_and_resets_the_state() {
        let calm = Wind { strength: 0.0, direction: Vec3::X };
        let mut state = SwayState::default();
        let mut random = || 0.1;
        assert_eq!(sway(&calm, [100, 200, 300], &mut state, 15.0, 10.0, &mut random), [100, 200, 300]);
        assert_eq!(state.sign, 0.0);
    }
}
