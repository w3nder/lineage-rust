mod geometry;
mod beams;
pub mod ribbon;

use glam::{Mat3, Vec3};
use killer_assets::scene::emitters::EmitterAsset;
use ue2_level::{DrawStyle, Emitter, Range, SpriteEmitter};

pub use geometry::{beam_vertices, mesh_bounds, mesh_matrix, sprite_vertices};

#[derive(Clone, Debug, Default)]
pub struct Particle {
    pub vertex_frame: f32,
    pub location: Vec3,
    pub old_location: Vec3,
    pub start_location: Vec3,
    pub velocity: Vec3,
    pub acceleration: Vec3,
    pub start_size: Vec3,
    pub size: Vec3,
    pub start_spin: Vec3,
    pub spins: Vec3,
    pub center: Vec3,
    pub revolutions: Vec3,
    pub revolution_multiplier: Vec3,
    pub velocity_multiplier: Vec3,
    pub color_multiplier: Vec3,
    pub color: [u8; 4],
    pub time: f32,
    pub lifetime: f32,
    pub inverse_mass: f32,
    pub flags: u32,
    pub hits: u32,
    pub subdivision: i32,
    pub rotation: [i32; 3],
    pub bone: usize,
    pub old_mesh_location: Vec3,
    pub beam: Vec<Vec3>,
}

impl Particle {
    pub fn alive(&self) -> bool { self.flags & 1 != 0 }
    pub fn relative_time(&self) -> f32 { if self.lifetime == 0.0 { 0.0 } else { (self.time / self.lifetime).clamp(0.0, 1.0) } }
}

