use glam::Vec3;
use std::sync::Arc;
use killer_assets::client::Client;
use killer_assets::collision::resources::Resources;
use killer_assets::collision::world::{Spawned, Target, View, physics_volume_at, trace_spawned};
use killer_assets::pickup::TraceQuery;
use killer_world::walk::{ControllerCall, HitActor, Sweep, SweepHit, WORLD_FLAGS};

pub struct WorldSweep<'a> {
    pub client: &'a mut Client,
    pub resources: &'a mut Resources,
    pub views: &'a [View],
    pub targets: Vec<Target>,
    pub spawned: Vec<Spawned>,
    pub error: Option<String>,
    pub current: i32,
    pub calls: Vec<(i32, ControllerCall)>,
}

impl<'a> WorldSweep<'a> {
    pub fn new(client: &'a mut Client, resources: &'a mut Resources, views: &'a [View]) -> Self {
        Self { client, resources, views, targets: Vec::new(), spawned: Vec::new(), error: None, current: 0, calls: Vec::new() }
    }
}

fn hit_actor(selection: &killer_assets::collision::world::Selection) -> HitActor {
    if selection.target.is_some() {
        HitActor::Pawn
    } else {
        HitActor::Other { immediately_stop: selection.immediately_stop, pushable_mass: selection.pushable_mass }
    }
}

impl Sweep for WorldSweep<'_> {
    fn water_surface_crossed(&mut self, from: Vec3, to: Vec3) -> bool {
        match killer_assets::collision::world::water_surface_crossed(self.client, self.resources, self.views, from, to) {
            Ok(crossed) => crossed,
            Err(error) => {
                self.error.get_or_insert(error);
                false
            }
        }
    }

    fn sweep(&mut self, start: Vec3, end: Vec3, extent: Vec3, flags: u32) -> Option<SweepHit> {
        self.line_check(start, end, extent, flags).map(|(hit, _)| hit)
    }

    fn line_check(&mut self, start: Vec3, end: Vec3, extent: Vec3, _flags: u32) -> Option<(SweepHit, Vec3)> {
        let query = TraceQuery { start, end, extent, flags: WORLD_FLAGS };
        match trace_spawned(self.client, self.resources, self.views, query, &self.targets, &self.spawned) {
            Ok(selection) => selection.map(|selection| (SweepHit { time: selection.hit.time, normal: selection.hit.normal, actor: hit_actor(&selection) }, selection.hit.position)),
            Err(error) => {
                self.error.get_or_insert(error);
                None
            }
        }
    }

    fn encroaching(&mut self, location: Vec3, extent: Vec3) -> Option<bool> {
        match killer_assets::collision::world::point_check(self.client, self.resources, self.views, &self.spawned, location, extent) {
            Ok(blocked) => Some(blocked),
            Err(error) => {
                self.error.get_or_insert(error);
                None
            }
        }
    }

    fn controller_call(&mut self, call: ControllerCall) {
        self.calls.push((self.current, call));
    }
}

pub fn camera_hit(client: &mut Client, resources: &mut Resources, views: &[View], spawned: &[Spawned], from: Vec3, to: Vec3, extent: f32) -> Result<Option<crate::trace::Hit>, String> {
    let query = TraceQuery { start: from, end: to, extent: Vec3::splat(extent), flags: WORLD_FLAGS };
    let hit = trace_spawned(client, resources, views, query, &[], spawned)?;
    Ok(hit.map(|selection| crate::trace::Hit {
        distance: selection.hit.time * (to - from).length(),
        terrain: selection.surface == Some(killer_assets::collision::world::Surface::Terrain),
    }))
}

fn packet_coords(v: Vec3) -> [i32; 3] {
    [v.x.round() as i32, v.y.round() as i32, v.z.round() as i32]
}

