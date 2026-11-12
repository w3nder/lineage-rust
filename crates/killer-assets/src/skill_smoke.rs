use glam::Vec3;

use crate::engine_math::{Coords, coords_mul_rotator, transform_vector_by};
use crate::scene::transform::vector_rotation;
use crate::skill_effects::Actor;

pub const FAR_RADIUS: f32 = f32::from_bits(0x453b_8000);
pub const NEAR_RADIUS: f32 = f32::from_bits(0x44fa_0000);
pub const RIGHT_HAND: &str = "bip01 r hand";
pub const LEFT_HAND: &str = "bip01 l hand";
const AHEAD_100A: f32 = f32::from_bits(0x4396_0000);
const BEHIND_100B: f32 = f32::from_bits(0x4416_0000);
const AHEAD_100E: f32 = f32::from_bits(0x446d_8000);
const FORWARD: [f32; 3] = [1.0, 0.0, 0.0];
const SHAKE_SCALE: f32 = 1.0;
const SHAKE_UNIT: f64 = 1.0;
const SHAKE_DIVISOR: f32 = f32::from_bits(0x42c8_0000);
const SHAKE_FIXED_AXIS: [f32; 3] = [0.0, 1.0, 0.0];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SmokeShake {
    pub kind: u8,
    pub time: f32,
    pub scale: f32,
    pub frequency: f32,
    pub count: f32,
    pub direction: [f32; 3],
    pub second: [f32; 3],
    pub third: [f32; 3],
    pub origin: [f32; 3],
    pub intensity: f32,
    pub range: f32,
}

struct ShakeRow {
    kind: u8,
    time: u32,
    frequency: u32,
    count: u32,
    intensity: u32,
    range: f32,
    random: bool,
}

fn shake_row(skill_id: i32) -> Option<ShakeRow> {
    let row = |frequency: u32, intensity: u32, range: f32| ShakeRow { kind: 0, time: 0x40c0_0000, frequency, count: 0x42f0_0000, intensity, range, random: true };
    Some(match skill_id {
        0x100a => row(0x469c_4000, 0x43c8_0000, FAR_RADIUS),
        0x100b => row(0x465a_c000, 0x438c_0000, FAR_RADIUS),
        0x1010 | 0x1011 => row(0x465a_c000, 0x438c_0000, NEAR_RADIUS),
        0x100e => ShakeRow { kind: 5, time: 0x4120_0000, frequency: 0x459c_4000, count: 0x42c8_0000, intensity: 0x42c8_0000, range: NEAR_RADIUS, random: false },
        _ => return None,
    })
}

pub fn shake_draws_random(skill_id: i32) -> bool {
    shake_row(skill_id).is_some_and(|row| row.random)
}

fn divide(vector: [f32; 3], divisor: f32) -> [f32; 3] {
    let wide = SHAKE_UNIT / f64::from(divisor);
    let narrow = f64::from(wide as f32);
    [(f64::from(vector[0]) * narrow) as f32, (f64::from(vector[1]) * narrow) as f32, (f64::from(vector[2]) * wide) as f32]
}

