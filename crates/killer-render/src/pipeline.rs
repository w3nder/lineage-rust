use crate::skinned::{ObjectUniform, SkinnedVertex};
use bytemuck::{Pod, Zeroable};
use killer_assets::scene::sky::SkyTint;
use killer_assets::scene::{DrawKind, Vertex, ZoneLighting};
use l2_catalog::{AlphaOp, BlendFactor, ColorOp, CoordSource, FogMode, Material, Pass, StageSource};
use rustc_hash::FxHashMap;
use std::mem::size_of;
use std::num::NonZeroU64;

pub const MAX_STAGES: usize = 8;
pub const MATERIAL_STRIDE: u64 = 1024;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub eye: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug, PartialEq)]
pub struct ZoneUniform {
    pub sun: [f32; 4],
}

impl From<&ZoneLighting> for ZoneUniform {
    fn from(zone: &ZoneLighting) -> Self {
        Self { sun: [if zone.sun_affect { 1.0 } else { 0.0 }, 0.0, 0.0, 0.0] }
    }
}

pub const STAGE_SOURCE_TEXTURE: u32 = 0;
pub const STAGE_SOURCE_CUBE: u32 = 1;
pub const STAGE_SOURCE_FACTOR: u32 = 2;
pub const STAGE_SOURCE_DIFFUSE: u32 = 3;
pub const COORD_STREAM0: u32 = 0;
pub const COORD_STREAM1: u32 = 1;
pub const COORD_CAMERA_ENV: u32 = 2;
pub const COORD_WORLD_ENV: u32 = 3;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug, PartialEq, Default)]
pub struct StageGpu {
    pub color_op: u32,
    pub alpha_op: u32,
    pub flags: u32,
    pub frame: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug, PartialEq)]
pub struct MaterialGpu {
    pub ambient: [f32; 4],
    pub stages: [StageGpu; MAX_STAGES],
    pub uv_u: [[f32; 4]; MAX_STAGES],
    pub uv_v: [[f32; 4]; MAX_STAGES],
    pub tfactor: [f32; 4],
    pub alpha_ref: f32,
    pub alpha_test: u32,
    pub fog_mode: u32,
    pub stage_count: u32,
    pub kind: u32,
    pub sun_affect: u32,
    pub zone: u32,
    pub lit: u32,
    pub light_after: u32,
    pub has_mask: u32,
    pub lightmap_slices: u32,
    pub modulate_2x: u32,
    pub fade: [f32; 2],
    pub padding: [u32; 2],
    pub projection: [[[f32; 4]; 4]; MAX_STAGES],
}

pub fn color_op_index(op: ColorOp) -> u32 {
    match op {
        ColorOp::SelectTexture => 0,
        ColorOp::SelectCurrent => 1,
        ColorOp::Modulate => 2,
        ColorOp::Modulate2X => 3,
        ColorOp::Modulate4X => 4,
        ColorOp::Add => 5,
        ColorOp::Subtract => 6,
        ColorOp::BlendTextureAlpha => 7,
        ColorOp::BlendDiffuseAlpha => 8,
        ColorOp::BlendCurrentAlpha => 9,
        ColorOp::BlendFactorAlpha => 10,
        ColorOp::ModulateAlphaAddColor => 11,
        ColorOp::MultiplyAdd => 12,
        ColorOp::BlendCurrentFactorByTextureAlpha => 13,
        ColorOp::BlendTextureFactorByDiffuseAlpha => 14,
        ColorOp::ModulateCurrentByTextureAlpha => 15,
        ColorOp::AddSigned => 16,
        ColorOp::AddSigned2X => 17,
        ColorOp::AddSmooth => 18,
        ColorOp::BlendTextureAlphaPremultiplied => 19,
        ColorOp::ModulateColorAddAlpha => 20,
        ColorOp::ModulateInvAlphaAddColor => 21,
        ColorOp::ModulateInvColorAddAlpha => 22,
        ColorOp::Disable => 23,
        ColorOp::PreModulate => 24,
        ColorOp::MultiplyAddCurrent => 25,
        ColorOp::DotProduct3 => 26,
    }
}

