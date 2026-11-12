use crate::textures::{GpuTextures, SharedView, TextureSource};
use bytemuck::{Pod, Zeroable};
use killer_ui::draw::{DrawList, Quad};
use std::collections::HashMap;
use std::mem::size_of;
use std::ops::Range;
use std::sync::Arc;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug, PartialEq)]
pub struct UiVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

#[derive(Default)]
pub struct UiMesh {
    pub vertices: Vec<UiVertex>,
    pub indices: Vec<u32>,
    pub clips: Vec<(Range<u32>,[u32;4])>,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ScreenUniform {
    size: [f32; 2],
    _pad: [f32; 2],
}

struct Resolved {
    key: Arc<str>,
    size: [f32; 2],
}

struct Batch {
    texture: Arc<str>,
    indices: Range<u32>,
    clip: Option<[u32;4]>,
    additive: bool,
}

pub struct UiRenderer {
    pipeline: wgpu::RenderPipeline,
    depth_pipeline: wgpu::RenderPipeline,
    additive_pipeline: wgpu::RenderPipeline,
    additive_depth_pipeline: wgpu::RenderPipeline,
    premultiplied: bool,
    texture_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    screen_buffer: wgpu::Buffer,
    screen_group: wgpu::BindGroup,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    index_quads: usize,
    vertex_scratch: Vec<UiVertex>,
    bind_groups: HashMap<String, wgpu::BindGroup>,
    texture_views: HashMap<String, SharedView>,
    external: HashMap<String, (SharedView, [f32; 2])>,
    resolved: HashMap<String, Resolved>,
    used: Vec<Arc<str>>,
    batches: Vec<Batch>,
    radar_batch: Option<usize>,
}

impl UiRenderer {
    pub fn new(device: &wgpu::Device, color_format: wgpu::TextureFormat) -> Self {
        Self::with_samples(device, color_format, 1, false)
    }

    pub fn new_premultiplied(device: &wgpu::Device, color_format: wgpu::TextureFormat) -> Self {
        Self::with_samples(device,color_format,1,true)
    }

    pub fn set_sample_count(&mut self, device: &wgpu::Device, color_format: wgpu::TextureFormat, count: u32) {
        let replacement = Self::with_samples(device, color_format, count, self.premultiplied);
        self.pipeline = replacement.pipeline;
        self.depth_pipeline = replacement.depth_pipeline;
        self.additive_pipeline = replacement.additive_pipeline;
        self.additive_depth_pipeline = replacement.additive_depth_pipeline;
    }

