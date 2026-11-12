use crate::pawn::{Appearance, Part};
use crate::scene::transform::vector_rotation;
use glam::Vec3;
use l2_dat::armorgrp::ItemDrop;

pub mod assets;
pub mod placement;
pub mod mover;
pub mod drop;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spin {
    pub rotation: [i32;3],
    pub target: [i32;3],
    pub rate: [i32;3],
}

impl Spin {
    pub fn tick(&mut self, delta: f32, mover_ended: bool) -> bool {
        self.rotation = self.rotation.map(|angle| angle & 0xffff);
        self.target = self.target.map(|angle| angle & 0xffff);
        if mover_ended && (self.rotation == self.target || self.rate == [0;3]) { return true; }
        for axis in 0..3 {
            let step = self.rate[axis] as f64 * delta as f64;
            if !mover_ended {
                self.rotation[axis] = self.rotation[axis].wrapping_add(step as i32);
                continue;
            }
            let distance = (self.target[axis] - self.rotation[axis]) & 0xffff;
            let distance = if self.rate[axis] < 0 { distance - 0x10000 } else { distance };
            let advance = if self.rate[axis] < 0 { step >= distance as f64 } else { step <= distance as f64 };
            self.rotation[axis] = if advance { (self.rotation[axis] as f64 + step) as i32 } else { self.target[axis] };
        }
        false
    }
}