pub fn controller_commands(world: &mut killer_world::World, calls: &[(i32, ControllerCall)]) -> (Vec<killer_world::NetCommand>, bool) {
    use killer_world::NetCommand;
    let mut commands = Vec::new();
    let mut stopped = false;
    for &(object_id, call) in calls {
        let Some(me) = world.me.as_mut().filter(|me| me.object_id == object_id) else { continue };
        let heading = me.heading;
        let on_vehicle = me.vehicle.is_some();
        let base = me.vehicle.and_then(|id| world.vehicles.get(&id).map(|vehicle| (id, *vehicle)));
        match call {
            ControllerCall::CannotMoveAnymore { location, from_hit_wall } => match base {
                Some((vehicle_id, vehicle)) => commands.push(NetCommand::CannotMoveAnymoreInVehicle {
                    vehicle_id,
                    position: packet_coords(killer_world::world::vehicle_local(&vehicle, location)),
                    heading,
                }),
                None if !from_hit_wall => commands.push(NetCommand::CannotMoveAnymore { position: packet_coords(location), heading }),
                None => {}
            },
            ControllerCall::NotifyJumpApex => {}
            ControllerCall::Stop { location } => {
                killer_world::movement::stop(me);
                stopped = true;
                match (on_vehicle, base) {
                    (false, _) => commands.push(NetCommand::Validate { position: packet_coords(location), heading }),
                    (true, Some((vehicle_id, vehicle))) => commands.push(NetCommand::ValidateInVehicle {
                        vehicle_id,
                        position: packet_coords(killer_world::world::vehicle_local(&vehicle, location)),
                        heading,
                    }),
                    (true, None) => {}
                }
            }
        }
    }
    (commands, stopped)
}

pub const MAX_PLANAR_SPEED: f32 = 2000.0;

pub struct WalkPhysics {
    root: std::path::PathBuf,
    client: Option<Client>,
    resources: Resources,
    state: WalkState,
    disabled: bool,
    warned: std::collections::HashSet<String>,
}

impl WalkPhysics {
    pub fn new(root: std::path::PathBuf) -> Self {
        Self { root, client: None, resources: Resources::default(), state: WalkState::default(), disabled: false, warned: Default::default() }
    }

    pub fn absorb(&mut self, resources: Resources) {
        self.resources.absorb(resources);
    }

    pub fn camera_hit(&mut self, views: &[View], spawned: &[Spawned], from: Vec3, to: Vec3, extent: f32) -> Option<Option<crate::trace::Hit>> {
        if self.disabled {
            return None;
        }
        let client = self.client.as_mut()?;
        match camera_hit(client, &mut self.resources, views, spawned, from, to, extent) {
            Ok(hit) => Some(hit),
            Err(error) => {
                if self.warned.insert(error.clone()) {
                    log::warn!("colisão da câmera: {error}");
                }
                None
            }
        }
    }

    pub fn music_zone(&mut self, views: &[View], point: Vec3) -> Option<killer_assets::collision::world::MusicZone> {
        if self.disabled {
            return None;
        }
        let client = self.client.as_mut()?;
        match killer_assets::collision::world::music_volume_at(client, &mut self.resources, views, point) {
            Ok(zone) => zone,
            Err(error) => {
                if self.warned.insert(error.clone()) {
                    log::warn!("zona de música: {error}");
                }
                None
            }
        }
    }

    pub fn forget(&mut self, object_id: i32) {
        self.state.forget(object_id);
    }

    pub fn foot_volumes(&self) -> &std::collections::HashMap<i32, FootVolume> {
        &self.state.foot
    }

    pub fn swim_ticks(&self) -> &std::collections::HashMap<i32, f32> {
        &self.state.swim
    }

    pub fn snapshot(&self, world: &killer_world::World) -> Vec<(i32, Vec3)> {
        self.state.snapshot(world)
    }

    pub fn take_calls(&mut self) -> Vec<(i32, ControllerCall)> {
        std::mem::take(&mut self.state.calls)
    }

