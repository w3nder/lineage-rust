use crate::engine_math::{Coords, coords_div_rotator, ortho_rotation, rotator_forward, transform_vector_by};
use crate::scene::transform::vector_rotation;
use glam::Vec3;

pub type Rotator = [i32; 3];

pub const FIXED_DIRECTION: u32 = 0x2000;
pub const ROTATE_TO_DESIRED: u32 = 0x4000;
pub const INTERPOLATING: u32 = 0x8000;
pub const PHYS_WALKING: u8 = 1;
pub const PHYS_SPIDER: u8 = 9;
pub const PHYS_ROOT_MOTION: u8 = 12;
pub const PHYS_KARMA: u8 = 13;
pub const PHYS_KARMA_RAG_DOLL: u8 = 14;
pub const SLOPE_BIT: u32 = 0x100;
pub const ROLL_TO_DESIRED: u32 = 0x4000;
pub const WANT_CHANGE_YAW: u32 = 1;
pub const TURNING: u32 = 2;
pub const SLOPE_Z_LIMIT: f64 = 0.95;
pub const ROLL_SPEED_LIMIT_SQUARED: f64 = 40000.0;
pub const ACCELERATION_LIMIT_SQUARED: f64 = 10000.0;
pub const ACCELERATION_CAP: f32 = 28000.0;
pub const WALKING_ACCELERATION_CAP: f32 = 4096.0;
pub const ROLL_DECAY_RATE: f64 = 8.0;
pub const ROLL_FOLLOW_RATE: f64 = 5.0;
pub const PLAYER_TURN_FACTOR: i32 = 2;
const I64_LIMIT: f64 = 9_223_372_036_854_775_808.0;
const FULL_TURN: i32 = 0x10000;
const HALF_TURN: i32 = 0x8000;

pub fn ftol(value: f64) -> i32 {
    let whole = value.trunc();
    if whole.is_nan() || whole.abs() >= I64_LIMIT { 0 } else { (whole as i64) as i32 }
}

pub fn app_round(value: f32) -> i32 {
    (f64::from(value) + 0.5).floor() as i32
}

