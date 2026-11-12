use std::{path::Path, sync::Arc};
use winit::{dpi::{LogicalSize, PhysicalPosition}, event_loop::ActiveEventLoop, window::{Window, WindowLevel}};

pub const TEXTURE_DIR: &str = "systextures";
pub const TRUE_COLOR_PREFIX: &str = "sp_32b_";
pub const PALETTE_PREFIX: &str = "sp_256_";
pub const SEARCH_LIMIT: u32 = 100;
pub const FIRST_CRT_RAND: i32 = 41;
pub const TRUE_COLOR_SYSTEMS: [i32; 2] = [3, 4];
pub const RUNNING_OS_AT_SPLASH: i32 = 0;

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,
}

pub fn choose(client_root: &Path, running_os: i32, rand: i32) -> Option<std::path::PathBuf> {
    let prefix = if TRUE_COLOR_SYSTEMS.contains(&running_os) { TRUE_COLOR_PREFIX } else { PALETTE_PREFIX };
    let dir = client_root.join(TEXTURE_DIR);
    let found: Vec<_> = (0..SEARCH_LIMIT).map(|index| dir.join(format!("{prefix}{index:02}.bmp"))).filter(|path| path.is_file()).collect();
    let count = i32::try_from(found.len()).ok().filter(|count| *count > 0)?;
    found.into_iter().nth((rand % count) as usize)
}

fn le_u16(bytes: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(bytes.get(at..at + 2)?.try_into().ok()?))
}

fn le_u32(bytes: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?))
}

pub fn decode_bmp(bytes: &[u8]) -> Option<Image> {
    if bytes.get(..2)? != b"BM" {
        return None;
    }
    let data = le_u32(bytes, 10)? as usize;
    let header = le_u32(bytes, 14)? as usize;
    let width = le_u32(bytes, 18)? as i32;
    let height = le_u32(bytes, 22)? as i32;
    let bits = le_u16(bytes, 28)?;
    let compression = le_u32(bytes, 30)?;
    let colors = le_u32(bytes, 46)? as usize;
    if compression != 0 || width <= 0 || height == 0 {
        return None;
    }
    let (width, top_down, rows) = (width as usize, height < 0, height.unsigned_abs() as usize);
    let stride = (width * usize::from(bits)).div_ceil(32) * 4;
    let palette: Vec<u32> = if bits <= 8 {
        let count = if colors == 0 { 1usize << bits } else { colors };
        (0..count).map(|index| {
            let at = 14 + header + index * 4;
            Some(u32::from(*bytes.get(at + 2)?) << 16 | u32::from(*bytes.get(at + 1)?) << 8 | u32::from(*bytes.get(at)?))
        }).collect::<Option<_>>()?
    } else {
        Vec::new()
    };
    let mut pixels = vec![0u32; width * rows];
    for row in 0..rows {
        let line = bytes.get(data + row * stride..data + (row + 1) * stride)?;
        let target = if top_down { row } else { rows - 1 - row };
        for column in 0..width {
            pixels[target * width + column] = match bits {
                8 => *palette.get(usize::from(line[column]))?,
                24 => u32::from(line[column * 3 + 2]) << 16 | u32::from(line[column * 3 + 1]) << 8 | u32::from(line[column * 3]),
                32 => u32::from(line[column * 4 + 2]) << 16 | u32::from(line[column * 4 + 1]) << 8 | u32::from(line[column * 4]),
                _ => return None,
            };
        }
    }
    Some(Image { width: width as u32, height: rows as u32, pixels })
}

pub fn load(client_root: &Path) -> Option<Image> {
    let path = choose(client_root, RUNNING_OS_AT_SPLASH, FIRST_CRT_RAND)?;
    let bytes = std::fs::read(&path).ok()?;
    let bytes = l2_crypto::decrypt(&bytes, &path).ok()?;
    decode_bmp(&bytes)
}

