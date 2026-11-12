use crate::scene::lights::PointLight;
use crate::scene::sky::{BodyKind, Flare, SkyBodyDraw, SkyTint};
use crate::scene::{ActorCull, DrawKind, FogParams, GlobalFog, LightmapGroup, LodSelect, MeshDraw, Pass, Scene, SectorBounds, Vertex, ZoneLighting};
use crate::textures::{TextureBank, TextureData, TextureFormatKind};
use bytemuck::Zeroable;
use glam::Vec3;
use l2_catalog::{
    AlphaOp, BlendFactor, ColorOp, CoordSource, Cubemap, Fade, FogMode, Material, Mat3x2, Oscillator, Rotator, Stage, StageSource, TextureRef, Trigger, UvModifier,
};
use std::fs;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub const MAGIC: &[u8; 8] = b"KLRSCN\0\0";
pub const FORMAT_VERSION: u32 = 18;
mod movers;
const APP_DIR: &str = "killer";
const CACHE_DIR: &str = "cache";
const EXTENSION: &str = "scene";
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

pub fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(FNV_OFFSET, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME))
}

pub fn app_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let base = if cfg!(target_os = "macos") {
        home?.join("Library/Application Support")
    } else if cfg!(target_os = "windows") {
        PathBuf::from(std::env::var_os("APPDATA")?)
    } else {
        match std::env::var_os("XDG_CONFIG_HOME") {
            Some(dir) => PathBuf::from(dir),
            None => home?.join(".config"),
        }
    };
    Some(base.join(APP_DIR))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStamp {
    pub size: u64,
    pub hash: u64,
}

impl SourceStamp {
    pub fn include(&mut self, bytes: &[u8]) {
        for byte in (bytes.len() as u64).to_le_bytes().iter().chain(bytes) {
            self.hash = (self.hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME);
        }
    }

