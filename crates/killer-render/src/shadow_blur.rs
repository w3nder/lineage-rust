use bytemuck::{Pod, Zeroable};

pub const WEIGHTS: [f32; 9] = [1.0, 1.0, 1.0, 1.0, 1.5, 1.0, 1.0, 1.0, 1.0];
pub const GAIN: f32 = 0.303_030_3;
pub const STEP: f32 = 1.5;
pub const TAP_STRIDE: u64 = 256;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug, PartialEq)]
pub struct Tap {
    pub origin: [f32; 2],
    pub extent: [f32; 2],
    pub size: f32,
    pub weight: f32,
    pub pad: [f32; 2],
}

pub fn total() -> f32 {
    let mut sum = 0.0f32;
    for column in 0..3 {
        for row in 0..3 {
            sum += WEIGHTS[column + row * 3] * GAIN;
        }
    }
    sum
}

pub fn taps(size: f32) -> Vec<Tap> {
    let total = total();
    let mut taps = Vec::with_capacity(9);
    for a in 0..3 {
        for b in 0..3 {
            let (a, b) = (a as f32, b as f32);
            taps.push(Tap {
                origin: [STEP - b * STEP, STEP - a * STEP],
                extent: [(size - b * STEP) + STEP, (size - a * STEP) + STEP],
                size,
                weight: WEIGHTS[(a + b * 3.0) as usize] / total,
                pad: [0.0; 2],
            });
        }
    }
    taps
}

pub struct ShadowBlur {
    pub pipeline: wgpu::RenderPipeline,
    pub layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
    pub uniforms: wgpu::Buffer,
}

impl ShadowBlur {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat, size: u32) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("desfoque da sombra"), source: wgpu::ShaderSource::Wgsl(include_str!("shadow_blur.wgsl").into()) });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("desfoque da sombra"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2Array, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("desfoque da sombra"), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
        let additive = wgpu::BlendComponent { src_factor: wgpu::BlendFactor::SrcAlpha, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add };
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("desfoque da sombra"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: Some("vs_main"), buffers: &[], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: Some("fs_main"), targets: &[Some(wgpu::ColorTargetState { format, blend: Some(wgpu::BlendState { color: additive, alpha: additive }), write_mask: wgpu::ColorWrites::ALL })], compilation_options: Default::default() }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor { mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, address_mode_u: wgpu::AddressMode::ClampToEdge, address_mode_v: wgpu::AddressMode::ClampToEdge, ..Default::default() });
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor { label: Some("desfoque da sombra"), size: TAP_STRIDE * 9, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        for (index, tap) in taps(size as f32).iter().enumerate() {
            queue.write_buffer(&uniforms, TAP_STRIDE * index as u64, bytemuck::bytes_of(tap));
        }
        Self { pipeline, layout, sampler, uniforms }
    }

    pub fn group(&self, device: &wgpu::Device, source: &wgpu::TextureView) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("desfoque da sombra"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: &self.uniforms, offset: 0, size: std::num::NonZeroU64::new(size_of::<Tap>() as u64) }) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(source) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.sampler) },
            ],
        })
    }

    pub fn draw(&self, encoder: &mut wgpu::CommandEncoder, group: &wgpu::BindGroup, target: &wgpu::TextureView) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("desfoque da sombra"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: target, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store }, depth_slice: None })],
            ..Default::default()
        });
        pass.set_pipeline(&self.pipeline);
        for index in 0..9u32 {
            pass.set_bind_group(0, group, &[index * TAP_STRIDE as u32]);
            pass.draw(0..3, 0..1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).ok()?;
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()
    }

    #[test]
    fn the_blur_pass_softens_the_edge_and_saturates_the_inside_like_the_c4_tiles() {
        let Some((device, queue)) = device() else { return };
        let size = 64u32;
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let blur = ShadowBlur::new(&device, &queue, format, size);
        let make = |usage| device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: size, height: size, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });
        let source = make(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST);
        let target = make(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC);
        let mut rgba = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size / 2 {
                rgba[((y * size + x) * 4 + 3) as usize] = 255;
            }
        }
        queue.write_texture(source.as_image_copy(), &rgba, wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(size * 4), rows_per_image: None }, wgpu::Extent3d { width: size, height: size, depth_or_array_layers: 1 });
        let source_view = source.create_view(&wgpu::TextureViewDescriptor { dimension: Some(wgpu::TextureViewDimension::D2Array), array_layer_count: Some(1), ..Default::default() });
        let group = blur.group(&device, &source_view);
        let mut encoder = device.create_command_encoder(&Default::default());
        blur.draw(&mut encoder, &group, &target.create_view(&Default::default()));
        let readback = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: 256 * u64::from(size), usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
        encoder.copy_texture_to_buffer(target.as_image_copy(), wgpu::TexelCopyBufferInfo { buffer: &readback, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(256), rows_per_image: None } }, wgpu::Extent3d { width: size, height: size, depth_or_array_layers: 1 });
        queue.submit([encoder.finish()]);
        let slice = readback.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| ());
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let data = slice.get_mapped_range().unwrap();
        let alpha = |x: usize| data[32 * 256 + x * 4 + 3];
        assert_eq!(alpha(8), 255);
        assert_eq!(alpha(56), 0);
        let edge: Vec<u8> = (28..37).map(alpha).collect();
        assert!(edge.windows(2).all(|pair| pair[0] >= pair[1]), "{edge:?}");
        assert!(edge.iter().any(|&a| a > 0 && a < 255), "{edge:?}");
        assert_eq!(data[32 * 256 + 8 * 4], 0);
    }

    #[test]
    fn the_nine_taps_follow_get_10302d92_and_add_up_to_3_3() {
        let taps = taps(256.0);
        assert_eq!(taps.len(), 9);
        let total: f32 = taps.iter().map(|tap| tap.weight).sum();
        assert!((total - 3.3).abs() < 1.0e-4, "{total}");
        assert!((super::total() - 2.878_788).abs() < 1.0e-5);
        assert_eq!(taps[0].origin, [1.5, 1.5]);
        assert_eq!(taps[0].extent, [257.5, 257.5]);
        assert_eq!(taps[1].origin, [0.0, 1.5]);
        assert_eq!(taps[2].origin, [-1.5, 1.5]);
        assert_eq!(taps[2].extent, [254.5, 257.5]);
        assert_eq!(taps[8].origin, [-1.5, -1.5]);
        let center = taps.iter().max_by(|a, b| a.weight.total_cmp(&b.weight)).unwrap();
        assert_eq!(center.origin, [0.0, 0.0]);
        assert!((center.weight - 1.5 / super::total()).abs() < 1.0e-6);
    }
}
