use killer_assets::textures::{TextureData, TextureFormatKind};
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::sync::{Arc, Weak};
use ue2_assets::{Mip, TextureFormat, decode_rgba};
use wgpu::util::DeviceExt;

pub use killer_assets::textures::TextureSource;

pub type SharedView = Arc<wgpu::TextureView>;

const SWEEP_FLOOR: usize = 256;

pub struct GpuTextures {
    views: FxHashMap<String, (Weak<wgpu::TextureView>, [f32; 2])>,
    white: SharedView,
    white_array: SharedView,
    white_cube: SharedView,
    bc_supported: bool,
    texture_detail: Option<i32>,
    wet: RefCell<WetRuntime>,
    sweep_at: usize,
    pub no_fractal_anim: bool,
}

struct WetState {
    definition: Arc<l2_catalog::WetTexture>,
    simulation: ue2_assets::wet::WaterSimulation,
    prime_current: u8,
}

struct WetTarget {
    path: String,
    texture: wgpu::Texture,
    layer: u32,
}

struct WetRuntime {
    states: FxHashMap<String, WetState>,
    targets: FxHashMap<u32, Vec<WetTarget>>,
    random: ue2_assets::wet::FireRandom,
}

impl Default for WetRuntime {
    fn default() -> Self {
        Self { states: FxHashMap::default(), targets: FxHashMap::default(), random: ue2_assets::wet::FireRandom::from_seed(1) }
    }
}

fn white_data() -> Arc<TextureData> {
    Arc::new(TextureData { width: 1, height: 1, format: TextureFormatKind::Rgba8, mips: vec![vec![255, 255, 255, 255]] })
}