    pub fn apply(&mut self, world: &mut killer_world::World, before: &[(i32, Vec3)], views: &[View], spawned: &[Spawned], dt: f32, options: &killer_assets::options::ClientOptions) -> Vec<i32> {
        let handled = Vec::new();
        self.state.retain_present(world);
        if self.disabled || before.is_empty() || views.is_empty() || dt <= 0.0 {
            return handled;
        }
        if self.client.is_none() {
            match Client::open(&self.root) {
                Ok(client) => self.client = Some(client),
                Err(error) => {
                    log::warn!("física de andar desligada: {error}");
                    self.disabled = true;
                    return handled;
                }
            }
        }
        let Some(client) = self.client.as_mut() else { return handled };
        let started = killer_platform::time::Instant::now();
        let (handled, error) = walk_entities(client, &mut self.resources, &mut self.state, world, Frame { before, views, spawned, dt, options });
        if started.elapsed().as_millis() > 100 {
            log::warn!("física de andar levou {:?} com {} tile(s): {}", started.elapsed(), views.len(), views.iter().map(|view| view.name.as_str()).collect::<Vec<_>>().join(","));
        }
        if let Some(error) = error
            && self.warned.insert(error.clone())
        {
            log::warn!("física de andar: {error}");
        }
        handled
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FootVolume {
    pub grass: bool,
    pub water: bool,
    pub step: bool,
    pub step_sound_id: i32,
}

#[derive(Default)]
pub struct WalkState {
    falling: std::collections::HashMap<i32, Vec3>,
    pub foot: std::collections::HashMap<i32, FootVolume>,
    pub swim: std::collections::HashMap<i32, f32>,
    grounded: std::collections::HashSet<i32>,
    swimming: std::collections::HashSet<i32>,
    pub calls: Vec<(i32, ControllerCall)>,
    rand: Option<crate::world_sound::CrtRand>,
    grass: Option<(std::sync::Weak<ue2_level::Level>, bool, killer_assets::scene::deco::GrassField)>,
}

impl WalkState {
    pub fn forget(&mut self, object_id: i32) {
        self.falling.remove(&object_id);
        self.grounded.remove(&object_id);
        self.swimming.remove(&object_id);
    }

    pub fn retain_present(&mut self, world: &killer_world::World) {
        let present = |id: &i32| world.entities.contains_key(id) || world.me.as_ref().is_some_and(|me| me.object_id == *id);
        self.falling.retain(|id, _| present(id));
        self.foot.retain(|id, _| present(id));
        self.swim.retain(|id, _| present(id));
        self.grounded.retain(|id| present(id));
        self.swimming.retain(|id| present(id));
    }

    #[cfg(test)]
    pub fn is_grounded(&self, object_id: i32) -> bool {
        self.grounded.contains(&object_id)
    }

    #[cfg(test)]
    pub fn is_falling(&self, object_id: i32) -> bool {
        self.falling.contains_key(&object_id)
    }

    #[cfg(test)]
    pub fn is_swimming(&self, object_id: i32) -> bool {
        self.swimming.contains(&object_id)
    }

    pub fn snapshot(&self, world: &killer_world::World) -> Vec<(i32, Vec3)> {
        world.me.iter().chain(world.entities.values().filter(|entity| entity.dest.is_some() || !self.grounded.contains(&entity.object_id) || self.swimming.contains(&entity.object_id))).map(|entity| (entity.object_id, entity.position)).collect()
    }
}

pub const HEAD_OFFSET: f32 = 13.0;
pub const STEP_SOUND_ID_MAX: i32 = 1000;
pub const LANDING_RAND_SEED: u32 = 0x2545_f491;

pub fn step_sound_entry(musicinfo: &std::collections::HashMap<u32, Vec<String>>, step_sound_id: i32) -> Option<&[String]> {
    if !(1..=STEP_SOUND_ID_MAX).contains(&step_sound_id) {
        return None;
    }
    musicinfo.get(&step_sound_id.unsigned_abs()).map(Vec::as_slice).filter(|names| !names.is_empty())
}

pub fn step_sound_name(names: Option<&[String]>, mut rand: impl FnMut() -> i32) -> Option<&String> {
    let names = names?;
    let count = i32::try_from(names.len()).ok()?;
    names.get(usize::try_from(rand().rem_euclid(count)).ok()?)
}

fn volume_of(props: &killer_assets::collision::resources::PhysicsProps) -> killer_world::walk::Volume {
    killer_world::walk::Volume {
        gravity: Vec3::from(props.gravity),
        zone_velocity: Vec3::from(props.zone_velocity),
        terminal_velocity: props.terminal_velocity,
        ground_friction: props.ground_friction,
        fluid_friction: props.fluid_friction,
        water: props.l2_water,
    }
}

pub struct Frame<'a> {
    pub before: &'a [(i32, Vec3)],
    pub views: &'a [View],
    pub spawned: &'a [Spawned],
    pub dt: f32,
    pub options: &'a killer_assets::options::ClientOptions,
}

pub fn walk_entities(client: &mut Client, resources: &mut Resources, state: &mut WalkState, world: &mut killer_world::World, frame: Frame<'_>) -> (Vec<i32>, Option<String>) {
    let Frame { before, views, spawned, dt, options } = frame;
    use killer_world::walk::{AI_CONTROLLER_MIN_HIT_WALL, CONTROLLER_MIN_HIT_WALL, Controller, Landing, Outcome, Walker, enter_swimming, floor_probe, phys_falling, phys_swimming, phys_walking, process_landed, safe_normal};
    let mut handled = Vec::new();
    state.swim.clear();
    state.calls.clear();
    let world_me = world.me.as_ref().map(|me| me.object_id);
    let safe_direction = safe_normal;
    let mut sweep = WorldSweep::new(client, resources, views);
    sweep.spawned.extend_from_slice(spawned);
    let started: std::collections::HashMap<i32, Vec3> = before.iter().copied().collect();
    let mut bodies: std::collections::HashMap<i32, Target> = world
        .me
        .iter()
        .chain(world.entities.values())
        .filter(|entity| !entity.dead && entity.collision_radius > 0.0 && entity.collision_height > 0.0)
        .map(|entity| {
            let position = started.get(&entity.object_id).copied().unwrap_or(entity.position);
            (entity.object_id, Target { object_id: entity.object_id, spawned: entity.spawned, position, radius: entity.collision_radius, height: entity.collision_height })
        })
        .collect();
    let mut previous: Option<i32> = None;
    for &(object_id, from) in before {
        if let Some(done) = previous.replace(object_id) {
            let moved = if world_me == Some(done) { world.me.as_ref() } else { world.entities.get(&done) };
            if let (Some(entity), Some(body)) = (moved, bodies.get_mut(&done)) {
                body.position = entity.position;
            }
        }
        let found = if world_me == Some(object_id) { world.me.as_mut() } else { world.entities.get_mut(&object_id) };
        let Some(entity) = found else { continue };
        if entity.collision_radius <= 0.0 || entity.collision_height <= 0.0 {
            continue;
        }
        let velocity = (entity.position - from).with_z(0.0) / dt;
        if velocity.length() > MAX_PLANAR_SPEED {
            state.grounded.remove(&object_id);
            state.falling.remove(&object_id);
            continue;
        }
        sweep.current = object_id;
        sweep.targets.clear();
        if killer_world::walk::pawn_blocks_mover(killer_world::walk::LINEAGE_PAWN_CAN_IGNORE_COLLISION, killer_world::walk::LINEAGE_PAWN_CAN_BE_IGNORED_COLLISION) {
            sweep.targets.extend(bodies.iter().filter(|(id, _)| **id != object_id).map(|(_, target)| *target));
        }
        let mut walker = Walker::new(from, entity.collision_radius, entity.collision_height);
        walker.on_world = state.grounded.contains(&object_id);
        walker.velocity = velocity;
        walker.acceleration = velocity;
        let around = match (
            physics_volume_at(sweep.client, sweep.resources, views, from),
            physics_volume_at(sweep.client, sweep.resources, views, from + Vec3::Z * HEAD_OFFSET),
            physics_volume_at(sweep.client, sweep.resources, views, from - Vec3::Z * entity.collision_height),
        ) {
            (Ok(body), Ok(head), Ok(foot)) => Some((body, head, foot)),
            (Err(error), ..) | (_, Err(error), _) | (.., Err(error)) => {
                sweep.error.get_or_insert(error);
                None
            }
        };
        let Some((body, head, foot)) = around else { continue };
        walker.volume = volume_of(&body);
        walker.head_in_water = head.l2_water;
        let mut grass = false;
        if world_me == Some(object_id) && let Some(view) = views.first() {
            let fresh = state.grass.as_ref().is_some_and(|(level, deco, _)| *deco == options.render_deco && level.upgrade().is_some_and(|level| Arc::ptr_eq(&level, &view.level)));
            if !fresh {
                let field = killer_assets::scene::decoration_points(sweep.client, &view.level, options);
                state.grass = Some((Arc::downgrade(&view.level), options.render_deco, field));
            }
            grass = state.grass.as_ref().is_some_and(|(_, _, field)| field.count_within(from.x, from.y, killer_assets::scene::deco::GrassField::RADIUS) > 1);
        }
        state.foot.insert(object_id, FootVolume { grass, water: foot.l2_water, step: foot.l2_step, step_sound_id: foot.step_sound_id });
        let mut remaining = dt;
        walker.human = world_me == Some(object_id);
        walker.controller = if walker.human { Controller::Player } else { Controller::Ai };
        walker.my_controller = walker.human;
        walker.moving = entity.dest.is_some();
        walker.min_hit_wall = if walker.human { CONTROLLER_MIN_HIT_WALL } else { AI_CONTROLLER_MIN_HIT_WALL };
        walker.move_speed = entity.speed();
        walker.destination = entity.dest;
        walker.acceleration = entity.dest.map_or(Vec3::ZERO, |dest| safe_direction(dest - from));
        let mut swimming = state.swimming.contains(&object_id);
        if swimming {
            walker.velocity = state.falling.get(&object_id).copied().unwrap_or(Vec3::ZERO);
            state.swim.insert(object_id, dt);
            match phys_swimming(&mut sweep, &mut walker, dt) {
                Outcome::Surfaced { remaining: left } => {
                    state.swimming.remove(&object_id);
                    state.falling.remove(&object_id);
                    state.grounded.remove(&object_id);
                    swimming = false;
                    remaining = left.max(0.0);
                    walker.velocity = velocity;
                }
                _ => {
                    state.falling.insert(object_id, walker.velocity);
                    entity.position = walker.location;
                    handled.push(object_id);
                    continue;
                }
            }
        }
        if !swimming && !state.grounded.contains(&object_id) {
            let fall_velocity = state.falling.get(&object_id).copied().unwrap_or(Vec3::ZERO);
            walker.velocity = fall_velocity;
            let old_velocity = walker.velocity;
            match phys_falling(&mut sweep, &mut walker, dt, 0) {
                Outcome::Landed { remaining: left, .. } => {
                    let landing = {
                        let generator = state.rand.get_or_insert(crate::world_sound::CrtRand::seeded(LANDING_RAND_SEED));
                        let mut frand = || generator.fraction();
                        process_landed(&mut sweep, &mut walker, !entity.dead, &mut frand)
                    };
                    match landing {
                        Landing::Respotted => {
                            state.falling.insert(object_id, walker.velocity);
                            entity.position = walker.location;
                            handled.push(object_id);
                            continue;
                        }
                        Landing::Stopped => {
                            state.falling.remove(&object_id);
                            state.grounded.insert(object_id);
                            entity.position = walker.location;
                            handled.push(object_id);
                            continue;
                        }
                        Landing::Landed => {}
                    }
                    state.falling.remove(&object_id);
                    state.grounded.insert(object_id);
                    walker.on_world = true;
                    remaining = left;
                    walker.velocity = velocity;
                }
                Outcome::Swimming { remaining: left } => {
                    match enter_swimming(&mut sweep, &mut walker, from, old_velocity, dt, left, 0) {
                        Outcome::Surfaced { .. } => {
                            state.falling.insert(object_id, walker.velocity);
                        }
                        _ => {
                            state.swimming.insert(object_id);
                            state.falling.insert(object_id, walker.velocity);
                        }
                    }
                    entity.position = walker.location;
                    handled.push(object_id);
                    continue;
                }
                _ => {
                    state.falling.insert(object_id, walker.velocity);
                    entity.position = walker.location;
                    handled.push(object_id);
                    continue;
                }
            }
        }
        match phys_walking(&mut sweep, &mut walker, remaining) {
            Outcome::Walking | Outcome::Halted => {
                entity.position = walker.location;
                entity.blocked = walker.location.truncate() == from.truncate();
                handled.push(object_id);
            }
            Outcome::Swimming { remaining: left } => {
                let old_velocity = velocity;
                if matches!(enter_swimming(&mut sweep, &mut walker, from, old_velocity, dt, left, 0), Outcome::Swimming { .. }) {
                    state.swimming.insert(object_id);
                    state.grounded.remove(&object_id);
                    state.falling.insert(object_id, walker.velocity);
                }
                entity.position = walker.location;
                handled.push(object_id);
            }
            Outcome::Blocked => {
                entity.blocked = walker.location.truncate() == from.truncate();
                let mut start = Walker::new(from, entity.collision_radius, entity.collision_height);
                start.on_world = true;
                if floor_probe(&mut sweep, &start).time >= 1.0 {
                    state.grounded.remove(&object_id);
                }
                entity.position = walker.location;
                handled.push(object_id);
            }
            Outcome::Falling { .. } => {
                state.grounded.remove(&object_id);
                state.falling.insert(object_id, walker.velocity);
                entity.position = walker.location;
                handled.push(object_id);
            }
            Outcome::Landed { .. } | Outcome::Surfaced { .. } => {}
        }
    }
    state.calls.append(&mut sweep.calls);
    (handled, sweep.error.take())
}

#[cfg(test)]
mod tests {
    #[test]
    fn walk_state_forgets_the_actors_that_left_the_world() {
        let mut state = WalkState::default();
        let mut world = killer_world::World::default();
        world.entities.insert(7, crate::characters::demo_entity(7, killer_world::EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: true }, Vec3::ZERO, 0, 20.0));
        for id in [7, 8] {
            state.falling.insert(id, Vec3::ZERO);
            state.foot.insert(id, FootVolume::default());
            state.swim.insert(id, 0.0);
            state.grounded.insert(id);
            state.swimming.insert(id);
        }
        state.retain_present(&world);
        assert_eq!(state.falling.keys().copied().collect::<Vec<_>>(), [7]);
        assert_eq!(state.foot.keys().copied().collect::<Vec<_>>(), [7]);
        assert_eq!(state.swim.keys().copied().collect::<Vec<_>>(), [7]);
        assert_eq!(state.grounded.iter().copied().collect::<Vec<_>>(), [7]);
        assert_eq!(state.swimming.iter().copied().collect::<Vec<_>>(), [7]);
    }

