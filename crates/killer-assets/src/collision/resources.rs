use crate::client::Client;
use l2_catalog::StaticCollision;
use rustc_hash::FxHashMap;
use std::{borrow::Cow, collections::HashMap, mem::size_of, sync::{Arc,Weak}};
use ue2_level::{BspModel,Level};

const CACHE_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone)]
pub struct Bounds {
    pub bounds: [[f32;3];2],
    pub model: Option<String>,
}

pub struct Actor {
    pub index: usize,
    pub trace_flags: u32,
    pub primitive: Primitive,
    pub pre_pivot: [f32;3],
    pub static_mesh_actor: bool,
    pub immediately_stop: bool,
    pub pushable_mass: Option<f32>,
    pub placed: std::sync::OnceLock<Placed>,
}

#[derive(Clone, Copy, Debug)]
pub struct Placed {
    pub transform: glam::Mat4,
    pub corners: Option<(glam::Vec3,glam::Vec3)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicsProps {
    pub gravity: [f32;3],
    pub zone_velocity: [f32;3],
    pub ground_friction: f32,
    pub terminal_velocity: f32,
    pub fluid_friction: f32,
    pub priority: i32,
    pub water: bool,
    pub l2_water: bool,
    pub l2_step: bool,
    pub step_sound_id: i32,
}

pub struct VolumeActor {
    pub index: usize,
    pub brush: String,
    pub props: PhysicsProps,
}

#[derive(Debug, Clone)]
pub struct MusicVolumeActor {
    pub index: usize,
    pub brush: String,
    pub music_id: i32,
    pub force_play: bool,
    pub looped: bool,
}

pub enum Primitive {
    Mesh(String),
    Model(String),
    Cylinder { radius: f32, height: f32 },
}

pub const COLLISION_FIELDS: [&str;9] = ["bCollideActors","bWorldGeometry","bBlockNonZeroExtentTraces","bUseCylinderCollision","bProjTarget","bBlockActors","bBlockPlayers","bImmediatelyStop","bPushable"];

pub fn eligible_trace_flags(mover: bool, flags: &[bool;9]) -> Option<u32> {
    let trace_flags = if mover { 2 } else if flags[1] { 0x80 } else if flags[4] || (flags[5] && flags[6]) { 0x20 } else { 0 };
    (flags[0] && flags[2] && trace_flags != 0).then_some(trace_flags)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClassCollision {
    pub trace_flags: u32,
    pub cylinder: Option<(f32,f32)>,
    pub pre_pivot: [f32;3],
    pub draw_scale: f32,
    pub draw_scale_3d: [f32;3],
    pub static_mesh_actor: bool,
    pub immediately_stop: bool,
    pub pushable_mass: Option<f32>,
}

impl ClassCollision {
    pub fn scale(&self) -> [f32;3] {
        self.draw_scale_3d.map(|axis| axis * self.draw_scale)
    }

    pub fn unscaled(&self) -> bool {
        self.draw_scale == 1.0 && self.draw_scale_3d == [1.0;3]
    }
}

pub struct Tile {
    pub actors: Vec<Actor>,
    pub volumes: Vec<VolumeActor>,
    pub default_volume: Option<PhysicsProps>,
    pub music_volumes: Vec<MusicVolumeActor>,
    pub terrains: Vec<super::terrain::Vertices>,
}

struct CachedMesh {
    mesh: Arc<StaticCollision>,
    used: u64,
    bytes: usize,
}

struct CachedModel {
    model: Arc<BspModel>,
    used: u64,
    bytes: usize,
}

fn lowercase(path: &str) -> Cow<'_, str> {
    if path.bytes().any(|byte| byte.is_ascii_uppercase() || !byte.is_ascii()) { Cow::Owned(path.to_lowercase()) } else { Cow::Borrowed(path) }
}

#[derive(Default)]
pub struct Resources {
    bounds: FxHashMap<String,Bounds>,
    meshes: FxHashMap<String,CachedMesh>,
    models: FxHashMap<String,CachedModel>,
    tiles: Vec<(Weak<Level>,Arc<Tile>)>,
    classes: FxHashMap<String,Option<ClassCollision>>,
    clock: u64,
    bytes: usize,
}

impl Resources {
    pub fn tile(&mut self, client: &mut Client, name: &str, level: &Arc<Level>) -> Result<Arc<Tile>,String> {
        self.tiles.retain(|(level,_)| level.strong_count() > 0);
        if let Some((_,tile)) = self.tiles.iter().find(|(key,_)| key.ptr_eq(&Arc::downgrade(level))) { return Ok(tile.clone()); }
        let fields = COLLISION_FIELDS;
        let mut classes = HashMap::new();
        let mut sizes = HashMap::new();
        let mut static_mesh_actors = HashMap::new();
        let mut decorations = HashMap::new();
        let mut masses = HashMap::new();
        let mut actors = Vec::new();
        for (index,actor) in level.actors.iter().enumerate() {
            let path = format!("{name}.{}",actor.name);
            let (class,flags,pre_pivot,brush,size,mass) = client.catalog().inspect_object(&path,"",|package,file,export| {
                let properties = ue2_assets::object_properties(package,file,export).ok()?;
                let brush = ue2_assets::find(&properties,"Brush").and_then(|p| p.object(package)).and_then(|object| match object {
                    ue2_package::ObjectRef::Null => None,
                    ue2_package::ObjectRef::Export(_) => Some(format!("{name}.{}",package.object_path(object))),
                    _ => Some(package.object_path(object)),
                });
                let size = ["CollisionRadius","CollisionHeight"].map(|name| ue2_assets::find(&properties,name).and_then(ue2_assets::Property::float));
                let mass = ue2_assets::find(&properties,"Mass").and_then(ue2_assets::Property::float);
                Some((package.object_path(export.class),fields.map(|name| ue2_assets::find(&properties,name).and_then(ue2_assets::Property::bool)),
                    ue2_assets::find(&properties,"PrePivot").and_then(ue2_assets::Property::vector).unwrap_or([0.0;3]),brush,size,mass))
            }).ok_or_else(|| format!("propriedades de colisão ausentes: {path}"))?;
            if !classes.contains_key(&class) {
                let defaults = client.class_boolean_defaults(&class,&fields).map_err(|e| format!("defaults {class}: {e}"))?;
                classes.insert(class.clone(),defaults);
            }
            let defaults = &classes[&class];
            let flags: [bool;9] = std::array::from_fn(|i| flags[i].or_else(|| defaults.get(fields[i]).copied()).unwrap_or(false));
            let Some(trace_flags) = eligible_trace_flags(actor.mover.is_some(),&flags) else { continue; };
            let resolved = actor.static_mesh.as_ref().filter(|mesh| client.catalog().inspect_object(mesh,"StaticMesh",|_,_,_| Some(())).is_some());
            let primitive = if !flags[3] && let Some(mesh) = resolved {
                self.bounds(client,mesh)?;
                Primitive::Mesh(mesh.clone())
            } else if !flags[3] && let Some(brush) = brush {
                Primitive::Model(brush)
            } else {
                if !sizes.contains_key(&class) {
                    sizes.insert(class.clone(),client.class_float_defaults(&class,&["CollisionRadius","CollisionHeight"]).map_err(|e| format!("dimensões {class}: {e}"))?);
                }
                let size = ["CollisionRadius","CollisionHeight"].map(|name| sizes[&class].get(name).copied()).into_iter().zip(size)
                    .map(|(default,value)| value.or(default).unwrap_or(0.0)).collect::<Vec<_>>();
                Primitive::Cylinder { radius:size[0],height:size[1] }
            };
            if !static_mesh_actors.contains_key(&class) {
                let is_a = client.class_is_a(&class,"Engine.StaticMeshActor").map_err(|e| format!("herança {class}: {e}"))?;
                static_mesh_actors.insert(class.clone(),is_a);
            }
            if !decorations.contains_key(&class) {
                let is_a = client.class_is_a(&class,"Engine.Decoration").map_err(|e| format!("herança {class}: {e}"))?;
                decorations.insert(class.clone(),is_a);
            }
            let pushable_mass = if decorations[&class] && flags[8] {
                if !masses.contains_key(&class) {
                    let defaults = client.class_float_defaults(&class,&["Mass"]).map_err(|e| format!("massa {class}: {e}"))?;
                    masses.insert(class.clone(),defaults.get("Mass").copied());
                }
                Some(mass.or(masses[&class]).unwrap_or(0.0))
            } else {
                None
            };
            actors.push(Actor { index,trace_flags,primitive,pre_pivot,static_mesh_actor:static_mesh_actors[&class],immediately_stop:flags[7],pushable_mass,placed:Default::default() });
        }
        let mut volumes = Vec::new();
        let mut music_volumes = Vec::new();
        let mut default_volume = None;
        for (index,actor) in level.actors.iter().enumerate() {
            if !actor.class.to_ascii_lowercase().ends_with("volume") { continue; }
            let path = format!("{name}.{}",actor.name);
            let (class,props_brush) = client.catalog().inspect_object(&path,"",|package,file,export| {
                let properties = ue2_assets::object_properties(package,file,export).ok()?;
                let brush = ue2_assets::find(&properties,"Brush").and_then(|p| p.object(package)).and_then(|object| match object {
                    ue2_package::ObjectRef::Null => None,
                    ue2_package::ObjectRef::Export(_) => Some(format!("{name}.{}",package.object_path(object))),
                    _ => Some(package.object_path(object)),
                });
                let read = |field: &str| ue2_assets::find(&properties,field);
                let own = (
                    ["Gravity","ZoneVelocity"].map(|field| read(field).and_then(ue2_assets::Property::vector)),
                    ["GroundFriction","TerminalVelocity","FluidFriction"].map(|field| read(field).and_then(ue2_assets::Property::float)),
                    ["bWaterVolume","bL2WaterVolume","bL2StepVolume"].map(|field| read(field).and_then(ue2_assets::Property::bool)),
                    ["Priority","StepSoundID"].map(|field| read(field).and_then(ue2_assets::Property::int)),
                    (read("nMusicID").and_then(ue2_assets::Property::int),["bForcePlayMusic","bLoopMusic"].map(|field| read(field).and_then(ue2_assets::Property::bool))),
                );
                Some((package.object_path(export.class),(brush,own)))
            }).ok_or_else(|| format!("propriedades de volume ausentes: {path}"))?;
            let (brush,(vectors,floats,bools,ints,(music_id,music_flags))) = props_brush;
            if client.class_is_a(&class,"Engine.MusicVolume").map_err(|e| format!("herança {class}: {e}"))? {
                if let Some(brush) = brush {
                    let int_defaults = client.class_int_defaults(&class,&["nMusicID"]).map_err(|e| format!("defaults {class}: {e}"))?;
                    let bool_defaults = client.class_boolean_defaults(&class,&["bForcePlayMusic","bLoopMusic"]).map_err(|e| format!("defaults {class}: {e}"))?;
                    let flag = |own: Option<bool>,field: &str| own.or_else(|| bool_defaults.get(field).copied()).unwrap_or(false);
                    music_volumes.push(MusicVolumeActor {
                        index,brush,
                        music_id: music_id.or_else(|| int_defaults.get("nMusicID").copied()).unwrap_or(0),
                        force_play: flag(music_flags[0],"bForcePlayMusic"),looped: flag(music_flags[1],"bLoopMusic"),
                    });
                }
                continue;
            }
            if !client.class_is_a(&class,"Engine.PhysicsVolume").map_err(|e| format!("herança {class}: {e}"))? { continue; }
            let vector_defaults = client.class_vector_defaults(&class,&["Gravity","ZoneVelocity"]).map_err(|e| format!("defaults {class}: {e}"))?;
            let float_defaults = client.class_float_defaults(&class,&["GroundFriction","TerminalVelocity","FluidFriction"]).map_err(|e| format!("defaults {class}: {e}"))?;
            let bool_defaults = client.class_boolean_defaults(&class,&["bWaterVolume","bL2WaterVolume","bL2StepVolume"]).map_err(|e| format!("defaults {class}: {e}"))?;
            let int_defaults = client.class_int_defaults(&class,&["Priority","StepSoundID"]).map_err(|e| format!("defaults {class}: {e}"))?;
            let pick = |own: Option<[f32;3]>,field: &str| own.or_else(|| vector_defaults.get(field).copied()).unwrap_or([0.0;3]);
            let float = |own: Option<f32>,field: &str| own.or_else(|| float_defaults.get(field).copied()).unwrap_or(0.0);
            let flag = |own: Option<bool>,field: &str| own.or_else(|| bool_defaults.get(field).copied()).unwrap_or(false);
            let int = |own: Option<i32>,field: &str| own.or_else(|| int_defaults.get(field).copied()).unwrap_or(0);
            let props = PhysicsProps {
                gravity: pick(vectors[0],"Gravity"),zone_velocity: pick(vectors[1],"ZoneVelocity"),
                ground_friction: float(floats[0],"GroundFriction"),terminal_velocity: float(floats[1],"TerminalVelocity"),fluid_friction: float(floats[2],"FluidFriction"),
                water: flag(bools[0],"bWaterVolume"),l2_water: flag(bools[1],"bL2WaterVolume"),l2_step: flag(bools[2],"bL2StepVolume"),
                priority: int(ints[0],"Priority"),step_sound_id: int(ints[1],"StepSoundID"),
            };
            if class.eq_ignore_ascii_case("Engine.DefaultPhysicsVolume") {
                default_volume = Some(props);
            } else if let Some(brush) = brush {
                volumes.push(VolumeActor { index,brush,props });
            }
        }
        let terrains = level.terrains.iter().map(|terrain| {
            let heightmap = client.catalog().heightmap_with_base(&terrain.heightmap,terrain.base_height)
                .ok_or_else(|| format!("heightmap de colisão ausente: {}",terrain.heightmap))?;
            super::terrain::Vertices::new(terrain,[heightmap.width,heightmap.height],&heightmap.samples)
        }).collect::<Result<_,_>>()?;
        let tile = Arc::new(Tile { actors,volumes,default_volume,music_volumes,terrains });
        self.tiles.push((Arc::downgrade(level),tile.clone()));
        Ok(tile)
    }

