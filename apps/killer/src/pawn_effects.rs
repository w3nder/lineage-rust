use crate::characters::{self, AppearanceKey, CharacterCache};
use killer_assets::{projectile::Projectile, characters::CharacterMesh, client::Client, options::ClientOptions, pawn::PawnEffect, scene::emitters::EmitterAsset, textures::{BankSource, TextureBank}};
use killer_render::renderer::{Renderer, EffectId, EffectPlacement};
use killer_render::skinned::{CharacterId, CharacterInstance};
use killer_world::World;
use std::collections::HashMap;

#[derive(Default)]
pub struct Effects {
    weapons: HashMap<(i32, usize), (AppearanceKey, EffectId)>,
    active: HashMap<u64, EffectId>,
    loose: Vec<EffectId>,
    camera: Vec<EffectId>,
    viewer: Option<(glam::Vec3, [i32; 3], u8)>,
    pub assets: HashMap<String, Option<EmitterAsset>>,
    trailer: HashMap<String, killer_assets::client::TrailerClass>,
    flights: Vec<Flight>,
    arrow_meshes: HashMap<i32, Option<ArrowMesh>>,
    arrow_bank: TextureBank,
    arrows: Vec<ArrowFlight>,
    firecrackers: Vec<Firecracker>,
    enter_events: Vec<EffectId>,
    quest_arrow: Option<EffectId>,
    pub bank: TextureBank,
}

pub const BANK_TRIM_BYTES: usize = 128 * 1024 * 1024;

impl Effects {
    pub fn trim_banks(&mut self) -> usize {
        self.bank.trim_over(BANK_TRIM_BYTES) + self.arrow_bank.trim_over(BANK_TRIM_BYTES)
    }
}

const FIRECRACKER_SEED: &str = "LineageEffect.e_u519_fireseed_";
const FIRECRACKER_BOOM: &str = "LineageEffect.e_u519_firecracker_";
const FIRECRACKER_ROCKETS: usize = 8;
const FIRECRACKER_ACCELERATION: f32 = -200.0;
const FIRECRACKER_MAX_RISE: f32 = 4.0;

struct Rocket {
    start: f32,
    speed: f32,
    ceiling: f32,
    offset: glam::Vec3,
    class: usize,
    rise_from: f32,
    z: f32,
    yaw: i32,
    effect: Option<EffectId>,
    done: bool,
}

struct Firecracker {
    owner: i32,
    origin: Option<glam::Vec3>,
    elapsed: f32,
    count: usize,
    rockets: Vec<Rocket>,
}

impl Firecracker {
    fn new(owner: i32, mut rand: impl FnMut() -> i32) -> Self {
        let count = (3 + rand() % 3) as usize;
        let rockets = (0..FIRECRACKER_ROCKETS)
            .map(|_| {
                let start = (rand() % 5) as f32 * 0.4;
                let speed = (rand() % 50 + 200) as f32;
                let ceiling = (rand() % 15 + 20) as f32 * 4.5;
                let x = ((rand() % 7) * 8 - 24) as f32;
                let y = ((rand() % 7) * 8 - 24) as f32;
                let class = (rand() % 4) as usize;
                Rocket { start, speed, ceiling, offset: glam::Vec3::new(x, y, 0.0), class, rise_from: 0.0, z: 0.0, yaw: 0, effect: None, done: false }
            })
            .collect();
        Self { owner, origin: None, elapsed: 0.0, count, rockets }
    }
}

const ARROW_CLASS: &str = "LineageEffect.NArrow";

struct ArrowMesh {
    id: CharacterId,
    mesh: CharacterMesh,
}

struct ArrowFlight {
    item: i32,
    projectile: Projectile,
    attacker: i32,
    light: bool,
    victim: i32,
    scale: f32,
    effects: Vec<crate::characters::WorldSpawn>,
    launched: bool,
    hit: killer_world::anim::Hit,
    victim_pawn: bool,
    curvature: i32,
}

const PROJECTILE_BASE: &str = "Engine.NProjectile";
const PROJECTILE_DEFAULT_ACC_SPEED: f32 = 10.0;
const MOVER_PROJECTILES: [&str; 3] = ["LineageEffect.m_u003_b", "LineageEffect.s_u019_c", "LineageEffect.s_u020_c"];

