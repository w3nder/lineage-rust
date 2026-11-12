use l2_catalog::{Catalog, RawTexture};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufWriter};
use std::path::Path;
use std::sync::Arc;
use ue2_assets::{Image, Mip, TextureFormat, decode_rgba};

const MIN_BC_MIP_EXTENT: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormatKind {
    Rgba8,
    Bc1,
    Bc2,
    Bc3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormatKind,
    pub mips: Vec<Vec<u8>>,
}

pub trait TextureSource {
    fn texture_lod(&mut self, _path: &str) -> Option<(i32, i32)> { None }

    fn texture(&mut self, path: &str) -> Arc<TextureData>;

    fn wet_texture(&mut self, _path: &str) -> Option<Arc<l2_catalog::WetTexture>> { None }

    fn alpha_map(&mut self, path: &str) -> Arc<TextureData> {
        self.texture(path)
    }
}

pub struct TextureBank {
    textures: HashMap<String, Arc<TextureData>>,
    alpha_maps: HashMap<String, Arc<TextureData>>,
    checker_data: Arc<TextureData>,
    warned: HashSet<String>,
    wet: Box<WetDefinitions>,
}

#[derive(Default)]
struct WetDefinitions {
    by_path: HashMap<String, Option<Arc<l2_catalog::WetTexture>>>,
    pinned: HashSet<String>,
}

impl Default for TextureBank {
    fn default() -> Self {
        Self::new()
    }
}

impl TextureBank {
    pub fn new() -> Self {
        Self { textures: HashMap::new(), alpha_maps: HashMap::new(), checker_data: Arc::new(checker_texture_data()), warned: HashSet::new(), wet: Box::default() }
    }

    pub fn alpha_map(&mut self, catalog: &mut Catalog, path: &str) -> Arc<TextureData> {
        let key = path.to_lowercase();
        if let Some(data) = self.alpha_maps.get(&key) {
            return data.clone();
        }
        let data = match catalog.texture_raw(path) {
            Some(raw) => Arc::new(build_texture_data(&alpha_map_raw(raw))),
            None => self.texture(catalog, path),
        };
        self.alpha_maps.insert(key, data.clone());
        data
    }

    pub fn texture(&mut self, catalog: &mut Catalog, path: &str) -> Arc<TextureData> {
        let key = path.to_lowercase();
        if let Some(data) = self.textures.get(&key) {
            return data.clone();
        }
        if let Some((width, height, rgba)) = killer_font::rgba_for_texture(path) {
            let data = Arc::new(TextureData { width, height, format: TextureFormatKind::Rgba8, mips: vec![rgba] });
            self.textures.insert(key, data.clone());
            return data;
        }
        let raw = catalog.try_texture_raw(path);
        if let Err(why) = &raw
            && self.warned.insert(key.clone())
        {
            log::warn!("textura {path}: {why}; xadrez no lugar");
        }
        let data = resolve_texture_data(raw.ok(), &self.checker_data);
        if Arc::ptr_eq(&data, &self.checker_data) && self.warned.insert(key.clone()) {
            log::warn!("textura {path}: sem mips utilizáveis; xadrez no lugar");
        }
        self.textures.insert(key, data.clone());
        data
    }

    pub fn insert(&mut self, name: &str, data: TextureData) {
        self.insert_shared(name, Arc::new(data));
    }

    pub fn insert_shared(&mut self, name: &str, data: Arc<TextureData>) {
        let key = name.to_lowercase();
        self.wet.pinned.insert(key.clone());
        self.textures.insert(key, data);
    }

    pub fn insert_shared_alpha_map(&mut self, name: &str, data: Arc<TextureData>) {
        let key = name.to_lowercase();
        self.wet.pinned.insert(format!("alpha:{key}"));
        self.alpha_maps.insert(key, data);
    }

    pub fn trim_over(&mut self, limit: usize) -> usize {
        if self.cached_bytes() > limit { self.trim() } else { 0 }
    }

