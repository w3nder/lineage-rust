use glam::{Mat4, Vec3};
use ue2_level::{BspTree, Level};

use super::lights::{PointLight, select};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Bounds {
    pub min: Vec3,
    pub max: Vec3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ambient {
    pub vector: [f32; 3],
    pub glow: u8,
}

impl Ambient {
    pub fn color(self, time: f32) -> [u8; 3] {
        ambient_color(Vec3::from(self.vector), self.glow, time)
    }
}

pub fn actor_glow(level: &Level, actor: &ue2_level::Actor) -> u8 {
    let mut current = actor;
    let mut visited = std::collections::HashSet::new();
    while current.use_owner_ambient && visited.insert(&current.name) {
        let Some(owner) = current.owner.as_deref().and_then(|name| level.actors.iter().find(|a| a.name == name)) else { break };
        current = owner;
    }
    current.ambient_glow
}

impl Bounds {
    pub fn transformed(values: [[f32; 3]; 2], world: Mat4) -> Self {
        let mut result = Self { min: Vec3::splat(f32::INFINITY), max: Vec3::splat(f32::NEG_INFINITY) };
        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    let p = world.transform_point3(Vec3::new(values[x][0], values[y][1], values[z][2]));
                    result.min = result.min.min(p);
                    result.max = result.max.max(p);
                }
            }
        }
        result
    }

    pub fn union(self, other: Self) -> Self {
        Self { min: self.min.min(other.min), max: self.max.max(other.max) }
    }

    pub fn center(self) -> Vec3 { (self.min + self.max) * 0.5 }
    pub fn extent(self) -> Vec3 { (self.max - self.min) * 0.5 }
}

