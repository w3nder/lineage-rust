use crate::camera::{Camera, Frustum};
use crate::environment::{Environment, EnvironmentUniform, SkyState};
use crate::fog::{FogFrame, FogInput, FogState};
use crate::headless::CLEAR_RGBA;
use crate::pipeline::{COLOR_FORMAT, CameraUniform, DrawContext, MATERIAL_STRIDE, MAX_STAGES, MaterialGpu, PipelineKey, Pipelines, ZoneUniform};
use crate::present::Present;
use crate::skinned::{CharacterId, CharacterInstance, ObjectUniform, SkinnedVertex, aligned_stride, valid_sections};
use crate::sky::{QUAD_INDICES, body_quad, flare_quad, lens_flare, pixel_projection, place_body, project, sky_camera};
use crate::textures::{GpuTextures, SharedView, TextureSource};
pub use crate::viewport::ViewportTarget;
use crate::ui::UiRenderer;
use glam::Mat4;
use killer_assets::characters::CharacterMesh;
use killer_assets::scene::sky::{Flare, SkyBodyDraw, SkyTint, flare_material};
use killer_assets::scene::actor_view::{ActorView, FrameInput, Levels, LodInput};
use killer_assets::scene::props::{lod_for, time_reactor_shows};
use killer_assets::scene::lights::{MAX_HARDWARE_LIGHTS, PointLight};
use killer_assets::scene::{texture_material, ActorCull, DrawKind, GlobalFog, MeshDraw, Pass, Scene, SectorBounds, ZoneLighting};
use ue2_level::L2FogInfo;
use killer_assets::textures::TextureData;
use killer_assets::tiles::tile_name;
use std::sync::Arc;
use std::collections::HashSet;
use killer_ui::draw::DrawList;
use l2_catalog::{AlphaOp, BlendFactor, ColorOp, CoordSource, FogMode, Material, StageSource};
use std::num::NonZeroU64;
use wgpu::util::DeviceExt;

mod particles;
pub use particles::{EffectId, EffectPlacement, RibbonInput};
mod movers;
mod material_upload;
mod spawned;
use material_upload::MaterialUpload;
mod radar_pass;

const OBJECT_SLOTS: u64 = 16;

struct GpuCorona {
    light: killer_assets::scene::lights::CoronaLight,
    state: crate::coronas::CoronaState,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    material: GpuMaterial,
    tile: usize,
}
const PALETTE_MATRICES: u64 = 256;
const MATERIAL_SLOTS: u64 = 1024;

struct PassViews {
    textures: Vec<SharedView>,
    clamps: Vec<bool>,
    cube: SharedView,
    mask: SharedView,
    lightmap: SharedView,
}

struct GpuPass {
    key: PipelineKey,
    bind_group: wgpu::BindGroup,
    views: PassViews,
    slot: u32,
    animated: bool,
    pass: usize,
    /// `L2ColorModifier`'s stage changes this pass when the actor carries an alpha (`pipeline::modifier_class`).
    fades: bool,
}

struct GpuMaterial {
    material: Material,
    context: DrawContext,
    passes: Vec<GpuPass>,
}

struct GpuDraw {
    instances: Option<wgpu::Buffer>,
    instance_count: u32,
    projector: Option<usize>,
    source: usize,
    moving: Option<movers::MovingDraw>,
    spawned: Option<u64>,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    base_vertex: i32,
    first_index: u32,
    index_count: u32,
    material: GpuMaterial,
    pass: Pass,
    sort_level: u8,
    center: glam::Vec3,
    radius: f32,
    zone: u32,
    sector: Option<SectorBounds>,
    actor: Option<ActorCull>,
    view: Option<usize>,
    tile: usize,
}

/// `AActor`'s sight and LOD state, with what it decided on the last frame the actor was in a visible leaf.
struct ActorState {
    view: ActorView,
    levels: Levels,
    stamp: u64,
    slots: [u32; 3],
}

const SHARED_VERTEX_BYTES: u64 = 48 << 20;
const SHARED_INDEX_BYTES: u64 = 24 << 20;

#[derive(Default)]
struct UniformSlots {
    groups: Vec<(wgpu::Buffer, wgpu::BindGroup)>,
    next: usize,
}

struct Geometry {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    base_vertex: i32,
    first_index: u32,
}

const NO_FADE: u32 = u32::MAX;
const FADE_CAPACITY: usize = 256;

impl Default for ActorState {
    fn default() -> Self {
        Self { view: ActorView::default(), levels: [Some(None); 3], stamp: 0, slots: [NO_FADE; 3] }
    }
}

/// A sun or moon: its quad is rewritten every frame from the hour and the sky camera.
struct GpuBody {
    body: SkyBodyDraw,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    material: GpuMaterial,
    zone: u32,
    tile: usize,
}

struct GpuFlare {
    flare: Flare,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    material: GpuMaterial,
    tile: usize,
}

/// One loaded level: its zones and lightmaps live in a contiguous range of the renderer's tables.
struct GpuTile {
    projectors: Vec<killer_assets::scene::projectors::ProjectorState>,
    visible_projectors: Vec<bool>,
    moving: Vec<movers::MovingState>,
    name: String,
    zone_base: u32,
    zone_count: usize,
    lightmap_base: usize,
    lightmap_count: usize,
    visible: bool,
    fog_infos: Vec<L2FogInfo>,
    views: Vec<ActorState>,
    view_ids: rustc_hash::FxHashMap<u32, usize>,
}

/// Where the viewer stands this frame: the local zone of the tile that contains `position`, the position and the frame time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewer {
    pub zone: u8,
    pub position: glam::Vec3,
    pub dt: f32,
}

struct GpuSection {
    first_index: u32,
    index_count: u32,
    material: GpuMaterial,
    translucent: bool,
}

struct GpuCharacter {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    sections: Vec<GpuSection>,
    radius: f32,
}

#[derive(Clone, Copy)]
struct DrawInstance {
    character: usize,
    offset: u32,
    center: glam::Vec3,
    actor_id: i32,
    position: glam::Vec3,
    collision_height: f32,
    shadow_caster: bool,
    radius: f32,
}

struct ShadowTarget {
    _color: wgpu::Texture,
    color_view: wgpu::TextureView,
    _blurred: wgpu::Texture,
    blurred_view: wgpu::TextureView,
    blurred_array_view: wgpu::TextureView,
    blur_group: wgpu::BindGroup,
    _depth: wgpu::Texture,
    depth_view: wgpu::TextureView,
    owner: Option<i32>,
}

struct ShadowTargets {
    entries: Vec<ShadowTarget>,
}

impl ShadowTargets {
    fn new(device: &wgpu::Device, blur: &crate::shadow_blur::ShadowBlur) -> Self {
        let entries = (0..crate::shadow::SHADOW_TARGETS)
            .map(|_| {
                let color = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("sombra do pawn"),
                    size: wgpu::Extent3d { width: crate::shadow::SHADOW_SIZE, height: crate::shadow::SHADOW_SIZE, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: crate::pipeline::COLOR_FORMAT,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });
                let depth = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("profundidade da sombra do pawn"),
                    size: wgpu::Extent3d { width: crate::shadow::SHADOW_SIZE, height: crate::shadow::SHADOW_SIZE, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: crate::pipeline::DEPTH_FORMAT,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                });
                let color_view = color.create_view(&wgpu::TextureViewDescriptor::default());
                let color_array_view = color.create_view(&wgpu::TextureViewDescriptor { dimension: Some(wgpu::TextureViewDimension::D2Array), array_layer_count: Some(1), ..Default::default() });
                let blurred = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("sombra do pawn desfocada"),
                    size: wgpu::Extent3d { width: crate::shadow::SHADOW_SIZE, height: crate::shadow::SHADOW_SIZE, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: crate::pipeline::COLOR_FORMAT,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });
                let blurred_view = blurred.create_view(&wgpu::TextureViewDescriptor::default());
                let blurred_array_view = blurred.create_view(&wgpu::TextureViewDescriptor { dimension: Some(wgpu::TextureViewDimension::D2Array), array_layer_count: Some(1), ..Default::default() });
                let blur_group = blur.group(device, &color_array_view);
                let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
                ShadowTarget { _color: color, color_view, _blurred: blurred, blurred_view, blurred_array_view, blur_group, _depth: depth, depth_view, owner: None }
            })
            .collect();
        Self { entries }
    }

    fn begin_frame(&mut self) {
        for target in &mut self.entries {
            target.owner = None;
        }
    }

    fn acquire(&mut self, actor_id: i32) -> Option<usize> {
        let index = self.entries.iter().position(|target| target.owner == Some(actor_id)).or_else(|| self.entries.iter().position(|target| target.owner.is_none()))?;
        self.entries[index].owner = Some(actor_id);
        Some(index)
    }
}

struct ShadowProjector {
    material: GpuMaterial,
    projection: crate::shadow::Projection,
}

struct ObjectBuffers {
    stride: u64,
    objects: wgpu::Buffer,
    palette: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

struct MaterialSlots {
    buffer: wgpu::Buffer,
    free: Vec<u32>,
    next: u32,
}

impl MaterialSlots {
    fn new(device: &wgpu::Device, slots: u64) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("materiais"), size: MATERIAL_STRIDE * slots, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        Self { buffer, free: Vec::new(), next: 0 }
    }

    fn capacity(&self) -> u32 {
        (self.buffer.size() / MATERIAL_STRIDE) as u32
    }

    fn available(&self) -> u32 {
        self.capacity() - self.next + self.free.len() as u32
    }

    fn take(&mut self) -> Option<u32> {
        if let Some(slot) = self.free.pop() {
            return Some(slot);
        }
        (self.next < self.capacity()).then(|| {
            self.next += 1;
            self.next - 1
        })
    }

    fn grow(&mut self, device: &wgpu::Device, wanted: u32) {
        let slots = u64::from((self.capacity() + wanted).next_power_of_two());
        self.buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("materiais"), size: MATERIAL_STRIDE * slots, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    }

    fn write(&self, queue: &wgpu::Queue, slot: u32, material: &MaterialGpu) {
        queue.write_buffer(&self.buffer, u64::from(slot) * MATERIAL_STRIDE, bytemuck::bytes_of(material));
    }
}

pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipelines: Pipelines,
    textures: GpuTextures,
    samplers: [wgpu::Sampler; 2],
    texture_filtering: crate::video::TextureFiltering,
    color_format: wgpu::TextureFormat,
    sample_count: u32,
    multisample_frame: Option<crate::video::MultisampleFrame>,
    draws: Vec<GpuDraw>,
    bodies: Vec<GpuBody>,
    flares: Vec<GpuFlare>,
    coronas: Vec<GpuCorona>,
    particle_actors: Vec<particles::GpuEmitter>,
    l2_event_actors: Vec<(usize, usize)>,
    ui_size: Option<(u32, u32)>,
    next_effect: u64,
    particle_time: f32,
    moving_time: f32,
    movers_auto_tick: bool,
    projectors: bool,
    projector_view_actor: Option<String>,
    projector_behind_view: bool,
    projector_wireframe: bool,
    door_bindings: std::collections::HashMap<i32, movers::DoorBinding>,
    particle_delta: Option<f32>,
    particle_seeded_random: u32,
    corona_random: u32,
    sun_visibility: crate::light_visibility::SunVisibility,
    sky: SkyState,
    sky_hour: f32,
    flare_clear: bool,
    fog: FogState,
    global_fog: GlobalFog,
    fog_level: usize,
    viewer: Option<Viewer>,
    last_fog: Option<FogFrame>,
    tiles: Vec<GpuTile>,
    fade_values: Vec<f32>,
    fade_buffer: wgpu::Buffer,
    fade_capacity: usize,
    frame_stamp: u64,
    lightmaps: Vec<(SharedView, u32)>,
    lightmap_revision: u32,
    zones: Vec<ZoneLighting>,
    zone_buffer: wgpu::Buffer,
    env_buffer: wgpu::Buffer,
    uniform_slots: std::cell::RefCell<UniformSlots>,
    characters: Vec<Option<GpuCharacter>>,
    instances: Vec<DrawInstance>,
    object_bytes: Vec<u8>,
    palette_matrices: Vec<[[f32; 4]; 4]>,
    object_buffers: ObjectBuffers,
    shadow_targets: ShadowTargets,
    shadow_blur: crate::shadow_blur::ShadowBlur,
    shadow_projectors: Vec<ShadowProjector>,
    pawn_shadow: bool,
    keep_min_frame_rate: bool,
    materials: MaterialSlots,
    material_upload: MaterialUpload,
    time: f32,
    terrain_range: f32,
    view_target: Option<glam::Vec3>,
    present: Option<Present>,
    postprocess: Option<crate::postprocess::PostProcess>,
    pub(crate) ui: UiRenderer,
    world_text: UiRenderer,
    lines: crate::lines::LineRenderer,
    radar: radar_pass::RadarGpu,
}