impl GpuTextures {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let bc_supported = device.features().contains(wgpu::Features::TEXTURE_COMPRESSION_BC);
        let white = white_data();
        let (white_view, _) = upload_layers(device, queue, std::slice::from_ref(&white), wgpu::TextureViewDimension::D2, bc_supported);
        let (white_array, _) = upload_layers(device, queue, std::slice::from_ref(&white), wgpu::TextureViewDimension::D2Array, bc_supported);
        let faces: Vec<Arc<TextureData>> = (0..6).map(|_| white.clone()).collect();
        let (white_cube, _) = upload_layers(device, queue, &faces, wgpu::TextureViewDimension::Cube, bc_supported);
        Self { views: FxHashMap::default(), white: white_view, white_array, white_cube, bc_supported, texture_detail: None, wet: RefCell::new(WetRuntime::default()), sweep_at: SWEEP_FLOOR, no_fractal_anim: false }
    }

    fn cached(&self, key: &str) -> Option<(SharedView, [f32; 2])> {
        let (view, size) = self.views.get(key)?;
        Some((view.upgrade()?, *size))
    }

    fn remember(&mut self, key: String, entry: &(SharedView, [f32; 2])) {
        self.views.insert(key, (Arc::downgrade(&entry.0), entry.1));
        if self.views.len() >= self.sweep_at {
            self.views.retain(|_, (view, _)| view.strong_count() > 0);
            self.sweep_at = (self.views.len() * 2).max(SWEEP_FLOOR);
        }
    }

    pub fn white(&self) -> SharedView {
        self.white.clone()
    }

    pub fn white_array(&self) -> SharedView {
        self.white_array.clone()
    }

    pub fn white_cube(&self) -> SharedView {
        self.white_cube.clone()
    }

    pub fn len(&self) -> usize {
        self.views.values().filter(|(view, _)| view.strong_count() > 0).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn collect_unused(&mut self) {
        self.views.retain(|_, (view, _)| view.strong_count() > 0);
        self.sweep_at = (self.views.len() * 2).max(SWEEP_FLOOR);
        if self.wet.get_mut().targets.is_empty() {
            *self.wet.get_mut() = WetRuntime::default();
        }
    }

    pub fn set_texture_detail(&mut self, detail: i32) {
        self.texture_detail = Some(detail);
        self.views.clear();
        self.wet.get_mut().targets.clear();
    }

    fn texture_data(&self, source: &mut dyn TextureSource, path: &str) -> Arc<TextureData> {
        if path.is_empty() { return white_data(); }
        let data = source.texture(path);
        let Some(detail) = self.texture_detail else { return data; };
        let (minimum, maximum) = source.texture_lod(path).unwrap_or_else(|| {
            let maximum = (0..data.mips.len()).position(|level| {
                let w = (data.width >> level).max(1);
                let h = (data.height >> level).max(1);
                !(w >= 64 && (h >= 64 || w >= 256 || h >= 256))
            }).unwrap_or(data.mips.len().saturating_sub(1));
            (0, maximum as i32)
        });
        let first = crate::video::first_texture_mip(data.width, data.height, detail, minimum, maximum)
            .min(data.mips.len().saturating_sub(1));
        if first == 0 { return data; }
        Arc::new(TextureData { width: (data.width >> first).max(1), height: (data.height >> first).max(1), format: data.format, mips: data.mips[first..].to_vec() })
    }

    pub fn release_material(&mut self, slot: u32) {
        if self.wet.get_mut().targets.remove(&slot).is_some() {
            let prefix=format!("wet#{slot}#");
            self.views.retain(|key,_| !key.starts_with(&prefix));
            let runtime = self.wet.get_mut();
            let used: std::collections::HashSet<_> = runtime.targets.values().flatten().map(|target| target.path.as_str()).collect();
            runtime.states.retain(|path, _| used.contains(path.as_str()));
        }
    }

    pub fn evaluate_material(&self, queue: &wgpu::Queue, slot: u32, time: f64) {
        let mut runtime = self.wet.borrow_mut();
        let WetRuntime { states, targets, random } = &mut *runtime;
        let Some(targets) = targets.get(&slot) else { return; };
        for target in targets {
            let Some(state) = states.get_mut(&target.path) else { continue; };
            state.definition.evaluate_with_animation(&mut state.simulation,&mut state.prime_current,time,random,self.no_fractal_anim);
            let bytes = state.definition.rgba(&state.simulation.pixels);
            queue.write_texture(
                wgpu::TexelCopyTextureInfo { texture: &target.texture, mip_level: 0, origin: wgpu::Origin3d { x:0, y:0, z:target.layer }, aspect: wgpu::TextureAspect::All },
                &bytes,
                wgpu::TexelCopyBufferLayout { offset:0, bytes_per_row:Some(state.simulation.width as u32*4), rows_per_image:Some(state.simulation.height as u32) },
                wgpu::Extent3d { width:state.simulation.width as u32, height:state.simulation.height as u32, depth_or_array_layers:1 },
            );
        }
    }

    pub fn material_array_view(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, source: &mut dyn TextureSource, paths: &[String], slot: u32) -> SharedView {
        let definitions: Vec<_> = paths.iter().map(|path| source.wet_texture(path)).collect();
        if definitions.iter().all(Option::is_none) { return self.array_view(device,queue,source,paths); }
        let key = format!("wet#{slot}#{}", paths.join("|").to_lowercase());
        if let Some(entry)=self.cached(&key) { return entry.0.clone(); }
        let layers: Vec<_> = paths.iter().zip(&definitions).map(|(path,definition)| {
            if let Some(wet)=definition {
                Arc::new(TextureData { width:wet.simulation.width as u32,height:wet.simulation.height as u32,format:TextureFormatKind::Rgba8,mips:vec![wet.rgba(&wet.source)] })
            } else { self.texture_data(source, path) }
        }).collect();
        let entry=upload_layers(device,queue,&layers,wgpu::TextureViewDimension::D2Array,self.bc_supported);
        let runtime=self.wet.get_mut();
        for (layer,(path,definition)) in paths.iter().zip(definitions).enumerate() {
            if let Some(definition)=definition {
                let path=path.to_lowercase();
                runtime.states.entry(path.clone()).or_insert_with(||WetState { simulation:definition.simulation.clone(),definition,prime_current:0 });
                runtime.targets.entry(slot).or_default().push(WetTarget { path,texture:entry.0.texture().clone(),layer:layer as u32 });
            }
        }
        self.remember(key, &entry);
        entry.0
    }

    pub fn view(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, source: &mut dyn TextureSource, path: &str) -> SharedView {
        self.sized_view(device, queue, source, path).0
    }

    pub fn sized_view(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, source: &mut dyn TextureSource, path: &str) -> (SharedView, [f32; 2]) {
        if path.is_empty() {
            return (self.white.clone(), [1.0, 1.0]);
        }
        self.layered_view(device, queue, source, std::slice::from_ref(&path.to_owned()), wgpu::TextureViewDimension::D2)
    }

    pub fn alpha_map_view(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, source: &mut dyn TextureSource, path: &str) -> SharedView {
        let key = format!("D2#alpha#{}", path.to_lowercase());
        if let Some(entry) = self.cached(&key) {
            return entry.0.clone();
        }
        let data = if path.is_empty() { white_data() } else { source.alpha_map(path) };
        let entry = upload_layers(device, queue, std::slice::from_ref(&data), wgpu::TextureViewDimension::D2, self.bc_supported);
        self.remember(key, &entry);
        entry.0
    }

    pub fn array_view(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, source: &mut dyn TextureSource, paths: &[String]) -> SharedView {
        if paths.is_empty() || paths[0].is_empty() {
            return self.white_array.clone();
        }
        self.layered_view(device, queue, source, paths, wgpu::TextureViewDimension::D2Array).0
    }

    pub fn cube_view(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, source: &mut dyn TextureSource, faces: &[String; 6]) -> SharedView {
        self.layered_view(device, queue, source, faces, wgpu::TextureViewDimension::Cube).0
    }

    pub fn projector_facing(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) -> SharedView {
        let key = "ProjectorFacingCube32";
        if let Some(entry) = self.cached(key) { return entry.0.clone(); }
        let faces: Vec<_> = killer_assets::scene::projectors::facing_cube(32).into_iter().map(|data| Arc::new(TextureData {
            width: 32, height: 32, format: TextureFormatKind::Rgba8, mips: vec![data],
        })).collect();
        let entry = upload_layers(device, queue, &faces, wgpu::TextureViewDimension::Cube, self.bc_supported);
        self.remember(key.to_owned(), &entry);
        entry.0
    }

    pub fn data_array_view(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, key: &str, layers: &[Arc<TextureData>]) -> SharedView {
        let key = format!("D2Array#data#{key}");
        if let Some(entry) = self.cached(&key) {
            return entry.0.clone();
        }
        let entry = upload_layers(device, queue, layers, wgpu::TextureViewDimension::D2Array, self.bc_supported);
        self.remember(key, &entry);
        entry.0
    }

    fn layered_view(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, source: &mut dyn TextureSource, paths: &[String], dimension: wgpu::TextureViewDimension) -> (SharedView, [f32; 2]) {
        let key = format!("{:?}#{}", dimension, paths.iter().map(|path| path.to_lowercase()).collect::<Vec<_>>().join("|"));
        if let Some(entry) = self.cached(&key) {
            return entry.clone();
        }
        let original_size = paths.first().filter(|path| !path.is_empty()).map(|path| source.texture(path)).map(|data| [data.width as f32, data.height as f32]);
        let layers: Vec<Arc<TextureData>> = paths.iter().map(|path| if path.is_empty() { white_data() } else { self.texture_data(source, path) }).collect();
        let mut entry = upload_layers(device, queue, &layers, dimension, self.bc_supported);
        if let Some(size) = original_size { entry.1 = size; }
        self.remember(key, &entry);
        entry
    }
}