    pub fn trim(&mut self) -> usize {
        let before = self.textures.len() + self.alpha_maps.len();
        let pinned = &self.wet.pinned;
        self.textures.retain(|key, data| Arc::strong_count(data) > 1 || pinned.contains(key));
        self.alpha_maps.retain(|key, data| Arc::strong_count(data) > 1 || pinned.contains(&format!("alpha:{key}")));
        before - self.textures.len() - self.alpha_maps.len()
    }

    pub fn cached_bytes(&self) -> usize {
        self.textures.values().chain(self.alpha_maps.values()).map(|data| data.mips.iter().map(Vec::len).sum::<usize>()).sum()
    }

    pub fn textures(&self) -> impl Iterator<Item = (String, Arc<TextureData>)> + '_ {
        let mut names: Vec<&String> = self.textures.keys().collect();
        names.sort();
        names.into_iter().map(|name| (name.clone(), self.textures[name].clone()))
    }

    pub fn alpha_maps(&self) -> impl Iterator<Item = (String, Arc<TextureData>)> + '_ {
        let mut names: Vec<&String> = self.alpha_maps.keys().collect();
        names.sort();
        names.into_iter().map(|name| (name.clone(), self.alpha_maps[name].clone()))
    }

    pub fn has_texture(&self, path: &str) -> bool {
        self.textures.contains_key(&path.to_lowercase())
    }

    pub fn texture_available(&self, path: &str) -> Option<bool> {
        self.textures.get(&path.to_lowercase()).map(|data| !Arc::ptr_eq(data, &self.checker_data))
    }

    pub fn insert_dds(&mut self, name: &str, bytes: &[u8]) -> Result<(u32, u32), String> {
        let data = dds_texture_data(bytes)?;
        let size = (data.width, data.height);
        self.insert(name, data);
        Ok(size)
    }

    pub fn missing(&self) -> impl Iterator<Item = &str> {
        self.warned.iter().map(String::as_str)
    }

    pub fn size(&mut self, catalog: &mut Catalog, path: &str) -> [f32; 2] {
        catalog.texture_size(path).unwrap_or([64.0, 64.0])
    }
}

pub struct BankSource<'a> {
    pub bank: &'a mut TextureBank,
    pub catalog: &'a mut Catalog,
}

impl TextureSource for BankSource<'_> {
    fn texture_lod(&mut self, path: &str) -> Option<(i32, i32)> {
        if self.bank.textures.get(&path.to_lowercase()).is_some_and(|data| data.mips.len() == 1) { return Some((0, 0)); }
        self.catalog.texture_lod(path)
    }
    fn wet_texture(&mut self, path: &str) -> Option<Arc<l2_catalog::WetTexture>> {
        self.bank.wet.by_path.entry(path.to_lowercase()).or_insert_with(|| self.catalog.wet_texture(path).map(Arc::new)).clone()
    }
    fn texture(&mut self, path: &str) -> Arc<TextureData> {
        self.bank.texture(self.catalog, path)
    }

    fn alpha_map(&mut self, path: &str) -> Arc<TextureData> {
        self.bank.alpha_map(self.catalog, path)
    }
}

pub fn alpha_map_raw(mut raw: RawTexture) -> RawTexture {
    if raw.format == TextureFormat::P8
        && let Some(palette) = raw.palette.as_mut()
    {
        for color in palette.iter_mut() {
            color[3] = color[0];
        }
    }
    raw
}

pub fn alpha_map_image(raw: RawTexture) -> Option<Image> {
    let raw = alpha_map_raw(raw);
    let mip = raw.mips.first()?;
    let source = Mip { width: mip.width, height: mip.height, data: &mip.data };
    let rgba = decode_rgba(raw.format, &source, raw.palette.as_deref()).ok()?;
    Some(Image { width: mip.width, height: mip.height, rgba })
}

fn resolve_texture_data(raw: Option<RawTexture>, checker: &Arc<TextureData>) -> Arc<TextureData> {
    match raw.map(|raw| build_texture_data(&raw)) {
        Some(data) if !data.mips.is_empty() => Arc::new(data),
        _ => checker.clone(),
    }
}

