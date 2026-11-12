use crate::engine_math::{Coords, coords_div_rotator, coords_mul_rotator, sin_tab, transform_point_by, transform_vector_by};
use crate::scene::lights::PointLight;
use crate::scene::transform::vector_rotation;
use glam::Vec3;

pub const LIGHT_TYPE_STEADY: u8 = 1;
pub const LIGHT_TYPE_PULSE: u8 = 2;
pub const EFFECT_STATIC_SPOT: u8 = 8;
pub const EFFECT_SPOTLIGHT: u8 = 12;
pub const EFFECT_POINT: u8 = 0x11;
pub const EFFECT_SUNLIGHT: u8 = 0x13;
pub const ATTACH_WORLD: u8 = 0;
pub const ATTACH_ACTOR: u8 = 1;
pub const ATTACH_AIMED: u8 = 3;
pub const KIND_WORLD: i32 = 0;
pub const KIND_ACTOR: i32 = 1;
pub const KIND_SKILL: u8 = 2;
pub const MAX_HARDWARE_INDEX: usize = 8;
pub const SHOT_LIGHT_SKILLS: [i32; 2] = [0x11, 0x51];

const PERIOD_UNITS: f32 = f32::from_bits(0x4a0c_0000);
const PULSE_AMPLITUDE: f32 = f32::from_bits(0x3ec7_ae14);
const PULSE_BASE: f32 = f32::from_bits(0x3f19_999a);
const LIFE_EPSILON: f32 = f32::from_bits(0x38d1_b717);
const RADIUS_SCALE: f32 = f32::from_bits(0x41c8_0000);
const RADIUS_BIAS: f32 = 1.0;
const FTOL_LIMIT: f64 = 9_223_372_036_854_775_808.0;
const ADD_LIFE: f32 = f32::from_bits(0x3e4c_cccd);
const ADD_RADIUS: f32 = f32::from_bits(0x41f0_0000);
const ADD_PLANE: [f32; 4] = [1.0; 4];
const HIT_BACK: [f32; 3] = [-1.0, 0.0, 0.0];
const SHOT_FORWARD: [f32; 3] = [1.0, 0.0, 0.0];
const SHOT_LIFE: f32 = f32::from_bits(0x3e99_999a);
const SHOT_PLANE: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const ABNORMAL_AIM: [f32; 3] = [-1.0, 0.0, -1.0];
const ABNORMAL_LIFE: f32 = f32::from_bits(0x3f33_3333);
const ABNORMAL_PERIOD: u8 = 0x19;
const ABNORMAL_PHASE: u8 = 0xc0;
const ABNORMAL_RED: [f32; 4] = [f32::from_bits(0x3ec0_c0c1), 0.0, 0.0, 1.0];
const ABNORMAL_PURPLE: [f32; 4] = [f32::from_bits(0x3e80_8081), 0.0, f32::from_bits(0x3ec0_c0c1), 1.0];
const SWEEP_LIFE: f32 = f32::from_bits(0x4120_0000);
const SWEEP_PLANE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const SWEEP_PERIOD: u8 = 0x19;
const SWEEP_PHASE: u8 = 0xc0;
const PHASE_KIND: i32 = 2;
const PHASE_RADIUS: f32 = f32::from_bits(0x41f0_0000);
const PHASE_SHOT_LIFE: f32 = f32::from_bits(0x3f00_0000);
const PHASE_SHOT_SMALL_RADIUS: f32 = f32::from_bits(0x41a0_0000);
const PHASE_SHOT_HEIGHT: f32 = f32::from_bits(0x3fc0_0000);
const LIGHT_TYPE_SHOT: u8 = 7;
const INIT_BLUE: [f32; 4] = [f32::from_bits(0x3e99_999a), f32::from_bits(0x3e99_999a), f32::from_bits(0x3f66_6666), 1.0];
const INIT_ROSE: [f32; 4] = [f32::from_bits(0x3f33_3333), f32::from_bits(0x3e99_999a), f32::from_bits(0x3e99_999a), 1.0];
const INIT_GREY: [f32; 4] = [0.5, 0.5, 0.5, 0.5];
const INIT_RED: [f32; 4] = [f32::from_bits(0x3f66_6666), f32::from_bits(0x3e4c_cccd), f32::from_bits(0x3e4c_cccd), 0.5];
const INIT_DARK_RED: [f32; 4] = [f32::from_bits(0x3f33_3333), 0.0, 0.0, 1.0];
const INIT_DEEP_RED: [f32; 4] = [f32::from_bits(0x3f19_999a), 0.0, 0.0, 1.0];
const SHOT_WHITE: [f32; 4] = [f32::from_bits(0x3f66_6666), f32::from_bits(0x3f66_6666), f32::from_bits(0x3f66_6666), f32::from_bits(0x3f33_3333)];

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Actor {
    pub location: [f32; 3],
    pub rotation: [i32; 3],
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LightActor {
    pub class: &'static str,
    pub location: [f32; 3],
    pub rotation: [i32; 3],
    pub pre_pivot: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrailerFollow {
    pub pre_pivot: bool,
    pub same_rotation: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhaseLightList {
    Caster,
    Target,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhaseLightAim {
    Caster,
    CasterHead,
    CasterAtTargetHeight,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PhaseLightLife {
    CastTime,
    Seconds(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhaseLight {
    pub spawn: usize,
    pub list: PhaseLightList,
    pub light_type: u8,
    pub effect: u8,
    pub plane: [f32; 4],
    pub life: PhaseLightLife,
    pub radius: f32,
    pub attach: u8,
    pub raised: bool,
    pub aim: PhaseLightAim,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhasePawn {
    pub location: [f32; 3],
    pub height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Init {
    pub kind: i32,
    pub location: [f32; 3],
    pub rotation: [i32; 3],
    pub light_type: u8,
    pub effect: u8,
    pub plane: [f32; 4],
    pub life: f32,
    pub radius: f32,
    pub attach: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PawnLight {
    pub kind: u8,
    pub light_type: u8,
    pub effect: u8,
    pub attach: u8,
    pub color: [f32; 4],
    pub plane: [f32; 4],
    pub location: [f32; 3],
    pub direction: [f32; 3],
    pub radius: f32,
    pub elapsed: f32,
    pub life: f32,
    pub period: u8,
    pub phase: u8,
    pub target: [f32; 3],
    pub revision: u32,
    pub actor: Option<LightActor>,
}

fn ftol(value: f64) -> i32 {
    let truncated = value.trunc();
    if !(-FTOL_LIMIT..FTOL_LIMIT).contains(&truncated) {
        return 0;
    }
    truncated as i64 as i32
}

fn scale_plane(plane: [f32; 4], scale: f32) -> [f32; 4] {
    plane.map(|value| value * scale)
}

fn sum(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [(f64::from(a[0]) + f64::from(b[0])) as f32, (f64::from(a[1]) + f64::from(b[1])) as f32, (f64::from(a[2]) + f64::from(b[2])) as f32]
}

fn difference(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [(f64::from(a[0]) - f64::from(b[0])) as f32, (f64::from(a[1]) - f64::from(b[1])) as f32, (f64::from(a[2]) - f64::from(b[2])) as f32]
}

pub fn rotator_vector(rotation: [i32; 3]) -> [f32; 3] {
    coords_div_rotator(&Coords::UNIT, rotation).x
}

impl PawnLight {
    pub fn init(&mut self, init: &Init) {
        self.kind = init.kind as u8;
        self.light_type = init.light_type;
        self.effect = init.effect;
        self.plane = init.plane;
        self.location = init.location;
        self.direction = rotator_vector(init.rotation);
        self.radius = ((f64::from(init.radius) + f64::from(RADIUS_BIAS)) * f64::from(RADIUS_SCALE)) as f32;
        self.attach = init.attach;
        self.life = init.life;
        self.color = scale_plane(self.plane, self.intensity(0.0));
    }

    pub fn intensity(&self, time: f32) -> f32 {
        match self.light_type {
            LIGHT_TYPE_STEADY => 1.0,
            LIGHT_TYPE_PULSE => {
                let divisor = i32::from(self.period).max(1);
                let units = ftol(f64::from(time) * f64::from(PERIOD_UNITS) / f64::from(divisor) + f64::from(i32::from(self.phase) << 8));
                (f64::from(sin_tab(units)) * f64::from(PULSE_AMPLITUDE) + f64::from(PULSE_BASE)) as f32
            }
            _ => {
                let divisor = i32::from(self.period) + 1;
                let units = ftol(f64::from(time) * f64::from(PERIOD_UNITS) / f64::from(divisor) + f64::from(i32::from(self.phase) << 8));
                if units & 1 != 0 { 0.0 } else { 1.0 }
            }
        }
    }

    pub fn update(&mut self, dt: f32) -> bool {
        if dt == 0.0 || self.life == 0.0 {
            return false;
        }
        let left = f64::from(self.life) - f64::from(dt);
        self.life = left as f32;
        if left <= f64::from(LIFE_EPSILON) {
            return false;
        }
        let elapsed = (f64::from(dt) + f64::from(self.elapsed)) as f32;
        self.elapsed = elapsed;
        self.color = scale_plane(self.plane, self.intensity(elapsed));
        self.revision = self.revision.wrapping_add(1);
        true
    }

    pub fn position(&self, actor: Option<Actor>) -> [f32; 3] {
        match (self.attach, actor) {
            (ATTACH_ACTOR | ATTACH_AIMED, Some(actor)) => sum(actor.location, transform_point_by(self.location, &coords_mul_rotator(&Coords::UNIT, actor.rotation))),
            _ => self.location,
        }
    }

    pub fn aim(&self, actor: Option<Actor>) -> [f32; 3] {
        match (self.attach, actor) {
            (ATTACH_ACTOR, Some(actor)) => transform_point_by(self.direction, &coords_mul_rotator(&Coords::UNIT, actor.rotation)),
            (ATTACH_AIMED, Some(actor)) => {
                let rotation = vector_rotation(Vec3::from(difference(self.target, actor.location)));
                transform_point_by(self.direction, &coords_mul_rotator(&Coords::UNIT, rotation))
            }
            _ => self.direction,
        }
    }

    pub fn hardware(&self, actor: Option<Actor>) -> Option<PointLight> {
        if self.effect == EFFECT_SUNLIGHT {
            return None;
        }
        Some(PointLight {
            source: usize::MAX,
            position: Vec3::from(self.position(actor)),
            radius: self.radius,
            color: [self.color[0], self.color[1], self.color[2]],
            brightness: 0.0,
            direction: Vec3::from(self.aim(actor)),
            effect: self.effect,
            cone: 0,
            special_lit: false,
        })
    }
}

pub fn add_pawn_light(list: &mut Vec<PawnLight>, location: [f32; 3], rotation: [i32; 3], kind: i32) -> Option<&mut PawnLight> {
    if kind != KIND_WORLD && kind != KIND_ACTOR {
        return None;
    }
    let mut light = PawnLight::default();
    light.init(&Init {
        kind,
        location,
        rotation,
        light_type: LIGHT_TYPE_STEADY,
        effect: EFFECT_SPOTLIGHT,
        plane: ADD_PLANE,
        life: ADD_LIFE,
        radius: ADD_RADIUS,
        attach: kind as u8,
    });
    list.insert(0, light);
    list.first_mut()
}

pub fn tick(list: &mut Vec<PawnLight>, dt: f32) {
    let mut index = list.len();
    while index > 0 {
        index -= 1;
        if !list[index].update(dt) {
            list.remove(index);
        }
    }
}

pub fn remove_kind(list: &mut Vec<PawnLight>, kind: u8) {
    let mut index = list.len();
    while index > 0 {
        index -= 1;
        if list[index].kind == kind {
            list.remove(index);
        }
    }
}

pub fn hit_light(attacker: [f32; 3], victim: [f32; 3], victim_radius: f32) -> ([f32; 3], [i32; 3]) {
    let rotation = vector_rotation(Vec3::from(difference(victim, attacker)));
    let back = transform_vector_by(HIT_BACK, &coords_mul_rotator(&Coords::UNIT, rotation));
    let radius = f64::from(victim_radius);
    let x = (f64::from((f64::from(back[0]) * radius) as f32) * 2.0) as f32;
    let y = (f64::from((f64::from(back[1]) * radius) as f32) * 2.0) as f32;
    let z = f64::from(back[2]) * radius * 2.0;
    let location = [(f64::from(victim[0]) + f64::from(x)) as f32, (f64::from(victim[1]) + f64::from(y)) as f32, (f64::from(victim[2]) + z) as f32];
    (location, rotation)
}

pub fn shot_light(caster: [f32; 3], target: [f32; 3], target_radius: f32) -> ([f32; 3], [i32; 3], Init) {
    let rotation = vector_rotation(Vec3::from(difference(target, caster)));
    let forward = transform_vector_by(SHOT_FORWARD, &coords_mul_rotator(&Coords::UNIT, rotation));
    let radius = f64::from(target_radius);
    let x = (f64::from((f64::from(forward[0]) * radius) as f32) * 2.0) as f32;
    let y = (f64::from((f64::from(forward[1]) * radius) as f32) * 2.0) as f32;
    let z = f64::from(forward[2]) * radius * 2.0;
    let location = [(f64::from(target[0]) - f64::from(x)) as f32, (f64::from(target[1]) - f64::from(y)) as f32, (f64::from(target[2]) - z) as f32];
    let init = Init {
        kind: KIND_WORLD,
        location,
        rotation,
        light_type: LIGHT_TYPE_STEADY,
        effect: EFFECT_SPOTLIGHT,
        plane: SHOT_PLANE,
        life: SHOT_LIFE,
        radius: ADD_RADIUS,
        attach: ATTACH_WORLD,
    };
    (location, rotation, init)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AbnormalColor {
    Red,
    Purple,
}

pub fn add_abnormal_light(list: &mut Vec<PawnLight>, radius: f32, height: f32, color: AbnormalColor) {
    let location = [(f64::from(radius) * 2.0) as f32, 0.0, (f64::from(height) * 2.0) as f32];
    let rotation = vector_rotation(Vec3::from(ABNORMAL_AIM));
    let Some(light) = add_pawn_light(list, location, rotation, KIND_ACTOR) else { return };
    light.period = ABNORMAL_PERIOD;
    light.phase = ABNORMAL_PHASE;
    light.init(&Init {
        kind: KIND_ACTOR,
        location,
        rotation,
        light_type: LIGHT_TYPE_PULSE,
        effect: EFFECT_SPOTLIGHT,
        plane: match color {
            AbnormalColor::Red => ABNORMAL_RED,
            AbnormalColor::Purple => ABNORMAL_PURPLE,
        },
        life: ABNORMAL_LIFE,
        radius: (f64::from(radius) * 2.0 + f64::from(height)) as f32,
        attach: ATTACH_ACTOR,
    });
}

pub fn add_sweep_light(list: &mut Vec<PawnLight>, pawn: Actor, height: f32) {
    let location = [pawn.location[0], pawn.location[1], (f64::from(height) * 2.0 + f64::from(pawn.location[2])) as f32];
    let mut light = PawnLight::default();
    light.init(&Init {
        kind: KIND_ACTOR,
        location,
        rotation: pawn.rotation,
        light_type: LIGHT_TYPE_PULSE,
        effect: EFFECT_POINT,
        plane: SWEEP_PLANE,
        life: SWEEP_LIFE,
        radius: ADD_RADIUS,
        attach: ATTACH_WORLD,
    });
    light.target = location;
    light.period = SWEEP_PERIOD;
    light.phase = SWEEP_PHASE;
    list.insert(0, light);
}

const fn init_light(effect: u8, plane: [f32; 4], aim: PhaseLightAim, spawn: usize) -> PhaseLight {
    PhaseLight {
        spawn,
        list: PhaseLightList::Caster,
        light_type: LIGHT_TYPE_STEADY,
        effect,
        plane,
        life: PhaseLightLife::CastTime,
        radius: PHASE_RADIUS,
        attach: ATTACH_AIMED,
        raised: false,
        aim,
    }
}

pub fn cast_phase_light(skill_id: i32) -> Option<PhaseLight> {
    Some(match skill_id {
        0x4e7 => init_light(EFFECT_POINT, INIT_BLUE, PhaseLightAim::CasterHead, 0),
        0x3ed | 0x3ee | 0x3f0 | 0x3f1 | 0x3f2 | 0x4e2 | 0x4e4 | 0x4ec | 0x4ed | 0x504 => init_light(EFFECT_SPOTLIGHT, INIT_ROSE, PhaseLightAim::Caster, 0),
        0x494 | 0x4fa | 0xfc1 | 0xfed | 0x1040 | 0x1062 => init_light(EFFECT_POINT, INIT_GREY, PhaseLightAim::CasterHead, 0),
        0x81 | 0x484 | 0x488 | 0x48b | 0x490 | 0x491 | 0x492 | 0x4ef | 0x4f5 | 0xfc3 | 0xfec | 0x1015 | 0x1017 | 0x1066 | 0x1067 | 0x106b | 0x106d | 0x106e => {
            init_light(EFFECT_POINT, INIT_RED, PhaseLightAim::CasterHead, 0)
        }
        0x44d | 0x454 | 0x1008 | 0x105c => init_light(EFFECT_SPOTLIGHT, INIT_DARK_RED, PhaseLightAim::Caster, 0),
        0x3ea | 0x3eb | 0x3ec | 0x3ef | 0x4cd | 0x4e1 | 0x4e3 | 0x4e5 | 0x4e8 | 0x502 | 0x554 | 0x555 | 0x1001 | 0x1072 => {
            init_light(EFFECT_SPOTLIGHT, INIT_ROSE, PhaseLightAim::Caster, 0)
        }
        0x49d | 0x4c4 | 0x4ce | 0x509 | 0xfba | 0xfca | 0xfcb | 0xfee | 0xff7 | 0x1004 | 0x1012 | 0x103d | 0x103e | 0x1052 | 0x109c | 0x109d | 0x109e => {
            init_light(EFFECT_SPOTLIGHT, INIT_DEEP_RED, PhaseLightAim::Caster, 1)
        }
        0x444 | 0x447 | 0x448 | 0x449 | 0x44b | 0x44e | 0x450 | 0x451 | 0x4b8 | 0x4b9 | 0x4ba | 0x4bd | 0x4de | 0x4df | 0x4e0 | 0x556 | 0x557 | 0x108e => {
            init_light(EFFECT_SPOTLIGHT, INIT_DARK_RED, PhaseLightAim::Caster, 0)
        }
        0x552 | 0x553 | 0x1219 | 0x121f => init_light(EFFECT_SPOTLIGHT, INIT_ROSE, PhaseLightAim::Caster, 0),
        _ => return None,
    })
}

pub fn shot_phase_light(skill_id: i32, stage_two: bool) -> Option<PhaseLight> {
    match skill_id {
        0x1d | 0x5f if stage_two => Some(PhaseLight {
            spawn: 0,
            list: PhaseLightList::Target,
            light_type: LIGHT_TYPE_SHOT,
            effect: EFFECT_POINT,
            plane: INIT_RED,
            life: PhaseLightLife::Seconds(PHASE_SHOT_LIFE),
            radius: PHASE_RADIUS,
            attach: ATTACH_AIMED,
            raised: false,
            aim: PhaseLightAim::CasterAtTargetHeight,
        }),
        0x117 | 0x4f0 | 0x4f1 => Some(PhaseLight {
            spawn: 0,
            list: PhaseLightList::Target,
            light_type: LIGHT_TYPE_SHOT,
            effect: EFFECT_POINT,
            plane: SHOT_WHITE,
            life: PhaseLightLife::Seconds(PHASE_SHOT_LIFE),
            radius: PHASE_SHOT_SMALL_RADIUS,
            attach: ATTACH_ACTOR,
            raised: true,
            aim: PhaseLightAim::Caster,
        }),
        _ => None,
    }
}

pub fn add_phase_light(list: &mut Vec<PawnLight>, light: &PhaseLight, actor: LightActor, caster: PhasePawn, target: PhasePawn, cast_time: f32) {
    let location = if light.raised { [0.0, 0.0, (f64::from(target.height) * f64::from(PHASE_SHOT_HEIGHT)) as f32] } else { [0.0; 3] };
    let mut added = PawnLight::default();
    added.init(&Init {
        kind: PHASE_KIND,
        location,
        rotation: [0; 3],
        light_type: light.light_type,
        effect: light.effect,
        plane: light.plane,
        life: match light.life {
            PhaseLightLife::CastTime => cast_time,
            PhaseLightLife::Seconds(seconds) => seconds,
        },
        radius: light.radius,
        attach: light.attach,
    });
    added.target = match light.aim {
        PhaseLightAim::Caster => caster.location,
        PhaseLightAim::CasterHead => [caster.location[0], caster.location[1], (f64::from(caster.height) + f64::from(caster.location[2])) as f32],
        PhaseLightAim::CasterAtTargetHeight => [caster.location[0], caster.location[1], (f64::from(target.height) + f64::from(caster.location[2])) as f32],
    };
    added.actor = Some(actor);
    list.insert(0, added);
}

pub fn follow_actor(actor: &LightActor, owner: Actor, trailer: Option<TrailerFollow>) -> Actor {
    match trailer {
        Some(follow) => Actor {
            location: if follow.pre_pivot { sum(owner.location, actor.pre_pivot) } else { owner.location },
            rotation: if follow.same_rotation { owner.rotation } else { actor.rotation },
        },
        None => Actor { location: actor.location, rotation: actor.rotation },
    }
}

pub fn hardware_lights<'a>(list: &'a [PawnLight], actor: Option<Actor>, first: usize) -> impl Iterator<Item = PointLight> + 'a {
    hardware_lights_with(list, first, move |_| actor)
}

pub fn hardware_lights_with<'a>(list: &'a [PawnLight], first: usize, actor: impl Fn(&PawnLight) -> Option<Actor> + 'a) -> impl Iterator<Item = PointLight> + 'a {
    list.iter().take(MAX_HARDWARE_INDEX.saturating_sub(first)).filter_map(move |light| light.hardware(actor(light)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn steady() -> PawnLight {
        let mut light = PawnLight::default();
        light.init(&Init {
            kind: 0,
            location: [10.0, 20.0, 30.0],
            rotation: [0, 16384, 0],
            light_type: LIGHT_TYPE_STEADY,
            effect: EFFECT_SPOTLIGHT,
            plane: [1.0, 0.5, 0.25, 1.0],
            life: 0.3,
            radius: 30.0,
            attach: 0,
        });
        light
    }

    #[test]
    fn init_104c5af0_fills_the_fields_and_world_radius() {
        let light = steady();
        assert_eq!(light.radius, 775.0);
        assert_eq!(light.color, [1.0, 0.5, 0.25, 1.0]);
        assert_eq!(light.life, 0.3);
        assert_eq!(light.location, [10.0, 20.0, 30.0]);
        assert!((Vec3::from(light.direction) - Vec3::Y).length() < 1e-6);
        assert_eq!((light.kind, light.light_type, light.effect, light.attach), (0, 1, 12, 0));
    }

    #[test]
    fn update_104c59e0_consumes_life_and_stops_at_the_epsilon() {
        let mut light = steady();
        assert!(light.update(0.1));
        assert_eq!(light.revision, 1);
        assert!((light.elapsed - 0.1).abs() < 1e-7);
        assert!(light.update(0.1));
        assert!(!light.update(0.1));
        let mut zero = steady();
        assert!(!zero.update(0.0));
        let mut dead = steady();
        dead.life = 0.0;
        assert!(!dead.update(0.016));
    }

    #[test]
    fn tick_removes_expired_lights_from_the_back() {
        let mut list = Vec::new();
        add_pawn_light(&mut list, [0.0; 3], [0; 3], KIND_WORLD);
        add_pawn_light(&mut list, [1.0; 3], [0; 3], KIND_ACTOR);
        assert!(add_pawn_light(&mut list, [1.0; 3], [0; 3], 2).is_none());
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].attach, 1);
        list[1].life = 0.05;
        tick(&mut list, 0.1);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].attach, 1);
        tick(&mut list, 0.1);
        tick(&mut list, 0.1);
        assert!(list.is_empty());
    }

    #[test]
    fn get_intensity_104c5900_pulses_and_blinks_with_period_and_phase() {
        let mut light = steady();
        light.light_type = LIGHT_TYPE_PULSE;
        light.period = 0x19;
        light.phase = 0xc0;
        let start = light.intensity(0.0);
        assert!((start - (0.6 - 0.39)).abs() < 1e-6);
        let quarter = light.intensity(25.0 / 35.0 / 4.0);
        assert!((quarter - 0.6).abs() < 1e-3);
        light.light_type = 7;
        light.period = 0;
        light.phase = 0;
        assert_eq!(light.intensity(0.0), 1.0);
        assert_eq!(light.intensity(1.5 / (35.0 * 65536.0)), 0.0);
    }

    #[test]
    fn get_position_and_direction_follow_the_actor_by_attach_mode() {
        let mut light = steady();
        light.attach = ATTACH_ACTOR;
        light.location = [10.0, 0.0, 5.0];
        light.direction = [1.0, 0.0, 0.0];
        let actor = Actor { location: [100.0, 200.0, 300.0], rotation: [0, 16384, 0] };
        let position = Vec3::from(light.position(Some(actor)));
        assert!((position - Vec3::new(100.0, 210.0, 305.0)).length() < 1e-3);
        assert!((Vec3::from(light.aim(Some(actor))) - Vec3::Y).length() < 1e-5);
        assert_eq!(light.position(None), [10.0, 0.0, 5.0]);
        light.attach = ATTACH_AIMED;
        light.target = [100.0, 100.0, 300.0];
        assert!((Vec3::from(light.aim(Some(actor))) + Vec3::Y).length() < 1e-5);
        light.attach = ATTACH_WORLD;
        assert_eq!(light.position(Some(actor)), [10.0, 0.0, 5.0]);
    }

    #[test]
    fn hit_light_sits_two_radii_before_the_victim_facing_it() {
        let (location, rotation) = hit_light([0.0, 0.0, 0.0], [100.0, 0.0, 0.0], 20.0);
        assert_eq!(rotation, [0, 0, 0]);
        assert!((Vec3::from(location) - Vec3::new(60.0, 0.0, 0.0)).length() < 1e-4);
        let (shot, _, init) = shot_light([0.0, 0.0, 0.0], [100.0, 0.0, 0.0], 20.0);
        assert_eq!(shot, location);
        assert_eq!(init.plane, SHOT_PLANE);
    }

    #[test]
    fn abnormal_and_sweep_lights_pulse_with_period_25_and_phase_192() {
        let mut list = Vec::new();
        add_abnormal_light(&mut list, 10.0, 20.0, AbnormalColor::Purple);
        assert_eq!(list[0].location, [20.0, 0.0, 40.0]);
        assert_eq!(list[0].radius, 1025.0);
        assert_eq!((list[0].period, list[0].phase, list[0].attach), (0x19, 0xc0, 1));
        add_sweep_light(&mut list, Actor { location: [1.0, 2.0, 3.0], rotation: [0, 0, 0] }, 10.0);
        assert_eq!(list[0].location, [1.0, 2.0, 23.0]);
        assert_eq!(list[0].life, 10.0);
        assert_eq!(list[0].effect, EFFECT_POINT);
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn cast_phase_lights_cover_the_nine_init_sites_of_skill_effect_init() {
        let lights: Vec<(i32, PhaseLight)> = (0..0x1400).filter_map(|id| cast_phase_light(id).map(|light| (id, light))).collect();
        assert_eq!(lights.len(), 92);
        assert!(lights.iter().all(|(_, light)| light.list == PhaseLightList::Caster && light.life == PhaseLightLife::CastTime && light.attach == ATTACH_AIMED));
        assert_eq!(lights.iter().filter(|(_, light)| light.spawn == 1).count(), 17);
        let blue = cast_phase_light(0x4e7).expect("0x4e7");
        assert_eq!((blue.effect, blue.plane, blue.aim), (EFFECT_POINT, [0.3, 0.3, 0.9, 1.0], PhaseLightAim::CasterHead));
        let deep = cast_phase_light(0x49d).expect("0x49d");
        assert_eq!((deep.effect, deep.plane, deep.spawn, deep.aim), (EFFECT_SPOTLIGHT, [0.6, 0.0, 0.0, 1.0], 1, PhaseLightAim::Caster));
        assert_eq!(cast_phase_light(0x11), None);
    }

    #[test]
    fn phase_lights_are_kind_two_aimed_from_the_effect_and_removable() {
        let mut list = Vec::new();
        add_pawn_light(&mut list, [0.0; 3], [0; 3], KIND_WORLD);
        let caster = PhasePawn { location: [100.0, 200.0, 300.0], height: 40.0 };
        let actor = LightActor { class: "LineageEffect.e_u031_a", location: [0.0; 3], rotation: [0; 3], pre_pivot: [0.0, 0.0, 40.0] };
        add_phase_light(&mut list, &cast_phase_light(0x4e7).expect("0x4e7"), actor, caster, caster, 1.5);
        let light = list[0];
        assert_eq!((light.kind, light.light_type, light.effect, light.attach), (KIND_SKILL, LIGHT_TYPE_STEADY, EFFECT_POINT, ATTACH_AIMED));
        assert_eq!((light.life, light.radius, light.location), (1.5, 775.0, [0.0; 3]));
        assert_eq!(light.target, [100.0, 200.0, 340.0]);
        assert_eq!(light.actor, Some(actor));
        let effect = follow_actor(&actor, Actor { location: [100.0, 200.0, 300.0], rotation: [0, 16384, 0] }, Some(TrailerFollow { pre_pivot: true, same_rotation: false }));
        assert_eq!(effect, Actor { location: [100.0, 200.0, 340.0], rotation: [0; 3] });
        assert_eq!(light.position(Some(effect)), [100.0, 200.0, 340.0]);
        let still = follow_actor(&actor, Actor { location: [100.0, 200.0, 300.0], rotation: [0, 16384, 0] }, None);
        assert_eq!(still, Actor { location: [0.0; 3], rotation: [0; 3] });
        remove_kind(&mut list, KIND_SKILL);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].actor, None);
    }

    #[test]
    fn shot_phase_lights_blink_on_the_target_with_the_engine_parameters() {
        assert_eq!(shot_phase_light(0x1d, false), None);
        let red = shot_phase_light(0x5f, true).expect("0x5f");
        let caster = PhasePawn { location: [100.0, 200.0, 300.0], height: 40.0 };
        let target = PhasePawn { location: [500.0, 700.0, 300.0], height: 60.0 };
        let mut list = Vec::new();
        add_phase_light(&mut list, &red, LightActor::default(), caster, target, 9.0);
        assert_eq!((list[0].light_type, list[0].life, list[0].radius, list[0].target), (7, 0.5, 775.0, [100.0, 200.0, 360.0]));
        assert_eq!(list[0].color, [0.9, 0.2, 0.2, 0.5]);
        let white = shot_phase_light(0x117, false).expect("0x117");
        assert_eq!(white, shot_phase_light(0x117, true).expect("0x117"));
        add_phase_light(&mut list, &white, LightActor::default(), caster, target, 9.0);
        assert_eq!((list[0].attach, list[0].radius, list[0].location, list[0].target), (ATTACH_ACTOR, 525.0, [0.0, 0.0, 90.0], [100.0, 200.0, 300.0]));
        assert_eq!(white.list, PhaseLightList::Target);
        let at = Actor { location: [500.0, 700.0, 300.0], rotation: [0, 16384, 0] };
        assert_eq!(hardware_lights_with(&list, 0, |light| light.actor.map(|_| at)).count(), 2);
    }

    #[test]
    fn hardware_lights_fill_the_slots_left_after_the_level_lights() {
        let mut list = Vec::new();
        for _ in 0..6 {
            add_pawn_light(&mut list, [0.0; 3], [0; 3], KIND_WORLD);
        }
        assert_eq!(hardware_lights(&list, None, 4).count(), 4);
        assert_eq!(hardware_lights(&list, None, 0).count(), 6);
        let light = hardware_lights(&list, None, 0).next().expect("luz");
        assert_eq!((light.radius, light.effect, light.color), (775.0, EFFECT_SPOTLIGHT, [1.0, 1.0, 1.0]));
    }
}