    pub fn class_collision(&mut self, client: &mut Client, class: &str) -> Result<Option<ClassCollision>,String> {
        if let Some(cached) = self.classes.get(class) { return Ok(*cached); }
        let defaults = client.class_boolean_defaults(class,&COLLISION_FIELDS).map_err(|e| format!("defaults {class}: {e}"))?;
        let flags: [bool;9] = std::array::from_fn(|i| defaults.get(COLLISION_FIELDS[i]).copied().unwrap_or(false));
        let collision = match eligible_trace_flags(false,&flags) {
            None => None,
            Some(trace_flags) => {
                let floats = client.class_float_defaults(class,&["CollisionRadius","CollisionHeight","DrawScale","Mass"]).map_err(|e| format!("dimensões {class}: {e}"))?;
                let vectors = client.class_vector_defaults(class,&["PrePivot","DrawScale3D"]).map_err(|e| format!("escala {class}: {e}"))?;
                let float = |name: &str| floats.get(name).copied();
                let static_mesh_actor = client.class_is_a(class,"Engine.StaticMeshActor").map_err(|e| format!("herança {class}: {e}"))?;
                let decoration = client.class_is_a(class,"Engine.Decoration").map_err(|e| format!("herança {class}: {e}"))?;
                Some(ClassCollision {
                    trace_flags,
                    cylinder: flags[3].then(|| (float("CollisionRadius").unwrap_or(0.0),float("CollisionHeight").unwrap_or(0.0))),
                    pre_pivot: vectors.get("PrePivot").copied().unwrap_or([0.0;3]),
                    draw_scale: float("DrawScale").unwrap_or(1.0),
                    draw_scale_3d: vectors.get("DrawScale3D").copied().unwrap_or([1.0;3]),
                    static_mesh_actor,
                    immediately_stop: flags[7],
                    pushable_mass: (decoration && flags[8]).then(|| float("Mass").unwrap_or(0.0)),
                })
            }
        };
        self.classes.insert(class.to_owned(),collision);
        Ok(collision)
    }