    pub fn of(bytes: &[u8]) -> Self {
        Self { size: bytes.len() as u64, hash: fnv1a(bytes) }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CachedScene {
    pub scene: Scene,
    pub textures: Vec<(String, Arc<TextureData>)>,
    pub alpha_maps: Vec<(String, Arc<TextureData>)>,
}

impl CachedScene {
    pub fn from_bank(scene: Scene, bank: &TextureBank) -> Self {
        Self { scene, textures: bank.textures().collect(), alpha_maps: bank.alpha_maps().collect() }
    }

    pub fn into_bank(self) -> (Scene, TextureBank) {
        let mut bank = TextureBank::new();
        for (name, data) in self.textures {
            bank.insert_shared(&name, data);
        }
        for (name, data) in self.alpha_maps {
            bank.insert_shared_alpha_map(&name, data);
        }
        (self.scene, bank)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneCache {
    dir: PathBuf,
}

impl SceneCache {
    pub fn at(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn default_dir() -> Option<Self> {
        app_dir().map(|dir| Self::at(dir.join(CACHE_DIR)))
    }

    pub fn file_for(&self, map: &Path) -> PathBuf {
        let stem = map.file_stem().and_then(|s| s.to_str()).unwrap_or("mapa").to_lowercase();
        let key = fnv1a(map.to_string_lossy().to_lowercase().as_bytes());
        self.dir.join(format!("{stem}-{key:016x}.{EXTENSION}"))
    }

    pub fn read(&self, map: &Path, stamp: &SourceStamp) -> Option<CachedScene> {
        let file = fs::File::open(self.file_for(map)).ok()?;
        decode(BufReader::with_capacity(1 << 20, file), stamp).ok()
    }

    pub fn write(&self, map: &Path, stamp: &SourceStamp, cached: &CachedScene) -> io::Result<PathBuf> {
        self.write_parts(map, stamp, &cached.scene, &cached.textures, &cached.alpha_maps)
    }

    pub fn write_parts(&self, map: &Path, stamp: &SourceStamp, scene: &Scene, textures: &[(String, Arc<TextureData>)], alpha_maps: &[(String, Arc<TextureData>)]) -> io::Result<PathBuf> {
        fs::create_dir_all(&self.dir)?;
        let path = self.file_for(map);
        let temp = path.with_extension(format!("{EXTENSION}.tmp{}", std::process::id()));
        let result = (|| {
            let mut file = BufWriter::with_capacity(1 << 20, fs::File::create(&temp)?);
            encode(&mut file, stamp, scene, textures, alpha_maps)?;
            file.flush()?;
            fs::rename(&temp, &path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp);
        }
        result.map(|()| path)
    }
}

struct Writer<W: Write> {
    out: W,
    error: Option<io::Error>,
}

impl<W: Write> Writer<W> {
    fn put(&mut self, bytes: &[u8]) {
        if self.error.is_none()
            && let Err(e) = self.out.write_all(bytes)
        {
            self.error = Some(e);
        }
    }

    fn u8(&mut self, v: u8) {
        self.put(&[v]);
    }

    fn bool(&mut self, v: bool) {
        self.u8(u8::from(v));
    }

    fn u16(&mut self, v: u16) {
        self.put(&v.to_le_bytes());
    }

    fn u32(&mut self, v: u32) {
        self.put(&v.to_le_bytes());
    }

    fn u64(&mut self, v: u64) {
        self.put(&v.to_le_bytes());
    }

    fn i32(&mut self, v: i32) {
        self.put(&v.to_le_bytes());
    }

    fn f32(&mut self, v: f32) {
        self.put(&v.to_le_bytes());
    }

    fn len(&mut self, n: usize) {
        self.u32(n as u32);
    }

    fn str(&mut self, s: &str) {
        self.len(s.len());
        self.put(s.as_bytes());
    }

    fn bytes(&mut self, b: &[u8]) {
        self.len(b.len());
        self.put(b);
    }

    fn f32s(&mut self, v: &[f32]) {
        for x in v {
            self.f32(*x);
        }
    }

    fn i32s(&mut self, v: &[i32]) {
        for x in v {
            self.i32(*x);
        }
    }

    fn option<T>(&mut self, v: Option<T>, mut write: impl FnMut(&mut Self, T)) {
        match v {
            Some(v) => {
                self.bool(true);
                write(self, v);
            }
            None => self.bool(false),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheError {
    Truncated,
    BadMagic,
    Version(u32),
    Stale,
    Invalid(&'static str),
}

struct Reader<I: Read> {
    input: I,
}

type R<T> = Result<T, CacheError>;

const MAX_ITEMS: usize = 1 << 28;

impl<I: Read> Reader<I> {
    fn array<const N: usize>(&mut self) -> R<[u8; N]> {
        let mut out = [0u8; N];
        self.input.read_exact(&mut out).map_err(|_| CacheError::Truncated)?;
        Ok(out)
    }

    fn take(&mut self, n: usize) -> R<Vec<u8>> {
        let mut out = vec![0u8; n];
        self.input.read_exact(&mut out).map_err(|_| CacheError::Truncated)?;
        Ok(out)
    }

    fn u8(&mut self) -> R<u8> {
        Ok(self.array::<1>()?[0])
    }

    fn bool(&mut self) -> R<bool> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(CacheError::Invalid("bool")),
        }
    }

    fn u16(&mut self) -> R<u16> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    fn u32(&mut self) -> R<u32> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    fn u64(&mut self) -> R<u64> {
        Ok(u64::from_le_bytes(self.array()?))
    }

    fn i32(&mut self) -> R<i32> {
        Ok(i32::from_le_bytes(self.array()?))
    }

    fn f32(&mut self) -> R<f32> {
        Ok(f32::from_le_bytes(self.array()?))
    }

    fn len(&mut self) -> R<usize> {
        let n = self.u32()? as usize;
        if n > MAX_ITEMS {
            return Err(CacheError::Invalid("tamanho"));
        }
        Ok(n)
    }

    fn str(&mut self) -> R<String> {
        let n = self.len()?;
        String::from_utf8(self.take(n)?).map_err(|_| CacheError::Invalid("utf8"))
    }

    fn bytes(&mut self) -> R<Vec<u8>> {
        let n = self.len()?;
        self.take(n)
    }

    fn f32s<const N: usize>(&mut self) -> R<[f32; N]> {
        let mut out = [0.0; N];
        for x in &mut out {
            *x = self.f32()?;
        }
        Ok(out)
    }

    fn i32s<const N: usize>(&mut self) -> R<[i32; N]> {
        let mut out = [0; N];
        for x in &mut out {
            *x = self.i32()?;
        }
        Ok(out)
    }

    fn option<T>(&mut self, mut read: impl FnMut(&mut Self) -> R<T>) -> R<Option<T>> {
        if self.bool()? { read(self).map(Some) } else { Ok(None) }
    }

    fn vec<T>(&mut self, mut read: impl FnMut(&mut Self) -> R<T>) -> R<Vec<T>> {
        let n = self.len()?;
        let mut out = Vec::with_capacity(n.min(1 << 16));
        for _ in 0..n {
            out.push(read(self)?);
        }
        Ok(out)
    }

    fn at_end(&mut self) -> R<bool> {
        let mut probe = [0u8; 1];
        match self.input.read(&mut probe) {
            Ok(0) => Ok(true),
            Ok(_) => Ok(false),
            Err(_) => Err(CacheError::Truncated),
        }
    }
}

fn write_texture_ref<W: Write>(w: &mut Writer<W>, t: &TextureRef) {
    w.str(&t.path);
    w.len(t.frames.len());
    for frame in &t.frames {
        w.str(frame);
    }
    w.f32(t.rate);
    w.u32(t.total_frames);
    w.f32s(&t.size);
    w.bool(t.clamp[0]);
    w.bool(t.clamp[1]);
}

fn read_texture_ref<I: Read>(r: &mut Reader<I>) -> R<TextureRef> {
    Ok(TextureRef {
        path: r.str()?,
        frames: r.vec(|r| r.str())?,
        rate: r.f32()?,
        total_frames: r.u32()?,
        size: r.f32s()?,
        clamp: [r.bool()?, r.bool()?],
    })
}

fn write_uv<W: Write>(w: &mut Writer<W>, m: &UvModifier) {
    match m {
        UvModifier::Panner { direction, rate } => {
            w.u8(0);
            w.i32s(direction);
            w.f32(*rate);
        }
        UvModifier::Oscillator(o, size) => {
            w.u8(1);
            w.f32s(&o.rate);
            w.f32s(&o.phase);
            w.f32s(&o.amplitude);
            w.u8(o.kind[0]);
            w.u8(o.kind[1]);
            w.f32s(&o.offset);
            w.f32s(size);
        }
        UvModifier::Rotator(rot, size) => {
            w.u8(2);
            w.u8(rot.kind);
            w.i32s(&rot.rotation);
            w.f32s(&rot.offset);
            w.i32s(&rot.oscillation_rate);
            w.i32s(&rot.oscillation_amplitude);
            w.i32s(&rot.oscillation_phase);
            w.f32s(size);
        }
        UvModifier::Scaler { scale, offset, size } => {
            w.u8(3);
            w.f32s(scale);
            w.f32s(offset);
            w.f32s(size);
        }
        UvModifier::PannerTriggered { direction, rate, trigger } => {
            w.u8(4);
            w.i32s(direction);
            w.f32(*rate);
            write_trigger(w, trigger);
        }
        UvModifier::OscillatorTriggered(o, size, trigger) => {
            w.u8(5);
            w.f32s(&o.rate);
            w.f32s(&o.phase);
            w.f32s(&o.amplitude);
            w.u8(o.kind[0]);
            w.u8(o.kind[1]);
            w.f32s(&o.offset);
            w.f32s(size);
            write_trigger(w, trigger);
        }
        UvModifier::Matrix(matrix) => {
            w.u8(6);
            for row in matrix.0 {
                w.f32s(&row);
            }
        }
    }
}

fn write_trigger<W: Write>(w: &mut Writer<W>, trigger: &Trigger) {
    w.f32(trigger.stop_after);
    w.f32(trigger.triggered_time);
    w.bool(trigger.reverse);
}

fn read_trigger<I: Read>(r: &mut Reader<I>) -> R<Trigger> {
    Ok(Trigger { stop_after: r.f32()?, triggered_time: r.f32()?, reverse: r.bool()? })
}

fn read_uv<I: Read>(r: &mut Reader<I>) -> R<UvModifier> {
    Ok(match r.u8()? {
        0 => UvModifier::Panner { direction: r.i32s()?, rate: r.f32()? },
        1 => UvModifier::Oscillator(
            Oscillator { rate: r.f32s()?, phase: r.f32s()?, amplitude: r.f32s()?, kind: [r.u8()?, r.u8()?], offset: r.f32s()? },
            r.f32s()?,
        ),
        2 => UvModifier::Rotator(
            Rotator {
                kind: r.u8()?,
                rotation: r.i32s()?,
                offset: r.f32s()?,
                oscillation_rate: r.i32s()?,
                oscillation_amplitude: r.i32s()?,
                oscillation_phase: r.i32s()?,
            },
            r.f32s()?,
        ),
        3 => UvModifier::Scaler { scale: r.f32s()?, offset: r.f32s()?, size: r.f32s()? },
        4 => UvModifier::PannerTriggered { direction: r.i32s()?, rate: r.f32()?, trigger: read_trigger(r)? },
        5 => UvModifier::OscillatorTriggered(
            Oscillator { rate: r.f32s()?, phase: r.f32s()?, amplitude: r.f32s()?, kind: [r.u8()?, r.u8()?], offset: r.f32s()? },
            r.f32s()?,
            read_trigger(r)?,
        ),
        6 => UvModifier::Matrix(Mat3x2([r.f32s()?, r.f32s()?, r.f32s()?])),
        _ => return Err(CacheError::Invalid("uv")),
    })
}

fn color_op_code(op: ColorOp) -> u8 {
    match op {
        ColorOp::SelectTexture => 0,
        ColorOp::SelectCurrent => 1,
        ColorOp::Modulate => 2,
        ColorOp::Modulate2X => 3,
        ColorOp::Modulate4X => 4,
        ColorOp::Add => 5,
        ColorOp::Subtract => 6,
        ColorOp::BlendTextureAlpha => 7,
        ColorOp::BlendDiffuseAlpha => 8,
        ColorOp::BlendCurrentAlpha => 9,
        ColorOp::BlendFactorAlpha => 10,
        ColorOp::ModulateAlphaAddColor => 11,
        ColorOp::MultiplyAdd => 12,
        ColorOp::BlendCurrentFactorByTextureAlpha => 13,
        ColorOp::BlendTextureFactorByDiffuseAlpha => 14,
        ColorOp::ModulateCurrentByTextureAlpha => 15,
        ColorOp::AddSigned => 16,
        ColorOp::AddSigned2X => 17,
        ColorOp::AddSmooth => 18,
        ColorOp::BlendTextureAlphaPremultiplied => 19,
        ColorOp::ModulateColorAddAlpha => 20,
        ColorOp::ModulateInvAlphaAddColor => 21,
        ColorOp::ModulateInvColorAddAlpha => 22,
        ColorOp::Disable => 23,
        ColorOp::PreModulate => 24,
        ColorOp::MultiplyAddCurrent => 25,
        ColorOp::DotProduct3 => 26,
    }
}

fn color_op(code: u8) -> R<ColorOp> {
    Ok(match code {
        0 => ColorOp::SelectTexture,
        1 => ColorOp::SelectCurrent,
        2 => ColorOp::Modulate,
        3 => ColorOp::Modulate2X,
        4 => ColorOp::Modulate4X,
        5 => ColorOp::Add,
        6 => ColorOp::Subtract,
        7 => ColorOp::BlendTextureAlpha,
        8 => ColorOp::BlendDiffuseAlpha,
        9 => ColorOp::BlendCurrentAlpha,
        10 => ColorOp::BlendFactorAlpha,
        11 => ColorOp::ModulateAlphaAddColor,
        12 => ColorOp::MultiplyAdd,
        13 => ColorOp::BlendCurrentFactorByTextureAlpha,
        14 => ColorOp::BlendTextureFactorByDiffuseAlpha,
        15 => ColorOp::ModulateCurrentByTextureAlpha,
        16 => ColorOp::AddSigned,
        17 => ColorOp::AddSigned2X,
        18 => ColorOp::AddSmooth,
        19 => ColorOp::BlendTextureAlphaPremultiplied,
        20 => ColorOp::ModulateColorAddAlpha,
        21 => ColorOp::ModulateInvAlphaAddColor,
        22 => ColorOp::ModulateInvColorAddAlpha,
        23 => ColorOp::Disable,
        24 => ColorOp::PreModulate,
        25 => ColorOp::MultiplyAddCurrent,
        26 => ColorOp::DotProduct3,
        _ => return Err(CacheError::Invalid("color op")),
    })
}

fn alpha_op_code(op: AlphaOp) -> u8 {
    match op {
        AlphaOp::Texture => 0,
        AlphaOp::Current => 1,
        AlphaOp::Diffuse => 2,
        AlphaOp::TFactor => 3,
        AlphaOp::Modulate => 4,
        AlphaOp::Add => 5,
        AlphaOp::BlendDiffuseAlpha => 6,
    }
}

fn alpha_op(code: u8) -> R<AlphaOp> {
    Ok(match code {
        0 => AlphaOp::Texture,
        1 => AlphaOp::Current,
        2 => AlphaOp::Diffuse,
        3 => AlphaOp::TFactor,
        4 => AlphaOp::Modulate,
        5 => AlphaOp::Add,
        6 => AlphaOp::BlendDiffuseAlpha,
        _ => return Err(CacheError::Invalid("alpha op")),
    })
}

fn blend_code(b: BlendFactor) -> u8 {
    match b {
        BlendFactor::Zero => 0,
        BlendFactor::One => 1,
        BlendFactor::SrcColor => 2,
        BlendFactor::InvSrcColor => 3,
        BlendFactor::SrcAlpha => 4,
        BlendFactor::InvSrcAlpha => 5,
        BlendFactor::DstColor => 6,
    }
}

fn blend(code: u8) -> R<BlendFactor> {
    Ok(match code {
        0 => BlendFactor::Zero,
        1 => BlendFactor::One,
        2 => BlendFactor::SrcColor,
        3 => BlendFactor::InvSrcColor,
        4 => BlendFactor::SrcAlpha,
        5 => BlendFactor::InvSrcAlpha,
        6 => BlendFactor::DstColor,
        _ => return Err(CacheError::Invalid("blend")),
    })
}

fn write_stage<W: Write>(w: &mut Writer<W>, s: &Stage) {
    match &s.source {
        StageSource::Texture(t) => {
            w.u8(0);
            write_texture_ref(w, t);
        }
        StageSource::Cubemap(c) => {
            w.u8(1);
            for face in &c.faces {
                w.str(face);
            }
            w.f32s(&c.size);
        }
        StageSource::Factor => w.u8(2),
        StageSource::Diffuse => w.u8(3),
        StageSource::ProjectorFacing => w.u8(4),
    }
    w.len(s.uv.len());
    for m in &s.uv {
        write_uv(w, m);
    }
    match s.coord_source {
        CoordSource::Stream(n) => {
            w.u8(0);
            w.u8(n);
        }
        CoordSource::CameraEnvMap => w.u8(1),
        CoordSource::WorldEnvMap => w.u8(2),
        CoordSource::Projection { matrix, divide_z } => {
            w.u8(3);
            for row in matrix { w.f32s(&row); }
            w.bool(divide_z);
        }
        CoordSource::ProjectorNormal { matrix } => {
            w.u8(4);
            for row in matrix { w.f32s(&row); }
        }
        CoordSource::ProjectorStream => w.u8(5),
    }
    w.u8(color_op_code(s.color_op));
    w.u8(alpha_op_code(s.alpha_op));
    w.bool(s.swapped);
}

fn read_stage<I: Read>(r: &mut Reader<I>) -> R<Stage> {
    let source = match r.u8()? {
        0 => StageSource::Texture(read_texture_ref(r)?),
        1 => {
            let mut faces: [String; 6] = Default::default();
            for face in &mut faces {
                *face = r.str()?;
            }
            StageSource::Cubemap(Cubemap { faces, size: r.f32s()? })
        }
        2 => StageSource::Factor,
        3 => StageSource::Diffuse,
        4 => StageSource::ProjectorFacing,
        _ => return Err(CacheError::Invalid("stage source")),
    };
    let uv = r.vec(read_uv)?;
    let coord_source = match r.u8()? {
        0 => CoordSource::Stream(r.u8()?),
        1 => CoordSource::CameraEnvMap,
        2 => CoordSource::WorldEnvMap,
        3 => CoordSource::Projection { matrix: [r.f32s()?, r.f32s()?, r.f32s()?, r.f32s()?], divide_z: r.bool()? },
        4 => CoordSource::ProjectorNormal { matrix: [r.f32s()?, r.f32s()?, r.f32s()?, r.f32s()?] },
        5 => CoordSource::ProjectorStream,
        _ => return Err(CacheError::Invalid("coord source")),
    };
    Ok(Stage { source, uv, coord_source, color_op: color_op(r.u8()?)?, alpha_op: alpha_op(r.u8()?)?, swapped: r.bool()? })
}

fn write_fade<W: Write>(w: &mut Writer<W>, f: &Fade) {
    w.put(&f.color1);
    w.put(&f.color2);
    w.f32(f.period);
    w.f32(f.phase);
    w.bool(f.sinusoidal);
}

fn read_fade<I: Read>(r: &mut Reader<I>) -> R<Fade> {
    let color1 = r.array()?;
    let color2 = r.array()?;
    Ok(Fade { color1, color2, period: r.f32()?, phase: r.f32()?, sinusoidal: r.bool()? })
}

fn write_pass<W: Write>(w: &mut Writer<W>, p: &l2_catalog::Pass) {
    w.len(p.stages.start);
    w.len(p.stages.end);
    w.u8(blend_code(p.src));
    w.u8(blend_code(p.dst));
    w.bool(p.alpha_blend);
    w.bool(p.z_write);
    w.bool(p.z_test);
    w.option(p.alpha_test, |w, v| w.u8(v));
    w.bool(p.two_sided);
    w.u8(match p.fog {
        FogMode::Zone => 0,
        FogMode::Black => 1,
        FogMode::Gray127 => 2,
    });
    w.option(p.tfactor, |w, c| w.put(&c));
    w.option(p.fade, |w, f| write_fade(w, &f));
    w.bool(p.fade_alpha_replicate);
    w.bool(p.lit);
    w.len(p.light_after);
}

fn read_pass<I: Read>(r: &mut Reader<I>) -> R<l2_catalog::Pass> {
    let stages = Range { start: r.len()?, end: r.len()? };
    Ok(l2_catalog::Pass {
        stages,
        src: blend(r.u8()?)?,
        dst: blend(r.u8()?)?,
        alpha_blend: r.bool()?,
        z_write: r.bool()?,
        z_test: r.bool()?,
        alpha_test: r.option(|r| r.u8())?,
        two_sided: r.bool()?,
        fog: match r.u8()? {
            0 => FogMode::Zone,
            1 => FogMode::Black,
            2 => FogMode::Gray127,
            _ => return Err(CacheError::Invalid("fog")),
        },
        tfactor: r.option(|r| r.array())?,
        fade: r.option(read_fade)?,
        fade_alpha_replicate: r.bool()?,
        lit: r.bool()?,
        light_after: r.len()?,
    })
}

fn write_material<W: Write>(w: &mut Writer<W>, m: &Material) {
    w.len(m.stages.len());
    for s in &m.stages {
        write_stage(w, s);
    }
    w.len(m.passes.len());
    for p in &m.passes {
        write_pass(w, p);
    }
    w.u8(m.sorting_level);
    w.bool(m.transparent);
    w.str(&m.root_class);
    w.option(m.color_modifier.as_deref(), |w, s| w.str(s));
}

fn read_material<I: Read>(r: &mut Reader<I>) -> R<Material> {
    Ok(Material {
        stages: r.vec(read_stage)?,
        passes: r.vec(read_pass)?,
        sorting_level: r.u8()?,
        transparent: r.bool()?,
        root_class: r.str()?,
        color_modifier: r.option(|r| r.str())?,
    })
}

const COLUMN_POSITION: u8 = 1;
const COLUMN_NORMAL: u8 = 2;
const COLUMN_UV: u8 = 4;
const COLUMN_MASK_UV: u8 = 8;
const COLUMN_COLOR: u8 = 16;

fn constant_columns(vertices: &[Vertex]) -> u8 {
    let Some(first) = vertices.first() else { return 0 };
    let same = |bit: u8, pick: fn(&Vertex) -> &[f32]| {
        let reference = pick(first);
        if vertices.iter().all(|v| pick(v).iter().zip(reference).all(|(a, b)| a.to_bits() == b.to_bits())) { bit } else { 0 }
    };
    same(COLUMN_POSITION, |v| &v.position)
        | same(COLUMN_NORMAL, |v| &v.normal)
        | same(COLUMN_UV, |v| &v.uv)
        | same(COLUMN_MASK_UV, |v| &v.mask_uv)
        | same(COLUMN_COLOR, |v| &v.color)
}

fn write_column<W: Write>(w: &mut Writer<W>, vertices: &[Vertex], constant: bool, pick: fn(&Vertex) -> &[f32]) {
    if constant {
        w.f32s(pick(&vertices[0]));
        return;
    }
    let mut values = Vec::with_capacity(vertices.len().min(4096) * pick(&vertices[0]).len());
    for chunk in vertices.chunks(4096) {
        values.clear();
        values.extend(chunk.iter().flat_map(|v| pick(v).iter().copied()));
        w.put(bytemuck::cast_slice(&values));
    }
}

fn write_vertices<W: Write>(w: &mut Writer<W>, vertices: &[Vertex]) {
    w.len(vertices.len());
    if vertices.is_empty() {
        return;
    }
    let columns = constant_columns(vertices);
    w.u8(columns);
    write_column(w, vertices, columns & COLUMN_POSITION != 0, |v| &v.position);
    write_column(w, vertices, columns & COLUMN_NORMAL != 0, |v| &v.normal);
    write_column(w, vertices, columns & COLUMN_UV != 0, |v| &v.uv);
    write_column(w, vertices, columns & COLUMN_MASK_UV != 0, |v| &v.mask_uv);
    write_column(w, vertices, columns & COLUMN_COLOR != 0, |v| &v.color);
}

fn read_column<I: Read, const N: usize>(r: &mut Reader<I>, vertices: &mut [Vertex], constant: bool, pick: fn(&mut Vertex) -> &mut [f32; N]) -> R<()> {
    if constant {
        let value = r.f32s::<N>()?;
        for v in vertices.iter_mut() {
            *pick(v) = value;
        }
        return Ok(());
    }
    let mut values = vec![0f32; vertices.len().min(4096) * N];
    for vertices in vertices.chunks_mut(4096) {
        let values = &mut values[..vertices.len() * N];
        r.input.read_exact(bytemuck::cast_slice_mut(values)).map_err(|_| CacheError::Truncated)?;
        for (v, chunk) in vertices.iter_mut().zip(values.as_chunks::<N>().0) {
            pick(v).copy_from_slice(chunk);
        }
    }
    Ok(())
}

fn read_vertices<I: Read>(r: &mut Reader<I>) -> R<Vec<Vertex>> {
    let count = r.len()?;
    if count == 0 {
        return Ok(Vec::new());
    }
    let columns = r.u8()?;
    let mut vertices = vec![Vertex::zeroed(); count];
    read_column(r, &mut vertices, columns & COLUMN_POSITION != 0, |v| &mut v.position)?;
    read_column(r, &mut vertices, columns & COLUMN_NORMAL != 0, |v| &mut v.normal)?;
    read_column(r, &mut vertices, columns & COLUMN_UV != 0, |v| &mut v.uv)?;
    read_column(r, &mut vertices, columns & COLUMN_MASK_UV != 0, |v| &mut v.mask_uv)?;
    read_column(r, &mut vertices, columns & COLUMN_COLOR != 0, |v| &mut v.color)?;
    Ok(vertices)
}

fn write_draw<W: Write>(w: &mut Writer<W>, d: &MeshDraw) {
    write_vertices(w, &d.vertices);
    w.bytes(bytemuck::cast_slice(&d.instances));
    w.bytes(bytemuck::cast_slice(&d.indices));
    write_material(w, &d.material);
    w.option(d.mask.as_deref(), |w, s| w.str(s));
    w.u8(match d.kind {
        DrawKind::StaticMesh => 0,
        DrawKind::Bsp => 1,
        DrawKind::Terrain => 2,
        DrawKind::Actor => 3,
        DrawKind::Sprite => 4,
        DrawKind::Particle => 5,
        DrawKind::Projector => 6,
    });
    w.bool(d.sun_affect);
    match d.pass {
        Pass::Opaque => w.u8(0),
        Pass::Layer(n) => {
            w.u8(1);
            w.u16(n);
        }
        Pass::Translucent => w.u8(2),
        Pass::Projector => w.u8(3),
    }
    w.u8(d.zone);
    w.option(d.lightmap, |w, i| w.len(i));
    w.bool(d.modulate_2x);
    w.option(d.sector, |w, s| {
        w.f32s(&s.location);
        w.f32s(&s.min);
        w.f32s(&s.max);
    });
    w.option(d.fade, |w, f| w.f32s(&f));
    w.option(d.tint, |w, t| match t {
        SkyTint::SkyBox => w.u8(0),
        SkyTint::Hazering => w.u8(1),
        SkyTint::Cloud(i) => {
            w.u8(2);
            w.len(i);
        }
    });
    w.option(d.actor, |w, a| {
        w.u32(a.id);
        w.f32s(&a.ambient.vector);
        w.u8(a.ambient.glow);
        w.f32s(&a.xy);
        w.f32(a.range);
        w.bool(a.ignored_range);
        w.option(a.time_reactor, |w, t| w.f32s(&t));
        w.option(a.lod, |w, l| {
            w.u8(l.level);
            w.bool(l.blend);
            w.f32s(&l.ranges);
        });
    });
}

fn pod_vec<T: bytemuck::Pod, I: Read>(r: &mut Reader<I>) -> R<Vec<T>> {
    let bytes = r.len()?;
    let size = std::mem::size_of::<T>();
    if !bytes.is_multiple_of(size) {
        return Err(CacheError::Invalid("pod"));
    }
    let mut out: Vec<T> = vec![T::zeroed(); bytes / size];
    r.input.read_exact(bytemuck::cast_slice_mut(&mut out)).map_err(|_| CacheError::Truncated)?;
    Ok(out)
}

fn read_draw<I: Read>(r: &mut Reader<I>) -> R<MeshDraw> {
    let vertices = read_vertices(r)?;
    let instances = pod_vec(r)?;
    let indices: Vec<u32> = pod_vec(r)?;
    let material = read_material(r)?;
    let mask = r.option(|r| r.str())?;
    let kind = match r.u8()? {
        0 => DrawKind::StaticMesh,
        1 => DrawKind::Bsp,
        2 => DrawKind::Terrain,
        3 => DrawKind::Actor,
        4 => DrawKind::Sprite,
        5 => DrawKind::Particle,
        6 => DrawKind::Projector,
        _ => return Err(CacheError::Invalid("kind")),
    };
    let sun_affect = r.bool()?;
    let pass = match r.u8()? {
        0 => Pass::Opaque,
        1 => Pass::Layer(r.u16()?),
        2 => Pass::Translucent,
        3 => Pass::Projector,
        _ => return Err(CacheError::Invalid("pass")),
    };
    Ok(MeshDraw {
        instances, vertices,
        indices,
        material,
        mask,
        kind,
        sun_affect,
        pass,
        zone: r.u8()?,
        lightmap: r.option(|r| r.len())?,
        modulate_2x: r.bool()?,
        sector: r.option(|r| Ok(SectorBounds { location: r.f32s()?, min: r.f32s()?, max: r.f32s()? }))?,
        fade: r.option(|r| r.f32s())?,
        tint: r.option(|r| {
            Ok(match r.u8()? {
                0 => SkyTint::SkyBox,
                1 => SkyTint::Hazering,
                2 => SkyTint::Cloud(r.len()?),
                _ => return Err(CacheError::Invalid("tint")),
            })
        })?,
        actor: r.option(|r| {
            Ok(ActorCull {
                id: r.u32()?,
                ambient: crate::scene::actor_lighting::Ambient { vector: r.f32s()?, glow: r.u8()? },
                xy: r.f32s()?,
                range: r.f32()?,
                ignored_range: r.bool()?,
                time_reactor: r.option(|r| r.f32s())?,
                lod: r.option(|r| Ok(LodSelect { level: r.u8()?, blend: r.bool()?, ranges: r.f32s()? }))?,
            })
        })?,
    })
}

fn format_code(f: TextureFormatKind) -> u8 {
    match f {
        TextureFormatKind::Rgba8 => 0,
        TextureFormatKind::Bc1 => 1,
        TextureFormatKind::Bc2 => 2,
        TextureFormatKind::Bc3 => 3,
    }
}

fn format(code: u8) -> R<TextureFormatKind> {
    Ok(match code {
        0 => TextureFormatKind::Rgba8,
        1 => TextureFormatKind::Bc1,
        2 => TextureFormatKind::Bc2,
        3 => TextureFormatKind::Bc3,
        _ => return Err(CacheError::Invalid("format")),
    })
}

fn write_texture<W: Write>(w: &mut Writer<W>, t: &TextureData) {
    w.u32(t.width);
    w.u32(t.height);
    w.u8(format_code(t.format));
    w.len(t.mips.len());
    for mip in &t.mips {
        w.bytes(mip);
    }
}

fn read_texture<I: Read>(r: &mut Reader<I>) -> R<TextureData> {
    Ok(TextureData { width: r.u32()?, height: r.u32()?, format: format(r.u8()?)?, mips: r.vec(|r| r.bytes())? })
}

fn write_zone<W: Write>(w: &mut Writer<W>, z: &ZoneLighting) {
    w.option(z.fog, |w, f| {
        w.f32s(&f.color);
        w.f32(f.start);
        w.f32(f.end);
        w.f32(f.blend_time);
    });
    w.bool(z.sun_affect);
    w.u8(z.ambient_brightness);
    w.option(z.ambient_vector, |w, v| w.f32s(&v));
    w.bool(z.is_sky);
    w.f32s(&z.sky_location);
    w.bool(z.terrain_zone);
    w.bool(z.clear_to_fog);
}

fn read_zone<I: Read>(r: &mut Reader<I>) -> R<ZoneLighting> {
    Ok(ZoneLighting {
        fog: r.option(|r| Ok(FogParams { color: r.f32s()?, start: r.f32()?, end: r.f32()?, blend_time: r.f32()? }))?,
        sun_affect: r.bool()?,
        ambient_brightness: r.u8()?,
        ambient_vector: r.option(|r| r.f32s())?,
        is_sky: r.bool()?,
        sky_location: r.f32s()?,
        terrain_zone: r.bool()?,
        clear_to_fog: r.bool()?,
    })
}

fn write_fog_info<W: Write>(w: &mut Writer<W>, info: &ue2_level::L2FogInfo) {
    w.f32s(&info.location);
    w.f32s(&info.affect_range);
    for range in &info.fog_range {
        w.f32s(range);
    }
    w.f32(info.texture_distance);
    w.len(info.colors.len());
    for color in &info.colors {
        w.f32(color.time);
        w.put(&color.fog);
        w.put(&color.sky);
        w.len(color.cloud.len());
        for rgba in &color.cloud {
            w.put(rgba);
        }
        w.len(color.haze_ring.len());
        for rgba in &color.haze_ring {
            w.put(rgba);
        }
    }
    w.option(info.cloud_texture.as_deref(), |w, t| w.str(t));
}

fn read_fog_info<I: Read>(r: &mut Reader<I>) -> R<ue2_level::L2FogInfo> {
    let location = r.f32s()?;
    let affect_range = r.f32s()?;
    let mut fog_range = [[0.0; 2]; ue2_level::FOG_RANGE_LEVELS];
    for range in &mut fog_range {
        *range = r.f32s()?;
    }
    let texture_distance = r.f32()?;
    let colors = r.vec(|r| {
        Ok(ue2_level::FogColorInfo {
            time: r.f32()?,
            fog: r.array::<4>()?,
            sky: r.array::<4>()?,
            cloud: r.vec(|r| r.array::<4>())?,
            haze_ring: r.vec(|r| r.array::<4>())?,
        })
    })?;
    let cloud_texture = r.option(|r| r.str())?;
    Ok(ue2_level::L2FogInfo { location, affect_range, fog_range, texture_distance, colors, cloud_texture })
}

fn write_body<W: Write>(w: &mut Writer<W>, b: &SkyBodyDraw) {
    w.u8(match b.kind {
        BodyKind::Sun => 0,
        BodyKind::Moon => 1,
    });
    w.u8(b.env_type);
    w.f32s(&b.origin);
    w.f32(b.radius);
    w.f32(b.half_width_per_scale);
    write_material(w, &b.material);
    w.u8(b.zone);
}

fn read_body<I: Read>(r: &mut Reader<I>) -> R<SkyBodyDraw> {
    Ok(SkyBodyDraw {
        kind: match r.u8()? {
            0 => BodyKind::Sun,
            1 => BodyKind::Moon,
            _ => return Err(CacheError::Invalid("body")),
        },
        env_type: r.u8()?,
        origin: r.f32s()?,
        radius: r.f32()?,
        half_width_per_scale: r.f32()?,
        material: read_material(r)?,
        zone: r.u8()?,
    })
}

pub fn encode_to_vec(stamp: &SourceStamp, cached: &CachedScene) -> Vec<u8> {
    let mut out = Vec::with_capacity(1 << 20);
    if encode(&mut out, stamp, &cached.scene, &cached.textures, &cached.alpha_maps).is_err() {
        out.clear();
    }
    out
}

pub fn encode<W: Write>(out: W, stamp: &SourceStamp, scene: &Scene, textures: &[(String, Arc<TextureData>)], alpha_maps: &[(String, Arc<TextureData>)]) -> io::Result<()> {
    encode_draws(out, stamp, scene, textures, alpha_maps, scene.draws.iter(), |_| {})
}

pub fn encode_consuming<W: Write>(out: W, stamp: &SourceStamp, mut scene: Scene, consume: impl FnMut(MeshDraw)) -> io::Result<()> {
    let draws = std::mem::take(&mut scene.draws);
    encode_draws(out, stamp, &scene, &[], &[], draws.into_iter(), consume)
}

fn encode_draws<W: Write, T: std::borrow::Borrow<MeshDraw>>(out: W, stamp: &SourceStamp, scene: &Scene, textures: &[(String, Arc<TextureData>)], alpha_maps: &[(String, Arc<TextureData>)], draws: impl ExactSizeIterator<Item = T>, mut consume: impl FnMut(T)) -> io::Result<()> {
    let mut w = Writer { out, error: None };
    w.put(MAGIC);
    w.u32(FORMAT_VERSION);
    w.u64(stamp.size);
    w.u64(stamp.hash);
    w.len(scene.projectors.len());
    for projector in &scene.projectors {
        w.bool(projector.enabled);
        w.bool(projector.visibility.only_owner);
        w.bool(projector.visibility.owner_no_see);
        w.bool(projector.visibility.high_detail);
        w.bool(projector.visibility.super_high_detail);
        w.u8(projector.detail);
        w.len(projector.owners.len());
        for owner in &projector.owners { w.str(owner); }
        w.len(projector.draws.len());
        for &draw in &projector.draws { w.u32(draw as u32); }
    }
    w.len(scene.moving.len());
    for actor in &scene.moving { movers::write(&mut w, actor); }
    w.len(draws.len());
    for draw in draws {
        write_draw(&mut w, draw.borrow());
        if let Some(error) = w.error.take() { return Err(error); }
        consume(draw);
    }
    w.len(scene.lightmaps.len());
    for group in &scene.lightmaps {
        w.u32(group.width);
        w.u32(group.height);
        w.u8(format_code(group.format));
        w.len(group.slices.len());
        for slice in &group.slices {
            w.len(slice.len());
            for layer in slice {
                w.bytes(layer);
            }
        }
    }
    w.len(scene.zones.len());
    for zone in &scene.zones {
        write_zone(&mut w, zone);
    }
    w.len(scene.lights.len());
    for light in &scene.lights {
        w.u32(light.source as u32);
        w.f32s(&light.position.to_array());
        w.f32(light.radius);
        w.f32s(&light.color);
        w.f32(light.brightness);
        w.f32s(&light.direction.to_array());
        w.u8(light.effect);
        w.u8(light.cone);
        w.bool(light.special_lit);
    }
    w.f32s(&scene.bounds_min);
    w.len(scene.coronas.len());
    for corona in &scene.coronas {
        w.f32s(&corona.position.to_array());
        w.f32(corona.radius);
        w.u8(corona.hue);
        w.u8(corona.saturation);
        w.f32(corona.min_size);
        w.f32(corona.max_size);
        w.f32(corona.draw_scale);
        w.f32(corona.scale_glow);
        write_material(&mut w, &corona.material);
    }
    w.f32s(&scene.bounds_max);
    w.f32(scene.terrain_range);
    w.len(scene.sky_bodies.len());
    for body in &scene.sky_bodies {
        write_body(&mut w, body);
    }
    w.len(scene.flares.len());
    for flare in &scene.flares {
        w.str(&flare.texture);
        w.f32(flare.offset);
        w.f32(flare.scale);
    }
    w.f32(scene.global_fog.start);
    w.f32(scene.global_fog.end);
    w.f32(scene.global_fog.speed);
    w.u32(scene.global_fog.level as u32);
    w.len(scene.fog_infos.len());
    for info in &scene.fog_infos {
        write_fog_info(&mut w, info);
    }
    for list in [textures, alpha_maps] {
        w.len(list.len());
        for (name, data) in list {
            w.str(name);
            write_texture(&mut w, data);
        }
    }
    match w.error {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

pub fn decode<I: Read>(input: I, stamp: &SourceStamp) -> R<CachedScene> {
    let mut r = Reader { input };
    if r.array::<8>()? != *MAGIC {
        return Err(CacheError::BadMagic);
    }
    let version = r.u32()?;
    if version != FORMAT_VERSION {
        return Err(CacheError::Version(version));
    }
    let stored = SourceStamp { size: r.u64()?, hash: r.u64()? };
    if stored != *stamp {
        return Err(CacheError::Stale);
    }
    let projectors = r.vec(|r| Ok(crate::scene::projectors::ProjectorState {
        enabled: r.bool()?,
        visibility: ue2_level::projectors::Visibility {
            only_owner: r.bool()?, owner_no_see: r.bool()?,
            high_detail: r.bool()?, super_high_detail: r.bool()?,
        },
        detail: r.u8()?,
        owners: r.vec(|r| r.str())?,
        draws: r.vec(|r| Ok(r.u32()? as usize))?,
    }))?;
    let moving = r.vec(movers::read)?;
    let draws = r.vec(read_draw)?;
    let lightmaps = r.vec(|r| {
        Ok(LightmapGroup {
            width: r.u32()?,
            height: r.u32()?,
            format: format(r.u8()?)?,
            slices: r.vec(|r| r.vec(|r| r.bytes()))?,
        })
    })?;
    let zones = r.vec(read_zone)?;
    let lights = r.vec(|r| Ok(PointLight { source: r.u32()? as usize, position: Vec3::from(r.f32s::<3>()?), radius: r.f32()?, color: r.f32s()?, brightness: r.f32()?, direction: Vec3::from(r.f32s::<3>()?), effect: r.u8()?, cone: r.u8()?, special_lit: r.bool()? }))?;
    let bounds_min = r.f32s()?;
    let coronas = r.vec(|r| Ok(crate::scene::lights::CoronaLight {
        position: Vec3::from(r.f32s::<3>()?), radius: r.f32()?, hue: r.u8()?, saturation: r.u8()?,
        min_size: r.f32()?, max_size: r.f32()?, draw_scale: r.f32()?, scale_glow: r.f32()?, material: read_material(r)?,
    }))?;
    let bounds_max = r.f32s()?;
    let terrain_range = r.f32()?;
    let sky_bodies = r.vec(read_body)?;
    let flares = r.vec(|r| Ok(Flare { texture: r.str()?, offset: r.f32()?, scale: r.f32()? }))?;
    let global_fog = GlobalFog { start: r.f32()?, end: r.f32()?, speed: r.f32()?, level: r.u32()? as usize };
    let fog_infos = r.vec(read_fog_info)?;
    let textures = r.vec(|r| Ok((r.str()?, Arc::new(read_texture(r)?))))?;
    let alpha_maps = r.vec(|r| Ok((r.str()?, Arc::new(read_texture(r)?))))?;
    if !r.at_end()? {
        return Err(CacheError::Invalid("sobra"));
    }
    let scene = Scene { projectors, moving, emitters: Vec::new(), draws, lightmaps, zones, lights, coronas, bounds_min, bounds_max, terrain_range, sky_bodies, flares, global_fog, fog_infos };
    Ok(CachedScene { scene, textures, alpha_maps })
}

#[cfg(test)]
mod tests {
    #[test]
    fn texture_operation_codes_preserve_old_values_and_round_trip_new_operators() {
        use super::{color_op, color_op_code};
        use l2_catalog::ColorOp;
        for (code, op) in [ColorOp::SelectTexture, ColorOp::SelectCurrent, ColorOp::Modulate,
            ColorOp::Modulate2X, ColorOp::Modulate4X, ColorOp::Add, ColorOp::Subtract,
            ColorOp::BlendTextureAlpha, ColorOp::BlendDiffuseAlpha, ColorOp::BlendCurrentAlpha,
            ColorOp::BlendFactorAlpha, ColorOp::ModulateAlphaAddColor, ColorOp::MultiplyAdd,
            ColorOp::BlendCurrentFactorByTextureAlpha, ColorOp::BlendTextureFactorByDiffuseAlpha,
            ColorOp::ModulateCurrentByTextureAlpha, ColorOp::AddSigned, ColorOp::AddSigned2X,
            ColorOp::AddSmooth, ColorOp::BlendTextureAlphaPremultiplied, ColorOp::ModulateColorAddAlpha,
            ColorOp::ModulateInvAlphaAddColor, ColorOp::ModulateInvColorAddAlpha,
            ColorOp::Disable, ColorOp::PreModulate, ColorOp::MultiplyAddCurrent,
            ColorOp::DotProduct3].into_iter().enumerate() {
            assert_eq!(color_op_code(op), code as u8);
            assert_eq!(color_op(code as u8).unwrap(), op);
        }
        for code in 27..=255 {
            assert!(color_op(code).is_err());
        }
    }

    use super::*;
    use crate::scene::texture_material;

    fn sample_scene() -> Scene {
        let mut material = texture_material("L2_Lobby_T.Tree.Bark");
        material.stages.push(Stage {
            source: StageSource::Cubemap(Cubemap { faces: std::array::from_fn(|i| format!("Sky.Face{i}")), size: [256.0, 256.0] }),
            uv: vec![
                UvModifier::Panner { direction: [1, 2, 3], rate: 0.5 },
                UvModifier::Oscillator(Oscillator { rate: [0.1, 0.2], phase: [0.3, 0.4], amplitude: [0.5, 0.6], kind: [1, 2], offset: [0.7, 0.8] }, [64.0, 32.0]),
                UvModifier::Rotator(Rotator { kind: 2, rotation: [1, 2, 3], offset: [0.5, 0.5], oscillation_rate: [4, 5, 6], oscillation_amplitude: [7, 8, 9], oscillation_phase: [10, 11, 12] }, [128.0, 128.0]),
                UvModifier::Scaler { scale: [2.0, 2.0], offset: [1.0, 1.0], size: [8.0, 8.0] },
                UvModifier::PannerTriggered { direction: [4, 5, 6], rate: 0.25, trigger: Trigger { stop_after: 0.5, triggered_time: 1.5, reverse: true } },
                UvModifier::OscillatorTriggered(Oscillator { rate: [1.0, 1.0], phase: [0.0, 0.5], amplitude: [0.1, 0.1], kind: [0, 1], offset: [0.0, 0.0] }, [16.0, 16.0], Trigger { stop_after: 2.0, triggered_time: 0.0, reverse: false }),
                UvModifier::Matrix(Mat3x2([[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.5, 0.25, 1.0]])),
            ],
            coord_source: CoordSource::CameraEnvMap,
            color_op: ColorOp::BlendFactorAlpha,
            alpha_op: AlphaOp::TFactor,
            swapped: true,
        });
        material.passes.push(l2_catalog::Pass {
            stages: 1..2,
            src: BlendFactor::SrcAlpha,
            dst: BlendFactor::InvSrcAlpha,
            alpha_blend: true,
            z_write: false,
            z_test: true,
            alpha_test: Some(160),
            two_sided: true,
            fog: FogMode::Gray127,
            tfactor: Some([1, 2, 3, 4]),
            fade: Some(Fade { color1: [1, 1, 1, 1], color2: [2, 2, 2, 2], period: 3.0, phase: 0.5, sinusoidal: true }),
            fade_alpha_replicate: true,
            lit: false,
            light_after: 1,
        });
        material.color_modifier = Some("Sky.Mod".into());
        material.sorting_level = 3;
        material.transparent = true;
        let vertex = |x: f32| Vertex { position: [x, 2.0, 3.0], normal: [0.0, 0.0, 1.0], uv: [0.5, 0.25], mask_uv: [0.1, 0.2], color: [1.0, 0.5, 0.25, 1.0] };
        let draws = vec![
            MeshDraw {
                instances: Vec::new(), vertices: vec![vertex(0.0), vertex(1.0), vertex(2.0)],
                indices: vec![0, 1, 2],
                material: material.clone(),
                mask: Some("T_texture.Texture.layer0".into()),
                kind: DrawKind::Terrain,
                sun_affect: true,
                pass: Pass::Layer(7),
                zone: 1,
                lightmap: Some(0),
                modulate_2x: true,
                sector: Some(SectorBounds { location: [1.0; 3], min: [0.0; 3], max: [2.0; 3] }),
                fade: Some([100.0, 200.0]),
                tint: Some(SkyTint::Cloud(2)),
                actor: Some(ActorCull { id: 77, ambient: crate::scene::actor_lighting::Ambient { vector: [0.1, 0.2, 0.3], glow: 255 }, xy: [1.0, 2.0], range: 3000.0, ignored_range: false, time_reactor: Some([6.0, 18.0]), lod: Some(LodSelect { level: 1, blend: true, ranges: [1024.0, 2048.0] }) }),
            },
            MeshDraw {
                instances: Vec::new(), vertices: vec![vertex(3.0), vertex(3.0), Vertex { uv: [0.0, 0.0], ..vertex(3.0) }, Vertex { color: [0.0; 4], ..vertex(3.0) }],
                indices: vec![0, 0, 0],
                material: texture_material("Bark"),
                mask: None,
                kind: DrawKind::Sprite,
                sun_affect: false,
                pass: Pass::Translucent,
                zone: 0,
                lightmap: None,
                modulate_2x: false,
                sector: None,
                fade: None,
                tint: None,
                actor: None,
            },
        ];
        Scene {
            projectors: vec![crate::scene::projectors::ProjectorState {
                enabled: true,
                visibility: ue2_level::projectors::Visibility { only_owner: true, owner_no_see: false, high_detail: true, super_high_detail: true },
                owners: vec!["Projector0".into(), "Owner0".into()], detail: 2, draws: vec![0, 1],
            }],
            moving: Vec::new(),
            draws,
            emitters: Vec::new(),
            coronas: vec![crate::scene::lights::CoronaLight { position: Vec3::new(4.0, 5.0, 6.0), radius: 125.0, hue: 26, saturation: 153, min_size: 2.0, max_size: 400.0, draw_scale: 0.25, scale_glow: 0.5, material: crate::scene::sky::flare_material("corona") }],
            lightmaps: vec![LightmapGroup { width: 4, height: 2, format: TextureFormatKind::Rgba8, slices: vec![vec![vec![1, 2, 3], vec![4]], vec![]] }],
            zones: vec![
                ZoneLighting::default(),
                ZoneLighting { fog: Some(FogParams { color: [0.1, 0.2, 0.3], start: 10.0, end: 20.0, blend_time: 1.5 }), sun_affect: false, ambient_brightness: 30, ambient_vector: Some([0.2; 3]), is_sky: true, sky_location: [1.0, 2.0, 3.0], terrain_zone: true, clear_to_fog: true },
            ],
            lights: vec![PointLight { position: Vec3::new(1.0, 2.0, 3.0), radius: 64.0, color: [1.0, 0.5, 0.0], brightness: 64.0, ..Default::default() }],
            bounds_min: [-1.0; 3],
            bounds_max: [1.0; 3],
            terrain_range: 12000.0,
            sky_bodies: vec![SkyBodyDraw { kind: BodyKind::Moon, env_type: 2, origin: [0.0, 1.0, 2.0], radius: 9000.0, half_width_per_scale: 16.0, material, zone: 4 }],
            flares: vec![Flare { texture: "L2_Skies.FlareSet.Flare02".into(), offset: -0.12, scale: 0.5 }],
            global_fog: GlobalFog { start: 8192.0, end: 20480.0, speed: 1.0, level: 3 },
            fog_infos: vec![ue2_level::L2FogInfo {
                location: [1.0, 2.0, 3.0],
                affect_range: [0.0, 5000.0],
                fog_range: [[0.0, 1000.0], [0.0, 2000.0], [0.0, 3000.0], [0.0, 4000.0], [0.0, 5000.0]],
                texture_distance: 1500.0,
                colors: vec![ue2_level::FogColorInfo {
                    time: 6.0,
                    fog: [10, 20, 30, 255],
                    sky: [40, 50, 60, 255],
                    cloud: vec![[1, 2, 3, 4], [5, 6, 7, 8]],
                    haze_ring: vec![[9, 10, 11, 12]],
                }],
                cloud_texture: Some("L2_Skies.Cloud.Cloud01".into()),
            }],
        }
    }

    fn sample() -> CachedScene {
        let mut bank = TextureBank::new();
        bank.insert("L2_Lobby_T.Tree.Bark", TextureData { width: 4, height: 4, format: TextureFormatKind::Bc1, mips: vec![vec![9; 8], vec![1; 8]] });
        bank.insert_shared_alpha_map("T_texture.Texture.layer0", Arc::new(TextureData { width: 2, height: 2, format: TextureFormatKind::Rgba8, mips: vec![vec![7; 16]] }));
        CachedScene::from_bank(sample_scene(), &bank)
    }

    #[test]
    fn decoration_instance_cache_preserves_every_float_bit() {
        let mut scene = sample_scene();
        scene.draws[0].instances = vec![crate::scene::DecorationInstance {
            world: std::array::from_fn(|column| std::array::from_fn(|row| (column * 4 + row) as f32 * -0.75)),
            mask_uv: [-0.0, f32::from_bits(0x7fc00017)], color: [0.1, 0.3, 0.7, 1.0],
        }];
        let stamp = SourceStamp { size: 0, hash: 0 };
        let mut bytes = Vec::new();
        encode(&mut bytes, &stamp, &scene, &[], &[]).unwrap();
        let restored = decode(bytes.as_slice(), &stamp).unwrap();
        assert_eq!(bytemuck::cast_slice::<_, u8>(&scene.draws[0].instances), bytemuck::cast_slice::<_, u8>(&restored.scene.draws[0].instances));
        assert!(restored.scene.draws[1].instances.is_empty());
        let mut consuming = Vec::new();
        encode_consuming(&mut consuming, &stamp, scene, |_| {}).unwrap();
        assert_eq!(consuming, bytes);
    }

    #[test]
    fn retaining_scene_draws_remaps_projector_receivers_without_removing_projectors() {
        let mut scene = sample_scene();
        let first = scene.draws[0].clone();
        scene.retain_draws(|draw| draw != &first);
        assert_eq!(scene.projectors.len(), 1);
        assert_eq!(scene.projectors[0].draws, vec![0]);
        scene.retain_draws(|_| false);
        assert_eq!(scene.projectors.len(), 1);
        assert!(scene.projectors[0].draws.is_empty());
    }

    #[test]
    fn consuming_scene_preserves_bytes_and_stops_handoff_on_write_failure() {
        let stamp = SourceStamp { size: 17, hash: 23 };
        let scene = sample_scene();
        let mut expected = Vec::new();
        encode(&mut expected, &stamp, &scene, &[], &[]).unwrap();
        let count = scene.draws.len();
        let mut actual = Vec::new();
        let mut consumed = 0;
        encode_consuming(&mut actual, &stamp, scene, |_| consumed += 1).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(consumed, count);

        struct Failing;
        impl Write for Failing {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> { Err(io::Error::other("falha esperada")) }
            fn flush(&mut self) -> io::Result<()> { Ok(()) }
        }
        let mut consumed = 0;
        assert!(encode_consuming(Failing, &stamp, sample_scene(), |_| consumed += 1).is_err());
        assert_eq!(consumed, 0);
    }

    #[test]
    fn cache_columns_keep_identical_bytes_across_chunk_boundaries() {
        let vertices: Vec<_> = (0..8201).map(|index| Vertex { position:[index as f32,0.0,-0.0],normal:[0.0,0.0,1.0],uv:[index as f32 / 3.0,0.5],mask_uv:[0.0;2],color:[1.0;4] }).collect();
        let mut bytes = Vec::new();
        write_column(&mut Writer {out:&mut bytes,error:None},&vertices,false,|v| &v.position);
        let expected: Vec<f32> = vertices.iter().flat_map(|v|v.position).collect();
        assert_eq!(bytes,bytemuck::cast_slice::<f32,u8>(&expected));
        let mut restored=vec![Vertex::zeroed();vertices.len()];
        read_column(&mut Reader {input:bytes.as_slice()},&mut restored,false,|v| &mut v.position).unwrap();
        for (original,restored) in vertices.iter().zip(&restored) {
            assert_eq!(original.position.map(f32::to_bits),restored.position.map(f32::to_bits));
        }
    }

    #[test]
    #[ignore]
    fn original_lobby_scene_survives_worker_cache_without_textures() {
        let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let mut client = crate::client::Client::open(Path::new(&root)).unwrap();
        let level = client.load_level("lobby").unwrap();
        let options = crate::options::ClientOptions::load(&mut client).unwrap();
        let mut scene = crate::scene::build_scene_with(&mut client,&level.level,&options,Default::default());
        let emitters = crate::scene::emitters::build(&mut client,&level.level,&options);
        assert!(scene.emitters == emitters, "emitters reconstruídos devem ser idênticos");
        scene.emitters.clear();
        drop(emitters);
        drop(level);
        drop(client);
        let count = scene.draws.len();
        for (index,draw) in std::mem::take(&mut scene.draws).into_iter().enumerate() {
            let mut bytes = Vec::new();
            let mut writer = Writer {out:&mut bytes,error:None};
            write_draw(&mut writer,&draw);
            assert!(writer.error.is_none());
            let loaded = read_draw(&mut Reader {input:bytes.as_slice()}).unwrap();
            assert!(bytemuck::cast_slice::<Vertex,u8>(&draw.vertices) == bytemuck::cast_slice::<Vertex,u8>(&loaded.vertices), "vértices do draw {index}/{count} mudaram");
            assert!(draw.indices == loaded.indices, "índices do draw {index}/{count} mudaram");
            assert!(bytemuck::cast_slice::<crate::scene::DecorationInstance,u8>(&draw.instances) == bytemuck::cast_slice::<crate::scene::DecorationInstance,u8>(&loaded.instances), "instâncias do draw {index}/{count} mudaram");
            let mut draw = draw;
            let mut loaded = loaded;
            draw.instances.clear(); loaded.instances.clear();
            draw.vertices.clear(); draw.indices.clear();
            loaded.vertices.clear(); loaded.indices.clear();
            assert!(draw == loaded, "metadados do draw {index}/{count} mudaram");
        }
        let stamp = SourceStamp {size:0,hash:0};
        let mut bytes = Vec::new();
        encode(&mut bytes,&stamp,&scene,&[],&[]).unwrap();
        let cached = decode(bytes.as_slice(),&stamp).unwrap();
        assert!(scene == cached.scene, "metadados da cena mudaram no cache");
        assert!(cached.textures.is_empty());
        assert!(cached.alpha_maps.is_empty());
        eprintln!("lobby: {count} draws e todos os metadados conferidos; comparação um draw por vez");
    }

    #[test]
    fn a_scene_survives_the_round_trip_and_the_bank_gets_its_textures_back() {
        let cached = sample();
        let stamp = SourceStamp { size: 4_703_446, hash: fnv1a(b"lobby") };
        let bytes = encode_to_vec(&stamp, &cached);
        assert_eq!(&bytes[..8], MAGIC);
        let back = decode(&bytes[..], &stamp).unwrap();
        assert_eq!(back, cached);
        let (scene, bank) = back.into_bank();
        assert_eq!(scene, cached.scene);
        assert_eq!(bank.textures().count(), 1);
        assert_eq!(bank.alpha_maps().count(), 1);
        assert_eq!(bank.textures().next().unwrap().0, "l2_lobby_t.tree.bark");
    }

    #[test]
    fn a_different_source_version_or_damage_is_rejected() {
        let cached = sample();
        let stamp = SourceStamp { size: 10, hash: 20 };
        let bytes = encode_to_vec(&stamp, &cached);
        assert_eq!(decode(&bytes[..], &SourceStamp { size: 10, hash: 21 }), Err(CacheError::Stale));
        assert_eq!(decode(&bytes[..], &SourceStamp { size: 11, hash: 20 }), Err(CacheError::Stale));
        let mut wrong_version = bytes.clone();
        wrong_version[8] = FORMAT_VERSION as u8 + 1;
        assert_eq!(decode(&wrong_version[..], &stamp), Err(CacheError::Version(FORMAT_VERSION + 1)));
        let mut wrong_magic = bytes.clone();
        wrong_magic[0] = b'X';
        assert_eq!(decode(&wrong_magic[..], &stamp), Err(CacheError::BadMagic));
        assert_eq!(decode(&bytes[..bytes.len() - 1], &stamp), Err(CacheError::Truncated));
        let mut longer = bytes.clone();
        longer.push(0);
        assert_eq!(decode(&longer[..], &stamp), Err(CacheError::Invalid("sobra")));
        assert_eq!(decode(&[][..], &stamp), Err(CacheError::Truncated));
    }

    #[test]
    fn the_cache_file_is_named_by_map_stem_and_path_hash_and_written_atomically() {
        let dir = std::env::temp_dir().join(format!("killer-cache-{}", std::process::id()));
        let cache = SceneCache::at(dir.clone());
        let map = Path::new("/client/maps/Lobby.unr");
        let file = cache.file_for(map);
        assert!(file.starts_with(&dir));
        assert!(file.file_name().unwrap().to_str().unwrap().starts_with("lobby-"));
        assert!(file.extension().is_some_and(|e| e == "scene"));
        assert_eq!(cache.file_for(Path::new("/client/maps/LOBBY.UNR")), file);
        assert_ne!(cache.file_for(Path::new("/other/maps/lobby.unr")), file);
        let stamp = SourceStamp { size: 1, hash: 2 };
        assert_eq!(cache.read(map, &stamp), None);
        let cached = sample();
        let written = cache.write(map, &stamp, &cached).unwrap();
        assert_eq!(written, file);
        assert_eq!(cache.read(map, &stamp), Some(cached));
        assert_eq!(cache.read(map, &SourceStamp { size: 1, hash: 3 }), None);
        assert!(fs::read_dir(&dir).unwrap().flatten().all(|e| e.path().extension().is_some_and(|x| x == "scene")));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn constant_vertex_columns_are_stored_once() {
        let vertex = |x: f32, u: f32| Vertex { position: [x, 0.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [u, 0.0], mask_uv: [0.5, 0.5], color: [1.0; 4] };
        let vertices = vec![vertex(0.0, 0.0), vertex(1.0, 0.5), vertex(2.0, 1.0)];
        assert_eq!(constant_columns(&vertices), COLUMN_NORMAL | COLUMN_MASK_UV | COLUMN_COLOR);
        assert_eq!(constant_columns(&[]), 0);
        assert_eq!(constant_columns(&vertices[..1]), COLUMN_POSITION | COLUMN_NORMAL | COLUMN_UV | COLUMN_MASK_UV | COLUMN_COLOR);
        let mut out = Vec::new();
        let mut w = Writer { out: &mut out, error: None };
        write_vertices(&mut w, &vertices);
        assert_eq!(out.len(), 4 + 1 + 3 * 12 + 12 + 3 * 8 + 8 + 16);
        let back = read_vertices(&mut Reader { input: &out[..] }).unwrap();
        assert_eq!(back, vertices);
    }

    #[test]
    fn stamp_and_hash_are_stable() {
        assert_eq!(fnv1a(b""), FNV_OFFSET);
        assert_eq!(fnv1a(b"a"), 0xaf63_dc4c_8601_ec8c);
        let stamp = SourceStamp::of(b"abc");
        assert_eq!(stamp.size, 3);
        assert_eq!(stamp.hash, fnv1a(b"abc"));
        assert!(app_dir().is_some_and(|dir| dir.ends_with("killer")));
    }
}