    fn with_samples(device: &wgpu::Device, color_format: wgpu::TextureFormat, sample_count: u32, premultiplied: bool) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ui"),
            source: wgpu::ShaderSource::Wgsl(include_str!("ui.wgsl").into()),
        });
        let screen_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ui tela"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ui textura"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui"),
            bind_group_layouts: &[Some(&screen_layout), Some(&texture_layout)],
            immediate_size: 0,
        });
        let blend = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: if premultiplied {wgpu::BlendFactor::One} else {wgpu::BlendFactor::SrcAlpha},
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        };
        let additive_blend = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        };
        let make_pipeline = |depth, blend: wgpu::BlendState| device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ui"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(vertex_layout())],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(blend),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: depth,
            multisample: wgpu::MultisampleState { count: sample_count, ..Default::default() },
            multiview_mask: None,
            cache: None,
        });
        let depth_state = || Some(wgpu::DepthStencilState {
            format: crate::pipeline::DEPTH_FORMAT,
            depth_write_enabled: Some(false), depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: Default::default(), bias: Default::default(),
        });
        let pipeline = make_pipeline(None, blend);
        let depth_pipeline = make_pipeline(depth_state(), blend);
        let additive_pipeline = make_pipeline(None, additive_blend);
        let additive_depth_pipeline = make_pipeline(depth_state(), additive_blend);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let screen_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ui tela"),
            contents: bytemuck::bytes_of(&ScreenUniform { size: [1.0, 1.0], _pad: [0.0; 2] }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let screen_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ui tela"),
            layout: &screen_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: screen_buffer.as_entire_binding() }],
        });
        let vertices = growable(device, wgpu::BufferUsages::VERTEX, size_of::<UiVertex>() as u64 * 4);
        let indices = growable(device, wgpu::BufferUsages::INDEX, size_of::<u32>() as u64 * 6);
        Self {
            pipeline, depth_pipeline, additive_pipeline, additive_depth_pipeline, premultiplied,
            texture_layout,
            sampler,
            screen_buffer,
            screen_group,
            vertices,
            indices,
            index_quads: 0,
            vertex_scratch: Vec::new(),
            bind_groups: HashMap::new(),
            texture_views: HashMap::new(),
            external: HashMap::new(),
            resolved: HashMap::new(),
            used: Vec::new(),
            batches: Vec::new(),
            radar_batch: None,
        }
    }

    pub fn set_texture_filtering(&mut self, device: &wgpu::Device, filtering: crate::video::TextureFiltering) {
        self.sampler = device.create_sampler(&filtering.sampler(wgpu::AddressMode::Repeat));
        for (path, view) in &self.texture_views {
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ui textura"), layout: &self.texture_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                ],
            });
            self.bind_groups.insert(path.clone(), group);
        }
    }

    pub fn set_external_texture(&mut self, name: &str, view: SharedView, size: [f32; 2]) {
        let key = name.to_lowercase();
        self.bind_groups.remove(&key);
        self.texture_views.remove(&key);
        self.resolved.retain(|_, resolved| *resolved.key != *key);
        self.external.insert(key, (view, size));
    }

    pub fn clear_external_texture(&mut self, name: &str) {
        let key = name.to_lowercase();
        self.external.remove(&key);
        self.bind_groups.remove(&key);
        self.texture_views.remove(&key);
        self.resolved.retain(|_, resolved| *resolved.key != *key);
    }

    pub fn reload_textures(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, textures: &mut GpuTextures, source: &mut dyn TextureSource) {
        self.resolved.clear();
        for (path, group) in &mut self.bind_groups {
            if self.external.contains_key(path) {
                continue;
            }
            let view = textures.view(device, queue, source, path);
            self.texture_views.insert(path.clone(), view.clone());
            *group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ui textura"), layout: &self.texture_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                ],
            });
        }
    }

    pub fn clear(&mut self) {
        self.batches.clear();
        self.bind_groups.clear();
        self.texture_views.clear();
        self.resolved.clear();
    }

    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        list: &DrawList,
        textures: &mut GpuTextures,
        source: &mut dyn TextureSource,
    ) {
        self.batches.clear();
        self.used.clear();
        let vertices = &mut self.vertex_scratch;
        vertices.clear();
        self.radar_batch = None;
        let split = list.radar_at.map(|at| at.min(list.quads.len()));
        let (head, tail) = list.quads.split_at(split.unwrap_or(0));
        let mut fresh = false;
        let same = |a: &Quad, b: &Quad| a.texture == b.texture && a.additive == b.additive;
        let runs = head.chunk_by(same).map(|run| (run, false)).chain(tail.chunk_by(same).enumerate().map(|(index, run)| (run, index == 0 && split.is_some())));
        for (run, boundary) in runs {
            if boundary {
                self.radar_batch = Some(self.batches.len());
                fresh = true;
            }
            let quad = &run[0];
            let cached = self.resolved.get(quad.texture.as_str()).filter(|r| self.bind_groups.contains_key(&*r.key));
            let (key, size) = match cached {
                Some(r) => (r.key.clone(), r.size),
                None => {
                    let (view, size) = match self.external.get(&quad.texture.to_lowercase()) {
                        Some((view, size)) => (view.clone(), *size),
                        None => textures.sized_view(device, queue, source, &quad.texture),
                    };
                    let key: Arc<str> = quad.texture.to_lowercase().into();
                    if !self.bind_groups.contains_key(&*key) {
                        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("ui textura"),
                            layout: &self.texture_layout,
                            entries: &[
                                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                            ],
                        });
                        self.texture_views.insert(key.to_string(), view);
                        self.bind_groups.insert(key.to_string(), group);
                    }
                    self.resolved.insert(quad.texture.to_string(), Resolved { key: key.clone(), size });
                    (key, size)
                }
            };
            let start = (vertices.len() / 4 * 6) as u32;
            for quad in run { vertices.extend(quad_vertices(quad, size)); }
            let end = (vertices.len() / 4 * 6) as u32;
            match self.batches.last_mut() {
                Some(batch) if batch.texture == key && batch.additive == quad.additive && !fresh => batch.indices.end = end,
                _ => self.batches.push(Batch { texture: key.clone(), indices: start..end, clip: None, additive: quad.additive }),
            }
            fresh = false;
            if self.used.last() != Some(&key) { self.used.push(key); }
        }
        if split.is_some() && self.radar_batch.is_none() {
            self.radar_batch = Some(self.batches.len());
        }
        self.used.sort_unstable();
        self.used.dedup();
        let used = &self.used;
        let listed = |key: &str| used.binary_search_by(|u| (**u).cmp(key)).is_ok();
        self.bind_groups.retain(|key, _| listed(key));
        self.texture_views.retain(|key, _| listed(key));
        self.resolved.retain(|_, r| listed(&r.key));
        write_growable(device, queue, &mut self.vertices, wgpu::BufferUsages::VERTEX, bytemuck::cast_slice(vertices));
        if list.quads.len() > self.index_quads {
            let indices: Vec<u32> = (0..list.quads.len() as u32).flat_map(|quad| {
                let base = quad * 4;
                [base, base + 1, base + 2, base, base + 2, base + 3]
            }).collect();
            write_growable(device, queue, &mut self.indices, wgpu::BufferUsages::INDEX, bytemuck::cast_slice(&indices));
            self.index_quads = list.quads.len();
        }
    }

    pub fn upload_mesh(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, mesh: &UiMesh, texture: &wgpu::TextureView) {
        self.batches.clear();
        if mesh.indices.is_empty() {return;}
        let key: Arc<str> = Arc::from("");
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fonte do mundo"), layout: &self.texture_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(texture) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
            ],
        });
        self.bind_groups.insert(key.to_string(),group);
        write_growable(device,queue,&mut self.vertices,wgpu::BufferUsages::VERTEX,bytemuck::cast_slice(&mesh.vertices));
        write_growable(device,queue,&mut self.indices,wgpu::BufferUsages::INDEX,bytemuck::cast_slice(&mesh.indices));
        self.index_quads=0;
        for (indices,clip) in &mesh.clips {
            match self.batches.last_mut() {
                Some(batch) if batch.clip==Some(*clip) && batch.indices.end==indices.start => batch.indices.end=indices.end,
                _ => self.batches.push(Batch {texture:key.clone(),indices:indices.clone(),clip:Some(*clip),additive:false}),
            }
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, queue: &wgpu::Queue, color: &wgpu::TextureView, size: (u32,u32)) {
        self.render_depth(encoder,queue,color,size,None);
    }

    pub fn radar_batch(&self) -> usize {
        self.radar_batch.unwrap_or(0)
    }

    pub fn render_depth(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        color: &wgpu::TextureView,
        size: (u32, u32),
        depth: Option<&wgpu::TextureView>,
    ) {
        self.render_part(encoder, queue, color, size, depth, 0..self.batches.len());
    }

    pub fn render_part(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        color: &wgpu::TextureView,
        size: (u32, u32),
        depth: Option<&wgpu::TextureView>,
        batches: Range<usize>,
    ) {
        let batches = batches.start.min(self.batches.len())..batches.end.min(self.batches.len());
        if batches.is_empty() {
            return;
        }
        queue.write_buffer(
            &self.screen_buffer,
            0,
            bytemuck::bytes_of(&ScreenUniform { size: [size.0 as f32, size.1 as f32], _pad: [0.0; 2] }),
        );
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ui"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                depth_slice: None,
            })],
            depth_stencil_attachment: depth.map(|view|wgpu::RenderPassDepthStencilAttachment {
                view, depth_ops: None, stencil_ops: None,
            }),
            ..Default::default()
        });
        let mut additive = false;
        pass.set_pipeline(if depth.is_some() {&self.depth_pipeline} else {&self.pipeline});
        pass.set_bind_group(0, &self.screen_group, &[]);
        pass.set_vertex_buffer(0, self.vertices.slice(..));
        pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint32);
        for batch in &self.batches[batches] {
            let Some(group) = self.bind_groups.get(&*batch.texture) else { continue };
            if let Some([x,y,w,h])=batch.clip {
                if w==0 || h==0 {continue;}
                pass.set_scissor_rect(x,y,w,h);
            }
            if batch.additive != additive {
                additive = batch.additive;
                let (plain, brighten) = if depth.is_some() { (&self.depth_pipeline, &self.additive_depth_pipeline) } else { (&self.pipeline, &self.additive_pipeline) };
                pass.set_pipeline(if additive { brighten } else { plain });
            }
            pass.set_bind_group(1, group, &[]);
            pass.draw_indexed(batch.indices.clone(), 0, 0..1);
        }
    }
}