const SHADER: &str = "
@group(0) @binding(0) var image: texture_2d<f32>;
@group(0) @binding(1) var sampler_state: sampler;
struct Vertex { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn vertex_main(@builtin(vertex_index) index: u32) -> Vertex {
    var corners = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    var out: Vertex;
    out.position = vec4<f32>(corners[index], 0.0, 1.0);
    out.uv = vec2<f32>((corners[index].x + 1.0) * 0.5, (1.0 - corners[index].y) * 0.5);
    return out;
}
@fragment fn fragment_main(in: Vertex) -> @location(0) vec4<f32> {
    return textureSample(image, sampler_state, in.uv);
}
";

pub fn premultiplied(image: &Image) -> Vec<u8> {
    let key = image.pixels.first().copied().unwrap_or(0) & 0x00ff_ffff;
    image.pixels.iter().flat_map(|pixel| {
        let color = pixel & 0x00ff_ffff;
        if color == key { [0, 0, 0, 0] } else { [(color >> 16) as u8, (color >> 8) as u8, color as u8, 255] }
    }).collect()
}

pub struct Splash {
    pub window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
}

impl Splash {
    pub fn open(event_loop: &ActiveEventLoop, client_root: &Path) -> Option<Self> {
        let image = load(client_root)?;
        let mut attributes = Window::default_attributes()
            .with_title("Lineage II")
            .with_decorations(false)
            .with_resizable(false)
            .with_transparent(true)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_inner_size(LogicalSize::new(image.width, image.height));
        #[cfg(target_os = "macos")]
        {
            use winit::platform::macos::WindowAttributesExtMacOS;
            attributes = attributes.with_has_shadow(false);
        }
        if let Some(monitor) = event_loop.primary_monitor() {
            let size = monitor.size();
            let scale = monitor.scale_factor();
            let (width, height) = (f64::from(image.width) * scale, f64::from(image.height) * scale);
            attributes = attributes.with_position(PhysicalPosition::new(
                monitor.position().x + ((f64::from(size.width) - width) / 2.0) as i32,
                monitor.position().y + ((f64::from(size.height) - height) / 2.0) as i32,
            ));
        }
        let window = Arc::new(event_loop.create_window(attributes).ok()?);
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).ok()?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions { compatible_surface: Some(&surface), ..Default::default() })).ok()?;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()?;
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats.iter().copied().find(|format| !format.is_srgb()).or_else(|| caps.formats.first().copied())?;
        let alpha_mode = [wgpu::CompositeAlphaMode::PreMultiplied, wgpu::CompositeAlphaMode::PostMultiplied].into_iter().find(|mode| caps.alpha_modes.contains(mode)).unwrap_or(wgpu::CompositeAlphaMode::Auto);
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Auto,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: image.width, height: image.height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            texture.as_image_copy(),
            &premultiplied(&image),
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(image.width * 4), rows_per_image: Some(image.height) },
            wgpu::Extent3d { width: image.width, height: image.height, depth_or_array_layers: 1 },
        );
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor { mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, ..Default::default() });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&texture.create_view(&Default::default())) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Wgsl(SHADER.into()) });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: None, bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: Some("vertex_main"), buffers: &[], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_main"),
                targets: &[Some(wgpu::ColorTargetState { format, blend: None, write_mask: wgpu::ColorWrites::ALL })],
                compilation_options: Default::default(),
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        let mut splash = Self { window, surface, device, queue, config, pipeline, bind_group };
        splash.paint();
        Some(splash)
    }

    pub fn resize(&mut self) {
        let size = self.window.inner_size();
        self.config.width = size.width.max(1);
        self.config.height = size.height.max(1);
        self.surface.configure(&self.device, &self.config);
    }

    pub fn paint(&mut self) {
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            _ => return,
        };
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &view, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store }, depth_slice: None })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        self.queue.submit([encoder.finish()]);
        self.queue.present(frame);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bitmap(bits: u16, pixels: &[u8], palette: &[[u8; 4]], width: i32, height: i32) -> Vec<u8> {
        let data = 54 + palette.len() * 4;
        let mut out = vec![0u8; data];
        out[..2].copy_from_slice(b"BM");
        out[10..14].copy_from_slice(&(data as u32).to_le_bytes());
        out[14..18].copy_from_slice(&40u32.to_le_bytes());
        out[18..22].copy_from_slice(&width.to_le_bytes());
        out[22..26].copy_from_slice(&height.to_le_bytes());
        out[28..30].copy_from_slice(&bits.to_le_bytes());
        out[46..50].copy_from_slice(&(palette.len() as u32).to_le_bytes());
        for (index, color) in palette.iter().enumerate() {
            out[54 + index * 4..58 + index * 4].copy_from_slice(color);
        }
        out.extend_from_slice(pixels);
        out
    }

    #[test]
    fn bmp_8_bit_palette_rows_are_bottom_up_and_padded_to_four_bytes() {
        let bytes = bitmap(8, &[0, 1, 0, 0, 1, 0, 0, 0], &[[1, 2, 3, 0], [10, 20, 30, 0]], 2, 2);
        let image = decode_bmp(&bytes).unwrap();
        assert_eq!((image.width, image.height), (2, 2));
        assert_eq!(image.pixels, vec![0x1e140a, 0x030201, 0x030201, 0x1e140a]);
    }

    #[test]
    fn the_corner_color_becomes_a_transparent_key() {
        let image = Image { width: 2, height: 1, pixels: vec![0x00ff00, 0x102030] };
        assert_eq!(premultiplied(&image), vec![0, 0, 0, 0, 0x10, 0x20, 0x30, 255]);
    }

    #[test]
    fn bmp_32_bit_reads_bgr_and_ignores_alpha() {
        let image = decode_bmp(&bitmap(32, &[1, 2, 3, 255], &[], 1, 1)).unwrap();
        assert_eq!(image.pixels, vec![0x030201]);
        assert!(decode_bmp(b"nope").is_none());
    }

    #[test]
    fn splash_c4_picks_the_rand_modulo_of_the_existing_files() {
        let dir = std::env::temp_dir().join(format!("killer-splash-{}", std::process::id()));
        std::fs::create_dir_all(dir.join(TEXTURE_DIR)).unwrap();
        for index in 1..=11 {
            std::fs::write(dir.join(TEXTURE_DIR).join(format!("sp_256_{index:02}.bmp")), b"x").unwrap();
            std::fs::write(dir.join(TEXTURE_DIR).join(format!("sp_32b_{index:02}.bmp")), b"x").unwrap();
        }
        assert!(choose(&dir, 0, FIRST_CRT_RAND).unwrap().ends_with("sp_256_09.bmp"));
        assert!(choose(&dir, 4, 3).unwrap().ends_with("sp_32b_04.bmp"));
        assert!(choose(&dir, 0, 11).unwrap().ends_with("sp_256_01.bmp"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    #[ignore]
    fn the_client_splash_bitmaps_decode_to_their_original_size() {
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").unwrap());
        for prefix in [PALETTE_PREFIX, TRUE_COLOR_PREFIX] {
            for index in 1..=11 {
                let path = root.join(TEXTURE_DIR).join(format!("{prefix}{index:02}.bmp"));
                let bytes = std::fs::read(&path).unwrap();
                let image = decode_bmp(&l2_crypto::decrypt(&bytes, &path).unwrap()).unwrap();
                println!("SPLASH {prefix}{index:02} {}x{}", image.width, image.height);
            }
        }
    }
}