#[derive(Debug, Default)]
pub struct EmitterState {
    pub vertex_animation: ue2_assets::VertexAnimation,
    pub particles: Vec<Particle>,
    pub active: usize,
    pub index: usize,
    pub fraction: f32,
    pub other_index: usize,
    pub mesh_index: usize,
    pub delay: f32,
    pub reset_time: f32,
    pub warmed: bool,
    pub all_dead: bool,
    pub inactive: bool,
    pub inactive_time: f32,
    pub elapsed: f32,
    pub noise_time: f32,
    pub trigger_count: u32,
    pub deferred: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Collision { pub location: Vec3, pub normal: Vec3 }

#[derive(Clone, Copy, Debug)]
pub struct ActorForce { pub kind: u8, pub location: Vec3, pub velocity: Vec3, pub radius: f32, pub scale: f32 }

pub trait ParticleWorld {
    fn trace(&mut self, _from: Vec3, _to: Vec3, _flags: u32) -> Option<Collision> { None }
    fn joints(&self, _actor: &str) -> &[Vec3] { &[] }
    fn trigger(&mut self, _actor: &str) {}
    fn forces(&self) -> &[ActorForce] { &[] }
}

pub struct EmptyWorld;
impl ParticleWorld for EmptyWorld {}

pub struct ParticleSystem {
    pub actor: Emitter,
    pub states: Vec<EmitterState>,
    pub random: u32,
    pub seeded_random: u32,
    pub offset: Vec3,
    pub reset_time: f32,
    pub killed: bool,
    pub destroyed: bool,
}

pub fn random_range(random: &mut u32, range: Range) -> f32 {
    let fraction = crate::coronas::app_frand(random);
    (f64::from(range[1]) + f64::from(fraction) * (f64::from(range[0]) - f64::from(range[1]))) as f32
}

pub fn app_srand(random: &mut u32) -> f32 {
    *random = random.wrapping_mul(0x0bb38435).wrapping_add(0x3619636b);
    f32::from_bits(0x3f800000 | (*random & 0x7fffff)) - 1.0
}

pub fn random_vector(random: &mut u32, ranges: [Range; 3]) -> Vec3 {
    let y = random_range(random, ranges[1]);
    let x = random_range(random, ranges[0]);
    let z = random_range(random, ranges[2]);
    Vec3::new(x, y, z)
}

pub fn safe_normal(v: Vec3) -> Vec3 {
    let length = v.as_dvec3().length_squared();
    if length < 1e-8 { Vec3::ZERO } else { v * (1.0 / length.sqrt()) as f32 }
}

fn random_normal(random: &mut u32) -> Vec3 {
    loop {
        let z = 1.0 - 2.0 * crate::coronas::app_frand(random);
        let y = 1.0 - 2.0 * crate::coronas::app_frand(random);
        let x = 1.0 - 2.0 * crate::coronas::app_frand(random);
        let v = Vec3::new(x, y, z);
        if v.length_squared() > 0.1 && v.length_squared() <= 1.0 { return safe_normal(v); }
    }
}

fn sine(units: i32) -> f32 { (((units >> 2) & 0x3fff) as f64 * std::f64::consts::TAU / 16384.0).sin() as f32 }

pub fn rotator_matrix(rotator: [i32; 3]) -> Mat3 {
    let [sp, sy, sr] = rotator.map(sine);
    let [cp, cy, cr] = rotator.map(|v| sine(v.wrapping_add(16384)));
    Mat3::from_cols(Vec3::new(cp * cy, cp * sy, sp), Vec3::new(sr * sp * cy - cr * sy, sr * sp * sy + cr * cy, -sr * cp), Vec3::new(-(cr * sp * cy + sr * sy), cy * sr - cr * sp * sy, cr * cp))
}

pub fn rotate_axis(v: Vec3, angle: i32, axis: Vec3) -> Vec3 {
    let sin = sine(angle);
    let cos = sine(angle.wrapping_add(16384));
    v * cos + axis.cross(v) * sin + axis * axis.dot(v) * (1.0 - cos)
}

fn normal_rotation(normal: Vec3, positive_z: bool) -> Mat3 {
    let units = 32768.0 / std::f32::consts::PI;
    let pitch = (normal.z.atan2(normal.truncate().length()) * units) as i32 - if positive_z { 16384 } else { 0 };
    rotator_matrix([pitch, (normal.y.atan2(normal.x) * units) as i32, 0])
}

fn spawn_rotation(e: &SpriteEmitter, actor: &Emitter, position: bool) -> Mat3 {
    match e.use_rotation_from {
        1 => rotator_matrix(actor.rotation) * rotator_matrix(e.rotation_offset),
        2 => rotator_matrix(e.rotation_offset),
        3 => normal_rotation(Vec3::from(e.rotation_normal), position && e.effect_axis == 1),
        _ => Mat3::IDENTITY,
    }
}

pub fn curve<const N: usize>(points: impl IntoIterator<Item = (f32, [f32; N])>, time: f32, initial: [f32; N]) -> Option<[f32; N]> {
    let (mut previous_time, mut previous) = (0.0, initial);
    for (at, value) in points {
        if time <= at {
            let fraction = if at == 0.0 { 1.0 } else { (time - previous_time) / (at - previous_time) };
            return Some(std::array::from_fn(|i| previous[i] + fraction * (value[i] - previous[i])));
        }
        previous_time = at;
        previous = value;
    }
    None
}

fn curve_time(repeats: f32, relative: f32) -> f32 { let t = (repeats + 1.0) * relative; t - t.floor() }
fn byte(value: f32) -> u8 { value as i32 as u8 }

pub fn scales(e: &SpriteEmitter, p: &mut Particle) {
    let rel = p.relative_time();
    let size = if !e.use_size_scale { 1.0 } else if e.use_regular_size_scale { 1.0 / (p.time + 1.0) }
        else if p.lifetime != 0.0 { curve(e.size_scale.iter().map(|v| (v[0], [v[1]])), curve_time(e.size_scale_repeats, rel), [1.0]).unwrap_or([1.0])[0] } else { 1.0 };
    p.size = p.start_size * size;
    if p.lifetime != 0.0 {
        if e.use_velocity_scale && let Some(value) = curve(e.velocity_scale.iter().copied(), curve_time(e.velocity_scale_repeats, rel), [1.0; 3]) { p.velocity_multiplier = Vec3::from(value); }
        if e.use_revolution_scale && let Some(value) = curve(e.revolution_scale.iter().copied(), curve_time(e.revolution_scale_repeats, rel), [1.0; 3]) { p.revolution_multiplier = Vec3::from(value); }
    }
    let mut color = [255; 4];
    if e.use_color_scale && p.lifetime != 0.0 && let Some(c) = curve(e.color_scale.iter().map(|(t, c)| (*t, c.map(f32::from))), curve_time(e.color_scale_repeats, rel), [255.0; 4]) {
        color = c.map(byte);
        if e.draw_style != DrawStyle::AlphaBlend { color[3] = 255; }
    }
    for (i, channel) in color[..3].iter_mut().enumerate() { *channel = byte(f32::from(*channel) * p.color_multiplier[i]); }
    let out = e.fade_out && p.time > e.fade_out_start && p.lifetime != e.fade_out_start;
    let fade = if out { Some((e.fade_out_factor, (p.time - e.fade_out_start) / (p.lifetime - e.fade_out_start))) }
        else if e.fade_in && p.time < e.fade_in_end && e.fade_in_end != 0.0 { Some((e.fade_in_factor, (e.fade_in_end - p.time) / e.fade_in_end)) } else { None };
    if let Some((factor, t)) = fade {
        match e.draw_style {
            DrawStyle::Modulated => color = [127, 127, 127, 255u8.wrapping_sub(byte(255.0 * t * factor[3]))],
            DrawStyle::AlphaBlend => color[3] = byte((f32::from(color[3]) - factor[3] * t * 255.0).max(0.0)),
            _ => for i in 0..4 { color[i] = byte(f32::from(color[i]) - f32::from(color[i]) * factor[i] * t); },
        }
    }
    if e.opacity < 1.0 && e.draw_style != DrawStyle::Regular {
        if matches!(e.draw_style, DrawStyle::AlphaBlend | DrawStyle::Modulated | DrawStyle::AlphaModulate) { color[3] = byte(f32::from(color[3]) * e.opacity); }
        else { for channel in &mut color[..3] { *channel = byte(f32::from(*channel) * e.opacity); } }
    }
    p.color = color;
}

impl ParticleSystem {
    pub fn new(actor: &Emitter, seed: u32) -> Self {
        let mut system = Self { actor: actor.clone(), states: Vec::new(), random: seed, seeded_random: seed, offset: Vec3::ZERO, reset_time: 0.0, killed: false, destroyed: false };
        system.offset = random_vector(&mut system.random, actor.global_offset);
        system.reset_time = random_range(&mut system.random, actor.time_till_reset);
        for e in &actor.sprites {
            system.states.push(EmitterState { particles: vec![Particle::default(); e.max_particles as usize], delay: random_range(&mut system.random, e.initial_delay), reset_time: random_range(&mut system.random, e.auto_reset_time), ..Default::default() });
        }
        system
    }