    pub fn absorb(&mut self, other: Resources) {
        self.bounds.extend(other.bounds);
        self.tiles.retain(|(level, _)| level.strong_count() > 0);
        self.tiles.extend(other.tiles.into_iter().filter(|(level, _)| level.strong_count() > 0));
        for (key, class) in other.classes {
            self.classes.entry(key).or_insert(class);
        }
        for (key, mut mesh) in other.meshes {
            if !self.meshes.contains_key(&key) {
                self.clock = self.clock.wrapping_add(1);
                mesh.used = self.clock;
                self.bytes += mesh.bytes;
                self.meshes.insert(key, mesh);
            }
        }
        for (key, mut model) in other.models {
            if !self.models.contains_key(&key) {
                self.clock = self.clock.wrapping_add(1);
                model.used = self.clock;
                self.bytes += model.bytes;
                self.models.insert(key, model);
            }
        }
        self.reserve(0);
    }

    pub fn has_tile(&self, level: &Arc<Level>) -> bool {
        self.tiles.iter().any(|(key, _)| key.ptr_eq(&Arc::downgrade(level)))
    }

    pub fn bounds(&mut self, client: &mut Client, path: &str) -> Result<Bounds,String> {
        if let Some(bounds) = self.bounds.get(path) { return Ok(bounds.clone()); }
        let key = lowercase(path);
        if let Some(bounds) = self.bounds.get(key.as_ref()) {
            let bounds = bounds.clone();
            self.bounds.insert(path.to_owned(),bounds.clone());
            return Ok(bounds);
        }
        let (mut bounds,model) = client.catalog().static_mesh_collision_bounds(path).ok_or_else(|| format!("limites de colisão ausentes: {path}"))?;
        if let Some(path) = &model {
            let model = self.model(client,path)?;
            bounds[0] = std::array::from_fn(|axis| bounds[0][axis].min(model.bounding_box[0][axis]));
            bounds[1] = std::array::from_fn(|axis| bounds[1][axis].max(model.bounding_box[1][axis]));
        }
        let bounds = Bounds { bounds,model };
        self.bounds.insert(key.into_owned(),bounds.clone());
        Ok(bounds)
    }