impl Renderer {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, color_format: wgpu::TextureFormat) -> Self {
        let pipelines = Pipelines::new(&device, COLOR_FORMAT);
        let present = (color_format != COLOR_FORMAT).then(|| Present::new(&device, color_format));
        let textures = GpuTextures::new(&device, &queue);
        let sampler = |address: wgpu::AddressMode| {
            device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: address,
                address_mode_v: address,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::MipmapFilterMode::Linear,
                ..Default::default()
            })
        };
        let samplers = [sampler(wgpu::AddressMode::Repeat), sampler(wgpu::AddressMode::ClampToEdge)];
        let zone_buffer = zone_buffer(&device, &[], 0);
        let env_buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("ambiente"), size: size_of::<EnvironmentUniform>() as u64, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let stride = aligned_stride(size_of::<ObjectUniform>() as u64, u64::from(device.limits().min_uniform_buffer_offset_alignment));
        let object_buffers = object_buffers(&device, &pipelines.object_layout, stride, OBJECT_SLOTS, PALETTE_MATRICES);
        let materials = MaterialSlots::new(&device, MATERIAL_SLOTS);
        let shadow_blur = crate::shadow_blur::ShadowBlur::new(&device, &queue, COLOR_FORMAT, crate::shadow::SHADOW_SIZE);
        let shadow_targets = ShadowTargets::new(&device, &shadow_blur);
        let ui = UiRenderer::new(&device, COLOR_FORMAT);
        let world_text = UiRenderer::new_premultiplied(&device, COLOR_FORMAT);
        let lines = crate::lines::LineRenderer::new(&device, COLOR_FORMAT, 1);
        let fade_buffer = fade_buffer(&device, FADE_CAPACITY);
        let radar = radar_pass::RadarGpu::new(&device);
        Self {
            device,
            queue,
            pipelines,
            textures,
            samplers,
            texture_filtering: crate::video::TextureFiltering { trilinear: true, mipmaps: true },
            color_format,
            sample_count: 1,
            multisample_frame: None,
            draws: Vec::new(),
            bodies: Vec::new(),
            flares: Vec::new(),
            coronas: Vec::new(),
            particle_actors: Vec::new(),
            l2_event_actors: Vec::new(),
            ui_size: None,
            next_effect: 0,
            particle_time: 0.0,
            moving_time: 0.0,
            movers_auto_tick: true,
            projectors: true,
            projector_view_actor: None,
            projector_behind_view: false,
            projector_wireframe: false,
            door_bindings: std::collections::HashMap::new(),
            particle_delta: None,
            particle_seeded_random: 1,
            corona_random: 1,
            sun_visibility: crate::light_visibility::SunVisibility::default(),
            sky: SkyState::default(),
            sky_hour: 0.0,
            flare_clear: true,
            fog: FogState::default(),
            global_fog: GlobalFog::default(),
            fog_level: 0,
            viewer: None,
            last_fog: None,
            tiles: Vec::new(),
            fade_values: Vec::new(),
            fade_buffer,
            fade_capacity: FADE_CAPACITY,
            frame_stamp: 0,
            lightmaps: Vec::new(),
            lightmap_revision: 0,
            zones: Vec::new(),
            zone_buffer,
            env_buffer,
            uniform_slots: Default::default(),
            characters: Vec::new(),
            instances: Vec::new(),
            object_bytes: Vec::new(),
            palette_matrices: Vec::new(),
            object_buffers,
            shadow_targets,
            shadow_blur,
            shadow_projectors: Vec::new(),
            pawn_shadow: true,
            keep_min_frame_rate: false,
            materials,
            material_upload: MaterialUpload::default(),
            time: 0.0,
            terrain_range: f32::INFINITY,
            view_target: None,
            present,
            postprocess: None,
            ui, world_text, lines,
            radar,
        }
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn reload_texture_detail(&mut self, detail: i32, source: &mut dyn TextureSource) {
        self.textures.set_texture_detail(detail);
        let textures = &mut self.textures;
        let device = &self.device;
        let queue = &self.queue;
        let mut reload = |material: &mut GpuMaterial| {
            for pass in &mut material.passes {
                let stages = material.material.pass_stages(&material.material.passes[pass.pass]);
                for (index, stage) in stages.iter().take(MAX_STAGES).enumerate() {
                    match &stage.source {
                        StageSource::Texture(texture) => {
                            let mut paths = vec![texture.path.clone()];
                            if texture.animated() { paths.extend(texture.frames.iter().cloned()); }
                            pass.views.textures[index] = textures.material_array_view(device, queue, source, &paths, pass.slot);
                        }
                        StageSource::Cubemap(cube) => pass.views.cube = textures.cube_view(device, queue, source, &cube.faces),
                        _ => {}
                    }
                }
            }
        };
        for draw in &mut self.draws { reload(&mut draw.material); }
        for body in &mut self.bodies { reload(&mut body.material); }
        for flare in &mut self.flares { reload(&mut flare.material); }
        for corona in &mut self.coronas { reload(&mut corona.material); }
        for draw in self.particle_actors.iter_mut().flat_map(|actor| actor.draws.iter_mut().flatten()) { reload(&mut draw.material); }
        for character in self.characters.iter_mut().flatten().chain(self.radar.meshes.iter_mut().flatten()) {
            for section in &mut character.sections { reload(&mut section.material); }
        }
        self.ui.reload_textures(device, queue, textures, source);
        self.refresh_materials();
    }

    pub fn set_postprocess(&mut self, settings: crate::postprocess::PostProcessSettings) -> Result<(), &'static str> {
        settings.validate()?;
        if settings.active() && !crate::postprocess::supported(&self.device) { return Err("postprocess target formats unavailable"); }
        if settings.active() && self.sample_count != 1 { return Err("postprocess requires antialiasing disabled"); }
        if !settings.active() {
            self.pipelines.set_color_format(&self.device, COLOR_FORMAT);
            if let Some(postprocess) = &mut self.postprocess { postprocess.settings = settings; }
            return Ok(());
        }
        self.present.get_or_insert_with(|| Present::new(&self.device, self.color_format));
        let format = if settings.hdr { wgpu::TextureFormat::Rgba16Float } else { COLOR_FORMAT };
        if self.postprocess.is_none() {
            self.postprocess = Some(crate::postprocess::PostProcess::new(&self.device, settings, self.time));
        } else if let Some(postprocess) = &mut self.postprocess { postprocess.set_settings(&self.device, settings); }
        self.pipelines.set_color_format(&self.device, format);
        Ok(())
    }

    pub fn set_antialiasing(&mut self, adapter: &wgpu::Adapter, index: i32) -> Result<(), &'static str> {
        if index < 0 || index > crate::video::multisample_capability(adapter, &self.device) {
            return Err("unsupported multisample selection");
        }
        let count = match index { 1 => 2, 2 => 4, _ => 1 };
        if count != 1 && self.postprocess.as_ref().is_some_and(|postprocess| postprocess.settings.active()) { return Err("antialiasing requires postprocess disabled"); }
        if count != self.sample_count {
            self.pipelines.set_sample_count(&self.device, count);
            self.ui.set_sample_count(&self.device, COLOR_FORMAT, count);
            self.world_text.set_sample_count(&self.device, COLOR_FORMAT, count);
            self.lines.set_sample_count(&self.device, COLOR_FORMAT, count);
            self.sample_count = count;
            self.multisample_frame = None;
        }
        Ok(())
    }

    pub fn texture_filtering(&self) -> crate::video::TextureFiltering {
        self.texture_filtering
    }

    pub fn set_texture_filtering(&mut self, filtering: crate::video::TextureFiltering) {
        if self.texture_filtering == filtering { return; }
        self.texture_filtering = filtering;
        self.ui.set_texture_filtering(&self.device, filtering);
        self.samplers = [wgpu::AddressMode::Repeat, wgpu::AddressMode::ClampToEdge]
            .map(|address| self.device.create_sampler(&filtering.sampler(address)));
        self.refresh_materials();
    }

    pub fn set_gamma(&mut self, settings: crate::video::GammaSettings) {
        let present = self.present.get_or_insert_with(|| Present::new(&self.device, self.color_format));
        present.set_gamma(&self.queue, settings);
    }

    pub fn set_video_clipping(&mut self, clipping: &killer_assets::options::ClippingRange, fog: GlobalFog) {
        self.terrain_range = clipping.terrain * 2048.0;
        self.global_fog = fog;
        self.fog_level = fog.level;
        for draw in &mut self.draws {
            if let Some(actor) = &mut draw.actor {
                actor.range = if actor.lod.is_some() { clipping.static_mesh_lod } else { clipping.static_mesh } * 2048.0;
            }
        }
    }

    pub fn set_time(&mut self, time: f32) {
        self.time = time;
    }

    pub fn set_no_fractal_anim(&mut self, disabled: bool) {
        self.textures.no_fractal_anim=disabled;
    }

    pub fn set_projectors(&mut self, enabled: bool) {
        self.projectors = enabled;
    }

    pub fn set_pawn_shadow(&mut self, enabled: bool, keep_min_frame_rate: bool) {
        self.pawn_shadow = enabled;
        self.keep_min_frame_rate = keep_min_frame_rate;
    }

    pub fn set_projector_view(&mut self, actor: Option<&str>, behind_view: bool, wireframe: bool) {
        self.projector_view_actor = actor.map(str::to_owned);
        self.projector_behind_view = behind_view;
        self.projector_wireframe = wireframe;
        self.update_projector_visibility();
    }

    fn update_projector_visibility(&mut self) {
        for tile in &mut self.tiles {
            tile.visible_projectors = killer_assets::scene::projectors::visible_projectors(
                &tile.projectors, self.projector_view_actor.as_deref(), self.projector_behind_view, self.projector_wireframe,
            );
        }
    }

    pub fn time(&self) -> f32 {
        self.time
    }

    pub fn set_view_target(&mut self, target: Option<glam::Vec3>) {
        self.view_target = target;
    }

    /// `RenderLevel`'s distance cut and `RenderStaticMesh`'s time reactor and LOD choice, all 2D from the pawn.
    fn actor_visible(&self, draw: &GpuDraw, eye: glam::Vec3) -> bool {
        let Some(actor) = draw.actor else { return true };
        if let Some([show, hide]) = actor.time_reactor
            && !time_reactor_shows(show, hide, self.sky_hour)
        {
            return false;
        }
        if let Some(state) = draw.view.and_then(|view| self.tiles.get(draw.tile)?.views.get(view)).filter(|state| state.stamp != 0) {
            return state.levels[actor.lod.map_or(0, |lod| usize::from(lod.level))].is_some();
        }
        let pawn = self.view_target.unwrap_or(eye);
        let distance = ((actor.xy[0] - pawn.x).powi(2) + (actor.xy[1] - pawn.y).powi(2)).sqrt();
        if !actor.ignored_range && distance >= actor.range {
            return false;
        }
        actor.lod.is_none_or(|lod| lod_for(distance, lod.ranges[0], lod.ranges[1]) == lod.level)
    }

    /// `RenderLevel` and `RenderStaticMesh` for every actor with a draw inside the frustum: the range band and LOD blend decide, per
    /// LOD level, whether the actor draws and with which alpha. The alphas go to the instance buffer the `faded` pipelines read.
    fn update_actor_views(&mut self, frustum: &Frustum, eye: glam::Vec3) {
        self.frame_stamp += 1;
        self.fade_values.clear();
        let pawn = self.view_target.unwrap_or(eye);
        for draw in &self.draws {
            let (Some(index), Some(actor)) = (draw.view, draw.actor) else { continue };
            let Some(tile) = self.tiles.get_mut(draw.tile) else { continue };
            if !tile.visible || draw.index_count == 0 || !frustum.contains_sphere(draw.center, draw.radius) {
                continue;
            }
            let state = &mut tile.views[index];
            if state.stamp == self.frame_stamp {
                continue;
            }
            state.stamp = self.frame_stamp;
            let distance = ((actor.xy[0] - pawn.x).powi(2) + (actor.xy[1] - pawn.y).powi(2)).sqrt();
            let lod = actor.lod.map(|lod| LodInput { ranges: lod.ranges, blend: lod.blend });
            state.levels = state.view.frame(FrameInput { distance, range: actor.range, ignored_range: actor.ignored_range, lod });
            state.slots = [NO_FADE; 3];
            for (level, entry) in state.levels.iter().enumerate() {
                if let Some(Some(alpha)) = entry {
                    state.slots[level] = self.fade_values.len() as u32;
                    self.fade_values.push(f32::from(*alpha) / 255.0);
                }
            }
        }
        if self.fade_values.is_empty() {
            return;
        }
        for draw in &self.draws {
            if Self::fade_slot(&self.tiles, draw).is_some() {
                for pass in draw.material.passes.iter().filter(|pass| pass.fades) {
                    self.pipelines.prepare(&self.device, pass.key.faded(), false);
                }
            }
        }
        if self.fade_values.len() > self.fade_capacity {
            self.fade_capacity = self.fade_values.len().next_power_of_two();
            self.fade_buffer = fade_buffer(&self.device, self.fade_capacity);
        }
        self.queue.write_buffer(&self.fade_buffer, 0, bytemuck::cast_slice(&self.fade_values));
    }

    /// `AActor::Tick` of the actors on the `AddL2AvtActors` list, after the frame drew with the alphas it left.
    fn tick_actor_views(&mut self, dt: f32) {
        for state in self.tiles.iter_mut().flat_map(|tile| tile.views.iter_mut()) {
            if state.view.animating() {
                state.view.tick(dt);
            }
        }
    }

    fn fade_slot(tiles: &[GpuTile], draw: &GpuDraw) -> Option<u32> {
        let level = draw.actor?.lod.map_or(0, |lod| usize::from(lod.level));
        let state = tiles.get(draw.tile)?.views.get(draw.view?)?;
        (state.stamp != 0).then_some(state.slots[level]).filter(|&slot| slot != NO_FADE)
    }

    fn sector_visible(&self, draw: &GpuDraw, frustum: &Frustum, eye: glam::Vec3) -> bool {
        if !self.actor_visible(draw, eye) {
            return false;
        }
        if let Some([_, max]) = draw.material.context.fade
            && (draw.center - eye).length_squared() > (draw.radius + max).powi(2)
        {
            return false;
        }
        match draw.sector {
            Some(sector) => {
                let reference = self.view_target.unwrap_or(eye);
                let location = glam::Vec3::from(sector.location);
                let distance = (location.truncate() - reference.truncate()).length();
                distance <= self.terrain_range && frustum.contains_box(glam::Vec3::from(sector.min), glam::Vec3::from(sector.max))
            }
            None => frustum.contains_sphere(draw.center, draw.radius),
        }
    }

    pub fn clear(&mut self) {
        self.door_bindings.clear();
        self.l2_event_actors.clear();
        self.clear_shadow_projectors();
        self.moving_time = self.time;
        let (effects, map): (Vec<_>, Vec<_>) = self.particle_actors.drain(..).partition(|actor| actor.effect.is_some());
        self.particle_actors = effects;
        for actor in map {
            for draw in actor.draws.into_iter().flatten() {
                for pass in draw.material.passes { self.textures.release_material(pass.slot); self.materials.free.push(pass.slot); }
            }
        }
        self.particle_time = self.time;
        self.sun_visibility = crate::light_visibility::SunVisibility::default();
        for corona in self.coronas.drain(..) {
            for pass in corona.material.passes { self.textures.release_material(pass.slot); self.materials.free.push(pass.slot); }
        }
        for draw in self.draws.drain(..) {
            for pass in draw.material.passes {
                self.textures.release_material(pass.slot); self.materials.free.push(pass.slot);
            }
        }
        for body in self.bodies.drain(..) {
            for pass in body.material.passes {
                self.textures.release_material(pass.slot); self.materials.free.push(pass.slot);
            }
        }
        for flare in self.flares.drain(..) {
            for pass in flare.material.passes {
                self.textures.release_material(pass.slot); self.materials.free.push(pass.slot);
            }
        }
        self.tiles.clear();
        self.zones.clear();
        self.lightmaps.clear();
        self.textures.collect_unused();
    }

    pub fn tiles(&self) -> impl Iterator<Item = &str> {
        self.tiles.iter().map(|tile| tile.name.as_str())
    }

    pub fn has_tile(&self, name: &str) -> bool {
        self.tiles.iter().any(|tile| tile.name == name)
    }

    pub fn set_tile_visible(&mut self, name: &str, visible: bool) {
        for tile in self.tiles.iter_mut().filter(|tile| tile.name == name) {
            tile.visible = visible;
        }
    }

    /// The line of sight to the sun is clear: `RenderLensFlare`'s `SingleLineCheck` found nothing.
    pub fn set_lens_flare_clear(&mut self, clear: bool) {
        self.flare_clear = clear;
    }

    pub fn update_sun_visibility(&mut self, viewer: glam::Vec3, direction: glam::Vec3, instances: &[CharacterInstance], clear: impl FnMut(glam::Vec3, glam::Vec3) -> bool) {
        let mut positions: Vec<_> = instances.iter().map(|instance| (instance.actor_id, instance.actor_position)).collect();
        positions.dedup();
        self.sun_visibility.update(self.time, viewer, direction, positions, clear);
    }

    pub fn update_coronas(&mut self, camera: &Camera, aspect: f32, dt: f32, mut clear: impl FnMut(glam::Vec3, glam::Vec3) -> bool) {
        let frustum = camera.frustum(aspect);
        let mut candidates = [None; crate::coronas::MAX_CORONAS];
        let mut cursor = 0;
        for (index, corona) in self.coronas.iter_mut().enumerate() {
            corona.state.update(&corona.light, self.time, dt, false, false);
            if self.tiles.get(corona.tile).is_some_and(|tile| tile.visible)
                && camera.eye.distance_squared(corona.light.position) <= corona.light.radius.powi(2)
                && frustum.contains_sphere(corona.light.position, 0.0)
            {
                candidates[cursor % crate::coronas::MAX_CORONAS] = Some(index);
                cursor += 1;
            }
        }
        let mut active = self.coronas.iter().filter(|corona| corona.state.fade > 0.0).count();
        for index in candidates.into_iter().flatten() {
            let corona = &mut self.coronas[index];
            if corona.state.needs_trace(self.time, crate::coronas::app_frand(&mut self.corona_random)) {
                corona.state.visible = clear(camera.eye, corona.light.position);
                corona.state.checked = Some(self.time);
            }
            if corona.state.visible && (corona.state.fade > 0.0 || active < crate::coronas::MAX_CORONAS) {
                if corona.state.fade == 0.0 { active += 1; }
                corona.state.fade = (corona.state.fade + 2.0 * corona.light.scale_glow * (dt * 3.0).min(1.0)).min(1.0);
            }
        }
    }

    /// The pawn's zone and position, which pick the fog of the frame (`FLevelSceneNode::Render`).
    pub fn set_flash(&mut self, scale: f32, fog: [f32; 3]) {
        let present = self.present.get_or_insert_with(|| Present::new(&self.device, self.color_format));
        present.set_flash(&self.queue, scale, fog);
    }

    pub fn set_viewer(&mut self, viewer: Viewer) {
        self.viewer = Some(viewer);
    }

    /// The fog `SetDistanceFog` received on the last frame.
    pub fn fog(&self) -> Option<&FogFrame> {
        self.last_fog.as_ref()
    }

    fn fog_frame(&mut self, environment: &Environment, eye: glam::Vec3) -> FogFrame {
        let viewer = self.viewer.unwrap_or(Viewer { zone: 0, position: eye, dt: 0.0 });
        let zone = self.zone(self.zone_base_at(viewer.position) + u32::from(viewer.zone));
        let fog_infos: Vec<L2FogInfo> = self.tiles.iter().filter(|tile| tile.visible).flat_map(|tile| tile.fog_infos.iter().cloned()).collect();
        let sky = environment.sky;
        let input = FogInput {
            zone,
            hour: environment.hour,
            hazering: sky.hazering,
            sky: sky.sky_box,
            clouds: sky.clouds.to_vec(),
            haze: vec![sky.hazering],
            view: viewer.position,
            dt: viewer.dt,
            global: self.global_fog,
            fog_level: self.fog_level,
            fog_infos: &fog_infos,
        };
        self.fog.update(&input)
    }

    pub fn remove_tile(&mut self, name: &str) {
        let Some(index) = self.tiles.iter().position(|tile| tile.name == name) else { return };
        self.remove_particle_tile(index);
        let mut coronas = Vec::new();
        for mut corona in self.coronas.drain(..) {
            if corona.tile == index {
                for pass in corona.material.passes { self.textures.release_material(pass.slot); self.materials.free.push(pass.slot); }
            } else {
                if corona.tile > index {
                    corona.tile -= 1;
                    corona.material.context.zone -= self.tiles[index].zone_count as u32;
                }
                coronas.push(corona);
            }
        }
        self.coronas = coronas;
        let mut kept = Vec::with_capacity(self.draws.len());
        for draw in self.draws.drain(..) {
            if draw.tile == index {
                for pass in draw.material.passes {
                    self.textures.release_material(pass.slot); self.materials.free.push(pass.slot);
                }
            } else {
                kept.push(draw);
            }
        }
        self.draws = kept;
        let mut bodies = Vec::new();
        for body in self.bodies.drain(..) {
            if body.tile == index {
                for pass in body.material.passes {
                    self.textures.release_material(pass.slot); self.materials.free.push(pass.slot);
                }
            } else {
                bodies.push(body);
            }
        }
        self.bodies = bodies;
        let mut flares = Vec::new();
        for flare in self.flares.drain(..) {
            if flare.tile == index {
                for pass in flare.material.passes {
                    self.textures.release_material(pass.slot); self.materials.free.push(pass.slot);
                }
            } else {
                flares.push(flare);
            }
        }
        self.flares = flares;
        let removed = self.tiles.remove(index);
        let zone_range = removed.zone_base as usize..removed.zone_base as usize + removed.zone_count;
        let lightmap_range = removed.lightmap_base..removed.lightmap_base + removed.lightmap_count;
        self.zones.drain(zone_range);
        self.lightmaps.drain(lightmap_range);
        for tile in self.tiles.iter_mut().skip(index) {
            tile.zone_base -= removed.zone_count as u32;
            tile.lightmap_base -= removed.lightmap_count;
        }
        for draw in &mut self.draws {
            if draw.tile > index {
                draw.tile -= 1;
                draw.zone -= removed.zone_count as u32;
                draw.material.context.zone = draw.zone;
            }
        }
        for body in &mut self.bodies {
            if body.tile > index {
                body.tile -= 1;
                body.zone -= removed.zone_count as u32;
                body.material.context.zone = body.zone;
            }
        }
        for flare in &mut self.flares {
            if flare.tile > index {
                flare.tile -= 1;
                flare.material.context.zone -= removed.zone_count as u32;
            }
        }
        self.zone_buffer = zone_buffer(&self.device, &self.zones, 0);
        self.uniform_slots.get_mut().groups.clear();
        self.refresh_materials();
    }

    /// Replaces the lightmap array of one group of a loaded tile (the slices `FLightMap::GetTextureData` regenerated) and rebinds the draws that sample it.
    pub fn replace_lightmap_group(&mut self, name: &str, group: usize, data: &killer_assets::scene::LightmapGroup) {
        let Some(tile) = self.tiles.iter().position(|tile| tile.name == name) else { return };
        if group >= self.tiles[tile].lightmap_count {
            return;
        }
        let slot = self.tiles[tile].lightmap_base + group;
        let layers: Vec<Arc<TextureData>> = data.slices.iter().map(|mips| Arc::new(TextureData { width: data.width, height: data.height, format: data.format, mips: mips.clone() })).collect();
        self.lightmap_revision += 1;
        let key = format!("{name}#lightmap{group}#revision{}", self.lightmap_revision);
        let view = self.textures.data_array_view(&self.device, &self.queue, &key, &layers);
        let previous = std::mem::replace(&mut self.lightmaps[slot], (view.clone(), layers.len() as u32)).0;
        for draw in self.draws.iter_mut().filter(|draw| draw.tile == tile) {
            for pass in &mut draw.material.passes {
                if Arc::ptr_eq(&pass.views.lightmap, &previous) {
                    pass.views.lightmap = view.clone();
                    pass.bind_group = bind_material(&self.device, &self.pipelines.material_layout, &self.samplers, &self.materials.buffer, pass.slot, &pass.views);
                }
            }
        }
    }

    /// Swaps the draws of one kind in a loaded tile, keeping its zones and lightmaps (the terrain after a neighbour stitches to it).
    pub fn replace_draws(&mut self, name: &str, kind: DrawKind, draws: &[MeshDraw], textures: &mut dyn TextureSource) {
        self.replace_draws_matching(name, |kind_value, _| kind_value == kind, draws, textures);
    }

    pub fn replace_decorations(&mut self, name: &str, draws: &[MeshDraw], textures: &mut dyn TextureSource) {
        self.replace_draws_matching(name, |kind, fade| kind == DrawKind::Terrain && fade, draws, textures);
    }

    fn replace_draws_matching(&mut self, name: &str, matches: impl Fn(DrawKind, bool) -> bool, draws: &[MeshDraw], textures: &mut dyn TextureSource) {
        let Some(index) = self.tiles.iter().position(|tile| tile.name == name) else { return };
        let mut kept = Vec::with_capacity(self.draws.len());
        for draw in self.draws.drain(..) {
            if draw.tile == index && matches(draw.material.context.kind, draw.material.context.fade.is_some()) {
                for pass in draw.material.passes {
                    self.textures.release_material(pass.slot); self.materials.free.push(pass.slot);
                }
            } else {
                kept.push(draw);
            }
        }
        self.draws = kept;
        self.reserve_materials(draws.iter().map(|draw| draw.material.passes.len() as u32).sum());
        for draw in draws.iter().filter(|draw| matches(draw.kind, draw.fade.is_some())) {
            if let Some(gpu) = self.gpu_draw(draw, textures, index) {
                self.draws.push(gpu);
            }
        }
    }

    fn refresh_materials(&mut self) {
        let time = self.time;
        let refresh = |device: &wgpu::Device, queue: &wgpu::Queue, layout: &wgpu::BindGroupLayout, materials: &MaterialSlots, samplers: &[wgpu::Sampler; 2], material: &mut GpuMaterial| {
            for pass in &mut material.passes {
                let gpu = MaterialGpu::new(&material.material, &material.material.passes[pass.pass], &material.context, time);
                materials.write(queue, pass.slot, &gpu);
                pass.bind_group = bind_material(device, layout, samplers, &materials.buffer, pass.slot, &pass.views);
            }
        };
        for draw in &mut self.draws {
            refresh(&self.device, &self.queue, &self.pipelines.material_layout, &self.materials, &self.samplers, &mut draw.material);
        }
        for body in &mut self.bodies {
            refresh(&self.device, &self.queue, &self.pipelines.material_layout, &self.materials, &self.samplers, &mut body.material);
        }
        for flare in &mut self.flares {
            refresh(&self.device, &self.queue, &self.pipelines.material_layout, &self.materials, &self.samplers, &mut flare.material);
        }
        for corona in &mut self.coronas {
            refresh(&self.device, &self.queue, &self.pipelines.material_layout, &self.materials, &self.samplers, &mut corona.material);
        }
        for draw in self.particle_actors.iter_mut().flat_map(|a| a.draws.iter_mut().flatten()) {
            refresh(&self.device, &self.queue, &self.pipelines.material_layout, &self.materials, &self.samplers, &mut draw.material);
        }
        for character in self.characters.iter_mut().flatten() {
            for section in &mut character.sections {
                refresh(&self.device, &self.queue, &self.pipelines.material_layout, &self.materials, &self.samplers, &mut section.material);
            }
        }
    }

    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }

    fn reserve_materials(&mut self, wanted: u32) {
        if self.materials.available() >= wanted {
            return;
        }
        self.materials.grow(&self.device, wanted);
        self.refresh_materials();
    }

    pub fn upload(&mut self, scene: &Scene, textures: &mut dyn TextureSource) {
        self.clear();
        self.add_tile("", scene, textures);
    }

    pub fn upload_with_texture_detail(&mut self, scene: &Scene, textures: &mut dyn TextureSource, detail: i32) {
        self.clear();
        self.reload_texture_detail(detail, textures);
        self.add_tile("", scene, textures);
    }

    pub fn add_tile(&mut self, name: &str, scene: &Scene, textures: &mut dyn TextureSource) {
        self.remove_tile(name);
        self.reserve_materials(scene.draws.iter().map(|draw| draw.material.passes.len() as u32).sum());
        self.terrain_range = scene.terrain_range;
        let highest_zone = scene.draws.iter().map(|draw| usize::from(draw.zone)).max().unwrap_or(0);
        let zone_count = scene.zones.len().max(highest_zone + 1);
        let zone_base = self.zones.len() as u32;
        self.zones.extend((0..zone_count).map(|id| scene.zones.get(id).copied().unwrap_or_default()));
        self.zone_buffer = zone_buffer(&self.device, &self.zones, 0);
        self.uniform_slots.get_mut().groups.clear();
        let lightmap_base = self.lightmaps.len();
        for (index, group) in scene.lightmaps.iter().enumerate() {
            let layers: Vec<Arc<TextureData>> = group
                .slices
                .iter()
                .map(|mips| Arc::new(TextureData { width: group.width, height: group.height, format: group.format, mips: mips.clone() }))
                .collect();
            let view = self.textures.data_array_view(&self.device, &self.queue, &format!("{name}#lightmap{index}"), &layers);
            self.lightmaps.push((view, layers.len() as u32));
        }
        let tile = self.tiles.len();
        self.global_fog = scene.global_fog;
        self.fog_level = scene.global_fog.level;
        self.tiles.push(GpuTile { projectors: scene.projectors.clone(), visible_projectors: Vec::new(), moving: Vec::new(), name: name.to_owned(), zone_base, zone_count, lightmap_base, lightmap_count: scene.lightmaps.len(), visible: true, fog_infos: scene.fog_infos.clone(), views: Vec::new(), view_ids: rustc_hash::FxHashMap::default() });
        self.update_projector_visibility();
        let mut projector_indices = vec![None; scene.draws.len()];
        for (projector, state) in scene.projectors.iter().enumerate() {
            for &draw in &state.draws {
                if let Some(index) = projector_indices.get_mut(draw) { *index = Some(projector); }
            }
        }
        let geometries = self.shared_geometry(&scene.draws);
        for ((draw, projector), geometry) in scene.draws.iter().zip(projector_indices).zip(geometries) {
            if let Some(mut gpu) = self.gpu_draw_geometry(draw, textures, tile, geometry) {
                gpu.projector = projector;
                self.draws.push(gpu);
            }
        }
        let sky_zone = scene.sky_bodies.first().map_or(0, |body| zone_base + u32::from(body.zone));
        for light in &scene.coronas {
            let context = DrawContext { ambient: Default::default(), kind: DrawKind::Sprite, sun_affect: false, zone: zone_base, has_mask: false, lightmap_slices: 0, modulate_2x: false, fade: None, tint: None };
            let Some(material) = self.gpu_material(&light.material, context, None, None, textures, false) else { continue };
            let (vertices, indices) = self.quad_buffers();
            self.coronas.push(GpuCorona { light: light.clone(), state: crate::coronas::CoronaState::default(), vertices, indices, material, tile });
        }
        for body in &scene.sky_bodies {
            let zone = zone_base + u32::from(body.zone);
            let context = DrawContext { ambient: Default::default(), kind: DrawKind::Sprite, sun_affect: false, zone, has_mask: false, lightmap_slices: 0, modulate_2x: false, fade: None, tint: None };
            let Some(material) = self.gpu_material(&body.material, context, None, None, textures, false) else { continue };
            let (vertices, indices) = self.quad_buffers();
            self.bodies.push(GpuBody { body: body.clone(), vertices, indices, material, zone, tile });
        }
        for flare in &scene.flares {
            let context = DrawContext { ambient: Default::default(), kind: DrawKind::Sprite, sun_affect: false, zone: sky_zone, has_mask: false, lightmap_slices: 0, modulate_2x: false, fade: None, tint: None };
            let Some(material) = self.gpu_material(&flare_material(&flare.texture), context, None, None, textures, false) else { continue };
            let (vertices, indices) = self.quad_buffers();
            self.flares.push(GpuFlare { flare: flare.clone(), vertices, indices, material, tile });
        }
        self.upload_particles(&scene.emitters, textures, tile);
        self.upload_movers(&scene.moving, textures, tile);
    }

    fn quad_buffers(&self) -> (wgpu::Buffer, wgpu::Buffer) {
        let vertices = self.device.create_buffer(&wgpu::BufferDescriptor { label: Some("sprite"), size: (size_of::<killer_assets::scene::Vertex>() * 4) as u64, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let indices = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("sprite"), contents: bytemuck::cast_slice(&QUAD_INDICES), usage: wgpu::BufferUsages::INDEX });
        (vertices, indices)
    }

    fn tint_color(&self, tint: Option<SkyTint>) -> Option<[f32; 3]> {
        match tint? {
            SkyTint::SkyBox => Some(self.sky.sky_box),
            SkyTint::Hazering => Some(self.sky.hazering),
            SkyTint::Cloud(index) => self.sky.clouds.get(index).copied(),
        }
    }

    /// O espaço em que a interface é desenhada. O original não tem fator de escala: ele desenha a
    /// interface em pixels fixos na resolução escolhida. Numa tela com fator 2 o quadro tem o dobro
    /// dos pixels, então a interface é montada no espaço do cliente e a GPU a amplia até o quadro —
    /// mesmo tamanho de tela do original, com o mundo em 3D na resolução cheia. `None` = igual ao
    /// quadro.
    pub fn set_lines(&mut self, lines: &[crate::lines::Line]) {
        self.lines.upload(&self.device, &self.queue, lines);
    }

    pub fn set_ui_size(&mut self, size: Option<(u32, u32)>) {
        self.ui_size = size;
    }

    pub fn set_ui_external_texture(&mut self, name: &str, view: SharedView, size: [f32; 2]) {
        self.ui.set_external_texture(name, view, size);
    }

    pub fn clear_ui_external_texture(&mut self, name: &str) {
        self.ui.clear_external_texture(name);
    }

    pub fn set_ui(&mut self, list: &DrawList, textures: &mut dyn TextureSource) {
        self.ui.upload(&self.device, &self.queue, list, &mut self.textures, textures);
    }

    pub fn update_world_text(&mut self, mesh: &crate::ui::UiMesh, texture: &wgpu::TextureView) {
        self.world_text.upload_mesh(&self.device,&self.queue,mesh,texture);
    }

    pub fn clear_world_text(&mut self) {
        self.world_text.clear();
    }

    pub fn clear_ui(&mut self) {
        self.ui.clear();
        self.world_text.clear();
    }

    fn gpu_material(&mut self, material: &Material, context: DrawContext, mask: Option<&str>, lightmap: Option<usize>, textures: &mut dyn TextureSource, skinned: bool) -> Option<GpuMaterial> {
        self.reserve_materials(material.passes.len() as u32);
        let mut passes = Vec::with_capacity(material.passes.len());
        for (index, pass) in material.passes.iter().enumerate() {
            let slot = self.materials.take()?;
            self.textures.release_material(slot);
            let gpu = MaterialGpu::at(material, pass, &context, self.time, self.tint_color(context.tint));
            self.materials.write(&self.queue, slot, &gpu);
            let mut key = PipelineKey::of(pass, context.kind);
            key.bsp_projector = material.root_class == "projectormaterial";
            self.pipelines.prepare(&self.device, key, skinned);
            let views = self.pass_views(material, index, slot, mask, lightmap, textures);
            let bind_group = bind_material(&self.device, &self.pipelines.material_layout, &self.samplers, &self.materials.buffer, slot, &views);
            let animated = pass.animated() || context.tint.is_some() || context.ambient.glow == 255 || material.pass_stages(pass).iter().any(|stage| stage.animated());
            passes.push(GpuPass { key, bind_group, views, slot, animated, pass: index, fades: crate::pipeline::modifier_class(pass) & 3 != 0 });
        }
        Some(GpuMaterial { material: material.clone(), context, passes })
    }

    fn gpu_draw_geometry(&mut self, draw: &MeshDraw, textures: &mut dyn TextureSource, tile: usize, geometry: Geometry) -> Option<GpuDraw> {
        let lightmap = draw.lightmap.map(|group| self.tiles[tile].lightmap_base + group);
        let zone = self.tiles[tile].zone_base + u32::from(draw.zone);
        self.gpu_draw_with(draw, textures, tile, zone, lightmap, geometry)
    }

    fn gpu_draw(&mut self, draw: &MeshDraw, textures: &mut dyn TextureSource, tile: usize) -> Option<GpuDraw> {
        let lightmap = draw.lightmap.map(|group| self.tiles[tile].lightmap_base + group);
        let zone = self.tiles[tile].zone_base + u32::from(draw.zone);
        self.gpu_draw_at(draw, textures, tile, zone, lightmap)
    }

    fn own_geometry(&self, draw: &MeshDraw) -> Geometry {
        let vertices = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&draw.vertices), usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST });
        let indices = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&draw.indices), usage: wgpu::BufferUsages::INDEX });
        Geometry { vertices, indices, base_vertex: 0, first_index: 0 }
    }

    fn shared_geometry(&self, draws: &[MeshDraw]) -> Vec<Geometry> {
        let mut output: Vec<Option<Geometry>> = (0..draws.len()).map(|_| None).collect();
        let mut start = 0;
        while start < draws.len() {
            let mut end = start;
            let (mut vertex_bytes, mut index_bytes) = (0u64, 0u64);
            while end < draws.len() {
                let vertices = std::mem::size_of_val(draws[end].vertices.as_slice()) as u64;
                let indices = std::mem::size_of_val(draws[end].indices.as_slice()) as u64;
                if end > start && (vertex_bytes + vertices > SHARED_VERTEX_BYTES || index_bytes + indices > SHARED_INDEX_BYTES) {
                    break;
                }
                vertex_bytes += vertices;
                index_bytes += indices;
                end += 1;
            }
            let group = &draws[start..end];
            if vertex_bytes == 0 || index_bytes == 0 {
                for (slot, draw) in output[start..end].iter_mut().zip(group) {
                    *slot = Some(self.own_geometry(draw));
                }
                start = end;
                continue;
            }
            let vertices = self.device.create_buffer(&wgpu::BufferDescriptor { label: Some("malhas do mapa"), size: vertex_bytes, usage: wgpu::BufferUsages::VERTEX, mapped_at_creation: true });
            let indices = self.device.create_buffer(&wgpu::BufferDescriptor { label: Some("índices do mapa"), size: index_bytes, usage: wgpu::BufferUsages::INDEX, mapped_at_creation: true });
            {
                let (Ok(mut vertex_view), Ok(mut index_view)) = (vertices.slice(..).get_mapped_range_mut(), indices.slice(..).get_mapped_range_mut()) else {
                    for (slot, draw) in output[start..end].iter_mut().zip(group) {
                        *slot = Some(self.own_geometry(draw));
                    }
                    start = end;
                    continue;
                };
                let (mut vertex_at, mut index_at) = (0usize, 0usize);
                let (mut base_vertex, mut first_index) = (0i32, 0u32);
                for (slot, draw) in output[start..end].iter_mut().zip(group) {
                    let vertex_data: &[u8] = bytemuck::cast_slice(&draw.vertices);
                    let index_data: &[u8] = bytemuck::cast_slice(&draw.indices);
                    vertex_view.slice(vertex_at..vertex_at + vertex_data.len()).copy_from_slice(vertex_data);
                    index_view.slice(index_at..index_at + index_data.len()).copy_from_slice(index_data);
                    *slot = Some(Geometry { vertices: vertices.clone(), indices: indices.clone(), base_vertex, first_index });
                    vertex_at += vertex_data.len();
                    index_at += index_data.len();
                    base_vertex += draw.vertices.len() as i32;
                    first_index += draw.indices.len() as u32;
                }
            }
            vertices.unmap();
            indices.unmap();
            start = end;
        }
        output.into_iter().flatten().collect()
    }

    fn gpu_draw_at(&mut self, draw: &MeshDraw, textures: &mut dyn TextureSource, tile: usize, zone: u32, lightmap: Option<usize>) -> Option<GpuDraw> {
        let geometry = self.own_geometry(draw);
        self.gpu_draw_with(draw, textures, tile, zone, lightmap, geometry)
    }

    fn gpu_draw_with(&mut self, draw: &MeshDraw, textures: &mut dyn TextureSource, tile: usize, zone: u32, lightmap: Option<usize>, geometry: Geometry) -> Option<GpuDraw> {
        let Geometry { vertices, indices, base_vertex, first_index } = geometry;
        let lightmap_slices = lightmap.and_then(|group| self.lightmaps.get(group)).map_or(0, |(_, slices)| *slices);
        let context = DrawContext { ambient: draw.actor.map(|a| a.ambient).unwrap_or_default(), kind: draw.kind, sun_affect: draw.sun_affect, zone, has_mask: draw.mask.is_some(), lightmap_slices, modulate_2x: draw.modulate_2x, fade: draw.fade, tint: draw.tint };
        let mut material = self.gpu_material(&draw.material, context, draw.mask.as_deref(), lightmap, textures, false)?;
        let instances = (!draw.instances.is_empty()).then(|| {
            for pass in &mut material.passes {
                pass.key.instanced = true;
                self.pipelines.prepare(&self.device, pass.key, false);
            }
            self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("decorações"), contents: bytemuck::cast_slice(&draw.instances), usage: wgpu::BufferUsages::VERTEX })
        });
        let sum = draw.world_vertices().fold(glam::Vec3::ZERO, |acc, v| acc + glam::Vec3::from(v.position));
        let center = sum / draw.world_vertex_count().max(1) as f32;
        let radius = draw.world_vertices().map(|v| glam::Vec3::from(v.position).distance(center)).fold(0.0, f32::max);
        let view = draw.actor.filter(|_| draw.kind == DrawKind::StaticMesh).map(|cull| {
            let GpuTile { views, view_ids, .. } = &mut self.tiles[tile];
            *view_ids.entry(cull.id).or_insert_with(|| {
                views.push(ActorState::default());
                views.len() - 1
            })
        });
        Some(GpuDraw { instances, instance_count: draw.instances.len().max(1) as u32, projector: None, source: 0, moving: None, spawned: None, vertices, indices, base_vertex, first_index, index_count: draw.indices.len() as u32, material, pass: draw.pass, sort_level: draw.material.sorting_level, center, radius, zone, sector: draw.sector, actor: draw.actor, view, tile })
    }

    pub fn upload_character(&mut self, mesh: &CharacterMesh, textures: &mut dyn TextureSource) -> CharacterId {
        let character = self.gpu_character(mesh, textures);
        match self.characters.iter().position(Option::is_none) {
            Some(slot) => {
                self.characters[slot] = Some(character);
                CharacterId(slot)
            }
            None => {
                self.characters.push(Some(character));
                CharacterId(self.characters.len() - 1)
            }
        }
    }

    fn gpu_character(&mut self, mesh: &CharacterMesh, textures: &mut dyn TextureSource) -> GpuCharacter {
        let vertices: Vec<SkinnedVertex> = mesh.vertices.iter().map(SkinnedVertex::from).collect();
        let vertices = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("personagem"), contents: bytemuck::cast_slice(&vertices), usage: wgpu::BufferUsages::VERTEX });
        let indices = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("personagem"), contents: bytemuck::cast_slice(&mesh.indices), usage: wgpu::BufferUsages::INDEX });
        let mut sections = Vec::with_capacity(mesh.sections.len());
        for section in valid_sections(mesh) {
            let context = DrawContext { ambient: Default::default(), kind: DrawKind::Actor, sun_affect: true, zone: 0, has_mask: false, lightmap_slices: 0, modulate_2x: true, fade: None, tint: None };
            let Some(material) = self.gpu_material(&section.material, context, None, None, textures, true) else { continue };
            sections.push(GpuSection { first_index: section.first_index, index_count: section.index_count, material, translucent: section.pass == Pass::Translucent });
        }
        GpuCharacter { vertices, indices, sections, radius: mesh.bounding_sphere[3] }
    }

    pub fn remove_character(&mut self, id: CharacterId) {
        if let Some(slot) = self.characters.get_mut(id.0)
            && let Some(character) = slot.take()
        {
            for section in character.sections {
                for pass in section.material.passes {
                    self.textures.release_material(pass.slot); self.materials.free.push(pass.slot);
                }
            }
        }
        self.instances.retain(|instance| instance.character != id.0);
    }

    pub fn set_instances(&mut self, instances: &[CharacterInstance]) {
        let stride = self.object_buffers.stride;
        let mut objects = std::mem::take(&mut self.object_bytes);
        let mut palette = std::mem::take(&mut self.palette_matrices);
        objects.clear();
        palette.clear();
        self.instances.clear();
        for instance in instances {
            let known = self.characters.get(instance.mesh.0).is_some_and(Option::is_some);
            if !known || instance.palette.is_empty() {
                continue;
            }
            let center = instance.world.w_axis.truncate();
            let zone = self.zone_base_at(center) + u32::from(instance.zone);
            let sun_affect = instance.sun_affect;
            let sun_visible = sun_affect && self.sun_visibility.visible(instance.actor_id);
            let limit = 4usize.saturating_sub(usize::from(sun_visible));
            let level = &instance.lights[..instance.lights.len().min(limit)];
            let first = level.len() + usize::from(sun_visible);
            let lights: Vec<PointLight> = level.iter().chain(instance.pawn_lights.iter().take(MAX_HARDWARE_LIGHTS.saturating_sub(first))).copied().collect();
            let mut uniform = ObjectUniform::new(instance.world, palette.len() as u32, instance.palette.len() as u32, zone, sun_affect, &lights);
            uniform.sun_visible = u32::from(sun_visible);
            uniform.ambient = [f32::from(instance.ambient[0]) / 255.0, f32::from(instance.ambient[1]) / 255.0, f32::from(instance.ambient[2]) / 255.0, 0.0];
            uniform.overlay = instance.overlay;
            uniform.set_light_sphere(instance.light_sphere, &lights);
            let offset = objects.len();
            objects.extend_from_slice(bytemuck::bytes_of(&uniform));
            objects.resize(offset + stride as usize, 0);
            palette.extend(instance.palette.iter().map(Mat4::to_cols_array_2d));
            let radius = self.characters[instance.mesh.0].as_ref().map_or(0.0, |character| {
                character.radius * instance.world.x_axis.truncate().length().max(instance.world.y_axis.truncate().length()).max(instance.world.z_axis.truncate().length())
            });
            self.instances.push(DrawInstance {
                character: instance.mesh.0,
                offset: offset as u32,
                center: instance.world.w_axis.truncate(),
                actor_id: instance.actor_id,
                position: instance.actor_position,
                collision_height: instance.collision_height,
                shadow_caster: instance.shadow_caster,
                radius,
            });
        }
        if !self.instances.is_empty()
            && (objects.len() as u64 > self.object_buffers.objects.size() || palette.len() as u64 * 64 > self.object_buffers.palette.size())
        {
            let slots = (objects.len() as u64 / stride).next_power_of_two().max(self.object_buffers.objects.size() / stride);
            let matrices = (palette.len() as u64).next_power_of_two().max(self.object_buffers.palette.size() / 64);
            self.object_buffers = object_buffers(&self.device, &self.pipelines.object_layout, stride, slots, matrices);
        }
        self.object_bytes = objects;
        self.palette_matrices = palette;
    }

    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    pub fn shadow_count(&self) -> usize {
        self.shadow_targets.entries.iter().filter(|target| target.owner.is_some()).count()
    }

    pub fn visible_draw_count(&self, camera: &Camera, aspect: f32) -> usize {
        self.visible_stats(camera, aspect).0
    }

    /// Draws and triangles of the world that this frame would issue.
    pub fn visible_stats(&self, camera: &Camera, aspect: f32) -> (usize, usize) {
        let frustum = camera.frustum(aspect);
        self.draws
            .iter()
            .filter(|draw| draw.index_count != 0 && self.tile_visible(draw) && !self.zone(draw.zone).is_sky && self.sector_visible(draw, &frustum, camera.eye))
            .fold((0, 0), |acc, draw| (acc.0 + 1, acc.1 + draw.index_count as usize / 3))
    }

    fn upload_instances(&self) {
        if self.instances.is_empty() {
            return;
        }
        self.queue.write_buffer(&self.object_buffers.objects, 0, &self.object_bytes);
        self.queue.write_buffer(&self.object_buffers.palette, 0, bytemuck::cast_slice(&self.palette_matrices));
    }

    fn animate_materials(&mut self) {
        let mut upload = std::mem::take(&mut self.material_upload);
        let mut emit = |offset, bytes: &[u8]| self.queue.write_buffer(&self.materials.buffer, offset, bytes);
        let time = self.time;
        let mut refresh = |material: &GpuMaterial| {
            for pass in material.passes.iter().filter(|pass| pass.animated) {
                let gpu = MaterialGpu::at(&material.material, &material.material.passes[pass.pass], &material.context, time, self.tint_color(material.context.tint));
                upload.push(pass.slot, &gpu, &mut emit);
            }
        };
        for draw in &self.draws {
            refresh(&draw.material);
        }
        let mut refreshed=vec![false;self.characters.len()];
        for instance in &self.instances {
            let Some(seen)=refreshed.get_mut(instance.character) else {continue};
            if *seen {continue;}
            *seen=true;
            let Some(character)=self.characters[instance.character].as_ref() else {continue};
            for section in &character.sections {
                refresh(&section.material);
            }
        }
        upload.flush(&mut emit);
        self.material_upload = upload;
    }

    fn pass_views(&mut self, material: &Material, pass: usize, slot: u32, mask: Option<&str>, lightmap: Option<usize>, textures: &mut dyn TextureSource) -> PassViews {
        let stages = material.pass_stages(&material.passes[pass]);
        let mut views = Vec::with_capacity(MAX_STAGES);
        let mut clamps = Vec::with_capacity(MAX_STAGES);
        let mut cube = None;
        let mask = mask.map(|path| self.textures.alpha_map_view(&self.device, &self.queue, textures, path));
        for stage in stages.iter().take(MAX_STAGES) {
            match &stage.source {
                StageSource::Texture(texture) => {
                    let mut paths = Vec::with_capacity(texture.frames.len() + 1);
                    paths.push(texture.path.clone());
                    if texture.animated() {
                        paths.extend(texture.frames.iter().cloned());
                    }
                    views.push(self.textures.material_array_view(&self.device, &self.queue, textures, &paths, slot));
                    clamps.push(texture.clamp[0] || texture.clamp[1]);
                }
                StageSource::Cubemap(cubemap) => {
                    cube = Some(self.textures.cube_view(&self.device, &self.queue, textures, &cubemap.faces));
                    views.push(self.textures.white_array());
                    clamps.push(false);
                }
                StageSource::Factor | StageSource::Diffuse => {
                    views.push(self.textures.white_array());
                    clamps.push(false);
                }
                StageSource::ProjectorFacing => {
                    cube = Some(self.textures.projector_facing(&self.device, &self.queue));
                    views.push(self.textures.white_array());
                    clamps.push(true);
                }
            }
        }
        while views.len() < MAX_STAGES {
            views.push(self.textures.white_array());
            clamps.push(false);
        }
        let lightmap = lightmap.and_then(|group| self.lightmaps.get(group)).map_or_else(|| self.textures.white_array(), |(view, _)| view.clone());
        PassViews { textures: views, clamps, cube: cube.unwrap_or_else(|| self.textures.white_cube()), mask: mask.unwrap_or_else(|| self.textures.white()), lightmap }
    }

    fn zone(&self, id: u32) -> ZoneLighting {
        self.zones.get(id as usize).copied().unwrap_or_default()
    }

    /// `GetCurrentSWLevel`: the tile that contains the point when it is loaded, else the main one.
    fn zone_base_at(&self, position: glam::Vec3) -> u32 {
        let name = tile_name(position.x, position.y);
        self.tiles.iter().find(|tile| tile.name == name).or(self.tiles.first()).map_or(0, |tile| tile.zone_base)
    }

    fn tile_visible(&self, draw: &GpuDraw) -> bool {
        (self.projectors || draw.pass != Pass::Projector) && self.tiles.get(draw.tile).is_none_or(|tile| {
            tile.visible && draw.projector.is_none_or(|index| tile.visible_projectors.get(index).copied().unwrap_or(false))
        })
    }

    fn slot_group(&self, uniform: &CameraUniform) -> wgpu::BindGroup {
        let mut slots = self.uniform_slots.borrow_mut();
        if slots.next == slots.groups.len() {
            let buffer = self.device.create_buffer(&wgpu::BufferDescriptor { label: Some("câmera"), size: size_of::<CameraUniform>() as u64, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
            let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("grupo da câmera"),
                layout: &self.pipelines.group0_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: self.env_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: self.zone_buffer.as_entire_binding() },
                ],
            });
            slots.groups.push((buffer, group));
        }
        let (buffer, group) = &slots.groups[slots.next];
        self.queue.write_buffer(buffer, 0, bytemuck::bytes_of(uniform));
        let group = group.clone();
        slots.next += 1;
        group
    }

    fn group0(&self, camera: &Camera, aspect: f32) -> wgpu::BindGroup {
        self.slot_group(&CameraUniform { view_proj: camera.view_proj(aspect).to_cols_array_2d(), view: camera.view().to_cols_array_2d(), eye: camera.eye.extend(1.0).to_array() })
    }

    fn shadow_group0(&self, projection: &crate::shadow::Projection) -> wgpu::BindGroup {
        self.slot_group(&CameraUniform { view_proj: projection.view_proj.to_cols_array_2d(), view: Mat4::IDENTITY.to_cols_array_2d(), eye: projection.camera.extend(1.0).to_array() })
    }

    fn shadow_material(&mut self, target: usize, projection: &crate::shadow::Projection) -> Option<GpuMaterial> {
        let mut material = texture_material("");
        material.root_class = "projectormaterial".to_owned();
        material.transparent = true;
        material.stages[0].coord_source = CoordSource::Projection { matrix: projection.texture_matrix.to_cols_array_2d(), divide_z: true };
        material.stages[0].color_op = ColorOp::SelectTexture;
        material.stages[0].alpha_op = AlphaOp::Texture;
        {
            let pass = material.passes.first_mut()?;
            pass.src = BlendFactor::SrcAlpha;
            pass.dst = BlendFactor::InvSrcAlpha;
            pass.alpha_blend = true;
            pass.z_write = false;
            pass.z_test = true;
            pass.fog = FogMode::Black;
            pass.lit = false;
            pass.light_after = 1;
        }
        let pass = material.passes.first()?;
        self.reserve_materials(1);
        let slot = self.materials.take()?;
        let gpu = MaterialGpu::new(&material, pass, &DrawContext { ambient: Default::default(), kind: DrawKind::Projector, sun_affect: false, zone: 0, has_mask: false, lightmap_slices: 0, modulate_2x: false, fade: None, tint: None }, self.time);
        self.materials.write(&self.queue, slot, &gpu);
        let key = PipelineKey::of(pass, DrawKind::Projector);
        self.pipelines.prepare(&self.device, key, false);
        self.pipelines.prepare(&self.device, PipelineKey { instanced: true, ..key }, false);
        let target_view = Arc::new(self.shadow_targets.entries[target].blurred_array_view.clone());
        let views = PassViews {
            textures: std::iter::once(target_view).chain((1..MAX_STAGES).map(|_| self.textures.white_array())).collect(),
            clamps: vec![true; MAX_STAGES],
            cube: self.textures.white_cube(),
            mask: self.textures.white(),
            lightmap: self.textures.white_array(),
        };
        let bind_group = bind_material(&self.device, &self.pipelines.material_layout, &self.samplers, &self.materials.buffer, slot, &views);
        Some(GpuMaterial { material, context: DrawContext { ambient: Default::default(), kind: DrawKind::Projector, sun_affect: false, zone: 0, has_mask: false, lightmap_slices: 0, modulate_2x: false, fade: None, tint: None }, passes: vec![GpuPass { key, bind_group, views, slot, animated: false, pass: 0, fades: false }] })
    }

    fn clear_shadow_projectors(&mut self) {
        for projector in self.shadow_projectors.drain(..) {
            for pass in projector.material.passes {
                self.textures.release_material(pass.slot); self.materials.free.push(pass.slot);
            }
        }
    }

    fn render_shadows(&mut self, encoder: &mut wgpu::CommandEncoder, environment: &Environment) {
        if !self.pawn_shadow || self.keep_min_frame_rate {
            return;
        }
        let Some(viewer) = self.viewer else { return };
        self.clear_shadow_projectors();
        self.shadow_targets.begin_frame();
        let mut seen = HashSet::new();
        let candidates: Vec<_> = self.instances.iter().filter(|instance| instance.shadow_caster && seen.insert(instance.actor_id)).copied().collect();
        for instance in candidates {
            if crate::shadow::hidden(false, true, instance.position.to_array(), None, Some(crate::shadow::Viewer { controller: viewer.position.to_array(), pawn: None }), 0.2) {
                continue;
            }
            let mut light = crate::shadow::LightInfo { blob: false, direction: [0.0, 0.0, 10.0], distance: 1500.0, max_trace_distance: 1500 };
            light.update(crate::shadow::LightContext {
                default_shadow: 0,
                has_viewport: true,
                pawn: Some(viewer.position.to_array()),
                projector: instance.position.to_array(),
                owner: None,
                clipping_range: 0.2,
                sun: Some(crate::shadow::Sun { hour: f64::from(environment.hour), direction: environment.sun_dir.to_array() }),
                has_actor: true,
                collision_height: instance.collision_height,
            });
            let projection = crate::shadow::projection(instance.position, instance.radius, glam::Vec3::from(light.direction), light.distance);
            let Some(target_index) = self.shadow_targets.acquire(instance.actor_id) else { break };
            let target = &self.shadow_targets.entries[target_index];
            let group0 = self.shadow_group0(&projection);
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("silhueta da sombra"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &target.color_view, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store }, depth_slice: None })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { view: &target.depth_view, depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }), stencil_ops: None }),
                ..Default::default()
            });
            pass.set_pipeline(&self.pipelines.shadow);
            pass.set_bind_group(0, &group0, &[]);
            pass.set_bind_group(1, &self.object_buffers.bind_group, &[instance.offset]);
            let Some(character) = self.characters.get(instance.character).and_then(Option::as_ref) else { continue };
            pass.set_vertex_buffer(0, character.vertices.slice(..));
            pass.set_index_buffer(character.indices.slice(..), wgpu::IndexFormat::Uint32);
            for section in &character.sections {
                pass.draw_indexed(section.first_index..section.first_index + section.index_count, 0, 0..1);
            }
            drop(pass);
            self.shadow_blur.draw(encoder, &self.shadow_targets.entries[target_index].blur_group, &self.shadow_targets.entries[target_index].blurred_view);
            if let Some(material) = self.shadow_material(target_index, &projection) {
                self.shadow_projectors.push(ShadowProjector { material, projection });
            }
        }
    }

    fn draw_shadow_projectors<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        for projector in &self.shadow_projectors {
            let Some(gpu_pass) = projector.material.passes.first() else { continue };
            let camera = crate::camera::Camera {
                eye: projector.projection.camera,
                target: projector.projection.origin,
                fov_x_deg: projector.projection.fov_deg,
                near: 1.0,
                far: projector.projection.distance * 2.0,
            };
            let frustum = camera.frustum(1.0);
            pass.set_bind_group(1, &gpu_pass.bind_group, &[0]);
            let mut bound: Option<(&wgpu::Buffer, &wgpu::Buffer)> = None;
            for draw in self.draws.iter().filter(|draw| draw.index_count != 0 && matches!(draw.material.context.kind, DrawKind::Bsp | DrawKind::Terrain | DrawKind::StaticMesh) && self.tile_visible(draw) && frustum.contains_sphere(draw.center, draw.radius)) {
                let key = PipelineKey { instanced: draw.instances.is_some(), ..gpu_pass.key };
                let Some(pipeline) = self.pipelines.statics(&key) else { continue };
                pass.set_pipeline(pipeline);
                if let Some(instances) = &draw.instances { pass.set_vertex_buffer(1, instances.slice(..)); }
                if bound != Some((&draw.vertices, &draw.indices)) {
                    pass.set_vertex_buffer(0, draw.vertices.slice(..));
                    pass.set_index_buffer(draw.indices.slice(..), wgpu::IndexFormat::Uint32);
                    bound = Some((&draw.vertices, &draw.indices));
                }
                pass.draw_indexed(draw.first_index..draw.first_index + draw.index_count, draw.base_vertex, 0..draw.instance_count);
            }
        }
    }

    pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder, output: &wgpu::TextureView, depth: &wgpu::TextureView, size: (u32, u32), camera: &Camera, environment: &Environment) {
        self.render_scene(encoder, (output, depth), size, camera, environment, true);
    }

    pub fn viewport_target(&self, size: (u32, u32)) -> ViewportTarget {
        ViewportTarget::new(&self.device, size)
    }

    pub fn render_viewport(&mut self, target: &mut ViewportTarget, camera: &Camera, environment: &Environment) -> bool {
        if self.postprocess.as_ref().is_some_and(|postprocess| postprocess.settings.active()) {
            return false;
        }
        let mut encoder = self.device.create_command_encoder(&Default::default());
        let present = self.present.take();
        std::mem::swap(&mut self.multisample_frame, &mut target.multisample);
        let particle_delta = self.particle_delta.replace(0.0);
        let (color, depth, size) = (target.color_view.clone(), target.depth_view.clone(), target.size);
        self.render_scene(&mut encoder, (&color, &depth), size, camera, environment, false);
        self.particle_delta = particle_delta;
        std::mem::swap(&mut self.multisample_frame, &mut target.multisample);
        self.present = present;
        self.queue.submit([encoder.finish()]);
        true
    }

    fn render_scene(&mut self, encoder: &mut wgpu::CommandEncoder, (output, depth): (&wgpu::TextureView, &wgpu::TextureView), size: (u32, u32), camera: &Camera, environment: &Environment, overlay: bool) {
        self.update_projector_visibility();
        let dt = self.viewer.map_or((self.time - self.moving_time).max(0.0), |v| v.dt);
        if overlay {
            self.moving_time = self.time;
            if self.movers_auto_tick { self.tick_movers(dt); }
        }
        self.prepare_particles(camera, size.0 as f32 / size.1 as f32);
        let frame = self.present.as_mut().map(|present| present.frame_view(&self.device, size));
        let resolved_color = frame.as_ref().unwrap_or(output);
        let post_input = self.postprocess.as_mut().filter(|postprocess| postprocess.settings.active()).map(|postprocess| postprocess.input(&self.device, size));
        let scene_color = post_input.as_ref().unwrap_or(resolved_color);
        if self.sample_count > 1 && self.multisample_frame.as_ref().is_none_or(|frame| frame.size != size) {
            self.multisample_frame = Some(crate::video::MultisampleFrame::new(&self.device, size, self.sample_count));
        }
        let multisample = self.multisample_frame.as_ref().map(|frame| (frame.color.clone(), frame.depth.clone()));
        let color = multisample.as_ref().map_or(scene_color, |frame| &frame.0);
        let depth = multisample.as_ref().map_or(depth, |frame| &frame.1);
        self.sky = environment.sky;
        self.sky_hour = environment.hour;
        let fog = self.fog_frame(environment, camera.eye);
        if let Some(over) = &fog.sky {
            self.sky.sky_box = over.sky;
            if let Some(haze) = over.haze.first() {
                self.sky.hazering = *haze;
            }
            for (slot, cloud) in self.sky.clouds.iter_mut().zip(&over.clouds) {
                *slot = *cloud;
            }
        }
        self.upload_instances();
        self.animate_materials();
        let aspect = size.0 as f32 / size.1 as f32;
        let mut env_uniform: EnvironmentUniform = environment.into();
        env_uniform.fog_color = [fog.values.color[0], fog.values.color[1], fog.values.color[2], if fog.enabled { 1.0 } else { 0.0 }];
        env_uniform.fog_range = [fog.values.start, fog.values.end, 0.0, 0.0];
        self.last_fog = Some(fog);
        self.queue.write_buffer(&self.env_buffer, 0, bytemuck::bytes_of(&env_uniform));
        self.uniform_slots.get_mut().next = 0;
        self.render_shadows(encoder, environment);
        let viewer_zone = self.viewer.map_or_else(ZoneLighting::default, |viewer| self.zone(self.zone_base_at(viewer.position) + u32::from(viewer.zone)));
        let clear = match (viewer_zone.clear_to_fog, viewer_zone.fog) {
            (true, Some(zone_fog)) => wgpu::Color { r: f64::from(zone_fog.color[0]), g: f64::from(zone_fog.color[1]), b: f64::from(zone_fog.color[2]), a: 1.0 },
            _ => wgpu::Color { r: f64::from(CLEAR_RGBA[0]) / 255.0, g: f64::from(CLEAR_RGBA[1]) / 255.0, b: f64::from(CLEAR_RGBA[2]) / 255.0, a: 1.0 },
        };
        self.update_actor_views(&camera.frustum(aspect), camera.eye);
        let (sky, world): (Vec<&GpuDraw>, Vec<&GpuDraw>) = self.draws.iter().chain(self.particle_actors.iter().flat_map(|a| a.draws.iter().flatten())).filter(|draw| draw.index_count != 0 && self.tile_visible(draw)).partition(|draw| self.zone(draw.zone).is_sky);
        let mut color_load = wgpu::LoadOp::Clear(clear);
        let mut sky_zones: Vec<u32> = sky.iter().map(|draw| draw.zone).chain(self.bodies.iter().filter(|body| self.tiles.get(body.tile).is_none_or(|tile| tile.visible)).map(|body| body.zone)).collect();
        sky_zones.sort_unstable();
        sky_zones.dedup();
        if !sky_zones.is_empty() {
            let mut pass = begin_pass(encoder, "céu", color, depth, color_load);
            for zone in sky_zones {
                let sky_view = sky_camera(camera, glam::Vec3::from(self.zone(zone).sky_location));
                let group0 = self.group0(&sky_view, aspect);
                pass.set_bind_group(0, &group0, &[]);
                self.draw_all(&mut pass, &ordered(sky.iter().copied().filter(|draw| draw.zone == zone), sky_view.eye));
                self.draw_bodies(&mut pass, zone, &sky_view, aspect);
            }
            color_load = wgpu::LoadOp::Load;
        }
        let group0 = self.group0(camera, aspect);
        let mut pass = begin_pass(encoder, "mundo", color, depth, color_load);
        pass.set_bind_group(0, &group0, &[]);
        let frustum = camera.frustum(aspect);
        let draws = ordered(world.into_iter().filter(|draw| self.sector_visible(draw, &frustum, camera.eye)), camera.eye);
        let split = draws.iter().position(|draw| draw.pass == Pass::Translucent).unwrap_or(draws.len());
        let (solid, translucent) = draws.split_at(split);
        self.draw_all(&mut pass, solid);
        self.draw_shadow_projectors(&mut pass);
        self.draw_characters(&mut pass, false, camera.eye);
        self.draw_all(&mut pass, translucent);
        self.draw_characters(&mut pass, true, camera.eye);
        for corona in self.coronas.iter().filter(|c| c.state.fade > 0.0 && self.tiles.get(c.tile).is_some_and(|t| t.visible)) {
            self.queue.write_buffer(&corona.vertices, 0, bytemuck::cast_slice(&crate::coronas::quad(&corona.light, camera, aspect, corona.state.fade)));
            pass.set_vertex_buffer(0, corona.vertices.slice(..));
            pass.set_index_buffer(corona.indices.slice(..), wgpu::IndexFormat::Uint32);
            for gpu_pass in &corona.material.passes {
                let Some(pipeline) = self.pipelines.statics(&gpu_pass.key) else { continue };
                self.textures.evaluate_material(&self.queue, gpu_pass.slot, f64::from(self.time));
                pass.set_pipeline(pipeline);
                pass.set_bind_group(1, &gpu_pass.bind_group, &[0]);
                pass.draw_indexed(0..6, 0, 0..1);
            }
        }
        self.lines.draw(&mut pass, &self.queue, camera.view_proj(aspect));
        drop(pass);
        self.draw_lens_flare(encoder, color, depth, size, camera, aspect);
        if post_input.is_some() && let Some(postprocess) = &mut self.postprocess {
            postprocess.render(&self.device, encoder, resolved_color, self.time, self.texture_filtering.trilinear);
        }
        let ui_color = if post_input.is_some() { resolved_color } else { color };
        if overlay {
            self.world_text.render_depth(encoder, &self.queue, ui_color, size, Some(depth));
            let ui_size = self.ui_size.unwrap_or(size);
            let radar_batch = self.ui.radar_batch();
            self.ui.render_part(encoder, &self.queue, ui_color, ui_size, Some(depth), 0..radar_batch);
            self.render_radar(encoder, ui_color, depth, size, &env_uniform);
            self.ui.render_part(encoder, &self.queue, ui_color, ui_size, Some(depth), radar_batch..usize::MAX);
        }
        if multisample.is_some() {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("video resolve"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color, resolve_target: Some(resolved_color), depth_slice: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                ..Default::default()
            });
        }
        if let Some(present) = &self.present {
            present.blit(encoder, output);
        }
        if overlay {
            self.tick_actor_views(dt);
        }
    }

    fn draw_bodies<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, zone: u32, sky_view: &Camera, aspect: f32) {
        let view = sky_view.view_proj(aspect);
        for body in self.bodies.iter().filter(|body| body.zone == zone && self.tiles.get(body.tile).is_none_or(|tile| tile.visible)) {
            let Some(placement) = place_body(&body.body, &self.sky) else { continue };
            let quad = body_quad(placement.center, placement.half, view, placement.color);
            self.queue.write_buffer(&body.vertices, 0, bytemuck::cast_slice(&quad));
            pass.set_vertex_buffer(0, body.vertices.slice(..));
            pass.set_index_buffer(body.indices.slice(..), wgpu::IndexFormat::Uint32);
            for gpu_pass in &body.material.passes {
                self.textures.evaluate_material(&self.queue, gpu_pass.slot, f64::from(self.time));
                let Some(pipeline) = self.pipelines.statics(&gpu_pass.key) else { continue };
                pass.set_pipeline(pipeline);
                pass.set_bind_group(1, &gpu_pass.bind_group, &[0]);
                pass.draw_indexed(0..QUAD_INDICES.len() as u32, 0, 0..1);
            }
        }
    }

    /// The sun's pixel position this frame, when a visible sun is in front of the camera.
    fn sun_pixel(&self, camera: &Camera, aspect: f32, size: (u32, u32)) -> Option<glam::Vec2> {
        let sun = self.bodies.iter().find(|body| body.body.kind == killer_assets::scene::sky::BodyKind::Sun && self.tiles.get(body.tile).is_none_or(|tile| tile.visible))?;
        let placement = place_body(&sun.body, &self.sky)?;
        let sky_view = sky_camera(camera, glam::Vec3::from(self.zone(sun.zone).sky_location));
        project(sky_view.view_proj(aspect), placement.center, size.0 as f32, size.1 as f32)
    }

    fn draw_lens_flare(&self, encoder: &mut wgpu::CommandEncoder, color: &wgpu::TextureView, depth: &wgpu::TextureView, size: (u32, u32), camera: &Camera, aspect: f32) {
        if self.flares.is_empty() || !self.flare_clear || self.sky.phase == killer_assets::timeenv::PHASE_NIGHT {
            return;
        }
        let Some(sun) = self.sun_pixel(camera, aspect, size) else { return };
        let (width, height) = (size.0 as f32, size.1 as f32);
        let quads = lens_flare(sun, width, height, &self.flares.iter().map(|flare| flare.flare.clone()).collect::<Vec<_>>());
        let group0 = self.slot_group(&CameraUniform { view_proj: pixel_projection(width, height).to_cols_array_2d(), view: Mat4::IDENTITY.to_cols_array_2d(), eye: [0.0, 0.0, 0.0, 1.0] });
        for (flare, quad) in self.flares.iter().zip(&quads) {
            self.queue.write_buffer(&flare.vertices, 0, bytemuck::cast_slice(&flare_quad(*quad)));
        }
        let mut pass = begin_pass(encoder, "lens flare", color, depth, wgpu::LoadOp::Load);
        pass.set_bind_group(0, &group0, &[]);
        for flare in self.flares.iter().filter(|flare| self.tiles.get(flare.tile).is_none_or(|tile| tile.visible)) {
            pass.set_vertex_buffer(0, flare.vertices.slice(..));
            pass.set_index_buffer(flare.indices.slice(..), wgpu::IndexFormat::Uint32);
            for gpu_pass in &flare.material.passes {
                self.textures.evaluate_material(&self.queue, gpu_pass.slot, f64::from(self.time));
                let Some(pipeline) = self.pipelines.statics(&gpu_pass.key) else { continue };
                pass.set_pipeline(pipeline);
                pass.set_bind_group(1, &gpu_pass.bind_group, &[0]);
                pass.draw_indexed(0..QUAD_INDICES.len() as u32, 0, 0..1);
            }
        }
    }

    fn draw_characters<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, translucent: bool, eye: glam::Vec3) {
        let mut instances: Vec<&DrawInstance> = self.instances.iter().collect();
        if translucent {
            instances.sort_by(|a, b| (b.center - eye).length_squared().total_cmp(&(a.center - eye).length_squared()));
        }
        let mut previous_pipeline=None;
        let mut previous_material=None;
        let mut previous_character=None;
        for instance in instances {
            let Some(character) = self.characters.get(instance.character).and_then(Option::as_ref) else { continue };
            let in_layer = |section: &GpuSection| section.translucent == translucent;
            if !character.sections.iter().any(in_layer) {
                continue;
            }
            pass.set_bind_group(2, &self.object_buffers.bind_group, &[instance.offset]);
            if previous_character!=Some(instance.character) {
                pass.set_vertex_buffer(0, character.vertices.slice(..));
                pass.set_index_buffer(character.indices.slice(..), wgpu::IndexFormat::Uint32);
                previous_character=Some(instance.character);
            }
            for section in character.sections.iter().filter(|section| in_layer(section)) {
                for gpu_pass in &section.material.passes {
                    let Some(pipeline) = self.pipelines.skinned(&gpu_pass.key) else { continue };
                    self.textures.evaluate_material(&self.queue, gpu_pass.slot, f64::from(self.time));
                    if previous_pipeline!=Some(gpu_pass.key) {
                        pass.set_pipeline(pipeline);
                        previous_pipeline=Some(gpu_pass.key);
                    }
                    if previous_material!=Some(gpu_pass.slot) {
                        pass.set_bind_group(1, &gpu_pass.bind_group, &[0]);
                        previous_material=Some(gpu_pass.slot);
                    }
                    pass.draw_indexed(section.first_index..section.first_index + section.index_count, 0, 0..1);
                }
            }
        }
    }

    fn draw_all<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, draws: &[&'a GpuDraw]) {
        let mut previous_pipeline=None;
        let mut previous_material=None;
        let mut bound: Option<(&wgpu::Buffer, &wgpu::Buffer)> = None;
        for &draw in draws {
            if let Some(instances) = &draw.instances { pass.set_vertex_buffer(1, instances.slice(..)); }
            if bound != Some((&draw.vertices, &draw.indices)) {
                pass.set_vertex_buffer(0, draw.vertices.slice(..));
                pass.set_index_buffer(draw.indices.slice(..), wgpu::IndexFormat::Uint32);
                bound = Some((&draw.vertices, &draw.indices));
            }
            let fade = Self::fade_slot(&self.tiles, draw);
            for gpu_pass in &draw.material.passes {
                let faded = fade.filter(|_| gpu_pass.fades);
                let key = if faded.is_some() { gpu_pass.key.faded() } else { gpu_pass.key };
                let Some(pipeline) = self.pipelines.statics(&key) else { continue };
                self.textures.evaluate_material(&self.queue, gpu_pass.slot, f64::from(self.time));
                if previous_pipeline!=Some(key) {
                    pass.set_pipeline(pipeline);
                    previous_pipeline=Some(key);
                }
                if let Some(slot) = faded {
                    let offset = u64::from(slot) * size_of::<f32>() as u64;
                    pass.set_vertex_buffer(1, self.fade_buffer.slice(offset..offset + size_of::<f32>() as u64));
                }
                if previous_material!=Some(gpu_pass.slot) {
                    pass.set_bind_group(1, &gpu_pass.bind_group, &[0]);
                    previous_material=Some(gpu_pass.slot);
                }
                pass.draw_indexed(draw.first_index..draw.first_index + draw.index_count, draw.base_vertex, 0..draw.instance_count);
            }
        }
    }
}

