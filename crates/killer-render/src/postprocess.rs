use wgpu::util::DeviceExt;

pub fn supported(device: &wgpu::Device) -> bool {
    let required = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
    [wgpu::TextureFormat::Rgba16Float, wgpu::TextureFormat::R32Float, crate::pipeline::COLOR_FORMAT]
        .into_iter()
        .all(|format| format.guaranteed_format_features(device.features()).allowed_usages.contains(required))
        && wgpu::TextureFormat::Rgba16Float
            .guaranteed_format_features(device.features())
            .flags
            .contains(wgpu::TextureFormatFeatureFlags::BLENDABLE)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PostProcessSettings {
    configuration_supported: bool,
    pub glow: bool,
    pub hdr: bool,
    pub glow_type: i32,
    pub blur_type: i32,
    pub opacity: u8,
    pub luminance: f32,
    pub middle_gray: f32,
    pub white_cutoff: f32,
    pub threshold: f32,
    pub bloom_scale: f32,
    pub rgb_cutoff: f32,
    pub gray: f32,
    pub coefficient: f32,
    pub exp_base: f32,
    pub exp_coefficient: f32,
    pub clamp_min: f32,
    pub clamp_max: f32,
}

impl PostProcessSettings {
    pub fn read(index: i32, keep_minimum: bool, env: &killer_ui::ini::IniFile) -> Self {
        let section = format!("S{index}");
        let number = |key, default| env.get(&section, key).map_or(default, native_float);
        let active = index > 0 && !keep_minimum;
        let boolean = |key| {
            let units = env.get(&section, key).map(|value| value.encode_utf16().collect::<Vec<_>>());
            let supported = units.as_ref().is_none_or(|units| units.iter().take(79).all(|unit| *unit <= 255));
            let value = killer_ui::ini_value::config_bool(
                units.as_deref(),
                |units| {
                    units
                        .iter()
                        .copied()
                        .map(|unit| if (65..=90).contains(&unit) { unit + 32 } else { unit })
                        .eq("true".encode_utf16())
                },
                |unit| matches!(unit, 9..=13 | 32 | 160),
            )
            .unwrap_or(false);
            (value, supported)
        };
        let glow = boolean("Glow");
        let hdr = boolean("Hdr");
        Self {
            configuration_supported: glow.1 && hdr.1,
            glow: active && glow.0,
            hdr: active && hdr.0,
            glow_type: env.int(&section, "GlowType").unwrap_or(1),
            blur_type: env.int(&section, "FinalBlendBlurType").unwrap_or(1),
            opacity: env.int(&section, "FinalBlendOpacity").unwrap_or(85) as u8,
            luminance: number("Luminance", 0.08),
            middle_gray: number("MiddleGray", 0.18),
            white_cutoff: number("WhiteCutoff", 0.8),
            threshold: number("Threshold", 0.35),
            bloom_scale: number("BloomScale", 1.5),
            rgb_cutoff: number("RGBCutoff", 0.8),
            gray: number("Gray", 0.6),
            coefficient: number("FinalCoef", 3.5),
            exp_base: number("ExpBase", 0.98),
            exp_coefficient: number("ExpCoef", 30.0),
            clamp_min: number("ClampMin", 0.25),
            clamp_max: number("ClampMax", 0.75),
        }
    }

    pub fn validate(self) -> Result<(), &'static str> {
        if self.configuration_supported {
            Ok(())
        } else {
            Err("postprocess boolean requires native locale classification")
        }
    }

    pub fn active(self) -> bool {
        self.glow || self.hdr
    }
}