    pub fn mesh(&mut self, client: &mut Client, path: &str) -> Result<Arc<StaticCollision>,String> {
        self.clock = self.clock.wrapping_add(1);
        let key = lowercase(path);
        if let Some(cached) = self.meshes.get_mut(key.as_ref()) {
            cached.used = self.clock;
            return Ok(cached.mesh.clone());
        }
        let mesh = Arc::new(client.catalog().static_mesh_collision(path).ok_or_else(|| format!("colisão ausente: {path}"))?);
        let bytes = mesh.positions.capacity()*size_of::<[f32;3]>() + mesh.triangles.capacity()*size_of::<ue2_assets::CollisionTriangle>()
            + mesh.nodes.capacity()*size_of::<ue2_assets::CollisionNode>();
        self.reserve(bytes);
        if bytes <= CACHE_BYTES {
            self.bytes += bytes;
            self.meshes.insert(key.into_owned(),CachedMesh { mesh:mesh.clone(),used:self.clock,bytes });
        }
        Ok(mesh)
    }

    pub fn model(&mut self, client: &mut Client, path: &str) -> Result<Arc<BspModel>,String> {
        self.clock = self.clock.wrapping_add(1);
        let key = lowercase(path);
        if let Some(cached) = self.models.get_mut(key.as_ref()) {
            cached.used = self.clock;
            return Ok(cached.model.clone());
        }
        let mut model = client.catalog().inspect_object(path,"Model",|package,file,export| ue2_level::read_model(package,file,export).ok().flatten())
            .ok_or_else(|| format!("modelo de colisão ausente: {path}"))?;
        model.polygons = Vec::new();
        model.zones = Vec::new();
        model.render = ue2_level::BspRender {
            collision_bounds: std::mem::take(&mut model.render.collision_bounds),
            leaf_hulls: std::mem::take(&mut model.render.leaf_hulls),
            root_outside: model.render.root_outside,
            linked: model.render.linked,
            ..Default::default()
        };
        let bytes = model.tree.nodes.capacity()*size_of::<ue2_level::BspNode>()
            + model.render.collision_bounds.capacity()*size_of::<([[f32;3];2],u8)>()
            + model.render.leaf_hulls.capacity()*size_of::<i32>();
        let model = Arc::new(model);
        self.reserve(bytes);
        if bytes <= CACHE_BYTES {
            self.bytes += bytes;
            self.models.insert(key.into_owned(),CachedModel { model:model.clone(),used:self.clock,bytes });
        }
        Ok(model)
    }