    pub fn kill(&mut self) {
        self.killed = true;
        self.actor.auto_destroy = true;
        self.actor.auto_reset = false;
        for e in &mut self.actor.sprites { e.respawn_dead_particles = false; e.particles_per_second = 0.0; e.initial_particles_per_second = 0.0; e.auto_reset = false; e.auto_destroy = false; }
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        for e in self.actor.sprites.iter_mut().filter(|e| !e.class.is_empty()) {
            e.disabled = disabled;
        }
    }

    pub fn l2_event(&mut self, cmd: u8, on: i32) {
        if cmd == ue2_level::LEC_SHOW {
            self.set_disabled(on == 0);
        }
    }

    pub fn trigger(&mut self) {
        for i in 0..self.states.len() {
            let e = &self.actor.sprites[i];
            if e.class.is_empty() || !e.disabled {
                continue;
            }
            let (trigger_disabled, reset_on_trigger, range) = (e.trigger_disabled, e.reset_on_trigger, e.spawn_on_trigger);
            if trigger_disabled {
                self.actor.sprites[i].disabled = false;
            }
            if reset_on_trigger {
                self.reset(i);
            }
            let count = random_range(&mut self.random, range) + self.states[i].trigger_count as i32 as f32;
            let state = &mut self.states[i];
            state.trigger_count = count as i32 as u32;
            if state.trigger_count != 0 {
                state.all_dead = false;
                state.inactive = false;
            }
        }
    }

