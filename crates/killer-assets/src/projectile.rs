use crate::scene::transform::vector_rotation;
use glam::Vec3;

pub const PHYS_NONE: u8 = 0;
pub const PHYS_PROJECTILE: u8 = 17;
pub const HERMITE_MIN_DISTANCE: f32 = 550.0;
pub const HERMITE_DURATION: f32 = 0.4;
pub const QUARTER_PI: f32 = std::f32::consts::FRAC_PI_4;
const SAFE_NORMAL_LIMIT: f32 = 1e-8;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hermite {
    pub initial_velocity: Vec3,
    pub final_velocity: Vec3,
    pub initial_location: Vec3,
    pub duration: f32,
    pub elapsed: f32,
    pub displacement: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    pub normal: Vec3,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Projectile {
    pub location: Vec3,
    pub rotation: [i32; 3],
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub speed: f32,
    pub acc_speed: f32,
    pub target: Vec3,
    pub last_target_rotation: [i32; 3],
    pub directional: bool,
    pub hermite: Option<Hermite>,
    pub physics: u8,
}

pub fn safe_normal(v: Vec3) -> Vec3 {
    if v.length_squared() <= SAFE_NORMAL_LIMIT { Vec3::ZERO } else { v / v.length() }
}

pub fn hermite(p0: Vec3, p1: Vec3, v0: Vec3, v1: Vec3, t: f64) -> Vec3 {
    let (t2, t3) = (t * t, t * t * t);
    let weight = |a: f64, b: f64, c: f64, d: f64| (a * t3 + b * t2 + c * t + d) as f32;
    p0 * weight(2.0, -3.0, 0.0, 1.0) + v0 * weight(1.0, -2.0, 1.0, 0.0) + p1 * weight(-2.0, 3.0, 0.0, 0.0) + v1 * weight(1.0, -1.0, 0.0, 0.0)
}

impl Projectile {
    pub fn new(location: Vec3, rotation: [i32; 3], acc_speed: f32, directional: bool) -> Self {
        Self {
            location,
            rotation,
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            speed: 0.0,
            acc_speed,
            target: location,
            last_target_rotation: [0; 3],
            directional,
            hermite: None,
            physics: PHYS_NONE,
        }
    }

    pub fn launch(&mut self, target: Vec3) {
        self.physics = PHYS_PROJECTILE;
        self.target = target;
        if self.directional && self.hermite.is_none() {
            let aim = vector_rotation(target - self.location);
            self.last_target_rotation = aim;
            self.rotation = aim;
        }
    }

    pub fn prepare_interpolation(&mut self, curvature: i32, delta: Vec3) -> f32 {
        let distance = delta.length();
        if distance <= HERMITE_MIN_DISTANCE {
            return 0.0;
        }
        let quarter = f64::from(QUARTER_PI);
        let count = f64::from(curvature);
        let x = (count * quarter) / ((f64::from(distance) - f64::from(HERMITE_MIN_DISTANCE)) * quarter + count);
        let theta = quarter - x;
        let magnitude = (f64::from(distance) * theta.tan()) as f32;
        self.hermite = Some(Hermite {
            initial_velocity: Vec3::new(delta.x, delta.y, delta.z + magnitude),
            final_velocity: Vec3::new(delta.x, delta.y, delta.z - magnitude),
            initial_location: self.location,
            duration: HERMITE_DURATION,
            elapsed: 0.0,
            displacement: distance,
        });
        theta as f32
    }

    pub fn tick(&mut self, dt: f32, target: Vec3) -> Option<Hit> {
        if self.physics != PHYS_PROJECTILE {
            return None;
        }
        self.target = target;
        let direction = safe_normal(target - self.location);
        if self.hermite.is_none() {
            if self.directional {
                let aim = vector_rotation(direction);
                for ((current, new), last) in self.rotation.iter_mut().zip(aim).zip(self.last_target_rotation) {
                    *current = current.wrapping_add(new.wrapping_sub(last));
                }
                self.last_target_rotation = aim;
            } else {
                self.rotation = vector_rotation(target - self.location);
            }
        }
        self.speed = self.velocity.length();
        self.velocity = direction * self.speed;
        self.acceleration = direction * self.acc_speed;
        let old = self.location;
        self.velocity += self.acceleration * dt;
        if self.acceleration != Vec3::ZERO && self.velocity.length_squared() > self.acceleration.length_squared() {
            self.velocity = safe_normal(self.velocity) * self.acceleration.length();
        }
        let mut delta = self.velocity * dt;
        if let Some(curve) = self.hermite.as_mut().filter(|curve| curve.duration > 0.0) {
            curve.elapsed += dt;
            let length = (target - curve.initial_location).length();
            let scale = length / curve.displacement;
            curve.initial_velocity *= scale;
            curve.final_velocity *= scale;
            curve.displacement = length;
            let point = hermite(
                curve.initial_location,
                target,
                curve.initial_velocity,
                curve.final_velocity,
                f64::from(curve.elapsed / curve.duration),
            );
            delta = point - self.location;
            self.rotation = vector_rotation(delta);
        }
        self.location += delta;
        let travelled = (self.location - old).length();
        let remaining = (old - target).length();
        let mut hit = None;
        if remaining <= travelled {
            let normal = -safe_normal(target - self.location);
            self.location = target;
            self.physics = PHYS_NONE;
            hit = Some(Hit { normal });
        }
        if dt > 0.0 {
            self.velocity = (self.location - old) / dt;
        }
        hit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: f32 = 1.0 / 60.0;

    fn fly(projectile: &mut Projectile, target: Vec3, limit: usize) -> Option<(usize, Hit)> {
        (1..=limit).find_map(|frame| projectile.tick(FRAME, target).map(|hit| (frame, hit)))
    }

    #[test]
    fn a_projectile_does_nothing_until_it_is_launched() {
        let mut projectile = Projectile::new(Vec3::ZERO, [0; 3], 3000.0, false);
        assert_eq!(projectile.tick(FRAME, Vec3::new(100.0, 0.0, 0.0)), None);
        assert_eq!(projectile.location, Vec3::ZERO);
    }

    #[test]
    fn the_first_frame_starts_from_rest_and_only_the_acceleration_moves_it() {
        let mut projectile = Projectile::new(Vec3::ZERO, [0; 3], 3000.0, false);
        let target = Vec3::new(1000.0, 0.0, 0.0);
        projectile.launch(target);
        assert_eq!(projectile.tick(FRAME, target), None);
        assert_eq!(projectile.speed, 0.0);
        assert!((projectile.location.x - 3000.0 * FRAME * FRAME).abs() < 1e-3, "{}", projectile.location.x);
    }

    #[test]
    fn it_reaches_the_target_when_the_step_covers_the_remaining_distance() {
        let mut projectile = Projectile::new(Vec3::ZERO, [0; 3], 3000.0, false);
        let target = Vec3::new(1000.0, 0.0, 0.0);
        projectile.launch(target);
        let (frame, hit) = fly(&mut projectile, target, 600).expect("chegou");
        assert!((49..=52).contains(&frame), "{frame}");
        assert_eq!(projectile.location, target);
        assert_eq!(projectile.physics, PHYS_NONE);
        assert!((hit.normal - Vec3::new(1.0, 0.0, 0.0)).length() < 1e-4, "{:?}", hit.normal);
    }

    #[test]
    fn the_velocity_is_capped_at_the_acceleration_magnitude() {
        let mut projectile = Projectile::new(Vec3::ZERO, [0; 3], 10.0, false);
        let target = Vec3::new(1.0e6, 0.0, 0.0);
        projectile.launch(target);
        for _ in 0..50 {
            projectile.tick(1.0, target);
        }
        assert!(projectile.velocity.length() <= 10.0 + 1e-3, "{}", projectile.velocity.length());
    }

    #[test]
    fn the_flight_homes_on_a_target_that_moves() {
        let mut projectile = Projectile::new(Vec3::ZERO, [0; 3], 3000.0, false);
        let mut target = Vec3::new(1000.0, 0.0, 0.0);
        projectile.launch(target);
        for _ in 0..20 {
            projectile.tick(FRAME, target);
        }
        target = Vec3::new(500.0, 800.0, 0.0);
        projectile.tick(FRAME, target);
        let toward = safe_normal(target - projectile.location);
        assert!(projectile.velocity.normalize().dot(toward) > 0.999);
    }

    #[test]
    fn a_non_directional_projectile_always_faces_the_target() {
        let mut projectile = Projectile::new(Vec3::ZERO, [0; 3], 3000.0, false);
        let target = Vec3::new(0.0, 500.0, 0.0);
        projectile.launch(target);
        projectile.tick(FRAME, target);
        assert_eq!(projectile.rotation, vector_rotation(target - projectile.location));
    }

    #[test]
    fn a_directional_projectile_is_pre_aimed_and_adds_the_change_of_direction() {
        let mut projectile = Projectile::new(Vec3::ZERO, [7, 11, 13], 3000.0, true);
        let target = Vec3::new(0.0, 500.0, 0.0);
        projectile.launch(target);
        assert_eq!(projectile.rotation, vector_rotation(target));
        assert_eq!(projectile.last_target_rotation, vector_rotation(target));
        let before = projectile.rotation;
        let turned = Vec3::new(500.0, 0.0, 0.0);
        projectile.tick(FRAME, turned);
        let aim = vector_rotation(turned - Vec3::ZERO);
        assert_eq!(projectile.last_target_rotation, aim);
        for axis in 0..3 {
            assert_eq!(projectile.rotation[axis], before[axis] + aim[axis] - vector_rotation(target)[axis]);
        }
    }

    #[test]
    fn short_shots_are_not_interpolated_and_long_ones_follow_the_engine_formula() {
        let mut projectile = Projectile::new(Vec3::ZERO, [0; 3], 3000.0, false);
        assert_eq!(projectile.prepare_interpolation(1000, Vec3::new(550.0, 0.0, 0.0)), 0.0);
        assert!(projectile.hermite.is_none());
        let theta = projectile.prepare_interpolation(1000, Vec3::new(1000.0, 0.0, 0.0));
        let quarter = f64::from(QUARTER_PI);
        let x = (1000.0 * quarter) / (450.0 * quarter + 1000.0);
        assert!((f64::from(theta) - (quarter - x)).abs() < 1e-6);
        let curve = projectile.hermite.unwrap();
        let magnitude = (1000.0 * (quarter - x).tan()) as f32;
        assert_eq!(curve.initial_velocity, Vec3::new(1000.0, 0.0, magnitude));
        assert_eq!(curve.final_velocity, Vec3::new(1000.0, 0.0, -magnitude));
        assert_eq!((curve.duration, curve.displacement), (0.4, 1000.0));
    }

    #[test]
    fn the_hermite_curve_starts_and_ends_on_its_points() {
        let (p0, p1) = (Vec3::new(1.0, 2.0, 3.0), Vec3::new(40.0, -5.0, 9.0));
        let (v0, v1) = (Vec3::new(10.0, 0.0, 20.0), Vec3::new(10.0, 0.0, -20.0));
        assert!((hermite(p0, p1, v0, v1, 0.0) - p0).length() < 1e-5);
        assert!((hermite(p0, p1, v0, v1, 1.0) - p1).length() < 1e-5);
    }

    #[test]
    fn an_interpolated_arrow_arcs_above_the_straight_line_and_still_arrives() {
        let mut projectile = Projectile::new(Vec3::ZERO, [0; 3], 3000.0, true);
        let target = Vec3::new(1200.0, 0.0, 0.0);
        projectile.prepare_interpolation(1000, target);
        projectile.launch(target);
        let mut peak = 0.0f32;
        let mut arrived = false;
        for _ in 0..240 {
            let hit = projectile.tick(FRAME, target);
            peak = peak.max(projectile.location.z);
            if hit.is_some() {
                arrived = true;
                break;
            }
        }
        assert!(arrived);
        assert!(peak > 10.0, "{peak}");
        assert_eq!(projectile.location, target);
    }
}