struct Flight {
    projectile: Projectile,
    effect: EffectId,
    shot: crate::characters::ProjectileSpawn,
    zone: u8,
    launched: bool,
}

fn target_point(cache: &CharacterCache, world: &World, object_id: i32) -> Option<glam::Vec3> {
    cache.effect_target(object_id).or_else(|| {
        let entity = world.me.iter().find(|me| me.object_id == object_id).or_else(|| world.entities.get(&object_id))?;
        Some(entity.position + glam::Vec3::Z * (entity.collision_height * 0.5))
    })
}

impl Effects {
    pub fn arrow_instances(&self, view: &impl crate::items::Geometry) -> Vec<CharacterInstance> {
        use killer_assets::scene::actor_lighting::{Bounds, Query};
        self.arrows
            .iter()
            .filter(|flight| flight.launched)
            .filter_map(|flight| {
                let Some(Some(arrow)) = self.arrow_meshes.get(&flight.item) else { return None };
                let position = flight.projectile.location;
                let world = killer_assets::collision::transform::actor(
                    ue2_level::Placement { location: position.to_array(), rotation: flight.projectile.rotation },
                    [flight.scale, 1.0, 1.0],
                    [0.0; 3],
                ) * arrow.mesh.local;
                let bounds = Bounds::transformed(arrow.mesh.bounding_box, world);
                let center = world.transform_point3(glam::Vec3::from_slice(&arrow.mesh.bounding_sphere[..3]));
                let radius = arrow.mesh.bounding_sphere[3] * arrow.mesh.scale.abs().max_element();
                let lighting = view.actor_lighting(Query { position, center, radius, bounds });
                Some(CharacterInstance {
                    overlay: [0.0; 4],
                    actor_id: 0,
                    actor_position: position,
                    collision_height: 0.0,
                    shadow_caster: false,
                    light_sphere: [center.x, center.y, center.z, radius],
                    mesh: arrow.id,
                    world,
                    palette: vec![glam::Mat4::IDENTITY; arrow.mesh.bones.len()],
                    zone: view.zone_at(position),
                    lights: lighting.lights,
                    pawn_lights: Vec::new(),
                    ambient: lighting.ambient,
                    sun_affect: lighting.sun_affect,
                })
            })
            .collect()
    }

    fn spawn_arrows(&mut self, renderer: &mut Renderer, cache: &mut CharacterCache, client: &mut Client) {
        let spawns = cache.take_arrow_spawns();
        if spawns.is_empty() {
            return;
        }
        for spawn in &spawns {
            let Some(model) = &spawn.model else {
                log::warn!("flecha de {}: sem modelo do item", spawn.attacker);
                continue;
            };
            if !self.arrow_meshes.contains_key(&model.item) {
                let part = killer_assets::pawn::Part::new(&model.mesh, model.texture.iter().cloned().collect());
                let loaded = killer_assets::characters::load_part(client, &part).map(|mesh| {
                    let id = renderer.upload_character(&mesh, &mut BankSource { bank: &mut self.arrow_bank, catalog: client.catalog() });
                    ArrowMesh { id, mesh }
                });
                log::info!("flecha item {:#x}: malha {} textura {:?} carregada={}", model.item, model.mesh, model.texture, loaded.is_some());
                self.arrow_meshes.insert(model.item, loaded);
            }
        }
        let acc_speed = client
            .class_float_defaults(ARROW_CLASS, &["AccSpeed"])
            .ok()
            .and_then(|defaults| defaults.get("AccSpeed").copied())
            .unwrap_or(PROJECTILE_DEFAULT_ACC_SPEED);
        let directional = client
            .class_boolean_defaults(ARROW_CLASS, &["bDirectional"])
            .ok()
            .and_then(|defaults| defaults.get("bDirectional").copied())
            .unwrap_or(false);
        for spawn in spawns {
            let item = spawn.model.as_ref().map(|model| model.item);
            if !item.is_some_and(|item| matches!(self.arrow_meshes.get(&item), Some(Some(_)))) {
                for effect in spawn.effects {
                    cache.push_world_spawn(effect);
                }
                cache.push_arrow_impact(spawn.victim, spawn.hit);
                if spawn.victim_pawn && characters::arrow_damage_act(spawn.hit, true) {
                    cache.attacked(spawn.victim);
                }
                continue;
            }
            self.arrows.push(ArrowFlight {
                item: item.unwrap_or_default(),
                projectile: Projectile::new(spawn.start, [0; 3], acc_speed, directional),
                attacker: spawn.attacker,
                light: spawn.light,
                victim: spawn.victim,
                scale: spawn.scale,
                effects: spawn.effects,
                launched: false,
                hit: spawn.hit,
                victim_pawn: spawn.victim_pawn,
                curvature: spawn.curvature,
            });
        }
    }

