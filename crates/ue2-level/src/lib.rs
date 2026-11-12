//! Placed actors and scene cameras read from an Unreal Engine 2 map package.
//!
//! Only what the client draws from is extracted: where each actor sits and what it shows. Property
//! values stay in the package; nothing is converted.

mod bsp;
mod emitters;
mod fog;
mod lighting;
mod movement;
pub mod movers;
pub mod projectors;
mod shots;
mod sky;
mod terrain;
mod zones;

use std::collections::BTreeMap;

use ue2_assets::{Error, Property, find, object_properties};
use ue2_package::{ObjectRef, Package};

pub use bsp::{BspModel, read as read_model, HIDDEN as HIDDEN_SURFACE_FLAGS, PF_L2WATER, BspNode, BspPolygon, BspRender, BspSection, BspTree, BspVertex, Leaf, LightBitmapRecord, LightMap, LightMapGroup, NodeDraw, RenderSurface, StaticLightMap, Zone};
pub use emitters::{BeamEndpoint, BeamShape, DrawStyle, Emitter, MeshShape, RibbonShape, Range, SpriteEmitter, class_emitter};
pub use fog::{FOG_RANGE_LEVELS, FogColorInfo, L2FogInfo};
pub use lighting::{LAYER_ALL_MIXED, LAYER_ALL_UNKNOWN, SECTOR_MAP_LEN, SECTOR_MAP_SIDE, TerrainSector};
pub use movement::{Movement, Sway, rotator_vector};
pub use shots::Shot;
pub use sky::{LENS_FLARE_SLOTS, SkyActors, SkyBody, SkyZoneInfo};
pub use terrain::{Coords, DecoLayer, Terrain, TerrainLayer, TerrainNative};
pub use zones::ZoneProperties;

/// Pitch, yaw and roll in Unreal units, 65536 to a full turn.
pub type Rotator = [i32; 3];

/// A position and facing in world units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub location: [f32; 3],
    pub rotation: Rotator,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Actor {
    pub class: String,
    pub name: String,
    pub tag: Option<String>,
    /// Editor groups, without the implicit `None`.
    pub groups: Vec<String>,
    pub placement: Placement,
    /// `DrawScale` times `DrawScale3D` on each axis.
    pub scale: [f32; 3],
    pub draw_scale: f32,
    pub draw_scale_3d: [f32; 3],
    /// Object path of the static mesh, e.g. `L2_Lobby.BTree_A`.
    pub static_mesh: Option<String>,
    /// Material overrides by slot, as object paths; `None` keeps the mesh's own material for that slot.
    pub skins: Vec<Option<String>>,
    /// Drawn at full brightness, ignoring lighting.
    pub unlit: bool,
    pub sun_affect: bool,
    pub special_lit: bool,
    pub max_lights: u8,
    pub ambient_glow: u8,
    pub owner: Option<String>,
    pub use_owner_ambient: bool,
    /// Precomputed RGBA lighting per mesh vertex; empty when the level stores none.
    pub lighting: Vec<[u8; 4]>,
    /// How the actor sways, for a movable static mesh actor.
    pub movement: Option<Movement>,
    pub l2_movement: Option<movers::L2Movement>,
    pub movement_tags: Vec<String>,
    pub movement_event: Option<String>,
    pub mover: Option<movers::Mover>,
    pub server_id: Option<i32>,
    /// `bHidden`: `FilterActor` drops it.
    pub hidden: bool,
    /// `bStatic`: distributed by BSP leaf, never ticked by the level.
    pub is_static: bool,
    /// `bIgnoredRange`: exempt from the `ClippingRange` cut.
    pub ignored_range: bool,
    /// `bTimeReactor`: drawn only from `ShowTime` to `HideTime` (hours, cyclic).
    pub time_reactor: Option<[f32; 2]>,
    pub l2_events: Vec<L2Event>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct L2Event {
    pub id: i32,
    pub cmd: u8,
}

pub const LEC_SHOW: u8 = 1;
pub const LEC_PLAY: u8 = 2;

#[derive(Debug, Clone, PartialEq)]
pub struct Light {
    pub name: String,
    pub location: [f32; 3],
    pub radius: f32,
    pub brightness: f32,
    pub hue: u8,
    pub saturation: u8,
    pub light_type: u8,
    pub effect: u8,
    /// `EmitterLightType`: 0 lights pawns only, 1 the world only, 2 both.
    pub emitter_light_type: u8,
    /// `bTimeLight`: switched by `LightOnTime`/`LightOffTime`.
    pub time_light: bool,
    /// `LightPeriod`/`LightPhase` of the pulsing types.
    pub period: u8,
    pub phase: u8,
    /// `LightCone`: the software spot aperture, `k = 1 − cone/256`.
    pub cone: u8,
    /// `Rotation.Vector()`: where a spot or the sun points.
    pub direction: [f32; 3],
    /// The actor's `Rotation` in engine units, what `AActor::GetViewRotation` hands the spot and cone effects.
    pub rotation: [i32; 3],
    /// `bSpecialLit`: only lights actors with the same flag.
    pub special_lit: bool,
    /// `bCorona` with its sprite and sizes.
    pub corona: Option<Corona>,
}

/// What `FCoronaRender` needs from a `bCorona` light.
#[derive(Debug, Clone, PartialEq)]
pub struct Corona {
    pub skin: String,
    pub min_size: f32,
    pub max_size: f32,
    pub draw_scale: f32,
    /// `ScaleGlow`: the fade speed is `2 · ScaleGlow` per second.
    pub scale_glow: f32,
    /// `UseOwnFinalBlend`: the skin's own material instead of the standard translucent blend.
    pub own_blend: bool,
}

/// Linear distance fog of a zone.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fog {
    /// RGB; alpha is unused.
    pub color: [u8; 4],
    /// Distance where fog begins, in world units.
    pub start: f32,
    /// Distance where fog hides everything.
    pub end: f32,
    /// `DistanceFogBlendTime`: seconds of transition when entering from a terrain zone.
    pub blend_time: f32,
}