    fn reserve(&mut self, bytes: usize) {
        while self.bytes.saturating_add(bytes) > CACHE_BYTES {
            let mesh = self.meshes.iter().min_by_key(|(_,entry)| entry.used).map(|(key,entry)| (key.clone(),entry.used));
            let model = self.models.iter().min_by_key(|(_,entry)| entry.used).map(|(key,entry)| (key.clone(),entry.used));
            match (mesh,model) {
                (Some((key,used)),model) if model.as_ref().is_none_or(|(_,other)| used <= *other) => {
                    if let Some(old) = self.meshes.remove(&key) { self.bytes -= old.bytes; }
                }
                (_,Some((key,_))) => {
                    if let Some(old) = self.models.remove(&key) { self.bytes -= old.bytes; }
                }
                _ => break,
            }
        }
    }

    pub fn cached_bytes(&self) -> usize { self.bytes }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eligible_trace_flags_follow_should_trace_and_the_nonzero_extent_rule() {
        let flags = |on: &[usize]| std::array::from_fn::<bool,9,_>(|i| on.contains(&i));
        assert_eq!(eligible_trace_flags(false,&flags(&[0,1,2])),Some(0x80));
        assert_eq!(eligible_trace_flags(true,&flags(&[0,1,2])),Some(2));
        assert_eq!(eligible_trace_flags(false,&flags(&[0,2,4])),Some(0x20));
        assert_eq!(eligible_trace_flags(false,&flags(&[0,2,5,6])),Some(0x20));
        assert_eq!(eligible_trace_flags(false,&flags(&[0,2,5])),None);
        assert_eq!(eligible_trace_flags(false,&flags(&[1,2])),None);
        assert_eq!(eligible_trace_flags(false,&flags(&[0,1])),None);
        let shape = ClassCollision { trace_flags:0x80,cylinder:None,pre_pivot:[0.0;3],draw_scale:2.0,draw_scale_3d:[1.0,0.5,3.0],static_mesh_actor:true,immediately_stop:true,pushable_mass:None };
        assert_eq!(shape.scale(),[2.0,1.0,6.0]);
        assert!(!shape.unscaled());
        assert!(ClassCollision { draw_scale:1.0,draw_scale_3d:[1.0;3],..shape }.unscaled());
    }