#[derive(Clone, Copy)]
enum Compressed {
    Bc1,
    Bc2,
    Bc3,
}

fn compressed_kind(format: TextureFormatKind) -> Option<Compressed> {
    match format {
        TextureFormatKind::Bc1 => Some(Compressed::Bc1),
        TextureFormatKind::Bc2 => Some(Compressed::Bc2),
        TextureFormatKind::Bc3 => Some(Compressed::Bc3),
        TextureFormatKind::Rgba8 => None,
    }
}

fn bc_wgpu_format(kind: Compressed) -> wgpu::TextureFormat {
    match kind {
        Compressed::Bc1 => wgpu::TextureFormat::Bc1RgbaUnorm,
        Compressed::Bc2 => wgpu::TextureFormat::Bc2RgbaUnorm,
        Compressed::Bc3 => wgpu::TextureFormat::Bc3RgbaUnorm,
    }
}

fn dxt_format(kind: Compressed) -> TextureFormat {
    match kind {
        Compressed::Bc1 => TextureFormat::Dxt1,
        Compressed::Bc2 => TextureFormat::Dxt3,
        Compressed::Bc3 => TextureFormat::Dxt5,
    }
}

fn decode_mips_to_rgba8(width: u32, height: u32, format: TextureFormat, mips: &[Vec<u8>]) -> Vec<Vec<u8>> {
    mips.iter()
        .enumerate()
        .map(|(level, bytes)| {
            let mip_width = (width >> level).max(1);
            let mip_height = (height >> level).max(1);
            let mip = Mip { width: mip_width, height: mip_height, data: bytes };
            decode_rgba(format, &mip, None).unwrap_or_else(|_| vec![0u8; (mip_width * mip_height * 4) as usize])
        })
        .collect()
}