fn quad_vertices(quad: &Quad, size: [f32; 2]) -> [UiVertex; 4] {
    let (u0, v0) = (quad.u / size[0], quad.v / size[1]);
    let (u1, v1) = ((quad.u + quad.uw) / size[0], (quad.v + quad.vh) / size[1]);
    let (x0, y0, x1, y1) = (quad.x, quad.y, quad.x + quad.w, quad.y + quad.h);
    let (cx, cy) = (quad.x + quad.w / 2.0, quad.y + quad.h / 2.0);
    let (sin, cos) = quad.rotation.sin_cos();
    let place = |x: f32, y: f32| {
        if quad.rotation == 0.0 {
            return [x, y];
        }
        let (dx, dy) = (x - cx, y - cy);
        [cx + dx * cos - dy * sin, cy + dx * sin + dy * cos]
    };
    let vertex = |position: [f32; 2], uv: [f32; 2]| UiVertex { position: [position[0],position[1],quad.depth], uv, color: quad.color };
    [
        vertex(place(x0, y0), [u0, v0]),
        vertex(place(x1, y0), [u1, v0]),
        vertex(place(x1, y1), [u1, v1]),
        vertex(place(x0, y1), [u0, v1]),
    ]
}

fn vertex_layout() -> wgpu::VertexBufferLayout<'static> {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x4];
    wgpu::VertexBufferLayout {
        array_stride: size_of::<UiVertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &ATTRIBUTES,
    }
}

