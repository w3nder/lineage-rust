use super::{GpuCharacter, ObjectBuffers, Renderer, object_buffers};
use crate::environment::EnvironmentUniform;
use crate::pipeline::{CameraUniform, DEPTH_FORMAT};
use crate::radar::{RadarClear, RadarMesh, RadarStep, actor_matrix, projection, view_matrix, viewport};
use crate::skinned::ObjectUniform;
use crate::textures::TextureSource;
use glam::Mat4;
use killer_assets::characters::CharacterMesh;

const SHADER: &str = include_str!("../radar_clear.wgsl");

struct ClearPipelines {
    format: wgpu::TextureFormat,
    samples: u32,
    depth: wgpu::RenderPipeline,
    all: wgpu::RenderPipeline,
}

struct RadarDraw {
    mesh: usize,
    object: u32,
    camera: usize,
}

pub(super) struct RadarGpu {
    pub(super) meshes: Vec<Option<GpuCharacter>>,
    locals: Vec<Mat4>,
    loaded: bool,
    steps: Vec<RadarStep>,
    objects: Option<ObjectBuffers>,
    cameras: Vec<(wgpu::Buffer, wgpu::BindGroup)>,
    environment: wgpu::Buffer,
    clears: Option<ClearPipelines>,
}

impl RadarGpu {
    pub(super) fn new(device: &wgpu::Device) -> Self {
        let environment = device.create_buffer(&wgpu::BufferDescriptor { label: Some("ambiente do radar"), size: size_of::<EnvironmentUniform>() as u64, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        Self { meshes: (0..RadarMesh::ALL.len()).map(|_| None).collect(), locals: vec![Mat4::IDENTITY; RadarMesh::ALL.len()], loaded: false, steps: Vec::new(), objects: None, cameras: Vec::new(), environment, clears: None }
    }
}

fn clear_pipelines(device: &wgpu::Device, format: wgpu::TextureFormat, samples: u32) -> ClearPipelines {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("limpeza do radar"), source: wgpu::ShaderSource::Wgsl(SHADER.into()) });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("limpeza do radar"), bind_group_layouts: &[], immediate_size: 0 });
    let pipeline = |write_mask: wgpu::ColorWrites| {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("limpeza do radar"),
            layout: Some(&layout),
            vertex: wgpu::VertexState { module: &module, entry_point: Some("vs_main"), buffers: &[], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState { module: &module, entry_point: Some("fs_main"), targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask })], compilation_options: Default::default() }),
            primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState { format: DEPTH_FORMAT, depth_write_enabled: Some(true), depth_compare: Some(wgpu::CompareFunction::Always), stencil: Default::default(), bias: Default::default() }),
            multisample: wgpu::MultisampleState { count: samples, ..Default::default() },
            multiview_mask: None,
            cache: None,
        })
    };
    ClearPipelines { format, samples, depth: pipeline(wgpu::ColorWrites::empty()), all: pipeline(wgpu::ColorWrites::ALL) }
}

impl Renderer {
    pub fn radar_meshes_loaded(&self) -> bool {
        self.radar.loaded
    }

    pub fn radar_mesh_ready(&self, mesh: RadarMesh) -> bool {
        self.radar.meshes.get(mesh.index()).is_some_and(Option::is_some)
    }

    pub fn set_radar_meshes(&mut self, meshes: &[(RadarMesh, CharacterMesh)], textures: &mut dyn TextureSource) {
        for (mesh, source) in meshes {
            let mut unlit = source.clone();
            for section in &mut unlit.sections {
                for pass in &mut section.material.passes {
                    pass.lit = false;
                    pass.two_sided = true;
                }
            }
            let character = self.gpu_character(&unlit, textures);
            self.radar.meshes[mesh.index()] = Some(character);
            self.radar.locals[mesh.index()] = unlit.local;
        }
        self.radar.loaded = true;
    }

    pub fn set_radar(&mut self, steps: Vec<RadarStep>) {
        self.radar.steps = steps;
    }

