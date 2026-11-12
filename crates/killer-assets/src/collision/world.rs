use super::{point_check::PointHit,model::{Hit,Model},resources::{ClassCollision,PhysicsProps,Placed,Primitive,Resources},static_mesh::Mesh,terrain::{Quad,QuadTrace}};
use crate::{client::Client,pickup::{TraceHit,TraceQuery},scene::terrain::{edge_turned,quad_visible}};
use glam::{Mat4,Vec3};
use rustc_hash::FxHashMap;
use std::sync::Arc;
use ue2_level::{Level,Placement};

pub struct Pose {
    pub placement: Placement,
    pub mesh: Option<String>,
    pub disabled: bool,
}

pub struct View {
    pub name: String,
    pub level: Arc<Level>,
    pub poses: FxHashMap<String,Pose>,
}

#[derive(Clone, Copy, Debug)]
pub struct Target {
    pub object_id: i32,
    pub spawned: u64,
    pub position: Vec3,
    pub radius: f32,
    pub height: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Spawned {
    pub class: String,
    pub mesh: String,
    pub placement: Placement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    Level,
    Terrain,
    StaticMeshActor,
}

#[derive(Clone, Copy, Debug)]
pub struct Selection {
    pub hit: TraceHit,
    pub target: Option<Target>,
    pub surface: Option<Surface>,
    pub immediately_stop: bool,
    pub pushable_mass: Option<f32>,
    pub server_id: Option<i32>,
}

pub fn trace(client: &mut Client, resources: &mut Resources, views: &[View], query: TraceQuery) -> Result<Option<TraceHit>,String> {
    if query.flags != 0x86 { return Err(format!("trace com flags não suportadas: {:x}",query.flags)); }
    line_check(client,resources,views,query,&[],&[],query.extent).map(|hit| hit.map(|hit| hit.hit))
}

pub fn trace_selection(client: &mut Client, resources: &mut Resources, views: &[View], query: TraceQuery) -> Result<Option<Selection>,String> {
    if query.flags != 0x86 { return Err(format!("trace com flags não suportadas: {:x}",query.flags)); }
    line_check(client,resources,views,query,&[],&[],query.extent)
}

pub fn trace_blocking(client: &mut Client, resources: &mut Resources, views: &[View], query: TraceQuery, targets: &[Target]) -> Result<Option<Selection>,String> {
    if query.flags != 0x86 { return Err(format!("trace com flags não suportadas: {:x}",query.flags)); }
    line_check(client,resources,views,query,targets,&[],query.extent)
}

pub fn trace_spawned(client: &mut Client, resources: &mut Resources, views: &[View], query: TraceQuery, targets: &[Target], spawned: &[Spawned]) -> Result<Option<Selection>,String> {
    if query.flags != 0x86 { return Err(format!("trace com flags não suportadas: {:x}",query.flags)); }
    line_check(client,resources,views,query,targets,spawned,query.extent)
}

pub fn trace_targets(client: &mut Client, resources: &mut Resources, views: &[View], query: TraceQuery, targets: &[Target], spawned: &[Spawned]) -> Result<Option<Selection>,String> {
    if !matches!(query.flags,0x200bf | 0x300bf) { return Err(format!("trace com flags não suportadas: {:x}",query.flags)); }
    line_check(client,resources,views,query,targets,spawned,Vec3::ZERO)
}

fn line_check(client: &mut Client, resources: &mut Resources, views: &[View], query: TraceQuery, targets: &[Target], spawned: &[Spawned], level_extent: Vec3) -> Result<Option<Selection>,String> {
    let Some(main) = views.first() else { return Ok(None); };
    let mut hits = Vec::new();
    let mut fraction = 1.0;
    let mut end = query.end;
    let flags = query.flags | 0x400;
    for view in views {
        let model = Model { tree:&view.level.bsp_tree,data:&view.level.bsp_render,transform:None,unscaled:true };
        let hit = if level_extent == Vec3::ZERO { model.trace_line(query.start.to_array(),end.to_array(),flags) }
            else { model.trace(query.start.to_array(),end.to_array(),level_extent.to_array(),flags) };
        if let Some(hit) = hit {
            let hit = TraceHit { position:Vec3::from(hit.position),normal:Vec3::from(hit.normal),time:hit.time*fraction,non_level_actor:false };
            fraction = shorten(query.start,hit,5.0);
            end = query.start+(query.end-query.start)*fraction;
            hits.push(Selection { hit,target:None,surface:Some(Surface::Level),immediately_stop:false,pushable_mass:None,server_id:None });
        }
    }
    let level = &main.level;
    let tile = resources.tile(client,&main.name,level)?;
    let level_info = level.actors.iter().find(|actor| actor.class.eq_ignore_ascii_case("LevelInfo")).map(|actor| actor.name.as_str());
    let zone_name = |zone: usize| level.zones.get(zone).and_then(|zone| zone.actor.as_deref()).or(level_info);
    for zone in 0..64 {
        let Some(name) = zone_name(zone) else { continue; };
        let Some(info) = level.zone_infos.get(name).filter(|info| info.terrain_zone) else { continue; };
        for name in &info.terrains {
            let index = level.terrains.iter().position(|terrain| terrain.name.eq_ignore_ascii_case(name))
                .ok_or_else(|| format!("terrain de zona ausente: {}.{name}",main.name))?;
            let terrain = &level.terrains[index];
            let grid = &tile.terrains[index];
            let coords = terrain.native.to_heightmap.ok_or_else(|| format!("coordenadas de colisão ausentes: {}.{name}",main.name))?;
            let mut hit = None;
            QuadTrace { start:query.start.to_array(),end:end.to_array(),extent:query.extent.to_array(),to_heightmap:coords,alternate:None,size:grid.size }
                .visit(|x,y| Quad { vertices:[grid.position(x,y),grid.position(x+1,y),grid.position(x+1,y+1),grid.position(x,y+1)],
                    turned:edge_turned(&terrain.edge_turn,grid.size[0],x,y),inverted:terrain.inverted,
                    visible:quad_visible(&terrain.visible_quads,grid.size[0],x,y),deleted:false }
                    .trace(query.start.to_array(),end.to_array(),query.extent.to_array(),false,&mut hit));
            if let Some(hit) = hit {
                let region = zone_name(level.bsp_tree.zone_at(hit.position) as usize);
                if region != zone_name(zone) && region != level_info { continue; }
                let hit = TraceHit { position:Vec3::from(hit.position),normal:Vec3::from(hit.normal),time:hit.time*fraction,non_level_actor:true };
                fraction = shorten(query.start,hit,20.0);
                end = query.start+(query.end-query.start)*fraction;
                hits.push(Selection { hit,target:None,surface:Some(Surface::Terrain),immediately_stop:false,pushable_mass:None,server_id:None });
            }
        }
    }
    for view in views {
        let tile = resources.tile(client,&view.name,&view.level)?;
        let mut nearest = None;
        for shape in &tile.actors {
            if shape.trace_flags & query.flags == 0 { continue; }
            let actor = &view.level.actors[shape.index];
            let pose = view.poses.get(&actor.name);
            if pose.is_some_and(|pose| pose.disabled) { continue; }
            let placement = pose.map_or(actor.placement,|pose| pose.placement);
            let path = match &shape.primitive {
                Primitive::Mesh(path) => Some(pose.and_then(|pose| pose.mesh.as_deref()).unwrap_or(path)),
                _ => None,
            };
            let cached = if pose.is_none() { shape.placed.get().copied() } else { None };
            let placed = match cached {
                Some(placed) => placed,
                None => {
                    let transform = super::transform::actor(placement,actor.scale,shape.pre_pivot);
                    let corners = match (&shape.primitive,path) {
                        (Primitive::Mesh(_),Some(path)) => Some(world_box(resources.bounds(client,path)?.bounds,transform)),
                        (Primitive::Model(path),_) => Some(world_box(resources.model(client,path)?.bounding_box,transform)),
                        _ => None,
                    };
                    let placed = Placed { transform,corners };
                    if pose.is_none() { let _ = shape.placed.set(placed); }
                    placed
                }
            };
            let transform = placed.transform;
            if let Some((lo,hi)) = placed.corners && !touches(lo,hi,query.start,end,query.extent) { continue; }
            let unscaled = actor.draw_scale == 1.0 && actor.draw_scale_3d == [1.0;3];
            let hit = match &shape.primitive {
                Primitive::Mesh(_) => {
                    let Some(path) = path else { continue };
                    let mesh = resources.mesh(client,path)?;
                    let simple = if query.extent == Vec3::ZERO { mesh.simple_line } else { mesh.simple_box };
                    if simple && flags & 0x100 == 0 && let Some(path) = &mesh.model {
                        let data = resources.model(client,path)?;
                        Model { tree:&data.tree,data:&data.render,transform:Some(transform),unscaled }.trace(query.start.to_array(),end.to_array(),query.extent.to_array(),flags)
                    } else {
                        let world_to_local = super::transform::world_to_local(placement,actor.draw_scale,actor.draw_scale_3d,shape.pre_pivot);
                        Mesh { vertices:&mesh.positions,triangles:&mesh.triangles,nodes:&mesh.nodes,transform,world_to_local }
                            .trace(query.start.to_array(),end.to_array(),query.extent.to_array())
                    }
                }
                Primitive::Model(path) => {
                    let data = resources.model(client,path)?;
                    Model { tree:&data.tree,data:&data.render,transform:Some(transform),unscaled }.trace(query.start.to_array(),end.to_array(),query.extent.to_array(),flags)
                }
                Primitive::Cylinder { radius,height } => super::cylinder::trace(placement.location,*radius,*height,query.start.to_array(),end.to_array(),query.extent.to_array()),
            };
            if let Some(hit) = hit {
                let hit = TraceHit { position:Vec3::from(hit.position),normal:Vec3::from(hit.normal),time:hit.time*fraction,non_level_actor:true };
                if nearest.as_ref().is_none_or(|old: &Selection| hit.time < old.hit.time) { nearest = Some(Selection { hit,target:None,surface:shape.static_mesh_actor.then_some(Surface::StaticMeshActor),immediately_stop:shape.immediately_stop,pushable_mass:shape.pushable_mass,server_id:actor.server_id }); }
            }
        }
        for piece in spawned {
            let [x,y,_] = piece.placement.location;
            if crate::tiles::TileId::of(x,y).name() != view.name { continue; }
            let Some(shape) = resources.class_collision(client,&piece.class)? else { continue; };
            if shape.trace_flags & query.flags == 0 { continue; }
            if let Some(hit) = spawned_hit(client,resources,piece,&shape,query,end,flags)? {
                let hit = TraceHit { position:Vec3::from(hit.position),normal:Vec3::from(hit.normal),time:hit.time*fraction,non_level_actor:true };
                if nearest.as_ref().is_none_or(|old: &Selection| hit.time < old.hit.time) { nearest = Some(Selection { hit,target:None,surface:shape.static_mesh_actor.then_some(Surface::StaticMeshActor),immediately_stop:shape.immediately_stop,pushable_mass:shape.pushable_mass,server_id:None }); }
            }
        }
        for target in targets {
            let tile = crate::tiles::TileId::of(target.position.x,target.position.y).name();
            if tile != view.name { continue; }
            if let Some(hit) = super::cylinder::trace(target.position.to_array(),target.radius,target.height,query.start.to_array(),end.to_array(),query.extent.to_array()) {
                let hit = TraceHit { position:Vec3::from(hit.position),normal:Vec3::from(hit.normal),time:hit.time*fraction,non_level_actor:true };
                if nearest.as_ref().is_none_or(|old: &Selection| hit.time < old.hit.time) { nearest = Some(Selection { hit,target:Some(*target),surface:None,immediately_stop:false,pushable_mass:None,server_id:None }); }
            }
        }
        if hits.len() < 64 && let Some(hit) = nearest { hits.push(hit); }
    }
    Ok(hits.into_iter().min_by(|a,b| a.hit.time.total_cmp(&b.hit.time)))
}

pub fn point_check(client: &mut Client, resources: &mut Resources, views: &[View], spawned: &[Spawned], location: Vec3, extent: Vec3) -> Result<bool,String> {
    let Some(main) = views.first() else { return Ok(false); };
    let (at,size) = (location.to_array(),extent.to_array());
    let mut hit = PointHit { location:[0.0;3],normal:[0.0;3],item:0 };
    for view in views {
        let model = Model { tree:&view.level.bsp_tree,data:&view.level.bsp_render,transform:None,unscaled:true };
        if !model.point_check(at,size,0).0 { return Ok(true); }
    }
    let level = &main.level;
    let tile = resources.tile(client,&main.name,level)?;
    let level_info = level.actors.iter().find(|actor| actor.class.eq_ignore_ascii_case("LevelInfo")).map(|actor| actor.name.as_str());
    let zone_name = |zone: usize| level.zones.get(zone).and_then(|zone| zone.actor.as_deref()).or(level_info);
    let (low,high,flat) = ([at[0],at[1],at[2]-size[2]],[at[0],at[1],at[2]+size[2]],[size[0],size[1],0.0]);
    for zone in 0..64 {
        let Some(name) = zone_name(zone) else { continue; };
        let Some(info) = level.zone_infos.get(name).filter(|info| info.terrain_zone) else { continue; };
        for name in &info.terrains {
            let index = level.terrains.iter().position(|terrain| terrain.name.eq_ignore_ascii_case(name))
                .ok_or_else(|| format!("terrain de zona ausente: {}.{name}",main.name))?;
            let terrain = &level.terrains[index];
            let grid = &tile.terrains[index];
            let coords = terrain.native.to_heightmap.ok_or_else(|| format!("coordenadas de colisão ausentes: {}.{name}",main.name))?;
            let mut found = None;
            QuadTrace { start:low,end:high,extent:flat,to_heightmap:coords,alternate:None,size:grid.size }
                .visit(|x,y| Quad { vertices:[grid.position(x,y),grid.position(x+1,y),grid.position(x+1,y+1),grid.position(x,y+1)],
                    turned:edge_turned(&terrain.edge_turn,grid.size[0],x,y),inverted:terrain.inverted,
                    visible:quad_visible(&terrain.visible_quads,grid.size[0],x,y),deleted:false }
                    .trace(low,high,flat,false,&mut found));
            if let Some(found) = found {
                let lifted = [found.position[0],found.position[1],found.position[2]+size[2]];
                if zone_name(level.bsp_tree.zone_at(lifted) as usize) == zone_name(zone) { return Ok(true); }
            }
        }
    }
    for view in views {
        let tile = resources.tile(client,&view.name,&view.level)?;
        for shape in &tile.actors {
            if shape.trace_flags & 0x86 == 0 { continue; }
            let actor = &view.level.actors[shape.index];
            let pose = view.poses.get(&actor.name);
            if pose.is_some_and(|pose| pose.disabled) { continue; }
            let placement = pose.map_or(actor.placement,|pose| pose.placement);
            let path = match &shape.primitive {
                Primitive::Mesh(path) => Some(pose.and_then(|pose| pose.mesh.as_deref()).unwrap_or(path)),
                _ => None,
            };
            let transform = super::transform::actor(placement,actor.scale,shape.pre_pivot);
            let corners = match (&shape.primitive,path) {
                (Primitive::Mesh(_),Some(path)) => Some(world_box(resources.bounds(client,path)?.bounds,transform)),
                (Primitive::Model(path),_) => Some(world_box(resources.model(client,path)?.bounding_box,transform)),
                _ => None,
            };
            if let Some((lo,hi)) = corners && !touches(lo,hi,location,location,extent) { continue; }
            let unscaled = actor.draw_scale == 1.0 && actor.draw_scale_3d == [1.0;3];
            let blocked = match &shape.primitive {
                Primitive::Mesh(_) => {
                    let Some(path) = path else { continue };
                    let mesh = resources.mesh(client,path)?;
                    if mesh.simple_box && let Some(model) = &mesh.model {
                        let data = resources.model(client,model)?;
                        !Model { tree:&data.tree,data:&data.render,transform:Some(transform),unscaled }.point_check(at,size,0).0
                    } else {
                        let world_to_local = super::transform::world_to_local(placement,actor.draw_scale,actor.draw_scale_3d,shape.pre_pivot);
                        Mesh { vertices:&mesh.positions,triangles:&mesh.triangles,nodes:&mesh.nodes,transform,world_to_local }.point_check(at,size,&mut hit)
                    }
                }
                Primitive::Model(path) => {
                    let data = resources.model(client,path)?;
                    !Model { tree:&data.tree,data:&data.render,transform:Some(transform),unscaled }.point_check(at,size,0).0
                }
                Primitive::Cylinder { radius,height } => super::cylinder::point_check(placement.location,*radius,*height,at,size,&mut hit),
            };
            if blocked { return Ok(true); }
        }
        for piece in spawned {
            let [x,y,_] = piece.placement.location;
            if crate::tiles::TileId::of(x,y).name() != view.name { continue; }
            let Some(shape) = resources.class_collision(client,&piece.class)? else { continue; };
            if shape.trace_flags & 0x86 == 0 { continue; }
            if let Some((radius,height)) = shape.cylinder {
                if super::cylinder::point_check(piece.placement.location,radius,height,at,size,&mut hit) { return Ok(true); }
                continue;
            }
            let transform = super::transform::actor(piece.placement,shape.scale(),shape.pre_pivot);
            let (lo,hi) = world_box(resources.bounds(client,&piece.mesh)?.bounds,transform);
            if !touches(lo,hi,location,location,extent) { continue; }
            let mesh = resources.mesh(client,&piece.mesh)?;
            let blocked = if mesh.simple_box && let Some(model) = &mesh.model {
                let data = resources.model(client,model)?;
                !Model { tree:&data.tree,data:&data.render,transform:Some(transform),unscaled:shape.unscaled() }.point_check(at,size,0).0
            } else {
                let world_to_local = super::transform::world_to_local(piece.placement,shape.draw_scale,shape.draw_scale_3d,shape.pre_pivot);
                Mesh { vertices:&mesh.positions,triangles:&mesh.triangles,nodes:&mesh.nodes,transform,world_to_local }.point_check(at,size,&mut hit)
            };
            if blocked { return Ok(true); }
        }
    }
    Ok(false)
}

fn spawned_hit(client: &mut Client, resources: &mut Resources, piece: &Spawned, shape: &ClassCollision, query: TraceQuery, end: Vec3, flags: u32) -> Result<Option<Hit>,String> {
    if let Some((radius,height)) = shape.cylinder {
        return Ok(super::cylinder::trace(piece.placement.location,radius,height,query.start.to_array(),end.to_array(),query.extent.to_array()));
    }
    let transform = super::transform::actor(piece.placement,shape.scale(),shape.pre_pivot);
    let (lo,hi) = world_box(resources.bounds(client,&piece.mesh)?.bounds,transform);
    if !touches(lo,hi,query.start,end,query.extent) { return Ok(None); }
    let mesh = resources.mesh(client,&piece.mesh)?;
    let simple = if query.extent == Vec3::ZERO { mesh.simple_line } else { mesh.simple_box };
    if simple && flags & 0x100 == 0 && let Some(path) = &mesh.model {
        let data = resources.model(client,path)?;
        return Ok(Model { tree:&data.tree,data:&data.render,transform:Some(transform),unscaled:shape.unscaled() }.trace(query.start.to_array(),end.to_array(),query.extent.to_array(),flags));
    }
    let world_to_local = super::transform::world_to_local(piece.placement,shape.draw_scale,shape.draw_scale_3d,shape.pre_pivot);
    Ok(Mesh { vertices:&mesh.positions,triangles:&mesh.triangles,nodes:&mesh.nodes,transform,world_to_local }
        .trace(query.start.to_array(),end.to_array(),query.extent.to_array()))
}

fn shorten(start: Vec3,hit: TraceHit,margin: f64) -> f32 {
    let [x,y,z] = (hit.position-start).to_array().map(f64::from);
    let distance = (x*x+y*y+z*z).sqrt();
    (((distance+margin)*hit.time as f64)/(distance+0.0001_f32 as f64)).min(1.0) as f32
}

fn world_box(bounds: [[f32;3];2],transform: Mat4) -> (Vec3,Vec3) {
    let mut lo = Vec3::splat(f32::INFINITY);
    let mut hi = Vec3::splat(f32::NEG_INFINITY);
    for corner in 0..8 {
        let point = transform.transform_point3(Vec3::from(std::array::from_fn::<_,3,_>(|axis| bounds[usize::from(corner & (1<<axis) != 0)][axis])));
        lo = lo.min(point);
        hi = hi.max(point);
    }
    (lo,hi)
}

fn touches(lo: Vec3,hi: Vec3,start: Vec3,end: Vec3,extent: Vec3) -> bool {
    let min = start.min(end)-extent;
    let max = start.max(end)+extent;
    (0..3).all(|axis| min[axis] <= hi[axis] && max[axis] >= lo[axis])
}

pub const DEFAULT_VOLUME_PRIORITY: i32 = -1_000_000;

pub fn default_physics_props() -> PhysicsProps {
    PhysicsProps {
        gravity: [0.0,0.0,-1500.0],zone_velocity: [0.0;3],ground_friction: 8.0,terminal_velocity: 2500.0,fluid_friction: 0.3,
        priority: DEFAULT_VOLUME_PRIORITY,water: false,l2_water: false,l2_step: false,step_sound_id: -1,
    }
}

pub fn physics_volume_at(client: &mut Client, resources: &mut Resources, views: &[View], point: Vec3) -> Result<PhysicsProps,String> {
    let mut best = None::<PhysicsProps>;
    for view in views {
        let tile = resources.tile(client,&view.name,&view.level)?;
        if let Some(default) = tile.default_volume && best.is_none() {
            best = Some(PhysicsProps { priority: DEFAULT_VOLUME_PRIORITY,..default });
        }
        for volume in &tile.volumes {
            if best.is_some_and(|best| best.priority >= volume.props.priority) { continue; }
            let actor = &view.level.actors[volume.index];
            let pose = view.poses.get(&actor.name);
            if pose.is_some_and(|pose| pose.disabled) { continue; }
            let placement = pose.map_or(actor.placement,|pose| pose.placement);
            let data = resources.model(client,&volume.brush)?;
            let transform = super::transform::actor(placement,actor.scale,[0.0;3]);
            let unscaled = actor.draw_scale == 1.0 && actor.draw_scale_3d == [1.0;3];
            let inside = Model { tree:&data.tree,data:&data.render,transform:Some(transform),unscaled }
                .trace_line(point.to_array(),(point+Vec3::Z*0.01).to_array(),0).is_some();
            if inside { best = Some(volume.props); }
        }
    }
    Ok(best.unwrap_or_else(default_physics_props))
}

pub fn music_volume_at(client: &mut Client, resources: &mut Resources, views: &[View], point: Vec3) -> Result<Option<MusicZone>,String> {
    let mut found = None;
    for view in views {
        let tile = resources.tile(client,&view.name,&view.level)?;
        for volume in &tile.music_volumes {
            let actor = &view.level.actors[volume.index];
            let pose = view.poses.get(&actor.name);
            if pose.is_some_and(|pose| pose.disabled) { continue; }
            let placement = pose.map_or(actor.placement,|pose| pose.placement);
            let data = resources.model(client,&volume.brush)?;
            let transform = super::transform::actor(placement,actor.scale,[0.0;3]);
            let unscaled = actor.draw_scale == 1.0 && actor.draw_scale_3d == [1.0;3];
            let inside = Model { tree:&data.tree,data:&data.render,transform:Some(transform),unscaled }
                .trace_line(point.to_array(),(point+Vec3::Z*0.01).to_array(),0).is_some();
            if inside { found = Some(MusicZone { tile: view.name.clone(),index: volume.index,music_id: volume.music_id,force_play: volume.force_play,looped: volume.looped }); }
        }
    }
    Ok(found)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MusicZone {
    pub tile: String,
    pub index: usize,
    pub music_id: i32,
    pub force_play: bool,
    pub looped: bool,
}

pub fn water_surface_crossed(client: &mut Client, resources: &mut Resources, views: &[View], from: Vec3, to: Vec3) -> Result<bool,String> {
    for view in views {
        let tile = resources.tile(client,&view.name,&view.level)?;
        for volume in tile.volumes.iter().filter(|volume| volume.props.l2_water) {
            let actor = &view.level.actors[volume.index];
            let pose = view.poses.get(&actor.name);
            if pose.is_some_and(|pose| pose.disabled) { continue; }
            let placement = pose.map_or(actor.placement,|pose| pose.placement);
            let data = resources.model(client,&volume.brush)?;
            let transform = super::transform::actor(placement,actor.scale,[0.0;3]);
            let unscaled = actor.draw_scale == 1.0 && actor.draw_scale_3d == [1.0;3];
            let model = Model { tree:&data.tree,data:&data.render,transform:Some(transform),unscaled };
            if model.trace_line(from.to_array(),to.to_array(),0).is_some() { return Ok(true); }
        }
    }
    Ok(false)
}