    fn reset(&mut self, index: usize) {
        let e = &self.actor.sprites[index];
        let state = &mut self.states[index];
        state.active = 0;
        state.index = 0;
        state.other_index = 0;
        state.warmed = false;
        state.all_dead = false;
        state.delay = random_range(&mut self.random, e.initial_delay);
        state.reset_time = random_range(&mut self.random, e.auto_reset_time);
    }

    fn other(&self, index: i32, cursor: usize) -> Option<(&SpriteEmitter, &Particle)> {
        if index < 0 || self.states.is_empty() { return None; }
        let index = (index as usize).min(self.states.len() - 1);
        let state = &self.states[index];
        if state.active == 0 { return None; }
        Some((&self.actor.sprites[index], &state.particles[cursor % state.active]))
    }

    #[expect(clippy::too_many_arguments, reason = "SpawnParticle nativo mais a fonte de assets e o mundo")]
    pub fn spawn(&mut self, emitter: usize, slot: usize, age: f32, flags: u32, local: Option<Vec3>, asset: &EmitterAsset, world: &mut impl ParticleWorld) {
        let e = self.actor.sprites[emitter].clone();
        if self.killed || e.max_particles == 0 || e.lifetime[1] <= 0.0 || slot >= self.states[emitter].particles.len() { return; }
        let owner = Vec3::from(self.actor.location);
        let rotation = rotator_matrix(self.actor.rotation);
        let mut p = Particle { location: Vec3::from(e.start_offset), velocity: random_vector(&mut self.random, e.start_velocity), color_multiplier: Vec3::ONE, ..Default::default() };
        if matches!(e.start_location_shape, 0 | 3) { p.location += random_vector(&mut self.random, e.start_location); }
        if matches!(e.start_location_shape, 1 | 3) { p.location += random_normal(&mut self.random) * random_range(&mut self.random, e.sphere_radius); }
        if matches!(e.start_location_shape, 2 | 3) {
            let polar = random_vector(&mut self.random, e.start_location_polar);
            let (a, b) = (polar.x.to_radians(), polar.y.to_radians());
            p.location += Vec3::new(a.cos() * b.sin(), a.sin() * b.sin(), b.cos()) * polar.z;
        }
        let vertices = &asset.parts[emitter].spawn_vertices;
        if e.mesh_spawning != 0 && !vertices.is_empty() {
            let max = vertices.len().saturating_sub(1);
            if max > 0 {
                let index = if e.mesh_spawning == 1 { let index = self.states[emitter].mesh_index; self.states[emitter].mesh_index += 1; index % max }
                    else { (app_srand(&mut self.seeded_random) * max as f32) as usize % max };
                let v = &vertices[index];
                let normal = Vec3::from(v.normal);
                if e.spawn_only_normal && normal.dot(safe_normal(Vec3::from(e.mesh_normal))) < 1.0 - 2.0 * random_range(&mut self.random, e.mesh_normal_threshold) { self.states[emitter].particles[slot].flags &= !1; return; }
                let mut scale = random_vector(&mut self.random, e.mesh_scale_range);
                if e.uniform_mesh_scale { scale = Vec3::splat(scale.x); }
                p.location += Vec3::from(v.position) * scale;
                if e.velocity_from_mesh {
                    let mut scale = random_vector(&mut self.random, e.velocity_scale_range);
                    if e.uniform_velocity_scale { scale = Vec3::splat(scale.x); }
                    p.velocity += normal * scale;
                }
                if e.use_color_from_mesh { p.color_multiplier = Vec3::from_slice(&v.color); }
            }
        }
        let joints = e.skeletal_actor.as_deref().map(|actor| world.joints(actor)).unwrap_or(&[]);
        let bone = (random_range(&mut self.random, e.relative_bone_index) * (joints.len() as f32 - 1.0)) as i32;
        p.bone = bone.max(0) as usize;
        if e.use_skeletal_location != 0 && let Some(position) = joints.get(p.bone.min(joints.len().saturating_sub(1))) {
            p.old_mesh_location = *position * Vec3::from(e.skeletal_scale);
            p.location += p.old_mesh_location;
        }
        self.states[emitter].other_index += 1;
        let cursor = self.states[emitter].other_index;
        if let Some((other, particle)) = self.other(e.add_location_from_other, cursor) {
            p.location += particle.location;
            if matches!(other.coordinate_system, 0 | 4) { p.location -= owner; }
        }
        p.location = spawn_rotation(&e, &self.actor, true) * p.location;
        p.center = random_vector(&mut self.random, e.revolution_center);
        p.revolutions = random_vector(&mut self.random, e.revolutions_per_second);
        p.acceleration = Vec3::from(e.acceleration);
        if matches!(e.coordinate_system, 0 | 3) {
            if local.is_none() { p.location += owner + Vec3::from(self.actor.velocity) * age; }
            p.center += owner;
        }
        if matches!(e.coordinate_system, 3 | 4) { p.rotation = self.actor.rotation; }
        if e.coordinate_system == 4 {
            p.center += owner;
            if e.independent_spray_accel { p.acceleration = rotation.transpose() * p.acceleration; }
        }
        p.location += local.unwrap_or(self.offset);
        p.old_location = p.location;
        p.start_location = p.location;
        p.color_multiplier *= random_vector(&mut self.random, e.color_multiplier);
        p.lifetime = random_range(&mut self.random, e.lifetime);
        p.time = random_range(&mut self.random, e.initial_time) + age;
        p.flags = flags | 1;
        let mass = random_range(&mut self.random, e.start_mass);
        p.inverse_mass = if mass == 0.0 { 0.0 } else { 1.0 / mass };
        p.start_size = random_vector(&mut self.random, e.start_size);
        if e.uniform_size || (e.class == "SpriteEmitter" && e.direction == 0) { p.start_size = Vec3::splat(p.start_size.x); }
        p.size = p.start_size;
        p.velocity += p.acceleration * age;
        p.velocity_multiplier = Vec3::ONE;
        p.revolution_multiplier = Vec3::ONE;
        p.velocity = spawn_rotation(&e, &self.actor, false) * p.velocity;
        if e.velocity_direction != 0 {
            let dir = safe_normal(if matches!(e.coordinate_system, 1 | 4) { p.location } else { owner - p.location });
            match e.velocity_direction { 1 => p.velocity *= -dir, 2 => p.velocity *= dir, 3 => p.velocity += dir * random_range(&mut self.random, e.start_velocity_radial), _ => {} }
        }
        if let Some((_, other)) = self.other(e.add_velocity_from_other, cursor) {
            let velocity = other.velocity;
            p.velocity += velocity * random_vector(&mut self.random, e.add_velocity_multiplier);
        }
        if e.coordinate_system == 4 {
            p.location = rotation * p.location;
            if local.is_none() { p.location += owner + Vec3::from(self.actor.velocity) * age; }
            p.velocity = rotation * p.velocity;
            p.acceleration = if e.independent_spray_accel { Vec3::from(e.acceleration) } else { rotation * Vec3::from(e.acceleration) };
        }
        p.location += p.velocity * age;
        p.start_spin = random_vector(&mut self.random, e.start_spin) * 65535.0;
        p.spins = random_vector(&mut self.random, e.spins_per_second);
        for i in 0..3 { if crate::coronas::app_frand(&mut self.random) < e.spin_ccw_or_cw[i] { p.spins[i] *= -1.0; } }
        p.spins *= 65535.0;
        p.subdivision = if e.random_subdivision {
            let (start, end) = if e.subdivision_range[1] == 0 { (0, e.subdivisions[0].saturating_mul(e.subdivisions[1])) } else { (e.subdivision_range[0], e.subdivision_range[1]) };
            (crate::coronas::app_frand(&mut self.random) * (end as f32 - start as f32)) as i32 + start as i32
        } else { -1 };
        if p.lifetime > 0.0 && p.time > p.lifetime {
            let over = p.time % p.lifetime;
            self.spawn(emitter, slot, over, 0, None, asset, world);
            return;
        }
        self.states[emitter].particles[slot] = p;
        self.states[emitter].all_dead = false;
        if e.beam.is_some() { self.spawn_beam(emitter, slot, age, asset, world); }
        if let Some(mesh) = &asset.parts[emitter].vertex_mesh {
            self.states[emitter].vertex_animation.play(&mesh.mesh.sequences, &mesh.name, self.actor.speed_rate, 0.0, true, Vec3::from(self.actor.velocity).length());
        }
    }