    pub(super) fn render_radar(&mut self, encoder: &mut wgpu::CommandEncoder, color: &wgpu::TextureView, depth: &wgpu::TextureView, size: (u32, u32), environment: &EnvironmentUniform) {
        if self.radar.steps.is_empty() || size.0 == 0 || size.1 == 0 {
            return;
        }
        let screen = self.ui_size.filter(|ui| ui.0 > 0 && ui.1 > 0).unwrap_or(size);
        let aspect = screen.0 as f32 / screen.1 as f32;
        let proj = projection(aspect);
        let scale = (size.0 as f32 / screen.0 as f32, size.1 as f32 / screen.1 as f32);
        let [x, y, w, h] = viewport(screen);
        let rect = [x as f32 * scale.0, y as f32 * scale.1, w as f32 * scale.0, h as f32 * scale.1];
        let rect = [rect[0].min(size.0 as f32), rect[1].min(size.1 as f32), rect[2].min(size.0 as f32 - rect[0].min(size.0 as f32)), rect[3].min(size.1 as f32 - rect[1].min(size.1 as f32))];
        if rect[2] <= 0.0 || rect[3] <= 0.0 {
            return;
        }
        let mut quiet = *environment;
        quiet.fog_color[3] = 0.0;
        quiet.fog_range = [0.0; 4];
        self.queue.write_buffer(&self.radar.environment, 0, bytemuck::bytes_of(&quiet));
        let stride = self.object_buffers.stride;
        let mut objects = Vec::new();
        let mut draws = Vec::new();
        let mut cameras: Vec<CameraUniform> = Vec::new();
        for step in &self.radar.steps {
            let RadarStep::Draw { mesh, location, rotation, camera } = *step else { continue };
            if self.radar.meshes[mesh.index()].is_none() {
                continue;
            }
            let world = actor_matrix(location, rotation) * self.radar.locals[mesh.index()];
            let uniform = ObjectUniform::new(world, 0, 1, 0, false, &[]);
            let offset = objects.len();
            objects.extend_from_slice(bytemuck::bytes_of(&uniform));
            objects.resize(offset + stride as usize, 0);
            let view = view_matrix(&camera);
            cameras.push(CameraUniform { view_proj: (proj * view).to_cols_array_2d(), view: view.to_cols_array_2d(), eye: camera.location.extend(1.0).to_array() });
            draws.push(RadarDraw { mesh: mesh.index(), object: offset as u32, camera: cameras.len() - 1 });
        }
        let slots = (objects.len() as u64 / stride).max(1);
        if self.radar.objects.as_ref().is_none_or(|buffers| buffers.objects.size() < slots * stride) {
            self.radar.objects = Some(object_buffers(&self.device, &self.pipelines.object_layout, stride, slots.next_power_of_two(), 1));
        }
        while self.radar.cameras.len() < cameras.len() {
            let buffer = self.device.create_buffer(&wgpu::BufferDescriptor { label: Some("câmera do radar"), size: size_of::<CameraUniform>() as u64, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
            let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("câmera do radar"),
                layout: &self.pipelines.group0_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: self.radar.environment.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: self.zone_buffer.as_entire_binding() },
                ],
            });
            self.radar.cameras.push((buffer, group));
        }
        for (uniform, (buffer, _)) in cameras.iter().zip(&self.radar.cameras) {
            self.queue.write_buffer(buffer, 0, bytemuck::bytes_of(uniform));
        }
        let format = self.pipelines.color_format();
        let samples = self.pipelines.sample_count();
        if self.radar.clears.as_ref().is_none_or(|clears| clears.format != format || clears.samples != samples) {
            self.radar.clears = Some(clear_pipelines(&self.device, format, samples));
        }
        let (Some(buffers), Some(clears)) = (self.radar.objects.as_ref(), self.radar.clears.as_ref()) else { return };
        if !objects.is_empty() {
            self.queue.write_buffer(&buffers.objects, 0, &objects);
        }
        self.queue.write_buffer(&buffers.palette, 0, bytemuck::cast_slice(&[Mat4::IDENTITY.to_cols_array_2d()]));
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("radar"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: color, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }, depth_slice: None })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { view: depth, depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }), stencil_ops: None }),
            ..Default::default()
        });
        pass.set_viewport(rect[0], rect[1], rect[2], rect[3], 0.0, 1.0);
        pass.set_scissor_rect(rect[0] as u32, rect[1] as u32, (rect[2] as u32).max(1), (rect[3] as u32).max(1));
        let mut next = draws.iter();
        for step in &self.radar.steps {
            match step {
                RadarStep::Clear(clear) => {
                    pass.set_pipeline(match clear { RadarClear::Depth => &clears.depth, RadarClear::All => &clears.all });
                    pass.draw(0..3, 0..1);
                }
                RadarStep::Draw { mesh, .. } => {
                    if self.radar.meshes[mesh.index()].is_none() {
                        continue;
                    }
                    let Some(draw) = next.next() else { break };
                    let Some(character) = self.radar.meshes[draw.mesh].as_ref() else { continue };
                    let Some((_, group)) = self.radar.cameras.get(draw.camera) else { continue };
                    pass.set_bind_group(0, group, &[]);
                    pass.set_bind_group(2, &buffers.bind_group, &[draw.object]);
                    pass.set_vertex_buffer(0, character.vertices.slice(..));
                    pass.set_index_buffer(character.indices.slice(..), wgpu::IndexFormat::Uint32);
                    for section in &character.sections {
                        for gpu_pass in &section.material.passes {
                            let Some(pipeline) = self.pipelines.skinned(&gpu_pass.key) else { continue };
                            self.textures.evaluate_material(&self.queue, gpu_pass.slot, f64::from(self.time));
                            pass.set_pipeline(pipeline);
                            pass.set_bind_group(1, &gpu_pass.bind_group, &[0]);
                            pass.draw_indexed(section.first_index..section.first_index + section.index_count, 0, 0..1);
                        }
                    }
                }
            }
        }
    }
}
