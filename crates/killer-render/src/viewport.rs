use crate::pipeline::{COLOR_FORMAT, DEPTH_FORMAT};
use crate::video::MultisampleFrame;
use std::sync::Arc;

pub struct ViewportTarget {
    pub(crate) size: (u32, u32),
    pub(crate) color_view: Arc<wgpu::TextureView>,
    pub(crate) depth_view: wgpu::TextureView,
    pub(crate) multisample: Option<MultisampleFrame>,
    texture: Arc<wgpu::Texture>,
}

impl ViewportTarget {
    pub(crate) fn new(device: &wgpu::Device, size: (u32, u32)) -> Self {
        let size = (size.0.max(1), size.1.max(1));
        let make = |format, usage| device.create_texture(&wgpu::TextureDescriptor {
            label: Some("viewport"),
            size: wgpu::Extent3d { width: size.0, height: size.1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });
        let color = make(COLOR_FORMAT, wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC);
        let depth = make(DEPTH_FORMAT, wgpu::TextureUsages::RENDER_ATTACHMENT);
        Self { size, color_view: Arc::new(color.create_view(&Default::default())), depth_view: depth.create_view(&Default::default()), multisample: None, texture: Arc::new(color) }
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn view(&self) -> Arc<wgpu::TextureView> {
        self.color_view.clone()
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }
}