    use super::*;

    #[test]
    fn step_sound_name_reads_music_info_ids_one_to_a_thousand_and_rolls_rand_modulo_the_count() {
        let mut musicinfo = std::collections::HashMap::new();
        musicinfo.insert(0, vec!["INTRO".to_owned()]);
        musicinfo.insert(7, vec!["S.a".to_owned(), "S.b".to_owned(), "S.c".to_owned()]);
        musicinfo.insert(1000, vec!["S.last".to_owned()]);
        musicinfo.insert(1001, vec!["S.out".to_owned()]);
        musicinfo.insert(9, Vec::new());
        assert_eq!(step_sound_entry(&musicinfo, 0), None);
        assert_eq!(step_sound_entry(&musicinfo, -7), None);
        assert_eq!(step_sound_entry(&musicinfo, 1001), None);
        assert_eq!(step_sound_entry(&musicinfo, 9), None);
        assert_eq!(step_sound_entry(&musicinfo, 8), None);
        assert_eq!(step_sound_entry(&musicinfo, 1000).map(<[String]>::len), Some(1));
        let names = step_sound_entry(&musicinfo, 7);
        assert_eq!(step_sound_name(names, || 32767).map(String::as_str), Some("S.b"));
        assert_eq!(step_sound_name(names, || 5).map(String::as_str), Some("S.c"));
        let mut rolled = false;
        assert_eq!(step_sound_name(None, || { rolled = true; 0 }), None);
        assert!(!rolled);
    }
}