fn build_texture_data(raw: &RawTexture) -> TextureData {
    let (width, height) = raw.mips.first().map(|mip| (mip.width, mip.height)).unwrap_or((0, 0));
    let kind = match raw.format {
        TextureFormat::Dxt1 => Some(TextureFormatKind::Bc1),
        TextureFormat::Dxt3 => Some(TextureFormatKind::Bc2),
        TextureFormat::Dxt5 => Some(TextureFormatKind::Bc3),
        _ => None,
    };
    match kind {
        Some(format) => {
            let mips = raw
                .mips
                .iter()
                .filter(|mip| mip.width >= MIN_BC_MIP_EXTENT && mip.height >= MIN_BC_MIP_EXTENT)
                .map(|mip| mip.data.clone())
                .collect();
            TextureData { width, height, format, mips }
        }
        None => {
            let mips = raw
                .mips
                .iter()
                .filter_map(|mip| {
                    let source = Mip { width: mip.width, height: mip.height, data: &mip.data };
                    decode_rgba(raw.format, &source, raw.palette.as_deref()).ok()
                })
                .collect();
            TextureData { width, height, format: TextureFormatKind::Rgba8, mips }
        }
    }
}

pub fn write_png(path: &Path, width: u32, height: u32, rgba: &[u8]) -> io::Result<()> {
    let file = BufWriter::new(File::create(path)?);
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(io::Error::other)?;
    writer.write_image_data(rgba).map_err(io::Error::other)
}

pub fn checker_image() -> Image {
    let mut rgba = Vec::with_capacity(8 * 8 * 4);
    for y in 0..8 {
        for x in 0..8 {
            let magenta = (x + y) % 2 == 0;
            rgba.extend_from_slice(if magenta { &[255, 0, 255, 255] } else { &[0, 0, 0, 255] });
        }
    }
    Image { width: 8, height: 8, rgba }
}

pub const DDS_MAGIC: &[u8; 4] = b"DDS ";
pub const DDS_HEADER_SIZE: usize = 128;
const DDPF_FOURCC: u32 = 0x4;
const DDPF_RGB: u32 = 0x40;
const DDPF_ALPHAPIXELS: u32 = 0x1;

pub fn dds_texture_data(bytes: &[u8]) -> Result<TextureData, String> {
    if bytes.len() < DDS_HEADER_SIZE || &bytes[..4] != DDS_MAGIC {
        return Err(format!("DDS: cabeçalho inválido ({} bytes)", bytes.len()));
    }
    let u32_at = |at: usize| u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
    let height = u32_at(12);
    let width = u32_at(16);
    let pf_flags = u32_at(80);
    let fourcc = &bytes[84..88];
    let bit_count = u32_at(88);
    let body = &bytes[DDS_HEADER_SIZE..];
    let (format, size) = if pf_flags & DDPF_FOURCC != 0 {
        let blocks = (width.max(1).div_ceil(4) * height.max(1).div_ceil(4)) as usize;
        match fourcc {
            b"DXT1" => (TextureFormatKind::Bc1, blocks * 8),
            b"DXT3" => (TextureFormatKind::Bc2, blocks * 16),
            b"DXT5" => (TextureFormatKind::Bc3, blocks * 16),
            other => return Err(format!("DDS: fourcc {:?} não suportado", String::from_utf8_lossy(other))),
        }
    } else if pf_flags & DDPF_RGB != 0 && bit_count == 32 && pf_flags & DDPF_ALPHAPIXELS != 0 {
        (TextureFormatKind::Rgba8, (width * height * 4) as usize)
    } else {
        return Err(format!("DDS: formato de pixel {pf_flags:#x}/{bit_count} bits não suportado"));
    };
    if body.len() < size {
        return Err(format!("DDS: {} bytes de imagem, esperados {size}", body.len()));
    }
    let mut mip = body[..size].to_vec();
    if format == TextureFormatKind::Rgba8 {
        let (r_mask, b_mask) = (u32_at(92), u32_at(100));
        if r_mask == 0x00ff_0000 && b_mask == 0x0000_00ff {
            for px in mip.as_chunks_mut::<4>().0 {
                px.swap(0, 2);
            }
        }
    }
    Ok(TextureData { width, height, format, mips: vec![mip] })
}