    fn fly_arrows(&mut self, cache: &mut CharacterCache, world: &World, dt: f32) {
        let mut arrived = Vec::new();
        let mut lights = Vec::new();
        let mut struck = Vec::new();
        let mut impacts = Vec::new();
        self.arrows.retain_mut(|flight| {
            let Some(point) = target_point(cache, world, flight.victim) else { return false };
            if !flight.launched {
                if flight.curvature > 0 {
                    flight.projectile.prepare_interpolation(flight.curvature, point - flight.projectile.location);
                }
                flight.projectile.launch(point);
                flight.launched = true;
            }
            if flight.projectile.tick(dt, point).is_some() {
                impacts.push((flight.attacker, flight.victim, flight.hit));
                arrived.append(&mut flight.effects);
                if flight.victim_pawn {
                    struck.push((flight.attacker, flight.victim, flight.hit));
                }
                if flight.light {
                    lights.push((flight.attacker, flight.victim));
                }
                return false;
            }
            true
        });
        for effect in arrived {
            cache.push_world_spawn(effect);
        }
        let find = |id: i32| world.me.iter().find(|me| me.object_id == id).or_else(|| world.entities.get(&id));
        for (attacker, victim, hit) in impacts {
            if find(attacker).is_some_and(|entity| characters::is_pawn(&entity.kind)) {
                cache.push_arrow_impact(victim, hit);
            }
        }
        for (attacker, victim, hit) in struck {
            let attacker_pawn = find(attacker).is_some_and(|entity| characters::is_pawn(&entity.kind));
            if characters::arrow_damage_act(hit, attacker_pawn) {
                cache.attacked(victim);
            }
        }
        for (attacker, victim) in lights {
            let (Some(attacker), Some(victim)) = (find(attacker), find(victim)) else { continue };
            if characters::is_pawn(&attacker.kind) && characters::is_pawn(&victim.kind) {
                characters::add_hit_light(cache.pawn_lights_mut(victim.object_id), attacker.position, victim.position, victim.collision_radius);
            }
        }
    }

    pub fn fly(&mut self, renderer: &mut Renderer, cache: &mut CharacterCache, world: &World, dt: f32) {
        self.fly_arrows(cache, world, dt);
        let mut arrivals = Vec::new();
        self.flights.retain_mut(|flight| {
            let Some(point) = target_point(cache, world, flight.shot.target) else {
                renderer.remove_effect(flight.effect);
                return false;
            };
            if !flight.launched {
                flight.projectile.launch(point);
                flight.launched = true;
            }
            let hit = flight.projectile.tick(dt, point);
            renderer.place_effect(
                flight.effect,
                EffectPlacement { location: flight.projectile.location, rotation: flight.projectile.rotation, zone: flight.zone, visible: true },
            );
            if let Some(hit) = &hit {
                arrivals.push(crate::characters::Arrival { shot: flight.shot.clone(), location: flight.projectile.location, rotation: flight.projectile.rotation, normal: hit.normal });
            }
            if hit.is_some() || renderer.effect_finished(flight.effect) {
                renderer.remove_effect(flight.effect);
                return false;
            }
            true
        });
        for arrival in arrivals {
            cache.push_arrival(arrival);
        }
    }

    fn firecracker_asset(&mut self, client: &mut Client, options: &ClientOptions, class: &str) -> Option<EmitterAsset> {
        self.assets
            .entry(class.to_owned())
            .or_insert_with(|| client.emitter_class(class).map(|emitter| killer_assets::scene::emitters::effect_asset(client, &emitter, options)))
            .clone()
    }

    pub fn firecrackers(&mut self, renderer: &mut Renderer, cache: &mut CharacterCache, client: &mut Client, world: &mut World, options: &ClientOptions, dt: f32) {
        self.start_firecrackers(cache, client, world, options);
        self.launch_firecrackers(renderer, cache, client, world, dt);
    }