fn growable(device: &wgpu::Device, usage: wgpu::BufferUsages, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ui"),
        size: size.max(4),
        usage: usage | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn write_growable(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    buffer: &mut wgpu::Buffer,
    usage: wgpu::BufferUsages,
    bytes: &[u8],
) {
    if bytes.is_empty() {
        return;
    }
    if buffer.size() < bytes.len() as u64 {
        *buffer = growable(device, usage, (bytes.len() as u64).next_power_of_two());
    }
    queue.write_buffer(buffer, 0, bytes);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::Camera;
    use crate::environment::{Environment, TIMEENV_FIXTURE};
    use crate::headless::{CLEAR_RGBA, Headless};
    use glam::vec3;
    use killer_assets::textures::{TextureData, TextureFormatKind};
    use killer_assets::timeenv::TimeEnv;
    use std::sync::Arc;

    struct Checker;

    fn solid(width: u32, height: u32, rgba: [u8; 4]) -> TextureData {
        TextureData {
            width,
            height,
            format: TextureFormatKind::Rgba8,
            mips: vec![rgba.repeat((width * height) as usize)],
        }
    }

    fn halves() -> TextureData {
        let mut rgba = Vec::new();
        for _ in 0..8 {
            for x in 0..8 {
                rgba.extend_from_slice(if x < 4 { &[255, 255, 255, 255] } else { &[0, 0, 0, 255] });
            }
        }
        TextureData { width: 8, height: 8, format: TextureFormatKind::Rgba8, mips: vec![rgba] }
    }

    impl TextureSource for Checker {
        fn texture(&mut self, path: &str) -> Arc<TextureData> {
            Arc::new(match path {
                "white" => solid(1, 1, [255, 255, 255, 255]),
                "checker8" => halves(),
                _ => solid(2, 2, [255, 0, 0, 255]),
            })
        }
    }

    fn camera() -> Camera {
        Camera { eye: vec3(0.0, -300.0, 0.0), target: vec3(0.0, 0.0, 0.0), fov_x_deg: 60.0, near: 1.0, far: 5000.0 }
    }

    fn env() -> Environment {
        Environment::from_time(&TimeEnv::parse(TIMEENV_FIXTURE), 12.0)
    }

    fn pixel(rgba: &[u8], x: usize, y: usize) -> [u8; 4] {
        let p = &rgba[(y * 64 + x) * 4..][..4];
        [p[0], p[1], p[2], p[3]]
    }

    fn covered(rgba: &[u8], x: usize, y: usize) -> bool {
        pixel(rgba, x, y) != CLEAR_RGBA
    }

    #[test]
    fn quad_vertices_map_texels_to_normalized_uv() {
        let quad = Quad {
            x: 16.0,
            y: 0.0,
            w: 268.0,
            h: 20.0,
            u: 0.0,
            v: 0.0,
            uw: 32.0,
            vh: 20.0,
            texture: "mid".into(),
            color: [1.0; 4],
            rotation: 0.0, depth: 0.0, additive: false,
        };
        let v = quad_vertices(&quad, [16.0, 32.0]);
        assert_eq!(v[0].position, [16.0, 0.0, 0.0]);
        assert_eq!(v[2].position, [284.0, 20.0, 0.0]);
        assert_eq!(v[0].uv, [0.0, 0.0]);
        assert_eq!(v[2].uv, [2.0, 0.625]);
        let turned = Quad { x: 0.0, y: 0.0, w: 10.0, h: 10.0, rotation: std::f32::consts::FRAC_PI_2, ..quad };
        let v = quad_vertices(&turned, [16.0, 32.0]);
        assert!((v[0].position[0] - 10.0).abs() < 1e-4 && v[0].position[1].abs() < 1e-4);
        assert!((v[1].position[0] - 10.0).abs() < 1e-4 && (v[1].position[1] - 10.0).abs() < 1e-4);
    }

    #[test]
    fn ui_releases_hidden_textures_and_preserves_live_views_when_the_scene_changes() {
        let mut headless = Headless::new(64, 64).unwrap();
        let mut list = DrawList::default();
        list.draw_texture(0, 0, 32, 32, 0, 0, 8, 8, "checker8", 1.0);
        headless.renderer_mut().set_ui(&list, &mut Checker);
        assert_eq!(headless.renderer_mut().texture_count(), 1);
        let expected = headless.render(&camera(), &env()).unwrap();
        headless.renderer_mut().clear();
        assert_eq!(headless.renderer_mut().texture_count(), 1);
        headless.renderer_mut().set_ui(&list, &mut Checker);
        assert_eq!(headless.render(&camera(), &env()).unwrap(), expected);
        headless.renderer_mut().set_ui(&DrawList::default(), &mut Checker);
        assert_eq!(headless.renderer_mut().texture_count(), 0);
        headless.renderer_mut().set_ui(&list, &mut Checker);
        assert_eq!(headless.render(&camera(), &env()).unwrap(), expected);
        headless.renderer_mut().clear_ui();
        assert_eq!(headless.renderer_mut().texture_count(), 0);
    }

    #[test]
    fn ui_quad_covers_exactly_its_pixel_rect() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let mut list = DrawList::default();
        list.draw_texture(10, 20, 30, 8, 0, 0, 30, 8, "white", 1.0);
        headless.renderer_mut().set_ui(&list, &mut Checker);
        let rgba = headless.render(&camera(), &env()).unwrap();
        assert!(covered(&rgba, 10, 20) && covered(&rgba, 39, 27));
        assert!(!covered(&rgba, 9, 20) && !covered(&rgba, 40, 20) && !covered(&rgba, 10, 28));
    }

    #[test]
    fn ui_alpha_blends_over_the_world() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let mut list = DrawList::default();
        list.draw_texture(0, 0, 64, 64, 0, 0, 64, 64, "white", 0.5);
        headless.renderer_mut().set_ui(&list, &mut Checker);
        let rgba = headless.render(&camera(), &env()).unwrap();
        let p = pixel(&rgba, 32, 32);
        assert!(p[0] > CLEAR_RGBA[0] && p[0] < 255, "{p:?}");
    }

    #[test]
    fn a_brighten_quad_adds_its_tint_over_the_pixels_below_instead_of_covering_them() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let mut list = DrawList::default();
        list.draw_texture(0, 0, 64, 64, 0, 0, 64, 64, "white", 0.5);
        headless.renderer_mut().set_ui(&list, &mut Checker);
        let base = pixel(&headless.render(&camera(), &env()).unwrap(), 32, 32);
        list.draw_brighten_texture(0, 0, 64, 64, "white", killer_ui::color::Argb(0xFF008080));
        headless.renderer_mut().set_ui(&list, &mut Checker);
        let lit = pixel(&headless.render(&camera(), &env()).unwrap(), 32, 32);
        assert_eq!(lit[0], base[0], "o vermelho de 0xFF008080 é zero: nada somado");
        assert!(lit[1] > base[1] && lit[2] > base[2], "{base:?} -> {lit:?}");
        let batches: Vec<bool> = headless.renderer_mut().ui.batches.iter().map(|b| b.additive).collect();
        assert_eq!(batches, [false, true]);
    }

    #[test]
    fn stretched_middle_repeats_the_texture() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let mut list = DrawList::default();
        list.draw_texture(0, 0, 64, 8, 0, 0, 64, 8, "checker8", 1.0);
        headless.renderer_mut().set_ui(&list, &mut Checker);
        let rgba = headless.render(&camera(), &env()).unwrap();
        assert_ne!(pixel(&rgba, 2, 2), pixel(&rgba, 6, 2));
        assert_eq!(pixel(&rgba, 2, 2), pixel(&rgba, 10, 2));
    }

    #[test]
    fn consecutive_quads_of_one_texture_form_a_single_batch() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let mut list = DrawList::default();
        list.draw_texture(0, 0, 8, 8, 0, 0, 8, 8, "white", 1.0);
        list.draw_texture(8, 0, 8, 8, 0, 0, 8, 8, "White", 1.0);
        list.draw_texture(16, 0, 8, 8, 0, 0, 8, 8, "checker8", 1.0);
        list.draw_texture(24, 0, 8, 8, 0, 0, 8, 8, "white", 1.0);
        headless.renderer_mut().set_ui(&list, &mut Checker);
        let batches: Vec<(String, Range<u32>)> =
            headless.renderer_mut().ui.batches.iter().map(|b| (b.texture.to_string(), b.indices.clone())).collect();
        assert_eq!(
            batches,
            vec![("white".to_string(), 0..12), ("checker8".to_string(), 12..18), ("white".to_string(), 18..24)]
        );
    }

    #[test]
    #[ignore]
    fn ui_upload_cost_for_text_runs() {
        use std::hash::{Hash, Hasher};
        let mut headless = Headless::new(64, 64).unwrap();
        for run in [1, 64, 10000] {
            let mut list = DrawList::default();
            for index in 0..10000 {
                let texture = if (index / run) % 2 == 0 { "white" } else { "checker8" };
                list.draw_texture(index % 64, index / 64 % 64, 4, 4, 0, 0, 1, 1, texture, 0.5);
            }
            let mut samples = Vec::new();
            for sample in 0..8 {
                let start = std::time::Instant::now();
                for _ in 0..20 {
                    let renderer = headless.renderer_mut();
                    renderer.set_ui(&list, &mut Checker);
                    renderer.queue().submit([]);
                    renderer.device().poll(wgpu::PollType::wait_indefinitely()).unwrap();
                }
                if sample > 0 { samples.push(start.elapsed().as_secs_f64() * 1000.0 / 20.0); }
            }
            samples.sort_by(f64::total_cmp);
            let mut hash = std::collections::hash_map::DefaultHasher::new();
            headless.render(&camera(), &env()).unwrap().hash(&mut hash);
            eprintln!("UI 10000 quads, run={run}, median {:.3} ms/upload frame, pixels {:016x}", samples[3], hash.finish());
        }
    }

    #[test]
    fn reused_ui_geometry_matches_fresh_buffers_after_growth_shrink_and_clear() {
        let mut reused = Headless::new(64, 64).unwrap();
        for count in [0, 20, 1, 0, 80, 3] {
            let mut list = DrawList::default();
            for index in 0..count {
                let texture = if index % 3 == 0 { "checker8" } else { "white" };
                list.draw_texture(index % 8 * 8, index / 8 * 6, 8, 6, 0, 0, 8, 8, texture, 0.5);
            }
            reused.renderer_mut().set_ui(&list, &mut Checker);
            let mut fresh = Headless::new(64, 64).unwrap();
            fresh.renderer_mut().set_ui(&list, &mut Checker);
            assert_eq!(reused.render(&camera(), &env()).unwrap(), fresh.render(&camera(), &env()).unwrap(), "{count} quads");
            if count == 0 { reused.renderer_mut().clear_ui(); }
        }
    }

    #[test]
    fn an_empty_list_leaves_the_world_untouched() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        headless.renderer_mut().set_ui(&DrawList::default(), &mut Checker);
        let rgba = headless.render(&camera(), &env()).unwrap();
        assert_eq!(pixel(&rgba, 32, 32), CLEAR_RGBA);
    }
}