fn checker_texture_data() -> TextureData {
    let image = checker_image();
    TextureData { width: image.width, height: image.height, format: TextureFormatKind::Rgba8, mips: vec![image.rgba] }
}

#[cfg(test)]
mod tests {
    #[test]
    fn trim_drops_only_unreferenced_unpinned_copies() {
        let data = |byte: u8| TextureData { width: 1, height: 1, format: TextureFormatKind::Rgba8, mips: vec![vec![byte; 4]] };
        let mut bank = TextureBank::new();
        bank.insert("pinned", data(1));
        let held = Arc::new(data(2));
        bank.textures.insert("held".into(), held.clone());
        bank.textures.insert("free".into(), Arc::new(data(3)));
        bank.alpha_maps.insert("free".into(), Arc::new(data(4)));
        assert_eq!(bank.cached_bytes(), 16);
        assert_eq!(bank.trim_over(1024), 0);
        assert_eq!(bank.trim_over(0), 2);
        assert!(bank.has_texture("pinned") && bank.has_texture("held") && !bank.has_texture("free"));
        drop(held);
        assert_eq!(bank.trim(), 1);
    }

    use super::*;
    use l2_catalog::RawMip;

    fn dds(fourcc: &[u8; 4], w: u32, h: u32, body_len: usize) -> Vec<u8> {
        let mut out = vec![0u8; DDS_HEADER_SIZE];
        out[..4].copy_from_slice(DDS_MAGIC);
        out[4..8].copy_from_slice(&124u32.to_le_bytes());
        out[12..16].copy_from_slice(&h.to_le_bytes());
        out[16..20].copy_from_slice(&w.to_le_bytes());
        out[76..80].copy_from_slice(&32u32.to_le_bytes());
        out[80..84].copy_from_slice(&DDPF_FOURCC.to_le_bytes());
        out[84..88].copy_from_slice(fourcc);
        out.extend(std::iter::repeat_n(0xabu8, body_len));
        out
    }

    #[test]
    fn a_dxt1_crest_dds_becomes_a_bc1_texture_of_16_by_16_registered_under_its_name() {
        let data = dds_texture_data(&dds(b"DXT1", 16, 16, 128)).unwrap();
        assert_eq!((data.width, data.height, data.format, data.mips[0].len()), (16, 16, TextureFormatKind::Bc1, 128));
        let ally = dds_texture_data(&dds(b"DXT1", 8, 16, 64)).unwrap();
        assert_eq!((ally.width, ally.height, ally.mips[0].len()), (8, 16, 64));
        assert!(dds_texture_data(&dds(b"DXT1", 16, 16, 100)).is_err());
        assert!(dds_texture_data(b"nope").is_err());
        assert!(dds_texture_data(&dds(b"DXT2", 16, 16, 128)).is_err());
        let mut bank = TextureBank::new();
        assert_eq!(bank.insert_dds("crest.crest_5", &dds(b"DXT1", 16, 16, 128)), Ok((16, 16)));
        assert!(bank.textures.contains_key("crest.crest_5"));
    }

    #[test]
    fn a_p8_alpha_map_takes_its_alpha_from_the_palettes_red_like_calc_layer_tex_coords() {
        let palette: Vec<[u8; 4]> = vec![[7, 1, 2, 255], [200, 3, 4, 0]];
        let raw = RawTexture { format: TextureFormat::P8, palette: Some(palette), mips: vec![RawMip { width: 2, height: 1, data: vec![0, 1] }] };
        let image = alpha_map_image(raw.clone()).unwrap();
        assert_eq!(image.rgba, [7, 1, 2, 7, 200, 3, 4, 200]);
        let data = build_texture_data(&alpha_map_raw(raw));
        assert_eq!(data.mips[0][3], 7);
        let rgba = RawTexture { format: TextureFormat::Rgba8, palette: None, mips: vec![RawMip { width: 1, height: 1, data: vec![1, 2, 3, 9] }] };
        assert_eq!(alpha_map_image(rgba).unwrap().rgba, [3, 2, 1, 9]);
    }

