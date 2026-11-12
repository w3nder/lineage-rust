use super::{GpuDraw, MaterialGpu, Renderer};
use crate::camera::Camera;
use crate::particles::ribbon::{RibbonPoint, RibbonState};
use crate::particles::{EmptyWorld, ParticleSystem, beam_vertices, mesh_matrix, rotator_matrix, sprite_vertices};
use killer_assets::ribbon_point::{self, Bone, Trailer};
use crate::textures::TextureSource;
use glam::{Mat3, Vec3};
use killer_assets::scene::emitters::EmitterAsset;
use killer_assets::scene::{Pass, Vertex};

const L2_EVENT_SMOKE: &str = "antarascave_smoke";

fn smoke_chance(actor: [f32; 3], location: [f32; 3], radius: f32) -> f32 {
    let squared: f64 = (0..3).map(|i| f64::from(location[i] - actor[i]).powi(2)).sum();
    let t = squared.sqrt() / f64::from(radius);
    (2.0 / ((-t).exp() + t.exp())) as f32
}

pub(super) struct GpuEmitter {
    pub asset: EmitterAsset,
    pub system: ParticleSystem,
    pub draws: Vec<Vec<GpuDraw>>,
    pub tile: usize,
    pub effect: Option<Effect>,
    pub ribbons: Vec<Option<RibbonEmitter>>,
    pub ribbon_input: Option<RibbonInput>,
}