fn native_float(value: &str) -> f32 {
    let value = value.trim_start();
    value
        .char_indices()
        .map(|(index, _)| index)
        .chain(std::iter::once(value.len()))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .find_map(|end| value[..end].parse::<f64>().ok())
        .unwrap_or(0.0) as f32
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Parameters {
    mode: [f32; 4],
    glow: [f32; 4],
    bloom: [f32; 4],
    hdr: [f32; 4],
    adaptation: [f32; 4],
}

struct Frames {
    size: (u32, u32),
    scene: wgpu::TextureView,
    glow: [wgpu::TextureView; 5],
    combined: wgpu::TextureView,
    hdr_output: wgpu::TextureView,
}

pub(crate) struct PostProcess {
    pub settings: PostProcessSettings,
    pub scene_format: wgpu::TextureFormat,
    layout: wgpu::BindGroupLayout,
    color_pipeline: wgpu::RenderPipeline,
    luminance_pipeline: wgpu::RenderPipeline,
    frames: Option<Frames>,
    luminance: [wgpu::TextureView; 6],
    history: usize,
    last_time: f32,
    hdr_initialized: bool,
}

impl PostProcess {
    pub fn new(device: &wgpu::Device, settings: PostProcessSettings, time: f32) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("native postprocess"),
            entries: &[
                texture_entry(0),
                texture_entry(1),
                texture_entry(2),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("native postprocess"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("native postprocess"),
            source: wgpu::ShaderSource::Wgsl(include_str!("postprocess.wgsl").into()),
        });
        let pipeline = |format| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("native postprocess"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            })
        };
        Self {
            settings,
            scene_format: if settings.hdr { wgpu::TextureFormat::Rgba16Float } else { crate::pipeline::COLOR_FORMAT },
            color_pipeline: pipeline(crate::pipeline::COLOR_FORMAT),
            luminance_pipeline: pipeline(wgpu::TextureFormat::R32Float),
            layout,
            frames: None,
            luminance: [64, 16, 4, 1, 1, 1].map(|size| target(device, (size, size), wgpu::TextureFormat::R32Float)),
            history: 4,
            last_time: time,
            hdr_initialized: false,
        }
    }

    pub fn set_settings(&mut self, device: &wgpu::Device, settings: PostProcessSettings) {
        self.settings = settings;
        let format = if settings.hdr { wgpu::TextureFormat::Rgba16Float } else { crate::pipeline::COLOR_FORMAT };
        if self.scene_format != format {
            if let Some(frames) = &mut self.frames {
                frames.scene = target(device, frames.size, format);
            }
            self.scene_format = format;
        }
    }

    pub fn input(&mut self, device: &wgpu::Device, size: (u32, u32)) -> wgpu::TextureView {
        if self.frames.as_ref().is_none_or(|frames| frames.size != size) {
            self.frames = Some(Frames {
                size,
                scene: target(device, size, self.scene_format),
                glow: [1, 4, 4, 2, 2]
                    .map(|divisor| target(device, (size.0 / divisor, size.1 / divisor), crate::pipeline::COLOR_FORMAT)),
                combined: target(device, size, crate::pipeline::COLOR_FORMAT),
                hdr_output: target(device, size, crate::pipeline::COLOR_FORMAT),
            });
        }
        self.frames
            .as_ref()
            .map(|frames| frames.scene.clone())
            .unwrap_or_else(|| target(device, size, self.scene_format))
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        output: &wgpu::TextureView,
        time: f32,
        trilinear: bool,
    ) {
        let Some(frames) = &self.frames else { return };
        let settings = self.settings;
        if settings.hdr && !self.hdr_initialized {
            self.last_time = time;
            self.hdr_initialized = true;
        }
        let parameters = Parameters {
            mode: [0.0, f32::from(u8::from(trilinear)), 0.0, 0.0],
            glow: [settings.luminance, settings.middle_gray, settings.white_cutoff, settings.threshold],
            bloom: [settings.bloom_scale, settings.rgb_cutoff, f32::from(settings.opacity) / 255.0, 0.0],
            hdr: [settings.gray, settings.coefficient, settings.clamp_min, settings.clamp_max],
            adaptation: [settings.exp_base, time - self.last_time, settings.exp_coefficient, 0.0],
        };
        let mut draw = |mode, textures: [&wgpu::TextureView; 3], output: &wgpu::TextureView, float: bool| {
            let params = Parameters { mode: [mode as f32, parameters.mode[1], 0.0, 0.0], ..parameters };
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("native postprocess parameters"),
                contents: bytemuck::bytes_of(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            });
            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("native postprocess"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(textures[0]) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(textures[1]) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(textures[2]) },
                    wgpu::BindGroupEntry { binding: 3, resource: buffer.as_entire_binding() },
                ],
            });
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("native postprocess"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            pass.set_pipeline(if float { &self.luminance_pipeline } else { &self.color_pipeline });
            pass.set_bind_group(0, &group, &[]);
            pass.draw(0..3, 0..1);
        };
        let scene = if settings.hdr {
            let lum = &self.luminance;
            draw(9, [&frames.scene; 3], &lum[0], true);
            for i in 0..3 {
                draw(if i == 2 { 11 } else { 10 }, [&lum[i]; 3], &lum[i + 1], true);
            }
            let next = if self.history == 4 { 5 } else { 4 };
            draw(12, [&lum[self.history], &lum[3], &lum[3]], &lum[next], true);
            draw(13, [&frames.scene, &lum[next], &lum[3]], &frames.hdr_output, false);
            self.history = next;
            self.last_time = time;
            &frames.hdr_output
        } else {
            &frames.scene
        };
        if settings.glow {
            let g = &frames.glow;
            for i in [0, 1, 3] {
                draw(0, [scene; 3], &g[i], false);
            }
            if matches!(settings.glow_type, 1 | 2) {
                draw(settings.glow_type, [&g[1]; 3], &g[2], false);
            }
            for (mode, source, dest) in [(3, 2, 1), (4, 1, 2), (3, 2, 1), (4, 1, 2)] {
                draw(mode, [&g[source]; 3], &g[dest], false);
            }
            if matches!(settings.blur_type, 1 | 2) {
                draw(settings.blur_type + 4, [&g[3]; 3], &g[4], false);
            }
            draw(7, [&g[0], &g[1], &g[1]], &frames.combined, false);
            draw(8, [&frames.combined, &g[4], &g[4]], output, false);
        } else {
            draw(0, [scene; 3], output, false);
        }
    }
}