    #[test]
    fn missing_texture_data_falls_back_to_checker_once() {
        let dir = std::env::temp_dir().join("killer-assets-checker-data");
        std::fs::create_dir_all(&dir).unwrap();
        let mut catalog = Catalog::open(&dir);
        let mut bank = TextureBank::new();
        assert_eq!(bank.texture_available("Nope.Missing"), None);
        let a = bank.texture(&mut catalog, "Nope.Missing");
        let b = bank.texture(&mut catalog, "Nope.Missing");
        assert_eq!((a.width, a.height), (8, 8));
        assert!(Arc::ptr_eq(&a, &b));
        assert_eq!(a.format, TextureFormatKind::Rgba8);
        assert_eq!(a.mips.len(), 1);
        assert_eq!(&a.mips[0][..4], &[255, 0, 255, 255]);
        assert_eq!(&a.mips[0][4..8], &[0, 0, 0, 255]);
        assert_eq!(bank.size(&mut catalog, "Nope.Missing"), [64.0, 64.0]);
        assert_eq!(bank.missing().collect::<Vec<_>>(), ["nope.missing"]);
        assert_eq!(bank.texture_available("NOPE.MISSING"), Some(false));
        bank.insert("Nope.Missing", checker_texture_data());
        assert_eq!(bank.texture_available("nope.missing"), Some(true));
        assert_eq!(bank.missing().collect::<Vec<_>>(), ["nope.missing"]);
    }

    #[test]
    fn write_png_round_trips_the_checker() {
        let path = std::env::temp_dir().join("killer-assets-checker.png");
        let image = checker_image();
        write_png(&path, image.width, image.height, &image.rgba).unwrap();
        let decoder = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(&path).unwrap()));
        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut buf).unwrap();
        assert_eq!((info.width, info.height), (8, 8));
        assert_eq!(&buf[..info.buffer_size()], &image.rgba[..]);
    }

    fn dxt1_mip(width: u32, height: u32) -> RawMip {
        let blocks = (width.div_ceil(4) * height.div_ceil(4)) as usize;
        RawMip { width, height, data: vec![0xAB; blocks * 8] }
    }

    #[test]
    fn build_texture_data_keeps_dxt_bytes_and_drops_tail_mips_below_4x4() {
        let raw = RawTexture {
            format: TextureFormat::Dxt1,
            palette: None,
            mips: vec![dxt1_mip(8, 8), dxt1_mip(4, 4), dxt1_mip(2, 2), dxt1_mip(1, 1)],
        };
        let data = build_texture_data(&raw);
        assert_eq!((data.width, data.height), (8, 8));
        assert_eq!(data.format, TextureFormatKind::Bc1);
        assert_eq!(data.mips.len(), 2);
        assert_eq!(data.mips[0].len(), 32);
        assert_eq!(data.mips[1].len(), 8);
    }

    #[test]
    fn build_texture_data_decodes_rgba8_mips() {
        let raw = RawTexture {
            format: TextureFormat::Rgba8,
            palette: None,
            mips: vec![RawMip { width: 1, height: 1, data: vec![1, 2, 3, 4] }],
        };
        let data = build_texture_data(&raw);
        assert_eq!(data.format, TextureFormatKind::Rgba8);
        assert_eq!(data.mips, vec![vec![3, 2, 1, 4]]);
    }

    #[test]
    fn build_texture_data_keeps_bc2_and_bc3_block_sizes() {
        let dxt3 = RawTexture {
            format: TextureFormat::Dxt3,
            palette: None,
            mips: vec![RawMip { width: 4, height: 4, data: vec![0x11; 16] }],
        };
        let data = build_texture_data(&dxt3);
        assert_eq!(data.format, TextureFormatKind::Bc2);
        assert_eq!(data.mips, vec![vec![0x11; 16]]);

        let dxt5 = RawTexture {
            format: TextureFormat::Dxt5,
            palette: None,
            mips: vec![RawMip { width: 8, height: 8, data: vec![0x22; 64] }],
        };
        let data = build_texture_data(&dxt5);
        assert_eq!(data.format, TextureFormatKind::Bc3);
        assert_eq!(data.mips, vec![vec![0x22; 64]]);
    }

    #[test]
    fn build_texture_data_p8_without_palette_has_no_mips() {
        let raw = RawTexture {
            format: TextureFormat::P8,
            palette: None,
            mips: vec![RawMip { width: 2, height: 2, data: vec![0, 1, 2, 3] }],
        };
        let data = build_texture_data(&raw);
        assert!(data.mips.is_empty(), "{:?}", data.mips);
    }

    #[test]
    fn texture_with_no_decodable_mips_falls_back_to_checker() {
        let raw = RawTexture {
            format: TextureFormat::P8,
            palette: None,
            mips: vec![RawMip { width: 2, height: 2, data: vec![0, 1, 2, 3] }],
        };
        let checker = Arc::new(checker_texture_data());
        let data = resolve_texture_data(Some(raw), &checker);
        assert_eq!(data.mips.len(), 1);
        assert_eq!(&data.mips[0][..4], &[255, 0, 255, 255]);
    }
}