fn bind_material(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, samplers: &[wgpu::Sampler; 2], buffer: &wgpu::Buffer, slot: u32, views: &PassViews) -> wgpu::BindGroup {
    let sampler = |clamp: bool| &samplers[usize::from(clamp)];
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&views.textures[0]) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(sampler(views.clamps[0])) },
            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&views.textures[1]) },
            wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(sampler(views.clamps[1])) },
            wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&views.textures[2]) },
            wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::Sampler(sampler(views.clamps[2])) },
            wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::TextureView(&views.cube) },
            wgpu::BindGroupEntry { binding: 7, resource: wgpu::BindingResource::Sampler(sampler(true)) },
            wgpu::BindGroupEntry { binding: 8, resource: wgpu::BindingResource::TextureView(&views.mask) },
            wgpu::BindGroupEntry { binding: 9, resource: wgpu::BindingResource::Sampler(sampler(false)) },
            wgpu::BindGroupEntry { binding: 11, resource: wgpu::BindingResource::TextureView(&views.lightmap) },
            wgpu::BindGroupEntry { binding: 12, resource: wgpu::BindingResource::Sampler(sampler(true)) },
            wgpu::BindGroupEntry { binding: 13, resource: wgpu::BindingResource::TextureView(&views.textures[3]) },
            wgpu::BindGroupEntry { binding: 14, resource: wgpu::BindingResource::Sampler(sampler(views.clamps[3])) },
            wgpu::BindGroupEntry { binding: 15, resource: wgpu::BindingResource::TextureView(&views.textures[4]) },
            wgpu::BindGroupEntry { binding: 16, resource: wgpu::BindingResource::Sampler(sampler(views.clamps[4])) },
            wgpu::BindGroupEntry { binding: 17, resource: wgpu::BindingResource::TextureView(&views.textures[5]) },
            wgpu::BindGroupEntry { binding: 18, resource: wgpu::BindingResource::Sampler(sampler(views.clamps[5])) },
            wgpu::BindGroupEntry { binding: 19, resource: wgpu::BindingResource::TextureView(&views.textures[6]) },
            wgpu::BindGroupEntry { binding: 20, resource: wgpu::BindingResource::Sampler(sampler(views.clamps[6])) },
            wgpu::BindGroupEntry { binding: 21, resource: wgpu::BindingResource::TextureView(&views.textures[7]) },
            wgpu::BindGroupEntry { binding: 22, resource: wgpu::BindingResource::Sampler(sampler(views.clamps[7])) },
            wgpu::BindGroupEntry {
                binding: 10,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer, offset: u64::from(slot) * MATERIAL_STRIDE, size: NonZeroU64::new(size_of::<MaterialGpu>() as u64) }),
            },
        ],
    })
}

