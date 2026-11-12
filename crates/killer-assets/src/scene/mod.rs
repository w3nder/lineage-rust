pub mod actor_view;
pub mod actor_lighting;
pub mod bsp;
pub mod deco;
mod instancing;
pub use instancing::DecorationInstance;
pub mod emitters;
pub mod lightmap_regen;
pub mod lights;
pub mod props;
pub mod movers;
pub mod projectors;
pub mod sky;
pub mod sway;
pub mod terrain;
pub mod terrain_normals;
pub mod transform;

use crate::client::Client;
use crate::options::{ClientOptions, fog_index, range_units};
use crate::textures::TextureFormatKind;
use bytemuck::{Pod, Zeroable};
use l2_catalog::{AlphaOp, BlendFactor, ColorOp, CoordSource, FogMode, Material, Stage, StageSource, TextureRef};
use ue2_level::{Level, Terrain, ZoneProperties};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub mask_uv: [f32; 2],
    pub color: [f32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Pass {
    Opaque,
    Layer(u16),
    Projector,
    Translucent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DrawKind {
    StaticMesh,
    Bsp,
    Terrain,
    Actor,
    /// A camera-facing quad lit only by its vertex color (`RenderSun`/`RenderMoon`/`RenderSprite`).
    Sprite,
    Particle,
    Projector,
}

/// One of the LOD meshes of a `bStaticMeshLod` actor: drawn when the pawn's 2D distance picks `level`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LodSelect {
    pub level: u8,
    /// `bStaticMeshLodBlend`: the cross-fade of 1.8 s between neighbouring LODs.
    pub blend: bool,
    /// `LodRange01/02 · 2048`.
    pub ranges: [f32; 2],
}

/// What `RenderLevel` and `RenderStaticMesh` test per frame before drawing an actor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActorCull {
    /// Unique among the actors one `Builder` made; the key of the actor's `ActorView` state.
    pub id: u32,
    pub ambient: actor_lighting::Ambient,
    pub xy: [f32; 2],
    /// `ClippingRange.StaticMesh` or `.StaticMeshLod` in world units.
    pub range: f32,
    pub ignored_range: bool,
    pub time_reactor: Option<[f32; 2]>,
    pub lod: Option<LodSelect>,
}