pub fn alpha_op_index(op: AlphaOp) -> u32 {
    match op {
        AlphaOp::Texture => 0,
        AlphaOp::Current => 1,
        AlphaOp::Diffuse => 2,
        AlphaOp::TFactor => 3,
        AlphaOp::Modulate => 4,
        AlphaOp::Add => 5,
        AlphaOp::BlendDiffuseAlpha => 6,
    }
}

pub fn fog_mode_index(fog: FogMode) -> u32 {
    match fog {
        FogMode::Zone => 0,
        FogMode::Black => 1,
        FogMode::Gray127 => 2,
    }
}

pub fn kind_index(kind: DrawKind) -> u32 {
    match kind {
        DrawKind::StaticMesh => 0,
        DrawKind::Bsp => 1,
        DrawKind::Terrain => 2,
        DrawKind::Actor => 3,
        DrawKind::Sprite => 4,
        DrawKind::Particle => 5,
        DrawKind::Projector => 6,
    }
}

pub struct DrawContext {
    pub ambient: killer_assets::scene::actor_lighting::Ambient,
    pub kind: DrawKind,
    pub sun_affect: bool,
    pub zone: u32,
    pub has_mask: bool,
    pub lightmap_slices: u32,
    pub modulate_2x: bool,
    pub fade: Option<[f32; 2]>,
    pub tint: Option<SkyTint>,
}

impl MaterialGpu {
    pub fn new(material: &Material, pass: &Pass, context: &DrawContext, time: f32) -> Self {
        Self::at(material, pass, context, time, None)
    }