fn object_buffers(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, stride: u64, slots: u64, matrices: u64) -> ObjectBuffers {
    let objects = device.create_buffer(&wgpu::BufferDescriptor { label: Some("objetos"), size: stride * slots.max(1), usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    let palette = device.create_buffer(&wgpu::BufferDescriptor { label: Some("paleta"), size: 64 * matrices.max(1), usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("objeto"),
        layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: &objects, offset: 0, size: NonZeroU64::new(size_of::<ObjectUniform>() as u64) }) },
            wgpu::BindGroupEntry { binding: 1, resource: palette.as_entire_binding() },
        ],
    });
    ObjectBuffers { stride, objects, palette, bind_group }
}

fn zone_buffer(device: &wgpu::Device, zones: &[ZoneLighting], highest_zone: usize) -> wgpu::Buffer {
    let count = zones.len().max(highest_zone + 1);
    let uniforms: Vec<ZoneUniform> = (0..count).map(|id| ZoneUniform::from(&zones.get(id).copied().unwrap_or_default())).collect();
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("zonas"), contents: bytemuck::cast_slice(&uniforms), usage: wgpu::BufferUsages::STORAGE })
}

fn fade_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor { label: Some("alfa dos actors"), size: (capacity * size_of::<f32>()) as u64, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false })
}

