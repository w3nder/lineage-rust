use glam::Vec3;

use crate::engine_math::{Coords, coords_mul_rotator, transform_vector_by};
use crate::scene::transform::vector_rotation;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Cast,
    PreShot,
    Shot,
    Explosion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Party {
    Null,
    Caster,
    Target,
    Projectile,
    Hit,
    Area,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Bone {
    RightHand,
    LeftHand,
    Named(&'static str),
    Index(i32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Scalar {
    Const(f32),
    CastTime,
    CastOffset(f32),
    OverRadius(f32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Node {
    Float(f32),
    Int(i32),
    Field { target: bool, offset: u16 },
    Op(Op, u16, u16),
    Neg(u16),
    ToF32(u16),
    IntToFloat(u16),
    Ftol(u16),
    Tvb { axis: u8, vector: [u16; 3], rotator: [u16; 3] },
    Rotation { axis: u8, vector: [u16; 3] },
    Slot { party: Party, offset: u16 },
    Normal(u8),
    Random(u8),
    Sqrt(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SiteKind {
    SkillEffect,
    Level,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Site {
    Known { kind: SiteKind, parent: Party, instigator: Party, mode: i32, vector: [u16; 3], rotator: [u16; 3], scale: u16, flags: [i32; 4] },
    PawnLight { pawn: Party, vector: [u16; 3], rotator: [u16; 3], kind: i32 },
    LightInit { kind: i32, vector: [u16; 3], rotator: [u16; 3], byte_a: i32, byte_b: i32, plane: [u16; 4], float_a: u16, float_b: u16, byte_c: i32 },
    ViewShake { kind: i32, values: [u16; 18] },
    Unsupported(&'static str),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Step {
    Spawn { class: &'static str, site: u16 },
    Attach(Bone),
    Trailer { points: i32, time: Scalar, flag: i32 },
    Life(Scalar),
    Delayed { time: Scalar, a: i32, b: i32 },
    SizeScale(Scalar),
    PawnLight { site: u16 },
    LightInit { site: u16 },
    ViewShake { site: u16 },
    DamageEffect { site: u16 },
    DamageAct(Party),
    TrailerPhysics { physics: u8, pre_pivot: bool, same_rotation: bool },
    Sound { mode: i32 },
    Unsupported(&'static str),
}

type Variants = &'static [(u8, &'static [Step])];

include!("skill_effect_data.rs");

pub const SHOT_STAGE_TWO: u8 = 1;
pub const SELF_TARGET: u8 = 2;
pub const NPC: u8 = 4;
pub const FLAG_548: u8 = 8;
pub const TARGET_LIST: u8 = 16;

pub fn steps(phase: Phase, skill_id: i32, env: u8) -> &'static [Step] {
    let Ok(index) = TABLE.binary_search_by(|(p, id, _, _)| (*p as u8, *id).cmp(&(phase as u8, skill_id))) else { return &[] };
    let (_, _, mask, variants) = TABLE[index];
    variants.iter().find(|(bits, _)| *bits == env & mask).map_or(&[], |(_, steps)| *steps)
}

pub fn skills(phase: Phase) -> impl Iterator<Item = i32> {
    TABLE.iter().filter(move |(p, _, _, _)| *p == phase).map(|(_, id, _, _)| *id)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Actor {
    pub position: Vec3,
    pub rotation: [i32; 3],
    pub radius: f32,
    pub height: f32,
    pub aim: [i32; 3],
    pub skill_rate: f32,
    pub draw_scale: f32,
    pub mesh_origin_z: Option<f32>,
    pub mover: Option<Vec3>,
}

impl Actor {
    fn float(&self, offset: u16) -> Option<f32> {
        Some(match offset {
            0x1bc => self.position.x,
            0x1c0 => self.position.y,
            0x1c4 => self.position.z,
            0x2f0 => self.radius,
            0x2f4 => self.height,
            0x600 => self.skill_rate,
            _ => return None,
        })
    }

    fn int(&self, offset: u16) -> Option<i32> {
        Some(match offset {
            0x1c8 => self.rotation[0],
            0x1cc => self.rotation[1],
            0x1d0 => self.rotation[2],
            0x320 => self.aim[0],
            0x324 => self.aim[1],
            0x328 => self.aim[2],
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Context {
    pub caster: Option<Actor>,
    pub target: Option<Actor>,
    pub cast_time: f32,
    pub projectile: Option<Actor>,
    pub hit: Option<Actor>,
    pub area: Option<Actor>,
    pub normal: Vec3,
    pub randoms: [f32; 2],
}

impl Context {
    fn actor(&self, party: Party) -> Option<&Actor> {
        match party {
            Party::Null => None,
            Party::Caster => self.caster.as_ref(),
            Party::Target => self.target.as_ref(),
            Party::Projectile => self.projectile.as_ref(),
            Party::Hit => self.hit.as_ref(),
            Party::Area => self.area.as_ref(),
        }
    }

    fn field(&self, target: bool, offset: u16) -> Option<&Actor> {
        let _ = offset;
        if target { self.target.as_ref() } else { self.caster.as_ref() }
    }

    fn slot(&self, party: Party, offset: u16) -> Option<f32> {
        let actor = self.actor(party)?;
        match (party, offset) {
            (Party::Projectile, 0x4b0 | 0x4b4 | 0x4b8) => actor.float(offset - 0x4b0 + 0x1bc),
            (Party::Hit, 0x410 | 0x414 | 0x418) => actor.mover.map(|vector| vector[usize::from((offset - 0x410) / 4)]),
            _ => actor.float(offset),
        }
    }

    fn eval_f(&self, nodes: &[Node], node: u16) -> Option<f64> {
        Some(match nodes[usize::from(node)] {
            Node::Float(value) => f64::from(value),
            Node::Int(bits) => f64::from(f32::from_bits(bits as u32)),
            Node::Field { target, offset } => f64::from(self.field(target, offset)?.float(offset)?),
            Node::Slot { party, offset } => f64::from(self.slot(party, offset)?),
            Node::Normal(axis) => f64::from(self.normal[usize::from(axis)]),
            Node::Random(index) => f64::from(*self.randoms.get(usize::from(index))?),
            Node::Sqrt(a) => self.eval_f(nodes, a)?.sqrt(),
            Node::Op(op, a, b) => {
                let (a, b) = (self.eval_f(nodes, a)?, self.eval_f(nodes, b)?);
                match op {
                    Op::Add => a + b,
                    Op::Sub => a - b,
                    Op::Mul => a * b,
                    Op::Div => a / b,
                }
            }
            Node::Neg(a) => -self.eval_f(nodes, a)?,
            Node::ToF32(a) => f64::from(self.eval_f(nodes, a)? as f32),
            Node::IntToFloat(a) => f64::from(self.eval_i(nodes, a)?),
            Node::Tvb { axis, vector, rotator } => {
                let vector = [self.eval_f(nodes, vector[0])? as f32, self.eval_f(nodes, vector[1])? as f32, self.eval_f(nodes, vector[2])? as f32];
                let rotator = [self.eval_i(nodes, rotator[0])?, self.eval_i(nodes, rotator[1])?, self.eval_i(nodes, rotator[2])?];
                let coords = coords_mul_rotator(&Coords::UNIT, rotator);
                f64::from(transform_vector_by(vector, &coords)[usize::from(axis)])
            }
            Node::Ftol(_) | Node::Rotation { .. } => return None,
        })
    }

    fn eval_i(&self, nodes: &[Node], node: u16) -> Option<i32> {
        Some(match nodes[usize::from(node)] {
            Node::Int(value) => value,
            Node::Field { target, offset } => self.field(target, offset)?.int(offset)?,
            Node::Slot { party, offset } => self.actor(party)?.int(offset)?,
            Node::Ftol(a) => self.eval_f(nodes, a)? as i32,
            Node::Rotation { axis, vector } => {
                let vector = Vec3::new(self.eval_f(nodes, vector[0])? as f32, self.eval_f(nodes, vector[1])? as f32, self.eval_f(nodes, vector[2])? as f32);
                vector_rotation(vector)[usize::from(axis)]
            }
            _ => return None,
        })
    }

    fn vector(&self, nodes: &[Node], vector: [u16; 3]) -> Option<Vec3> {
        Some(Vec3::new(self.eval_f(nodes, vector[0])? as f32, self.eval_f(nodes, vector[1])? as f32, self.eval_f(nodes, vector[2])? as f32))
    }

    fn rotator(&self, nodes: &[Node], rotator: [u16; 3]) -> Option<[i32; 3]> {
        Some([self.eval_i(nodes, rotator[0])?, self.eval_i(nodes, rotator[1])?, self.eval_i(nodes, rotator[2])?])
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Planned {
    pub class: &'static str,
    pub via_level: bool,
    pub projectile: bool,
    pub mode: i32,
    pub location: Vec3,
    pub rotation: [i32; 3],
    pub owner: Party,
    pub relative: bool,
    pub pre_pivot: Vec3,
    pub speed_rate: Option<f32>,
    pub bone: Option<Bone>,
    pub trailer: Option<(i32, f32, i32)>,
    pub life: Option<f32>,
    pub delayed: Option<(f32, i32, i32)>,
    pub size_scale: Option<f32>,
    pub trailer_physics: Option<(u8, bool, bool)>,
}

fn scalar(value: Scalar, context: &Context) -> f32 {
    match value {
        Scalar::Const(value) => value,
        Scalar::CastTime => context.cast_time,
        Scalar::CastOffset(offset) => context.cast_time + offset,
        Scalar::OverRadius(constant) => (f64::from(constant) / f64::from(context.caster.map_or(0.0, |caster| caster.radius))) as f32,
    }
}

pub struct SiteValues {
    pub vector: Vec3,
    pub rotator: [i32; 3],
    pub scale: f32,
}

pub fn evaluate_site(site: &Site, context: &Context) -> Option<SiteValues> {
    evaluate_site_in(NODES, site, context)
}

fn evaluate_site_in(nodes: &[Node], site: &Site, context: &Context) -> Option<SiteValues> {
    match site {
        Site::Known { vector, rotator, scale, .. } => Some(SiteValues {
            vector: context.vector(nodes, *vector)?,
            rotator: context.rotator(nodes, *rotator)?,
            scale: context.eval_f(nodes, *scale)? as f32,
        }),
        Site::PawnLight { vector, rotator, .. } | Site::LightInit { vector, rotator, .. } => {
            Some(SiteValues { vector: context.vector(nodes, *vector)?, rotator: context.rotator(nodes, *rotator)?, scale: 1.0 })
        }
        Site::ViewShake { .. } | Site::Unsupported(_) => None,
    }
}

/// `AActor::SpawnSkillEffect` (0x104ca880), bloco que só roda no modo relativo (`flags[0] != 0`) com pai não nulo:
/// `TrailerPrePivot.z` vem do modo (1: negativo, 3: positivo) e da escala da malha do pai, ou da altura de colisão
/// quando o pai não tem malha esquelética; `x`/`y` recebem o vetor apenas se `x` e `y` forem não nulos, e `z` do vetor
/// é somado em `y`.
pub fn pre_pivot(mode: i32, vector: Vec3, parent: &Actor) -> Vec3 {
    let mut pivot = Vec3::ZERO;
    let base = match parent.mesh_origin_z {
        Some(origin_z) => (f64::from(parent.draw_scale) * f64::from(origin_z)) as f32,
        None => parent.height,
    };
    match mode {
        1 => pivot.z = (-f64::from(base)) as f32,
        3 => pivot.z = base,
        _ => {}
    }
    if vector.x != 0.0 && vector.y != 0.0 {
        pivot.x += vector.x;
        pivot.y += vector.y;
    }
    if vector.z != 0.0 {
        pivot.y += vector.z;
    }
    pivot
}

fn spawn_planned(nodes: &[Node], phase: Phase, class: &'static str, site: &Site, context: &Context) -> Option<Planned> {
    let Site::Known { kind, parent, instigator, mode, flags, .. } = site else { return None };
    let values = evaluate_site_in(nodes, site, context)?;
    let by_spawn_skill_effect = *kind == SiteKind::SkillEffect;
    let relative = by_spawn_skill_effect && flags[0] != 0;
    let (location, rotation) = if relative {
        (Vec3::ZERO, [0, 0, 0])
    } else {
        match context.actor(*parent) {
            Some(actor) if by_spawn_skill_effect && flags[2] == 0 => (actor.position, actor.rotation),
            _ if by_spawn_skill_effect && *parent != Party::Null && flags[2] == 0 => return None,
            _ => (values.vector, values.rotator),
        }
    };
    let pivot = match (relative, context.actor(*parent)) {
        (true, Some(actor)) => pre_pivot(*mode, values.vector, actor),
        _ => Vec3::ZERO,
    };
    let adjusts = by_spawn_skill_effect && phase == Phase::Cast && *instigator != Party::Null;
    Some(Planned {
        class,
        via_level: !by_spawn_skill_effect,
        projectile: !by_spawn_skill_effect || flags[2] != 0,
        mode: *mode,
        location,
        rotation,
        owner: if by_spawn_skill_effect { *instigator } else { *parent },
        relative,
        pre_pivot: pivot,
        speed_rate: (by_spawn_skill_effect && flags[1] != 0).then_some(values.scale),
        bone: None,
        trailer: None,
        life: adjusts.then_some(context.cast_time + 1.0),
        delayed: None,
        size_scale: None,
        trailer_physics: None,
    })
}

fn modify(last: Option<&mut Planned>, step: &Step, context: &Context) {
    let Some(last) = last else { return };
    match step {
        Step::Attach(bone) => last.bone = Some(*bone),
        Step::Trailer { points, time, flag } => last.trailer = Some((*points, scalar(*time, context), *flag)),
        Step::Life(value) => last.life = Some(scalar(*value, context)),
        Step::Delayed { time, a, b } => last.delayed = Some((last.delayed.map_or(0.0, |(total, _, _)| total) + scalar(*time, context), *a, *b)),
        Step::SizeScale(value) => last.size_scale = Some(last.size_scale.unwrap_or(1.0) * scalar(*value, context)),
        Step::TrailerPhysics { physics, pre_pivot, same_rotation } => last.trailer_physics = Some((*physics, *pre_pivot, *same_rotation)),
        _ => {}
    }
}

pub fn plan(phase: Phase, steps: &[Step], context: &Context) -> Vec<Planned> {
    let mut out: Vec<Planned> = Vec::new();
    for step in steps {
        match step {
            Step::Spawn { class, site } => out.extend(spawn_planned(NODES, phase, class, &SITES[usize::from(*site)], context)),
            _ => modify(out.last_mut(), step, context),
        }
    }
    out
}

pub fn plan_spawn(phase: Phase, steps: &[Step], index: usize, context: &Context) -> Option<Planned> {
    let (class, site) = steps
        .iter()
        .filter_map(|step| match step {
            Step::Spawn { class, site } => Some((*class, *site)),
            _ => None,
        })
        .nth(index)?;
    spawn_planned(NODES, phase, class, &SITES[usize::from(site)], context)
}

pub mod explosion {
    use super::*;

    include!("skill_explosion_data.rs");

    pub fn steps(env: u8, skill_id: i32) -> &'static [Step] {
        let Ok(index) = TABLE.binary_search_by(|(_, id, _, _)| id.cmp(&skill_id)) else { return &[] };
        let (_, _, mask, variants) = TABLE[index];
        variants.iter().find(|(bits, _)| *bits == env & mask).map_or(&[], |(_, steps)| *steps)
    }

    pub fn skills() -> impl Iterator<Item = i32> {
        TABLE.iter().map(|(_, id, _, _)| *id)
    }

    pub fn evaluate_site(site: &Site, context: &Context) -> Option<SiteValues> {
        evaluate_site_in(NODES, site, context)
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct LightInit {
        pub kind: i32,
        pub location: Vec3,
        pub rotation: [i32; 3],
        pub byte_a: i32,
        pub byte_b: i32,
        pub plane: [f32; 4],
        pub float_a: f32,
        pub float_b: f32,
        pub byte_c: i32,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Light {
        pub pawn: Party,
        pub area: Option<usize>,
        pub location: Vec3,
        pub rotation: [i32; 3],
        pub kind: i32,
        pub init: Option<LightInit>,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Shake {
        pub kind: i32,
        pub values: [f32; 18],
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct Spawned {
        pub planned: Planned,
        pub damage_effect: bool,
        pub area: Option<usize>,
    }

    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct Explosion {
        pub spawns: Vec<Spawned>,
        pub lights: Vec<Light>,
        pub shakes: Vec<Shake>,
        pub damage_acts: Vec<(Party, Option<usize>)>,
        pub sounds: Vec<i32>,
    }

    fn node_reads_area(node: u16) -> bool {
        match NODES[usize::from(node)] {
            Node::Slot { party, .. } => party == Party::Area,
            Node::Op(_, a, b) => node_reads_area(a) || node_reads_area(b),
            Node::Neg(a) | Node::ToF32(a) | Node::IntToFloat(a) | Node::Ftol(a) | Node::Sqrt(a) => node_reads_area(a),
            Node::Tvb { vector, rotator, .. } => vector.iter().chain(rotator.iter()).any(|node| node_reads_area(*node)),
            Node::Rotation { vector, .. } => vector.iter().any(|node| node_reads_area(*node)),
            Node::Float(_) | Node::Int(_) | Node::Field { .. } | Node::Normal(_) | Node::Random(_) => false,
        }
    }

    fn site_reads_area(site: u16) -> bool {
        match SITES[usize::from(site)] {
            Site::Known { parent, instigator, vector, rotator, scale, .. } => {
                parent == Party::Area || instigator == Party::Area || vector.iter().chain(rotator.iter()).chain([scale].iter()).any(|node| node_reads_area(*node))
            }
            Site::PawnLight { pawn, vector, rotator, .. } => pawn == Party::Area || vector.iter().chain(rotator.iter()).any(|node| node_reads_area(*node)),
            Site::LightInit { vector, rotator, plane, float_a, float_b, .. } => {
                vector.iter().chain(rotator.iter()).chain(plane.iter()).chain([float_a, float_b].iter()).any(|node| node_reads_area(*node))
            }
            Site::ViewShake { values, .. } => values.iter().any(|node| node_reads_area(*node)),
            Site::Unsupported(_) => false,
        }
    }

    pub fn reads_area(step: &Step) -> bool {
        match step {
            Step::Spawn { site, .. } | Step::DamageEffect { site } | Step::PawnLight { site } | Step::ViewShake { site } => site_reads_area(*site),
            Step::DamageAct(party) => *party == Party::Area,
            _ => false,
        }
    }

    fn modifies_last(step: &Step) -> bool {
        matches!(
            step,
            Step::Attach(_) | Step::Trailer { .. } | Step::Life(_) | Step::Delayed { .. } | Step::SizeScale(_) | Step::TrailerPhysics { .. } | Step::LightInit { .. }
        )
    }

    fn light_init(site: &Site, context: &Context) -> Option<LightInit> {
        let Site::LightInit { kind, vector, rotator, byte_a, byte_b, plane, float_a, float_b, byte_c } = *site else { return None };
        let float = |node: u16| context.eval_f(NODES, node).map(|value| value as f32);
        Some(LightInit {
            kind,
            location: context.vector(NODES, vector)?,
            rotation: context.rotator(NODES, rotator)?,
            byte_a,
            byte_b,
            plane: [float(plane[0])?, float(plane[1])?, float(plane[2])?, float(plane[3])?],
            float_a: float(float_a)?,
            float_b: float(float_b)?,
            byte_c,
        })
    }

    fn run(step: &Step, context: &Context, area: Option<usize>, out: &mut Explosion) {
        let damage_effect = matches!(step, Step::DamageEffect { .. });
        match step {
            Step::Spawn { site, .. } | Step::DamageEffect { site } => {
                let class = if let Step::Spawn { class, .. } = step { *class } else { "" };
                if let Some(planned) = spawn_planned(NODES, Phase::Explosion, class, &SITES[usize::from(*site)], context) {
                    out.spawns.push(Spawned { planned, damage_effect, area });
                }
            }
            Step::PawnLight { site } => {
                let Site::PawnLight { pawn, kind, .. } = SITES[usize::from(*site)] else { return };
                if let Some(values) = evaluate_site(&SITES[usize::from(*site)], context) {
                    out.lights.push(Light { pawn, area, location: values.vector, rotation: values.rotator, kind, init: None });
                }
            }
            Step::LightInit { site } => {
                let init = light_init(&SITES[usize::from(*site)], context);
                if let (Some(light), Some(init)) = (out.lights.last_mut(), init) {
                    light.init = Some(init);
                }
            }
            Step::ViewShake { site } => {
                let Site::ViewShake { kind, values } = SITES[usize::from(*site)] else { return };
                let mut evaluated = [0.0; 18];
                for (slot, node) in evaluated.iter_mut().zip(values) {
                    let Some(value) = context.eval_f(NODES, node) else { return };
                    *slot = value as f32;
                }
                out.shakes.push(Shake { kind, values: evaluated });
            }
            Step::DamageAct(party) => out.damage_acts.push((*party, if *party == Party::Area { area } else { None })),
            Step::Sound { mode } => out.sounds.push(*mode),
            _ => modify(out.spawns.last_mut().map(|spawned| &mut spawned.planned), step, context),
        }
    }

    pub fn plan(steps: &[Step], context: &Context, areas: &[Actor]) -> Explosion {
        let mut out = Explosion::default();
        let mut index = 0;
        while index < steps.len() {
            if !reads_area(&steps[index]) {
                run(&steps[index], context, None, &mut out);
                index += 1;
                continue;
            }
            let mut end = index + 1;
            while end < steps.len() && (reads_area(&steps[end]) || modifies_last(&steps[end])) {
                end += 1;
            }
            for (element, area) in areas.iter().enumerate() {
                let context = Context { area: Some(*area), ..*context };
                for step in &steps[index..end] {
                    run(step, &context, Some(element), &mut out);
                }
            }
            index = end;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn actor(radius: f32) -> Actor {
        Actor { position: Vec3::new(100.0, 200.0, 300.0), rotation: [0, 16384, 0], radius, height: 40.0, aim: [0, 0, 0], skill_rate: 1.3, draw_scale: 1.0, mesh_origin_z: Some(-20.0), mover: None }
    }

    fn context(radius: f32) -> Context {
        let target = Actor { position: Vec3::new(500.0, 700.0, 300.0), radius: 30.0, height: 60.0, ..actor(30.0) };
        Context { caster: Some(actor(radius)), target: Some(target), cast_time: 2.0, ..Context::default() }
    }

    #[test]
    fn spawn_skill_effect_adjusts_life_only_while_casting() {
        let list = steps(Phase::Cast, 2, 0);
        let planned = plan(Phase::Cast, list, &context(20.0));
        assert_eq!(planned[0].life, Some(3.0));
        assert_eq!(planned[0].size_scale, None);
        let shot = plan(Phase::Shot, list, &context(20.0));
        assert_eq!(shot[0].life, None);
    }

    #[test]
    fn phase_light_actors_are_the_spawns_named_by_the_skill_tables() {
        for skill in 0..0x1400 {
            let Some(light) = crate::pawn_light::cast_phase_light(skill) else { continue };
            for env in [0, SELF_TARGET, NPC, SELF_TARGET | NPC] {
                let planned = plan_spawn(Phase::Cast, steps(Phase::Cast, skill, env), light.spawn, &context(20.0)).expect("ator");
                assert_eq!(planned.relative, light.spawn == 0, "{skill:#x}");
                assert_eq!(planned.owner, Party::Caster, "{skill:#x}");
            }
        }
        for skill in [0x1d, 0x5f, 0x117, 0x4f0, 0x4f1] {
            let light = crate::pawn_light::shot_phase_light(skill, true).expect("luz");
            for env in [SHOT_STAGE_TWO, SHOT_STAGE_TWO | SELF_TARGET] {
                let planned = plan_spawn(Phase::Shot, steps(Phase::Shot, skill, env), light.spawn, &context(20.0)).expect("ator");
                assert!(planned.relative && planned.owner == Party::Target, "{skill:#x}");
            }
        }
    }

    fn explosion_context() -> Context {
        let caster = actor(20.0);
        let hit = Actor { position: Vec3::new(500.0, 700.0, 300.0), radius: 30.0, height: 60.0, ..actor(30.0) };
        let impact = Vec3::new(500.0, 700.0, 315.0);
        Context {
            caster: Some(caster),
            target: Some(hit),
            projectile: Some(Actor { position: impact, ..Actor::default() }),
            hit: Some(hit),
            normal: -(impact - caster.position).normalize(),
            randoms: [0.25, 0.75],
            ..Context::default()
        }
    }

    #[test]
    fn explosion_repeats_the_area_block_once_per_target_of_the_list() {
        let context = explosion_context();
        let areas = [Actor { position: Vec3::new(590.0, 660.0, 300.0), ..actor(15.0) }, Actor { position: Vec3::new(-50.0, 20.0, 10.0), ..actor(15.0) }];
        let list = explosion::steps(explosion::EXPLOSION_AREA, 0x18);
        let single = explosion::plan(list, &context, &areas[..1]);
        let double = explosion::plan(list, &context, &areas);
        assert_eq!(double.spawns.len(), single.spawns.len() + 1);
        let tail: Vec<_> = double.spawns.iter().filter(|spawned| spawned.area.is_some()).collect();
        assert_eq!(tail.iter().map(|spawned| spawned.area).collect::<Vec<_>>(), vec![Some(0), Some(1)]);
        assert!(tail.iter().all(|spawned| spawned.planned.bone == Some(Bone::Index(0)) && spawned.planned.owner == Party::Area));
        assert_eq!(double.sounds, vec![3]);
        assert!(explosion::plan(list, &context, &[]).spawns.iter().all(|spawned| spawned.area.is_none()));
    }

    #[test]
    fn explosion_100f_shakes_the_view_with_the_two_frand_values_of_0x104bfe8d() {
        let context = explosion_context();
        let planned = explosion::plan(explosion::steps(0, 0x100f), &context, &[]);
        assert_eq!(planned.spawns.len(), 1);
        let shake = planned.shakes[0];
        let (first, second) = (f64::from(context.randoms[0]), f64::from(context.randoms[1]));
        let k = 1.0 / (first * first + second * second).sqrt();
        let x = (second * k) as f32;
        let y = (first * k) as f32;
        assert_eq!(&shake.values[..4], &[6.0, 1.0, 10000.0, 120.0]);
        assert_eq!(shake.values[4], (f64::from(x) / 100.0) as f32);
        assert_eq!(shake.values[5], (f64::from(y) / 100.0) as f32);
        assert_eq!(shake.values[7], (200.0 * f64::from(x) + 200.0 * f64::from(x)) as f32);
        assert_eq!(&shake.values[10..13], &[120.0, 120.0, 0.0]);
        assert_eq!(&shake.values[13..16], &[500.0, 700.0, 240.0]);
        assert_eq!(&shake.values[16..], &[200.0, 2000.0]);
        assert_eq!(planned.sounds, vec![3]);
    }

    #[test]
    fn explosion_171_trails_the_caster_with_set_physics_10_and_lights_the_hit_pawn() {
        let planned = explosion::plan(explosion::steps(0, 0x171), &explosion_context(), &[]);
        assert_eq!(planned.damage_acts, vec![(Party::Hit, None)]);
        assert_eq!(planned.lights.len(), 1);
        assert_eq!(planned.lights[0].pawn, Party::Hit);
        let spawn = &planned.spawns[0].planned;
        assert_eq!((spawn.class, spawn.via_level, spawn.owner), ("LineageEffect.sp_agility_ta", true, Party::Caster));
        assert_eq!(spawn.location, planned.lights[0].location);
        assert_eq!(spawn.trailer_physics, Some((10, true, false)));
    }

    #[test]
    fn explosion_light_init_completes_the_light_added_just_before() {
        let planned = explosion::plan(explosion::steps(0, 0xfe5), &explosion_context(), &[]);
        let init = planned.lights.last().and_then(|light| light.init).expect("Init");
        assert_eq!((init.kind, init.byte_a, init.byte_b, init.byte_c), (0, 1, 12, 0));
        assert_eq!(init.plane, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!((init.float_a, init.float_b), (f32::from_bits(1050253722), 30.0));
    }

    #[test]
    fn explosion_without_owner_takes_the_caster_branch_and_resolves_no_caster() {
        let mut context = explosion_context();
        context.caster = None;
        let planned = explosion::plan(explosion::steps(explosion::EXPLOSION_OWNER_NULL, 0x7), &context, &[]);
        assert!(!planned.spawns.is_empty());
        assert!(planned.sounds.is_empty());
    }

    #[test]
    fn pre_pivot_equals_the_value_spawn_skill_effect_writes_in_the_engine() {
        let cases: serde_json::Value = serde_json::from_str(include_str!("../tests/data/spawn_pivot_oracle.json")).expect("fixture");
        for (index, case) in cases.as_array().expect("casos").iter().enumerate() {
            if !case["relative"].as_bool().expect("relative") {
                continue;
            }
            let number = |value: &serde_json::Value| value.as_f64().expect("número") as f32;
            let vector = case["vector"].as_array().expect("vetor");
            let vector = Vec3::new(number(&vector[0]), number(&vector[1]), number(&vector[2]));
            let parent = Actor {
                height: number(&case["height"]),
                draw_scale: number(&case["draw_scale"]),
                mesh_origin_z: case["has_mesh"].as_bool().expect("malha").then(|| number(&case["origin_z"])),
                ..actor(20.0)
            };
            let pivot = pre_pivot(case["mode"].as_i64().expect("modo") as i32, vector, &parent);
            let want = case["pivot"].as_array().expect("pivô");
            for axis in 0..3 {
                assert_eq!(pivot[axis].to_bits(), number(&want[axis]).to_bits(), "caso {index}: eixo {axis}: {pivot:?} != {want:?}");
            }
        }
    }
}