pub(super) struct RibbonEmitter {
    pub state: RibbonState,
    pub params: ribbon_point::Emitter,
    pub warnings: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RibbonInput {
    pub start: Option<Bone>,
    pub end: Option<Bone>,
    pub owner: Option<ribbon_point::Actor>,
    pub owner_mesh: bool,
    pub instance: bool,
}

fn ribbon_emitters(definition: &ue2_level::Emitter) -> Vec<Option<RibbonEmitter>> {
    definition
        .sprites
        .iter()
        .map(|sprite| {
            let shape = sprite.ribbon.as_ref()?;
            let mut state = RibbonState::new(shape);
            state.spawn(shape);
            Some(RibbonEmitter {
                state,
                params: ribbon_point::Emitter {
                    coordinate_system: sprite.coordinate_system,
                    axis: shape.axis,
                    flags: u32::from(shape.flags),
                    width: shape.width,
                    origin_offset: shape.origin_offset,
                    start_offset: shape.bone_offsets[0],
                    end_offset: shape.bone_offsets[1],
                    rotation: sprite.rotation_offset,
                    has_start_name: shape.bone_names[0].is_some(),
                },
                warnings: 0,
            })
        })
        .collect()
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Effect {
    pub id: EffectId,
    pub speed_rate: f32,
    pub visible: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EffectId(u64);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EffectPlacement {
    pub location: Vec3,
    pub rotation: [i32; 3],
    pub zone: u8,
    pub visible: bool,
}

fn emitter_visible(tiles: &[super::GpuTile], actor: &GpuEmitter, target: Vec3) -> bool {
    let shown = actor.effect.map_or_else(|| tiles.get(actor.tile).is_some_and(|t| t.visible), |effect| effect.visible);
    shown && (Vec3::from(actor.system.actor.location).truncate() - target.truncate()).length() <= actor.asset.range
}

impl Renderer {
    pub fn set_particle_delta(&mut self, dt: Option<f32>) { self.particle_delta = dt; }

    pub fn tick_particles(&mut self, dt: f32, target: Vec3) {
        for actor in &mut self.particle_actors {
            let visible = emitter_visible(&self.tiles, actor, target);
            actor.system.random = self.corona_random;
            actor.system.seeded_random = self.particle_seeded_random;
            actor.system.tick(dt * actor.effect.map_or(1.0, |effect| effect.speed_rate), visible, &actor.asset, &mut EmptyWorld);
            self.corona_random = actor.system.random;
            self.particle_seeded_random = actor.system.seeded_random;
        }
    }

    pub fn add_effect(&mut self, asset: &EmitterAsset, textures: &mut dyn TextureSource) -> EffectId {
        let id = EffectId(self.next_effect);
        self.next_effect += 1;
        self.reserve_materials(asset.parts.iter().zip(&asset.definition.sprites).map(|(p, e)| p.draws.iter().map(|d| d.material.passes.len() as u32).sum::<u32>() * copies(e)).sum());
        let draws = asset.parts.iter().zip(&asset.definition.sprites).map(|(part, e)| {
            let mut output = Vec::new();
            for (source_index, source) in part.draws.iter().enumerate() {
                for _ in 0..copies(e) {
                    if let Some(mut gpu) = self.gpu_draw_at(source, textures, usize::MAX, u32::from(source.zone), None) {
                        gpu.source = source_index;
                        output.push(gpu);
                    }
                }
            }
            output
        }).collect();
        let mut system = ParticleSystem::new(&asset.definition, self.corona_random);
        system.seeded_random = self.particle_seeded_random;
        self.corona_random = system.random;
        let ribbons = ribbon_emitters(&asset.definition);
        self.particle_actors.push(GpuEmitter { asset: asset.clone(), system, draws, tile: usize::MAX, effect: Some(Effect { id, speed_rate: 1.0, visible: false }), ribbons, ribbon_input: None });
        id
    }

    pub fn place_effect(&mut self, id: EffectId, placement: EffectPlacement) {
        let zone = self.zone_base_at(placement.location) + u32::from(placement.zone);
        let Some(actor) = self.particle_actors.iter_mut().find(|a| a.effect.is_some_and(|e| e.id == id)) else { return };
        actor.system.actor.location = placement.location.to_array();
        actor.system.actor.rotation = placement.rotation;
        if let Some(effect) = &mut actor.effect {
            effect.visible = placement.visible;
        }
        for draw in actor.draws.iter_mut().flatten() {
            draw.zone = zone;
            draw.material.context.zone = zone;
        }
    }

    pub fn set_effect_ribbon(&mut self, id: EffectId, input: RibbonInput) {
        if let Some(actor) = self.particle_actors.iter_mut().find(|a| a.effect.is_some_and(|e| e.id == id)) {
            actor.ribbon_input = Some(input);
        }
    }

    pub fn attach_trailer(&mut self, id: EffectId, trailer: &Trailer, bone_name: &str) {
        let Some(actor) = self.particle_actors.iter_mut().find(|a| a.effect.is_some_and(|e| e.id == id)) else { return };
        let Some(index) = actor.ribbons.iter().position(Option::is_some) else { return };
        if let Some(shape) = actor.system.actor.sprites[index].ribbon.as_mut() {
            shape.bone_offsets = [trailer.start_offset, trailer.end_offset];
            shape.width = trailer.width;
            if let Some(points) = trailer.num_points {
                shape.num_points = points;
            }
            shape.threshold = trailer.threshold;
            shape.threshold2 = trailer.threshold2;
            shape.bone_names = [Some(bone_name.to_owned()), Some(bone_name.to_owned())];
            if let Some(ribbon) = &mut actor.ribbons[index] {
                ribbon.state = RibbonState::new(shape);
                ribbon.state.spawn(shape);
                ribbon.params.start_offset = trailer.start_offset;
                ribbon.params.end_offset = trailer.end_offset;
                ribbon.params.width = trailer.width;
                ribbon.params.has_start_name = true;
            }
        }
    }

    pub fn set_effect_speed_rate(&mut self, id: EffectId, speed_rate: f32) {
        if let Some(effect) = self.particle_actors.iter_mut().filter_map(|a| a.effect.as_mut()).find(|e| e.id == id) {
            effect.speed_rate = speed_rate;
        }
    }

    pub fn set_effect_visible(&mut self, id: EffectId, visible: bool) {
        if let Some(effect) = self.particle_actors.iter_mut().filter_map(|a| a.effect.as_mut()).find(|e| e.id == id) {
            effect.visible = visible;
        }
    }

    pub fn remove_effect(&mut self, id: EffectId) {
        let Some(index) = self.particle_actors.iter().position(|a| a.effect.is_some_and(|e| e.id == id)) else { return };
        let actor = self.particle_actors.remove(index);
        for draw in actor.draws.into_iter().flatten() { for pass in draw.material.passes { self.textures.release_material(pass.slot); self.materials.free.push(pass.slot); } }
    }

    pub fn effect_finished(&self, id: EffectId) -> bool {
        self.particle_actors.iter().find(|a| a.effect.is_some_and(|e| e.id == id)).is_none_or(|a| a.system.destroyed)
    }

    pub fn effect_count(&self) -> usize {
        self.particle_actors.iter().filter(|a| a.effect.is_some()).count()
    }

    pub fn l2_event_trigger(&mut self, id: i32, on: i32) {
        for actor in &mut self.particle_actors {
            let commands: Vec<u8> = ue2_level::l2_event_commands(&actor.asset.definition.l2_events, id).collect();
            for cmd in commands {
                actor.system.l2_event(cmd, on);
            }
        }
    }

    pub fn l2_event_actor_count(&self) -> usize {
        self.l2_event_actors.len()
    }

    pub fn find_l2_event_actors(&mut self, tag: &str) {
        let mut found = Vec::new();
        for tile in 0..self.tiles.len() {
            let map = self.particle_actors.iter().filter(|actor| actor.effect.is_none() && actor.tile == tile);
            for (nth, actor) in map.enumerate() {
                if actor.asset.definition.tag.as_deref().is_some_and(|own| own.eq_ignore_ascii_case(tag)) {
                    found.push((tile, nth));
                }
            }
        }
        self.l2_event_actors.extend(found);
    }

    pub fn exec_l2_event_actors(&mut self, tag: &str, location: [f32; 3], radius: f32) {
        let smoke = tag.eq_ignore_ascii_case(L2_EVENT_SMOKE);
        for (tile, nth) in self.l2_event_actors.clone() {
            let Some(index) = self.particle_actors.iter().enumerate().filter(|(_, actor)| actor.effect.is_none() && actor.tile == tile).nth(nth).map(|(index, _)| index) else { continue };
            let actor = &mut self.particle_actors[index];
            if !actor.asset.definition.tag.as_deref().is_some_and(|own| own.eq_ignore_ascii_case(tag)) {
                continue;
            }
            if smoke && radius != 0.0 {
                if location == [0.0; 3] {
                    continue;
                }
                let chance = smoke_chance(actor.system.actor.location, location, radius);
                if crate::coronas::app_frand(&mut self.corona_random) >= chance {
                    continue;
                }
            }
            actor.system.random = self.corona_random;
            actor.system.trigger();
            self.corona_random = actor.system.random;
        }
    }

    pub fn l2_event_smoke(&mut self, location: [f32; 3], radius: f32) {
        if self.l2_event_actors.is_empty() {
            self.find_l2_event_actors(L2_EVENT_SMOKE);
            return;
        }
        self.exec_l2_event_actors(L2_EVENT_SMOKE, location, radius);
    }

    pub(super) fn upload_particles(&mut self, assets: &[EmitterAsset], textures: &mut dyn TextureSource, tile: usize) {
        self.reserve_materials(assets.iter().flat_map(|a| a.parts.iter().zip(&a.definition.sprites)).map(|(p, e)| p.draws.iter().map(|d| d.material.passes.len() as u32).sum::<u32>() * copies(e)).sum());
        for asset in assets {
            let draws = asset.parts.iter().zip(&asset.definition.sprites).map(|(part, e)| {
                let mut output = Vec::new();
                for (source_index, source) in part.draws.iter().enumerate() {
                    for _ in 0..copies(e) {
                        if let Some(mut gpu) = self.gpu_draw(source, textures, tile) {
                            gpu.source = source_index;
                            output.push(gpu);
                        }
                        for (projector, material) in part.projector_materials.get(source_index).into_iter().flatten() {
                            let mut projected = source.clone();
                            projected.material = material.clone();
                            projected.kind = killer_assets::scene::DrawKind::Projector;
                            projected.pass = Pass::Projector;
                            if let Some(mut gpu) = self.gpu_draw(&projected, textures, tile) {
                                gpu.projector = Some(*projector);
                                gpu.source = source_index;
                                output.push(gpu);
                            }
                        }
                    }
                }
                output
            }).collect();
            let mut system = ParticleSystem::new(&asset.definition, self.corona_random);
            system.seeded_random = self.particle_seeded_random;
            self.corona_random = system.random;
            let ribbons = ribbon_emitters(&asset.definition);
            self.particle_actors.push(GpuEmitter { asset: asset.clone(), system, draws, tile, effect: None, ribbons, ribbon_input: None });
        }
    }

    pub(super) fn remove_particle_tile(&mut self, index: usize) {
        if self.l2_event_actors.iter().any(|&(tile, _)| tile == index) {
            self.l2_event_actors.clear();
        }
        for (tile, _) in &mut self.l2_event_actors {
            if *tile > index {
                *tile -= 1;
            }
        }
        let mut kept = Vec::new();
        for mut actor in self.particle_actors.drain(..) {
            if actor.effect.is_none() && actor.tile == index {
                for draw in actor.draws.into_iter().flatten() { for pass in draw.material.passes { self.textures.release_material(pass.slot); self.materials.free.push(pass.slot); } }
            } else {
                if actor.effect.is_none() && actor.tile > index {
                    actor.tile -= 1;
                    for draw in actor.draws.iter_mut().flatten() {
                        draw.tile -= 1;
                        draw.zone -= self.tiles[index].zone_count as u32;
                        draw.material.context.zone = draw.zone;
                    }
                }
                kept.push(actor);
            }
        }
        self.particle_actors = kept;
    }

    pub(super) fn prepare_particles(&mut self, camera: &Camera, aspect: f32) {
        let dt = self.particle_delta.unwrap_or_else(|| self.viewer.map_or((self.time - self.particle_time).max(0.0), |v| v.dt));
        self.particle_time = self.time;
        let target = self.view_target.unwrap_or(camera.eye);
        for actor in &mut self.particle_actors {
            let visible = emitter_visible(&self.tiles, actor, target);
            actor.system.random = self.corona_random;
            actor.system.seeded_random = self.particle_seeded_random;
            actor.system.tick(dt * actor.effect.map_or(1.0, |effect| effect.speed_rate), visible, &actor.asset, &mut EmptyWorld);
            for (i, draws) in actor.draws.iter_mut().enumerate() {
                let definition = &actor.system.actor.sprites[i];
                let own_blend = definition.mesh.as_ref().or(definition.vertex_mesh.as_ref()).is_some_and(|m| m.own_blend);
                let copies = copies(definition) as usize;
                let bounds = actor.asset.parts[i].mesh_bounds.and_then(|mesh| crate::particles::mesh_bounds(definition, &actor.system.actor, &actor.system.states[i].particles[..actor.system.states[i].active], mesh));
                for (section, draw) in draws.iter_mut().enumerate() {
                    draw.index_count = 0;
                    if !visible || actor.system.destroyed || definition.disabled { continue; }
                    let source = &actor.asset.parts[i].draws[draw.source];
                    let mut vertices = Vec::<Vertex>::new();
                    let mut indices = Vec::<u32>::new();
                    let state = &mut actor.system.states[i];
                    let mut vertex_pose = actor.asset.parts[i].vertex_mesh.as_ref().map(|mesh| ue2_assets::VertexPose::new(&mesh.mesh));
                    let mut factor = [1.0; 4];
                    if let (Some(shape), Some(Some(ribbon))) = (definition.ribbon.as_ref(), actor.ribbons.get_mut(i)) {
                        let Some(particle) = state.particles[..state.active].first_mut().filter(|p| p.alive()) else { continue };
                        particle.location = Vec3::from(actor.system.actor.location);
                        let host = ribbon_point::Host {
                            actor: ribbon_point::Actor { location: actor.system.actor.location, rotation: actor.system.actor.rotation },
                            owner: actor.ribbon_input.and_then(|input| input.owner),
                            owner_mesh: actor.ribbon_input.is_some_and(|input| input.owner_mesh),
                        };
                        let input = actor.ribbon_input.unwrap_or(RibbonInput { start: None, end: None, owner: None, owner_mesh: false, instance: false });
                        let mut warnings = Vec::new();
                        let sample = ribbon_point::get_new_ribbon_point(&mut ribbon.params, Some(&host), input.instance, input.start, input.end, &mut warnings, shape.width);
                        ribbon.warnings += warnings.len();
                        let point = RibbonPoint { edges: [Vec3::from(sample.a), Vec3::from(sample.b)], width: sample.width, ..Default::default() };
                        ribbon.state.update(shape, dt, false, particle.time, particle.lifetime, point);
                        let (mut ribbon_vertices, ribbon_indices) = ribbon.state.vertices(shape, false, particle.color);
                        if ribbon_indices.is_empty() { continue; }
                        if ribbon.params.coordinate_system == ribbon_point::COORDINATE_RELATIVE {
                            let reference = match host.owner {
                                Some(owner) if matches!(ribbon.params.axis, ribbon_point::AXIS_BONE_TO_BONE | ribbon_point::AXIS_START_BONE_DIRECTION) => owner,
                                _ => host.actor,
                            };
                            let rotation = rotator_matrix(reference.rotation);
                            let location = Vec3::from(reference.location);
                            for vertex in &mut ribbon_vertices {
                                vertex.position = (rotation * Vec3::from(vertex.position) + location).to_array();
                            }
                        }
                        vertices = ribbon_vertices;
                        indices = ribbon_indices;
                    } else {
                    for (_, p) in state.particles[..state.active].iter_mut().enumerate().filter(|(slot, p)| p.alive() && (!own_blend || *slot == section % copies)) {
                        let base = vertices.len() as u32;
                        if let Some(shape) = definition.mesh.as_ref().or(definition.vertex_mesh.as_ref()) {
                            let matrix = mesh_matrix(definition, &actor.system.actor, p);
                            let normal = Mat3::from_mat4(matrix).inverse().transpose();
                            let color = p.color.map(|c| f32::from(c) / 255.0);
                            factor = color;
                            let animated = if let (Some(pose), Some(mesh)) = (&mut vertex_pose, &actor.asset.parts[i].vertex_mesh) {
                                match pose.sample(state.vertex_animation.sequence, p.vertex_frame, mesh.mesh.vertex_count) {
                                    Ok(points) => Some(points),
                                    Err(_) => continue,
                                }
                            } else { None };
                            vertices.extend(source.vertices.iter().enumerate().map(|(index, v)| {
                                let mut v = *v;
                                if let (Some(points), Some(mesh)) = (animated, &actor.asset.parts[i].vertex_mesh) {
                                    let point = points[usize::from(mesh.mesh.wedges[index].vertex)];
                                    v.position = point.position;
                                    v.normal = point.normal;
                                }
                                Vertex {
                                position: matrix.transform_point3(Vec3::from(v.position)).to_array(),
                                normal: (normal * Vec3::from(v.normal)).normalize_or_zero().to_array(),
                                color: if !shape.own_blend { color } else { [0.0, 0.0, 0.0, 1.0] }, ..v
                            }}));
                            indices.extend(source.indices.iter().map(|v| *v + base));
                        } else if definition.beam.is_some() {
                            let (v, ix) = beam_vertices(definition, &actor.system.actor, p, camera.eye, &mut state.noise_time, &mut actor.system.random);
                            vertices.extend(v);
                            indices.extend(ix.into_iter().map(|v| v + base));
                        } else {
                            vertices.extend(sprite_vertices(definition, &actor.system.actor, p, camera, aspect));
                            indices.extend([base, base + 1, base + 2, base + 1, base + 3, base + 2]);
                        }
                    }
                    }
                    if indices.is_empty() { continue; }
                    if own_blend && let Some(map) = &actor.asset.lighting && let Some(bounds) = bounds {
                        use killer_assets::scene::{actor_lighting::Query, lights::{hardware_attenuation, hardware_cone}};
                        let center = bounds.center();
                        let radius = bounds.extent().length();
                        let sample = map.sample(Query { position: Vec3::from(actor.system.actor.location), center, radius, bounds }, 0, self.time);
                        for vertex in &mut vertices {
                            let position = Vec3::from(vertex.position);
                            let normal = Vec3::from(vertex.normal);
                            let mut color = Vec3::ZERO;
                            for light in &sample.lights {
                                let offset = light.position - position;
                                let distance = offset.length();
                                if distance >= light.radius || !hardware_cone(light, position) { continue; }
                                let [a, b] = hardware_attenuation(light, center, radius);
                                let lambert = normal.dot(offset.normalize_or_zero()).max(0.0);
                                color += Vec3::from(light.color) * (0.5 * lambert / (a + b * distance));
                            }
                            vertex.color = color.extend(1.0).to_array();
                        }
                    }
                    write_buffer(&self.device, &self.queue, &mut draw.vertices, bytemuck::cast_slice(&vertices), wgpu::BufferUsages::VERTEX);
                    write_buffer(&self.device, &self.queue, &mut draw.indices, bytemuck::cast_slice(&indices), wgpu::BufferUsages::INDEX);
                    draw.index_count = indices.len() as u32;
                    draw.center = Vec3::from(actor.system.actor.location);
                    draw.radius = vertices.iter().map(|v| Vec3::from(v.position).distance(draw.center)).fold(0.0, f32::max);
                    for pass in &draw.material.passes {
                        let mut gpu = MaterialGpu::new(&draw.material.material, &draw.material.material.passes[pass.pass], &draw.material.context, state.elapsed);
                        gpu.padding[0] = u32::from(definition.disable_fogging || actor.system.actor.disable_fogging);
                        if own_blend { gpu.tfactor = factor; }
                        self.materials.write(&self.queue, pass.slot, &gpu);
                    }
                }
            }
            self.corona_random = actor.system.random;
            self.particle_seeded_random = actor.system.seeded_random;
        }
    }
}

fn copies(e: &ue2_level::SpriteEmitter) -> u32 { if e.mesh.as_ref().or(e.vertex_mesh.as_ref()).is_some_and(|m| m.own_blend) { e.max_particles } else { 1 } }

fn write_buffer(device: &wgpu::Device, queue: &wgpu::Queue, buffer: &mut wgpu::Buffer, bytes: &[u8], usage: wgpu::BufferUsages) {
    if buffer.size() < bytes.len() as u64 || !buffer.usage().contains(wgpu::BufferUsages::COPY_DST) {
        *buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("partículas"), size: (bytes.len().max(4) as u64).next_power_of_two(), usage: usage | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    }
    queue.write_buffer(buffer, 0, bytes);
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_assets::scene::Scene;
    use killer_assets::scene::emitters::EmitterPart;
    use killer_assets::textures::{TextureData, TextureFormatKind, checker_image};
    use std::sync::Arc;
    use ue2_level::{Emitter, L2Event, LEC_PLAY, LEC_SHOW, SpriteEmitter};

    struct Checker;

    impl TextureSource for Checker {
        fn texture(&mut self, _: &str) -> Arc<TextureData> {
            Arc::new(TextureData { width: 8, height: 8, format: TextureFormatKind::Rgba8, mips: vec![checker_image().rgba] })
        }

        fn alpha_map(&mut self, name: &str) -> Arc<TextureData> {
            self.texture(name)
        }
    }

    fn emitter(location: [f32; 3], tag: Option<&str>, l2_events: Vec<L2Event>, disabled: bool) -> EmitterAsset {
        let sprite = SpriteEmitter { class: "SpriteEmitter".into(), disabled, max_particles: 2, spawn_on_trigger: [1.0, 1.0], ..Default::default() };
        let definition = Emitter { location, tag: tag.map(str::to_owned), sprites: vec![sprite], l2_events, ..Default::default() };
        EmitterAsset { definition, parts: vec![EmitterPart::default()], zone: 0, range: 12288.0, lighting: None }
    }

    fn disabled(renderer: &Renderer) -> Vec<bool> {
        renderer.particle_actors.iter().map(|actor| actor.system.actor.sprites[0].disabled).collect()
    }

    #[test]
    fn smoke_chance_is_the_hyperbolic_secant_of_distance_over_radius() {
        assert_eq!(smoke_chance([0.0; 3], [0.0; 3], 1800.0), 1.0);
        let t = 1.0f64;
        assert_eq!(smoke_chance([1800.0, 0.0, 0.0], [0.0; 3], 1800.0), (2.0 / (t.exp() + (-t).exp())) as f32);
        assert!((smoke_chance([0.0, 3000.0, 4000.0], [0.0; 3], 5000.0) - 0.648_054_3).abs() < 1e-6);
        assert!(smoke_chance([1e6, 0.0, 0.0], [0.0; 3], 1800.0) < 1e-30);
    }

    #[test]
    fn l2_event_trigger_10556180_dispatches_show_to_every_matching_emitter_of_every_tile() {
        let Ok(mut headless) = crate::headless::Headless::new(16, 16) else { return };
        let renderer = &mut headless.renderer;
        let show = |id| L2Event { id, cmd: LEC_SHOW };
        let first = Scene { emitters: vec![emitter([0.0; 3], None, vec![show(2_418_001)], false), emitter([0.0; 3], None, vec![], false)], ..Scene::default() };
        let second = Scene { emitters: vec![emitter([0.0; 3], None, vec![L2Event { id: 2_418_001, cmd: LEC_PLAY }, show(2_418_002), show(2_418_001)], false)], ..Scene::default() };
        renderer.add_tile("24_18", &first, &mut Checker);
        renderer.add_tile("24_19", &second, &mut Checker);
        renderer.l2_event_trigger(2_418_001, 0);
        assert_eq!(disabled(renderer), [true, false, true]);
        renderer.l2_event_trigger(2_418_002, 1);
        assert_eq!(disabled(renderer), [true, false, false]);
        renderer.l2_event_trigger(2_418_001, -128);
        assert_eq!(disabled(renderer), [false, false, false]);
        renderer.l2_event_trigger(7, 0);
        assert_eq!(disabled(renderer), [false, false, false]);
    }

    #[test]
    fn view_shake_12211_finds_the_tagged_emitters_first_and_triggers_them_by_chance_after() {
        let Ok(mut headless) = crate::headless::Headless::new(16, 16) else { return };
        let renderer = &mut headless.renderer;
        let near = emitter([100.0, 0.0, 0.0], Some("AntarasCave_Smoke"), vec![], true);
        let far = emitter([1e6, 0.0, 0.0], Some(L2_EVENT_SMOKE), vec![], true);
        let other = emitter([0.0; 3], Some("Smoke"), vec![], true);
        renderer.add_tile("25_21", &Scene { emitters: vec![near, other, far], ..Scene::default() }, &mut Checker);
        renderer.l2_event_smoke([0.0, 0.0, 0.0], 1800.0);
        assert_eq!(renderer.l2_event_actors, [(0, 0), (0, 2)]);
        assert_eq!(disabled(renderer), [true, true, true]);
        renderer.l2_event_smoke([0.0, 0.0, 0.0], 1800.0);
        assert_eq!(disabled(renderer), [true, true, true]);
        let mut random = renderer.corona_random;
        let roll = crate::coronas::app_frand(&mut random);
        renderer.l2_event_smoke([1.0, 0.0, 0.0], 1800.0);
        let chance = smoke_chance([100.0, 0.0, 0.0], [1.0, 0.0, 0.0], 1800.0);
        assert_eq!(disabled(renderer)[0], roll >= chance);
        assert_eq!(disabled(renderer)[1..], [true, true]);
        renderer.remove_tile("25_21");
        assert!(renderer.l2_event_actors.is_empty());
    }

    #[test]
    fn exec_l2_event_actors_105a27b0_triggers_without_chance_for_another_tag_or_zero_radius() {
        let Ok(mut headless) = crate::headless::Headless::new(16, 16) else { return };
        let renderer = &mut headless.renderer;
        let tagged = |tag| emitter([1e6, 0.0, 0.0], Some(tag), vec![], true);
        renderer.add_tile("a", &Scene::default(), &mut Checker);
        renderer.add_tile("b", &Scene { emitters: vec![tagged("Gate"), tagged(L2_EVENT_SMOKE)], ..Scene::default() }, &mut Checker);
        renderer.find_l2_event_actors("gate");
        renderer.find_l2_event_actors(L2_EVENT_SMOKE);
        assert_eq!(renderer.l2_event_actors, [(1, 0), (1, 1)]);
        renderer.exec_l2_event_actors("Gate", [5.0; 3], 1800.0);
        assert_eq!(disabled(renderer), [false, true]);
        assert_eq!(renderer.particle_actors[0].system.states[0].trigger_count, 1);
        renderer.exec_l2_event_actors(L2_EVENT_SMOKE, [5.0; 3], 0.0);
        assert_eq!(disabled(renderer), [false, false]);
        renderer.remove_tile("a");
        assert_eq!(renderer.l2_event_actors, [(0, 0), (0, 1)]);
        assert_eq!(renderer.l2_event_actor_count(), 2);
    }
}