pub fn fixed_turn(flags: u32, current: i32, desired: i32, delta: i32) -> i32 {
    let start = current as u32 & 0xffff;
    if delta == 0 {
        return start as i32;
    }
    let target = desired as u32 & 0xffff;
    let finish = |amount: i32| (start.wrapping_add(amount as u32) & 0xffff) as i32;
    if flags & FIXED_DIRECTION != 0 {
        if flags & ROTATE_TO_DESIRED == 0 {
            return finish(delta);
        }
        if delta < 1 {
            let from = if start < target { start + 0x10000 } else { start };
            let gap = target.wrapping_sub(from) as i32;
            return finish(if gap <= delta { delta } else { gap });
        }
        let to = if target < start { target + 0x10000 } else { target };
        let gap = to.wrapping_sub(start) as i32;
        return finish(if delta <= gap { delta } else { gap });
    }
    if flags & ROTATE_TO_DESIRED != 0 {
        let magnitude = delta.wrapping_abs();
        let behind = if target < start {
            let gap = (start - target) as i32;
            if gap > 0x7fff {
                let ahead = target.wrapping_sub(start).wrapping_add(0x10000) as i32;
                return finish(if magnitude < ahead { magnitude } else { ahead });
            }
            gap
        } else {
            let gap = (target - start) as i32;
            if gap < HALF_TURN {
                return finish(if magnitude < gap { magnitude } else { gap });
            }
            start.wrapping_sub(target).wrapping_add(0x10000) as i32
        };
        let amount = if magnitude < behind { magnitude } else { behind };
        return (start.wrapping_sub(amount as u32) & 0xffff) as i32;
    }
    start as i32
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActorBody {
    pub flags: u32,
    pub rotation: Rotator,
    pub desired: Rotator,
    pub rate: Rotator,
}

pub fn scale_rotator(rotator: Rotator, scale: f32) -> Rotator {
    rotator.map(|axis| ftol(f64::from(axis) * f64::from(scale)))
}

pub fn actor_physics_rotation(actor: &mut ActorBody, delta: f32, move_actor: impl FnOnce(Rotator) -> Rotator) -> bool {
    let rotating = actor.flags & ROTATE_TO_DESIRED != 0;
    if !rotating && actor.flags & FIXED_DIRECTION == 0 {
        return false;
    }
    if rotating && actor.rotation == actor.desired {
        return false;
    }
    let mut next = actor.rotation;
    let step = scale_rotator(actor.rate, delta);
    for axis in [1, 0, 2] {
        if step[axis] != 0 && (!rotating || actor.desired[axis] != next[axis]) {
            next[axis] = fixed_turn(actor.flags, next[axis], actor.desired[axis], step[axis]);
        }
    }
    if next != actor.rotation {
        actor.rotation = move_actor(next);
    }
    rotating && actor.rotation == actor.desired
}

pub fn move_actor_rotation(rotation: &mut Rotator, deleted: bool, movable: bool, collides: bool, encroached: impl FnOnce() -> bool, next: Rotator) -> bool {
    if deleted || !movable {
        return false;
    }
    if *rotation == next {
        return true;
    }
    let previous = *rotation;
    *rotation = next;
    if collides && encroached() {
        *rotation = previous;
        return false;
    }
    true
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControllerBody {
    pub flags: u32,
    pub rotation: Rotator,
    pub desired: Rotator,
    pub player_turn_rate: Option<i32>,
    pub spider_axis: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PawnBody {
    pub physics: u8,
    pub flags: u32,
    pub rotation: Rotator,
    pub desired: Rotator,
    pub rate: Rotator,
    pub velocity: [f32; 3],
    pub accel_rate: f32,
    pub bits_1400: u32,
    pub bits_1404: u32,
    pub floor_normal: [f32; 3],
    pub controller: Option<ControllerBody>,
}

fn is_nearly_zero(vector: [f32; 3]) -> bool {
    vector.iter().all(|axis| axis.abs() < 0.0001)
}

pub fn actor_find_slope_rotation(normal: [f32; 3], rotation: Rotator) -> Rotator {
    let fallback = [0, rotation[1], 0];
    let [nx, ny, nz] = normal.map(f64::from);
    let tilted = f64::from(normal[2]) < SLOPE_Z_LIMIT;
    if !tilted || is_nearly_zero(normal) {
        return fallback;
    }
    let forward = rotator_forward([0, rotation[1], 0]).to_array().map(f64::from);
    let [fx, fy, fz] = forward;
    let along = ny * fy + fz * nz + nx * fx;
    let projected = [(fx - f64::from((nx * along) as f32)) as f32, (fy - f64::from((ny * along) as f32)) as f32, (fz - f64::from((nz * along) as f32)) as f32];
    let pitch = vector_rotation(Vec3::from(projected))[0];
    let [tx, ty, tz] = projected.map(f64::from);
    let side = [(ty * nz - tz * ny) as f32, (tz * nx - tx * nz) as f32, (ny * tx - ty * nx) as f32];
    let roll = vector_rotation(Vec3::from(side))[0];
    [pitch, rotation[1], roll]
}

pub fn spider_slope_rotation(floor: [f32; 3], axis: [f32; 3]) -> Rotator {
    let [fx, fy, fz] = floor.map(f64::from);
    let [ax, ay, az] = axis.map(f64::from);
    let x = [(ay * fz - az * fy) as f32, (az * fx - ax * fz) as f32, (ax * fy - ay * fx) as f32];
    ortho_rotation(&Coords { origin: [0.0; 3], x, y: axis, z: floor })
}

pub fn pawn_find_slope_rotation(pawn: &PawnBody, normal: [f32; 3], rotation: Rotator) -> Rotator {
    match pawn.controller {
        Some(controller) if pawn.physics == PHYS_SPIDER => spider_slope_rotation(pawn.floor_normal, controller.spider_axis),
        _ => actor_find_slope_rotation(normal, rotation),
    }
}

fn blend(ratio: f64) -> f64 {
    if 1.0 > ratio || ratio.is_nan() { ratio } else { 1.0 }
}

fn decay_roll(roll: i32, factor: f64) -> i32 {
    if roll < HALF_TURN {
        ftol((1.0 - factor) * f64::from(roll))
    } else {
        ftol(f64::from(FULL_TURN - roll) * factor + f64::from(roll))
    }
}

fn size_squared(vector: [f32; 3]) -> f64 {
    let [x, y, z] = vector.map(f64::from);
    x * x + y * y + z * z
}

fn bank_roll(pawn: &mut PawnBody, rotation: Rotator, delta: f32, old_velocity: [f32; 3]) -> i32 {
    let roll_rate = pawn.rate[2];
    if roll_rate <= 0 {
        return 0;
    }
    let mut roll = rotation[2] & 0xffff;
    if roll < HALF_TURN {
        roll = roll.min(roll_rate);
    } else if roll < FULL_TURN - roll_rate {
        roll = FULL_TURN - roll_rate;
    }
    let dt = f64::from(delta);
    if pawn.physics == PHYS_WALKING && size_squared(pawn.velocity) < ROLL_SPEED_LIMIT_SQUARED {
        return decay_roll(roll, blend(dt * ROLL_DECAY_RATE));
    }
    let scale = 1.0 / delta;
    let accel = [0, 1, 2].map(|axis| scale * (pawn.velocity[axis] - old_velocity[axis]));
    let magnitude = size_squared(accel);
    if magnitude <= ACCELERATION_LIMIT_SQUARED || magnitude.is_nan() {
        return decay_roll(roll, blend(dt * ROLL_DECAY_RATE));
    }
    let cap = if pawn.physics == PHYS_WALKING { WALKING_ACCELERATION_CAP } else { ACCELERATION_CAP };
    let local = transform_vector_by(accel, &coords_div_rotator(&Coords::UNIT, [rotation[0], rotation[1], 0]));
    let lateral = f64::from(local[1]) / f64::from(pawn.accel_rate) * f64::from(cap);
    let target = if local[1] > 0.0 { roll_rate.min(ftol(lateral)) } else { (FULL_TURN - roll_rate).max(ftol(lateral + f64::from(FULL_TURN))) };
    pawn.rotation[2] &= 0xffff;
    if target > HALF_TURN {
        if pawn.rotation[2] < HALF_TURN {
            pawn.rotation[2] += FULL_TURN;
        }
    } else if pawn.rotation[2] > HALF_TURN {
        pawn.rotation[2] -= FULL_TURN;
    }
    let follow = blend(dt * ROLL_FOLLOW_RATE);
    ftol((1.0 - follow) * f64::from(pawn.rotation[2]) + f64::from(target) * follow)
}

pub fn pawn_physics_rotation(pawn: &mut PawnBody, delta: f32, old_velocity: [f32; 3]) -> Option<Rotator> {
    let mut rotation = pawn.rotation;
    let grounded = pawn.physics == PHYS_WALKING || pawn.physics == PHYS_ROOT_MOTION;
    pawn.flags &= !FIXED_DIRECTION;
    let mut step = 0;
    if let Some(mut controller) = pawn.controller {
        match controller.player_turn_rate.filter(|_| controller.flags & ROTATE_TO_DESIRED != 0) {
            Some(turn_rate) => {
                step = app_round((f64::from(turn_rate.wrapping_mul(PLAYER_TURN_FACTOR)) * f64::from(delta)) as f32);
                if !grounded && controller.desired[0] != rotation[0] {
                    rotation[0] = fixed_turn(pawn.flags, rotation[0], controller.desired[0], step);
                }
                if controller.desired[1] != rotation[1] {
                    rotation[1] = fixed_turn(pawn.flags, rotation[1], controller.desired[1], step);
                }
                controller.rotation = rotation;
                pawn.controller = Some(controller);
            }
            None => {
                step = ftol(f64::from(pawn.rate[1]) * f64::from(delta));
                pawn.flags = (pawn.flags & !FIXED_DIRECTION) | ROTATE_TO_DESIRED;
                if pawn.desired[1] != rotation[1] {
                    rotation[1] = fixed_turn(pawn.flags, rotation[1], pawn.desired[1], step);
                }
            }
        }
    }
    if pawn.bits_1404 & ROLL_TO_DESIRED != 0 {
        if pawn.desired[2] != rotation[2] {
            rotation[2] = fixed_turn(pawn.flags, rotation[2], pawn.desired[2], step);
        }
    } else if pawn.bits_1400 & SLOPE_BIT != 0 {
        if grounded {
            rotation = pawn_find_slope_rotation(pawn, pawn.floor_normal, rotation);
        } else {
            let amount = ftol(f64::from(pawn.rate[1]) * f64::from(delta));
            rotation[0] = fixed_turn(pawn.flags, rotation[0], 0, amount);
            rotation[2] = fixed_turn(pawn.flags, rotation[2], 0, amount);
        }
    } else {
        rotation[2] = bank_roll(pawn, rotation, delta, old_velocity);
    }
    (rotation != pawn.rotation).then_some(rotation)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PawnTurnState {
    pub flags: u32,
    pub turning_dir: i32,
    pub wanted_yaw: i32,
    pub yaw_rate: i32,
    pub yaw_override: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TurnGate {
    pub net_login: bool,
    pub controller: bool,
    pub latent: bool,
    pub busy: bool,
}

fn wrap_wanted(angle: i32) -> i32 {
    let mut angle = angle;
    while angle < -0x7fff {
        angle += FULL_TURN;
    }
    while angle > HALF_TURN {
        angle -= FULL_TURN;
    }
    angle
}

pub fn pawn_tick_turn(state: &mut PawnTurnState, rotation: &mut Rotator, desired: &mut Rotator, lineage_yaw: Option<&mut f32>, gate: TurnGate, delta: f32) {
    let dt = f64::from(delta);
    if state.flags & TURNING != 0 && gate.controller && !gate.latent {
        *desired = *rotation;
        let travel = f64::from(state.turning_dir.wrapping_mul(state.yaw_rate));
        desired[1] = ftol(travel * dt + f64::from(desired[1]));
        if let Some(yaw) = lineage_yaw {
            *yaw = (travel * dt + f64::from(*yaw)) as f32;
        }
        return;
    }
    if gate.net_login && state.flags & WANT_CHANGE_YAW != 0 && gate.controller && !gate.latent && !gate.busy {
        *desired = *rotation;
        state.wanted_yaw = wrap_wanted(state.wanted_yaw);
        rotation[1] = wrap_wanted(rotation[1]);
        let (wanted, current) = (state.wanted_yaw, rotation[1]);
        let step = f64::from(state.yaw_rate) * dt;
        let arrives = if wanted < current { f64::from(current) < step + f64::from(wanted) } else { f64::from(wanted) - step < f64::from(current) && current < wanted };
        if arrives {
            desired[1] = wanted;
            state.flags &= !WANT_CHANGE_YAW;
            state.yaw_override = 0;
        } else {
            let speed = if state.yaw_override == 0 { state.yaw_rate } else { state.yaw_override };
            let travel = f64::from(speed) * dt;
            desired[1] = if current < wanted { ftol(travel + f64::from(desired[1])) } else { ftol(f64::from(desired[1]) - travel) };
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RotationDefaults {
    pub rate: Rotator,
    pub keyboard_rate: Rotator,
    pub rotate_to_desired: bool,
    pub accel_rate: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RotationRates {
    pub players: std::collections::HashMap<usize, i32>,
    pub npcs: std::collections::HashMap<u32, i32>,
}

pub const PAWN_ACCEL_RATE: f32 = 2048.0;

pub fn turn_yaw(heading: i32, desired: i32, rate_yaw: i32, delta: f32) -> i32 {
    let rotation = [0, heading, 0];
    let mut body = PawnBody {
        physics: PHYS_WALKING,
        flags: ROTATE_TO_DESIRED,
        rotation,
        desired: [0, desired, 0],
        rate: [0, rate_yaw, 0],
        velocity: [0.0; 3],
        accel_rate: PAWN_ACCEL_RATE,
        bits_1400: 0,
        bits_1404: 0,
        floor_normal: [0.0, 0.0, 1.0],
        controller: Some(ControllerBody { flags: 0, rotation, desired: [0; 3], player_turn_rate: Some(0), spider_axis: [0.0; 3] }),
    };
    let Some(next) = pawn_physics_rotation(&mut body, delta, [0.0; 3]) else { return heading };
    let mut current = rotation;
    move_actor_rotation(&mut current, false, true, true, || false, next);
    current[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_actor_rotation_10309782_keeps_equal_rotations_and_reverts_on_encroachment() {
        let mut rotation = [0, 100, 0];
        assert!(move_actor_rotation(&mut rotation, false, true, true, || unreachable!(), [0, 100, 0]));
        assert!(move_actor_rotation(&mut rotation, false, true, false, || unreachable!(), [0, 200, 0]));
        assert_eq!(rotation, [0, 200, 0]);
        assert!(!move_actor_rotation(&mut rotation, false, true, true, || true, [0, 300, 0]));
        assert_eq!(rotation, [0, 200, 0]);
        assert!(move_actor_rotation(&mut rotation, false, true, true, || false, [0, 300, 0]));
        assert_eq!(rotation, [0, 300, 0]);
        assert!(!move_actor_rotation(&mut rotation, true, true, true, || false, [0, 1, 0]));
        assert!(!move_actor_rotation(&mut rotation, false, false, true, || false, [0, 1, 0]));
        assert_eq!(rotation, [0, 300, 0]);
    }

    #[test]
    fn turn_yaw_105fa168_takes_the_short_way_by_the_rate_times_the_frame() {
        assert_eq!(turn_yaw(0, 30000, 65000, 0.01), 649);
        assert_eq!(turn_yaw(0, 60000, 65000, 0.01), 65536 - 649);
        assert_eq!(turn_yaw(100, 300, 65000, 0.01), 300);
        assert_eq!(turn_yaw(100, 300, 0, 0.01), 100);
    }
}
