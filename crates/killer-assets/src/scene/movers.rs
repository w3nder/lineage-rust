use super::{MeshDraw, props};
use crate::{client::Client, options::ClippingRange};
use ue2_level::{
    Level, Placement,
    movers::{L2Movement, Mover},
};

#[derive(Debug, Clone, PartialEq)]
pub struct MovingAsset {
    pub name: String,
    pub mesh: Option<String>,
    pub origin: Placement,
    pub movement: Option<L2Movement>,
    pub mover: Option<Mover>,
    pub draws: Vec<MeshDraw>,
    pub grades: [Vec<MeshDraw>; 7],
    pub attached: Vec<AttachedActor>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttachedActor {
    pub name: String,
    pub origin: Placement,
    pub emitter: Option<usize>,
}

pub fn attached_actors(level: &Level, actor: &ue2_level::Actor) -> Vec<AttachedActor> {
    level
        .actors
        .iter()
        .filter(|child| {
            child.movement_event.as_ref().is_some_and(|event| {
                actor
                    .movement_tags
                    .iter()
                    .take(12)
                    .take_while(|t| !t.eq_ignore_ascii_case("None"))
                    .any(|t| t.eq_ignore_ascii_case(event))
            })
        })
        .map(|child| AttachedActor {
            name: child.name.clone(),
            origin: child.placement,
            emitter: level
                .actors
                .iter()
                .filter(|a| a.class.eq_ignore_ascii_case("Emitter"))
                .position(|a| a.name == child.name),
        })
        .collect()
}

pub fn build(client: &mut Client, level: &Level, clipping: &ClippingRange) -> Vec<MovingAsset> {
    let mut builder = props::Builder::new(level);
    level
        .actors
        .iter()
        .filter(|a| a.l2_movement.is_some() || a.mover.is_some())
        .map(|actor| {
            let draws = builder.build(client, level, clipping, std::iter::once(actor));
            let grades = std::array::from_fn(|i| {
                let Some(path) = actor.mover.as_ref().and_then(|m| m.meshes[i].as_ref()) else {
                    return Vec::new();
                };
                let mut grade = actor.clone();
                grade.static_mesh = Some(path.clone());
                grade.lighting.clear();
                builder.build(client, level, clipping, std::iter::once(&grade))
            });
            MovingAsset {
                name: actor.name.clone(),
                mesh: actor.static_mesh.clone(),
                origin: actor.placement,
                movement: actor.l2_movement.clone(),
                mover: actor.mover.clone(),
                draws,
                grades,
                attached: attached_actors(level, actor),
            }
        })
        .collect()
}