#[derive(Clone, Copy, Debug)]
pub struct Query {
    pub position: Vec3,
    pub center: Vec3,
    pub radius: f32,
    pub bounds: Bounds,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sample {
    pub lights: Vec<PointLight>,
    pub ambient: [u8; 3],
    pub sun_affect: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LightMap {
    tree: BspTree,
    leaves: Vec<(usize, Vec<PointLight>)>,
    zones: Vec<(Vec3, bool)>,
}

pub fn ambient_color(vector: Vec3, glow: u8, time: f32) -> [u8; 3] {
    let glow = if glow == 255 {
        f64::from(time * 8.0).sin() * f64::from(0.2_f32) + 0.25
    } else {
        f64::from(glow) / 255.0
    };
    vector.to_array().map(|c| (((f64::from(c) + glow) as f32 * 255.0).floor()).clamp(0.0, 255.0) as u8)
}

pub fn box_leaves(tree: &BspTree, bounds: Bounds) -> Vec<usize> {
    let center = bounds.center();
    let extent = bounds.extent();
    let mut result = Vec::new();
    let mut stack = vec![0usize];
    let mut visited = std::collections::HashSet::new();
    while let Some(index) = stack.pop() {
        if !visited.insert(index) { continue; }
        let Some(node) = tree.nodes.get(index) else { continue };
        let normal = Vec3::new(node.plane[0], node.plane[1], node.plane[2]);
        let distance = normal.dot(center) - node.plane[3];
        let pushout = normal.abs().dot(extent);
        for (touches, child, leaf) in [(distance < pushout, node.back, node.leaf_back), (-distance < pushout, node.front, node.leaf_front)] {
            if !touches { continue; }
            if let Ok(child) = usize::try_from(child) { stack.push(child); }
            else if let Ok(leaf) = usize::try_from(leaf) { result.push(leaf); }
        }
    }
    result
}

impl LightMap {
    pub fn new(level: &Level, lights: &[PointLight]) -> Self {
        let leaves = level.bsp_render.leaves.iter().map(|leaf| {
            let names: std::collections::HashSet<_> = usize::try_from(leaf.permeating).ok().into_iter()
                .flat_map(|start| level.bsp_render.lights.iter().skip(start).take_while(|name| name.is_some()).flatten()).collect();
            let lights = lights.iter().filter(|light| level.lights.get(light.source).is_some_and(|source| names.contains(&source.name))).copied().collect();
            (usize::try_from(leaf.zone).unwrap_or(0), lights)
        }).collect();
        let zones = super::zone_lighting(level).iter().map(|zone| (Vec3::from(zone.ambient_vector.unwrap_or([0.0; 3])), zone.sun_affect)).collect();
        Self { tree: level.bsp_tree.clone(), leaves, zones }
    }

    pub fn sample(&self, query: Query, glow: u8, time: f32) -> Sample {
        let mut ambient = Vec3::ZERO;
        let mut sun_affect = false;
        let mut lights = Vec::new();
        for leaf in box_leaves(&self.tree, query.bounds) {
            let Some((zone, candidates)) = self.leaves.get(leaf) else { continue };
            if let Some((vector, sun)) = self.zones.get(*zone) {
                ambient = ambient.max(*vector);
                sun_affect |= sun;
            }
            for light in candidates {
                if !lights.contains(light) { lights.push(*light); }
            }
        }
        Sample { lights: select(&lights, query.center, query.radius, false, 4, false), ambient: ambient_color(ambient, glow, time), sun_affect }
    }

    pub fn zone_ambient(&self, bounds: Bounds) -> (Vec3, bool) {
        let mut ambient = Vec3::ZERO;
        let mut sun_affect = false;
        for leaf in box_leaves(&self.tree, bounds) {
            if let Some((zone, _)) = self.leaves.get(leaf)
                && let Some((vector, sun)) = self.zones.get(*zone)
            {
                ambient = ambient.max(*vector);
                sun_affect |= sun;
            }
        }
        (ambient, sun_affect)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ue2_level::BspNode;

    #[test]
    fn update_render_data_104ffae0_and_fcolor_10371120_preserve_glow_branches() {
        assert_eq!(ambient_color(Vec3::splat(0.27725613), 0, 0.0), [70; 3]);
        assert_eq!(ambient_color(Vec3::ZERO, 128, 0.0), [128; 3]);
        assert_eq!(ambient_color(Vec3::ZERO, 255, 0.0), [63; 3]);
        assert_eq!(ambient_color(Vec3::ZERO, 255, std::f32::consts::FRAC_PI_2 / 8.0), [114; 3]);
        assert_eq!(ambient_color(Vec3::ZERO, 255, 3.0 * std::f32::consts::FRAC_PI_2 / 8.0), [12; 3]);
        assert_eq!(ambient_color(Vec3::new(-2.0, 2.0, 0.5), 0, 0.0), [0, 255, 127]);
    }

    #[test]
    fn box_leaves_105b4d60_and_update_render_data_use_all_touched_zones() {
        let tree = BspTree { nodes: vec![BspNode { flags: 0, vertices: 0, coplanar: -1, collision_bound: -1, render_bound: -1, surface: -1, plane: [1.0, 0.0, 0.0, 0.0], front: -1, back: -1, zone_front: 1, zone_back: 2, leaf_front: 0, leaf_back: 1 }] };
        let map = LightMap { tree, leaves: vec![(0, Vec::new()), (1, Vec::new())], zones: vec![(Vec3::new(0.1, 0.4, 0.2), false), (Vec3::new(0.3, 0.2, 0.1), true)] };
        let query = Query { position: Vec3::ZERO, center: Vec3::ZERO, radius: 1.0, bounds: Bounds { min: -Vec3::ONE, max: Vec3::ONE } };
        let sample = map.sample(query, 0, 0.0);
        assert_eq!(sample.ambient, [76, 102, 51]);
        assert!(sample.sun_affect);
        let front = Bounds { min: Vec3::ZERO, max: Vec3::ONE };
        assert_eq!(box_leaves(&map.tree, front), [0]);
        assert!(box_leaves(&map.tree, Bounds::default()).is_empty());
    }
}