fn begin_pass<'a>(encoder: &'a mut wgpu::CommandEncoder, label: &str, color: &'a wgpu::TextureView, depth: &'a wgpu::TextureView, color_load: wgpu::LoadOp<wgpu::Color>) -> wgpu::RenderPass<'a> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: color, resolve_target: None, ops: wgpu::Operations { load: color_load, store: wgpu::StoreOp::Store }, depth_slice: None })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { view: depth, depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }), stencil_ops: None }),
        ..Default::default()
    })
}

fn ordered<'a>(draws: impl Iterator<Item = &'a GpuDraw>, eye: glam::Vec3) -> Vec<&'a GpuDraw> {
    let mut opaque = Vec::new();
    let mut layers = Vec::new();
    let mut translucent = Vec::new();
    let mut projectors = Vec::new();
    for draw in draws {
        match draw.pass {
            Pass::Opaque => opaque.push(draw),
            Pass::Layer(_) => layers.push(draw),
            Pass::Translucent => translucent.push(draw),
            Pass::Projector => projectors.push(draw),
        }
    }
    let replaces = |draw: &&GpuDraw| {
        draw.material.passes.first().is_some_and(|pass| {
            pass.key.dst == l2_catalog::BlendFactor::Zero && pass.key.src == l2_catalog::BlendFactor::One && pass.key.z_write && pass.key.z_test && !pass.fades
        })
    };
    for run in opaque.chunk_by_mut(|a, b| replaces(a) == replaces(b)) {
        if run.first().is_some_and(replaces) {
            run.sort_by_key(|d| d.material.passes.first().map_or(u32::MAX, |pass| pass.slot));
        }
    }
    layers.sort_by_key(|d| match d.pass {
        Pass::Layer(i) => i,
        _ => 0,
    });
    translucent.sort_by(|a, b| {
        let da = (a.center - eye).length_squared();
        let db = (b.center - eye).length_squared();
        db.total_cmp(&da).then(a.sort_level.cmp(&b.sort_level))
    });
    opaque.into_iter().chain(layers).chain(projectors).chain(translucent).collect()
}