    #[test]
    fn shared_cache_evicts_least_recent_mesh_or_model_without_invalidating_active_query() {
        let mesh = Arc::new(StaticCollision { bounding_box:[[0.0;3];2],simple_line:false,simple_box:false,positions:vec![[1.0,2.0,3.0]],triangles:Vec::new(),nodes:Vec::new(),model:None });
        let model = Arc::new(BspModel { bounding_box:[[0.0;3];2],polygons:Vec::new(),tree:Default::default(),zones:Vec::new(),render:Default::default() });
        let mut cache = Resources::default();
        let half = CACHE_BYTES/2;
        cache.meshes.insert("old".into(),CachedMesh { mesh:mesh.clone(),used:1,bytes:half });
        cache.models.insert("new".into(),CachedModel { model:model.clone(),used:2,bytes:half });
        cache.bytes = CACHE_BYTES;
        cache.reserve(1);
        assert!(cache.meshes.is_empty());
        assert_eq!(cache.models.len(),1);
        assert_eq!(cache.cached_bytes(),half);
        assert_eq!(Arc::strong_count(&mesh),1);
        assert_eq!(mesh.positions[0],[1.0,2.0,3.0]);
        cache.reserve(CACHE_BYTES+1);
        assert!(cache.models.is_empty());
        assert_eq!(cache.cached_bytes(),0);
        assert_eq!(Arc::strong_count(&model),1);
    }
}