    fn advance(&mut self, emitter: usize) {
        let state = &mut self.states[emitter];
        if state.particles.is_empty() { return; }
        state.active = state.active.max(state.index + 1);
        state.index = (state.index + 1) % state.particles.len();
    }

    pub fn spawn_particles(&mut self, emitter: usize, rate: f32, dt: f32, asset: &EmitterAsset, world: &mut impl ParticleWorld) {
        if rate <= 0.0 || !rate.is_finite() { self.states[emitter].fraction = 0.0; return; }
        let input = self.states[emitter].fraction;
        let count = input + rate * dt;
        let mut n = (count.floor().max(0.0) as usize).min(self.states[emitter].particles.len());
        self.states[emitter].fraction = count - count.floor();
        if self.states[emitter].trigger_count != 0 { n = n.min(self.states[emitter].trigger_count as usize); self.states[emitter].trigger_count -= n as u32; }
        for i in 0..n {
            let age = dt + input / rate - (i + 1) as f32 / rate;
            self.spawn(emitter, self.states[emitter].index, age, 4, None, asset, world);
            self.advance(emitter);
        }
    }

    pub fn tick(&mut self, dt: f32, visible: bool, asset: &EmitterAsset, world: &mut impl ParticleWorld) {
        if self.destroyed || !dt.is_finite() || dt < 0.0 { return; }
        let dt = dt * self.actor.speed_rate;
        if self.actor.rotate { for i in 0..3 { self.actor.rotation[i] = self.actor.rotation[i].wrapping_add((self.actor.rotation_per_second[i] as f32 * dt) as i32); } }
        let (mut all_dead, mut enabled, mut delayed) = (true, false, false);
        for i in 0..self.states.len() {
            let e = self.actor.sprites[i].clone();
            if e.disabled || e.class.is_empty() { continue; }
            enabled = true;
            let state = &mut self.states[i];
            if state.delay != 0.0 { state.delay -= dt; if state.delay > 0.0 { delayed = true; continue; } state.delay = 0.0; }
            if !state.warmed && e.warmup_ticks_per_second != 0.0 && e.relative_warmup_time != 0.0 {
                let steps = (((e.lifetime[0] + e.lifetime[1]) * 0.5) * e.relative_warmup_time * e.warmup_ticks_per_second) as u32;
                for _ in 0..steps { self.update_particles(i, 1.0 / e.warmup_ticks_per_second, asset, world); }
                self.states[i].warmed = true;
            }
            if !self.states[i].all_dead {
                let state = &mut self.states[i];
                state.inactive_time = if state.inactive { state.inactive_time + dt } else { 0.0 };
                if e.seconds_before_inactive == 0.0 || state.inactive_time <= e.seconds_before_inactive { self.update_particles(i, dt, asset, world); }
                self.states[i].inactive = !visible;
                all_dead = false;
                if e.use_actor_forces { for force in world.forces() { self.handle_actor_force(i, *force, dt); } }
            } else if !e.auto_reset || self.actor.auto_reset {
                if e.auto_destroy && !self.actor.auto_destroy { self.actor.sprites[i].disabled = true; }
            } else {
                self.states[i].reset_time -= dt;
                if self.states[i].reset_time <= 0.0 { self.reset(i); }
            }
        }
        if all_dead && enabled && !delayed && self.actor.auto_destroy && !self.states.is_empty() { self.destroyed = true; }
        if all_dead && self.actor.auto_reset && !self.actor.auto_destroy {
            self.reset_time -= dt;
            if self.reset_time <= 0.0 {
                self.offset = random_vector(&mut self.random, self.actor.global_offset);
                self.reset_time = random_range(&mut self.random, self.actor.time_till_reset);
                for i in 0..self.states.len() { self.reset(i); }
            }
        }
    }

