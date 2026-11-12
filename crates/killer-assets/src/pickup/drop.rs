use super::{Placement, Shape, Spin, TraceHit, rotation, settle};
use crate::scene::transform::vector_rotation;
use glam::Vec3;
use super::mover::{Host, Mover, Target};

pub const THROW_SOUND: &str = "ItemSound.Item_throw";
pub const GROUND_EFFECT: &str = "LineageEffect.e_u056_b";
pub const APPEAR_EFFECT: &str = "LineageEffect.e_u056_a";

#[derive(Clone, Copy, Debug)]
pub struct Request {
    pub packet_rotation: [i32;3],
    pub source: Option<Vec3>,
    pub shape: Shape,
    pub style: u32,
    pub kind: u32,
    pub index: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Trajectory {
    pub initial: Vec3,
    pub destination: Vec3,
    pub intermediate: Option<Vec3>,
    pub speed: f32,
    pub acceleration: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Plan {
    pub placement: Placement,
    pub spin: Spin,
    pub effect_position: Vec3,
    pub trajectory: Option<Trajectory>,
    pub kind: u32,
}

pub struct Motion {
    root: Mover,
    child: Option<Mover>,
}

impl Motion {
    pub fn new(trajectory: Trajectory) -> Self {
        let mut root = Mover::new(trajectory.initial);
        root.speed = trajectory.speed;
        root.acceleration = trajectory.acceleration;
        let child = trajectory.intermediate.map(|initial| {
            let mut child = Mover::new(initial);
            child.speed = trajectory.speed;
            child.acceleration = trajectory.acceleration;
            child.targets.push(Target { position:trajectory.destination,actor:None,tick:false });
            child.start(0.0,false);
            child
        });
        root.targets.push(Target { position:trajectory.destination,actor:child.as_ref().map(|_| 1),tick:child.is_some() });
        root.start(0.0,false);
        Self { root,child }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        self.root.tick(delta,&mut MotionHost { child:&mut self.child })
    }

    pub fn position(&self) -> Vec3 { self.root.position }
    pub fn ended(&self) -> bool { self.root.ended() }
}

struct MotionHost<'a> { child: &'a mut Option<Mover> }

impl Host for MotionHost<'_> {
    fn tick(&mut self, _: u64, delta: f32) {
        if let Some(mut child) = self.child.take() {
            child.tick(delta,self);
            *self.child = Some(child);
        }
    }
    fn position(&self, _: u64) -> Option<Vec3> { self.child.as_ref().map(|child| child.position) }
    fn ended(&self, _: u64) -> Option<bool> { self.child.as_ref().map(Mover::ended) }
    fn destroy(&mut self, _: u64) { *self.child = None; }
    fn move_actor(&mut self, position: Vec3, delta: Vec3) -> Vec3 { position + delta }
}

pub fn plan(request: Request, hit: TraceHit) -> Option<Plan> {
    if hit.position.z == 0.0 { return None; }
    let mut position = hit.position;
    if hit.non_level_actor { position.z += 2.0; }
    let mut effect_position = position;
    effect_position.z = if matches!(request.style,1|2) { (f64::from(position.z) - f64::from(request.shape.height) * 1.5) as f32 } else { position.z - request.shape.height };
    if matches!(request.style,3|4) { position.z -= request.shape.height; }
    position.z = settle(position.z,hit.time);
    let mut style = rotation(request.style,request.index);
    let trajectory = if matches!(request.kind,1..=3) {
        if let Some(source) = request.source {
            let delta = position - source;
            style[1] = vector_rotation(delta)[1].wrapping_add(if request.style == 2 { 0x4000 } else { 0x8000 });
            let [x,y,z] = delta.to_array().map(f64::from);
            let intermediate = Vec3::new(position.x,position.y,(position.z as f64 + (x*x+y*y+z*z).sqrt()) as f32);
            Some(Trajectory { initial:source,destination:position,intermediate:Some(intermediate),speed:300.0,acceleration:100.0 })
        } else {
            Some(Trajectory { initial:position+Vec3::Z*30.0,destination:position,intermediate:None,speed:100.0,acceleration:0.0 })
        }
    } else { None };
    let floor = vector_rotation(Vec3::new(hit.normal.z,hit.normal.y,-hit.normal.x));
    let rotation = std::array::from_fn(|axis| floor[axis].wrapping_add(style[axis])
        .wrapping_add(if request.kind == 5 { 0 } else { request.packet_rotation[axis] }));
    let mut rate = [0;3];
    if matches!(request.kind,1|2) {
        rate[if request.style == 2 { 2 } else { 0 }] = if request.index.is_multiple_of(2) { -540016 } else { 540016 };
    }
    let spin = Spin { rotation,target:if matches!(request.kind,1|2) { rotation } else { [0;3] },rate };
    Some(Plan { placement:Placement { position:trajectory.map_or(position,|t| t.initial),rotation,shape:request.shape },
        spin,effect_position,trajectory,kind:request.kind })
}