    pub fn enter_events(&mut self, renderer: &mut Renderer, client: &mut Client, world: &mut World, options: &ClientOptions) {
        self.enter_events.retain(|id| {
            let finished = renderer.effect_finished(*id);
            if finished {
                renderer.remove_effect(*id);
            }
            !finished
        });
        for spawn in world.effects.take_enter_effects() {
            let Some(mut asset) = self.firecracker_asset(client, options, &spawn.class) else { continue };
            killer_assets::weapon_effects::set_size_scale(&mut asset.definition, spawn.scale);
            let id = renderer.add_effect(&asset, &mut killer_assets::textures::BankSource { bank: &mut self.bank, catalog: client.catalog() });
            renderer.place_effect(id, EffectPlacement { location: glam::Vec3::from(spawn.location), rotation: [0, spawn.yaw, 0], zone: 0, visible: true });
            self.enter_events.push(id);
        }
    }

    fn start_firecrackers(&mut self, cache: &mut CharacterCache, client: &mut Client, world: &mut World, options: &ClientOptions) {
        for owner in world.effects.take_firecrackers() {
            let classes = (0..4).all(|i| {
                self.firecracker_asset(client, options, &format!("{FIRECRACKER_SEED}{i}")).is_some()
                    && self.firecracker_asset(client, options, &format!("{FIRECRACKER_BOOM}{i}")).is_some()
            });
            if classes && world.entity(owner).is_some() {
                self.firecrackers.push(Firecracker::new(owner, || cache.crt_rand()));
            }
        }
    }

    fn launch_firecrackers(&mut self, renderer: &mut Renderer, cache: &mut CharacterCache, client: &mut Client, world: &World, dt: f32) {
        let mut firecrackers = std::mem::take(&mut self.firecrackers);
        firecrackers.retain_mut(|show| {
            let Some(owner) = world.me.as_ref().filter(|me| me.object_id == show.owner).or_else(|| world.entity(show.owner)) else {
                for effect in show.rockets.iter().filter_map(|rocket| rocket.effect) {
                    renderer.remove_effect(effect);
                }
                return false;
            };
            let origin = *show.origin.get_or_insert(owner.position);
            let height = owner.collision_height;
            show.elapsed += dt;
            for rocket in show.rockets.iter_mut().take(show.count).filter(|rocket| !rocket.done) {
                match rocket.effect {
                    None if show.elapsed >= rocket.start => {
                        let position = origin + rocket.offset;
                        let Some(asset) = self.assets.get(&format!("{FIRECRACKER_SEED}{}", rocket.class)).cloned().flatten() else {
                            rocket.done = true;
                            continue;
                        };
                        let id = renderer.add_effect(&asset, &mut killer_assets::textures::BankSource { bank: &mut self.bank, catalog: client.catalog() });
                        let yaw = cache.crt_rand() & 0xffff;
                        rocket.yaw = yaw;
                        rocket.rise_from = position.z + height;
                        rocket.z = position.z;
                        rocket.ceiling += rocket.rise_from;
                        renderer.place_effect(id, EffectPlacement { location: position, rotation: [0, yaw, 0], zone: 0, visible: true });
                        rocket.effect = Some(id);
                    }
                    Some(id) => {
                        let t = show.elapsed - rocket.start;
                        let position = origin + rocket.offset;
                        if rocket.z < rocket.ceiling && FIRECRACKER_ACCELERATION * t + rocket.speed >= 0.0 && t < FIRECRACKER_MAX_RISE {
                            rocket.z = 0.5 * FIRECRACKER_ACCELERATION * t * t + rocket.speed * t + rocket.rise_from;
                            renderer.place_effect(id, EffectPlacement { location: glam::Vec3::new(position.x, position.y, rocket.z), rotation: [0, rocket.yaw, 0], zone: 0, visible: true });
                        } else {
                            renderer.remove_effect(id);
                            if let Some(asset) = self.assets.get(&format!("{FIRECRACKER_BOOM}{}", rocket.class)).cloned().flatten() {
                                let boom = renderer.add_effect(&asset, &mut killer_assets::textures::BankSource { bank: &mut self.bank, catalog: client.catalog() });
                                renderer.place_effect(boom, EffectPlacement { location: glam::Vec3::new(position.x, position.y, rocket.z), rotation: [0, 0, 0], zone: 0, visible: true });
                                self.loose.push(boom);
                            }
                            rocket.done = true;
                        }
                    }
                    None => {}
                }
            }
            !show.rockets.iter().take(show.count).all(|rocket| rocket.done)
        });
        self.firecrackers = firecrackers;
    }

