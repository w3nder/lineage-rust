use crate::camera::Camera;
use crate::environment::Environment;
use crate::pipeline::{COLOR_FORMAT, DEPTH_FORMAT};
use crate::renderer::Renderer;
use crate::textures::TextureSource;
use killer_assets::scene::Scene;
use thiserror::Error;

pub const CLEAR_RGBA: [u8; 4] = [0, 0, 0, 255];

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("nenhum adaptador de GPU")]
    NoAdapter,
    #[error("device: {0}")]
    Device(String),
    #[error("leitura de volta: {0}")]
    Readback(String),
}

pub struct Headless {
    pub(crate) renderer: Renderer,
    color: wgpu::Texture,
    depth: wgpu::Texture,
    width: u32,
    height: u32,
}

impl Headless {
    pub fn new(width: u32, height: u32) -> Result<Self, RenderError> {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).map_err(|_| RenderError::NoAdapter)?;
        let required_features = adapter.features() & wgpu::Features::TEXTURE_COMPRESSION_BC;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor { required_features, ..Default::default() })).map_err(|e| RenderError::Device(e.to_string()))?;
        let color = target(&device, width, height, COLOR_FORMAT, wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC);
        let depth = target(&device, width, height, DEPTH_FORMAT, wgpu::TextureUsages::RENDER_ATTACHMENT);
        let renderer = Renderer::new(device, queue, COLOR_FORMAT);
        Ok(Self { renderer, color, depth, width, height })
    }

    pub fn renderer_mut(&mut self) -> &mut Renderer {
        &mut self.renderer
    }

    pub fn upload(&mut self, scene: &Scene, textures: &mut dyn TextureSource) {
        self.renderer.upload(scene, textures);
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn targets(&self) -> (wgpu::TextureView, wgpu::TextureView) {
        (self.color.create_view(&Default::default()), self.depth.create_view(&Default::default()))
    }

    pub fn render(&mut self, camera: &Camera, environment: &Environment) -> Result<Vec<u8>, RenderError> {
        let color_view = self.color.create_view(&Default::default());
        let depth_view = self.depth.create_view(&Default::default());
        let mut encoder = self.renderer.device().create_command_encoder(&Default::default());
        self.renderer.render(&mut encoder, &color_view, &depth_view, (self.width, self.height), camera, environment);
        let bytes_per_row = (self.width * 4).div_ceil(256) * 256;
        let readback = self.renderer.device().create_buffer(&wgpu::BufferDescriptor { label: None, size: (bytes_per_row * self.height) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
        encoder.copy_texture_to_buffer(
            self.color.as_image_copy(),
            wgpu::TexelCopyBufferInfo { buffer: &readback, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(bytes_per_row), rows_per_image: None } },
            wgpu::Extent3d { width: self.width, height: self.height, depth_or_array_layers: 1 },
        );
        self.renderer.queue().submit([encoder.finish()]);
        let slice = readback.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.renderer.device().poll(wgpu::PollType::wait_indefinitely()).map_err(|e| RenderError::Readback(format!("{e:?}")))?;
        rx.recv().map_err(|e| RenderError::Readback(e.to_string()))?.map_err(|e| RenderError::Readback(e.to_string()))?;
        let data = slice.get_mapped_range().map_err(|e| RenderError::Readback(e.to_string()))?;
        let mut rgba = Vec::with_capacity((self.width * self.height * 4) as usize);
        for row in 0..self.height as usize {
            let start = row * bytes_per_row as usize;
            rgba.extend_from_slice(&data[start..start + (self.width * 4) as usize]);
        }
        drop(data);
        readback.unmap();
        Ok(rgba)
    }
}

fn target(device: &wgpu::Device, width: u32, height: u32, format: wgpu::TextureFormat, usage: wgpu::TextureUsages) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::TIMEENV_FIXTURE;
    use glam::{Vec3, vec3};
    use killer_assets::characters::CharacterMesh;
    use crate::renderer::Viewer;
    use killer_assets::scene::{ActorCull, DrawKind, FogParams, GlobalFog, LodSelect, MeshDraw, Pass, SectorBounds, Vertex, ZoneLighting, texture_material};
    use l2_catalog::{BlendFactor, Material, UvModifier};
    use killer_assets::textures::{TextureData, TextureFormatKind, checker_image};
    use killer_assets::timeenv::TimeEnv;
    use std::sync::Arc;

    struct Checker;

    impl TextureSource for Checker {
        fn texture(&mut self, _: &str) -> Arc<TextureData> {
            static DATA: std::sync::OnceLock<Arc<TextureData>> = std::sync::OnceLock::new();
            DATA.get_or_init(|| {
                Arc::new(TextureData { width: 8, height: 8, format: TextureFormatKind::Rgba8, mips: vec![checker_image().rgba] })
            })
            .clone()
        }

        fn alpha_map(&mut self, _: &str) -> Arc<TextureData> {
            let mut rgba = checker_image().rgba;
            for pixel in rgba.as_chunks_mut::<4>().0 {
                pixel[3] = pixel[0];
            }
            Arc::new(TextureData { width: 8, height: 8, format: TextureFormatKind::Rgba8, mips: vec![rgba] })
        }
    }

    struct White;

    impl TextureSource for White {
        fn texture(&mut self, _: &str) -> Arc<TextureData> {
            Arc::new(TextureData { width: 4, height: 4, format: TextureFormatKind::Rgba8, mips: vec![[255u8; 4].repeat(16)] })
        }
    }

    struct SyntheticMip;

    impl TextureSource for SyntheticMip {
        fn texture(&mut self, _: &str) -> Arc<TextureData> {
            static DATA: std::sync::OnceLock<Arc<TextureData>> = std::sync::OnceLock::new();
            DATA.get_or_init(|| {
                Arc::new(TextureData {
                    width: 4,
                    height: 4,
                    format: TextureFormatKind::Rgba8,
                    mips: vec![vec![255u8; 4 * 4 * 4], vec![0u8; 2 * 2 * 4]],
                })
            })
            .clone()
        }
    }

    const GRAY: u8 = 128;

    struct Gray;

    impl TextureSource for Gray {
        fn texture(&mut self, _: &str) -> Arc<TextureData> {
            static DATA: std::sync::OnceLock<Arc<TextureData>> = std::sync::OnceLock::new();
            DATA.get_or_init(|| Arc::new(TextureData { width: 4, height: 4, format: TextureFormatKind::Rgba8, mips: vec![gray_rgba()] })).clone()
        }
    }

    struct WhiteOverBlackMips;

    impl TextureSource for WhiteOverBlackMips {
        fn texture(&mut self, _: &str) -> Arc<TextureData> {
            let mips = vec![[255u8, 255, 255, 255].repeat(64 * 64), [0u8, 0, 0, 255].repeat(32 * 32), [0u8, 0, 0, 255].repeat(16 * 16)];
            Arc::new(TextureData { width: 64, height: 64, format: TextureFormatKind::Rgba8, mips })
        }
    }

    fn gray_rgba() -> Vec<u8> {
        [GRAY, GRAY, GRAY, 255].repeat(4 * 4)
    }

    fn noon() -> Environment {
        Environment::from_time(&TimeEnv::parse(TIMEENV_FIXTURE), 12.0)
    }

    fn quad_vertices(color: [f32; 4], mask_uv: [f32; 2]) -> Vec<Vertex> {
        quad_vertices_facing(color, mask_uv, [0.0, -1.0, 0.0])
    }

    fn quad_vertices_facing(color: [f32; 4], mask_uv: [f32; 2], normal: [f32; 3]) -> Vec<Vertex> {
        let v = |x: f32, z: f32| Vertex { position: [x, 0.0, z], normal, uv: [0.0, 0.0], mask_uv, color };
        vec![v(-100.0, -100.0), v(100.0, -100.0), v(100.0, 100.0), v(-100.0, 100.0)]
    }

    fn draw_of(vertices: Vec<Vertex>, kind: DrawKind, sun_affect: bool, texture: &str, zone: u8) -> MeshDraw {
        let indices = if kind == DrawKind::Bsp { vec![0, 1, 2, 0, 2, 3] } else { vec![0, 2, 1, 0, 3, 2] };
        MeshDraw { instances: Vec::new(), vertices, indices, material: texture_material(texture), mask: None, kind, sun_affect, pass: Pass::Opaque, zone, lightmap: None, modulate_2x: true, sector: None, fade: None, tint: None, actor: None }
    }

    fn blended(texture: &str, src: BlendFactor, dst: BlendFactor) -> Material {
        let mut material = texture_material(texture);
        let pass = &mut material.passes[0];
        pass.src = src;
        pass.dst = dst;
        pass.alpha_blend = true;
        pass.z_write = false;
        material.sorting_level = 2;
        material
    }

    fn alpha_material(texture: &str) -> Material {
        blended(texture, BlendFactor::SrcAlpha, BlendFactor::InvSrcAlpha)
    }

    fn flat(vertices: Vec<Vertex>, zone: u8) -> MeshDraw {
        draw_of(vertices, DrawKind::StaticMesh, false, "", zone)
    }

    fn unlit_zone() -> ZoneLighting {
        ZoneLighting { sun_affect: false, ..ZoneLighting::default() }
    }

    fn quad(color: [f32; 4]) -> Scene {
        let mut scene = Scene { zones: vec![unlit_zone()], ..Default::default() };
        scene.draws.push(flat(quad_vertices(color, [0.0, 0.0]), 0));
        scene
    }

    fn camera() -> Camera {
        Camera { eye: vec3(0.0, -300.0, 0.0), target: vec3(0.0, 0.0, 0.0), fov_x_deg: 60.0, near: 1.0, far: 5000.0 }
    }

    #[test]
    fn instanced_decorations_match_expanded_pixels() {
        use killer_assets::scene::DecorationInstance;
        let mut gpu = Headless::new(128, 128).unwrap();
        for translucent in [false, true] {
            let mut scene = quad([1.0; 4]);
            let draw = &mut scene.draws[0];
            draw.kind = DrawKind::Terrain;
            draw.material.passes[0].two_sided = true;
            draw.material.passes[0].lit = false;
            draw.fade = Some([200.0, 600.0]);
            if translucent {
                draw.material = alpha_material("");
                draw.material.passes[0].two_sided = true;
                draw.material.passes[0].lit = false;
                draw.pass = Pass::Translucent;
            }
            draw.instances = (0..6).map(|i| DecorationInstance {
                world: glam::Mat4::from_scale_rotation_translation(
                    vec3(if i % 2 == 0 { -0.5 } else { 0.5 }, 1.0, 0.5),
                    glam::Quat::from_rotation_y(i as f32 * 0.15),
                    vec3((i % 3) as f32 * 80.0 - 80.0, (i / 3) as f32 * 40.0, (i / 3) as f32 * 80.0 - 40.0),
                ).to_cols_array_2d(),
                mask_uv: [0.25, 0.75], color: [0.2 + i as f32 * 0.1, 0.6, 0.8, 0.75],
            }).collect();
            let mut expanded = scene.clone();
            let draw = &mut expanded.draws[0];
            let vertices = draw.world_vertices().collect();
            let indices = draw.world_indices().collect();
            draw.vertices = vertices;
            draw.indices = indices;
            draw.instances.clear();
            gpu.upload(&expanded, &mut White);
            let expected = gpu.render(&camera(), &noon()).unwrap();
            gpu.upload(&scene, &mut White);
            let actual = gpu.render(&camera(), &noon()).unwrap();
            assert!(expected.as_chunks::<4>().0.iter().any(|pixel| pixel[0] > 10));
            let mismatches = actual.iter().zip(&expected).filter(|(a, b)| a.abs_diff(**b) > 1).count();
            assert_eq!(mismatches, 0, "translucent={translucent}");
        }
    }

    struct CountedMips {
        data: Arc<TextureData>,
        calls: usize,
    }

    impl CountedMips {
        fn new(width: u32) -> Self {
            let mips = [[220, 40, 20, 255], [20, 220, 40, 255], [40, 20, 220, 255]].into_iter().enumerate().map(|(level, color)| {
                color.repeat(((width >> level) * (width >> level)) as usize)
            }).collect();
            Self { data: Arc::new(TextureData { width, height: width, format: TextureFormatKind::Rgba8, mips }), calls: 0 }
        }
    }

    impl TextureSource for CountedMips {
        fn texture(&mut self, _: &str) -> Arc<TextureData> {
            self.calls += 1;
            self.data.clone()
        }
    }

    fn mip_scene(count: usize) -> Scene {
        let draws = (0..count).map(|index| draw_of(quad_vertices([1.0; 4], [0.0; 2]), DrawKind::StaticMesh, false, &format!("mip{index}"), 0)).collect();
        Scene { draws, zones: vec![unlit_zone()], ..Default::default() }
    }

    #[test]
    fn initial_texture_detail_matches_reload_pixels_without_double_reads() {
        let scene = mip_scene(4);
        for detail in [0, 1, 2] {
            let mut old = Headless::new(64, 64).unwrap();
            let mut old_source = CountedMips::new(256);
            old.upload(&scene, &mut old_source);
            old.renderer.reload_texture_detail(detail, &mut old_source);
            let expected = old.render(&camera(), &noon()).unwrap();
            let mut new = Headless::new(64, 64).unwrap();
            let mut new_source = CountedMips::new(256);
            new.renderer.upload_with_texture_detail(&scene, &mut new_source, detail);
            assert_eq!(new.render(&camera(), &noon()).unwrap(), expected);
            assert_eq!(old_source.calls, new_source.calls * 2);
            for changed in [2, 0, 1] {
                old.renderer.reload_texture_detail(changed, &mut old_source);
                new.renderer.reload_texture_detail(changed, &mut new_source);
                assert_eq!(old.render(&camera(), &noon()).unwrap(), new.render(&camera(), &noon()).unwrap());
            }
        }
    }

    #[test]
    #[ignore]
    fn initial_full_texture_upload_cost() {
        let scene = mip_scene(32);
        let mut times = [Vec::new(), Vec::new()];
        for sample in 0..8 {
            for mode in [sample % 2, 1 - sample % 2] {
                let mut headless = Headless::new(64, 64).unwrap();
                let mut source = CountedMips::new(1024);
                let started = std::time::Instant::now();
                if mode == 0 {
                    headless.upload(&scene, &mut source);
                    headless.renderer.reload_texture_detail(0, &mut source);
                } else {
                    headless.renderer.upload_with_texture_detail(&scene, &mut source, 0);
                }
                headless.renderer.queue().submit([]);
                headless.renderer.device().poll(wgpu::PollType::wait_indefinitely()).unwrap();
                if sample > 0 { times[mode].push(started.elapsed().as_secs_f64() * 1000.0); }
                assert_eq!(source.calls, if mode == 0 {128} else {64});
            }
        }
        for samples in &mut times { samples.sort_by(f64::total_cmp); }
        eprintln!("Full 32 texturas 1024²/3 mips: {:.3} → {:.3} ms; 128 → 64 consultas à fonte", times[0][3], times[1][3]);
    }

    #[test]
    fn bsp_projector_depth_bias_keeps_native_float_units() {
        struct Colors;
        impl TextureSource for Colors {
            fn texture(&mut self, path: &str) -> Arc<TextureData> {
                if path == "projected" { White.texture(path) } else { Gray.texture(path) }
            }
        }
        let mut gpu = Headless::new(64, 64).unwrap();
        let mut base = quad([0.25; 4]);
        base.draws[0].material = texture_material("base");
        base.draws[0].material.passes[0].lit = false;
        for (distance, slope, visible) in [(0.03, 0.0, true), (0.3, 0.0, false), (0.3, 0.2, true)] {
            let mut material = texture_material("projected");
            material.root_class = "projectormaterial".into();
            material.passes[0].lit = false;
            material.passes[0].z_write = false;
            let mut scene = base.clone();
            let mut draw = quad_at_depth([1.0; 4], distance, material, Pass::Projector);
            draw.kind = DrawKind::Projector;
            scene.draws.push(draw);
            for draw in &mut scene.draws {
                for vertex in &mut draw.vertices {
                    vertex.position[1] += slope * vertex.position[0];
                }
            }
            gpu.upload(&scene, &mut Colors);
            gpu.renderer_mut().set_projectors(false);
            let background = gpu.render(&camera(), &noon()).unwrap();
            gpu.renderer_mut().set_projectors(true);
            let biased = gpu.render(&camera(), &noon()).unwrap();
            if visible {
                assert_ne!(center_pixel(&biased), center_pixel(&background));
            } else {
                assert_eq!(biased, background);
            }
        }
    }

    #[test]
    fn eight_material_stages_bind_each_texture_and_survive_cache() {
        use killer_assets::cache::{CachedScene, SourceStamp, decode, encode_to_vec};
        use killer_assets::textures::TextureBank;
        struct Colors;
        impl TextureSource for Colors {
            fn texture(&mut self, path: &str) -> Arc<TextureData> {
                let index: u8 = path.parse().unwrap();
                let color = [24 + index * 24, 192 - index * 16, 32 + index * 16, 255];
                Arc::new(TextureData { width: 2, height: 2, format: TextureFormatKind::Rgba8, mips: vec![color.repeat(4)] })
            }
        }
        let mut gpu = Headless::new(64, 64).unwrap();
        for count in 1..=8 {
            let mut material = texture_material("0");
            for index in 1..count {
                material.stages.push(texture_material(&index.to_string()).stages.remove(0));
            }
            material.passes[0].stages = 0..count;
            material.passes[0].lit = false;
            let scene = Scene { draws: vec![quad_at_depth([1.0; 4], 0.0, material, Pass::Opaque)], ..Default::default() };
            gpu.upload(&scene, &mut Colors);
            let pixels = gpu.render(&camera(), &noon()).unwrap();
            let index = (count - 1) as u8;
            assert_eq!(center_pixel(&pixels), [24 + index * 24, 192 - index * 16, 32 + index * 16, 255]);
            let stamp = SourceStamp::of(b"eight-stage-material");
            let bytes = encode_to_vec(&stamp, &CachedScene::from_bank(scene, &TextureBank::new()));
            let cached = decode(bytes.as_slice(), &stamp).unwrap();
            gpu.upload(&cached.scene, &mut Colors);
            assert_eq!(gpu.render(&camera(), &noon()).unwrap(), pixels);
        }
    }

    #[test]
    fn four_material_stages_preserve_texture_binding_and_diffuse_attenuation() {
        use l2_catalog::{AlphaOp, ColorOp, StageSource};
        use killer_assets::cache::{CachedScene, SourceStamp, decode, encode_to_vec};
        use killer_assets::textures::TextureBank;
        struct Colors;
        impl TextureSource for Colors {
            fn texture(&mut self, path: &str) -> Arc<TextureData> {
                let color = match path {
                    "base" => [64, 96, 128, 255],
                    "projected" => [128, 96, 64, 128],
                    "opacity" => [255, 255, 255, 128],
                    "fourth" => [192, 64, 32, 64],
                    _ => [255; 4],
                };
                Arc::new(TextureData { width: 2, height: 2, format: TextureFormatKind::Rgba8, mips: vec![color.repeat(4)] })
            }
        }
        let mut headless = Headless::new(64, 64).unwrap();
        for textured in [false, true] {
            let mut material = texture_material("base");
            let mut projection = texture_material("projected").stages.remove(0);
            projection.color_op = ColorOp::Modulate2X;
            let mut opacity = texture_material("opacity").stages.remove(0);
            opacity.color_op = ColorOp::SelectCurrent;
            opacity.alpha_op = AlphaOp::Modulate;
            let mut fourth = texture_material("fourth").stages.remove(0);
            if !textured {
                fourth.source = StageSource::Diffuse;
                fourth.color_op = ColorOp::SelectCurrent;
                fourth.alpha_op = AlphaOp::Modulate;
            }
            material.stages.extend([projection, opacity, fourth]);
            material.passes[0].stages = 0..4;
            material.passes[0].lit = false;
            let mut draw = quad_at_depth([0.5; 4], 0.0, material, Pass::Opaque);
            draw.kind = DrawKind::Projector;
            let scene = Scene { draws: vec![draw], ..Default::default() };
            headless.upload(&scene, &mut Colors);
            let pixels = headless.render(&camera(), &noon()).unwrap();
            let expected = if textured { [192, 64, 32, 64] } else { [64, 72, 64, 32] };
            let actual = center_pixel(&pixels);
            for i in 0..4 {
                assert!(actual[i].abs_diff(expected[i]) <= 1, "textured={textured}: {actual:?}, expected {expected:?}");
            }
            let stamp = SourceStamp::of(b"four-stage-material");
            let bytes = encode_to_vec(&stamp, &CachedScene::from_bank(scene, &TextureBank::new()));
            let cached = decode(bytes.as_slice(), &stamp).unwrap();
            headless.upload(&cached.scene, &mut Colors);
            assert_eq!(headless.render(&camera(), &noon()).unwrap(), pixels);
        }
    }

    #[test]
    fn projector_1000c9a0_front_and_back_faces_use_native_cube() {
        use killer_assets::scene::projectors::{calc_matrix, terrain_material};
        use ue2_level::{Placement, projectors::Projector};
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let base = quad([0.25, 0.25, 0.25, 1.0]);
        headless.upload(&base, &mut White);
        let before = headless.render(&camera(), &noon()).unwrap();
        for (normal, expected) in [([0.0, -1.0, 0.0], 254u8), ([0.0, 1.0, 0.0], 0)] {
            let def = Projector {
                placement: Placement { location: [0.0, -50.0, 0.0], rotation: [0, 16384, 0] },
                frame_blend: 2, draw_scale: 2.0, ..Default::default()
            };
            let projection = calc_matrix(&def, [128, 128], None);
            let material = terrain_material(&def, &projection, texture_material("white"), None);
            let mut draw = quad_at_depth([1.0; 4], 0.0, material, Pass::Projector);
            draw.kind = DrawKind::Projector;
            for vertex in &mut draw.vertices { vertex.normal = normal; }
            let mut scene = base.clone();
            scene.draws.push(draw);
            headless.upload(&scene, &mut White);
            let image = headless.render(&camera(), &noon()).unwrap();
            let background = center_pixel(&before);
            let expected = std::array::from_fn(|i| byte(f32::from(expected) / 255.0 + f32::from(background[i]) / 255.0 * (1.0 - f32::from(expected) / 255.0)));
            expect_rgb(center_pixel(&image), expected, 2);
            if normal[1] > 0.0 { assert_eq!(image, before); }
        }
    }

    #[test]
    fn projector_uv_orientation_and_z_divide_match_world_projection() {
        use killer_assets::scene::projectors::{calc_matrix, terrain_material};
        use ue2_level::{Placement, projectors::{Projector, ON_BACKFACES}};
        struct Gradient;
        impl TextureSource for Gradient {
            fn texture(&mut self, _: &str) -> Arc<TextureData> {
                let data = (0..4u8).flat_map(|y| (0..4u8).flat_map(move |x| [x * 64, y * 64, (3 - x) * 64, 255])).collect();
                Arc::new(TextureData { width: 4, height: 4, format: TextureFormatKind::Rgba8, mips: vec![data] })
            }
        }
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        for fov in [0, 45] {
            let def = Projector { placement: Placement { location: [0.0, -50.0, 0.0], rotation: [0, 16384, 0] }, fov, draw_scale: 2.0,
                frame_blend: 2, flags: ON_BACKFACES, ..Default::default() };
            let projection = calc_matrix(&def, [128, 128], None);
            let mut material = texture_material("gradient");
            if let l2_catalog::StageSource::Texture(texture) = &mut material.stages[0].source { texture.clamp = [true; 2]; }
            let material = terrain_material(&def, &projection, material, None);
            let mut draw = quad_at_depth([1.0; 4], 0.0, material, Pass::Projector);
            draw.kind = DrawKind::Projector;
            let scene = Scene { draws: vec![draw], zones: vec![unlit_zone()], ..Default::default() };
            headless.upload(&scene, &mut Gradient);
            let camera = camera();
            let rgba = headless.render(&camera, &noon()).unwrap();
            let inverse = camera.view_proj(1.0).inverse();
            for y in [20, 32, 44] {
                for x in [20, 32, 44] {
                    let ndc = glam::Vec4::new((x as f32 + 0.5) / 32.0 - 1.0, 1.0 - (y as f32 + 0.5) / 32.0, 0.5, 1.0);
                    let p = inverse * ndc;
                    let ray = p.truncate() / p.w - camera.eye;
                    let world = camera.eye + ray * (-camera.eye.y / ray.y);
                    let projected = projection.matrix * world.extend(1.0);
                    let uv = projected.truncate().truncate() / if fov == 0 { 1.0 } else { projected.z };
                    let s = (uv * 4.0 - glam::Vec2::splat(0.5)).clamp(glam::Vec2::ZERO, glam::Vec2::splat(3.0));
                    let expected = [byte(s.x * 64.0 / 255.0), byte(s.y * 64.0 / 255.0), byte((3.0 - s.x) * 64.0 / 255.0)];
                    let offset = ((y * 64 + x) * 4) as usize;
                    expect_rgb(rgba[offset..offset + 4].try_into().unwrap(), expected, 2);
                }
            }
        }
    }

    #[test]
    fn projector_10005f20_blends_and_projective_uv_survive_cache() {
        use killer_assets::scene::projectors::{calc_matrix, terrain_material};
        use killer_assets::textures::TextureBank;
        use killer_assets::cache::CachedScene;
        use ue2_level::{Placement, projectors::{Projector, ON_BACKFACES}};
        struct Projected;
        impl TextureSource for Projected {
            fn texture(&mut self, _: &str) -> Arc<TextureData> {
                Arc::new(TextureData { width: 2, height: 2, format: TextureFormatKind::Rgba8,
                    mips: vec![[64, 128, 192, 128].repeat(4)] })
            }
        }
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let base = quad([0.25, 0.25, 0.25, 1.0]);
        headless.upload(&base, &mut Checker);
        let before = headless.render(&camera(), &noon()).unwrap();
        let background = center_pixel(&before);
        for frame_blend in [1, 2, 3] {
            for (fov, material_blend) in [0, 45].into_iter().flat_map(|fov| [0, 1, 2].map(|blend| (fov, blend))) {
                let def = Projector { placement: Placement { location: [0.0, -50.0, 0.0], rotation: [0, 16384, 0] }, fov, draw_scale: 2.0,
                    frame_blend, material_blend, flags: ON_BACKFACES, ..Default::default() };
                let projection = calc_matrix(&def, [128, 128], None);
                let material = terrain_material(&def, &projection, texture_material("projected"), None);
                let mut scene = base.clone();
                let mut draw = quad_at_depth([1.0; 4], 0.0, material, Pass::Projector);
                draw.kind = DrawKind::Projector;
                scene.draws.push(draw);
                headless.upload(&scene, &mut Projected);
                let rgba = headless.render(&camera(), &noon()).unwrap();
                let expected: [u8; 3] = std::array::from_fn(|i| {
                    let dst = f32::from(background[i]) / 255.0;
                    let src = f32::from([64u8, 128, 192][i]) / 255.0;
                    let src = match material_blend {
                        1 => (2.0 * src).min(1.0),
                        2 => src,
                        _ => src,
                    };
                    byte(match frame_blend { 1 => dst * src * 2.0, 2 => src * (128.0 / 255.0) + dst * (127.0 / 255.0), _ => src + dst })
                });
                expect_rgb(center_pixel(&rgba), expected, 2);
                let cached = CachedScene::from_bank(scene.clone(), &TextureBank::new());
                let stamp = killer_assets::cache::SourceStamp::of(b"projectors");
                let encoded = killer_assets::cache::encode_to_vec(&stamp, &cached);
                let cached = killer_assets::cache::decode(encoded.as_slice(), &stamp).unwrap();
                assert_eq!(cached.scene, scene);
                headless.upload(&cached.scene, &mut Projected);
                assert_eq!(headless.render(&camera(), &noon()).unwrap(), rgba);
                headless.renderer_mut().set_projectors(false);
                assert_eq!(headless.render(&camera(), &noon()).unwrap(), before);
                headless.renderer_mut().set_projectors(true);
            }
        }
    }

    #[test]
    fn mover_gpu_open_reverse_damage_cache_and_neighbour_removal() {
        use killer_assets::scene::movers::MovingAsset;
        use ue2_level::{Placement, movers::Mover};
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let mut scene = quad([0.8, 0.2, 0.2, 1.0]);
        let origin = Placement { location: [0.0; 3], rotation: [0; 3] };
        let mut def = Mover { base: origin, key_pos: [[0.0; 3]; 8], key_rot: [[0; 3]; 8], meshes: std::array::from_fn(|_| None), server_id: Some(24180029), move_time: 1.0, glide: true };
        def.key_pos[1] = [400.0, 0.0, 0.0];
        def.meshes[3] = Some("damaged".into());
        let mut grades = std::array::from_fn(|_| Vec::new());
        grades[3] = quad([0.2, 0.8, 0.2, 1.0]).draws;
        scene.moving.push(MovingAsset { attached: Vec::new(), name: "Mover33".into(), mesh: Some("door".into()), origin, mover: Some(def), movement: None, draws: std::mem::take(&mut scene.draws), grades });
        let stamp = killer_assets::cache::SourceStamp::of(b"gpu-door");
        let cached = killer_assets::cache::CachedScene::from_bank(scene, &Default::default());
        let encoded = killer_assets::cache::encode_to_vec(&stamp, &cached);
        let scene = killer_assets::cache::decode(encoded.as_slice(), &stamp).unwrap().scene;
        headless.renderer.add_tile("empty", &Scene::default(), &mut White);
        headless.renderer.add_tile("24_18", &scene, &mut White);
        let closed = headless.render(&camera(), &noon()).unwrap();
        assert_ne!(center_pixel(&closed), CLEAR_RGBA);
        headless.renderer.door_status(100, false, 0, false);
        headless.renderer.tick_movers(1.0);
        assert_eq!(headless.render(&camera(), &noon()).unwrap(), closed);
        headless.renderer.door_info(100, 24180029);
        headless.renderer.door_status(100, false, 0, false);
        headless.renderer.tick_movers(0.5);
        let half = headless.render(&camera(), &noon()).unwrap();
        assert_ne!(half, closed);
        headless.renderer.door_status(100, true, 0, false);
        headless.renderer.tick_movers(0.5);
        assert_eq!(headless.render(&camera(), &noon()).unwrap(), closed);
        headless.renderer.door_status(100, true, 3, true);
        let damaged = headless.render(&camera(), &noon()).unwrap();
        assert_ne!(damaged, closed);
        headless.renderer.door_status(100, true, 4, true);
        assert_eq!(headless.render(&camera(), &noon()).unwrap(), damaged);
        headless.renderer.remove_tile("empty");
        assert_eq!(headless.render(&camera(), &noon()).unwrap(), damaged);
        headless.renderer.door_status(100, true, 6, false);
        assert_eq!(center_pixel(&headless.render(&camera(), &noon()).unwrap()), CLEAR_RGBA);
        headless.renderer.remove_tile("24_18");
        assert_eq!(headless.renderer.moving_actors().count(), 0);
        headless.renderer.door_info(100, 24180029);
        headless.renderer.door_status(100, false, 0, false);
        headless.renderer.add_tile("24_18", &scene, &mut White);
        headless.renderer.tick_movers(1.0);
        assert_eq!(center_pixel(&headless.render(&camera(), &noon()).unwrap()), CLEAR_RGBA);
    }

    #[test]
    fn terrain_sectors_are_culled_by_xy_distance_from_the_view_target_to_their_location() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let cam = camera();
        let color = [0.3, 0.3, 0.3, 1.0];
        let sector_at = |x: f32| {
            let mut draw = flat(quad_vertices(color, [0.0, 0.0]), 0);
            draw.sector = Some(SectorBounds { location: [x, 0.0, 0.0], min: [-100.0, 0.0, -100.0], max: [100.0, 0.0, 100.0] });
            draw
        };
        let scene_with = |x: f32| {
            let mut scene = Scene { zones: vec![unlit_zone()], terrain_range: 16384.0, ..Default::default() };
            scene.draws.push(sector_at(x));
            scene
        };
        headless.upload(&scene_with(1000.0), &mut Checker);
        assert_ne!(center_pixel(&headless.render(&cam, &noon()).unwrap()), &CLEAR_RGBA[..]);
        headless.upload(&scene_with(17000.0), &mut Checker);
        assert_eq!(center_pixel(&headless.render(&cam, &noon()).unwrap()), &CLEAR_RGBA[..]);
        headless.renderer.set_view_target(Some(vec3(16000.0, 0.0, 0.0)));
        assert_ne!(center_pixel(&headless.render(&cam, &noon()).unwrap()), &CLEAR_RGBA[..]);
        headless.renderer.set_view_target(None);
        let mut boxed = scene_with(1000.0);
        boxed.draws[0].sector.as_mut().unwrap().min = [5000.0, 0.0, -100.0];
        boxed.draws[0].sector.as_mut().unwrap().max = [5200.0, 0.0, 100.0];
        headless.upload(&boxed, &mut Checker);
        assert_eq!(center_pixel(&headless.render(&cam, &noon()).unwrap()), &CLEAR_RGBA[..]);
    }

    #[test]
    fn static_mesh_actors_are_cut_by_pawn_distance_and_switch_lod_meshes() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let cam = camera();
        let actor_at = |xy: [f32; 2], lod: Option<LodSelect>, time_reactor: Option<[f32; 2]>, color: [f32; 4]| {
            let mut draw = flat(quad_vertices(color, [0.0, 0.0]), 0);
            draw.actor = Some(ActorCull { id: 1, ambient: Default::default(), xy, range: 8192.0, ignored_range: false, time_reactor, lod });
            draw
        };
        let scene_of = |draws: Vec<MeshDraw>| Scene { zones: vec![unlit_zone()], draws, ..Default::default() };
        let red = [0.5, 0.0, 0.0, 1.0];
        let green = [0.0, 0.5, 0.0, 1.0];
        headless.upload(&scene_of(vec![actor_at([0.0, 0.0], None, None, red)]), &mut Checker);
        headless.renderer.set_view_target(Some(vec3(8000.0, 0.0, 0.0)));
        assert_ne!(center_pixel(&headless.render(&cam, &noon()).unwrap()), &CLEAR_RGBA[..]);
        headless.renderer.set_view_target(Some(vec3(8448.0, 0.0, 0.0)));
        assert_eq!(center_pixel(&headless.render(&cam, &noon()).unwrap()), &CLEAR_RGBA[..]);
        let lods = vec![
            actor_at([0.0, 0.0], Some(LodSelect { level: 0, blend: false, ranges: [4096.0, 8192.0] }), None, red),
            actor_at([0.0, 0.0], Some(LodSelect { level: 1, blend: false, ranges: [4096.0, 8192.0] }), None, green),
        ];
        headless.upload(&scene_of(lods), &mut Checker);
        headless.renderer.set_view_target(Some(vec3(4000.0, 0.0, 0.0)));
        let near = center_pixel(&headless.render(&cam, &noon()).unwrap()).to_vec();
        assert!(near[0] > near[1], "{near:?}");
        headless.renderer.set_view_target(Some(vec3(4100.0, 0.0, 0.0)));
        let mid = center_pixel(&headless.render(&cam, &noon()).unwrap()).to_vec();
        assert!(mid[1] > mid[0], "{mid:?}");
        headless.upload(&scene_of(vec![actor_at([0.0, 0.0], None, Some([20.0, 6.0]), red)]), &mut Checker);
        headless.renderer.set_view_target(None);
        assert_eq!(center_pixel(&headless.render(&cam, &noon()).unwrap()), &CLEAR_RGBA[..]);
        let night = Environment::from_time(&TimeEnv::parse(TIMEENV_FIXTURE), 23.0);
        assert_ne!(center_pixel(&headless.render(&cam, &night).unwrap()), &CLEAR_RGBA[..]);
    }

    fn fading_actor(id: u32, lod: Option<LodSelect>, color: [f32; 4]) -> MeshDraw {
        let mut draw = flat(quad_vertices(color, [0.0, 0.0]), 0);
        draw.actor = Some(ActorCull { id, ambient: Default::default(), xy: [0.0, 0.0], range: 8192.0, ignored_range: false, time_reactor: None, lod });
        draw
    }

    fn render_with_dt(headless: &mut Headless, dt: f32) -> Vec<u8> {
        headless.renderer.set_viewer(Viewer { zone: 0, position: camera().eye, dt });
        center_pixel(&headless.render(&camera(), &noon()).unwrap()).to_vec()
    }

    #[test]
    fn a_static_mesh_in_the_range_band_draws_one_frame_untouched_then_fades_in_through_the_alpha_pipeline() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let scene = Scene { zones: vec![unlit_zone()], draws: vec![fading_actor(1, None, [0.9, 0.9, 0.9, 1.0])], ..Default::default() };
        headless.upload(&scene, &mut Checker);
        headless.renderer.set_view_target(Some(vec3(8192.0 - 100.0, 0.0, 0.0)));
        let first = render_with_dt(&mut headless, 0.5);
        assert_ne!(first[..3], CLEAR_RGBA[..3]);
        let half = render_with_dt(&mut headless, 0.5);
        for channel in 0..3 {
            let (low, high) = (first[channel].min(CLEAR_RGBA[channel]), first[channel].max(CLEAR_RGBA[channel]));
            assert!(half[channel] > low && half[channel] < high, "channel {channel}: {half:?} between {first:?} and {CLEAR_RGBA:?}");
        }
        let full = render_with_dt(&mut headless, 0.5);
        assert_eq!(full[..3], first[..3]);
    }

    #[test]
    fn a_blended_lod_change_shows_both_meshes_until_the_cross_fade_ends() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let (red, green) = ([0.5, 0.0, 0.0, 1.0], [0.0, 0.5, 0.0, 1.0]);
        let level = |level| Some(LodSelect { level, blend: true, ranges: [4096.0, 8192.0] });
        let scene = Scene { zones: vec![unlit_zone()], draws: vec![fading_actor(1, level(0), red), fading_actor(1, level(1), green)], ..Default::default() };
        headless.upload(&scene, &mut Checker);
        headless.renderer.set_view_target(Some(vec3(1000.0, 0.0, 0.0)));
        let near = render_with_dt(&mut headless, 0.9);
        assert!(near[0] > near[1], "{near:?}");
        headless.renderer.set_view_target(Some(vec3(5000.0, 0.0, 0.0)));
        let start = render_with_dt(&mut headless, 0.9);
        assert!(start[0] > start[1], "the old mesh is still opaque on the first blended frame: {start:?}");
        let middle = render_with_dt(&mut headless, 0.9);
        assert!(middle[0] > CLEAR_RGBA[0] && middle[1] > CLEAR_RGBA[1], "both meshes show mid-blend: {middle:?}");
        let end = render_with_dt(&mut headless, 0.9);
        let done = render_with_dt(&mut headless, 0.9);
        assert!(done[1] > done[0], "{end:?} then {done:?}");
    }

    #[test]
    fn decoration_fade_scales_alpha_by_camera_distance_between_min_and_max() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let cam = camera();
        let color = [0.3, 0.3, 0.3, 1.0];
        let faded = |range: [f32; 2]| {
            let mut draw = MeshDraw { material: alpha_material(""), pass: Pass::Translucent, ..flat(quad_vertices(color, [0.0, 0.0]), 0) };
            draw.fade = Some(range);
            let mut scene = Scene { zones: vec![unlit_zone()], ..Default::default() };
            scene.draws.push(draw);
            scene
        };
        headless.upload(&faded([100.0, 500.0]), &mut Checker);
        let half = center_pixel(&headless.render(&cam, &noon()).unwrap()).to_vec();
        headless.upload(&faded([400.0, 500.0]), &mut Checker);
        let full = center_pixel(&headless.render(&cam, &noon()).unwrap()).to_vec();
        headless.upload(&faded([100.0, 200.0]), &mut Checker);
        let gone = center_pixel(&headless.render(&cam, &noon()).unwrap()).to_vec();
        assert_eq!(gone, CLEAR_RGBA.to_vec());
        assert_ne!(full, gone);
        for channel in 0..3 {
            let expected = (f32::from(full[channel]) + f32::from(gone[channel])) / 2.0;
            assert!((f32::from(half[channel]) - expected).abs() <= 3.0, "{half:?} {full:?} {gone:?}");
        }
    }

    fn center_pixel(rgba: &[u8]) -> &[u8] {
        &rgba[(32 * 64 + 32) * 4..][..4]
    }

    fn byte(value: f32) -> u8 {
        (value.clamp(0.0, 1.0) * 255.0).round() as u8
    }

    fn expect_rgb(actual: &[u8], expected: [u8; 3], tolerance: i32) {
        for i in 0..3 {
            assert!((actual[i] as i32 - expected[i] as i32).abs() <= tolerance, "{actual:?} vs {expected:?}");
        }
    }

    fn sun_toward(env: &Environment) -> [f32; 3] {
        (-env.sun_dir).to_array()
    }

    fn scaled(rgb: [f32; 3], k: f32) -> [f32; 3] {
        rgb.map(|c| c * k)
    }

    fn sum(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
    }

    fn pixel_of(tex: f32, d: [f32; 3]) -> [u8; 3] {
        d.map(|c| byte(tex * 2.0 * c.clamp(0.0, 1.0)))
    }

    fn quad_at_depth(color: [f32; 4], y: f32, material: Material, pass: Pass) -> MeshDraw {
        let vertices = quad_vertices(color, [0.0; 2]).into_iter().map(|mut v| {
            v.position[1] = y;
            v
        });
        MeshDraw { material, pass, ..flat(vertices.collect(), 0) }
    }

    #[test]
    fn alpha_blending_happens_in_gamma_space_like_the_d3d_target() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let mut scene = Scene { zones: vec![unlit_zone()], ..Default::default() };
        scene.draws.push(quad_at_depth([0.25, 0.25, 0.25, 1.0], 50.0, texture_material(""), Pass::Opaque));
        scene.draws.push(quad_at_depth([0.5, 0.5, 0.5, 0.5], 0.0, alpha_material(""), Pass::Translucent));
        headless.upload(&scene, &mut Checker);
        let rgba = headless.render(&camera(), &noon()).unwrap();
        expect_rgb(center_pixel(&rgba), [191, 191, 191], 1);
    }

    #[test]
    fn draws_sharing_one_tile_buffer_each_use_their_own_vertices_and_indices() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let colors = [[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]];
        for nearest in 0..3 {
            let mut scene = Scene { zones: vec![unlit_zone()], ..Default::default() };
            for (index, color) in colors.into_iter().enumerate() {
                let depth = if index == nearest { 0.0 } else { 200.0 + index as f32 * 10.0 };
                scene.draws.push(quad_at_depth(color, depth, texture_material(""), Pass::Opaque));
            }
            headless.upload(&scene, &mut Checker);
            let rgba = headless.render(&camera(), &noon()).unwrap();
            let expected = colors[nearest].map(|channel| (channel * 255.0) as i32);
            expect_rgb(center_pixel(&rgba), [expected[0] as u8, expected[1] as u8, expected[2] as u8], 2);
        }
    }

    #[test]
    fn more_draws_than_material_slots_still_all_upload() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let mut scene = Scene { zones: vec![unlit_zone()], ..Default::default() };
        for index in 0..1100 {
            scene.draws.push(quad_at_depth([0.5, 0.5, 0.5, 1.0], 20.0 + index as f32 * 0.01, texture_material(""), Pass::Opaque));
        }
        headless.upload(&scene, &mut Checker);
        assert_eq!(headless.renderer_mut().visible_draw_count(&camera(), 1.0), 1100);
        let rgba = headless.render(&camera(), &noon()).unwrap();
        expect_rgb(center_pixel(&rgba), [255, 255, 255], 0);
        let id = headless.renderer_mut().upload_character(&CharacterMesh::empty(), &mut Checker);
        headless.renderer_mut().remove_character(id);
        assert_eq!(headless.render(&camera(), &noon()).unwrap(), rgba);
    }

    #[test]
    fn ob_translucent_adds_the_source_to_what_is_behind() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let mut scene = Scene { zones: vec![unlit_zone()], ..Default::default() };
        scene.draws.push(quad_at_depth([0.25, 0.25, 0.25, 1.0], 50.0, texture_material(""), Pass::Opaque));
        scene.draws.push(quad_at_depth([0.5, 0.5, 0.5, 0.5], 0.0, blended("", BlendFactor::One, BlendFactor::One), Pass::Translucent));
        headless.upload(&scene, &mut Checker);
        let rgba = headless.render(&camera(), &noon()).unwrap();
        expect_rgb(center_pixel(&rgba), [255, 255, 255], 0);
        let mut darken = Scene { zones: vec![unlit_zone()], ..Default::default() };
        darken.draws.push(quad_at_depth([0.25, 0.25, 0.25, 1.0], 50.0, texture_material(""), Pass::Opaque));
        darken.draws.push(quad_at_depth([0.25, 0.25, 0.25, 1.0], 0.0, blended("", BlendFactor::DstColor, BlendFactor::SrcColor), Pass::Translucent));
        headless.upload(&darken, &mut Checker);
        let rgba = headless.render(&camera(), &noon()).unwrap();
        expect_rgb(center_pixel(&rgba), [128, 128, 128], 1);
    }

    struct Halves;

    impl TextureSource for Halves {
        fn texture(&mut self, _: &str) -> Arc<TextureData> {
            let mut rgba = Vec::with_capacity(8 * 8 * 4);
            for _ in 0..8 {
                for x in 0..8 {
                    rgba.extend_from_slice(if x < 4 { &[255, 0, 0, 255] } else { &[0, 0, 255, 255] });
                }
            }
            Arc::new(TextureData { width: 8, height: 8, format: TextureFormatKind::Rgba8, mips: vec![rgba] })
        }
    }

    #[test]
    fn a_tex_panner_moves_the_texture_by_rate_per_second() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let v = |x: f32, z: f32, u: f32| Vertex { position: [x, 0.0, z], normal: [0.0, -1.0, 0.0], uv: [u, 0.5], mask_uv: [0.0; 2], color: [0.5, 0.5, 0.5, 1.0] };
        let vertices = vec![v(-100.0, -100.0, 0.0), v(100.0, -100.0, 1.0), v(100.0, 100.0, 1.0), v(-100.0, 100.0, 0.0)];
        let mut material = texture_material("Synthetic.Halves");
        material.stages[0].uv.push(UvModifier::Panner { direction: [0, 0, 0], rate: 0.5 });
        let mut scene = Scene { zones: vec![unlit_zone()], ..Default::default() };
        scene.draws.push(MeshDraw { material, ..flat(vertices, 0) });
        headless.upload(&scene, &mut Halves);
        headless.renderer_mut().set_time(0.0);
        let at_zero = headless.render(&camera(), &noon()).unwrap();
        let east = pixel(&at_zero, 24, 32);
        let west = pixel(&at_zero, 40, 32);
        assert!(east[2] > 200 && east[0] < 50, "{east:?}");
        assert!(west[0] > 200 && west[2] < 50, "{west:?}");
        headless.renderer_mut().set_time(1.0);
        let at_one = headless.render(&camera(), &noon()).unwrap();
        let east = pixel(&at_one, 24, 32);
        let west = pixel(&at_one, 40, 32);
        assert!(east[0] > 200 && east[2] < 50, "{east:?}");
        assert!(west[2] > 200 && west[0] < 50, "{west:?}");
        headless.renderer_mut().set_time(2.0);
        assert_eq!(headless.render(&camera(), &noon()).unwrap(), at_zero);
    }

    #[test]
    #[ignore]
    fn water_utexoscillator_105a9750_matches_original_uv_at_zero_one_two_seconds_on_gpu() {
        use killer_assets::{client::Client, textures::{BankSource, TextureBank}};
        let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR obrigatório");
        let mut client = Client::open(std::path::Path::new(&root)).unwrap();
        let material = client.catalog().material("FX_E_T.WaterSurfaceShaderSet.WaterShader01").unwrap();
        let mut headless = Headless::new(64, 64).unwrap();
        let mut bank = TextureBank::new();
        let mut source = BankSource { bank: &mut bank, catalog: client.catalog() };
        for time in [0.0f32, 1.0, 2.0] {
            let offset = (((f64::from(time) * 0.1).fract() + 0.1) * std::f64::consts::TAU).sin() * 0.05;
            for stage in &material.stages {
                let uv = stage.matrix(time).apply([0.25, 0.75]);
                assert!((f64::from(uv[0]) - 0.25 - offset).abs() < 1.0 / 256.0);
                assert!((f64::from(uv[1]) - 0.75 - offset).abs() < 1.0 / 256.0);
            }
            let mut scene = Scene { zones: vec![unlit_zone()], ..Default::default() };
            scene.draws.push(quad_at_depth([0.5, 0.5, 0.5, 1.0], 0.0, material.clone(), Pass::Translucent));
            headless.upload(&scene, &mut source);
            headless.renderer_mut().set_time(time);
            let actual = headless.render(&camera(), &noon()).unwrap();
            for stage in &mut scene.draws[0].material.stages {
                stage.uv = vec![
                    UvModifier::Panner { direction: [0, 0, 0], rate: offset as f32 },
                    UvModifier::Panner { direction: [0, 16384, 0], rate: offset as f32 },
                ];
            }
            headless.upload(&scene, &mut source);
            headless.renderer_mut().set_time(1.0);
            let expected = headless.render(&camera(), &noon()).unwrap();
            assert!(actual.iter().zip(&expected).all(|(&a, &b)| a.abs_diff(b) <= 1), "t={time}");
            assert!(actual.as_chunks::<4>().0.iter().any(|p| *p != CLEAR_RGBA));
        }
    }

    #[test]
    fn clearing_the_renderer_drops_the_uploaded_textures() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let mut scene = Scene::default();
        scene.draws.push(draw_of(quad_vertices([1.0; 4], [0.0; 2]), DrawKind::StaticMesh, false, "Pack.Tex", 0));
        headless.upload(&scene, &mut Checker);
        assert_eq!(headless.renderer_mut().texture_count(), 1);
        headless.renderer_mut().clear();
        assert_eq!(headless.renderer_mut().texture_count(), 0);
        headless.upload(&scene, &mut Checker);
        assert_eq!(headless.renderer_mut().texture_count(), 1);
    }

    #[test]
    #[ignore]
    fn wettexture_procedural_pixels_reach_the_gpu_for_all_original_exports() {
        use killer_assets::{client::Client, textures::{BankSource, TextureBank}};
        let root=std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let mut client=Client::open(std::path::Path::new(&root)).unwrap();
        struct Pixels(Arc<TextureData>);
        impl TextureSource for Pixels {
            fn texture(&mut self,_: &str) -> Arc<TextureData> { self.0.clone() }
        }
        let mut actual=Headless::new(128,128).unwrap();
        let mut expected=Headless::new(128,128).unwrap();
        for name in ["2f_water_proshader","2SP_PRO_000","eeee"] {
            actual.renderer_mut().set_no_fractal_anim(false);
            let path=format!("speaking2F_t.Effect.{name}");
            let definition=client.catalog().wet_texture(&path).unwrap();
            let material=client.catalog().material(&path).unwrap();
            let mut scene=Scene { zones:vec![unlit_zone()],..Default::default() };
            scene.draws.push(quad_at_depth([0.5,0.5,0.5,1.0],0.0,material,Pass::Opaque));
            for (vertex,uv) in scene.draws[0].vertices.iter_mut().zip([[0.0,1.0],[1.0,1.0],[1.0,0.0],[0.0,0.0]]) { vertex.uv=uv; }
            let mut bank=TextureBank::new();
            bank.texture(client.catalog(),&path);
            let stamp=killer_assets::cache::SourceStamp::of(name.as_bytes());
            let cached=killer_assets::cache::CachedScene::from_bank(scene,&bank);
            let bytes=killer_assets::cache::encode_to_vec(&stamp,&cached);
            let (scene,mut bank)=killer_assets::cache::decode(&bytes[..],&stamp).unwrap().into_bank();
            let mut source=BankSource { bank:&mut bank,catalog:client.catalog() };
            actual.upload(&scene,&mut source);
            let mut state=definition.simulation.clone();
            let mut random=ue2_assets::wet::FireRandom::from_seed(1);
            let mut prime=0;
            let mut first=None;
            for frame in 1..=30 {
                definition.evaluate(&mut state,&mut prime,0.0,&mut random);
                let a=actual.render(&camera(),&noon()).unwrap();
                let mut pixels=Pixels(Arc::new(TextureData { width:state.width as u32,height:state.height as u32,format:TextureFormatKind::Rgba8,mips:vec![definition.rgba(&state.pixels)] }));
                expected.upload(&scene,&mut pixels);
                let b=expected.render(&camera(),&noon()).unwrap();
                assert_eq!(a,b,"{name} frame {frame}");
                if frame==1 { first=Some(a.clone()); }
                if frame==30 { assert_ne!(Some(a.clone()),first,"{name} não animou"); }
                if [1,2,30].contains(&frame) {
                    let file=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../docs/renders/wet-{name}-{frame}.png"));
                    killer_assets::textures::write_png(&file,128,128,&a).unwrap();
                }
            }
            actual.upload(&scene,&mut source);
            actual.renderer_mut().set_no_fractal_anim(true);
            let mut frozen=definition.simulation.clone();
            definition.evaluate_with_animation(&mut frozen,&mut 0,0.0,&mut random,true);
            let mut pixels=Pixels(Arc::new(TextureData { width:frozen.width as u32,height:frozen.height as u32,format:TextureFormatKind::Rgba8,mips:vec![definition.rgba(&frozen.pixels)] }));
            expected.upload(&scene,&mut pixels);
            let reference=expected.render(&camera(),&noon()).unwrap();
            assert_eq!(actual.render(&camera(),&noon()).unwrap(),reference);
            assert_eq!(actual.render(&camera(),&noon()).unwrap(),reference);
            let mut twice=scene.clone();
            twice.draws.push(twice.draws[0].clone());
            for (index,draw) in twice.draws.iter_mut().enumerate() {
                for vertex in &mut draw.vertices { vertex.position[0]=vertex.position[0]*0.5+if index==0 { -50.0 } else { 50.0 }; }
            }
            actual.renderer_mut().set_no_fractal_anim(false);
            actual.upload(&twice,&mut source);
            let mut expected_scene=twice.clone();
            for (index,draw) in expected_scene.draws.iter_mut().enumerate() {
                for stage in &mut draw.material.stages {
                    if let l2_catalog::StageSource::Texture(texture)=&mut stage.source { texture.path=format!("snapshot{index}"); }
                }
            }
            struct Snapshots(Vec<Arc<TextureData>>);
            impl TextureSource for Snapshots {
                fn texture(&mut self,path:&str) -> Arc<TextureData> { self.0[usize::from(path=="snapshot1")].clone() }
            }
            let mut shared=definition.simulation.clone();
            for frame in 1..=10 {
                let snapshots=(0..2).map(|_| {
                    definition.evaluate(&mut shared,&mut 0,0.0,&mut random);
                    Arc::new(TextureData { width:shared.width as u32,height:shared.height as u32,format:TextureFormatKind::Rgba8,mips:vec![definition.rgba(&shared.pixels)] })
                }).collect();
                expected.upload(&expected_scene,&mut Snapshots(snapshots));
                assert_eq!(actual.render(&camera(),&noon()).unwrap(),expected.render(&camera(),&noon()).unwrap(),"{name}: estado compartilhado, duas avaliações, frame {frame}");
            }
        }
    }

    #[test]
    fn draws_behind_the_camera_are_left_out_of_the_frame() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let mut scene = quad([1.0; 4]);
        let behind = quad_vertices([1.0; 4], [0.0; 2]).into_iter().map(|mut v| {
            v.position[1] = -600.0;
            v
        });
        scene.draws.push(flat(behind.collect(), 0));
        headless.upload(&scene, &mut Gray);
        assert_eq!(headless.renderer_mut().visible_draw_count(&camera(), 1.0), 1);
        let rgba = headless.render(&camera(), &noon()).unwrap();
        assert_ne!(center_pixel(&rgba), &CLEAR_RGBA);
    }

    #[test]
    fn canvas_projected_labels_obey_world_depth_with_msaa_and_postprocess() {
        let mut gpu=Headless::new(64,64).unwrap();
        let adapter=pollster::block_on(wgpu::Instance::default().request_adapter(&Default::default())).unwrap();
        let camera=camera();
        let depth=|y| {let clip=camera.view_proj(1.0)*vec3(0.0,y,0.0).extend(1.0);clip.z/clip.w};
        let pixel=|rgba:&[u8],x:usize,y:usize| -> [u8;4] {rgba[(y*64+x)*4..(y*64+x)*4+4].try_into().unwrap()};
        gpu.upload(&quad([0.2,0.2,0.2,1.0]),&mut White);
        let off=crate::postprocess::PostProcessSettings::read(0,false,&killer_ui::ini::parse_ini(""));
        for (aa,post) in [(0,false),(2,false),(0,true)] {
            if aa>crate::video::multisample_capability(&adapter,gpu.renderer.device()) {continue;}
            gpu.renderer.set_postprocess(off).unwrap();
            gpu.renderer.set_antialiasing(&adapter,aa).unwrap();
            if post {
                let settings=crate::postprocess::PostProcessSettings::read(1,false,&killer_ui::ini::parse_ini("[S1]\nGlow=true\nHdr=true"));
                assert!(settings.active());
                gpu.renderer.set_postprocess(settings).unwrap();
            }
            gpu.renderer.clear_ui();
            let baseline=gpu.render(&camera,&noon()).unwrap();
            let mut labels=killer_ui::draw::DrawList::default();
            labels.draw_texture(0,28,64,8,0,0,1,1,"white",1.0);
            labels.quads[0].depth=depth(200.0);
            gpu.renderer.set_ui(&labels,&mut White);
            let hidden=gpu.render(&camera,&noon()).unwrap();
            assert_eq!(pixel(&hidden,32,32),pixel(&baseline,32,32),"aa={aa} post={post}");
            assert_eq!(pixel(&hidden,2,32),[255;4]);
            labels.quads[0].depth=depth(-100.0);
            gpu.renderer.set_ui(&labels,&mut White);
            let front=gpu.render(&camera,&noon()).unwrap();
            assert_eq!(pixel(&front,32,32),[255;4]);
            labels.quads[0].depth=0.0;
            gpu.renderer.set_ui(&labels,&mut White);
            assert_eq!(pixel(&gpu.render(&camera,&noon()).unwrap(),32,32),[255;4]);
            gpu.renderer.clear_ui();
            let texture=target(gpu.renderer.device(),1,1,COLOR_FORMAT,wgpu::TextureUsages::TEXTURE_BINDING|wgpu::TextureUsages::COPY_DST);
            gpu.renderer.queue().write_texture(texture.as_image_copy(),&[255;4],wgpu::TexelCopyBufferLayout {offset:0,bytes_per_row:Some(4),rows_per_image:None},wgpu::Extent3d {width:1,height:1,depth_or_array_layers:1});
            let mut mesh=crate::ui::UiMesh {vertices:Vec::new(),indices:vec![0,1,2,0,2,3],clips:vec![(0..3,[0,0,64,64]),(3..6,[0,0,64,64])]};
            mesh.vertices=[(0.0,28.0),(64.0,28.0),(64.0,36.0),(0.0,36.0)].into_iter().map(|(x,y)|crate::ui::UiVertex {position:[x,y,depth(200.0)],uv:[0.0;2],color:[1.0;4]}).collect();
            gpu.renderer.update_world_text(&mesh,&texture.create_view(&Default::default()));
            assert_eq!(gpu.render(&camera,&noon()).unwrap(),hidden);
            for vertex in &mut mesh.vertices {vertex.position[2]=depth(-100.0);vertex.color=[0.0,1.0,0.0,1.0];}
            mesh.vertices.extend(mesh.vertices.clone().into_iter().map(|mut vertex| {vertex.position[2]=depth(-50.0);vertex.color=[1.0,0.0,0.0,1.0];vertex}));
            mesh.indices.extend([4,5,6,4,6,7]);mesh.clips=vec![(0..12,[0,0,64,64])];
            gpu.renderer.update_world_text(&mesh,&texture.create_view(&Default::default()));
            assert_eq!(pixel(&gpu.render(&camera,&noon()).unwrap(),32,32),[255,0,0,255]);
            gpu.renderer.clear_world_text();
            assert_eq!(gpu.render(&camera,&noon()).unwrap(),baseline);
        }
    }

    #[test]
    fn renders_a_vertex_colored_quad_over_the_clear_color() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        headless.upload(&quad([0.4, 0.4, 0.4, 1.0]), &mut Checker);
        let rgba = headless.render(&camera(), &noon()).unwrap();
        assert_eq!(rgba.len(), 64 * 64 * 4);
        let center = center_pixel(&rgba);
        let corner = &rgba[..4];
        assert!(center[0] > corner[0], "{center:?} vs {corner:?}");
        assert_eq!(corner, CLEAR_RGBA);
    }

    #[test]
    fn terrain_layers_above_the_base_draw_over_the_coplanar_opaque_pass() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let cam = camera();

        let base = flat(quad_vertices([0.4, 0.4, 0.4, 1.0], [0.0, 0.0]), 0);
        let mut scene_a = Scene { zones: vec![unlit_zone()], ..Default::default() };
        scene_a.draws.push(base.clone());
        headless.upload(&scene_a, &mut Checker);
        let a = headless.render(&cam, &noon()).unwrap();

        let mut scene_b = Scene { zones: vec![unlit_zone()], ..Default::default() };
        scene_b.draws.push(base);
        scene_b.draws.push(MeshDraw {
            instances: Vec::new(), vertices: quad_vertices([1.0, 0.0, 0.0, 1.0], [0.06, 0.06]),
            indices: vec![0, 2, 1, 0, 3, 2],
            material: alpha_material(""),
            mask: Some("Nope.Missing".to_string()),
            kind: DrawKind::StaticMesh,
            sun_affect: false,
            pass: Pass::Layer(1),
            zone: 0,
            lightmap: None,
            modulate_2x: true,
            sector: None,
            fade: None,
            tint: None,
            actor: None,
        });
        headless.upload(&scene_b, &mut Checker);
        let b = headless.render(&cam, &noon()).unwrap();

        let pixel_a = center_pixel(&a);
        let pixel_b = center_pixel(&b);
        assert_ne!(pixel_a, pixel_b, "{pixel_a:?} == {pixel_b:?}");
        assert!(pixel_b[0] > pixel_b[1], "{pixel_b:?}");
    }

    #[test]
    fn masked_alpha_uses_the_alpha_maps_alpha_channel_to_cut_out() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let cam = camera();
        let color = [0.3, 0.3, 0.3, 1.0];
        let masked = |mask_uv: [f32; 2]| MeshDraw {
            instances: Vec::new(), vertices: quad_vertices(color, mask_uv),
            indices: vec![0, 2, 1, 0, 3, 2],
            material: alpha_material(""),
            mask: Some("Nope.Missing".to_string()),
            kind: DrawKind::StaticMesh,
            sun_affect: false,
            pass: Pass::Translucent,
            zone: 0,
            lightmap: None,
            modulate_2x: true,
            sector: None,
            fade: None,
            tint: None,
            actor: None,
        };

        let mut visible = Scene { zones: vec![unlit_zone()], ..Default::default() };
        visible.draws.push(masked([0.0625, 0.0625]));
        headless.upload(&visible, &mut Checker);
        let visible_rgba = headless.render(&cam, &noon()).unwrap();

        headless.upload(&quad(color), &mut Checker);
        let opaque_rgba = headless.render(&cam, &noon()).unwrap();

        let visible_center = center_pixel(&visible_rgba);
        let opaque_center = center_pixel(&opaque_rgba);
        for i in 0..3 {
            assert!((visible_center[i] as i32 - opaque_center[i] as i32).abs() <= 4, "{visible_center:?} vs {opaque_center:?}");
        }
        expect_rgb(opaque_center, [byte(0.6); 3], 3);

        let mut cut = Scene { zones: vec![unlit_zone()], ..Default::default() };
        cut.draws.push(masked([0.1875, 0.0625]));
        headless.upload(&cut, &mut Checker);
        let cut_rgba = headless.render(&cam, &noon()).unwrap();
        let cut_center = center_pixel(&cut_rgba);
        for i in 0..4 {
            assert!((cut_center[i] as i32 - CLEAR_RGBA[i] as i32).abs() <= 25, "{cut_center:?}");
        }
    }

    #[test]
    fn textures_sample_half_a_mip_sharper_as_the_clients_default_mip_bias() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let v = |x: f32, z: f32, u: f32, w: f32| Vertex { position: [x, 0.0, z], normal: [0.0, -1.0, 0.0], uv: [u, w], mask_uv: [0.0; 2], color: [0.25, 0.25, 0.25, 1.0] };
        let vertices = vec![v(-100.0, -100.0, 0.0, 0.0), v(100.0, -100.0, 1.0, 0.0), v(100.0, 100.0, 1.0, 1.0), v(-100.0, 100.0, 0.0, 1.0)];
        let mut scene = Scene { zones: vec![unlit_zone()], ..Default::default() };
        scene.draws.push(draw_of(vertices, DrawKind::StaticMesh, false, "Synthetic.Mips", 0));
        headless.upload(&scene, &mut WhiteOverBlackMips);
        let rgba = headless.render(&camera(), &noon()).unwrap();
        let center = center_pixel(&rgba);
        let texels_per_pixel = 64.0f32 / 37.0;
        let white_share = 1.0 - texels_per_pixel.log2().clamp(0.0, 1.0);
        let unbiased = byte(white_share * 0.5);
        assert!(center[0] > unbiased + 30, "{center:?} vs {unbiased}");
    }

    #[test]
    fn static_mesh_facing_the_noon_sun_gets_half_ambient_plus_half_sun() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let env = noon();
        let mut scene = Scene::default();
        scene.draws.push(draw_of(quad_vertices_facing([0.0, 0.0, 0.0, 1.0], [0.0; 2], sun_toward(&env)), DrawKind::StaticMesh, true, "", 0));
        headless.upload(&scene, &mut Checker);
        let rgba = headless.render(&camera(), &env).unwrap();
        let sun = scaled(env.sun_color, env.sun_brightness * 0.5 * 0.875);
        let d = sum(scaled(env.static_mesh_ambient, 0.5), sun);
        expect_rgb(center_pixel(&rgba), pixel_of(1.0, d), 3);
        assert_eq!(pixel_of(1.0, d), [235, 235, 235]);
    }

    #[test]
    fn static_mesh_facing_away_from_the_sun_shows_the_table_ambient() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let env = noon();
        let mut scene = Scene::default();
        scene.draws.push(draw_of(quad_vertices_facing([0.0, 0.0, 0.0, 1.0], [0.0; 2], env.sun_dir.to_array()), DrawKind::StaticMesh, true, "", 0));
        headless.upload(&scene, &mut Checker);
        let rgba = headless.render(&camera(), &env).unwrap();
        expect_rgb(center_pixel(&rgba), [120, 120, 120], 3);
    }

    #[test]
    fn static_mesh_outside_a_sun_zone_keeps_only_its_color_stream() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let env = noon();
        let mut scene = Scene::default();
        scene.zones.push(unlit_zone());
        scene.draws.push(draw_of(quad_vertices_facing([0.25, 0.25, 0.25, 1.0], [0.0; 2], sun_toward(&env)), DrawKind::StaticMesh, false, "", 0));
        headless.upload(&scene, &mut Checker);
        let rgba = headless.render(&camera(), &env).unwrap();
        expect_rgb(center_pixel(&rgba), [128, 128, 128], 3);
    }

    #[test]
    fn static_mesh_with_its_own_sun_affect_in_a_zone_without_sun_gets_the_ambient_alone() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let env = noon();
        let mut scene = Scene::default();
        scene.zones.push(unlit_zone());
        scene.draws.push(draw_of(quad_vertices_facing([0.0, 0.0, 0.0, 1.0], [0.0; 2], sun_toward(&env)), DrawKind::StaticMesh, true, "", 0));
        headless.upload(&scene, &mut Checker);
        let rgba = headless.render(&camera(), &env).unwrap();
        expect_rgb(center_pixel(&rgba), [120, 120, 120], 3);
    }

    #[test]
    fn static_mesh_without_its_own_sun_affect_in_a_sun_zone_gets_the_sun_alone() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let env = noon();
        let mut scene = Scene::default();
        scene.zones.push(ZoneLighting::default());
        scene.draws.push(draw_of(quad_vertices_facing([0.0, 0.0, 0.0, 1.0], [0.0; 2], sun_toward(&env)), DrawKind::StaticMesh, false, "", 0));
        headless.upload(&scene, &mut Checker);
        let rgba = headless.render(&camera(), &env).unwrap();
        let sun = scaled(env.sun_color, env.sun_brightness * 0.5 * 0.875);
        expect_rgb(center_pixel(&rgba), pixel_of(1.0, sun), 3);
    }

    #[test]
    fn static_mesh_render_1063d6a3_halves_ambient_bytes_and_animates_glow() {
        use killer_assets::scene::actor_lighting::Ambient;
        let mut headless = Headless::new(64, 64).unwrap();
        let mut scene = Scene { zones: vec![unlit_zone()], ..Default::default() };
        let mut draw = draw_of(quad_vertices([0.0, 0.0, 0.0, 1.0], [0.0; 2]), DrawKind::StaticMesh, false, "", 0);
        draw.actor = Some(ActorCull { id: 1, ambient: Ambient { vector: [0.0; 3], glow: 71 }, xy: [0.0; 2], range: f32::INFINITY, ignored_range: true, time_reactor: None, lod: None });
        scene.draws.push(draw);
        headless.upload(&scene, &mut White);
        let rgba = headless.render(&camera(), &noon()).unwrap();
        assert_eq!(&center_pixel(&rgba)[..3], [70; 3]);
        scene.draws[0].actor.as_mut().unwrap().ambient.glow = 255;
        headless.upload(&scene, &mut White);
        for (time, expected) in [(0.0, 62), (std::f32::consts::FRAC_PI_2 / 8.0, 114), (3.0 * std::f32::consts::FRAC_PI_2 / 8.0, 12)] {
            headless.renderer_mut().set_time(time);
            let rgba = headless.render(&camera(), &noon()).unwrap();
            assert_eq!(&center_pixel(&rgba)[..3], [expected; 3]);
        }
    }

    #[test]
    fn actor_facing_the_noon_sun_gets_half_ambient_plus_full_sun() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let env = noon();
        let mut scene = Scene::default();
        scene.draws.push(draw_of(quad_vertices_facing([0.0, 0.0, 0.0, 1.0], [0.0; 2], sun_toward(&env)), DrawKind::Actor, true, "Synthetic.Gray", 0));
        headless.upload(&scene, &mut Gray);
        let rgba = headless.render(&camera(), &env).unwrap();
        let sun = scaled(env.sun_color, env.sun_brightness * 0.875);
        let d = sum(scaled(env.actor_ambient, 0.5), sun);
        let expected = pixel_of(GRAY as f32 / 255.0, d);
        expect_rgb(center_pixel(&rgba), expected, 3);
        assert!(expected[0] > GRAY && expected[0] < 255, "{expected:?}");
    }

    #[test]
    fn bsp_without_sun_affect_and_half_lightmap_leaves_the_texture_unchanged() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let mut scene = Scene::default();
        scene.draws.push(draw_of(quad_vertices([1.0; 4], [0.0; 2]), DrawKind::Bsp, false, "Synthetic.Gray", 0));
        headless.upload(&scene, &mut Gray);
        let rgba = headless.render(&camera(), &noon()).unwrap();
        expect_rgb(center_pixel(&rgba), [GRAY; 3], 2);
    }

    fn lightmap_slices(values: &[u8]) -> Vec<Vec<Vec<u8>>> {
        values.iter().map(|&value| vec![[value, value, value, 255].repeat(4), [value, value, value, 255].to_vec()]).collect()
    }

    #[test]
    fn bsp_lightmap_slice_of_the_hour_multiplies_the_texture_twice() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let mut scene = Scene { zones: vec![unlit_zone()], ..Default::default() };
        scene.lightmaps.push(killer_assets::scene::LightmapGroup { width: 2, height: 2, format: TextureFormatKind::Rgba8, slices: lightmap_slices(&[64, 128, 255, 32, 32, 32, 32, 32]) });
        let mut draw = draw_of(quad_vertices([1.0; 4], [0.5; 2]), DrawKind::Bsp, false, "Synthetic.Gray", 0);
        draw.lightmap = Some(0);
        scene.draws.push(draw);
        headless.upload(&scene, &mut Gray);
        let mut env = noon();
        env.hour = 0.0;
        expect_rgb(center_pixel(&headless.render(&camera(), &env).unwrap()), [byte(GRAY as f32 / 255.0 * 2.0 * 64.0 / 255.0); 3], 2);
        env.hour = 2.99;
        expect_rgb(center_pixel(&headless.render(&camera(), &env).unwrap()), [byte(GRAY as f32 / 255.0 * 2.0 * 64.0 / 255.0); 3], 2);
        env.hour = 3.0;
        expect_rgb(center_pixel(&headless.render(&camera(), &env).unwrap()), [byte(GRAY as f32 / 255.0 * 2.0 * 128.0 / 255.0); 3], 2);
        env.hour = 6.5;
        expect_rgb(center_pixel(&headless.render(&camera(), &env).unwrap()), [255; 3], 0);
        let mut half = scene.clone();
        half.draws[0].modulate_2x = false;
        headless.upload(&half, &mut Gray);
        env.hour = 3.0;
        expect_rgb(center_pixel(&headless.render(&camera(), &env).unwrap()), [byte(GRAY as f32 / 255.0 * 128.0 / 255.0); 3], 2);
        let mut unlit = scene.clone();
        for pass in &mut unlit.draws[0].material.passes {
            pass.lit = false;
        }
        headless.upload(&unlit, &mut Gray);
        expect_rgb(center_pixel(&headless.render(&camera(), &env).unwrap()), [GRAY; 3], 1);
    }

    #[test]
    fn regenerated_lightmap_group_replaces_the_texture_the_bsp_draws_sample() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let mut scene = Scene { zones: vec![unlit_zone()], ..Default::default() };
        scene.lightmaps.push(killer_assets::scene::LightmapGroup { width: 2, height: 2, format: TextureFormatKind::Rgba8, slices: lightmap_slices(&[64, 128, 255, 32, 32, 32, 32, 32]) });
        let mut draw = draw_of(quad_vertices([1.0; 4], [0.5; 2]), DrawKind::Bsp, false, "Synthetic.Gray", 0);
        draw.lightmap = Some(0);
        scene.draws.push(draw);
        headless.upload(&scene, &mut Gray);
        let mut env = noon();
        env.hour = 0.0;
        let frame = headless.render(&camera(), &env).unwrap();
        let before = center_pixel(&frame);
        expect_rgb(before, [byte(GRAY as f32 / 255.0 * 2.0 * 64.0 / 255.0); 3], 2);
        let regenerated = killer_assets::scene::LightmapGroup { width: 2, height: 2, format: TextureFormatKind::Rgba8, slices: lightmap_slices(&[200, 128, 255, 32, 32, 32, 32, 32]) };
        headless.renderer_mut().replace_lightmap_group("", 0, &regenerated);
        let frame = headless.render(&camera(), &env).unwrap();
        let after = center_pixel(&frame);
        expect_rgb(after, [byte(GRAY as f32 / 255.0 * 2.0 * 200.0 / 255.0); 3], 2);
        assert_ne!(before, after);
    }

    #[test]
    fn unlit_static_mesh_shows_the_texture_without_ambient_sun_or_the_double_modulation() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let env = noon();
        let mut scene = Scene { zones: vec![unlit_zone()], ..Default::default() };
        scene.draws.push(draw_of(quad_vertices([0.25; 4], [0.0; 2]), DrawKind::StaticMesh, true, "Synthetic.Gray", 0));
        headless.upload(&scene, &mut Gray);
        let rgba = headless.render(&camera(), &env).unwrap();
        let lit = center_pixel(&rgba);
        assert_ne!(lit[..3], [GRAY; 3]);
        scene.draws[0].modulate_2x = false;
        headless.upload(&scene, &mut Gray);
        expect_rgb(center_pixel(&headless.render(&camera(), &env).unwrap()), [GRAY; 3], 1);
    }

    #[test]
    fn a_viewport_target_receives_the_world_without_the_overlay_and_feeds_a_ui_quad() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let env = noon();
        let mut scene = Scene::default();
        scene.draws.push(draw_of(quad_vertices([1.0; 4], [0.0; 2]), DrawKind::Bsp, true, "Synthetic.Gray", 0));
        headless.upload(&scene, &mut Gray);
        let direct = headless.render(&camera(), &env).unwrap();
        let expected = center_pixel(&direct).to_vec();
        assert_ne!(expected[..3], CLEAR_RGBA[..3]);
        let mut overlay = killer_ui::draw::DrawList::default();
        overlay.draw_texture(0, 0, 64, 64, 0, 0, 1, 1, "Synthetic.Gray", 1.0);
        headless.renderer_mut().set_ui(&overlay, &mut Gray);
        let renderer = headless.renderer_mut();
        let mut target = renderer.viewport_target((32, 32));
        assert!(renderer.render_viewport(&mut target, &camera(), &env));
        renderer.set_ui(&killer_ui::draw::DrawList::default(), &mut Gray);
        let device = renderer.device().clone();
        let queue = renderer.queue().clone();
        let readback = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: 256 * 32, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            target.texture().as_image_copy(),
            wgpu::TexelCopyBufferInfo { buffer: &readback, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(256), rows_per_image: None } },
            wgpu::Extent3d { width: 32, height: 32, depth_or_array_layers: 1 },
        );
        queue.submit([encoder.finish()]);
        let slice = readback.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| ());
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        let data = slice.get_mapped_range().unwrap();
        let inside = data[16 * 256 + 16 * 4..][..4].to_vec();
        drop(data);
        expect_rgb(&inside, [expected[0], expected[1], expected[2]], 4);
        let mut list = killer_ui::draw::DrawList::default();
        list.draw_texture(0, 0, 64, 64, 0, 0, 32, 32, "fishing:viewport", 1.0);
        renderer.set_ui_external_texture("fishing:viewport", target.view(), [32.0, 32.0]);
        renderer.set_ui(&list, &mut Gray);
        let away = Camera { eye: vec3(0.0, -300.0, 0.0), target: vec3(0.0, -600.0, 0.0), fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
        let shown = headless.render(&away, &env).unwrap();
        expect_rgb(center_pixel(&shown), [expected[0], expected[1], expected[2]], 4);
        assert_eq!(shown[..4], CLEAR_RGBA);
    }

    #[test]
    fn bsp_with_sun_affect_is_tinted_by_twice_the_bsp_ambient() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let env = noon();
        let mut scene = Scene::default();
        scene.draws.push(draw_of(quad_vertices([1.0; 4], [0.0; 2]), DrawKind::Bsp, true, "Synthetic.Gray", 0));
        headless.upload(&scene, &mut Gray);
        let rgba = headless.render(&camera(), &env).unwrap();
        let expected = env.bsp_ambient.map(|c| byte(GRAY as f32 / 255.0 * 2.0 * c));
        expect_rgb(center_pixel(&rgba), expected, 3);
    }

    #[test]
    fn terrain_adds_half_sun_by_intensity_and_half_ambient_to_the_vertex_color() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let env = noon();
        let mut scene = Scene::default();
        scene.draws.push(draw_of(quad_vertices([0.0, 0.0, 0.0, 1.0], [0.0; 2]), DrawKind::Terrain, true, "Synthetic.Gray", 0));
        headless.upload(&scene, &mut Gray);
        let rgba = headless.render(&camera(), &env).unwrap();
        let sun = scaled(env.terrain_sun.0, env.terrain_sun.1 * 0.5);
        let d = sum(sun, scaled(env.terrain_ambient, 0.5));
        expect_rgb(center_pixel(&rgba), pixel_of(GRAY as f32 / 255.0, d), 3);
    }

    #[test]
    fn terrain_intensity_zero_leaves_only_the_ambient() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let env = noon();
        let mut scene = Scene::default();
        scene.draws.push(draw_of(quad_vertices([0.0, 0.0, 0.0, 0.0], [0.0; 2]), DrawKind::Terrain, true, "Synthetic.Gray", 0));
        headless.upload(&scene, &mut Gray);
        let rgba = headless.render(&camera(), &env).unwrap();
        let d = scaled(env.terrain_ambient, 0.5);
        expect_rgb(center_pixel(&rgba), pixel_of(GRAY as f32 / 255.0, d), 3);
        assert_eq!(center_pixel(&rgba)[3], 255);
    }

    fn intensity_slices(values: [u8; 8]) -> Vec<Vec<Vec<u8>>> {
        values.iter().map(|&value| vec![[value, value, value, 255].repeat(3 * 3)]).collect()
    }

    #[test]
    fn terrain_intensity_comes_from_the_two_slices_around_the_hour() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let env = noon();
        let mut scene = Scene::default();
        scene.lightmaps.push(killer_assets::scene::LightmapGroup { width: 3, height: 3, format: TextureFormatKind::Rgba8, slices: intensity_slices([0, 0, 0, 153, 131, 0, 0, 52]) });
        let mut draw = draw_of(quad_vertices([0.0, 0.0, 0.0, 1.0], [1.5 / 2.0; 2]), DrawKind::Terrain, true, "Synthetic.Gray", 0);
        draw.lightmap = Some(0);
        scene.draws.push(draw);
        headless.upload(&scene, &mut Gray);
        let expected = |intensity: f32, hour: f32| {
            let env = Environment::from_time(&TimeEnv::parse(TIMEENV_FIXTURE), hour);
            let sun = scaled(env.terrain_sun.0, env.terrain_sun.1 * 0.5 * intensity);
            pixel_of(GRAY as f32 / 255.0, sum(sun, scaled(env.terrain_ambient, 0.5)))
        };
        let render = |headless: &mut Headless, hour: f32| {
            let mut env = env;
            env.hour = hour;
            headless.render(&camera(), &env).unwrap()
        };
        expect_rgb(center_pixel(&render(&mut headless, 12.0)), expected(142.0 / 255.0, 12.0), 3);
        expect_rgb(center_pixel(&render(&mut headless, 10.5)), expected(153.0 / 255.0, 12.0), 3);
        expect_rgb(center_pixel(&render(&mut headless, 13.5)), expected(131.0 / 255.0, 12.0), 3);
        expect_rgb(center_pixel(&render(&mut headless, 23.0)), expected(43.0 / 255.0, 12.0), 3);
    }

    fn textured_quad(uv_scale: f32) -> Scene {
        let v = |x: f32, z: f32, u: f32, w: f32| Vertex { position: [x, 0.0, z], normal: [0.0, -1.0, 0.0], uv: [u, w], mask_uv: [0.0, 0.0], color: [1.0; 4] };
        let mut scene = Scene::default();
        scene.draws.push(flat(vec![v(-100.0, -100.0, 0.0, 0.0), v(100.0, -100.0, uv_scale, 0.0), v(100.0, 100.0, uv_scale, uv_scale), v(-100.0, 100.0, 0.0, uv_scale)], 0));
        scene.draws[0].material = texture_material("Synthetic.Mip");
        scene
    }

    #[test]
    fn mip_chain_upload_renders() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let cam = camera();

        headless.upload(&textured_quad(1.0), &mut SyntheticMip);
        let near = headless.render(&cam, &noon()).unwrap();
        let near_center = center_pixel(&near);
        assert!(near_center[0] > 200, "{near_center:?}");

        headless.upload(&textured_quad(64.0), &mut SyntheticMip);
        let far = headless.render(&cam, &noon()).unwrap();
        let far_center = center_pixel(&far);
        assert!(far_center[0] < 60, "{far_center:?}");
    }

    fn facing_south(x0: f32, x1: f32, y: f32, color: [f32; 4]) -> Vec<Vertex> {
        let v = |x: f32, z: f32| Vertex { position: [x, y, z], normal: [0.0, -1.0, 0.0], uv: [0.0, 0.0], mask_uv: [0.0; 2], color };
        vec![v(x0, -1000.0), v(x1, -1000.0), v(x1, 1000.0), v(x0, 1000.0)]
    }

    fn pixel(rgba: &[u8], x: usize, y: usize) -> &[u8] {
        &rgba[(y * 64 + x) * 4..][..4]
    }

    fn viewer_in(zone: u8) -> Viewer {
        Viewer { zone, position: camera().eye, dt: 1.0 }
    }

    #[test]
    fn the_viewers_zone_fog_fades_to_its_color_between_start_and_end_in_view_depth() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let mut scene = Scene { zones: vec![unlit_zone(), ZoneLighting { fog: Some(FogParams { color: [1.0, 0.0, 0.0], start: 100.0, end: 200.0, blend_time: 1.0 }), ..unlit_zone() }], ..Default::default() };
        scene.draws.push(flat(facing_south(0.0, 100.0, -250.0, [0.0, 1.0, 0.0, 1.0]), 0));
        scene.draws.push(flat(facing_south(-1000.0, 1000.0, 0.0, [0.0, 0.0, 1.0, 1.0]), 0));
        headless.upload(&scene, &mut Checker);
        headless.renderer.set_viewer(viewer_in(1));
        headless.render(&camera(), &noon()).unwrap();
        let rgba = headless.render(&camera(), &noon()).unwrap();
        let near = pixel(&rgba, 16, 32);
        let far = pixel(&rgba, 48, 32);
        assert!(near[0] <= 2 && near[1] >= 253 && near[2] <= 2, "{near:?}");
        assert!(far[0] >= 253 && far[1] <= 2 && far[2] <= 2, "{far:?}");
        let fog = headless.renderer.fog().unwrap().clone();
        assert!(fog.enabled && fog.values.start == 100.0 && fog.values.end == 200.0);
        headless.renderer.set_viewer(viewer_in(0));
        let rgba = headless.render(&camera(), &noon()).unwrap();
        let far = pixel(&rgba, 48, 32);
        assert!(far[0] <= 2 && far[1] <= 2 && far[2] >= 253, "{far:?}");
        assert!(!headless.renderer.fog().unwrap().enabled);
    }

    #[test]
    fn a_terrain_zone_viewer_takes_the_global_fog_and_the_hazering_color() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let terrain = ZoneLighting { terrain_zone: true, fog: Some(FogParams { color: [0.0; 3], start: 3000.0, end: 15000.0, blend_time: 1.0 }), ..unlit_zone() };
        let mut scene = Scene { zones: vec![unlit_zone(), terrain], global_fog: GlobalFog { start: 100.0, end: 200.0, speed: 1.0, level: 4 }, ..Default::default() };
        scene.draws.push(flat(facing_south(-1000.0, 1000.0, 0.0, [0.0, 0.0, 1.0, 1.0]), 0));
        headless.upload(&scene, &mut Checker);
        headless.renderer.set_viewer(viewer_in(1));
        let rgba = headless.render(&camera(), &noon()).unwrap();
        let far = pixel(&rgba, 48, 32);
        expect_rgb(far, [188, 203, 224], 2);
        let fog = headless.renderer.fog().unwrap().clone();
        assert_eq!((fog.values.start, fog.values.end), (100.0, 200.0));
    }

    #[test]
    fn translucent_draws_fade_to_black_and_modulated_ones_to_gray_not_to_the_fog_color() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let fogged = ZoneLighting { fog: Some(FogParams { color: [1.0, 0.0, 0.0], start: 100.0, end: 200.0, blend_time: 1.0 }), ..unlit_zone() };
        let mut scene = Scene { zones: vec![unlit_zone(), fogged], ..Default::default() };
        let mut translucent = MeshDraw { material: blended("", BlendFactor::One, BlendFactor::InvSrcColor), pass: Pass::Translucent, ..flat(facing_south(-1000.0, 1000.0, 0.0, [1.0; 4]), 0) };
        translucent.material.passes[0].fog = l2_catalog::FogMode::Black;
        scene.draws.push(translucent);
        headless.upload(&scene, &mut Checker);
        headless.renderer.set_viewer(viewer_in(1));
        headless.render(&camera(), &noon()).unwrap();
        let far = pixel(&headless.render(&camera(), &noon()).unwrap(), 48, 32).to_vec();
        assert_eq!(&far[..3], &CLEAR_RGBA[..3], "{far:?}");
    }

    #[test]
    fn tiles_keep_their_own_zones_when_added_and_removed() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let sunlit = ZoneLighting::default();
        let mut plain = Scene { zones: vec![unlit_zone(), unlit_zone()], ..Default::default() };
        plain.draws.push(flat(facing_south(-1000.0, 1000.0, 0.0, [0.0, 0.0, 0.3, 1.0]), 1));
        let mut lit = Scene { zones: vec![unlit_zone(), sunlit], ..Default::default() };
        lit.draws.push(flat(facing_south(-1000.0, 1000.0, 0.0, [0.0, 0.0, 0.3, 1.0]), 1));
        headless.renderer.clear();
        headless.renderer.add_tile("a", &plain, &mut Checker);
        headless.renderer.add_tile("b", &lit, &mut Checker);
        assert_eq!(headless.renderer.tiles().collect::<Vec<_>>(), ["a", "b"]);
        headless.renderer.set_tile_visible("a", false);
        let lit_pixel = center_pixel(&headless.render(&camera(), &noon()).unwrap()).to_vec();
        headless.renderer.set_tile_visible("a", true);
        headless.renderer.set_tile_visible("b", false);
        let plain_pixel = center_pixel(&headless.render(&camera(), &noon()).unwrap()).to_vec();
        assert!(lit_pixel[0] > plain_pixel[0] + 20 && lit_pixel[2] > plain_pixel[2], "{lit_pixel:?} vs {plain_pixel:?}");
        headless.renderer.remove_tile("a");
        headless.renderer.set_tile_visible("b", true);
        assert_eq!(headless.renderer.tiles().collect::<Vec<_>>(), ["b"]);
        assert_eq!(center_pixel(&headless.render(&camera(), &noon()).unwrap()), &lit_pixel[..]);
        headless.renderer.remove_tile("b");
        assert_eq!(center_pixel(&headless.render(&camera(), &noon()).unwrap()), &CLEAR_RGBA[..]);
    }

    #[test]
    fn sky_zone_is_seen_from_the_sky_location_with_the_camera_rotation_only() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let sky_location = [100_000.0, 0.0, 0.0];
        let mut scene = Scene { zones: vec![ZoneLighting::default(), ZoneLighting { is_sky: true, sky_location, ..ZoneLighting::default() }], ..Default::default() };
        let v = |x: f32, z: f32| Vertex { position: [100_000.0 + x, 300.0, z], normal: [0.0, -1.0, 0.0], uv: [0.0, 0.0], mask_uv: [0.0; 2], color: [1.0, 0.5, 0.0, 1.0] };
        scene.draws.push(flat(vec![v(-100.0, -100.0), v(100.0, -100.0), v(100.0, 100.0), v(-100.0, 100.0)], 1));
        headless.upload(&scene, &mut Checker);
        let here = headless.render(&camera(), &noon()).unwrap();
        let moved = Camera { eye: vec3(5000.0, 700.0, 100.0), target: vec3(5000.0, 1000.0, 100.0), ..camera() };
        let there = headless.render(&moved, &noon()).unwrap();
        assert_eq!(center_pixel(&here), center_pixel(&there));
        assert_ne!(center_pixel(&here), &CLEAR_RGBA[..]);
        assert_eq!(&here[..4], &CLEAR_RGBA[..]);
    }

    fn sky_box_material() -> Material {
        let mut material = texture_material("");
        material.stages.push(l2_catalog::Stage { source: l2_catalog::StageSource::Factor, uv: Vec::new(), coord_source: l2_catalog::CoordSource::Stream(0), color_op: l2_catalog::ColorOp::Modulate, alpha_op: l2_catalog::AlphaOp::Modulate, swapped: false });
        material.passes[0].stages = 0..2;
        material.passes[0].tfactor = Some([206, 150, 0, 255]);
        material.passes[0].lit = false;
        material.color_modifier = Some("L2_Skies.Shaders.SkybackgroundColor".into());
        material
    }

    #[test]
    fn the_sky_box_takes_the_hours_sky_box_color_instead_of_the_stored_one() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let sky_location = [100_000.0, 0.0, 0.0];
        let mut scene = Scene { zones: vec![unlit_zone(), ZoneLighting { is_sky: true, sky_location, ..ZoneLighting::default() }], ..Default::default() };
        let v = |x: f32, z: f32| Vertex { position: [100_000.0 + x, 300.0, z], normal: [0.0, -1.0, 0.0], uv: [0.0, 0.0], mask_uv: [0.0; 2], color: [1.0; 4] };
        let vertices = vec![v(-100.0, -100.0), v(100.0, -100.0), v(100.0, 100.0), v(-100.0, 100.0)];
        let mut draw = MeshDraw { material: sky_box_material(), ..flat(vertices, 1) };
        draw.tint = Some(killer_assets::scene::sky::SkyTint::SkyBox);
        scene.draws.push(draw);
        headless.upload(&scene, &mut White);
        let env = noon();
        let rgba = headless.render(&camera(), &env).unwrap();
        expect_rgb(center_pixel(&rgba), [66, 124, 176], 2);
        let midnight = Environment::from_time(&TimeEnv::parse(TIMEENV_FIXTURE), 0.0);
        let rgba = headless.render(&camera(), &midnight).unwrap();
        expect_rgb(center_pixel(&rgba), [40, 49, 70], 2);
    }

    #[test]
    fn the_sun_sprite_is_an_additive_quad_of_the_hours_scale_and_color() {
        let Ok(mut headless) = Headless::new(256, 256) else { return };
        let sky_location = [100_000.0, 0.0, 0.0];
        let mut scene = Scene { zones: vec![unlit_zone(), ZoneLighting { is_sky: true, sky_location, ..unlit_zone() }], ..Default::default() };
        let material = killer_assets::scene::sky::body_material("Sun");
        scene.sky_bodies.push(killer_assets::scene::sky::SkyBodyDraw { kind: killer_assets::scene::sky::BodyKind::Sun, env_type: 0, origin: sky_location, radius: 350.0, half_width_per_scale: 128.0, material, zone: 1 });
        headless.upload(&scene, &mut White);
        let env = noon();
        let sun_center = Vec3::from(sky_location) + env.sky.sun_body * 350.0;
        let cam = Camera { eye: Vec3::from(sky_location), target: sun_center, fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
        let rgba = headless.render(&cam, &env).unwrap();
        let center = &rgba[(128 * 256 + 128) * 4..][..3];
        let added = |i: usize| (CLEAR_RGBA[i] as i32 + [255, 219, 151][i]).min(255) as u8;
        expect_rgb(center, [added(0), added(1), added(2)], 3);
        let lit = |x: usize| rgba[(128 * 256 + x) * 4] > 100;
        let width = (0..256).filter(|&x| lit(x)).count() as f32;
        let expected = 2.0 * (128.0 * env.sky.sun_scale / 350.0) * (128.0 / 30f32.to_radians().tan());
        assert!((width - expected).abs() <= 4.0, "{width} vs {expected}");
        let small_hours = Environment::from_time(&TimeEnv::parse(TIMEENV_FIXTURE), 3.0);
        let rgba = headless.render(&cam, &small_hours).unwrap();
        assert_eq!(&rgba[(128 * 256 + 128) * 4..][..3], &CLEAR_RGBA[..3]);
    }

    #[test]
    fn world_draws_cover_the_sky() {
        let Ok(mut headless) = Headless::new(64, 64) else { return };
        let mut scene = Scene { zones: vec![unlit_zone(), ZoneLighting { is_sky: true, sky_location: [100_000.0, 0.0, 0.0], ..ZoneLighting::default() }], ..Default::default() };
        let v = |x: f32, z: f32| Vertex { position: [100_000.0 + x, 10.0, z], normal: [0.0, -1.0, 0.0], uv: [0.0, 0.0], mask_uv: [0.0; 2], color: [1.0, 0.5, 0.0, 1.0] };
        scene.draws.push(flat(vec![v(-100.0, -100.0), v(100.0, -100.0), v(100.0, 100.0), v(-100.0, 100.0)], 1));
        scene.draws.push(flat(facing_south(-1000.0, 1000.0, 0.0, [0.0, 0.0, 1.0, 1.0]), 0));
        headless.upload(&scene, &mut Checker);
        let rgba = headless.render(&camera(), &noon()).unwrap();
        let c = center_pixel(&rgba);
        assert!(c[0] <= 2 && c[1] <= 2 && c[2] >= 253, "{c:?}");
    }
}
