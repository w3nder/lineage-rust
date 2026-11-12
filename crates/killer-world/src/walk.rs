use glam::Vec3;

pub const WORLD_FLAGS: u32 = 0x86;
pub const FLOOR_FLAGS: u32 = 0xdf;
pub const MAX_STEP_HEIGHT: f32 = 10.0;
pub const FLOOR_PROBE: f32 = 12.0;
pub const WALKABLE_Z: f32 = 0.7;
pub const MIN_FLOOR_DISTANCE: f32 = 1.9;
pub const MAX_FLOOR_DISTANCE: f32 = 2.4;
pub const MOVE_MARGIN: f32 = 2.0;
pub const STEP_RETRY_SQUARED: f32 = 144.0;
pub const MAX_ITERATIONS: u32 = 8;
pub const PRECISE_STEP: f32 = 0.1;
pub const PRECISE_THRESHOLD: f32 = 0.05;
pub const NEARLY_ZERO: f32 = 0.0001;
pub const LEDGE_FLOOR_DROP: f32 = 4.0;
pub const LEDGE_SLOPE_CAP: f32 = 10.0;
pub const LEDGE_PROBE_DEPTH: f32 = 14.0;
pub const LEDGE_FLAGS: u32 = 0x2df;
pub const WALKABLE_Z_F64: f64 = 0.7;
pub const PUSH_FACING_MAX: f64 = -0.9;
pub const STOP_SLIDE_MIN: f32 = 0.1;
pub const HIT_WALL_DROP: f32 = 10.0;
pub const PRECISE_FRACTION: f32 = 0.5;
pub const CONTROLLER_MIN_HIT_WALL: f32 = -1.0;
pub const AI_CONTROLLER_MIN_HIT_WALL: f32 = -0.5;
pub const LINEAGE_PAWN_CAN_IGNORE_COLLISION: bool = true;
pub const LINEAGE_PAWN_CAN_BE_IGNORED_COLLISION: bool = true;