    pub fn sync(&mut self, renderer: &mut Renderer, cache: &mut CharacterCache, client: &mut Client, world: &World, options: &ClientOptions) {
        self.spawn_arrows(renderer, cache, client);
        sync_weapon_effects(renderer, cache, &mut self.weapons, client, world);
        sync_transients(renderer, cache, self, client, world, options);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn quest_arrow(&mut self, renderer: &mut Renderer, cache: &mut CharacterCache, client: &mut Client, world: &mut World, options: &ClientOptions, arrow_option: bool, zone: u8) {
        use crate::quest_arrow::{ARRIVAL_CLASS, ARRIVAL_PHYSICS, ARROW_CLASS, Frame, frame};
        let Some(me) = world.me.as_ref().filter(|_| !world.observer.ctrl_observer) else { return };
        let radar = &world.windows.radar;
        let target = glam::Vec3::from(radar.target.unwrap_or_default());
        let step = frame(radar.enabled && arrow_option, self.quest_arrow.is_some(), me.position, me.collision_height, target);
        let owner = characters::entity_key(me).map(|key| (me.object_id, key));
        match step {
            Frame::Idle => {}
            Frame::Spawn { location } => {
                if let Some(asset) = self.firecracker_asset(client, options, ARROW_CLASS) {
                    let id = renderer.add_effect(&asset, &mut BankSource { bank: &mut self.bank, catalog: client.catalog() });
                    renderer.place_effect(id, EffectPlacement { location, rotation: [0; 3], zone, visible: true });
                    self.quest_arrow = Some(id);
                }
            }
            Frame::Place { location, rotation, arrived } => {
                if let Some(id) = self.quest_arrow {
                    renderer.place_effect(id, EffectPlacement { location, rotation, zone, visible: true });
                }
                if arrived {
                    world.windows.radar.apply_control(2, 0, [0; 3]);
                }
            }
            Frame::Despawn { burst, pre_pivot } => {
                if let Some(id) = self.quest_arrow.take() {
                    renderer.remove_effect(id);
                }
                let same_rotation = self.trailer.entry(ARRIVAL_CLASS.to_owned()).or_insert_with(|| client.trailer_class(ARRIVAL_CLASS).unwrap_or_default()).same_rotation;
                cache.push_world_spawn(characters::WorldSpawn {
                    class: ARRIVAL_CLASS.to_owned(),
                    location: burst,
                    rotation: [0; 3],
                    zone,
                    modifiers: characters::EffectModifiers::default(),
                    owner: owner.map(|(object_id, key)| characters::OwnerSpawn { object_id, key, relative: false, pre_pivot }),
                    projectile: None,
                    trailer_physics: Some((ARRIVAL_PHYSICS, true, same_rotation)),
                });
            }
        }
    }

    pub fn set_viewer(&mut self, eye: glam::Vec3, rotation: [i32; 3], zone: u8) {
        self.viewer = Some((eye, rotation, zone));
    }

    pub fn clear(&mut self, renderer: &mut Renderer) {
        for flight in self.flights.drain(..) {renderer.remove_effect(flight.effect);}
        self.arrows.clear();
        for id in self.camera.drain(..) {renderer.remove_effect(id);}
        for (_, (_, id)) in self.weapons.drain() {renderer.remove_effect(id);}
        for (_, id) in self.active.drain() {renderer.remove_effect(id);}
        for id in self.loose.drain(..) {renderer.remove_effect(id);}
        if let Some(id) = self.quest_arrow.take() {renderer.remove_effect(id);}
        self.assets.clear();
        self.bank=TextureBank::new();
    }
}

fn sync_weapon_effects(renderer: &mut Renderer, cache: &mut CharacterCache, active: &mut HashMap<(i32, usize), (AppearanceKey, EffectId)>, client: &mut Client, world: &World) {
    let frames = cache.take_effect_frames();
    let mut seen = std::collections::HashSet::new();
    for frame in frames {
        let slot = (frame.object_id, frame.index);
        seen.insert(slot);
        if let Some((key, id)) = active.get(&slot).copied()
            && key != frame.key
        {
            renderer.remove_effect(id);
            active.remove(&slot);
        }
        if let std::collections::hash_map::Entry::Vacant(vacant) = active.entry(slot) {
            let Some(character) = cache.loaded_mut(frame.key) else { continue };
            let Some(loaded) = character.effect(frame.index) else { continue };
            let mut asset = loaded.asset.clone();
            match &loaded.effect {
                PawnEffect::Weapon(weapon) => killer_assets::weapon_effects::apply_weapon_scale(&mut asset.definition, weapon, frame.weapon_scale),
                PawnEffect::HeroAura => killer_assets::weapon_effects::set_size_scale(&mut asset.definition, frame.collision_radius * killer_assets::weapon_effects::HERO_AURA_SIZE_PER_RADIUS),
            }
            let id = renderer.add_effect(&asset, &mut BankSource { bank: &mut character.effect_bank, catalog: client.catalog() });
            vacant.insert((frame.key, id));
        }
        if let Some((_, id)) = active.get(&slot) {
            renderer.place_effect(*id, EffectPlacement { location: frame.location, rotation: frame.rotation, zone: frame.zone, visible: frame.visible });
        }
    }
    active.retain(|slot, (key, id)| {
        if seen.contains(slot) {
            return true;
        }
        let entity = world.me.iter().find(|me| me.object_id == slot.0).or_else(|| world.entities.get(&slot.0));
        if entity.and_then(characters::entity_key) == Some(*key) {
            renderer.set_effect_visible(*id, false);
            true
        } else {
            renderer.remove_effect(*id);
            false
        }
    });
}

fn sync_transients(renderer: &mut Renderer, cache: &mut CharacterCache, transients: &mut Effects, client: &mut Client, world: &World, options: &ClientOptions) {
    for spawn in cache.take_transient_spawns() {
        let loaded = transients.assets.entry(spawn.class.clone()).or_insert_with(|| {
            client.emitter_class(&spawn.class).map(|emitter| killer_assets::scene::emitters::effect_asset(client, &emitter, options))
        });
        let Some(mut asset) = loaded.clone() else {
            cache.forget_transient(spawn.handle);
            continue;
        };
        spawn.modifiers.apply(&mut asset.definition);
        if let Some(velocity) = spawn.stick_velocity {
            for sprite in asset.definition.sprites.iter_mut().take(2) {
                sprite.start_velocity[0] = [velocity, velocity];
            }
        }
        let id = renderer.add_effect(&asset, &mut BankSource { bank: &mut transients.bank, catalog: client.catalog() });
        if let Some(rate) = spawn.modifiers.speed_rate {
            renderer.set_effect_speed_rate(id, rate);
        }
        if let Some((trailer, bone)) = &spawn.trailer {
            renderer.attach_trailer(id, trailer, bone);
        }
        transients.active.insert(spawn.handle, id);
    }
    for class in cache.take_camera_effects() {
        let loaded = transients.assets.entry(class.clone()).or_insert_with(|| {
            client.emitter_class(&class).map(|emitter| killer_assets::scene::emitters::effect_asset(client, &emitter, options))
        });
        let Some(asset) = loaded.clone() else { continue };
        let id = renderer.add_effect(&asset, &mut BankSource { bank: &mut transients.bank, catalog: client.catalog() });
        transients.camera.push(id);
    }
    if let Some((eye, rotation, zone)) = transients.viewer {
        transients.camera.retain(|id| {
            if renderer.effect_finished(*id) {
                renderer.remove_effect(*id);
                return false;
            }
            renderer.place_effect(*id, EffectPlacement { location: eye, rotation, zone, visible: true });
            true
        });
    }
    for spawn in cache.take_world_spawns() {
        let loaded = transients.assets.entry(spawn.class.clone()).or_insert_with(|| {
            client.emitter_class(&spawn.class).map(|emitter| killer_assets::scene::emitters::effect_asset(client, &emitter, options))
        });
        let Some(mut asset) = loaded.clone() else { continue };
        if let Some(shot) = spawn.projectile
            && !MOVER_PROJECTILES.iter().any(|class| class.eq_ignore_ascii_case(&spawn.class))
            && client.class_is_a(&spawn.class, PROJECTILE_BASE).unwrap_or(false)
        {
            spawn.modifiers.apply(&mut asset.definition);
            let id = renderer.add_effect(&asset, &mut BankSource { bank: &mut transients.bank, catalog: client.catalog() });
            if let Some(rate) = spawn.modifiers.speed_rate {
                renderer.set_effect_speed_rate(id, rate);
            }
            let acc_speed = client
                .class_float_defaults(&spawn.class, &["AccSpeed"])
                .ok()
                .and_then(|defaults| defaults.get("AccSpeed").copied())
                .unwrap_or(PROJECTILE_DEFAULT_ACC_SPEED);
            let directional = client
                .class_boolean_defaults(&spawn.class, &["bDirectional"])
                .ok()
                .and_then(|defaults| defaults.get("bDirectional").copied())
                .unwrap_or(false);
            transients.flights.push(Flight {
                projectile: Projectile::new(spawn.location, spawn.rotation, acc_speed, directional),
                effect: id,
                shot,
                zone: spawn.zone,
                launched: false,
            });
            continue;
        }
        let mut trailer = *transients.trailer.entry(spawn.class.clone()).or_insert_with(|| client.trailer_class(&spawn.class).unwrap_or_default());
        cache.note_trailer_class(&spawn.class, trailer);
        if let Some((physics, pre_pivot, same_rotation)) = spawn.trailer_physics {
            trailer.physics = physics;
            trailer.pre_pivot = pre_pivot;
            trailer.same_rotation = same_rotation;
        }
        let follow = match spawn.owner {
            Some(owner) if trailer.physics == killer_assets::client::PHYS_TRAILER => {
                if trailer.relative_trail {
                    continue;
                }
                Some((owner, characters::OwnerFollow {
                    pre_pivot: trailer.pre_pivot.then_some(owner.pre_pivot),
                    rotation: (!trailer.same_rotation).then_some(spawn.rotation),
                }))
            }
            Some(owner) if owner.relative => continue,
            _ => None,
        };
        spawn.modifiers.apply(&mut asset.definition);
        let id = renderer.add_effect(&asset, &mut BankSource { bank: &mut transients.bank, catalog: client.catalog() });
        if let Some(rate) = spawn.modifiers.speed_rate {
            renderer.set_effect_speed_rate(id, rate);
        }
        if let Some((owner, follow)) = follow {
            let handle = cache.follow_owner(owner.object_id, owner.key, follow);
            transients.active.insert(handle, id);
            continue;
        }
        renderer.place_effect(id, EffectPlacement { location: spawn.location, rotation: spawn.rotation, zone: spawn.zone, visible: true });
        transients.loose.push(id);
    }
    transients.loose.retain(|id| {
        let finished = renderer.effect_finished(*id);
        if finished {
            renderer.remove_effect(*id);
        }
        !finished
    });
    let mut seen = std::collections::HashSet::new();
    for frame in cache.take_transient_frames() {
        if let Some(id) = transients.active.get(&frame.handle) {
            seen.insert(frame.handle);
            renderer.place_effect(*id, EffectPlacement { location: frame.location, rotation: frame.rotation, zone: frame.zone, visible: true });
            if let Some(ribbon) = frame.ribbon {
                renderer.set_effect_ribbon(*id, ribbon);
            }
        }
    }
    transients.active.retain(|handle, id| {
        let alive = cache.transient_alive(*handle, |object_id| world.me.iter().any(|me| me.object_id == object_id) || world.entities.contains_key(&object_id));
        if !alive || renderer.effect_finished(*id) {
            renderer.remove_effect(*id);
            cache.forget_transient(*handle);
            return false;
        }
        if !seen.contains(handle) {
            renderer.set_effect_visible(*id, false);
        }
        true
    });
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn firecracker_action_10471c60_draws_count_and_eight_rockets_from_rand() {
        let mut values = (0..).map(|i| i * 7 + 3);
        let show = Firecracker::new(1, || values.next().unwrap());
        assert_eq!(show.count, 3);
        let first = &show.rockets[0];
        assert_eq!((first.start, first.speed, first.ceiling), ((10 % 5) as f32 * 0.4, 217.0, (24 % 15 + 20) as f32 * 4.5));
        assert_eq!((first.offset.x, first.offset.y, first.class), (((31 % 7) * 8 - 24) as f32, ((38 % 7) * 8 - 24) as f32, (45 % 4) as usize));
        assert_eq!(show.rockets.len(), FIRECRACKER_ROCKETS);
    }

    #[test]
    #[ignore]
    fn shared_host_renders_enchants_hero_and_soulshots_and_releases_on_character_change() {
        use crate::characters::{Frame, LoadedCharacter};
        use crate::loader::{LoadResult, Shared, load_character};
        use glam::Vec3;
        use killer_assets::scene::{Scene, ZoneLighting};
        use killer_render::{camera::Camera, environment::Environment, headless::Headless};
        use std::sync::Arc;
        let root=std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let mut client=Client::open(std::path::Path::new(&root)).unwrap();
        let tables=client.tables().unwrap();
        let options=ClientOptions::load(&mut client).unwrap();
        let mut shared=Shared::default();
        let camera=Camera {eye:Vec3::new(90.0,-120.0,65.0),target:Vec3::new(0.0,0.0,15.0),fov_x_deg:60.0,near:1.0,far:1000.0};
        for (weapon,hero) in [(80,false),(150,false),(1,true)] {
            let mut world=characters::demo_world(Vec3::ZERO,0,0);
            world.entities.clear();
            let me=world.me.as_mut().unwrap();
            if let killer_world::EntityKind::Player {equipped,enchant_effect,hero:flag,..}=&mut me.kind {
                equipped.fill(0);
                equipped[l2_net::game::packets::EQUIP_RHAND]=weapon;
                *enchant_effect=16;
                *flag=hero;
            }
            let key=characters::appearance_key(&me.kind).unwrap();
            let object_id=me.object_id;
            let look=characters::look(key,&tables).unwrap();
            let LoadResult::Character {parts,animation,draw_scale,weapon_type,mut bank,effects:loaded,..}=load_character(&mut client,&mut shared,key,&look.appearance,look.draw_scale) else {panic!("personagem");};
            assert!(!loaded.effects.is_empty());
            let mut gpu=Headless::new(256,256).unwrap();
            gpu.upload(&Scene {zones:vec![ZoneLighting::default()],..Default::default()},&mut BankSource {bank:&mut TextureBank::new(),catalog:client.catalog()});
            gpu.renderer_mut().set_particle_delta(Some(0.05));
            let empty=gpu.render(&camera,&Environment::default()).unwrap();
            let parts=parts.into_iter().map(|mesh|{
                let id=gpu.renderer_mut().upload_character(&mesh,&mut BankSource {bank:&mut bank,catalog:client.catalog()});
                (id,Arc::new(mesh))
            }).collect();
            let character=LoadedCharacter::new(parts,animation,draw_scale,weapon_type).unwrap().with_effects(loaded.effects,loaded.bank);
            let mut cache=CharacterCache::default();
            cache.insert(key,character);
            let mut effects=Effects::default();
            let frame=Frame {display:Default::default(),viewer:camera.eye,dt:0.05};
            let mut capture=Vec::new();
            for tick in 0..12 {
                gpu.renderer_mut().set_time(tick as f32*0.05);
                characters::instances(&mut world,&mut cache,&tables,&frame,|_|0,|_|Default::default(),|_,_|true);
                effects.sync(gpu.renderer_mut(),&mut cache,&mut client,&world,&options);
                assert!(!effects.weapons.is_empty());
                capture=gpu.render(&camera,&Environment::default()).unwrap();
            }
            assert_ne!(capture,empty,"arma {weapon}, hero {hero}");
            killer_assets::textures::write_png(std::path::Path::new(&format!("/tmp/web-desktop-effects-{weapon}.png")),256,256,&capture).unwrap();
            cache.spawn_shot_effects(object_id,key,2150,&tables.soulshot);
            characters::instances(&mut world,&mut cache,&tables,&frame,|_|0,|_|Default::default(),|_,_|true);
            effects.sync(gpu.renderer_mut(),&mut cache,&mut client,&world,&options);
            assert!(!effects.active.is_empty(),"soulshot {weapon}");
            world.me=None;
            effects.sync(gpu.renderer_mut(),&mut cache,&mut client,&world,&options);
            assert!(effects.weapons.is_empty() && effects.active.is_empty());
            effects.clear(gpu.renderer_mut());
            assert_eq!(gpu.renderer_mut().effect_count(),0);
            assert_eq!(gpu.render(&camera,&Environment::default()).unwrap(),empty);
        }
    }
}
