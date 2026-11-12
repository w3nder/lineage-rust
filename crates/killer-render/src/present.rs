use crate::pipeline::COLOR_FORMAT;
use crate::video::GammaSettings;
use wgpu::util::DeviceExt;

pub struct Present {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    frame: Option<Frame>,
    gamma: wgpu::Buffer,
    flash: wgpu::Buffer,
}

struct Frame {
    view: wgpu::TextureView,
    group: wgpu::BindGroup,
    size: (u32, u32),
}

impl Present {
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("present"), source: wgpu::ShaderSource::Wgsl(include_str!("present.wgsl").into()) });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("present"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("present"), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
        let entry = if target_format.is_srgb() { "fs_srgb" } else { "fs_copy" };
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("present"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), buffers: &[], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some(entry),
                targets: &[Some(wgpu::ColorTargetState { format: target_format, blend: None, write_mask: wgpu::ColorWrites::ALL })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, cull_mode: None, ..Default::default() },
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor { mag_filter: wgpu::FilterMode::Nearest, min_filter: wgpu::FilterMode::Nearest, ..Default::default() });
        let identity: [[f32; 4]; 256] = std::array::from_fn(|index| [index as f32 / 255.0; 4]);
        let gamma = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("gamma ramp"), contents: bytemuck::cast_slice(&identity), usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST });
        let neutral: [[f32; 4]; 2] = [[1.0, 0.0, 0.0, 0.0], [0.0; 4]];
        let flash = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("flash"), contents: bytemuck::cast_slice(&neutral), usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST });
        Self { pipeline, layout, sampler, frame: None, gamma, flash }
    }

    pub fn set_gamma(&self, queue: &wgpu::Queue, settings: GammaSettings) {
        let values = settings.ramp().map(|value| [f32::from(value) / 65535.0; 4]);
        queue.write_buffer(&self.gamma, 0, bytemuck::cast_slice(&values));
    }

    pub fn set_flash(&self, queue: &wgpu::Queue, scale: f32, fog: [f32; 3]) {
        let values: [[f32; 4]; 2] = [[scale, 0.0, 0.0, 0.0], [fog[0], fog[1], fog[2], 0.0]];
        queue.write_buffer(&self.flash, 0, bytemuck::cast_slice(&values));
    }

    pub fn frame_view(&mut self, device: &wgpu::Device, size: (u32, u32)) -> wgpu::TextureView {
        match &self.frame {
            Some(frame) if frame.size == size => frame.view.clone(),
            _ => {
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("quadro"),
                    size: wgpu::Extent3d { width: size.0.max(1), height: size.1.max(1), depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: COLOR_FORMAT,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });
                let view = texture.create_view(&Default::default());
                let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("present"),
                    layout: &self.layout,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                        wgpu::BindGroupEntry { binding: 2, resource: self.gamma.as_entire_binding() },
                        wgpu::BindGroupEntry { binding: 3, resource: self.flash.as_entire_binding() },
                    ],
                });
                self.frame = Some(Frame { view: view.clone(), group, size });
                view
            }
        }
    }

    pub fn blit(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        let Some(frame) = &self.frame else { return };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("present"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: target, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store }, depth_slice: None })],
            depth_stencil_attachment: None,
            ..Default::default()
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &frame.group, &[]);
        pass.draw(0..3, 0..1);
    }
}