    /// Like `new`, with the `ColorModifier` color replaced by the hour's sky color (`UpdateSkyColors`).
    pub fn at(material: &Material, pass: &Pass, context: &DrawContext, time: f32, tint: Option<[f32; 3]>) -> Self {
        let mut stages = [StageGpu::default(); MAX_STAGES];
        let mut uv_u = [[0.0; 4]; MAX_STAGES];
        let mut uv_v = [[0.0; 4]; MAX_STAGES];
        let mut projection = [[[0.0; 4]; 4]; MAX_STAGES];
        let used = material.pass_stages(pass).iter().take(MAX_STAGES);
        let mut count = 0;
        for (slot, stage) in used.enumerate() {
            let source = match &stage.source {
                StageSource::Texture(_) => STAGE_SOURCE_TEXTURE,
                StageSource::Cubemap(_) => STAGE_SOURCE_CUBE,
                StageSource::Factor => STAGE_SOURCE_FACTOR,
                StageSource::Diffuse => STAGE_SOURCE_DIFFUSE,
                StageSource::ProjectorFacing => 4,
            };
            let coord = match stage.coord_source {
                CoordSource::Stream(0) => COORD_STREAM0,
                CoordSource::Stream(_) => COORD_STREAM1,
                CoordSource::CameraEnvMap => COORD_CAMERA_ENV,
                CoordSource::WorldEnvMap => COORD_WORLD_ENV,
                CoordSource::Projection { matrix, divide_z } => {
                    projection[slot] = matrix;
                    4 + u32::from(divide_z)
                }
                CoordSource::ProjectorNormal { matrix } => {
                    projection[slot] = matrix;
                    6
                }
                CoordSource::ProjectorStream => 7,
            };
            let matrix = stage.matrix(time).0;
            stages[slot] = StageGpu {
                color_op: color_op_index(stage.color_op),
                alpha_op: alpha_op_index(stage.alpha_op),
                flags: u32::from(stage.swapped) | (source << 1) | (coord << 4),
                frame: stage.frame(time),
            };
            uv_u[slot] = [matrix[0][0], matrix[1][0], matrix[2][0], 0.0];
            uv_v[slot] = [matrix[0][1], matrix[1][1], matrix[2][1], 0.0];
            if coord == COORD_CAMERA_ENV {
                projection[slot] = [
                    [matrix[0][0], matrix[0][1], matrix[0][2], 0.0],
                    [matrix[1][0], matrix[1][1], matrix[1][2], 0.0],
                    [matrix[2][0], matrix[2][1], matrix[2][2], 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ];
            }
            count += 1;
        }
        let mut tfactor = pass.tfactor_at(time).map(|channel| f32::from(channel) / 255.0);
        if let Some(rgb) = tint {
            tfactor[..3].copy_from_slice(&rgb);
        }
        Self {
            stages,
            uv_u,
            uv_v,
            tfactor,
            alpha_ref: pass.alpha_test.map_or(0.0, |alpha_ref| f32::from(alpha_ref) / 255.0),
            alpha_test: u32::from(pass.alpha_test.is_some()),
            fog_mode: fog_mode_index(pass.fog),
            stage_count: count,
            kind: kind_index(context.kind),
            sun_affect: u32::from(context.sun_affect),
            zone: context.zone,
            lit: u32::from(pass.lit),
            light_after: pass.light_after.min(MAX_STAGES) as u32,
            has_mask: u32::from(context.has_mask),
            lightmap_slices: context.lightmap_slices,
            modulate_2x: u32::from(context.modulate_2x),
            fade: context.fade.unwrap_or([0.0; 2]),
            padding: [0, modifier_class(pass)],
            projection,
            ambient: {
                let color = context.ambient.color(time).map(|c| f32::from(c >> 1) / 255.0);
                [color[0], color[1], color[2], 0.0]
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CullMode {
    None,
    Back,
    Front,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PipelineKey {
    pub src: BlendFactor,
    pub dst: BlendFactor,
    pub alpha_blend: bool,
    pub z_write: bool,
    pub z_test: bool,
    pub cull: CullMode,
    pub bsp_projector: bool,
    pub instanced: bool,
    pub faded: bool,
}

impl PipelineKey {
    pub fn of(pass: &Pass, kind: DrawKind) -> Self {
        let cull = match (pass.two_sided, kind) {
            (true, _) => CullMode::None,
            (false, DrawKind::Bsp) => CullMode::Front,
            (false, _) => CullMode::Back,
        };
        Self { src: pass.src, dst: pass.dst, alpha_blend: pass.alpha_blend, z_write: pass.z_write, z_test: pass.z_test, cull, bsp_projector: false, instanced: false, faded: false }
    }

    /// The pipeline of a pass under `UL2ColorModifier`'s alpha: `ONE/ZERO` is promoted to `SRCALPHA/INVSRCALPHA` and the alpha comes from the instance.
    pub fn faded(self) -> Self {
        let promote = self.src == BlendFactor::One && self.dst == BlendFactor::Zero;
        Self {
            src: if promote { BlendFactor::SrcAlpha } else { self.src },
            dst: if promote { BlendFactor::InvSrcAlpha } else { self.dst },
            alpha_blend: self.alpha_blend || promote,
            instanced: false,
            faded: true,
            ..self
        }
    }
}

/// How `L2ColorModifier`'s stage (0x10004fe0) treats a pass: 1 scales RGB by the alpha (`ONE/INVSRCCOLOR`), 2 replaces the alpha
/// (`SRCALPHA/INVSRCALPHA`, or `ONE/ZERO` once promoted), 0 leaves it untouched. Bit 2 makes the alpha `MODULATE` the pass's own.
pub fn modifier_class(pass: &Pass) -> u32 {
    let class = match (pass.src, pass.dst) {
        (BlendFactor::One, BlendFactor::InvSrcColor) => 1,
        (BlendFactor::One, BlendFactor::Zero) | (BlendFactor::SrcAlpha, BlendFactor::InvSrcAlpha) => 2,
        _ => 0,
    };
    class | if pass.alpha_blend || pass.alpha_test == Some(MASKED_ALPHA_REF) { 4 } else { 0 }
}

const MASKED_ALPHA_REF: u8 = 127;

pub fn blend_factor(factor: BlendFactor) -> wgpu::BlendFactor {
    match factor {
        BlendFactor::Zero => wgpu::BlendFactor::Zero,
        BlendFactor::One => wgpu::BlendFactor::One,
        BlendFactor::SrcColor => wgpu::BlendFactor::Src,
        BlendFactor::InvSrcColor => wgpu::BlendFactor::OneMinusSrc,
        BlendFactor::SrcAlpha => wgpu::BlendFactor::SrcAlpha,
        BlendFactor::InvSrcAlpha => wgpu::BlendFactor::OneMinusSrcAlpha,
        BlendFactor::DstColor => wgpu::BlendFactor::Dst,
    }
}

pub fn blend_state(key: &PipelineKey) -> Option<wgpu::BlendState> {
    key.alpha_blend.then(|| wgpu::BlendState {
        color: wgpu::BlendComponent { src_factor: blend_factor(key.src), dst_factor: blend_factor(key.dst), operation: wgpu::BlendOperation::Add },
        alpha: wgpu::BlendComponent::OVER,
    })
}

pub const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub const STATIC_SHADER: &str = concat!(include_str!("shader_common.wgsl"), include_str!("material.wgsl"), include_str!("shader.wgsl"));
pub const SKINNED_SHADER: &str = concat!(include_str!("shader_common.wgsl"), include_str!("material.wgsl"), include_str!("skinned.wgsl"));
pub const SHADOW_SHADER: &str = include_str!("shadow.wgsl");

fn vertex_layout() -> wgpu::VertexBufferLayout<'static> {
    const ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        0 => Float32x3, 1 => Float32x3, 2 => Float32x2, 3 => Float32x2, 4 => Float32x4
    ];
    wgpu::VertexBufferLayout { array_stride: size_of::<Vertex>() as u64, step_mode: wgpu::VertexStepMode::Vertex, attributes: &ATTRIBUTES }
}

fn decoration_instance_layout() -> wgpu::VertexBufferLayout<'static> {
    const ATTRIBUTES: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
        5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4,
        9 => Float32x2, 10 => Float32x4
    ];
    wgpu::VertexBufferLayout { array_stride: size_of::<killer_assets::scene::DecorationInstance>() as u64, step_mode: wgpu::VertexStepMode::Instance, attributes: &ATTRIBUTES }
}

fn fade_instance_layout() -> wgpu::VertexBufferLayout<'static> {
    const ATTRIBUTES: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![11 => Float32];
    wgpu::VertexBufferLayout { array_stride: size_of::<f32>() as u64, step_mode: wgpu::VertexStepMode::Instance, attributes: &ATTRIBUTES }
}

fn skinned_vertex_layout() -> wgpu::VertexBufferLayout<'static> {
    const ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        0 => Float32x3, 1 => Float32x3, 2 => Float32x2, 3 => Uint32x4, 4 => Float32x4
    ];
    wgpu::VertexBufferLayout { array_stride: size_of::<SkinnedVertex>() as u64, step_mode: wgpu::VertexStepMode::Vertex, attributes: &ATTRIBUTES }
}