fn usable(data: &TextureData) -> bool {
    data.width > 0 && data.height > 0 && !data.mips.is_empty()
}

fn same_layout(a: &TextureData, b: &TextureData) -> bool {
    a.width == b.width && a.height == b.height && a.format == b.format && a.mips.len() == b.mips.len()
}

fn upload_layers(device: &wgpu::Device, queue: &wgpu::Queue, layers: &[Arc<TextureData>], dimension: wgpu::TextureViewDimension, bc_supported: bool) -> (SharedView, [f32; 2]) {
    let fallback = white_data();
    let first = layers.first().filter(|data| usable(data)).cloned().unwrap_or(fallback.clone());
    let wanted = if dimension == wgpu::TextureViewDimension::Cube { 6 } else { layers.len().max(1) };
    let layers: Vec<Arc<TextureData>> = (0..wanted).map(|index| layers.get(index).filter(|data| same_layout(data, &first)).cloned().unwrap_or_else(|| first.clone())).collect();
    let mut format = wgpu::TextureFormat::Rgba8Unorm;
    let mut bytes = Vec::with_capacity(layers.iter().map(|data| data.mips.iter().map(Vec::len).sum::<usize>()).sum());
    let mut mip_count = 0;
    for data in &layers {
        let (layer_format, mips): (_, std::borrow::Cow<'_, [Vec<u8>]>) = match compressed_kind(data.format) {
            None => (wgpu::TextureFormat::Rgba8Unorm, std::borrow::Cow::Borrowed(data.mips.as_slice())),
            Some(bc) if bc_supported => (bc_wgpu_format(bc), std::borrow::Cow::Borrowed(data.mips.as_slice())),
            Some(bc) => (wgpu::TextureFormat::Rgba8Unorm, std::borrow::Cow::Owned(decode_mips_to_rgba8(data.width, data.height, dxt_format(bc), &data.mips))),
        };
        format = layer_format;
        mip_count = mips.len() as u32;
        for mip in mips.iter() {
            bytes.extend_from_slice(mip);
        }
    }
    let texture = device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: first.width, height: first.height, depth_or_array_layers: layers.len() as u32 },
            mip_level_count: mip_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &bytes,
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor { dimension: Some(dimension), ..Default::default() });
    (Arc::new(view), [first.width as f32, first.height as f32])
}