pub fn pawn_blocks_mover(mover_can_ignore_collision: bool, target_can_be_ignored_collision: bool) -> bool {
    !(mover_can_ignore_collision && target_can_be_ignored_collision)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HitActor {
    None,
    Pawn,
    Other { immediately_stop: bool, pushable_mass: Option<f32> },
}

impl HitActor {
    pub const WORLD: Self = Self::Other { immediately_stop: false, pushable_mass: None };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SweepHit {
    pub time: f32,
    pub normal: Vec3,
    pub actor: HitActor,
}

impl SweepHit {
    pub const NONE: Self = Self { time: 1.0, normal: Vec3::ZERO, actor: HitActor::None };

    pub const fn world(time: f32, normal: Vec3) -> Self {
        Self { time, normal, actor: HitActor::WORLD }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ControllerCall {
    CannotMoveAnymore { location: Vec3, from_hit_wall: bool },
    Stop { location: Vec3 },
    NotifyJumpApex,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Physics {
    Walking,
    Swimming,
}

pub trait Sweep {
    fn sweep(&mut self, start: Vec3, end: Vec3, extent: Vec3, flags: u32) -> Option<SweepHit>;

    fn water_surface_crossed(&mut self, _from: Vec3, _to: Vec3) -> bool {
        false
    }

    fn controller_call(&mut self, _call: ControllerCall) {}

    fn line_check(&mut self, start: Vec3, end: Vec3, extent: Vec3, flags: u32) -> Option<(SweepHit, Vec3)> {
        self.sweep(start, end, extent, flags).map(|hit| (hit, start + (end - start) * hit.time))
    }

    fn encroaching(&mut self, _location: Vec3, _extent: Vec3) -> Option<bool> {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Volume {
    pub gravity: Vec3,
    pub zone_velocity: Vec3,
    pub terminal_velocity: f32,
    pub ground_friction: f32,
    pub fluid_friction: f32,
    pub water: bool,
}

impl Volume {
    pub const DEFAULT_GRAVITY: f32 = -1500.0;
    pub const DEFAULT_TERMINAL_VELOCITY: f32 = 2500.0;
    pub const DEFAULT_GROUND_FRICTION: f32 = 8.0;
    pub const DEFAULT_FLUID_FRICTION: f32 = 0.3;

    pub fn level_default() -> Self {
        Self {
            gravity: Vec3::new(0.0, 0.0, Self::DEFAULT_GRAVITY),
            zone_velocity: Vec3::ZERO,
            terminal_velocity: Self::DEFAULT_TERMINAL_VELOCITY,
            ground_friction: Self::DEFAULT_GROUND_FRICTION,
            fluid_friction: Self::DEFAULT_FLUID_FRICTION,
            water: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Controller {
    Player,
    Ai,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Walker {
    pub location: Vec3,
    pub velocity: Vec3,
    pub extent: Vec3,
    pub gravity_up: bool,
    pub can_jump: bool,
    pub is_walking: bool,
    pub is_crouched: bool,
    pub can_walk_off_ledges: bool,
    pub avoid_ledges: bool,
    pub can_fly: bool,
    pub stop_at_ledges: bool,
    pub direct_hit_wall: bool,
    pub no_velocity_update: bool,
    pub my_controller: bool,
    pub moving: bool,
    pub min_hit_wall: f32,
    pub on_world: bool,
    pub resting: bool,
    pub floor_normal: Vec3,
    pub acceleration: Vec3,
    pub constant_acceleration: Vec3,
    pub air_control: f32,
    pub accel_rate: f32,
    pub ground_speed: f32,
    pub mass: f32,
    pub buoyancy: f32,
    pub volume: Volume,
    pub level_gravity_z: f32,
    pub head_in_water: bool,
    pub just_teleported: bool,
    pub controller_present: bool,
    pub controller: Controller,
    pub move_speed: f32,
    pub destination: Option<Vec3>,
    pub human: bool,
    pub on_water_floor: bool,
    pub notify_apex: bool,
    pub jump_over_wall: bool,
    pub water_entry_blocked: bool,
}

impl Walker {
    pub fn new(location: Vec3, radius: f32, height: f32) -> Self {
        Self {
            location,
            velocity: Vec3::ZERO,
            extent: Vec3::new(radius, radius, height),
            gravity_up: false,
            can_jump: true,
            is_walking: false,
            is_crouched: false,
            can_walk_off_ledges: false,
            avoid_ledges: false,
            can_fly: false,
            stop_at_ledges: false,
            direct_hit_wall: false,
            no_velocity_update: false,
            my_controller: false,
            moving: false,
            min_hit_wall: CONTROLLER_MIN_HIT_WALL,
            on_world: false,
            resting: false,
            floor_normal: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            constant_acceleration: Vec3::ZERO,
            air_control: 0.05,
            accel_rate: 2048.0,
            ground_speed: 600.0,
            mass: 100.0,
            buoyancy: 0.0,
            volume: Volume::level_default(),
            level_gravity_z: Volume::DEFAULT_GRAVITY,
            head_in_water: false,
            just_teleported: false,
            controller_present: true,
            controller: Controller::Player,
            move_speed: 0.0,
            destination: None,
            human: false,
            on_water_floor: false,
            notify_apex: false,
            jump_over_wall: false,
            water_entry_blocked: false,
        }
    }

    pub fn walking_or_crouched(&self) -> bool {
        self.is_walking || self.is_crouched
    }

    pub fn wants_ledge_check(&self) -> bool {
        match self.controller {
            Controller::Player => self.walking_or_crouched(),
            Controller::Ai => !self.can_fly,
        }
    }

    pub fn controller_refuses_fall(&self) -> bool {
        match self.controller {
            Controller::Player => false,
            Controller::Ai => !self.can_jump || self.stop_at_ledges,
        }
    }

    fn hit_wall_notify<S: Sweep>(&self, sweep: &mut S) -> bool {
        if self.controller == Controller::Player && self.my_controller && self.moving {
            sweep.controller_call(ControllerCall::CannotMoveAnymore { location: self.location, from_hit_wall: true });
        }
        true
    }

    fn gravity(&self) -> Vec3 {
        Vec3::new(0.0, 0.0, if self.gravity_up { 1.0 } else { -1.0 })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Outcome {
    Walking,
    Falling { remaining: f32 },
    Landed { remaining: f32, iterations: u32 },
    Swimming { remaining: f32 },
    Surfaced { remaining: f32 },
    Blocked,
    Halted,
}

pub fn nearly_zero(v: Vec3) -> bool {
    v.x.abs() < NEARLY_ZERO && v.y.abs() < NEARLY_ZERO && v.z.abs() < NEARLY_ZERO
}

pub fn safe_normal(v: Vec3) -> Vec3 {
    let squared = v.length_squared();
    if f64::from(squared) <= 1e-8 { Vec3::ZERO } else { v * (1.0 / f64::from(squared).sqrt()) as f32 }
}

pub fn move_actor<S: Sweep>(sweep: &mut S, location: &mut Vec3, extent: Vec3, delta: Vec3) -> SweepHit {
    let length = if nearly_zero(delta) { 0.0 } else { delta.length() };
    let direction = if length == 0.0 { delta } else { delta / length };
    let reach = delta + direction * MOVE_MARGIN;
    let mut hit = SweepHit::NONE;
    let mut applied = delta;
    if length != 0.0 {
        hit = sweep.sweep(*location, *location + reach, extent, WORLD_FLAGS).unwrap_or(SweepHit::NONE);
    }
    if hit.time < 1.0 {
        let travelled = (MOVE_MARGIN + length) * hit.time;
        if travelled > MOVE_MARGIN {
            applied = reach * hit.time - direction * MOVE_MARGIN;
            hit.time = (travelled - MOVE_MARGIN) / length;
        } else {
            applied = Vec3::ZERO;
            hit.time = 0.0;
        }
    }
    *location += applied;
    hit
}

pub fn move_actor_into<S: Sweep>(sweep: &mut S, location: &mut Vec3, extent: Vec3, delta: Vec3, hit: &mut SweepHit) {
    if delta.x == 0.0 && delta.y == 0.0 && delta.z == 0.0 {
        return;
    }
    *hit = move_actor(sweep, location, extent, delta);
}

pub fn two_wall_adjust(desired: Vec3, delta: &mut Vec3, hit_normal: Vec3, old_normal: Vec3, time: f32) {
    if hit_normal.dot(old_normal) > 0.0 {
        let along = delta.dot(hit_normal);
        *delta = (*delta - hit_normal * along) * (1.0 - time);
        if desired.dot(*delta) <= 0.0 {
            *delta = Vec3::ZERO;
        }
    } else {
        let cross = safe_normal(hit_normal.cross(old_normal));
        let amount = (1.0 - time) * cross.dot(*delta);
        *delta = cross * amount;
        if desired.dot(*delta) < 0.0 {
            *delta = -*delta;
        }
    }
}

fn controller_stop<S: Sweep>(sweep: &mut S, walker: &mut Walker) {
    walker.acceleration = Vec3::ZERO;
    walker.moving = false;
    walker.is_walking = false;
    sweep.controller_call(ControllerCall::Stop { location: walker.location });
}

fn extended_dot(a: Vec3, b: Vec3) -> f64 {
    f64::from(a.y) * f64::from(b.y) + f64::from(a.z) * f64::from(b.z) + f64::from(a.x) * f64::from(b.x)
}

pub fn process_hit_wall<S: Sweep>(sweep: &mut S, walker: &mut Walker, physics: Physics, normal: Vec3, actor: HitActor) {
    if matches!(actor, HitActor::None | HitActor::Pawn) {
        return;
    }
    if walker.direct_hit_wall || !walker.controller_present {
        return;
    }
    if walker.acceleration == Vec3::ZERO {
        return;
    }
    let mut normal = normal;
    let mut dir = safe_normal(walker.destination.unwrap_or(walker.location) - walker.location);
    if physics == Physics::Walking {
        normal.z = 0.0;
        dir.z = 0.0;
        normal = safe_normal(normal);
        dir = safe_normal(dir);
    }
    let facing = f64::from(normal.y) * f64::from(dir.y) + f64::from(normal.x) * f64::from(dir.x) + f64::from(dir.z) * f64::from(normal.z);
    if facing > f64::from(walker.min_hit_wall) {
        return;
    }
    let extent = walker.extent;
    move_actor(sweep, &mut walker.location, extent, Vec3::new(0.0, 0.0, -HIT_WALL_DROP));
}

pub fn step_up<S: Sweep>(sweep: &mut S, walker: &mut Walker, physics: Physics, grav_dir: Vec3, desired: Vec3, delta: Vec3, hit: &mut SweepHit) {
    let down = grav_dir * MAX_STEP_HEIGHT;
    let extent = walker.extent;
    let start = walker.location;
    let mut lifted = false;
    let mut back = Vec3::ZERO;
    if hit.normal.z.abs() < WALKABLE_Z || WALKABLE_Z_F64 <= f64::from(hit.normal.z) {
        move_actor(sweep, &mut walker.location, extent, down * -1.0);
        move_actor_into(sweep, &mut walker.location, extent, delta, hit);
        back = start - walker.location;
        lifted = true;
    } else if physics != Physics::Walking {
        let magnitude = delta.length();
        let slope = Vec3::new(delta.x, delta.y, magnitude * hit.normal.z + delta.z);
        move_actor_into(sweep, &mut walker.location, extent, slope, hit);
    }
    if hit.time < 1.0 {
        if lifted && !walker.hit_wall_notify(sweep) {
            move_actor(sweep, &mut walker.location, extent, back);
            move_actor(sweep, &mut walker.location, extent, down);
            controller_stop(sweep, walker);
            return;
        }
        let pushable = match hit.actor {
            HitActor::Other { pushable_mass: Some(mass), .. } if walker.human => Some(mass),
            _ => None,
        };
        match pushable {
            Some(mass) if extended_dot(desired, hit.normal) < PUSH_FACING_MAX => {
                walker.no_velocity_update = true;
                let ratio = f64::from(walker.mass) / (f64::from(mass) + f64::from(walker.mass));
                walker.velocity = Vec3::new(
                    (ratio * f64::from(walker.velocity.x)) as f32,
                    (ratio * f64::from(walker.velocity.y)) as f32,
                    (ratio * f64::from(walker.velocity.z)) as f32,
                );
                process_hit_wall(sweep, walker, physics, hit.normal, hit.actor);
            }
            _ => {
                if hit.normal.z.abs() < WALKABLE_Z && f64::from(STEP_RETRY_SQUARED) < f64::from(delta.length_squared()) * f64::from(hit.time) {
                    move_actor(sweep, &mut walker.location, extent, down);
                    let rest = delta * (1.0 - hit.time);
                    step_up(sweep, walker, physics, grav_dir, desired, rest, hit);
                    return;
                }
            }
        }
        process_hit_wall(sweep, walker, physics, hit.normal, hit.actor);
        let mut wall = hit.normal;
        wall.z = 0.0;
        let wall = safe_normal(wall);
        hit.normal = wall;
        let along = delta.dot(wall);
        let slide = (delta - wall * along) * (1.0 - hit.time);
        if delta.dot(slide) >= 0.0 {
            move_actor_into(sweep, &mut walker.location, extent, slide, hit);
            if hit.time < 1.0 {
                process_hit_wall(sweep, walker, physics, hit.normal, hit.actor);
                let mut again = slide;
                two_wall_adjust(desired, &mut again, hit.normal, wall, hit.time);
                move_actor_into(sweep, &mut walker.location, extent, again, hit);
                if walker.controller_present && walker.my_controller && lifted && again.length() < STOP_SLIDE_MIN {
                    sweep.controller_call(ControllerCall::CannotMoveAnymore { location: walker.location, from_hit_wall: false });
                    controller_stop(sweep, walker);
                    return;
                }
            }
        }
    }
    move_actor(sweep, &mut walker.location, extent, down);
}

pub fn floor_probe<S: Sweep>(sweep: &mut S, walker: &Walker) -> SweepHit {
    let down = walker.gravity() * FLOOR_PROBE;
    sweep.sweep(walker.location, walker.location + down, walker.extent, FLOOR_FLAGS).unwrap_or(SweepHit::NONE)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LedgeCheck {
    pub delta: Vec3,
    pub checked_fall: bool,
    pub can_jump: bool,
}

fn walkable_floor(hit: &SweepHit) -> bool {
    f64::from(WALKABLE_Z) <= f64::from(hit.normal.z)
}

pub fn check_for_ledges<S: Sweep>(sweep: &mut S, walker: &Walker, accel_dir: Vec3, delta: Vec3, grav_dir: Vec3, checked_fall: bool) -> LedgeCheck {
    let unchanged = LedgeCheck { delta, checked_fall, can_jump: walker.can_jump };
    let location = walker.location;
    let (radius, height) = (walker.extent.x, walker.extent.z);
    let body = walker.extent;
    if !walker.on_world {
        let below = location - Vec3::Z * LEDGE_FLOOR_DROP;
        if sweep.sweep(location, below, body, LEDGE_FLAGS).is_none() {
            return LedgeCheck { can_jump: true, ..unchanged };
        }
    }
    let lead = if walker.avoid_ledges { accel_dir * (0.5 * radius) } else { accel_dir * radius };
    let mut point = location + delta + lead;
    if sweep.sweep(location, point, body, LEDGE_FLAGS).is_some() {
        return unchanged;
    }
    let size = delta.length();
    let reach = (size + radius).max(10.0) + height + LEDGE_FLOOR_DROP;
    let floor = sweep.sweep(point, point + grav_dir * reach, Vec3::ZERO, FLOOR_FLAGS);
    if !walker.avoid_ledges {
        point = location + delta;
    }
    let beyond = match floor {
        None => true,
        Some(hit) if !walkable_floor(&hit) => true,
        Some(hit) => {
            let nz = hit.normal.z;
            let slope = (((size + radius) * (1.0 - nz * nz).max(0.0).sqrt()) / nz).min(LEDGE_SLOPE_CAP);
            height + LEDGE_FLOOR_DROP + slope < reach * hit.time
        }
    };
    if !beyond {
        return unchanged;
    }
    if sweep.sweep(location, point, body, LEDGE_FLAGS).is_some() {
        return unchanged;
    }
    let drop = |sweep: &mut S, from: Vec3| sweep.sweep(from, from + grav_dir * LEDGE_PROBE_DEPTH, body, LEDGE_FLAGS);
    let ground = drop(sweep, point);
    if ground.is_some_and(|hit| walkable_floor(&hit)) {
        return unchanged;
    }
    if walker.controller_refuses_fall() {
        return LedgeCheck { delta: Vec3::ZERO, ..unchanged };
    }
    let along = safe_normal(point - location);
    let side = Vec3::new(along.y * size, -along.x * size, 0.0);
    for sign in [1.0, -1.0] {
        let candidate = point + side * sign;
        if sweep.sweep(point, candidate, body, LEDGE_FLAGS).is_none()
            && drop(sweep, candidate).is_some_and(|hit| walkable_floor(&hit))
        {
            return LedgeCheck { delta: safe_normal(candidate - location) * size, ..unchanged };
        }
    }
    let mut result = unchanged;
    if !checked_fall && walker.controller_present {
        result.checked_fall = true;
        result.can_jump = walker.can_jump;
    }
    result
}

pub fn phys_walking<S: Sweep>(sweep: &mut S, walker: &mut Walker, dt: f32) -> Outcome {
    if !walker.controller_present {
        return Outcome::Walking;
    }
    let start = walker.location;
    let extent = walker.extent;
    let gravity = walker.gravity();
    let mut remaining = dt;
    let mut iterations = 0;
    let mut ledge_checked = false;
    let zone = walker.volume.zone_velocity;
    let current = if zone.length_squared() > 0.0 && (walker.human || zone.length_squared() > WALK_CURRENT_MIN_SQUARED) {
        Vec3::new(zone.x * ZONE_CURRENT_GAIN * dt, zone.y * ZONE_CURRENT_GAIN * dt, 0.0)
    } else {
        Vec3::ZERO
    };
    loop {
        if remaining <= 0.0 || iterations >= MAX_ITERATIONS {
            if !walker.just_teleported && !walker.no_velocity_update {
                walker.velocity = (walker.location - start) / dt;
            }
            walker.no_velocity_update = false;
            walker.velocity.z = 0.0;
            return Outcome::Walking;
        }
        iterations += 1;
        let step = if walker.my_controller && remaining > PRECISE_THRESHOLD { PRECISE_STEP.min(remaining * PRECISE_FRACTION) } else { remaining };
        remaining -= step;
        let delta = Vec3::new((walker.velocity.x + current.x) * step, (walker.velocity.y + current.y) * step, 0.0);
        let before = walker.location;
        let mut delta = delta;
        if !nearly_zero(delta) && walker.wants_ledge_check() {
            let heading = safe_normal(walker.acceleration);
            let checked = check_for_ledges(sweep, walker, heading, delta, gravity, ledge_checked);
            delta = checked.delta;
            ledge_checked = checked.checked_fall;
        }
        if nearly_zero(delta) {
            remaining = 0.0;
        } else {
            let mut hit = move_actor(sweep, &mut walker.location, extent, delta);
            if hit.time < 1.0 {
                if walker.controller_present && walker.my_controller && matches!(hit.actor, HitActor::Other { immediately_stop: true, .. }) {
                    sweep.controller_call(ControllerCall::CannotMoveAnymore { location: walker.location, from_hit_wall: false });
                    controller_stop(sweep, walker);
                    return Outcome::Halted;
                }
                let size = delta.length();
                let desired = delta / size;
                let rest = delta * (1.0 - hit.time);
                step_up(sweep, walker, Physics::Walking, gravity, desired, rest, &mut hit);
            }
            if walker.head_in_water {
                return Outcome::Swimming { remaining };
            }
        }
        let mut floor = floor_probe(sweep, walker);
        let distance = floor.time * FLOOR_PROBE;
        walker.floor_normal = floor.normal;
        let stands = f64::from(WALKABLE_Z) <= f64::from(floor.normal.z) || nearly_zero(delta) || floor.normal.dot(delta) >= 0.0;
        if stands {
            if floor.time >= 1.0 || (walker.on_world && distance <= MAX_FLOOR_DISTANCE) {
                if distance >= MIN_FLOOR_DISTANCE {
                    walker.resting = true;
                } else {
                    let lift = (MIN_FLOOR_DISTANCE + MAX_FLOOR_DISTANCE) * 0.5 - distance;
                    move_actor(sweep, &mut walker.location, extent, Vec3::new(0.0, 0.0, lift));
                    floor.time = 0.0;
                }
            } else {
                move_actor(sweep, &mut walker.location, extent, gravity * FLOOR_PROBE);
                walker.on_world = true;
            }
        } else {
            let scale = floor.normal.z * MAX_STEP_HEIGHT;
            let slide = Vec3::new(floor.normal.x * scale, floor.normal.y * scale, floor.normal.z * scale - MAX_STEP_HEIGHT);
            move_actor(sweep, &mut walker.location, extent, slide);
            walker.on_world = true;
        }
        if floor.time >= 1.0 || floor.normal.z < WALKABLE_Z {
            if !walker.can_jump || (!walker.can_walk_off_ledges && walker.walking_or_crouched()) {
                walker.location = start;
                walker.velocity = Vec3::ZERO;
                return Outcome::Blocked;
            }
            let moved = (walker.location - before).truncate().length();
            let length = delta.length();
            if length == 0.0 {
                remaining = 0.0;
            } else {
                remaining += (1.0 - (moved / length).min(1.0)) * step;
            }
            walker.velocity.z = 0.0;
            walker.on_world = false;
            return Outcome::Falling { remaining };
        }
    }
}

pub const AIR_CONTROL_MIN: f32 = 0.05;
pub const FALL_STEP_CAP: f32 = 0.1;
pub const MAX_UP_STEP: f32 = 600.0;
pub const LANDING_MIN_TIME: f32 = 0.1;
pub const LANDING_MIN_SPAN: f32 = 0.003;
pub const AIR_MIN_SPEED: f32 = 10.0;

fn fall_drag(walker: &Walker) -> (f32, f32) {
    let volume = &walker.volume;
    if volume.water {
        (walker.buoyancy, volume.fluid_friction)
    } else if volume.gravity.z != walker.level_gravity_z {
        (walker.buoyancy * 0.5, volume.fluid_friction * 0.5)
    } else {
        (0.0, 0.0)
    }
}

pub fn new_fall_velocity(walker: &Walker, velocity: Vec3, force: Vec3, dt: f32) -> Vec3 {
    let (a, b) = fall_drag(walker);
    let by_mass = 1.0 - a / walker.mass;
    let by_drag = 1.0 - b * dt;
    velocity * by_drag + force * (by_mass * 0.5 * dt)
}

pub fn phys_falling<S: Sweep>(sweep: &mut S, walker: &mut Walker, dt: f32, mut iterations: u32) -> Outcome {
    let extent = walker.extent;
    let saved_acceleration = walker.acceleration;
    walker.acceleration.z = 0.0;
    let mut air_control = walker.air_control;
    if air_control > AIR_CONTROL_MIN {
        let heading = safe_normal(walker.acceleration);
        let probe = Vec3::new(
            (air_control * walker.accel_rate * heading.x + walker.velocity.x) * dt,
            (air_control * walker.accel_rate * heading.y + walker.velocity.y) * dt,
            0.0,
        );
        if sweep.sweep(walker.location, walker.location + probe, extent, 0x286).is_some() {
            air_control = 0.0;
        }
    }
    let mut air_accel = air_control * walker.accel_rate;
    let mut speed_limit = 0.0;
    let planar = Vec3::new(walker.velocity.x, walker.velocity.y, 0.0).length();
    if planar >= AIR_MIN_SPEED || air_control <= 0.0 {
        if walker.ground_speed <= planar {
            if air_control > AIR_CONTROL_MIN {
                speed_limit = planar;
            } else {
                air_accel = 1.0;
            }
        }
    } else {
        air_accel += (AIR_MIN_SPEED - planar) / dt;
    }
    if air_accel * air_accel < walker.acceleration.length_squared() {
        walker.acceleration = safe_normal(walker.acceleration) * air_accel;
    }
    let outcome = fall_loop(sweep, walker, dt, &mut iterations, speed_limit, extent);
    walker.acceleration = saved_acceleration;
    outcome
}

fn fall_loop<S: Sweep>(sweep: &mut S, walker: &mut Walker, dt: f32, iterations: &mut u32, speed_limit: f32, extent: Vec3) -> Outcome {
    let mut remaining = dt;
    loop {
        if remaining <= 0.0 || *iterations > 7 {
            return Outcome::Falling { remaining: 0.0 };
        }
        *iterations += 1;
        let step = if walker.my_controller && remaining > FALL_STEP_CAP { (remaining * 0.5).min(FALL_STEP_CAP) } else { remaining };
        remaining -= step;
        walker.just_teleported = false;
        let old_location = walker.location;
        let mut old_velocity = walker.velocity;
        let volume = walker.volume;
        let force = volume.gravity + walker.acceleration + walker.constant_acceleration;
        walker.velocity = new_fall_velocity(walker, walker.velocity, force, step);
        if walker.controller_present && walker.notify_apex && walker.velocity.z <= 0.0 {
            walker.just_teleported = true;
            walker.notify_apex = false;
            sweep.controller_call(ControllerCall::NotifyJumpApex);
            walker.jump_over_wall = false;
        }
        if speed_limit != 0.0 {
            let planar = Vec3::new(walker.velocity.x, walker.velocity.y, 0.0);
            if speed_limit * speed_limit < planar.length_squared() {
                let limited = safe_normal(planar) * speed_limit;
                walker.velocity.x = limited.x;
                walker.velocity.y = limited.y;
            }
        }
        let mut delta = (volume.zone_velocity + walker.velocity) * step;
        if delta.z > MAX_UP_STEP {
            delta.z = MAX_UP_STEP;
        }
        let mut hit = move_actor(sweep, &mut walker.location, extent, delta);
        if walker.volume.water && !walker.water_entry_blocked {
            remaining += (1.0 - hit.time) * step;
            return Outcome::Swimming { remaining };
        }
        if hit.time < 1.0 {
            let normal = hit.normal;
            if f64::from(WALKABLE_Z) <= f64::from(normal.z) {
                remaining += (1.0 - hit.time) * step;
                if !walker.just_teleported && LANDING_MIN_TIME < hit.time && hit.time * step > LANDING_MIN_SPAN {
                    walker.velocity = (walker.location - old_location) / (hit.time * step);
                }
                return Outcome::Landed { remaining, iterations: *iterations };
            }
            let along = normal.dot(delta);
            let adjusted = (delta - normal * along) * (1.0 - hit.time);
            let mut adjusted = adjusted;
            if adjusted.dot(delta) >= 0.0 {
                let first = normal;
                hit = move_actor(sweep, &mut walker.location, extent, adjusted);
                if hit.time < 1.0 {
                    if f64::from(WALKABLE_Z) <= f64::from(hit.normal.z) {
                        return Outcome::Landed { remaining, iterations: *iterations };
                    }
                    let desired = safe_normal(delta);
                    two_wall_adjust(desired, &mut adjusted, hit.normal, first, hit.time);
                    let crease = first.z > 0.0 && hit.normal.z > 0.0 && adjusted.z == 0.0 && first.dot(hit.normal) < 0.0;
                    hit = move_actor(sweep, &mut walker.location, extent, adjusted);
                    if crease || f64::from(WALKABLE_Z) <= f64::from(hit.normal.z) {
                        return Outcome::Landed { remaining, iterations: *iterations };
                    }
                }
            }
            let moved = walker.location - old_location;
            old_velocity.x = moved.x / step;
            old_velocity.y = moved.y / step;
        }
        if walker.head_in_water && !walker.water_entry_blocked {
            return Outcome::Swimming { remaining };
        }
        if !walker.just_teleported {
            walker.velocity = (walker.location - old_location) / step - volume.zone_velocity;
            if walker.velocity.z < old_velocity.z || old_velocity.z >= 0.0 {
                walker.velocity = walker.velocity * 2.0 - old_velocity;
            }
            if volume.terminal_velocity * volume.terminal_velocity < walker.velocity.length_squared() {
                walker.velocity = safe_normal(walker.velocity) * volume.terminal_velocity;
            }
        }
    }
}

pub const WALK_MOVE_PROBE: f32 = 12.0;
pub const SPOT_SLICE_HEIGHT: f32 = 1.0;
pub const SPOT_FLOOR_TIME: f32 = 0.5;
pub const SPOT_SLICE_SCALE: f32 = 0.5;
pub const SPOT_OFFSET_SCALE: f32 = 0.55;
pub const SPOT_SHIFT_SCALE: f32 = 1.1;
pub const SPOT_SETTLE_SCALE: f32 = 0.2;
pub const LANDED_FLAGS: u32 = 0x297;
pub const LANDED_EXTENT_SCALE: f32 = 0.9;
pub const LANDED_PROBE_SCALE: f32 = 0.2;
pub const LANDED_PROBE_BASE: f32 = 8.0;
pub const LANDED_SPOT_SCALE: f32 = 1.1;
pub const LANDED_JITTER_RANGE: f64 = 60.0;
pub const LANDED_JITTER_HALF: f64 = 30.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TestMove {
    Stopped,
    Moved,
    Fell,
    HitGoal,
}

impl TestMove {
    pub const fn code(self) -> u8 {
        match self {
            Self::Stopped => 0,
            Self::Moved => 1,
            Self::Fell => 2,
            Self::HitGoal => 5,
        }
    }
}

pub const fn hit_goal(goal_is_navigation_point: bool) -> TestMove {
    if goal_is_navigation_point { TestMove::Stopped } else { TestMove::HitGoal }
}

pub fn walk_move<S: Sweep>(sweep: &mut S, walker: &mut Walker, delta: Vec3, goal: &mut impl FnMut(&SweepHit) -> Option<TestMove>, threshold: f32) -> TestMove {
    let start = walker.location;
    let extent = walker.extent;
    let sign = if walker.volume.gravity.z > 0.0 { 1.0 } else { -1.0 };
    let down = Vec3::new(0.0, 0.0, sign * MAX_STEP_HEIGHT);
    let delta = Vec3::new(delta.x, delta.y, 0.0);
    let mut reached = |hit: &SweepHit| if hit.actor == HitActor::None { None } else { goal(hit) };
    let hit = move_actor(sweep, &mut walker.location, extent, delta);
    if let Some(result) = reached(&hit) {
        return result;
    }
    let raised_from = walker.location;
    if hit.time < 1.0 {
        let rest = 1.0 - hit.time;
        move_actor(sweep, &mut walker.location, extent, Vec3::new(0.0, 0.0, -down.z));
        let forward = move_actor(sweep, &mut walker.location, extent, Vec3::new(delta.x * rest, delta.y * rest, 0.0));
        if let Some(result) = reached(&forward) {
            return result;
        }
        let landed = move_actor(sweep, &mut walker.location, extent, down);
        if landed.time < 1.0 && f64::from(landed.normal.z) < WALKABLE_Z_F64 {
            walker.location = raised_from;
            return TestMove::Stopped;
        }
    }
    let before_probe = walker.location;
    let floor = move_actor(sweep, &mut walker.location, extent, Vec3::new(0.0, 0.0, sign * WALK_MOVE_PROBE));
    if floor.time == 1.0 || f64::from(floor.normal.z) < WALKABLE_Z_F64 {
        walker.location = before_probe;
        return TestMove::Fell;
    }
    if let Some(result) = reached(&floor) {
        return result;
    }
    if threshold * threshold > (walker.location - start).length_squared() { TestMove::Stopped } else { TestMove::Moved }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SliceCheck {
    pub fits: bool,
    pub success: bool,
}

fn push_by_floor(location: &mut Vec3, floor_normal: Vec3, extent: Vec3) {
    location.x += floor_normal.x * extent.x;
    location.y += floor_normal.y * extent.x;
}

pub fn check_slice<S: Sweep>(sweep: &mut S, location: &mut Vec3, extent: Vec3) -> Option<SliceCheck> {
    let slice = Vec3::new(extent.x, extent.y, SPOT_SLICE_HEIGHT);
    if sweep.encroaching(*location, slice)? {
        return Some(SliceCheck { fits: false, success: true });
    }
    let start = *location;
    let floor = sweep.line_check(start, Vec3::new(start.x, start.y, start.z - (extent.z + extent.z)), slice, WORLD_FLAGS);
    let floor_normal = floor.map_or(Vec3::ZERO, |(hit, _)| hit.normal);
    match floor {
        Some((hit, _)) if hit.time <= SPOT_FLOOR_TIME => {
            let lift = (SPOT_FLOOR_TIME - hit.time) + (SPOT_FLOOR_TIME - hit.time);
            location.z = (lift * extent.z + location.z) + 1.0;
            if !sweep.encroaching(*location, extent)? {
                return Some(SliceCheck { fits: true, success: false });
            }
            push_by_floor(location, floor_normal, extent);
            return Some(SliceCheck { fits: !sweep.encroaching(*location, extent)?, success: false });
        }
        Some((hit, _)) => {
            let sink = (hit.time + hit.time) - 1.0;
            location.z = (location.z - sink * extent.z) + 1.0;
        }
        None => location.z -= extent.z,
    }
    if !sweep.encroaching(*location, extent)? {
        let up = Vec3::new(location.x, location.y, extent.z + location.z);
        match sweep.line_check(*location, up, extent, WORLD_FLAGS) {
            Some((_, at)) => *location = at,
            None => location.z += extent.z,
        }
        return Some(SliceCheck { fits: true, success: false });
    }
    push_by_floor(location, floor_normal, extent);
    Some(SliceCheck { fits: !sweep.encroaching(*location, extent)?, success: false })
}

fn scaled(value: f64, scale: f32) -> f32 {
    (value * f64::from(scale)) as f32
}

pub fn find_spot<S: Sweep>(sweep: &mut S, extent: Vec3, location: &mut Vec3) -> Option<bool> {
    if !sweep.encroaching(*location, extent)? {
        return Some(true);
    }
    if extent == Vec3::ZERO {
        return Some(false);
    }
    let start = *location;
    let slice = check_slice(sweep, location, extent)?;
    if slice.fits {
        return Some(true);
    }
    if !slice.success {
        return Some(false);
    }
    let half = Vec3::new(extent.x * SPOT_SLICE_SCALE, extent.y * SPOT_SLICE_SCALE, SPOT_SLICE_HEIGHT);
    let mut fits = 0;
    for i in [-1i32, 1] {
        for j in [-1i32, 1] {
            if fits >= 2 {
                continue;
            }
            let offset_y = scaled(f64::from(j) * f64::from(extent.y), SPOT_OFFSET_SCALE);
            let offset_x = scaled(f64::from(i) * f64::from(extent.x), SPOT_OFFSET_SCALE);
            let probe = Vec3::new(start.x + offset_x, start.y + offset_y, start.z);
            if !sweep.encroaching(probe, half)? {
                fits += 1;
                location.x = (f64::from(offset_x) * f64::from(SPOT_SHIFT_SCALE) + f64::from(location.x)) as f32;
                location.y = (f64::from(offset_y) * f64::from(SPOT_SHIFT_SCALE) + f64::from(location.y)) as f32;
            }
        }
    }
    if fits == 0 {
        return Some(false);
    }
    if fits == 1 {
        *location = Vec3::new(
            (f64::from(location.x) * 2.0 - f64::from(start.x)) as f32,
            (f64::from(location.y) * 2.0 - f64::from(start.y)) as f32,
            (f64::from(location.z) * 2.0 - f64::from(start.z)) as f32,
        );
    }
    if sweep.encroaching(*location, extent)? {
        return Some(false);
    }
    let back = Vec3::new(
        scaled(f64::from(start.x) - f64::from(location.x), SPOT_SETTLE_SCALE),
        scaled(f64::from(start.y) - f64::from(location.y), SPOT_SETTLE_SCALE),
        scaled(f64::from(start.z) - f64::from(location.z), SPOT_SETTLE_SCALE),
    );
    if let Some((_, at)) = sweep.line_check(*location, start + back, extent, WORLD_FLAGS) {
        *location = at;
    }
    Some(true)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Landing {
    Landed,
    Respotted,
    Stopped,
}

pub fn process_landed<S: Sweep>(sweep: &mut S, walker: &mut Walker, alive: bool, frand: &mut impl FnMut() -> f32) -> Landing {
    let (radius, height) = (walker.extent.x, walker.extent.z);
    let location = walker.location;
    let probe_extent = Vec3::new(radius * LANDED_EXTENT_SCALE, radius * LANDED_EXTENT_SCALE, height * LANDED_EXTENT_SCALE);
    let end = Vec3::new(location.x, location.y, location.z - (height * LANDED_PROBE_SCALE + LANDED_PROBE_BASE));
    let floor = sweep.sweep(location, end, probe_extent, LANDED_FLAGS);
    if floor.is_none_or(|hit| hit.time == 1.0) {
        let mut spot = location;
        let extent = Vec3::new(radius * LANDED_SPOT_SCALE, radius * LANDED_SPOT_SCALE, height * LANDED_SPOT_SCALE);
        if find_spot(sweep, extent, &mut spot) == Some(true) && spot != location {
            walker.location = spot;
            walker.velocity.x = (f64::from(frand()) * LANDED_JITTER_RANGE - LANDED_JITTER_HALF + f64::from(walker.velocity.x)) as f32;
            walker.velocity.y = (f64::from(frand()) * LANDED_JITTER_RANGE - LANDED_JITTER_HALF + f64::from(walker.velocity.y)) as f32;
            return Landing::Respotted;
        }
    }
    if !alive {
        return Landing::Stopped;
    }
    walker.acceleration = safe_normal(walker.acceleration);
    Landing::Landed
}

pub const SWIM_POP_MAX: f32 = 160.0;
pub const SWIM_POP_SCALE: f32 = 0.4;
pub const SWIM_POP_BASE: f32 = 40.0;
pub const SWIM_ENTRY_BASE: f32 = -80.0;
pub const SWIM_ENTRY_SCALE: f64 = 0.7;
pub const ZONE_CURRENT_GAIN: f32 = 25.0;
pub const ZONE_CURRENT_MIN_SQUARED: f32 = 90000.0;
pub const WALK_CURRENT_MIN_SQUARED: f32 = 10000.0;
pub const SWIM_UP_CAP: f32 = 100.0;
pub const SWIM_WALL_Z: f32 = 0.2;
pub const SWIM_LEDGE_MAX: f32 = 0.5;
pub const SWIM_LEDGE_MIN: f32 = -0.2;
pub const WATER_EXIT_RAISE: f32 = 13.0;
pub const FLOOR_PROBE_FRACTION: f32 = 1.0 / 3.0;
pub const SWIM_REMAINING_MIN: f32 = 0.01;

pub fn calc_velocity(walker: &mut Walker, dir: Vec3, dt: f32, speed: f32) {
    walker.velocity = dir * speed;
    let Some(destination) = walker.destination else { return };
    let mut to_target = destination - walker.location;
    to_target.z = 0.0;
    if dt <= 0.0 {
        return;
    }
    let reach = walker.velocity.length();
    let distance = to_target.length();
    if reach * dt >= distance {
        walker.velocity = to_target / dt;
    }
}

fn move_smooth<S: Sweep>(sweep: &mut S, walker: &mut Walker, delta: &mut Vec3) -> (f32, SweepHit) {
    let old = walker.location;
    let extent = walker.extent;
    let hit = move_actor(sweep, &mut walker.location, extent, *delta);
    let below = walker.location - Vec3::Z * (extent.z * FLOOR_PROBE_FRACTION);
    let floor = sweep.sweep(walker.location, below, extent, WORLD_FLAGS);
    walker.on_water_floor = floor.is_some_and(|floor| floor.time < 1.0 && f64::from(floor.normal.z) >= f64::from(WALKABLE_Z));
    let mut fraction = 0.0;
    if !walker.head_in_water {
        let raised = walker.location + Vec3::Z * WATER_EXIT_RAISE;
        let mut exit = walker.location;
        if sweep.water_surface_crossed(old, raised) && old.z < exit.z {
            exit.z = old.z;
        }
        if exit != walker.location {
            let moved = walker.location - old;
            let corrected = exit - old;
            fraction = corrected.length() / moved.length();
            if moved.dot(exit - walker.location) > 0.0 {
                fraction = 0.0;
            }
            let correction = exit - walker.location;
            move_actor(sweep, &mut walker.location, extent, correction);
            *delta = exit - old;
        }
    }
    (fraction, hit)
}

pub fn enter_swimming<S: Sweep>(sweep: &mut S, walker: &mut Walker, old_location: Vec3, old_velocity: Vec3, step: f32, remaining: f32, iterations: u32) -> Outcome {
    let extent = walker.extent;
    let mut remaining = remaining;
    if !walker.just_teleported {
        if step > 0.0 {
            walker.velocity = (walker.location - old_location) / step;
        }
        walker.velocity = walker.velocity * 2.0 - old_velocity;
        let terminal = walker.volume.terminal_velocity;
        if terminal * terminal < walker.velocity.length_squared() {
            walker.velocity = safe_normal(walker.velocity) * terminal;
        }
    }
    let mut exit = walker.location;
    if sweep.water_surface_crossed(walker.location, old_location + Vec3::Z * WATER_EXIT_RAISE) && walker.location.z < old_location.z {
        exit = old_location;
        exit.x = walker.location.x;
        exit.y = walker.location.y;
    }
    if exit != walker.location {
        let moved = (walker.location - old_location).length();
        let corrected = (exit - old_location).length();
        remaining += moved / corrected * step;
        let correction = exit - walker.location;
        move_actor(sweep, &mut walker.location, extent, correction);
    }
    if walker.head_in_water {
        if -SWIM_POP_MAX < walker.velocity.z && walker.velocity.z < 0.0 {
            let planar = Vec3::new(walker.velocity.x, walker.velocity.y, 0.0).length();
            walker.velocity.z = (f64::from(SWIM_ENTRY_BASE) - f64::from(planar) * SWIM_ENTRY_SCALE) as f32;
        }
        if remaining > SWIM_REMAINING_MIN && iterations < MAX_ITERATIONS {
            return Outcome::Swimming { remaining };
        }
        return Outcome::Swimming { remaining: 0.0 };
    }
    Outcome::Surfaced { remaining }
}

pub fn phys_swimming<S: Sweep>(sweep: &mut S, walker: &mut Walker, dt: f32) -> Outcome {
    let (drag_a, _) = fall_drag(walker);
    if walker.velocity.z > SWIM_UP_CAP && walker.mass != 0.0 {
        walker.velocity.z = walker.velocity.z / walker.mass * drag_a;
    }
    let start = walker.location;
    walker.just_teleported = false;
    let dir = if nearly_zero_exact(walker.acceleration) { walker.acceleration } else { safe_normal(walker.acceleration) };
    calc_velocity(walker, dir, dt, walker.move_speed);
    let current = if walker.human || walker.volume.zone_velocity.length_squared() > ZONE_CURRENT_MIN_SQUARED {
        walker.volume.zone_velocity * (ZONE_CURRENT_GAIN * dt)
    } else {
        Vec3::ZERO
    };
    let mut adjusted = (walker.velocity + current) * dt;
    let intended = adjusted;
    let (fraction, hit) = move_smooth(sweep, walker, &mut adjusted);
    let mut consumed = fraction * dt;
    if adjusted != intended {
        walker.velocity = adjusted / dt - current;
    }
    if hit.time < 1.0 {
        let gravity = Vec3::new(0.0, 0.0, if walker.volume.gravity.z > 0.0 { 1.0 } else { -1.0 });
        let heading = safe_normal(adjusted);
        let intent = safe_normal(walker.acceleration);
        let aligned = intent.z * gravity.z;
        if SWIM_WALL_Z <= hit.normal.z.abs() || SWIM_LEDGE_MAX <= aligned || aligned <= SWIM_LEDGE_MIN {
            process_hit_wall(sweep, walker, Physics::Swimming, hit.normal, hit.actor);
            let normal = hit.normal;
            let along = adjusted.dot(normal);
            let mut slide = (adjusted - normal * along) * (1.0 - hit.time);
            if slide.dot(adjusted) >= 0.0 {
                let (again, second) = move_smooth(sweep, walker, &mut slide);
                consumed *= (1.0 - hit.time) * again;
                if second.time < 1.0 {
                    process_hit_wall(sweep, walker, Physics::Swimming, second.normal, second.actor);
                    two_wall_adjust(heading, &mut slide, second.normal, normal, second.time);
                    let (last, _) = move_smooth(sweep, walker, &mut slide);
                    consumed *= (1.0 - second.time) * last;
                }
            }
        } else {
            let rest = adjusted * (1.0 - hit.time);
            let mut step_hit = hit;
            step_up(sweep, walker, Physics::Swimming, gravity, heading, rest, &mut step_hit);
        }
    }
    if !walker.just_teleported && consumed < dt {
        let keep_z = walker.velocity.z;
        walker.velocity = (walker.location - start) / (dt - consumed);
        walker.velocity.z = keep_z;
    }
    if !walker.head_in_water && walker.velocity.z > 0.0 && walker.velocity.z < SWIM_POP_MAX {
        let planar = Vec3::new(walker.velocity.x, walker.velocity.y, 0.0).length();
        walker.velocity.z = planar * SWIM_POP_SCALE + SWIM_POP_BASE;
    }
    if !walker.head_in_water {
        return Outcome::Surfaced { remaining: consumed };
    }
    Outcome::Swimming { remaining: consumed }
}

fn nearly_zero_exact(v: Vec3) -> bool {
    v == Vec3::ZERO
}

#[cfg(test)]
mod tests {
    #[test]
    fn lineage_pawns_never_block_each_other() {
        assert!(!super::pawn_blocks_mover(super::LINEAGE_PAWN_CAN_IGNORE_COLLISION, super::LINEAGE_PAWN_CAN_BE_IGNORED_COLLISION));
        assert!(super::pawn_blocks_mover(false, true));
        assert!(super::pawn_blocks_mover(true, false));
        assert!(super::pawn_blocks_mover(false, false));
    }

    use super::*;

    struct Plane {
        floor_until: Option<f32>,
        floor: f32,
        wall_x: Option<f32>,
        step: Option<(f32, f32)>,
    }

    fn box_hit(start: Vec3, end: Vec3, extent: Vec3, solid: &dyn Fn(Vec3) -> Option<Vec3>) -> Option<SweepHit> {
        let _ = extent;
        let mut last = None;
        let steps = 64;
        for index in 1..=steps {
            let time = index as f32 / steps as f32;
            if let Some(normal) = solid(start + (end - start) * time) {
                let mut low = (index - 1) as f32 / steps as f32;
                let mut high = time;
                for _ in 0..20 {
                    let mid = (low + high) * 0.5;
                    if solid(start + (end - start) * mid).is_some() { high = mid } else { low = mid }
                }
                last = Some(SweepHit::world(low, normal));
                break;
            }
        }
        last
    }

    impl Sweep for Plane {
        fn sweep(&mut self, start: Vec3, end: Vec3, extent: Vec3, _flags: u32) -> Option<SweepHit> {
            let floor = self.floor;
            let wall = self.wall_x;
            let step = self.step;
            let floor_until = self.floor_until;
            let solid = move |p: Vec3| -> Option<Vec3> {
                if let Some((x, top)) = step
                    && p.x + extent.x > x
                    && p.z - extent.z < top
                {
                    return Some(if top - (p.z - extent.z) < p.x + extent.x - x { Vec3::Z } else { Vec3::new(-1.0, 0.0, 0.0) });
                }
                if let Some(x) = wall
                    && p.x + extent.x > x
                {
                    return Some(Vec3::new(-1.0, 0.0, 0.0));
                }
                if p.z - extent.z < floor && floor_until.is_none_or(|edge| p.x < edge) { Some(Vec3::Z) } else { None }
            };
            box_hit(start, end, extent, &solid)
        }
    }

    fn walker_on(floor: f32) -> Walker {
        let mut walker = Walker::new(Vec3::new(0.0, 0.0, floor + 20.0 + 2.15), 10.0, 20.0);
        walker.on_world = true;
        walker
    }

    #[test]
    fn nearly_zero_and_safe_normal_follow_core() {
        assert!(nearly_zero(Vec3::new(0.00009, -0.00009, 0.0)));
        assert!(!nearly_zero(Vec3::new(0.0001, 0.0, 0.0)));
        assert_eq!(safe_normal(Vec3::new(0.0, 0.00005, 0.0)), Vec3::ZERO);
        assert!((safe_normal(Vec3::new(3.0, 0.0, 4.0)).length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn move_actor_stops_two_units_short_of_a_wall() {
        let mut world = Plane { floor_until: None, floor: -1000.0, wall_x: Some(100.0), step: None };
        let mut location = Vec3::ZERO;
        let hit = move_actor(&mut world, &mut location, Vec3::new(10.0, 10.0, 20.0), Vec3::new(200.0, 0.0, 0.0));
        assert!(hit.time < 1.0);
        assert!(location.x > 85.0 && location.x <= 88.5, "x={}", location.x);
        assert_eq!(hit.normal, Vec3::new(-1.0, 0.0, 0.0));
    }

    #[test]
    fn move_actor_into_keeps_the_previous_hit_for_an_exactly_zero_delta() {
        let mut world = Plane { floor_until: None, floor: -1000.0, wall_x: None, step: None };
        let mut location = Vec3::new(1.0, 2.0, 3.0);
        let stale = SweepHit::world(0.25, Vec3::new(-1.0, 0.0, 0.0));
        let mut hit = stale;
        move_actor_into(&mut world, &mut location, Vec3::new(10.0, 10.0, 20.0), Vec3::ZERO, &mut hit);
        assert_eq!(hit, stale);
        move_actor_into(&mut world, &mut location, Vec3::new(10.0, 10.0, 20.0), Vec3::new(0.00001, 0.0, 0.0), &mut hit);
        assert_eq!(hit, SweepHit::NONE);
    }

    #[test]
    fn move_actor_with_a_hit_inside_the_margin_does_not_move() {
        let mut world = Plane { floor_until: None, floor: -1000.0, wall_x: Some(91.0), step: None };
        let mut location = Vec3::new(80.5, 0.0, 0.0);
        let hit = move_actor(&mut world, &mut location, Vec3::new(10.0, 10.0, 20.0), Vec3::new(5.0, 0.0, 0.0));
        assert_eq!(hit.time, 0.0);
        assert_eq!(location.x, 80.5);
    }

    #[test]
    fn two_wall_adjust_slides_along_a_single_wall_and_follows_a_corner() {
        let mut delta = Vec3::new(10.0, 10.0, 0.0);
        two_wall_adjust(Vec3::new(1.0, 0.0, 0.0), &mut delta, Vec3::new(-1.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0), 0.0);
        assert_eq!(delta, Vec3::ZERO);
        let mut corner = Vec3::new(10.0, 10.0, 0.0);
        two_wall_adjust(Vec3::new(0.0, 1.0, 0.0), &mut corner, Vec3::new(-1.0, 0.0, 0.0), Vec3::new(0.0, -1.0, 0.0), 0.0);
        assert!(corner.x.abs() < 1e-5 && corner.y.abs() < 1e-5);
    }

    #[test]
    fn walking_on_flat_ground_keeps_the_hover_band_and_moves() {
        let mut world = Plane { floor_until: None, floor: 0.0, wall_x: None, step: None };
        let mut walker = walker_on(0.0);
        walker.velocity = Vec3::new(100.0, 0.0, 0.0);
        let outcome = phys_walking(&mut world, &mut walker, 0.1);
        assert_eq!(outcome, Outcome::Walking);
        assert!((walker.location.x - 10.0).abs() < 0.01);
        let gap = walker.location.z - 20.0;
        assert!((MIN_FLOOR_DISTANCE - 0.1..=MAX_FLOOR_DISTANCE + 0.1).contains(&gap), "gap={gap}");
    }

    #[test]
    fn walking_into_a_wall_stops_without_crossing_it() {
        let mut world = Plane { floor_until: None, floor: 0.0, wall_x: Some(60.0), step: None };
        let mut walker = walker_on(0.0);
        walker.velocity = Vec3::new(400.0, 0.0, 0.0);
        for _ in 0..30 {
            walker.velocity = Vec3::new(400.0, 0.0, 0.0);
            phys_walking(&mut world, &mut walker, 0.05);
        }
        assert!(walker.location.x + 10.0 <= 60.0, "x={}", walker.location.x);
        assert!(walker.location.x > 40.0);
    }

    #[test]
    fn a_step_within_ten_units_is_climbed() {
        let mut world = Plane { floor_until: None, floor: 0.0, wall_x: None, step: Some((40.0, 8.0)) };
        let mut walker = walker_on(0.0);
        for _ in 0..40 {
            walker.velocity = Vec3::new(200.0, 0.0, 0.0);
            phys_walking(&mut world, &mut walker, 0.05);
        }
        assert!(walker.location.x > 60.0, "x={}", walker.location.x);
        assert!(walker.location.z - 20.0 > 7.0, "z={}", walker.location.z);
    }

    #[test]
    fn a_step_above_ten_units_blocks() {
        let mut world = Plane { floor_until: None, floor: 0.0, wall_x: None, step: Some((40.0, 30.0)) };
        let mut walker = walker_on(0.0);
        for _ in 0..40 {
            walker.velocity = Vec3::new(200.0, 0.0, 0.0);
            phys_walking(&mut world, &mut walker, 0.05);
        }
        assert!(walker.location.x + 10.0 <= 40.5, "x={}", walker.location.x);
    }

    #[test]
    fn leaving_the_floor_starts_falling_with_the_remaining_time() {
        let mut world = Plane { floor_until: None, floor: -10000.0, wall_x: None, step: None };
        let mut walker = Walker::new(Vec3::new(0.0, 0.0, 22.15), 10.0, 20.0);
        assert!(walker.can_jump && !walker.walking_or_crouched());
        walker.velocity = Vec3::new(100.0, 0.0, 0.0);
        let outcome = phys_walking(&mut world, &mut walker, 0.1);
        assert!(matches!(outcome, Outcome::Falling { .. }));
    }

    #[test]
    fn walking_without_can_walk_off_ledges_reverts_and_so_does_a_pawn_that_cannot_jump() {
        let mut world = Plane { floor_until: None, floor: -10000.0, wall_x: None, step: None };
        let mut walking = Walker::new(Vec3::new(0.0, 0.0, 22.15), 10.0, 20.0);
        walking.is_walking = true;
        walking.velocity = Vec3::new(100.0, 0.0, 0.0);
        assert_eq!(phys_walking(&mut world, &mut walking, 0.1), Outcome::Blocked);
        walking.can_walk_off_ledges = true;
        walking.velocity = Vec3::new(100.0, 0.0, 0.0);
        assert!(matches!(phys_walking(&mut world, &mut walking, 0.1), Outcome::Falling { .. }));
        let mut walker = Walker::new(Vec3::new(0.0, 0.0, 22.15), 10.0, 20.0);
        walker.can_jump = false;
        walker.velocity = Vec3::new(100.0, 0.0, 0.0);
        let outcome = phys_walking(&mut world, &mut walker, 0.1);
        assert_eq!(outcome, Outcome::Blocked);
        assert_eq!(walker.location, Vec3::new(0.0, 0.0, 22.15));
    }

    #[test]
    fn free_fall_follows_half_g_t_squared_and_reaches_the_velocity_of_g_t() {
        let mut world = Plane { floor_until: None, floor: -1.0e9, wall_x: None, step: None };
        let mut walker = Walker::new(Vec3::new(0.0, 0.0, 0.0), 10.0, 20.0);
        walker.air_control = 0.0;
        let mut elapsed = 0.0;
        for _ in 0..60 {
            let outcome = phys_falling(&mut world, &mut walker, 1.0 / 60.0, 0);
            assert!(matches!(outcome, Outcome::Falling { .. }));
            elapsed += 1.0 / 60.0;
        }
        assert!((elapsed - 1.0_f32).abs() < 1e-4);
        assert!((walker.location.z + 750.0).abs() < 1.0, "z={}", walker.location.z);
        assert!((walker.velocity.z + 1500.0).abs() < 1.0, "vz={}", walker.velocity.z);
    }

    #[test]
    fn terminal_velocity_caps_the_fall() {
        let mut world = Plane { floor_until: None, floor: -1.0e9, wall_x: None, step: None };
        let mut walker = Walker::new(Vec3::ZERO, 10.0, 20.0);
        walker.air_control = 0.0;
        for _ in 0..240 {
            phys_falling(&mut world, &mut walker, 1.0 / 60.0, 0);
        }
        assert!((walker.velocity.length() - Volume::DEFAULT_TERMINAL_VELOCITY).abs() < 1.0, "v={}", walker.velocity.length());
    }

    #[test]
    fn falling_onto_a_floor_lands_with_the_remaining_time() {
        let mut world = Plane { floor_until: None, floor: 0.0, wall_x: None, step: None };
        let mut walker = Walker::new(Vec3::new(0.0, 0.0, 40.0), 10.0, 20.0);
        walker.air_control = 0.0;
        let mut landed = None;
        for _ in 0..120 {
            if let Outcome::Landed { remaining, .. } = phys_falling(&mut world, &mut walker, 1.0 / 60.0, 0) {
                landed = Some(remaining);
                break;
            }
        }
        assert!(landed.is_some_and(|remaining| remaining > 0.0));
        assert!(walker.location.z - 20.0 < 4.0 && walker.location.z - 20.0 > -1.0, "z={}", walker.location.z);
    }

    #[test]
    fn falling_against_a_wall_slides_down_it() {
        let mut world = Plane { floor_until: None, floor: -1.0e9, wall_x: Some(30.0), step: None };
        let mut walker = Walker::new(Vec3::new(0.0, 0.0, 0.0), 10.0, 20.0);
        walker.air_control = 0.0;
        walker.velocity = Vec3::new(300.0, 0.0, 0.0);
        for _ in 0..30 {
            phys_falling(&mut world, &mut walker, 1.0 / 60.0, 0);
        }
        assert!(walker.location.x + 10.0 <= 30.0, "x={}", walker.location.x);
        assert!(walker.location.z < -50.0);
    }

    #[test]
    fn check_for_ledges_flags_a_drop_ahead_and_lets_the_controller_stop_it() {
        let mut world = Plane { floor_until: Some(60.0), floor: 0.0, wall_x: None, step: None };
        let mut walker = walker_on(0.0);
        walker.is_walking = true;
        walker.location.x = 55.0;
        let delta = Vec3::new(8.0, 0.0, 0.0);
        let heading = Vec3::X;
        let open = check_for_ledges(&mut world, &walker, heading, delta, Vec3::NEG_Z, false);
        assert_eq!(open.delta, delta);
        assert!(open.checked_fall && open.can_jump);
        walker.controller = Controller::Ai;
        assert_eq!(check_for_ledges(&mut world, &walker, heading, delta, Vec3::NEG_Z, false).delta, delta);
        walker.can_jump = false;
        let refused = check_for_ledges(&mut world, &walker, heading, delta, Vec3::NEG_Z, false);
        assert_eq!(refused.delta, Vec3::ZERO);
        walker.location.x = 0.0;
        let safe = check_for_ledges(&mut world, &walker, heading, delta, Vec3::NEG_Z, false);
        assert_eq!(safe.delta, delta);
        assert!(!safe.checked_fall);
    }

    #[test]
    fn swimming_moves_along_the_acceleration_at_the_move_speed_and_stops_on_arrival() {
        let mut world = Plane { floor_until: None, floor: -1.0e9, wall_x: None, step: None };
        let mut walker = Walker::new(Vec3::ZERO, 10.0, 20.0);
        walker.head_in_water = true;
        walker.volume.water = true;
        walker.move_speed = 100.0;
        walker.acceleration = Vec3::new(1.0, 0.0, 0.0);
        walker.destination = Some(Vec3::new(50.0, 0.0, 0.0));
        let mut last = Outcome::Walking;
        for _ in 0..120 {
            last = phys_swimming(&mut world, &mut walker, 1.0 / 60.0);
        }
        assert!(matches!(last, Outcome::Swimming { .. }));
        assert!((walker.location.x - 50.0).abs() < 0.5, "x={}", walker.location.x);
    }

    #[test]
    fn swimming_with_the_head_out_of_the_water_hands_back_to_walking() {
        let mut world = Plane { floor_until: None, floor: -1.0e9, wall_x: None, step: None };
        let mut walker = Walker::new(Vec3::ZERO, 10.0, 20.0);
        walker.head_in_water = false;
        walker.move_speed = 100.0;
        walker.acceleration = Vec3::X;
        let outcome = phys_swimming(&mut world, &mut walker, 1.0 / 60.0);
        assert!(matches!(outcome, Outcome::Surfaced { .. }));
    }

    #[test]
    fn calc_velocity_limits_the_step_to_the_remaining_planar_distance() {
        let mut walker = Walker::new(Vec3::ZERO, 10.0, 20.0);
        walker.destination = Some(Vec3::new(1.0, 0.0, 5.0));
        calc_velocity(&mut walker, Vec3::X, 0.1, 100.0);
        assert!((walker.velocity.x - 10.0).abs() < 1e-4 && walker.velocity.z == 0.0);
    }
    #[test]
    fn a_water_current_pushes_a_walking_human_and_only_a_strong_one_pushes_others() {
        let mut world = Plane { floor_until: None, floor: 0.0, wall_x: None, step: None };
        let mut weak = walker_on(0.0);
        weak.volume.zone_velocity = Vec3::new(40.0, 0.0, 0.0);
        let mut human = weak;
        human.human = true;
        phys_walking(&mut world, &mut weak, 0.1);
        phys_walking(&mut world, &mut human, 0.1);
        assert!(weak.location.x.abs() < 0.01, "x={}", weak.location.x);
        assert!((human.location.x - 40.0 * ZONE_CURRENT_GAIN * 0.1 * 0.1).abs() < 0.01, "x={}", human.location.x);
        let mut strong = walker_on(0.0);
        strong.volume.zone_velocity = Vec3::new(120.0, 0.0, 0.0);
        phys_walking(&mut world, &mut strong, 0.1);
        assert!((strong.location.x - 120.0 * ZONE_CURRENT_GAIN * 0.1 * 0.1).abs() < 0.01, "x={}", strong.location.x);
    }

    #[test]
    fn controller_slots_follow_the_player_and_ai_variants() {
        let mut walker = walker_on(0.0);
        assert!(!walker.wants_ledge_check() && !walker.controller_refuses_fall());
        walker.is_crouched = true;
        assert!(walker.wants_ledge_check());
        walker.controller = Controller::Ai;
        walker.is_crouched = false;
        assert!(walker.wants_ledge_check() && !walker.controller_refuses_fall());
        walker.can_fly = true;
        assert!(!walker.wants_ledge_check());
        walker.can_jump = false;
        assert!(walker.controller_refuses_fall());
        walker.can_jump = true;
        walker.stop_at_ledges = true;
        assert!(walker.controller_refuses_fall());
    }

    struct Room {
        floor: f32,
        wall_x: Option<f32>,
        wall_y: Option<f32>,
        actor: HitActor,
        calls: Vec<ControllerCall>,
        floor_probes: usize,
    }

    impl Room {
        fn new(floor: f32, wall_x: Option<f32>, wall_y: Option<f32>, actor: HitActor) -> Self {
            Self { floor, wall_x, wall_y, actor, calls: Vec::new(), floor_probes: 0 }
        }
    }

    impl Sweep for Room {
        fn sweep(&mut self, start: Vec3, end: Vec3, extent: Vec3, flags: u32) -> Option<SweepHit> {
            if flags == FLOOR_FLAGS {
                self.floor_probes += 1;
            }
            let (floor, wall_x, wall_y) = (self.floor, self.wall_x, self.wall_y);
            let solid = move |p: Vec3| -> Option<Vec3> {
                if let Some(x) = wall_x
                    && p.x + extent.x > x
                {
                    return Some(Vec3::NEG_X);
                }
                if let Some(y) = wall_y
                    && p.y + extent.y > y
                {
                    return Some(Vec3::NEG_Y);
                }
                if p.z - extent.z < floor { Some(Vec3::Z) } else { None }
            };
            let actor = self.actor;
            box_hit(start, end, extent, &solid).map(|hit| if hit.normal.z == 0.0 { SweepHit { actor, ..hit } } else { hit })
        }

        fn controller_call(&mut self, call: ControllerCall) {
            self.calls.push(call);
        }
    }

    fn local_player(location: Vec3) -> Walker {
        let mut walker = Walker::new(location, 10.0, 20.0);
        walker.on_world = true;
        walker.human = true;
        walker.my_controller = true;
        walker.moving = true;
        walker
    }

    fn halts(calls: &[ControllerCall]) -> Vec<bool> {
        calls.iter().filter_map(|call| match call {
            ControllerCall::CannotMoveAnymore { from_hit_wall, .. } => Some(*from_hit_wall),
            ControllerCall::Stop { .. } | ControllerCall::NotifyJumpApex => None,
        }).collect()
    }

    fn stops(calls: &[ControllerCall]) -> usize {
        calls.iter().filter(|call| matches!(call, ControllerCall::Stop { .. })).count()
    }

    #[test]
    fn a_corner_that_leaves_less_than_a_tenth_stops_the_local_controller() {
        let mut room = Room::new(0.0, Some(100.0), Some(100.0), HitActor::WORLD);
        let mut walker = local_player(Vec3::new(87.0, 87.0, 22.15));
        walker.destination = Some(Vec3::new(500.0, 500.0, 22.15));
        walker.acceleration = safe_normal(Vec3::new(1.0, 1.0, 0.0));
        walker.velocity = Vec3::new(200.0, 200.0, 0.0);
        phys_walking(&mut room, &mut walker, 0.05);
        assert_eq!(halts(&room.calls), vec![true, false]);
        assert_eq!(stops(&room.calls), 1);
        assert!(matches!(room.calls.last(), Some(ControllerCall::Stop { .. })));
        assert_eq!(walker.acceleration, Vec3::ZERO);
        assert!(!walker.moving);
        let mut other = Room::new(0.0, Some(100.0), Some(100.0), HitActor::WORLD);
        let mut remote = local_player(Vec3::new(87.0, 87.0, 22.15));
        remote.my_controller = false;
        remote.acceleration = safe_normal(Vec3::new(1.0, 1.0, 0.0));
        remote.velocity = Vec3::new(200.0, 200.0, 0.0);
        phys_walking(&mut other, &mut remote, 0.05);
        assert!(other.calls.is_empty());
        assert!(remote.location.x + 10.0 <= 100.0 && remote.location.y + 10.0 <= 100.0);
    }

    #[test]
    fn hit_wall_notify_reports_only_while_the_local_controller_is_moving_and_always_lets_the_step_go_on() {
        for (moving, expected) in [(true, vec![true]), (false, vec![])] {
            let mut room = Room::new(0.0, Some(100.0), None, HitActor::WORLD);
            let mut walker = local_player(Vec3::new(80.0, 0.0, 22.15));
            walker.moving = moving;
            walker.velocity = Vec3::new(200.0, 0.001, 0.0);
            phys_walking(&mut room, &mut walker, 0.05);
            assert_eq!(halts(&room.calls), expected);
            assert!(walker.location.x + 10.0 <= 100.0);
        }
        let room = &mut Room::new(0.0, None, None, HitActor::WORLD);
        let mut ai = walker_on(0.0);
        ai.controller = Controller::Ai;
        assert!(ai.hit_wall_notify(room) && room.calls.is_empty());
    }

    #[test]
    fn an_immediately_stop_actor_ends_phys_walking_for_the_local_controller_only() {
        let stop = HitActor::Other { immediately_stop: true, pushable_mass: None };
        let mut room = Room::new(0.0, Some(100.0), None, stop);
        let mut walker = local_player(Vec3::new(80.0, 0.0, 22.15));
        walker.acceleration = Vec3::X;
        walker.velocity = Vec3::new(200.0, 0.0, 0.0);
        assert_eq!(phys_walking(&mut room, &mut walker, 0.05), Outcome::Halted);
        assert_eq!(room.calls, vec![ControllerCall::CannotMoveAnymore { location: walker.location, from_hit_wall: false }, ControllerCall::Stop { location: walker.location }]);
        assert_eq!(walker.velocity, Vec3::new(200.0, 0.0, 0.0));
        let mut other = Room::new(0.0, Some(100.0), None, stop);
        let mut remote = local_player(Vec3::new(80.0, 0.0, 22.15));
        remote.my_controller = false;
        remote.velocity = Vec3::new(200.0, 0.0, 0.0);
        assert_eq!(phys_walking(&mut other, &mut remote, 0.05), Outcome::Walking);
        assert!(other.calls.is_empty());
    }

    #[test]
    fn a_pushable_decoration_scales_the_velocity_by_the_mass_ratio_and_keeps_it() {
        let crate_ = HitActor::Other { immediately_stop: false, pushable_mass: Some(300.0) };
        let mut room = Room::new(0.0, Some(100.0), None, crate_);
        let mut walker = local_player(Vec3::new(85.0, 0.0, 22.15));
        walker.moving = false;
        walker.velocity = Vec3::new(200.0, 0.001, 0.0);
        phys_walking(&mut room, &mut walker, 0.05);
        assert!((walker.velocity.x - 50.0).abs() < 1e-4, "vx={}", walker.velocity.x);
        assert!(!walker.no_velocity_update);
        let mut side = Room::new(0.0, Some(100.0), None, crate_);
        let mut glancing = local_player(Vec3::new(85.0, 0.0, 22.15));
        glancing.moving = false;
        glancing.velocity = Vec3::new(200.0, 200.0, 0.0);
        phys_walking(&mut side, &mut glancing, 0.05);
        assert!(glancing.velocity.y > 100.0, "vy={}", glancing.velocity.y);
        let mut npc_room = Room::new(0.0, Some(100.0), None, crate_);
        let mut npc = walker_on(0.0);
        npc.location.x = 85.0;
        npc.velocity = Vec3::new(200.0, 0.0, 0.0);
        phys_walking(&mut npc_room, &mut npc, 0.05);
        assert!((npc.velocity.x - (npc.location.x - 85.0) / 0.05).abs() < 1e-3, "vx={}", npc.velocity.x);
    }

    #[test]
    fn a_ceiling_contact_moves_along_the_normal_only_outside_walking() {
        let ceiling = SweepHit { time: 0.5, normal: Vec3::new(0.6, 0.0, -0.8), actor: HitActor::WORLD };
        for (physics, expected) in [(Physics::Walking, Vec3::new(0.0, 0.0, -10.0)), (Physics::Swimming, Vec3::new(10.0, 0.0, -18.0))] {
            let mut room = Room::new(-1.0e9, None, None, HitActor::WORLD);
            let mut walker = Walker::new(Vec3::ZERO, 10.0, 20.0);
            let mut hit = ceiling;
            step_up(&mut room, &mut walker, physics, Vec3::NEG_Z, Vec3::X, Vec3::new(10.0, 0.0, 0.0), &mut hit);
            assert!((walker.location - expected).length() < 1e-4, "{physics:?}: {}", walker.location);
        }
    }

    #[test]
    fn process_hit_wall_drops_ten_units_only_for_a_wall_at_least_min_hit_wall_against_the_destination() {
        let run = |normal: Vec3, actor: HitActor, min_hit_wall: f32, acceleration: Vec3| {
            let mut room = Room::new(-1.0e9, None, None, HitActor::WORLD);
            let mut walker = Walker::new(Vec3::ZERO, 10.0, 20.0);
            walker.destination = Some(Vec3::new(100.0, 0.0, 30.0));
            walker.acceleration = acceleration;
            walker.min_hit_wall = min_hit_wall;
            process_hit_wall(&mut room, &mut walker, Physics::Walking, normal, actor);
            walker.location.z
        };
        let diagonal = safe_normal(Vec3::new(-1.0, 1.0, 0.0));
        assert_eq!(run(Vec3::NEG_X, HitActor::WORLD, CONTROLLER_MIN_HIT_WALL, Vec3::X), -HIT_WALL_DROP);
        assert_eq!(run(diagonal, HitActor::WORLD, CONTROLLER_MIN_HIT_WALL, Vec3::X), 0.0);
        assert_eq!(run(diagonal, HitActor::WORLD, AI_CONTROLLER_MIN_HIT_WALL, Vec3::X), -HIT_WALL_DROP);
        assert_eq!(run(Vec3::NEG_X, HitActor::Pawn, CONTROLLER_MIN_HIT_WALL, Vec3::X), 0.0);
        assert_eq!(run(Vec3::NEG_X, HitActor::None, CONTROLLER_MIN_HIT_WALL, Vec3::X), 0.0);
        assert_eq!(run(Vec3::NEG_X, HitActor::WORLD, CONTROLLER_MIN_HIT_WALL, Vec3::ZERO), 0.0);
    }

    #[test]
    fn the_local_controller_walks_in_steps_of_min_a_tenth_and_half_the_remaining_time() {
        let mut room = Room::new(0.0, None, None, HitActor::WORLD);
        let mut walker = local_player(Vec3::new(0.0, 0.0, 22.15));
        walker.velocity = Vec3::new(100.0, 0.0, 0.0);
        phys_walking(&mut room, &mut walker, 0.2);
        assert_eq!(room.floor_probes, 3);
        let mut plain = Room::new(0.0, None, None, HitActor::WORLD);
        let mut other = walker_on(0.0);
        other.velocity = Vec3::new(100.0, 0.0, 0.0);
        phys_walking(&mut plain, &mut other, 0.2);
        assert_eq!(plain.floor_probes, 1);
    }

    struct Boxes {
        solids: Vec<(Vec3, Vec3)>,
        point_check: bool,
        calls: Vec<ControllerCall>,
    }

    impl Boxes {
        fn new(solids: Vec<(Vec3, Vec3)>, point_check: bool) -> Self {
            Self { solids, point_check, calls: Vec::new() }
        }
    }

    fn slab(start: Vec3, end: Vec3, low: Vec3, high: Vec3) -> Option<(f32, Vec3)> {
        let delta = end - start;
        let (mut enter, mut leave, mut normal) = (f32::NEG_INFINITY, f32::INFINITY, Vec3::ZERO);
        for axis in 0..3 {
            let (s, d, lo, hi) = (start[axis], delta[axis], low[axis], high[axis]);
            if d == 0.0 {
                if s <= lo || s >= hi {
                    return None;
                }
                continue;
            }
            let (a, b) = ((lo - s) / d, (hi - s) / d);
            let (near, far, side) = if a < b { (a, b, -1.0) } else { (b, a, 1.0) };
            if near > enter {
                enter = near;
                normal = Vec3::ZERO;
                normal[axis] = side;
            }
            leave = leave.min(far);
        }
        (enter < leave && enter < 1.0 && leave > 0.0).then_some((enter.max(0.0), normal))
    }

    impl Sweep for Boxes {
        fn sweep(&mut self, start: Vec3, end: Vec3, extent: Vec3, _flags: u32) -> Option<SweepHit> {
            self.solids
                .iter()
                .filter_map(|(low, high)| slab(start, end, *low - extent, *high + extent))
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(time, normal)| SweepHit::world(time, normal))
        }

        fn controller_call(&mut self, call: ControllerCall) {
            self.calls.push(call);
        }

        fn encroaching(&mut self, location: Vec3, extent: Vec3) -> Option<bool> {
            if !self.point_check {
                return None;
            }
            let (low, high) = (location - extent, location + extent);
            Some(self.solids.iter().any(|(a, b)| low.x < b.x && high.x > a.x && low.y < b.y && high.y > a.y && low.z < b.z && high.z > a.z))
        }
    }

    fn floor_box(until_x: f32) -> (Vec3, Vec3) {
        (Vec3::new(-1000.0, -1000.0, -100.0), Vec3::new(until_x, 1000.0, 0.0))
    }

    #[test]
    fn walk_move_moves_falls_climbs_and_stops_like_the_engine_test_move() {
        assert_eq!([TestMove::Stopped, TestMove::Moved, TestMove::Fell, TestMove::HitGoal].map(TestMove::code), [0, 1, 2, 5]);
        assert_eq!(hit_goal(true), TestMove::Stopped);
        assert_eq!(hit_goal(false), TestMove::HitGoal);
        let mut never = |_: &SweepHit| -> Option<TestMove> { None };
        let mut flat = Boxes::new(vec![floor_box(1000.0)], false);
        let mut walker = Walker::new(Vec3::new(0.0, 0.0, 22.15), 10.0, 20.0);
        assert_eq!(walk_move(&mut flat, &mut walker, Vec3::new(50.0, 0.0, 99.0), &mut never, 1.0), TestMove::Moved);
        assert!((walker.location.x - 50.0).abs() < 1e-3 && (walker.location.z - 22.0).abs() < 1e-3, "{:?}", walker.location);
        let mut ledge = Boxes::new(vec![floor_box(20.0)], false);
        let mut walker = Walker::new(Vec3::new(0.0, 0.0, 22.15), 10.0, 20.0);
        assert_eq!(walk_move(&mut ledge, &mut walker, Vec3::new(50.0, 0.0, 0.0), &mut never, 1.0), TestMove::Fell);
        assert!((walker.location.x - 50.0).abs() < 1e-3 && (walker.location.z - 22.15).abs() < 1e-3, "{:?}", walker.location);
        let mut step = Boxes::new(vec![floor_box(1000.0), (Vec3::new(30.0, -1000.0, -100.0), Vec3::new(1000.0, 1000.0, 5.0))], false);
        let mut walker = Walker::new(Vec3::new(0.0, 0.0, 22.15), 10.0, 20.0);
        assert_eq!(walk_move(&mut step, &mut walker, Vec3::new(50.0, 0.0, 0.0), &mut never, 1.0), TestMove::Moved);
        assert!(walker.location.x > 40.0 && (walker.location.z - 27.0).abs() < 1e-3, "{:?}", walker.location);
        let wall = (Vec3::new(30.0, -1000.0, -100.0), Vec3::new(1000.0, 1000.0, 500.0));
        let mut blocked = Boxes::new(vec![floor_box(1000.0), wall], false);
        let mut walker = Walker::new(Vec3::new(0.0, 0.0, 22.15), 10.0, 20.0);
        assert_eq!(walk_move(&mut blocked, &mut walker, Vec3::new(50.0, 0.0, 0.0), &mut never, 30.0), TestMove::Stopped);
        let mut walker = Walker::new(Vec3::new(0.0, 0.0, 22.15), 10.0, 20.0);
        let mut goal = |hit: &SweepHit| (hit.normal == Vec3::NEG_X).then_some(hit_goal(false));
        assert_eq!(walk_move(&mut blocked, &mut walker, Vec3::new(50.0, 0.0, 0.0), &mut goal, 1.0), TestMove::HitGoal);
    }

    #[test]
    fn check_slice_lifts_out_of_the_floor_and_lowers_from_the_ceiling() {
        let mut floor = Boxes::new(vec![floor_box(1000.0)], true);
        let mut location = Vec3::new(0.0, 0.0, 15.0);
        let extent = Vec3::new(10.0, 10.0, 20.0);
        assert_eq!(check_slice(&mut floor, &mut location, extent), Some(SliceCheck { fits: true, success: false }));
        assert!((location.z - 22.0).abs() < 1e-3, "{location:?}");
        let mut ceiling = Boxes::new(vec![(Vec3::new(-1000.0, -1000.0, 30.0), Vec3::new(1000.0, 1000.0, 100.0))], true);
        let mut location = Vec3::new(0.0, 0.0, 15.0);
        assert_eq!(check_slice(&mut ceiling, &mut location, extent), Some(SliceCheck { fits: true, success: false }));
        assert!((location.z - 10.0).abs() < 1e-3, "{location:?}");
        let pillar = (Vec3::new(-2.0, -2.0, -100.0), Vec3::new(2.0, 2.0, 100.0));
        let mut inside = Boxes::new(vec![pillar], true);
        let mut location = Vec3::new(0.0, 0.0, 15.0);
        assert_eq!(check_slice(&mut inside, &mut location, extent), Some(SliceCheck { fits: false, success: true }));
        let mut blind = Boxes::new(vec![floor_box(1000.0)], false);
        assert_eq!(check_slice(&mut blind, &mut location, extent), None);
    }

    #[test]
    fn find_spot_keeps_a_free_spot_and_shifts_away_from_a_wall_slice() {
        let extent = Vec3::new(10.0, 10.0, 20.0);
        let mut empty = Boxes::new(Vec::new(), true);
        let mut location = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(find_spot(&mut empty, extent, &mut location), Some(true));
        assert_eq!(location, Vec3::new(1.0, 2.0, 3.0));
        let mut wall = Boxes::new(vec![(Vec3::new(5.0, -1000.0, -1000.0), Vec3::new(1000.0, 1000.0, 1000.0))], true);
        let mut location = Vec3::ZERO;
        assert_eq!(find_spot(&mut wall, Vec3::ZERO, &mut location), Some(true));
        assert_eq!(find_spot(&mut wall, extent, &mut location), Some(true));
        assert!(location.x < -4.9 && location.x > -12.0 && location.y.abs() < 1e-3, "{location:?}");
        let mut blind = Boxes::new(Vec::new(), false);
        let mut location = Vec3::ZERO;
        assert_eq!(find_spot(&mut blind, extent, &mut location), None);
    }

    #[test]
    fn process_landed_respots_without_floor_and_stops_the_dead() {
        let ceiling = (Vec3::new(-1000.0, -1000.0, 35.0), Vec3::new(1000.0, 1000.0, 100.0));
        let mut world = Boxes::new(vec![ceiling], true);
        let mut walker = Walker::new(Vec3::new(0.0, 0.0, 15.0), 10.0, 20.0);
        walker.velocity = Vec3::new(100.0, 50.0, -10.0);
        let mut rolls = [0.25f32, 0.75].into_iter();
        let mut frand = || rolls.next().unwrap_or(0.5);
        assert_eq!(process_landed(&mut world, &mut walker, true, &mut frand), Landing::Respotted);
        assert!(walker.location.z < 15.0, "{:?}", walker.location);
        assert_eq!(walker.velocity, Vec3::new(85.0, 65.0, -10.0));
        let mut blind = Boxes::new(Vec::new(), false);
        let mut walker = Walker::new(Vec3::new(0.0, 0.0, 15.0), 10.0, 20.0);
        walker.acceleration = Vec3::new(30.0, 40.0, 0.0);
        assert_eq!(process_landed(&mut blind, &mut walker, true, &mut frand), Landing::Landed);
        assert!((walker.acceleration - Vec3::new(0.6, 0.8, 0.0)).length() < 1e-6);
        let mut floor = Boxes::new(vec![floor_box(1000.0)], true);
        let mut walker = Walker::new(Vec3::new(0.0, 0.0, 22.15), 10.0, 20.0);
        assert_eq!(process_landed(&mut floor, &mut walker, false, &mut frand), Landing::Stopped);
        assert_eq!(walker.location, Vec3::new(0.0, 0.0, 22.15));
    }

    #[test]
    fn falling_clears_just_teleported_and_notifies_the_jump_apex_once() {
        let mut world = Boxes::new(Vec::new(), false);
        let mut walker = Walker::new(Vec3::ZERO, 10.0, 20.0);
        walker.air_control = 0.0;
        walker.just_teleported = true;
        walker.notify_apex = true;
        walker.jump_over_wall = true;
        walker.velocity = Vec3::new(0.0, 0.0, 30.0);
        phys_falling(&mut world, &mut walker, 0.01, 0);
        assert!(world.calls.is_empty());
        assert!(walker.notify_apex && walker.jump_over_wall && !walker.just_teleported);
        for _ in 0..10 {
            phys_falling(&mut world, &mut walker, 0.01, 0);
        }
        assert_eq!(world.calls, vec![ControllerCall::NotifyJumpApex]);
        assert!(!walker.notify_apex && !walker.jump_over_wall);
    }

    #[test]
    fn falling_into_water_swims_unless_the_associated_pawn_blocks_it() {
        let mut world = Boxes::new(Vec::new(), false);
        let mut walker = Walker::new(Vec3::ZERO, 10.0, 20.0);
        walker.air_control = 0.0;
        walker.volume.water = true;
        walker.just_teleported = true;
        assert!(matches!(phys_falling(&mut world, &mut walker, 0.01, 0), Outcome::Swimming { .. }));
        let mut walker = Walker::new(Vec3::ZERO, 10.0, 20.0);
        walker.air_control = 0.0;
        walker.volume.water = true;
        walker.head_in_water = true;
        walker.water_entry_blocked = true;
        assert!(matches!(phys_falling(&mut world, &mut walker, 0.01, 0), Outcome::Falling { .. }));
    }
}
