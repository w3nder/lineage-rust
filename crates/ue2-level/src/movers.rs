use crate::Placement;
use ue2_assets::{Property, find};
use ue2_package::{ObjectRef, Package};

#[derive(Debug, Clone, PartialEq)]
pub struct L2Movement {
    pub origin: Placement,
    pub initial_rotation: [i32; 3],
    pub max: [i32; 3],
    pub rate: [i32; 3],
    pub accel: [f32; 3],
    pub random_max: bool,
    pub random_start: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mover {
    pub base: Placement,
    pub key_pos: [[f32; 3]; 8],
    pub key_rot: [[i32; 3]; 8],
    pub meshes: [Option<String>; 7],
    pub server_id: Option<i32>,
    pub move_time: f32,
    pub glide: bool,
}

pub(crate) fn read_tags(package: &Package, properties: &[Property<'_>]) -> Vec<String> {
    let mut tags = Vec::new();
    for property in properties.iter().filter(|p| p.name.eq_ignore_ascii_case("L2MovementTag")) {
        if let Some(names) = property.names(package) {
            tags = names.into_iter().map(str::to_owned).collect();
        } else if let Some(name) = property.name_value(package) {
            let index = property.index as usize;
            if tags.len() <= index {
                tags.resize(index + 1, "None".to_owned());
            }
            tags[index] = name.to_owned();
        }
    }
    tags
}

pub(crate) fn read_movement(
    package: &Package,
    class: &str,
    properties: &[Property<'_>],
    origin: Placement,
) -> Option<L2Movement> {
    if !class.eq_ignore_ascii_case("MovableStaticMeshActor") {
        return None;
    }
    let get = |name| find(properties, name);
    let fields = get("L2AccelRatio").and_then(|p| p.fields(package)).unwrap_or_default();
    Some(L2Movement {
        origin,
        initial_rotation: get("L2OrgRotator").and_then(Property::rotator).unwrap_or_default(),
        max: get("L2RotatorMax").and_then(Property::rotator).unwrap_or_default(),
        rate: get("L2RotatorRate").and_then(Property::rotator).unwrap_or_default(),
        accel: ["PitchTime", "YawTime", "RollTime"]
            .map(|name| find(&fields, name).and_then(Property::float).unwrap_or(0.0)),
        random_max: get("bUseL2RotatorMaxRandom").and_then(Property::bool).unwrap_or(false),
        random_start: get("bUseL2RotatorRandomStart").and_then(Property::bool).unwrap_or(true),
    })
}

pub(crate) fn read_mover(
    package: &Package,
    class: &str,
    properties: &[Property<'_>],
    origin: Placement,
) -> Option<Mover> {
    if !class.eq_ignore_ascii_case("Mover") {
        return None;
    }
    let get = |name| find(properties, name);
    let indexed = |name: &str, index: usize| {
        properties.iter().find(|p| p.name.eq_ignore_ascii_case(name) && p.index as usize == index)
    };
    Some(Mover {
        base: Placement {
            location: get("BasePos").and_then(Property::vector).unwrap_or(origin.location),
            rotation: get("BaseRot").and_then(Property::rotator).unwrap_or(origin.rotation),
        },
        key_pos: std::array::from_fn(|i| indexed("KeyPos", i).and_then(Property::vector).unwrap_or_default()),
        key_rot: std::array::from_fn(|i| indexed("KeyRot", i).and_then(Property::rotator).unwrap_or_default()),
        meshes: std::array::from_fn(|i| {
            indexed("MeshGrade", i)
                .and_then(|p| p.object(package))
                .filter(|o| *o != ObjectRef::Null)
                .map(|o| package.object_path(o))
        }),
        server_id: get("L2ServerObjectRealID").and_then(Property::int),
        move_time: get("MoveTime").and_then(Property::float).unwrap_or(1.0),
        glide: get("MoverGlideType").and_then(Property::byte).unwrap_or(1) == 1,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct MovementState {
    pub current: [f32; 3],
    pub time: [f32; 3],
    pub cycle: [f32; 3],
    pub initialized: bool,
}

impl Default for MovementState {
    fn default() -> Self {
        Self { current: [-1.0; 3], time: [0.0; 3], cycle: [0.0; 3], initialized: false }
    }
}

impl MovementState {
    pub fn phys_l2_movement(&mut self, def: &L2Movement, dt: f32, mut random: impl FnMut() -> i32) -> Placement {
        let mut pose = def.origin;
        if !self.initialized {
            pose.rotation = def.initial_rotation;
        }
        for i in [0, 2, 1] {
            let max = def.max[i];
            let rate = def.rate[i];
            if !self.initialized {
                if max < 1 || rate < 1 {
                    continue;
                }
                let value = if def.random_max {
                    let a = random() % max;
                    (random() % max).wrapping_mul(a) % max
                } else {
                    max
                };
                self.current[i] = value as f32;
                self.cycle[i] = self.current[i] * 4.0 / rate as f32;
                self.time[i] = if def.random_start { (random() % 1000) as f32 * 0.001 } else { dt };
            } else if self.current[i] >= 0.0 {
                self.time[i] += dt;
                let ratio = max as f32 / self.current[i];
                let period = self.cycle[i] * 4.0 / (ratio * def.accel[i]);
                if self.time[i] < period || period.is_nan() {
                    let phase =
                        f64::from(self.time[i]) * f64::from(def.accel[i]) * f64::from(ratio) * std::f64::consts::PI
                            / f64::from(self.cycle[i]);
                    pose.rotation[i] =
                        (f64::from(def.origin.rotation[i]) + f64::from(self.current[i]) * phase.sin()) as i32;
                } else {
                    self.time[i] -= period;
                    self.current[i] = if def.random_max {
                        let a = ((random() % 100) as f32 * 0.005).clamp(0.2, 0.4);
                        let b = ((random() % 100) as f32 * 0.005).clamp(0.0, 0.5);
                        (a + b) * max as f32
                    } else {
                        max as f32
                    };
                    self.cycle[i] = self.current[i] * 4.0 / rate as f32;
                }
            }
        }
        self.initialized = true;
        pose
    }
}

impl Mover {
    pub fn key(&self, key: usize) -> Placement {
        let key = key.min(7);
        Placement {
            location: std::array::from_fn(|i| self.base.location[i] + self.key_pos[key][i]),
            rotation: std::array::from_fn(|i| self.base.rotation[i].wrapping_add(self.key_rot[key][i])),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MoverState {
    pub pose: Placement,
    pub old: Placement,
    pub alpha: f32,
    pub key: usize,
    pub previous: usize,
    pub moving: bool,
    pub open: bool,
    pub damage: i32,
    pub attackable: bool,
}

impl MoverState {
    pub fn new(def: &Mover) -> Self {
        Self {
            pose: def.key(0),
            old: def.key(0),
            alpha: 0.0,
            key: 0,
            previous: 0,
            moving: false,
            open: false,
            damage: 0,
            attackable: false,
        }
    }

    pub fn on_door_status_update(&mut self, def: &Mover, closed: bool, damage: i32, attackable: bool) -> bool {
        self.attackable = attackable;
        if closed == self.open {
            self.interpolate_to(def, usize::from(!closed));
        }
        self.open = !closed;
        if !(0..7).contains(&damage) {
            return false;
        }
        self.damage = damage;
        damage == 6 || def.meshes[damage as usize].is_some()
    }

    pub fn interpolate_to(&mut self, def: &Mover, key: usize) {
        if key == self.previous && self.key != self.previous {
            self.alpha = 1.0 - self.alpha;
            self.old = def.key(self.key);
        } else {
            self.old = self.pose;
            self.alpha = 0.0;
        }
        self.previous = self.key;
        self.key = key.min(7);
        self.moving = true;
    }

    pub fn phys_moving_brush(&mut self, def: &Mover, dt: f32) -> Placement {
        if self.moving && dt > 0.0 {
            self.alpha = (self.alpha + dt * (1.0 / def.move_time.max(0.005))).min(1.0);
            let a = self.alpha;
            let s = if def.glide { 3.0 * a * a - 2.0 * a * a * a } else { a };
            let target = def.key(self.key);
            self.pose = Placement {
                location: std::array::from_fn(|i| {
                    self.old.location[i] + (target.location[i] - self.old.location[i]) * s
                }),
                rotation: std::array::from_fn(|i| {
                    self.old.rotation[i]
                        .wrapping_add((target.rotation[i].wrapping_sub(self.old.rotation[i]) as f32 * s) as i32)
                }),
            };
            if self.alpha == 1.0 {
                self.moving = false;
                self.previous = self.key;
                self.alpha = 0.0;
            }
        }
        self.pose
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn door() -> Mover {
        let mut door = Mover {
            base: Placement { location: [0.0; 3], rotation: [0, -16384, 0] },
            key_pos: [[0.0; 3]; 8],
            key_rot: [[0; 3]; 8],
            meshes: std::array::from_fn(|_| None),
            server_id: Some(24180029),
            move_time: 1.0,
            glide: true,
        };
        door.key_rot[1] = [0, 19456, 0];
        door.key_pos[1] = [0.0, 0.0, -593.0];
        door
    }

    #[test]
    fn phys_moving_brush_10661e50_matches_mover33_and_finishes_at_the_key() {
        let def = door();
        let mut state = MoverState::new(&def);
        state.on_door_status_update(&def, false, 0, false);
        for yaw in [-13344, -6656, 32, 3072] {
            assert_eq!(state.phys_moving_brush(&def, 0.25).rotation[1], yaw);
        }
        assert!(!state.moving);
        assert_eq!(state.pose, def.key(1));
        state.on_door_status_update(&def, true, 0, false);
        assert_eq!(state.phys_moving_brush(&def, 2.0), def.key(0));
    }

    #[test]
    fn interpolate_to_reverses_without_resetting_remaining_progress() {
        let def = door();
        let mut state = MoverState::new(&def);
        state.on_door_status_update(&def, false, 0, false);
        state.phys_moving_brush(&def, 0.25);
        let pose = state.pose;
        state.on_door_status_update(&def, true, 0, false);
        assert_eq!(state.pose, pose);
        assert_eq!(state.alpha, 0.75);
        assert_eq!(state.phys_moving_brush(&def, 0.25), def.key(0));
    }

    #[test]
    fn on_door_status_update_1047fed0_preserves_mesh_on_missing_grades_except_six() {
        let mut def = door();
        def.meshes[3] = Some("Aden_castle_wallhole01d".into());
        let mut state = MoverState::new(&def);
        assert!(!state.on_door_status_update(&def, true, -1, true));
        assert!(state.attackable);
        assert_eq!(state.damage, 0);
        assert!(!state.on_door_status_update(&def, false, 2, false));
        assert!(state.moving);
        state.phys_moving_brush(&def, 0.25);
        assert!(state.on_door_status_update(&def, false, 3, false));
        assert_eq!(state.alpha, 0.25);
        assert!(state.on_door_status_update(&def, false, 6, false));
        assert!(!state.on_door_status_update(&def, false, 7, false));
        assert_eq!(state.damage, 6);
    }

    #[test]
    fn phys_l2_movement_10603bf0_uses_roll_acceleration_and_keeps_pitch_disabled() {
        let def = L2Movement {
            initial_rotation: [15, 300, 0],
            origin: Placement { location: [1.0, 2.0, 3.0], rotation: [15, 300, 0] },
            max: [524, 0, 1224],
            rate: [524, 0, 1224],
            accel: [0.0, 0.0, 1.0],
            random_max: false,
            random_start: false,
        };
        let mut state = MovementState::default();
        assert_eq!(state.phys_l2_movement(&def, 0.0, || 0), def.origin);
        for (dt, roll) in [(1.0, 865), (1.0, 1224), (2.0, 0), (12.0, 0)] {
            let pose = state.phys_l2_movement(&def, dt, || 0);
            assert!((pose.rotation[2] - roll).abs() <= 1);
            assert_eq!(pose.rotation[..2], def.origin.rotation[..2]);
            assert_eq!(pose.location, def.origin.location);
        }
    }

    #[test]
    fn phys_l2_movement_random_order_is_pitch_roll_yaw_and_cycle_uses_both_clamps() {
        let def = L2Movement {
            initial_rotation: [0; 3],
            origin: Placement { location: [0.0; 3], rotation: [0; 3] },
            max: [100; 3],
            rate: [100; 3],
            accel: [1.0; 3],
            random_max: true,
            random_start: true,
        };
        let mut state = MovementState::default();
        let mut values = [7, 8, 100, 9, 8, 200, 5, 6, 300].into_iter();
        state.phys_l2_movement(&def, 0.1, || values.next().unwrap());
        assert_eq!(state.current, [56.0, 30.0, 72.0]);
        assert!((state.time[0] - 0.1).abs() < 1e-6);
        assert!((state.time[1] - 0.3).abs() < 1e-6);
        assert!((state.time[2] - 0.2).abs() < 1e-6);
        state.phys_l2_movement(&def, 100.0, || 99);
        for value in state.current {
            assert!((value - 89.5).abs() < 1e-4);
        }
    }
}