/// Where a scene puts the camera and the fog of the zone it lands in.
#[derive(Debug, Clone, PartialEq)]
pub struct Warp {
    pub placement: Placement,
    pub fog: Option<Fog>,
    /// Name of the zone the camera lands in.
    pub zone: Option<String>,
    /// The zone's current time-of-day state, which picks terrain intensity maps.
    pub zone_state: Option<u8>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Level {
    pub actors: Vec<Actor>,
    /// Where each scene that starts with a warp puts the camera, by the scene's tag.
    pub warps: BTreeMap<String, Warp>,
    /// Every scene's camera shots, by the scene's tag.
    pub shots: BTreeMap<String, Vec<Shot>>,
    pub terrains: Vec<Terrain>,
    pub emitters: Vec<Emitter>,
    pub projectors: Vec<projectors::Projector>,
    pub detail_mode: u8,
    /// The level's BSP polygons that draw.
    pub bsp: Vec<BspPolygon>,
    /// The node tree of the same model, for finding the zone of a point.
    pub bsp_tree: BspTree,
    /// Sections, lightmaps and time slices the client draws the same model from.
    pub bsp_render: BspRender,
    /// The model's zones, indexed by the zone numbers in the tree and polygons.
    pub zones: Vec<Zone>,
    /// Lighting and fog of every zone actor, by actor name.
    pub zone_infos: BTreeMap<String, ZoneProperties>,
    pub lights: Vec<Light>,
    /// The sky zone and the sun/moon sprites of this level.
    pub sky: SkyActors,
    /// Local fogs, in actor order (the order their `Tick` registers them).
    pub fog_infos: Vec<L2FogInfo>,
}

const WORLD_LIGHT_RADIUS_STEP: f32 = 25.0;
const LIGHT_DEFAULT_RADIUS: f32 = 64.0;
const LIGHT_DEFAULT_BRIGHTNESS: f32 = 64.0;
const LIGHT_DEFAULT_SATURATION: u8 = 255;
const LIGHT_TYPE_STEADY: u8 = 1;
const LIGHT_TYPE_NONE: u8 = 0;
const LIGHT_DEFAULT_PERIOD: u8 = 32;
const LIGHT_DEFAULT_CONE: u8 = 128;
const CORONA_DEFAULT_MAX: f32 = 1000.0;

/// Reads every placed actor, recognized by the `Level` reference the editor stores in each of them.
pub fn read_level(package: &Package, file: &[u8]) -> Result<Level, Error> {
    read_level_in(package, file, None)
}

pub fn read_level_in(package: &Package, file: &[u8], package_name: Option<&str>) -> Result<Level, Error> {
    let mut level = Level { detail_mode: 2, ..Level::default() };
    let mut scenes = Vec::new();
    for (index, export) in package.exports().iter().enumerate() {
        // Brushes keep empty models of their own; the level's is the one with polygons.
        if package.class_name(export).eq_ignore_ascii_case("Model") {
            if let Some(model) = bsp::read(package, file, export)?
                && model.polygons.len() > level.bsp.len()
            {
                level.bsp = model.polygons;
                level.bsp_tree = model.tree;
                level.zones = model.zones;
                level.bsp_render = model.render;
            }
            continue;
        }
        if export.serial_size == 0 || matches!(export.class, ObjectRef::Null) {
            continue;
        }
        let properties = object_properties(package, file, export)?;
        if find(&properties, "Level").is_none() {
            continue;
        }
        let actor = actor(package, file, index, &properties, package_name)?;
        if actor.class.eq_ignore_ascii_case("LevelInfo") {
            level.detail_mode = find(&properties, "DetailMode").and_then(Property::byte).unwrap_or(2);
        }
        if let Some(projector) = projectors::read(package, &properties, package_name, &actor.class, &actor.name) {
            level.projectors.push(projector);
        }
        if let Some(mut light) = light(package, &actor.class, &properties) {
            light.name = actor.name.clone();
            level.lights.push(light);
        }
        if actor.class.eq_ignore_ascii_case("SceneManager") {
            scenes.push((actor.tag.clone(), properties));
        } else if actor.class.eq_ignore_ascii_case("TerrainInfo")
            && let Some(terrain) = terrain::read(package, file, index, &properties, package_name)
        {
            level.terrains.push(terrain);
        } else if actor.class.eq_ignore_ascii_case("Emitter") {
            let zone = zone(package, &properties).map(|zone| package.object_name(ObjectRef::Export(zone)).to_owned());
            level.emitters.push(emitters::read(package, file, &properties, zone, package_name)?);
        } else if actor.class.eq_ignore_ascii_case("L2FogInfo") {
            level.fog_infos.push(fog::read(package, &properties));
        } else if zones::is_zone_info(&actor.class) {
            if actor.class.eq_ignore_ascii_case("SkyZoneInfo") {
                level.sky.zone = Some(sky::sky_zone(package, &properties));
            }
            level.zone_infos.insert(actor.name.clone(), ZoneProperties::read(package, &actor.class, &properties));
        } else if actor.class.eq_ignore_ascii_case("NSun") {
            level.sky.sun = Some(sky::sky_body(package, &properties, false));
        } else if actor.class.eq_ignore_ascii_case("NMoon") {
            level.sky.moons.push(sky::sky_body(package, &properties, true));
        }
        level.actors.push(actor);
    }
    for (tag, properties) in scenes {
        let Some(tag) = tag else { continue };
        level.shots.insert(tag.clone(), shots::read(package, file, &properties)?);
        if let Some(warp) = first_warp(package, file, &properties)? {
            level.warps.insert(tag, warp);
        }
    }
    Ok(level)
}

fn actor(package: &Package, file: &[u8], export: usize, properties: &[Property<'_>], package_name: Option<&str>) -> Result<Actor, Error> {
    let get = |name: &str| find(properties, name);
    let path = |object: ObjectRef| match object {
        ObjectRef::Null => None,
        ObjectRef::Export(index) if package_name.is_some() => package_name.map(|name| format!("{name}.{}#{index}", package.object_path(object))),
        _ => Some(package.object_path(object)),
    };
    let draw_scale = get("DrawScale").and_then(Property::float).unwrap_or(1.0);
    let scale3d = get("DrawScale3D").and_then(Property::vector).unwrap_or([1.0; 3]);
    let exports = package.exports();
    let lighting = match get("StaticMeshInstance").and_then(|instance| instance.object(package)) {
        Some(ObjectRef::Export(instance)) => match exports.get(instance) {
            Some(instance) => lighting::vertex_colors(package, file, instance)?,
            None => Vec::new(),
        },
        _ => Vec::new(),
    };
    let class = exports.get(export).map(|export| package.class_name(export).to_owned()).unwrap_or_default();
    Ok(Actor {
        movement: movement::read(&class, properties, placement(properties)),
        l2_movement: movers::read_movement(package, &class, properties, placement(properties)),
        movement_tags: movers::read_tags(package, properties),
        movement_event: get("L2MoveEvent").and_then(|p| p.name_value(package)).filter(|n| !n.eq_ignore_ascii_case("None")).map(str::to_owned),
        mover: movers::read_mover(package, &class, properties, placement(properties)),
        server_id: get("L2ServerObjectRealID").and_then(Property::int),
        class,
        name: package.object_name(ObjectRef::Export(export)).to_owned(),
        tag: get("Tag").and_then(|tag| tag.name_value(package)).map(str::to_owned),
        groups: get("Group")
            .and_then(|group| group.name_value(package))
            .map(|groups| groups.split(',').filter(|group| *group != "None").map(str::to_owned).collect())
            .unwrap_or_default(),
        placement: placement(properties),
        scale: scale3d.map(|axis| axis * draw_scale),
        draw_scale,
        draw_scale_3d: scale3d,
        static_mesh: get("StaticMesh").and_then(|mesh| mesh.object(package)).and_then(path),
        skins: get("Skins")
            .and_then(|skins| skins.objects(package))
            .map(|skins| skins.into_iter().map(path).collect())
            .unwrap_or_default(),
        unlit: get("bUnlit").and_then(Property::bool).unwrap_or(false),
        sun_affect: get("bSunAffect").and_then(Property::bool).unwrap_or(false),
        special_lit: get("bSpecialLit").and_then(Property::bool).unwrap_or(false),
        max_lights: get("MaxLights").and_then(Property::byte).unwrap_or(4),
        ambient_glow: get("AmbientGlow").and_then(Property::byte).unwrap_or(0),
        owner: get("Owner").and_then(|p| p.object(package)).filter(|r| !matches!(r, ObjectRef::Null)).map(|r| package.object_name(r).to_owned()),
        use_owner_ambient: get("bUseOwnerAmbient").and_then(Property::bool).unwrap_or(false),
        lighting,
        hidden: get("bHidden").and_then(Property::bool).unwrap_or(false),
        is_static: get("bStatic").and_then(Property::bool).unwrap_or(false),
        ignored_range: get("bIgnoredRange").and_then(Property::bool).unwrap_or(false),
        time_reactor: get("bTimeReactor").and_then(Property::bool).unwrap_or(false).then(|| {
            let hour = |name: &str| get(name).and_then(Property::float).unwrap_or(0.0);
            [hour("ShowTime"), hour("HideTime")]
        }),
        l2_events: l2_events(package, properties),
    })
}

pub fn l2_events(package: &Package, properties: &[Property<'_>]) -> Vec<L2Event> {
    find(properties, "L2GameEvent")
        .and_then(|events| events.structs(package))
        .map(|events| {
            events
                .iter()
                .map(|fields| L2Event {
                    id: find(fields, "EventID").and_then(Property::int).unwrap_or(0),
                    cmd: find(fields, "EventCmd").and_then(Property::byte).unwrap_or(0),
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn l2_event_commands(events: &[L2Event], id: i32) -> impl Iterator<Item = u8> + '_ {
    events.iter().filter(move |event| event.cmd != 0 && event.id == id).map(|event| event.cmd)
}

pub fn light(package: &Package, class: &str, properties: &[Property<'_>]) -> Option<Light> {
    let get = |name: &str| find(properties, name);
    let byte = |name: &str, default: u8| get(name).and_then(Property::byte).unwrap_or(default);
    let float = |name: &str, default: f32| get(name).and_then(Property::float).unwrap_or(default);
    let default_type = if class.eq_ignore_ascii_case("Light") { LIGHT_TYPE_STEADY } else { LIGHT_TYPE_NONE };
    let light_type = byte("LightType", default_type);
    if light_type == LIGHT_TYPE_NONE && !get("bCorona").and_then(Property::bool).unwrap_or(false) {
        return None;
    }
    Some(Light {
        name: String::new(),
        location: placement(properties).location,
        radius: (float("LightRadius", LIGHT_DEFAULT_RADIUS) + 1.0) * WORLD_LIGHT_RADIUS_STEP,
        brightness: float("LightBrightness", LIGHT_DEFAULT_BRIGHTNESS),
        hue: byte("LightHue", 0),
        saturation: byte("LightSaturation", LIGHT_DEFAULT_SATURATION),
        light_type,
        effect: byte("LightEffect", 0),
        emitter_light_type: byte("EmitterLightType", 0),
        time_light: get("bTimeLight").and_then(Property::bool).unwrap_or(false),
        period: byte("LightPeriod", LIGHT_DEFAULT_PERIOD),
        phase: byte("LightPhase", 0),
        cone: byte("LightCone", LIGHT_DEFAULT_CONE),
        direction: movement::rotator_vector(placement(properties).rotation),
        rotation: placement(properties).rotation,
        special_lit: get("bSpecialLit").and_then(Property::bool).unwrap_or(false),
        corona: get("bCorona")
            .and_then(Property::bool)
            .unwrap_or(false)
            .then(|| {
                let skin = get("Skins")
                    .and_then(|skins| skins.objects(package))
                    .and_then(|skins| skins.into_iter().next())
                    .filter(|skin| !matches!(skin, ObjectRef::Null))
                    .map(|skin| package.object_path(skin))?;
                Some(Corona {
                    skin,
                    min_size: float("MinCoronaSize", 0.0),
                    max_size: float("MaxCoronaSize", CORONA_DEFAULT_MAX),
                    draw_scale: float("DrawScale", 1.0),
                    scale_glow: float("ScaleGlow", 1.0),
                    own_blend: get("UseOwnFinalBlend").and_then(Property::bool).unwrap_or(false),
                })
            })
            .flatten(),
    })
}

fn placement(properties: &[Property<'_>]) -> Placement {
    Placement {
        location: find(properties, "Location").and_then(Property::vector).unwrap_or_default(),
        rotation: find(properties, "Rotation").and_then(Property::rotator).unwrap_or_default(),
    }
}

/// The interpolation point the scene's first action warps to, if it starts with a warp.
fn first_warp(package: &Package, file: &[u8], scene: &[Property<'_>]) -> Result<Option<Warp>, Error> {
    let export = |object: Option<ObjectRef>| match object {
        Some(ObjectRef::Export(index)) => package.exports().get(index),
        _ => None,
    };
    let actions = find(scene, "Actions").and_then(|actions| actions.objects(package)).unwrap_or_default();
    let Some(action) = export(actions.first().copied()) else { return Ok(None) };
    if !package.class_name(action).eq_ignore_ascii_case("ActionWarp") {
        return Ok(None);
    }
    let action = object_properties(package, file, action)?;
    let Some(point) = export(find(&action, "IntPoint").and_then(|point| point.object(package))) else {
        return Ok(None);
    };
    let point = object_properties(package, file, point)?;
    // The zone an actor stands in is the first field of its Region.
    let zone = zone(package, &point);
    let (fog, zone_state) = match zone.and_then(|zone| package.exports().get(zone)) {
        Some(zone) => {
            let zone = object_properties(package, file, zone)?;
            let state = find(&zone, "CurZoneState");
            let state = state
                .and_then(Property::byte)
                .or_else(|| state.and_then(Property::int).and_then(|state| u8::try_from(state).ok()));
            (fog(&zone), state)
        }
        None => (None, None),
    };
    let zone = zone.map(|zone| package.object_name(ObjectRef::Export(zone)).to_owned());
    Ok(Some(Warp { placement: placement(&point), fog, zone, zone_state }))
}

/// The export index of the zone an actor stands in: the `Zone` field of its `Region`.
fn zone(package: &Package, actor: &[Property<'_>]) -> Option<usize> {
    let region = find(actor, "Region")?.fields(package)?;
    match find(&region, "Zone")?.object(package)? {
        ObjectRef::Export(index) => Some(index),
        _ => None,
    }
}

fn fog(zone: &[Property<'_>]) -> Option<Fog> {
    find(zone, "bDistanceFog").and_then(Property::bool).filter(|&fogged| fogged)?;
    let float = |name| find(zone, name).and_then(Property::float);
    // Unset values fall back to UT2004's ZoneInfo defaults.
    let color = find(zone, "DistanceFogColor").and_then(Property::color).unwrap_or([0, 0, 0, 255]);
    Some(Fog {
        color,
        start: float("DistanceFogStart").unwrap_or(3000.0),
        end: float("DistanceFogEnd").unwrap_or(8000.0),
        blend_time: float("DistanceFogBlendTime").unwrap_or(1.0),
    })
}

#[cfg(test)]
mod tests;