struct Variant {
    label: &'static str,
    layout: wgpu::PipelineLayout,
    shader: wgpu::ShaderModule,
    vertex: wgpu::VertexBufferLayout<'static>,
    pipelines: FxHashMap<PipelineKey, wgpu::RenderPipeline>,
}

impl Variant {
    fn prepare(&mut self, device: &wgpu::Device, color_format: wgpu::TextureFormat, key: PipelineKey, sample_count: u32) {
        self.pipelines.entry(key).or_insert_with(|| {
            let mut buffers = vec![Some(self.vertex.clone())];
            if key.instanced { buffers.push(Some(decoration_instance_layout())); }
            if key.faded { buffers.push(Some(fade_instance_layout())); }
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(self.label),
                layout: Some(&self.layout),
                vertex: wgpu::VertexState { module: &self.shader, entry_point: Some(if key.instanced { "vs_instanced" } else if key.faded { "vs_faded" } else { "vs_main" }), buffers: &buffers, compilation_options: Default::default() },
                fragment: Some(wgpu::FragmentState {
                    module: &self.shader,
                    entry_point: Some(if key.bsp_projector { "fs_bsp_projector" } else { "fs_main" }),
                    targets: &[Some(wgpu::ColorTargetState { format: color_format, blend: blend_state(&key), write_mask: wgpu::ColorWrites::ALL })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: match key.cull {
                        CullMode::None => None,
                        CullMode::Back => Some(wgpu::Face::Back),
                        CullMode::Front => Some(wgpu::Face::Front),
                    },
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(key.z_write),
                    depth_compare: Some(if key.z_test { wgpu::CompareFunction::LessEqual } else { wgpu::CompareFunction::Always }),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState { count: sample_count, ..Default::default() },
                multiview_mask: None,
                cache: None,
            })
        });
    }
}

pub struct Pipelines {
    pub group0_layout: wgpu::BindGroupLayout,
    pub material_layout: wgpu::BindGroupLayout,
    pub object_layout: wgpu::BindGroupLayout,
    pub shadow: wgpu::RenderPipeline,
    color_format: wgpu::TextureFormat,
    sample_count: u32,
    statics: Variant,
    skinned: Variant,
}

impl Pipelines {
    pub fn new(device: &wgpu::Device, color_format: wgpu::TextureFormat) -> Self {
        let statics_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("killer"), source: wgpu::ShaderSource::Wgsl(STATIC_SHADER.into()) });
        let skinned_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("skinned"), source: wgpu::ShaderSource::Wgsl(SKINNED_SHADER.into()) });
        let shadow_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("shadow"), source: wgpu::ShaderSource::Wgsl(SHADOW_SHADER.into()) });
        let uniform_entry = |binding: u32, visibility: wgpu::ShaderStages| wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
            count: None,
        };
        let storage_entry = |binding: u32, visibility: wgpu::ShaderStages| wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
            count: None,
        };
        let group0_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("group0"),
            entries: &[uniform_entry(0, wgpu::ShaderStages::VERTEX_FRAGMENT), uniform_entry(1, wgpu::ShaderStages::FRAGMENT), storage_entry(2, wgpu::ShaderStages::FRAGMENT)],
        });
        let texture_entry = |binding, view_dimension| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension, multisampled: false },
            count: None,
        };
        let sampler_entry = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };
        let material_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("material"),
            entries: &[
                texture_entry(0, wgpu::TextureViewDimension::D2Array),
                sampler_entry(1),
                texture_entry(2, wgpu::TextureViewDimension::D2Array),
                sampler_entry(3),
                texture_entry(4, wgpu::TextureViewDimension::D2Array),
                sampler_entry(5),
                texture_entry(6, wgpu::TextureViewDimension::Cube),
                sampler_entry(7),
                texture_entry(8, wgpu::TextureViewDimension::D2),
                sampler_entry(9),
                texture_entry(11, wgpu::TextureViewDimension::D2Array),
                sampler_entry(12),
                texture_entry(13, wgpu::TextureViewDimension::D2Array),
                sampler_entry(14),
                texture_entry(15, wgpu::TextureViewDimension::D2Array),
                sampler_entry(16),
                texture_entry(17, wgpu::TextureViewDimension::D2Array),
                sampler_entry(18),
                texture_entry(19, wgpu::TextureViewDimension::D2Array),
                sampler_entry(20),
                texture_entry(21, wgpu::TextureViewDimension::D2Array),
                sampler_entry(22),
                wgpu::BindGroupLayoutEntry {
                    binding: 10,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(size_of::<MaterialGpu>() as u64) },
                    count: None,
                },
            ],
        });
        let object_entry = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(size_of::<ObjectUniform>() as u64) },
            count: None,
        };
        let object_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("objeto"),
            entries: &[object_entry, storage_entry(1, wgpu::ShaderStages::VERTEX)],
        });
        let statics_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("killer"),
            bind_group_layouts: &[Some(&group0_layout), Some(&material_layout)],
            immediate_size: 0,
        });
        let skinned_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("skinned"),
            bind_group_layouts: &[Some(&group0_layout), Some(&material_layout), Some(&object_layout)],
            immediate_size: 0,
        });
        let shadow_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shadow"),
            bind_group_layouts: &[Some(&group0_layout), Some(&object_layout)],
            immediate_size: 0,
        });
        let shadow = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shadow"),
            layout: Some(&shadow_layout),
            vertex: wgpu::VertexState { module: &shadow_shader, entry_point: Some("vs_main"), buffers: &[Some(skinned_vertex_layout())], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState { module: &shadow_shader, entry_point: Some("fs_main"), targets: &[Some(wgpu::ColorTargetState { format: COLOR_FORMAT, blend: None, write_mask: wgpu::ColorWrites::ALL })], compilation_options: Default::default() }),
            primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState { format: DEPTH_FORMAT, depth_write_enabled: Some(true), depth_compare: Some(wgpu::CompareFunction::LessEqual), stencil: Default::default(), bias: Default::default() }),
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        let statics = Variant { label: "killer", layout: statics_layout, shader: statics_shader, vertex: vertex_layout(), pipelines: FxHashMap::default() };
        let skinned = Variant { label: "skinned", layout: skinned_layout, shader: skinned_shader, vertex: skinned_vertex_layout(), pipelines: FxHashMap::default() };
        Self { group0_layout, material_layout, object_layout, shadow, color_format, sample_count: 1, statics, skinned }
    }

    pub fn prepare(&mut self, device: &wgpu::Device, key: PipelineKey, skinned: bool) {
        let variant = if skinned { &mut self.skinned } else { &mut self.statics };
        variant.prepare(device, self.color_format, key, self.sample_count);
    }

    pub fn set_color_format(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) {
        if self.color_format == format { return; }
        self.color_format = format;
        for variant in [&mut self.statics, &mut self.skinned] {
            let keys: Vec<_> = variant.pipelines.keys().copied().collect();
            variant.pipelines.clear();
            for key in keys { variant.prepare(device, format, key, self.sample_count); }
        }
    }

    pub fn set_sample_count(&mut self, device: &wgpu::Device, count: u32) {
        if self.sample_count == count { return; }
        self.sample_count = count;
        for variant in [&mut self.statics, &mut self.skinned] {
            let keys: Vec<_> = variant.pipelines.keys().copied().collect();
            variant.pipelines.clear();
            for key in keys { variant.prepare(device, self.color_format, key, count); }
        }
    }

    pub fn statics(&self, key: &PipelineKey) -> Option<&wgpu::RenderPipeline> {
        self.statics.pipelines.get(key)
    }

    pub fn skinned(&self, key: &PipelineKey) -> Option<&wgpu::RenderPipeline> {
        self.skinned.pipelines.get(key)
    }

    pub fn color_format(&self) -> wgpu::TextureFormat {
        self.color_format
    }

    pub fn sample_count(&self) -> u32 {
        self.sample_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_assets::scene::texture_material;

    #[test]
    fn the_zone_uniform_carries_only_the_sun_flag() {
        let uniform = ZoneUniform::from(&ZoneLighting::default());
        assert_eq!(uniform.sun[0], 1.0);
        assert_eq!(ZoneUniform::from(&ZoneLighting { sun_affect: false, ..ZoneLighting::default() }).sun[0], 0.0);
    }

    #[test]
    fn the_material_uniform_fits_its_stride_and_packs_the_first_stage() {
        assert!(size_of::<MaterialGpu>() as u64 <= MATERIAL_STRIDE);
        assert_eq!(size_of::<MaterialGpu>() % 16, 0);
        let material = texture_material("Pack.Tex");
        let context = DrawContext { ambient: Default::default(), kind: DrawKind::Bsp, sun_affect: true, zone: 3, has_mask: false, lightmap_slices: 8, modulate_2x: true, fade: None, tint: None };
        let gpu = MaterialGpu::new(&material, &material.passes[0], &context, 0.0);
        assert_eq!(gpu.stage_count, 1);
        assert_eq!(gpu.stages[0], StageGpu { color_op: 0, alpha_op: 0, flags: 0, frame: 0 });
        assert_eq!(gpu.uv_u[0], [1.0, 0.0, 0.0, 0.0]);
        assert_eq!(gpu.uv_v[0], [0.0, 1.0, 0.0, 0.0]);
        assert_eq!((gpu.kind, gpu.sun_affect, gpu.zone, gpu.lit, gpu.light_after, gpu.alpha_test), (1, 1, 3, 1, 1, 0));
        assert_eq!(gpu.tfactor, [1.0; 4]);
        assert_eq!((gpu.lightmap_slices, gpu.modulate_2x), (8, 1));
    }

    #[test]
    fn a_camera_env_map_stage_carries_the_whole_tex_matrix_like_apply_tex_modifier_1000c5b0() {
        let mut material = texture_material("Pack.Tex");
        let rotator = l2_catalog::Rotator { kind: 0, rotation: [-16384, 0, 0], offset: [0.0; 2], oscillation_rate: [500_000, 0, 0], oscillation_amplitude: [700, 0, 0], oscillation_phase: [0; 3] };
        material.stages[0].coord_source = CoordSource::CameraEnvMap;
        material.stages[0].source = StageSource::Cubemap(l2_catalog::Cubemap { faces: std::array::from_fn(|i| format!("Pack.Face{i}")), size: [0.0; 2] });
        material.stages[0].uv = vec![l2_catalog::UvModifier::Rotator(rotator, [0.0; 2])];
        let context = DrawContext { ambient: Default::default(), kind: DrawKind::Actor, sun_affect: true, zone: 0, has_mask: false, lightmap_slices: 0, modulate_2x: true, fade: None, tint: None };
        let gpu = MaterialGpu::new(&material, &material.passes[0], &context, 2.0);
        assert_eq!(gpu.stages[0].flags, (STAGE_SOURCE_CUBE << 1) | (COORD_CAMERA_ENV << 4));
        assert_eq!(gpu.projection[0], [[0.0, 0.0, -1.0, 0.0], [0.0, 1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0]]);
        let plain = texture_material("Pack.Tex");
        assert_eq!(MaterialGpu::new(&plain, &plain.passes[0], &context, 2.0).projection[0], [[0.0; 4]; 4]);
    }

    #[test]
    fn the_pipeline_key_carries_blend_depth_and_cull_from_the_pass() {
        let material = texture_material("Pack.Tex");
        let key = PipelineKey::of(&material.passes[0], DrawKind::Bsp);
        assert_eq!(key, PipelineKey { src: BlendFactor::One, dst: BlendFactor::Zero, alpha_blend: false, z_write: true, z_test: true, cull: CullMode::Front, bsp_projector: false, instanced: false, faded: false });
        let mut bsp_two_sided = material.passes[0].clone();
        bsp_two_sided.two_sided = true;
        assert_eq!(PipelineKey::of(&bsp_two_sided, DrawKind::Bsp).cull, CullMode::None);
        assert_eq!(PipelineKey::of(&material.passes[0], DrawKind::StaticMesh).cull, CullMode::Back);
        assert_eq!(PipelineKey::of(&material.passes[0], DrawKind::Actor).cull, CullMode::Back);
        assert_eq!(PipelineKey::of(&material.passes[0], DrawKind::Terrain).cull, CullMode::Back);
        let mut two_sided = material.passes[0].clone();
        two_sided.two_sided = true;
        assert_eq!(PipelineKey::of(&two_sided, DrawKind::StaticMesh).cull, CullMode::None);
        assert!(blend_state(&key).is_none());
        let additive = PipelineKey { src: BlendFactor::One, dst: BlendFactor::One, alpha_blend: true, ..key };
        let state = blend_state(&additive).unwrap();
        assert_eq!((state.color.src_factor, state.color.dst_factor), (wgpu::BlendFactor::One, wgpu::BlendFactor::One));
    }
}
