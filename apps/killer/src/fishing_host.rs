use crate::characters::{CharacterCache, EffectModifiers, WorldSpawn, world_matrix};
use glam::{Mat4, Vec3};
use killer_assets::fishing_float::{self, FloatActor, FloatAsset, LINE_FLOAT_BONE, STATE_CAST, STATE_COMBAT};
use killer_assets::textures::{BankSource, TextureBank};
use killer_render::lines::Line;
use killer_render::renderer::Renderer;
use killer_render::skinned::{CharacterId, CharacterInstance};
use killer_world::fishing_line::{self, SpringSimul};
use killer_world::fishing_view::FishingView;
use std::sync::Arc;

pub const FLOAT_ACTOR_ID: i32 = -0x4c32;

#[derive(Default)]
pub struct FishingHost {
    asset: Option<Arc<FloatAsset>>,
    failed: bool,
    uploaded: Option<CharacterId>,
    bank: TextureBank,
    actor: Option<FloatActor>,
    animation: String,
    clock: f32,
    spring: Option<SpringSimul>,
    alpha: u8,
    was_combat: bool,
}

#[derive(Clone, Copy)]
pub struct FishingContext {
    pub me: i32,
    pub camera: Vec3,
    pub zone: u8,
    pub dt: f32,
}

pub struct FishingFrame {
    pub instance: Option<CharacterInstance>,
    pub lines: Vec<Line>,
}

impl FishingHost {
    pub fn reset(&mut self) {
        self.actor = None;
        self.spring = None;
        self.alpha = 0;
        self.clock = 0.0;
        self.was_combat = false;
    }

    pub fn update(
        &mut self,
        view: &mut FishingView,
        client: &mut killer_assets::client::Client,
        renderer: &mut Renderer,
        cache: &mut CharacterCache,
        context: FishingContext,
    ) -> FishingFrame {
        let FishingContext { me, camera, zone, dt } = context;
        let empty = FishingFrame { instance: None, lines: Vec::new() };
        if !view.active {
            self.reset();
            return empty;
        }
        let Some(float) = view.float else { return empty };
        if self.asset.is_none() && !self.failed {
            match fishing_float::load(client) {
                Some(asset) => self.asset = Some(Arc::new(asset)),
                None => {
                    log::warn!("flutuador {}: malha, animação ou nomes ausentes", fishing_float::MESH);
                    self.failed = true;
                }
            }
        }
        let Some(asset) = self.asset.clone() else { return empty };
        let id = match self.uploaded {
            Some(id) => id,
            None => {
                let id = renderer.upload_character(&asset.mesh, &mut BankSource { bank: &mut self.bank, catalog: client.catalog() });
                self.uploaded = Some(id);
                id
            }
        };
        let names = &asset.names;
        let actor = self.actor.get_or_insert_with(|| FloatActor::cast(view.kind, float));
        if self.animation.is_empty() || (actor.state == STATE_CAST && !self.was_combat && self.clock == 0.0) {
            self.animation = actor.cur_anim_name(names).unwrap_or_default().to_owned();
        }
        if view.calc_step == killer_world::fishing_view::CALC_STEP_COMBAT && actor.state == STATE_CAST {
            actor.state = STATE_COMBAT;
            actor.gut = view.gut;
            self.was_combat = true;
            self.animation = actor.cur_anim_name(names).unwrap_or_default().to_owned();
            self.clock = 0.0;
        }
        if let Some(action) = view.pull.take() {
            actor.elapsed = 0.0;
            actor.mode = i32::from(action);
        }
        if actor.state == STATE_COMBAT && actor.gut != view.gut {
            actor.gut = view.gut;
            self.animation = actor.cur_anim_name(names).unwrap_or_default().to_owned();
            self.clock = 0.0;
        }
        self.clock += dt;
        let heading = view.pawn_rotation[1];
        let (posed, skin) = fishing_float::float_pose(&asset.mesh, &asset.animation, &self.animation, self.clock);
        let ring_point = {
            let world = world_matrix(actor.location, heading, 1.0, asset.mesh.local);
            fishing_float::bone_position(&posed, world, LINE_FLOAT_BONE)
        };
        let output = actor.tick(dt, names, false, ring_point);
        if let Some(name) = output.play {
            self.animation = name;
            self.clock = 0.0;
        }
        if let Some(ring) = output.ring {
            cache.push_world_spawn(WorldSpawn {
                class: fishing_float::RING_CLASS.to_owned(),
                location: ring.location,
                rotation: [0, heading, 0],
                zone,
                modifiers: EffectModifiers { speed_rate: Some(ring.speed_scale), size_scale: Some(ring.size_scale), ..Default::default() },
                owner: None,
                projectile: None,
                trailer_physics: None,
            });
        }
        let world: Mat4 = world_matrix(actor.location, heading, 1.0, asset.mesh.local);
        let lighting = cache.instance_lighting(me);
        let instance = CharacterInstance {
            actor_id: FLOAT_ACTOR_ID,
            actor_position: actor.location,
            collision_height: 0.0,
            shadow_caster: false,
            light_sphere: lighting.map_or([actor.location.x, actor.location.y, actor.location.z, 0.0], |l| l.light_sphere),
            mesh: id,
            world,
            palette: skin,
            zone: lighting.map_or(zone, |l| l.zone),
            lights: lighting.map(|l| l.lights.clone()).unwrap_or_default(),
            pawn_lights: lighting.map(|l| l.pawn_lights.clone()).unwrap_or_default(),
            ambient: lighting.map_or([255; 3], |l| l.ambient),
            sun_affect: lighting.is_some_and(|l| l.sun_affect),
            overlay: [0.0; 4],
        };
        let mut lines = Vec::new();
        let ends = fishing_float::bone_position(&posed, world, LINE_FLOAT_BONE).zip(cache.rod_point(me));
        if let Some((start, end)) = ends {
            let spring = self.spring.get_or_insert_with(|| SpringSimul::init(start, end, fishing_line::SEGMENTS));
            spring.calc_position(start, end);
            self.alpha = fishing_line::fade_tick(self.alpha, dt);
            let location = view.line_middle.unwrap_or(float);
            if let Some(ribbon) = fishing_line::ribbon(spring, camera, self.alpha, location) {
                let color = [0xc8, 0xc8, 0xc8, self.alpha];
                for side in 0..2 {
                    let points: Vec<Vec3> = ribbon.vertices.iter().skip(side).step_by(2).map(|vertex| vertex.position).collect();
                    lines.extend(points.windows(2).map(|pair| Line { start: pair[0], end: pair[1], color }));
                }
            }
        }
        FishingFrame { instance: Some(instance), lines }
    }
}
