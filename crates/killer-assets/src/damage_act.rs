use glam::{Mat3, Quat, Vec3};

use crate::engine_math::{cos_tab, sin_tab};

pub const SPINE_GATE_BONES: [&str; 2] = ["Bip01 Spine2", "Bip01_Spine2"];
pub const DESCENT_RATE: f32 = f32::from_bits(0x47a6_6666);
pub const FLOOR: f32 = f32::from_bits(0xc599_999a);
pub const RETURN_RATE: f32 = f32::from_bits(0x46cc_cccd);
pub const REST: f32 = f32::from_bits(0x3727_c5ac);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PawnDefaults {
    pub spine_rotation: bool,
    pub spine_bone: Option<String>,
    pub head_bone: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DamageBones {
    pub spine_rotation: bool,
    pub gate: bool,
    pub bones: [Option<usize>; 2],
}

impl DamageBones {
    pub fn resolve(defaults: &PawnDefaults, find: impl Fn(&str) -> Option<usize>) -> Self {
        let gate = find(SPINE_GATE_BONES[0]).or_else(|| find(SPINE_GATE_BONES[1])).is_some();
        let bone = |name: &Option<String>| name.as_deref().and_then(&find);
        Self { spine_rotation: defaults.spine_rotation, gate, bones: [bone(&defaults.spine_bone), bone(&defaults.head_bone)] }
    }

    pub fn ticks(&self, keep_min_frame_rate: bool) -> bool {
        self.spine_rotation && self.gate && !keep_min_frame_rate
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DamageAct {
    pub active: bool,
    pub status: i32,
    pub dist: f32,
}

impl DamageAct {
    pub fn attacked(&mut self) {
        self.active = true;
        self.status = 0;
    }

    pub fn spine_rotation(&mut self, dt: f32) {
        if !self.active {
            return;
        }
        match self.status {
            0 | 1 => {
                let value = f64::from(self.dist) - f64::from(dt) * f64::from(DESCENT_RATE);
                self.dist = value as f32;
                if value < f64::from(FLOOR) {
                    self.dist = FLOOR;
                    self.status = -1;
                } else if self.status == 0 {
                    self.status = 1;
                }
            }
            _ => {
                let value = f64::from(dt) * f64::from(RETURN_RATE) + f64::from(self.dist);
                self.dist = value as f32;
                if value > f64::from(REST) {
                    *self = DamageAct::default();
                }
            }
        }
    }

    pub fn yaw(&self) -> i32 {
        (-self.dist) as i32
    }
}

pub fn rotate_local(local: &mut [(Quat, Vec3)], bone: usize, yaw: i32) {
    let Some((rotation, _)) = local.get_mut(bone) else { return };
    let (sin, cos) = (sin_tab(yaw), cos_tab(yaw));
    let turn = Mat3::from_cols(Vec3::new(cos, -sin, 0.0), Vec3::new(sin, cos, 0.0), Vec3::Z);
    *rotation = Quat::from_mat3(&(Mat3::from_quat(*rotation) * turn)).normalize();
}

pub fn apply(local: &mut [(Quat, Vec3)], bones: &DamageBones, yaw: i32) {
    if !bones.spine_rotation {
        return;
    }
    for bone in bones.bones.into_iter().flatten() {
        rotate_local(local, bone, yaw);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine_math::{Coords, coords_div_rotator};

    #[test]
    fn spine_rotation_105de680_dips_to_the_floor_and_returns_to_rest() {
        let mut act = DamageAct::default();
        act.spine_rotation(0.1);
        assert_eq!(act, DamageAct::default());
        act.attacked();
        act.spine_rotation(0.01);
        assert_eq!((act.status, act.dist), (1, (-(f64::from(0.01f32) * f64::from(DESCENT_RATE))) as f32));
        assert_eq!(act.yaw(), 851);
        act.spine_rotation(0.05);
        assert_eq!((act.status, act.dist), (-1, FLOOR));
        assert_eq!(act.yaw(), 4915);
        act.spine_rotation(0.1);
        assert!(act.active && act.status == -1 && act.dist < 0.0);
        act.spine_rotation(0.1);
        assert_eq!(act, DamageAct::default());
    }

    #[test]
    fn a_new_hit_during_the_return_restarts_the_dip_from_the_current_distance() {
        let mut act = DamageAct { active: true, status: -1, dist: -2000.0 };
        act.attacked();
        act.spine_rotation(0.0);
        assert_eq!((act.status, act.dist), (1, -2000.0));
        let mut deep = DamageAct { active: true, status: 0, dist: -4000.0 };
        deep.spine_rotation(0.02);
        assert_eq!((deep.status, deep.dist), (-1, FLOOR));
    }

    #[test]
    fn rotate_local_equals_fcoords_division_by_the_yaw_on_the_rows_of_the_bone_rotation() {
        let rotation = Quat::from_euler(glam::EulerRot::XYZ, 0.3, -0.7, 1.1);
        for yaw in [0, 851, 4915, 16384, -3000] {
            let mut local = vec![(Quat::IDENTITY, Vec3::ZERO), (rotation, Vec3::new(1.0, 2.0, 3.0))];
            rotate_local(&mut local, 1, yaw);
            let before = Mat3::from_quat(rotation);
            let coords = Coords { origin: [0.0; 3], x: before.row(0).to_array(), y: before.row(1).to_array(), z: before.row(2).to_array() };
            let divided = coords_div_rotator(&coords, [0, yaw, 0]);
            let after = Mat3::from_quat(local[1].0);
            for (row, axis) in [divided.x, divided.y, divided.z].into_iter().enumerate() {
                assert!((after.row(row) - Vec3::from(axis)).length() < 1e-5, "yaw {yaw}, linha {row}");
            }
            assert_eq!(local[1].1, Vec3::new(1.0, 2.0, 3.0));
            assert_eq!(local[0].0, Quat::IDENTITY);
        }
    }

    #[test]
    fn damage_bones_need_bspinerotation_and_resolve_spine_and_head_by_name() {
        let names = ["Bip01", "Bip01_Spine2", "Bip01_head"];
        let find = |name: &str| names.iter().position(|bone| bone.eq_ignore_ascii_case(name));
        let defaults = PawnDefaults { spine_rotation: true, spine_bone: Some("bip01_spine2".into()), head_bone: Some("Bip01_head".into()) };
        let bones = DamageBones::resolve(&defaults, find);
        assert_eq!(bones, DamageBones { spine_rotation: true, gate: true, bones: [Some(1), Some(2)] });
        assert!(bones.ticks(false) && !bones.ticks(true));
        let mut local = vec![(Quat::IDENTITY, Vec3::ZERO); 3];
        apply(&mut local, &bones, 16384);
        assert_eq!(local[0].0, Quat::IDENTITY);
        assert!((Mat3::from_quat(local[2].0).row(0) - Vec3::Y).length() < 1e-6);
        let off = DamageBones::resolve(&PawnDefaults { spine_rotation: false, ..defaults.clone() }, find);
        let mut untouched = vec![(Quat::IDENTITY, Vec3::ZERO); 3];
        apply(&mut untouched, &off, 16384);
        assert!(untouched.iter().all(|(rotation, _)| *rotation == Quat::IDENTITY) && !off.ticks(false));
        let headless = DamageBones::resolve(&PawnDefaults { spine_bone: None, head_bone: Some("None".into()), ..defaults }, |name: &str| ["Bip01 Spine2"].iter().position(|bone| *bone == name));
        assert_eq!(headless, DamageBones { spine_rotation: true, gate: true, bones: [None, None] });
    }
}