pub fn appearance(drop: &ItemDrop) -> Appearance {
    let present = |name: &str| !name.is_empty() && !name.eq_ignore_ascii_case("None");
    let parts = drop.meshes.iter().enumerate().filter(|(_, mesh)| present(mesh)).map(|(index, mesh)| {
        let textures = drop.materials.iter().enumerate()
            .filter(|(slot, texture)| present(texture) && (*slot == index || !present(&drop.meshes[*slot])))
            .map(|(_, texture)| texture.clone()).collect();
        Part { slot: index as u8, ..Part::new(mesh, textures) }
    }).collect();
    Appearance { class_name: Some("Engine.L2Pickup".into()), parts, ..Default::default() }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shape {
    pub radius: f32,
    pub height: f32,
}

impl Shape {
    pub fn from_data(drop: &ItemDrop) -> Self {
        Self { radius: (drop.radius as i32) as f32, height: (drop.height as i32) as f32 }
    }

    pub fn mesh(&mut self, style: u32, floats_150: &[f32]) {
        if style != 0 {
            self.radius = 5.0;
            if !floats_150.is_empty() { self.height = floats_150.iter().sum(); }
        }
        self.height *= match style { 1 | 2 => 0.25, 3 | 4 => 0.5, _ => 1.0 };
        self.height = self.height.max(0.2);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TraceQuery {
    pub start: Vec3,
    pub end: Vec3,
    pub extent: Vec3,
    pub flags: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TraceHit {
    pub position: Vec3,
    pub normal: Vec3,
    pub time: f32,
    pub non_level_actor: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub position: Vec3,
    pub rotation: [i32; 3],
    pub shape: Shape,
}

pub fn settle(z: f32, time: f32) -> f32 {
    let rise = f64::from(time) * 12.0;
    if rise < f64::from(1.9_f32) { ((f64::from(2.4_f32) + f64::from(1.9_f32)) * 0.5 - rise + f64::from(z)) as f32 } else { z }
}

pub fn rotation(style: u32, index: usize) -> [i32; 3] {
    let turn = index as i32 * 0x140000 / 360;
    let pitch = match style { 1 => -0x4aaa - turn, 2 => -0x8000 - turn, 3 => 0x3555 - turn, 4 => -turn, _ => 0 };
    [pitch, 0, if style == 0 { 0 } else { 0x4000 }]
}

pub fn spawn_placement(
    position: Vec3,
    packet_rotation: [i32; 3],
    shape: Shape,
    style: u32,
    index: usize,
    mut trace: impl FnMut(TraceQuery) -> Option<TraceHit>,
    mut spawn: impl FnMut(Placement) -> bool,
) -> Option<Placement> {
    let style_rotation = rotation(style, index);
    for _ in 0..100 {
        let mut start = position + Vec3::Z * (shape.height + 20.0);
        let mut end = position;
        for _ in 0..5 {
            start.z += 20.0;
            end.z -= 50.0;
            let Some(hit) = trace(TraceQuery { start, end, extent: Vec3::new(shape.radius, shape.radius, shape.height), flags: 0x86 }) else { continue };
            if hit.position.z == 0.0 { continue; }
            let mut position = hit.position;
            if hit.non_level_actor { position.z += 2.0; }
            if matches!(style, 3 | 4) { position.z -= shape.height; }
            position.z = settle(position.z, hit.time);
            let floor = vector_rotation(Vec3::new(hit.normal.z, hit.normal.y, -hit.normal.x));
            let rotation = std::array::from_fn(|axis| floor[axis].wrapping_add(style_rotation[axis]).wrapping_add(packet_rotation[axis]));
            let placement = Placement { position, rotation, shape };
            if spawn(placement) { return Some(placement); }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_item_1047c050_retries_the_original_five_heights_and_stops_after_500() {
        let shape = Shape { radius: 5.0, height: 10.0 };
        let mut queries = Vec::new();
        let placed = spawn_placement(Vec3::new(1.0, 2.0, 100.0), [0; 3], shape, 0, 0, |query| { queries.push(query); None }, |_| panic!("sem hit"));
        assert!(placed.is_none());
        assert_eq!(queries.len(), 500);
        for block in queries.as_chunks::<5>().0 {
            for (step, query) in block.iter().enumerate() {
                assert_eq!(query.start, Vec3::new(1.0, 2.0, 150.0 + step as f32 * 20.0));
                assert_eq!(query.end, Vec3::new(1.0, 2.0, 50.0 - step as f32 * 50.0));
                assert_eq!(query.extent, Vec3::new(5.0, 5.0, 10.0));
                assert_eq!(query.flags, 0x86);
            }
        }
    }

    #[test]
    fn spawn_item_1047c050_keeps_all_style_branches_and_rejects_failed_spawn() {
        for style in 0..=5 {
            let mut shape = Shape { radius: 9.0, height: 10.0 };
            shape.mesh(style, &[20.0, 40.0]);
            assert_eq!(shape.height, [10.0, 15.0, 15.0, 30.0, 30.0, 60.0][style as usize]);
            assert_eq!(shape.radius, if style == 0 { 9.0 } else { 5.0 });
            let mut attempts = 0;
            let placed = spawn_placement(Vec3::new(0.0, 0.0, 100.0), [0, 16384, 0], shape, style, 1,
                |_| Some(TraceHit { position: Vec3::new(1.0, 2.0, 90.0), normal: Vec3::Z, time: 0.1, non_level_actor: true }),
                |_| { attempts += 1; attempts == 7 }).unwrap();
            assert_eq!(attempts, 7);
            let height = if matches!(style, 3 | 4) { shape.height } else { 0.0 };
            assert!((placed.position.z - (92.95 - height)).abs() < 0.00001);
            assert_eq!(placed.rotation[1], 16384);
            assert_eq!(placed.rotation[2], if style == 0 { 0 } else { 16384 });
        }
    }

    #[test]
    fn absent_mesh_texture_slots_feed_the_other_meshes_without_losing_order() {
        let drop = ItemDrop { meshes: ["a.Mesh".into(), "None".into(), "b.Mesh".into()], materials: ["a.Skin".into(), "a.Extra".into(), "b.Skin".into()], ..Default::default() };
        let appearance = appearance(&drop);
        assert_eq!(appearance.parts.len(), 2);
        assert_eq!(appearance.parts[0].textures, [Some("a.Skin".into()), Some("a.Extra".into())]);
        assert_eq!(appearance.parts[1].textures, [Some("a.Extra".into()), Some("b.Skin".into())]);
        assert_eq!((appearance.parts[0].slot, appearance.parts[1].slot), (0, 2));
    }
}
