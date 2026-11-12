use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use std::mem::size_of;
use wgpu::util::DeviceExt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line {
    pub start: Vec3,
    pub end: Vec3,
    pub color: [u8; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug, PartialEq)]
pub struct LineVertex {
    pub position: [f32; 3],
    pub color: [u8; 4],
}

pub fn vertices(lines: &[Line]) -> Vec<LineVertex> {
    lines
        .iter()
        .flat_map(|line| [line.start, line.end].map(|point| LineVertex { position: point.to_array(), color: line.color }))
        .collect()
}

pub struct LineRenderer {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::PipelineLayout,
    shader: wgpu::ShaderModule,
    view_buffer: wgpu::Buffer,
    view_group: wgpu::BindGroup,
    vertices: wgpu::Buffer,
    count: u32,
}

impl LineRenderer {
    pub fn new(device: &wgpu::Device, color_format: wgpu::TextureFormat, sample_count: u32) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("linhas"),
            source: wgpu::ShaderSource::Wgsl(include_str!("lines.wgsl").into()),
        });
        let view_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("linhas vista"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("linhas"),
            bind_group_layouts: &[Some(&view_layout)],
            immediate_size: 0,
        });
        let view_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("linhas vista"),
            contents: bytemuck::cast_slice(&Mat4::IDENTITY.to_cols_array()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let view_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("linhas vista"),
            layout: &view_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: view_buffer.as_entire_binding() }],
        });
        let pipeline = pipeline(device, &layout, &shader, color_format, sample_count);
        let vertices = vertex_buffer(device, 2);
        Self { pipeline, layout, shader, view_buffer, view_group, vertices, count: 0 }
    }

    pub fn set_sample_count(&mut self, device: &wgpu::Device, color_format: wgpu::TextureFormat, sample_count: u32) {
        self.pipeline = pipeline(device, &self.layout, &self.shader, color_format, sample_count);
    }

    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, lines: &[Line]) {
        let vertices = vertices(lines);
        self.count = vertices.len() as u32;
        if vertices.is_empty() {
            return;
        }
        let bytes: &[u8] = bytemuck::cast_slice(&vertices);
        if self.vertices.size() < bytes.len() as u64 {
            self.vertices = vertex_buffer(device, vertices.len().next_power_of_two());
        }
        queue.write_buffer(&self.vertices, 0, bytes);
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, queue: &wgpu::Queue, view_proj: Mat4) {
        if self.count == 0 {
            return;
        }
        queue.write_buffer(&self.view_buffer, 0, bytemuck::cast_slice(&view_proj.to_cols_array()));
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.view_group, &[]);
        pass.set_vertex_buffer(0, self.vertices.slice(..));
        pass.draw(0..self.count, 0..1);
    }
}

fn vertex_buffer(device: &wgpu::Device, count: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("linhas"),
        size: (size_of::<LineVertex>() * count.max(2)) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    color_format: wgpu::TextureFormat,
    sample_count: u32,
) -> wgpu::RenderPipeline {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Unorm8x4];
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("linhas"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: size_of::<LineVertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &ATTRIBUTES,
            })],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState { format: color_format, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::LineList, cull_mode: None, ..Default::default() },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: crate::pipeline::DEPTH_FORMAT,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: wgpu::MultisampleState { count: sample_count, ..Default::default() },
        multiview_mask: None,
        cache: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_line_becomes_a_pair_of_vertices_with_its_color() {
        let lines = [
            Line { start: Vec3::new(1.0, 2.0, 3.0), end: Vec3::new(4.0, 5.0, 6.0), color: [255, 0, 0, 255] },
            Line { start: Vec3::ZERO, end: Vec3::X, color: [0, 255, 0, 0] },
        ];
        let vertices = vertices(&lines);
        assert_eq!(vertices.len(), 4);
        assert_eq!(vertices[0], LineVertex { position: [1.0, 2.0, 3.0], color: [255, 0, 0, 255] });
        assert_eq!(vertices[1].position, [4.0, 5.0, 6.0]);
        assert_eq!(vertices[3], LineVertex { position: [1.0, 0.0, 0.0], color: [0, 255, 0, 0] });
        assert_eq!(size_of::<LineVertex>(), 16);
    }
}