/// A terrain sector as `ATerrainInfo::Render` culls it: `Location` against the view target in XY, the box against the frustum.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SectorBounds {
    pub location: [f32; 3],
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[derive(Clone, Debug, PartialEq)]
pub struct MeshDraw {
    pub instances: Vec<DecorationInstance>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: Material,
    pub mask: Option<String>,
    pub kind: DrawKind,
    pub sun_affect: bool,
    pub pass: Pass,
    pub zone: u8,
    pub lightmap: Option<usize>,
    pub modulate_2x: bool,
    pub sector: Option<SectorBounds>,
    /// `FadeoutRadius` of a decoration layer: alpha goes from 1 at `min` to 0 at `max` of camera distance.
    pub fade: Option<[f32; 2]>,
    /// Per-hour color the renderer writes over the material's TFACTOR (`UpdateSkyColors`).
    pub tint: Option<sky::SkyTint>,
    /// Distance cut, time reactor and LOD level of the static mesh actor this draw belongs to.
    pub actor: Option<ActorCull>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LightmapGroup {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormatKind,
    pub slices: Vec<Vec<Vec<u8>>>,
}

pub fn texture_material(path: &str) -> Material {
    let texture = TextureRef { path: path.to_owned(), frames: Vec::new(), rate: 0.0, total_frames: 0, size: [0.0; 2], clamp: [false; 2] };
    let stage = Stage { source: StageSource::Texture(texture), uv: Vec::new(), coord_source: CoordSource::Stream(0), color_op: ColorOp::SelectTexture, alpha_op: AlphaOp::Texture, swapped: false };
    Material {
        stages: vec![stage],
        passes: vec![l2_catalog::Pass {
            stages: 0..1,
            src: BlendFactor::One,
            dst: BlendFactor::Zero,
            alpha_blend: false,
            z_write: true,
            z_test: true,
            alpha_test: None,
            two_sided: false,
            fog: FogMode::Zone,
            tfactor: None,
            fade: None,
            fade_alpha_replicate: false,
            lit: true,
            light_after: 1,
        }],
        sorting_level: 0,
        transparent: false,
        root_class: "texture".to_owned(),
        color_modifier: None,
    }
}

pub fn resolve_material(client: &mut Client, path: &str) -> Material {
    client.catalog().material(path).unwrap_or_else(|| texture_material(path))
}

pub fn draw_pass(material: &Material) -> Pass {
    if material.requires_sorting() { Pass::Translucent } else { Pass::Opaque }
}

pub fn first_texture_path(material: &Material) -> String {
    material.first_texture().map(|texture| texture.path.clone()).unwrap_or_default()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FogParams {
    pub color: [f32; 3],
    pub start: f32,
    pub end: f32,
    /// `DistanceFogBlendTime`, seconds.
    pub blend_time: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZoneLighting {
    pub fog: Option<FogParams>,
    pub sun_affect: bool,
    pub ambient_brightness: u8,
    pub ambient_vector: Option<[f32; 3]>,
    pub is_sky: bool,
    pub sky_location: [f32; 3],
    /// `bTerrainZone`: the camera in it takes the global `[FOG]` ranges and the hazering color.
    pub terrain_zone: bool,
    /// `bClearToFogColor`: with `bDistanceFog`, the frame is cleared to `DistanceFogColor`.
    pub clear_to_fog: bool,
}

impl Default for ZoneLighting {
    fn default() -> Self {
        Self { fog: None, sun_affect: true, ambient_brightness: 0, ambient_vector: None, is_sky: false, sky_location: [0.0; 3], terrain_zone: false, clear_to_fog: false }
    }
}

impl From<&ZoneProperties> for ZoneLighting {
    fn from(info: &ZoneProperties) -> Self {
        Self {
            fog: info.distance_fog.map(|fog| FogParams {
                color: [fog.color[0], fog.color[1], fog.color[2]].map(|byte| byte as f32 / 255.0),
                start: fog.start,
                end: fog.end,
                blend_time: fog.blend_time,
            }),
            sun_affect: info.sun_affect,
            ambient_brightness: info.ambient_brightness,
            ambient_vector: info.ambient_vector,
            is_sky: info.is_sky,
            sky_location: info.location,
            terrain_zone: info.terrain_zone,
            clear_to_fog: info.clear_to_fog_color,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    pub projectors: Vec<projectors::ProjectorState>,
    pub moving: Vec<movers::MovingAsset>,
    pub emitters: Vec<emitters::EmitterAsset>,
    pub draws: Vec<MeshDraw>,
    pub lightmaps: Vec<LightmapGroup>,
    pub zones: Vec<ZoneLighting>,
    pub lights: Vec<lights::PointLight>,
    pub coronas: Vec<lights::CoronaLight>,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    /// `ClippingRange Terrain` in world units; sectors farther than this from the view target are not drawn.
    pub terrain_range: f32,
    pub sky_bodies: Vec<sky::SkyBodyDraw>,
    pub flares: Vec<sky::Flare>,
    /// `env.int [FOG]` ranges for the client's `ClippingRange.Terrain`, in world units, and `FogSpeed`.
    pub global_fog: GlobalFog,
    pub fog_infos: Vec<ue2_level::L2FogInfo>,
}

/// `GL2FogStart · 2048`, `GL2FogEnd · 2048` and `GL2FogSpeed`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlobalFog {
    pub start: f32,
    pub end: f32,
    pub speed: f32,
    /// Zero-based index into `FogRange1..5` of the `L2FogInfo`s.
    pub level: usize,
}

impl Default for GlobalFog {
    fn default() -> Self {
        Self::of(&ClientOptions::default())
    }
}

impl GlobalFog {
    pub fn of(options: &ClientOptions) -> Self {
        let (start, end) = options.fog_range();
        Self { start, end, speed: options.fog.speed, level: fog_index(options.clipping.terrain, options.keep_min_frame_rate) - 1 }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self { projectors: Vec::new(), moving: Vec::new(), emitters: Vec::new(), draws: Vec::new(), lightmaps: Vec::new(), zones: Vec::new(), lights: Vec::new(), coronas: Vec::new(), bounds_min: [f32::MAX; 3], bounds_max: [f32::MIN; 3], terrain_range: f32::INFINITY, sky_bodies: Vec::new(), flares: Vec::new(), global_fog: GlobalFog::default(), fog_infos: Vec::new() }
    }
}

impl Scene {
    pub fn retain_draws(&mut self, mut keep: impl FnMut(&MeshDraw) -> bool) {
        let mut indices = Vec::with_capacity(self.draws.len());
        let mut next = 0;
        self.draws.retain(|draw| {
            let retained = keep(draw);
            indices.push(retained.then_some(next));
            next += usize::from(retained);
            retained
        });
        for projector in &mut self.projectors {
            projector.draws = projector.draws.iter().filter_map(|&index| indices.get(index).copied().flatten()).collect();
        }
    }

    pub fn material_draws(&self) -> impl Iterator<Item = &MeshDraw> {
        self.draws.iter().chain(self.emitters.iter().flat_map(|a| &a.parts).flat_map(|p| &p.draws)).chain(self.moving.iter().flat_map(|a| a.draws.iter().chain(a.grades.iter().flatten())))
    }
    /// The sun, moons and lens flares of `level`, orbiting its sky zone.
    pub fn attach_sky(&mut self, client: &mut Client, level: &Level) {
        let Some(zone) = self.zones.iter().position(|zone| zone.is_sky).and_then(|zone| u8::try_from(zone).ok()) else { return };
        self.sky_bodies = sky::sky_bodies(client, &level.sky, zone);
        self.flares = sky::flares(&level.sky);
    }

    pub fn zone(&self, id: u8) -> ZoneLighting {
        self.zones.get(usize::from(id)).copied().unwrap_or_default()
    }

    fn push(&mut self, draw: MeshDraw) {
        if draw.indices.is_empty() {
            return;
        }
        if self.zone(draw.zone).is_sky {
            self.draws.push(draw);
            return;
        }
        for v in draw.world_vertices() {
            for axis in 0..3 {
                self.bounds_min[axis] = self.bounds_min[axis].min(v.position[axis]);
                self.bounds_max[axis] = self.bounds_max[axis].max(v.position[axis]);
            }
        }
        self.draws.push(draw);
    }
}

pub fn zone_lighting(level: &Level) -> Vec<ZoneLighting> {
    let fallback = level.actors.iter().find(|actor| actor.class.eq_ignore_ascii_case("LevelInfo")).and_then(|actor| level.zone_infos.get(&actor.name));
    level
        .zones
        .iter()
        .map(|zone| zone.actor.as_deref().and_then(|actor| level.zone_infos.get(actor)).or(fallback).map(ZoneLighting::from).unwrap_or_default())
        .collect()
}

pub fn sun_affect(zones: &[ZoneLighting], zone: u8) -> bool {
    zones.get(usize::from(zone)).is_none_or(|zone| zone.sun_affect)
}

pub fn terrain_zone(level: &Level, terrain: &Terrain) -> u8 {
    level
        .zones
        .iter()
        .position(|zone| zone.actor.as_deref().and_then(|actor| level.zone_infos.get(actor)).is_some_and(|info| info.terrain_zone))
        .and_then(|index| u8::try_from(index).ok())
        .unwrap_or_else(|| level.bsp_tree.zone_at(terrain.location))
}

/// The loaded levels a tile is stitched to: `+X`, `+Y` and the `(+X, +Y)` diagonal (`SetVertiEdge`, `SetHoriEdge`, `SetEndVertexZ`).
#[derive(Clone, Copy, Default)]
pub struct Neighbours<'a> {
    pub right: Option<&'a Level>,
    pub top: Option<&'a Level>,
    pub corner: Option<&'a Level>,
}

impl<'a> Neighbours<'a> {
    fn terrain_seam(&self) -> terrain::TerrainSeam<'a> {
        let first = |level: Option<&'a Level>| level.and_then(|level| level.terrains.first());
        terrain::TerrainSeam { right: first(self.right), top: first(self.top), corner: first(self.corner) }
    }
}

pub fn build_scene(client: &mut Client, level: &Level) -> Scene {
    let options = ClientOptions::load(client).unwrap_or_else(|why| {
        log::warn!("opções do cliente: {why}; valores padrão do engine");
        ClientOptions::default()
    });
    build_scene_with(client, level, &options, Neighbours::default())
}

pub fn build_scene_with(client: &mut Client, level: &Level, options: &ClientOptions, neighbours: Neighbours<'_>) -> Scene {
    let mut scene = Scene {
        projectors: Vec::new(),
        moving: movers::build(client, level, &options.clipping),
        emitters: emitters::build(client, level, options),
        draws: Vec::new(),
        lightmaps: bsp::lightmap_groups(&level.bsp_render),
        zones: zone_lighting(level),
        lights: lights::point_lights(&level.lights),
        coronas: if options.coronas { lights::coronas(client, &level.lights) } else { Vec::new() },
        bounds_min: [f32::MAX; 3],
        bounds_max: [f32::MIN; 3],
        terrain_range: range_units(options.clipping.terrain),
        sky_bodies: Vec::new(),
        flares: Vec::new(),
        global_fog: GlobalFog::of(options),
        fog_infos: level.fog_infos.clone(),
    };
    for terrain in &level.terrains {
        if !terrain.sectors.is_empty() {
            scene.lightmaps.push(terrain::intensity_group(terrain, terrain.native.heightmap_x.max(1) as usize, terrain.native.heightmap_y.max(1) as usize));
        }
    }
    for draw in terrain_draws(client, level, &scene.zones, neighbours) {
        scene.push(draw);
    }
    let (draws, mut projectors) = projectors::build_with_states(client, level, neighbours);
    for projector in &mut projectors {
        for index in &mut projector.draws { *index += scene.draws.len(); }
    }
    scene.projectors = projectors;
    for draw in draws {
        scene.push(draw);
    }
    for draw in decoration_draws(client, level, &scene.zones, options) {
        scene.push(draw);
    }
    for draw in bsp::build(client, &level.bsp_tree, &level.bsp_render, &scene.zones) {
        scene.push(draw);
    }
    for draw in props::build(client, level, &options.clipping) {
        scene.push(draw);
    }
    for draw in projectors::build_static_mesh_with_states(client, level, scene.draws.len(), &mut scene.projectors) {
        scene.push(draw);
    }
    if options.shared_sky {
        let zones = scene.zones.clone();
        scene.retain_draws(|draw| !zones.get(usize::from(draw.zone)).is_some_and(|zone| zone.is_sky));
    } else {
        scene.attach_sky(client, level);
    }
    for draw in &mut scene.draws {
        draw.tint = sky::sky_tint(&draw.material, &options.sky);
    }
    scene
}

/// Each terrain with the index of its intensity-map group: the BSP lightmap groups come first, then one group per terrain with sectors.
fn terrain_intensity_groups(level: &Level) -> Vec<(&Terrain, Option<usize>)> {
    let mut next = level.bsp_render.groups.len();
    level
        .terrains
        .iter()
        .map(|terrain| {
            let intensity = (!terrain.sectors.is_empty()).then(|| {
                next += 1;
                next - 1
            });
            (terrain, intensity)
        })
        .collect()
}

pub fn decoration_draws(client: &mut Client, level: &Level, zones: &[ZoneLighting], options: &ClientOptions) -> Vec<MeshDraw> {
    if options.keep_min_frame_rate { return Vec::new(); }
    let mut draws = Vec::new();
    for (terrain, intensity) in terrain_intensity_groups(level) {
        let zone = terrain_zone(level, terrain);
        draws.extend(deco::build(client, terrain, zone, sun_affect(zones, zone), intensity, options.render_deco));
    }
    draws.retain(|draw| !draw.indices.is_empty());
    draws
}

pub fn decoration_points(client: &mut Client, level: &Level, options: &ClientOptions) -> deco::GrassField {
    let mut points = Vec::new();
    for terrain in &level.terrains {
        points.extend(deco::points(client, terrain, options.render_deco));
    }
    deco::GrassField::new(&points)
}

/// The terrain draws of a level, stitched to `neighbours`, with the same lightmap indices `build_scene_with` gives them.
pub fn terrain_draws(client: &mut Client, level: &Level, zones: &[ZoneLighting], neighbours: Neighbours<'_>) -> Vec<MeshDraw> {
    let terrain_lights = lights::terrain_lights(&level.lights);
    let mut draws = Vec::new();
    for (terrain, intensity) in terrain_intensity_groups(level) {
        let zone = terrain_zone(level, terrain);
        draws.extend(terrain::build(client, terrain, zone, sun_affect(zones, zone), intensity, &terrain_lights, &neighbours.terrain_seam()));
    }
    draws.retain(|draw| !draw.indices.is_empty());
    draws
}

pub(crate) fn flat_normals(vertices: &mut [Vertex], indices: &[u32]) {
    for tri in indices.as_chunks::<3>().0 {
        let [a, b, c] = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        let pa = glam::Vec3::from(vertices[a].position);
        let pb = glam::Vec3::from(vertices[b].position);
        let pc = glam::Vec3::from(vertices[c].position);
        let n = (pb - pa).cross(pc - pa).normalize_or_zero();
        for i in [a, b, c] {
            let acc = glam::Vec3::from(vertices[i].normal) + n;
            vertices[i].normal = acc.to_array();
        }
    }
    for v in vertices {
        v.normal = glam::Vec3::from(v.normal).normalize_or(glam::Vec3::Z).to_array();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sun_affect_comes_from_the_zone_and_defaults_to_true() {
        let zones = vec![ZoneLighting::default(), ZoneLighting { sun_affect: false, ..ZoneLighting::default() }];
        assert!(sun_affect(&zones, 0));
        assert!(!sun_affect(&zones, 1));
        assert!(sun_affect(&zones, 7));
    }
}