    pub fn update_particles(&mut self, emitter: usize, dt: f32, asset: &EmitterAsset, world: &mut impl ParticleWorld) {
        let e = self.actor.sprites[emitter].clone();
        self.states[emitter].elapsed += dt;
        self.states[emitter].noise_time += dt;
        let mut rate = if self.states[emitter].active < e.max_particles as usize {
            if e.automatic_initial_spawning { e.max_particles as f32 / ((e.lifetime[0] + e.lifetime[1]) * 0.5) } else { e.initial_particles_per_second }
        } else { e.particles_per_second };
        if self.states[emitter].trigger_count != 0 { rate += e.spawn_on_trigger_pps; }
        if !self.killed {
            if rate > 0.0 { self.spawn_particles(emitter, rate, dt, asset, world); }
            let deferred = self.states[emitter].deferred.min(e.max_particles);
            for _ in 0..deferred { self.spawn(emitter, self.states[emitter].index, 0.0, 0, None, asset, world); self.advance(emitter); }
            self.states[emitter].deferred = 0;
        }
        let mut inactive = 0;
        for slot in 0..self.states[emitter].active {
            let mut p = self.states[emitter].particles[slot].clone();
            if !p.alive() { inactive += 1; continue; }
            let mut moving = p.flags & 2 == 0 || e.coordinate_system == 1;
            if p.flags & 4 != 0 { moving = false; p.flags &= !4; } else { p.time += dt; }
            if p.time > p.lifetime {
                if e.respawn_dead_particles {
                    let time = p.time - p.lifetime + random_range(&mut self.random, e.initial_time);
                    let over = if p.lifetime == 0.0 { 0.0 } else { time % p.lifetime };
                    self.spawn(emitter, slot, over, 0, None, asset, world);
                    p = self.states[emitter].particles[slot].clone();
                } else { p.flags &= !1; self.states[emitter].particles[slot] = p; inactive += 1; continue; }
            } else if moving {
                p.velocity += p.acceleration * dt;
                p.old_location = p.location;
                p.location += p.velocity * p.velocity_multiplier * dt;
                if e.use_skeletal_location == 2 && let Some(joint) = e.skeletal_actor.as_deref().and_then(|a| world.joints(a).get(p.bone)) {
                    let next = *joint * Vec3::from(e.skeletal_scale);
                    p.location += next - p.old_mesh_location;
                    p.old_mesh_location = next;
                }
                if e.use_revolution {
                    let mut d = p.location - p.center;
                    for axis in 0..3 { d = rotate_axis(d, (p.revolutions[axis] * p.revolution_multiplier[axis] * dt * 65536.0) as i32, [Vec3::X, Vec3::Y, Vec3::Z][axis]); }
                    p.location = p.center + d;
                }
            }
            let mut hit = None;
            if moving && e.coordinate_system != 1 {
                if e.use_collision {
                    let extent = safe_normal(p.location - p.old_location) * Vec3::from(e.extent_multiplier) * p.size;
                    hit = world.trace(p.old_location, p.location + extent, 0x84).map(|mut h| { h.location -= extent; h });
                }
                if e.use_collision_planes && hit.is_none() {
                    for plane in &e.collision_planes {
                        let normal = Vec3::from_slice(plane);
                        let a = p.old_location.dot(normal) - plane[3];
                        let b = p.location.dot(normal) - plane[3];
                        if a.signum() != b.signum() && a != b { hit = Some(Collision { location: p.old_location.lerp(p.location, a / (a - b)), normal }); break; }
                    }
                }
                if let Some(hit) = hit {
                    if !e.use_max_collisions || (p.hits + 1) < random_range(&mut self.random, e.max_collisions).floor() as u32 {
                        p.velocity -= Vec3::from(e.acceleration) * dt * 0.5;
                        p.velocity = (p.velocity - 2.0 * hit.normal * hit.normal.dot(p.velocity)) * random_vector(&mut self.random, e.damping_factor);
                        if e.damp_rotation { p.spins *= random_vector(&mut self.random, e.rotation_damping_factor); }
                        p.location = hit.location + hit.normal * 0.01;
                        p.old_location = p.location;
                        p.hits += 1;
                    } else if e.respawn_dead_particles { self.spawn(emitter, slot, dt * 0.5, 0, None, asset, world); p = self.states[emitter].particles[slot].clone(); }
                    else { p.flags &= !1; self.states[emitter].particles[slot] = p; inactive += 1; continue; }
                    if e.spawn_from_other >= 0 && dt != 0.0 && !self.states.is_empty() {
                        let other = (e.spawn_from_other as usize).min(self.states.len() - 1);
                        for _ in 0..e.spawn_amount {
                            let index = self.states[other].index;
                            self.spawn(other, index, dt, 0, Some(hit.location + hit.normal * 0.01), asset, world);
                            if e.use_spawned_velocity_scale && let Some(p) = self.states[other].particles.get_mut(index) { p.velocity += hit.normal * random_vector(&mut self.random, e.spawned_velocity_scale); }
                            self.advance(other);
                        }
                    }
                }
            }
            scales(&e, &mut p);
            for axis in 0..3 { if e.max_abs_velocity[axis] != 0.0 { p.velocity[axis] = p.velocity[axis].max(-e.max_abs_velocity[axis]).min(e.max_abs_velocity[axis]); } }
            let loss = random_vector(&mut self.random, e.velocity_loss);
            if e.coordinate_system == 4 && !e.independent_spray_accel { let r = rotator_matrix(self.actor.rotation); p.velocity -= r * ((r.transpose() * p.velocity) * loss * dt); }
            else { p.velocity -= p.velocity * loss * dt; }
            if hit.is_some() && e.min_squared_velocity != 0.0 && p.velocity.length_squared() < e.min_squared_velocity { p.flags |= 2; }
            self.states[emitter].particles[slot] = p;
        }
        self.states[emitter].all_dead = (inactive >= e.max_particles as usize || self.states[emitter].active == inactive) && rate == 0.0 && !e.respawn_dead_particles;
        if let Some(mesh) = &asset.parts[emitter].vertex_mesh {
            let state = &mut self.states[emitter];
            for p in &mut state.particles[..state.active] {
                state.vertex_animation.frame = p.vertex_frame;
                state.vertex_animation.update(&mesh.mesh.sequences, dt, Vec3::from(self.actor.velocity).length(), true, |_, _| true);
                p.vertex_frame = state.vertex_animation.frame;
            }
        }
    }

    pub fn handle_actor_force(&mut self, emitter: usize, force: ActorForce, dt: f32) {
        if force.kind != 1 || force.radius == 0.0 { return; }
        let radius_squared = force.radius * force.radius;
        let state = &mut self.states[emitter];
        for p in state.particles[..state.active].iter_mut().filter(|p| p.alive() && p.inverse_mass != 0.0) {
            let offset = p.location - force.location;
            let distance_squared = offset.length_squared();
            if distance_squared >= radius_squared { continue; }
            let mut direction = safe_normal(offset);
            if direction.dot(force.velocity) <= 0.0 { direction = -direction; }
            let speed = force.velocity.length() - direction.dot(p.velocity);
            let strength = speed * force.scale * dt;
            if speed > 0.0 && strength > 0.0 {
                p.velocity += direction * strength * distance_squared / radius_squared;
                p.flags &= !2;
            }
        }
    }
}
