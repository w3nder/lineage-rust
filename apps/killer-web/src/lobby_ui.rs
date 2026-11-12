use killer_render::renderer::Renderer;

pub struct LobbyUi {
    pub context: egui::Context,
    painter: egui_wgpu::Renderer,
    pub input: crate::lobby_input::Input,
}

impl LobbyUi {
    pub fn new(renderer: &Renderer, format: wgpu::TextureFormat) -> Self {
        Self {
            context: egui::Context::default(),
            painter: egui_wgpu::Renderer::new(renderer.device(), format, Default::default()),
            input: Default::default(),
        }
    }

    pub fn begin(&mut self, size: (u32, u32), scale: f32, time: f64) {
        self.context.set_pixels_per_point(scale);
        self.context.begin_pass(self.input.take(size, scale, time));
    }

    pub fn paint(
        &mut self,
        renderer: &Renderer,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        size: (u32, u32),
    ) -> Vec<wgpu::CommandBuffer> {
        let output = self.context.end_pass();
        for (id, deltas) in &output.textures_delta.set {
            for delta in deltas {
                self.painter.update_texture(renderer.device(), renderer.queue(), *id, delta);
            }
        }
        let jobs = self.context.tessellate(output.shapes, output.pixels_per_point);
        let screen =
            egui_wgpu::ScreenDescriptor { size_in_pixels: [size.0, size.1], pixels_per_point: output.pixels_per_point };
        let commands = self.painter.update_buffers(renderer.device(), renderer.queue(), encoder, &jobs, &screen);
        {
            let mut pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("web-lobby"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                        depth_slice: None,
                    })],
                    ..Default::default()
                })
                .forget_lifetime();
            self.painter.render(&mut pass, &jobs, &screen);
        }
        for id in &output.textures_delta.free {
            self.painter.free_texture(id);
        }
        commands
    }
}