pub fn smoke_shake(skill_id: i32, origin: Vec3, random: f32) -> Option<SmokeShake> {
    let row = shake_row(skill_id)?;
    let axis = if row.random { [0.0, random, 0.0] } else { SHAKE_FIXED_AXIS };
    let (x, y) = (f64::from(axis[0]), f64::from(axis[1]));
    let k = SHAKE_UNIT / (y * y + x * x).sqrt();
    let unit = [(x * k) as f32, (y * k) as f32, axis[2]];
    let intensity = f32::from_bits(row.intensity);
    let count = f32::from_bits(row.count);
    let second = unit.map(|component| {
        let product = (f64::from(intensity) * f64::from(component)) as f32;
        (f64::from(product) + f64::from(product)) as f32
    });
    Some(SmokeShake {
        kind: row.kind,
        time: f32::from_bits(row.time),
        scale: SHAKE_SCALE,
        frequency: f32::from_bits(row.frequency),
        count,
        direction: divide(unit, SHAKE_DIVISOR),
        second,
        third: [count, count, 0.0],
        origin: origin.to_array(),
        intensity,
        range: row.range,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SmokeAt {
    Point(Vec3),
    Bone { name: &'static str, z: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShotSmoke {
    pub at: SmokeAt,
    pub radius: f32,
}

fn forward(rotation: [i32; 3]) -> [f32; 3] {
    transform_vector_by(FORWARD, &coords_mul_rotator(&Coords::UNIT, rotation))
}

fn offset(origin: f32, axis: f32, distance: f32, sign: f64) -> f32 {
    let step = (f64::from(axis) * f64::from(distance)) as f32;
    (f64::from(origin) + sign * f64::from(step)) as f32
}

fn below(target: &Actor, drop: f32) -> f32 {
    (f64::from(target.position.z) - f64::from(drop)) as f32
}

fn point(origin: Vec3, rotation: [i32; 3], distance: f32, sign: f64, z: f32) -> SmokeAt {
    let axis = forward(rotation);
    SmokeAt::Point(Vec3::new(offset(origin.x, axis[0], distance, sign), offset(origin.y, axis[1], distance, sign), z))
}

pub fn shot_smoke(skill_id: i32, caster: &Actor, target: Option<&Actor>) -> Option<ShotSmoke> {
    let (at, radius) = match skill_id {
        0x100a | 0x100b => {
            let (distance, sign) = if skill_id == 0x100a { (AHEAD_100A, 1.0) } else { (BEHIND_100B, -1.0) };
            let at = match target {
                Some(target) => point(caster.position, caster.rotation, distance, sign, below(target, target.mesh_origin_z?)),
                None => SmokeAt::Point(Vec3::ZERO),
            };
            (at, FAR_RADIUS)
        }
        0x100e => {
            let at = match target {
                Some(target) => {
                    let delta = Vec3::new(
                        (f64::from(target.position.x) - f64::from(caster.position.x)) as f32,
                        (f64::from(target.position.y) - f64::from(caster.position.y)) as f32,
                        (f64::from(target.position.z) - f64::from(caster.position.z)) as f32,
                    );
                    point(caster.position, vector_rotation(delta), AHEAD_100E, 1.0, below(target, target.mesh_origin_z.unwrap_or(target.height)))
                }
                None => SmokeAt::Point(Vec3::ZERO),
            };
            (at, NEAR_RADIUS)
        }
        0x1010 => {
            let at = match target {
                Some(target) => SmokeAt::Bone { name: RIGHT_HAND, z: below(target, target.height) },
                None => SmokeAt::Point(Vec3::ZERO),
            };
            (at, NEAR_RADIUS)
        }
        0x1011 => (SmokeAt::Bone { name: LEFT_HAND, z: below(target?, target?.height) }, NEAR_RADIUS),
        _ => return None,
    };
    Some(ShotSmoke { at, radius })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pawn(position: [f32; 3], yaw: i32, height: f32, mesh_origin_z: Option<f32>) -> Actor {
        Actor { position: Vec3::from(position), rotation: [0, yaw, 0], height, mesh_origin_z, ..Actor::default() }
    }

    fn bits(smoke: ShotSmoke) -> [u32; 3] {
        let SmokeAt::Point(point) = smoke.at else { panic!("ponto") };
        [point.x.to_bits(), point.y.to_bits(), point.z.to_bits()]
    }

    #[test]
    fn smoke_points_match_exec_l2_event_actors_arguments_of_the_engine() {
        let caster = pawn([-1057.0, -1639.8, -222.0], 59294, 40.0, None);
        let target = pawn([-1445.9, -2359.0, 2791.5], 0, 20.86, Some(2.0));
        let smoke = shot_smoke(0x100a, &caster, Some(&target)).expect("0x100a");
        assert_eq!(smoke.radius, 3000.0);
        assert_eq!(bits(smoke), [0xc44a_4ac9, 0xc4e2_1b5b, 0x452e_5800]);
        assert_eq!(bits(shot_smoke(0x100b, &caster, Some(&target)).expect("0x100b")), [0xc4c2_1537, 0xc4a2_b617, 0x452e_5800]);
        let bare = pawn([-1445.9, -2359.0, 2791.5], 0, 20.86, None);
        let smoke = shot_smoke(0x100e, &caster, Some(&bare)).expect("0x100e");
        assert_eq!(smoke.radius, 2000.0);
        assert_eq!(bits(smoke), [0xc492_ec2c, 0xc4e8_5492, 0x452d_2a3d]);
        let caster = pawn([-2359.0, 2791.5, -830.9], 18398, 40.0, None);
        let target = pawn([1483.7, 891.0, -2322.3], 0, 84.96, Some(2.0));
        assert_eq!(bits(shot_smoke(0x100a, &caster, Some(&target)).expect("0x100a")), [0xc517_082e, 0x4540_def9, 0xc511_44cd]);
        assert_eq!(bits(shot_smoke(0x100b, &caster, Some(&target)).expect("0x100b")), [0xc50c_3fa4, 0x4509_aa0e, 0xc511_44cd]);
        let bare = pawn([1483.7, 891.0, -2322.3], 0, 84.96, None);
        assert_eq!(bits(shot_smoke(0x100e, &caster, Some(&bare)).expect("0x100e")), [0xc4c2_591a, 0x4515_9a04, 0xc516_7429]);
    }

    #[test]
    fn smoke_without_target_or_mesh_follows_the_engine_branches() {
        let caster = pawn([0.0; 3], 0, 40.0, None);
        let target = pawn([10.0, 20.0, 300.0], 0, 60.0, None);
        assert_eq!(shot_smoke(0x100a, &caster, Some(&target)), None);
        assert_eq!(shot_smoke(0x100a, &caster, None).map(|smoke| smoke.at), Some(SmokeAt::Point(Vec3::ZERO)));
        assert_eq!(shot_smoke(0x100e, &caster, None).map(|smoke| smoke.at), Some(SmokeAt::Point(Vec3::ZERO)));
        assert_eq!(shot_smoke(0x1010, &caster, Some(&target)), Some(ShotSmoke { at: SmokeAt::Bone { name: RIGHT_HAND, z: 240.0 }, radius: 2000.0 }));
        assert_eq!(shot_smoke(0x1011, &caster, Some(&target)), Some(ShotSmoke { at: SmokeAt::Bone { name: LEFT_HAND, z: 240.0 }, radius: 2000.0 }));
        assert_eq!(shot_smoke(0x1011, &caster, None), None);
        assert_eq!(shot_smoke(0x100f, &caster, Some(&target)), None);
    }

    #[test]
    fn add_view_shake_state_of_the_five_smoke_shots_uses_the_pushed_constants() {
        let origin = Vec3::new(10.0, -20.0, 30.0);
        let shake = smoke_shake(0x100a, origin, 0.5).expect("0x100a");
        assert_eq!((shake.kind, shake.time, shake.scale, shake.frequency, shake.count), (0, 6.0, 1.0, 20000.0, 120.0));
        assert_eq!((shake.intensity, shake.range, shake.origin, shake.third), (400.0, 3000.0, [10.0, -20.0, 30.0], [120.0, 120.0, 0.0]));
        assert_eq!(shake.second, [0.0, 800.0, 0.0]);
        assert_eq!(shake.direction, [0.0, 0.01_f32, 0.0]);
        let behind = smoke_shake(0x100b, origin, 0.25).expect("0x100b");
        assert_eq!((behind.frequency, behind.intensity, behind.range, behind.second), (14000.0, 280.0, 3000.0, [0.0, 560.0, 0.0]));
        for skill in [0x1010, 0x1011] {
            let hand = smoke_shake(skill, origin, 0.75).expect("mão");
            assert_eq!((hand.kind, hand.frequency, hand.intensity, hand.range, hand.count), (0, 14000.0, 280.0, 2000.0, 120.0));
        }
        let fixed = smoke_shake(0x100e, origin, 0.9).expect("0x100e");
        assert_eq!((fixed.kind, fixed.time, fixed.frequency, fixed.count, fixed.intensity, fixed.range), (5, 10.0, 5000.0, 100.0, 100.0, 2000.0));
        assert_eq!((fixed.second, fixed.third, fixed.direction), ([0.0, 200.0, 0.0], [100.0, 100.0, 0.0], [0.0, 0.01_f32, 0.0]));
        assert_eq!(fixed, smoke_shake(0x100e, origin, 0.1).expect("0x100e"));
        assert!(shake_draws_random(0x100a) && shake_draws_random(0x1011) && !shake_draws_random(0x100e) && !shake_draws_random(0x100f));
        assert_eq!(smoke_shake(0x100f, origin, 0.5), None);
        let zero = smoke_shake(0x100a, origin, 0.0).expect("0x100a");
        assert!(zero.direction[0].is_nan() && zero.direction[1].is_nan() && zero.direction[2] == 0.0);
    }
}
