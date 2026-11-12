use crate::collision::transform::safe_normal;
use crate::engine_math::{Coords, coords_div_rotator, coords_mul_rotator, transform_point_by, transform_vector_by};

pub const AXIS_BONE_TO_BONE: u8 = 3;
pub const AXIS_START_BONE_DIRECTION: u8 = 4;
pub const AXIS_NORMAL: u8 = 5;
pub const FLAG_BONES: u32 = 0x40;
pub const FLAG_DISTANCE: u32 = 0x80;
pub const COORDINATE_RELATIVE: u8 = 1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bone {
    pub origin: [f32; 3],
    pub rotation: [i32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Actor {
    pub location: [f32; 3],
    pub rotation: [i32; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Emitter {
    pub coordinate_system: u8,
    pub axis: u8,
    pub flags: u32,
    pub width: f32,
    pub origin_offset: f32,
    pub start_offset: [f32; 3],
    pub end_offset: [f32; 3],
    pub rotation: [i32; 3],
    pub has_start_name: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Host {
    pub actor: Actor,
    pub owner: Option<Actor>,
    pub owner_mesh: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Warning {
    MissingStartBone,
    MissingEndBone,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    pub a: [f32; 3],
    pub b: [f32; 3],
    pub width: f32,
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn size(v: [f32; 3]) -> f32 {
    let [x, y, z] = v.map(f64::from);
    ((x * x + y * y) + z * z).sqrt() as f32
}

fn is_zero(rotation: [i32; 3]) -> bool {
    rotation.iter().all(|&angle| angle & 0xffff == 0)
}

pub fn get_new_ribbon_point(
    emitter: &mut Emitter,
    host: Option<&Host>,
    instance: bool,
    start: Option<Bone>,
    end: Option<Bone>,
    warnings: &mut Vec<Warning>,
    initial_width: f32,
) -> Sample {
    let mut sample = Sample { a: [0.0; 3], b: [0.0; 3], width: initial_width };
    let Some(host) = host else { return sample };
    let bones = emitter.flags & FLAG_BONES != 0 && host.owner.is_some() && host.owner_mesh && emitter.has_start_name;
    let reference = match host.owner {
        Some(owner) if matches!(emitter.axis, AXIS_BONE_TO_BONE | AXIS_START_BONE_DIRECTION) => owner,
        _ => host.actor,
    };
    sample.width = emitter.width;
    if !bones || !instance {
        return sample;
    }
    if start.is_none() {
        warnings.push(Warning::MissingStartBone);
        emitter.flags &= !FLAG_BONES;
        emitter.axis = AXIS_NORMAL;
    }
    if emitter.axis != AXIS_BONE_TO_BONE {
        return sample;
    }
    if start.is_some() && end.is_none() {
        warnings.push(Warning::MissingEndBone);
        emitter.axis = AXIS_START_BONE_DIRECTION;
    }
    if emitter.axis != AXIS_BONE_TO_BONE {
        return sample;
    }
    let (Some(start), Some(end)) = (start, end) else { return sample };
    let start_coords = coords_mul_rotator(&Coords::UNIT, start.rotation);
    let end_coords = coords_mul_rotator(&Coords::UNIT, end.rotation);
    let first = transform_vector_by(emitter.start_offset, &start_coords);
    let second = transform_vector_by(emitter.end_offset, &end_coords);
    let a = add(first, start.origin);
    let b = add(second, end.origin);
    let distance = [b[0] - a[0], b[1] - a[1], ((f64::from(second[2]) + f64::from(end.origin[2])) - (f64::from(first[2]) + f64::from(start.origin[2]))) as f32];
    let turn = (!is_zero(emitter.rotation)).then(|| coords_mul_rotator(&Coords::UNIT, emitter.rotation));
    if emitter.flags & FLAG_DISTANCE != 0 {
        sample.width = size(distance);
    }
    match emitter.coordinate_system {
        COORDINATE_RELATIVE => {
            let inverse = coords_div_rotator(&Coords::UNIT, reference.rotation);
            let moved = [a[0] - reference.location[0], a[1] - reference.location[1], ((f64::from(first[2]) + f64::from(start.origin[2])) - f64::from(reference.location[2])) as f32];
            let local = transform_vector_by(moved, &inverse);
            let mut normal = transform_vector_by(safe_normal(distance), &inverse);
            if let Some(turn) = &turn {
                normal = transform_point_by(normal, turn);
            }
            let width = f64::from(emitter.width);
            let offset = f64::from(emitter.origin_offset);
            let remainder = f64::from((1.0 - offset) as f32);
            let x = f64::from(normal[0]) * width;
            let y = f64::from((f64::from(normal[1]) * width) as f32);
            let z = f64::from((f64::from(normal[2]) * width) as f32);
            sample.b = [
                (f64::from(local[0]) - x * offset) as f32,
                (f64::from(local[1]) - y * offset) as f32,
                (f64::from(local[2]) - f64::from((z * offset) as f32)) as f32,
            ];
            sample.a = [
                (x * remainder + f64::from(local[0])) as f32,
                (y * remainder + f64::from(local[1])) as f32,
                (f64::from((z * remainder) as f32) + f64::from(local[2])) as f32,
            ];
        }
        0 | 2 => sample.a = a,
        _ => sample.a = start.origin,
    }
    sample
}

pub const TRAILER_CLASS: &str = "LineageEffect.RibbonSet";
const TRAILER_END: f32 = 1.06;
const TRAILER_START: f32 = 0.53;
const TRAILER_THIRD: f32 = 0.333_333_34;
const TRAILER_HALF: f32 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trailer {
    pub start_offset: [f32; 3],
    pub end_offset: [f32; 3],
    pub width: f32,
    pub num_points: Option<i32>,
    pub threshold: f32,
    pub threshold2: f32,
    pub life: f32,
    pub field_e4: f32,
    pub field_fc: f32,
}

pub fn trailer(weapon_length: f32, mesh_scale: f32, shot_time: f32, points: i32, time: f32) -> Option<Trailer> {
    if weapon_length.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
        return None;
    }
    let length = f64::from(weapon_length) * f64::from(mesh_scale);
    let end = (f64::from(TRAILER_END) * length) as f32;
    Some(Trailer {
        start_offset: [(f64::from(TRAILER_START) * length) as f32, 0.0, 0.0],
        end_offset: [end, 0.0, 0.0],
        width: end,
        num_points: (points > 0).then_some(points),
        threshold: shot_time * TRAILER_HALF,
        threshold2: shot_time,
        life: time + shot_time,
        field_e4: shot_time,
        field_fc: shot_time * TRAILER_THIRD,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attach_trailer_104cb680_sizes_the_ribbon_from_the_weapon_length() {
        let t = trailer(40.0, 1.5, 0.9, 0, 0.8).unwrap();
        assert_eq!(t.start_offset, [(f64::from(0.53f32) * 60.0) as f32, 0.0, 0.0]);
        assert_eq!(t.end_offset, [(f64::from(1.06f32) * 60.0) as f32, 0.0, 0.0]);
        assert_eq!(t.width, t.end_offset[0]);
        assert_eq!(t.num_points, None);
        assert_eq!((t.threshold, t.threshold2), (0.45, 0.9));
        assert_eq!(t.life, 0.8 + 0.9);
        assert_eq!(t.field_fc, 0.9 * 0.333_333_34);
        assert_eq!(trailer(40.0, 1.0, 0.9, 12, 0.8).unwrap().num_points, Some(12));
        assert!(trailer(0.0, 1.0, 0.9, 0, 0.8).is_none());
        assert!(trailer(f32::NAN, 1.0, 0.9, 0, 0.8).is_none());
    }
}
