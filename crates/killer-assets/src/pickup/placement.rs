use super::{Placement, Shape, Spin, TraceQuery, drop::{self, Plan}};
use crate::{client::Client, collision::{resources::Resources, world::{View, trace}}};
use glam::Vec3;

pub struct Request {
    pub position: Vec3,
    pub rotation: [i32;3],
    pub style: u32,
    pub kind: Option<u32>,
    pub source: Option<Vec3>,
    pub pieces: Vec<(usize,Shape)>,
    pub views: Vec<View>,
}

pub fn place(client: &mut Client, resources: &mut Resources, request: &Request) -> Result<Vec<(usize,Plan)>,String> {
    let mut pieces = Vec::new();
    for &(index,shape) in &request.pieces {
        let mut error = None;
        let mut query = |query| {
            if error.is_some() { return None; }
            match trace(client,resources,&request.views,query) {
                Ok(hit) => hit,
                Err(reason) => { error = Some(reason); None }
            }
        };
        let plan = if let Some(kind) = request.kind {
            let mut plan = None;
            'attempts: for _ in 0..100 {
                let mut start = request.position + Vec3::Z*(shape.height+20.0);
                let mut end = request.position;
                for _ in 0..5 {
                    start.z += 20.0;
                    end.z -= 50.0;
                    if let Some(hit) = query(TraceQuery { start,end,extent:Vec3::new(shape.radius,shape.radius,shape.height),flags:0x86 }) {
                        plan = drop::plan(drop::Request { packet_rotation:request.rotation,source:request.source,shape,style:request.style,kind,index },hit);
                        if plan.is_some() { break 'attempts; }
                    }
                }
            }
            plan
        } else {
            super::spawn_placement(request.position,request.rotation,shape,request.style,index,&mut query,|_| true)
                .map(|placement: Placement| Plan { spin:Spin { rotation:placement.rotation,target:placement.rotation,rate:[0;3] },
                    effect_position:placement.position,placement,trajectory:None,kind:0 })
        };
        if let Some(error) = error { return Err(error); }
        let Some(plan) = plan else { return Err(format!("nenhuma colocação original válida após 500 tentativas, peça {index}, posição {:?}",request.position)); };
        pieces.push((index,plan));
    }
    Ok(pieces)
}