fn texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn target(device: &wgpu::Device, size: (u32, u32), format: wgpu::TextureFormat) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("native postprocess target"),
            size: wgpu::Extent3d { width: size.0.max(1), height: size.1.max(1), depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
        .create_view(&Default::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_1050c4b0_and_1050c720_read_every_native_preset() {
        let env = killer_ui::ini::parse_ini(killer_assets::options::ENV_INT_FIXTURE);
        for index in 1..=5 {
            let settings = PostProcessSettings::read(index, false, &env);
            assert!(settings.glow);
            assert_eq!(settings.hdr, index >= 3);
            assert_eq!(settings.glow_type, 2);
            assert_eq!(settings.blur_type, 2);
            assert!(!PostProcessSettings::read(index, true, &env).active());
        }
        let last = PostProcessSettings::read(5, false, &env);
        assert_eq!(last.clamp_max, 0.8);
        let missing = PostProcessSettings::read(7, false, &env);
        assert!(!missing.active());
        assert_eq!((missing.opacity, missing.glow_type, missing.blur_type), (85, 1, 1));
        assert_eq!(
            (missing.gray, missing.coefficient, missing.exp_base, missing.exp_coefficient),
            (0.6, 3.5, 0.98, 30.0)
        );
    }

    #[test]
    fn native_postprocess_graph_runs_all_presets_before_present() {
        use crate::{
            camera::Camera,
            environment::Environment,
            headless::{CLEAR_RGBA, Headless},
        };
        let env = killer_ui::ini::parse_ini(killer_assets::options::ENV_INT_FIXTURE);
        let camera = Camera {
            eye: glam::vec3(0.0, -300.0, 0.0),
            target: glam::Vec3::ZERO,
            fov_x_deg: 60.0,
            near: 1.0,
            far: 5000.0,
        };
        let mut gpu = Headless::new(16, 16).unwrap();
        for index in 1..=5 {
            let settings = PostProcessSettings::read(index, false, &env);
            gpu.renderer_mut().set_postprocess(settings).unwrap();
            let pixels = gpu.render(&camera, &Environment::default()).unwrap();
            let rgb = CLEAR_RGBA[..3].iter().map(|v| f32::from(*v) / 255.0).collect::<Vec<_>>();
            let luminance = rgb[0] * 0.27 + rgb[1] * 0.67 + rgb[2] * 0.06 + 0.0001;
            for pixel in pixels.as_chunks::<4>().0 {
                for channel in 0..3 {
                    let expected = if settings.hdr {
                        let factor = 0.166 * (2.0 * settings.gray + 3.0 * settings.clamp_min + luminance)
                            / (settings.clamp_min + 0.001);
                        let value = rgb[channel] * factor;
                        settings.coefficient * (value / (1.0 + value)).powi(2) * 255.0
                    } else {
                        f32::from(CLEAR_RGBA[channel])
                    };
                    assert!(
                        (f32::from(pixel[channel]) - expected).abs() <= 2.0,
                        "preset {index} channel {channel}: {} expected {expected}",
                        pixel[channel]
                    );
                }
            }
        }
        gpu.renderer_mut().set_postprocess(PostProcessSettings::read(0, false, &env)).unwrap();
        let pixels = gpu.render(&camera, &Environment::default()).unwrap();
        assert!(pixels.as_chunks::<4>().0.iter().all(|pixel| *pixel == CLEAR_RGBA));
    }
}