#[cfg(test)]
mod radar_names {
    use l2_catalog::Catalog;
    use std::path::PathBuf;

    /// O desenho da barra de ação no engine (`0x10529d80`–`0x1052a0e0`) tira a medida de cada peça
    /// da própria textura, chamando os slots `0x74` (`USize`) e `0x78` (`VSize`). Este teste fixa o
    /// que essas texturas medem no cliente, que é de onde saem o `GAUGE_BACK_TEXELS` e o
    /// `GAUGE_FILL_TEXELS` do `crates/killer-ui/src/gauge.rs`.
    #[test]
    #[ignore]
    fn the_minibar_pieces_measure_what_the_gauge_constants_say() {
        let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
        let mut catalog = Catalog::open(&root);
        let (bu, bv) = killer_ui::gauge::GAUGE_BACK_TEXELS;
        for name in killer_ui::gauge::GAUGE_BACK {
            let img = catalog.texture(name).unwrap_or_else(|| panic!("{name} não existe"));
            assert_eq!((img.width as i32, img.height as i32), (bu, bv), "{name}");
        }
        let (fu, fv) = killer_ui::gauge::GAUGE_FILL_TEXELS;
        for name in killer_ui::gauge::GAUGE_FILL {
            let img = catalog.texture(name).unwrap_or_else(|| panic!("{name} não existe"));
            assert_eq!((img.width as i32, img.height as i32), (fu, fv), "{name}");
        }
    }

    #[test]
    #[ignore]
    fn the_c4_radar_pieces_are_lineage_decos_meshes_painted_with_l2ui_textures() {
        let Some(root) = std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from) else { return };
        let mut catalog = Catalog::open(&root);
        for name in [
            "LineageDecos.radar_back",
            "LineageDecos.radar_arrow",
            "LineageDecos.radar_dot_party",
            "LineageDecos.radar_dot_target",
            "LineageDecos.radar_dot_tutorial1",
            "LineageDecos.radar_dot_tutorial2",
            "LineageDecos.radar_center",
            "LineageDecos.radar_n",
            "LineageDecos.radar_time",
        ] {
            assert!(catalog.texture_raw(name).is_none(), "{name} não é textura");
            assert_eq!(catalog.object_class(name).as_deref(), Some("SkeletalMesh"), "{name}");
        }
        for name in ["radar_back", "radar_arrow", "radar_myself", "radar_target", "radar_party", "radar_n", "radar_tutorial1", "radar_tutorial2"] {
            assert!(catalog.texture_raw(&format!("L2UI_CH3.Radar.{name}")).is_some(), "{name} deveria existir no cliente");
        }
    }
}
