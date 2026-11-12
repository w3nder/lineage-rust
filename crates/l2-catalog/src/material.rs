use std::ops::Range;

use ue2_assets::{Property, find, object_properties};
use ue2_package::ObjectRef;

use crate::l2_color_stage::{self, MaterialPath, Modifier, ModifierFlags, PassFlags, PassState, StageState};
use crate::uv::{
    Mat3x2, Oscillator, Rotator, TR_CONSTANTLY, TR_FIXED, Trigger, frame_index, oscillator_trigger_rate, panner,
    scaler, texture_matrix,
};
use crate::{Catalog, Fade, MAX_MATERIAL_DEPTH, MAX_TEXTURE_FRAMES, full_path, package_name};

#[cfg(test)]
#[path = "material_l2_tests.rs"]
mod l2_tests;

pub const ALPHA_REF_MASKED: u8 = 127;

const OB_NORMAL: u8 = 0;
const OB_MASKED: u8 = 1;
const OB_MODULATE: u8 = 2;
const OB_TRANSLUCENT: u8 = 3;
const OB_INVISIBLE: u8 = 4;
const OB_BRIGHTEN: u8 = 5;
const OB_DARKEN: u8 = 6;

const FB_OVERWRITE: u8 = 0;
const FB_MODULATE: u8 = 1;
const FB_ALPHA_BLEND: u8 = 2;
const FB_ALPHA_MODULATE: u8 = 3;
const FB_TRANSLUCENT: u8 = 4;
const FB_DARKEN: u8 = 5;
const FB_BRIGHTEN: u8 = 6;
const FB_INVISIBLE: u8 = 7;

const TCS_STREAM_MAX: u8 = 7;
const TCS_WORLD_ENV_MAP: u8 = 10;
const TCS_CAMERA_ENV_MAP: u8 = 11;
const EM_WORLD_SPACE: u8 = 0;
const EM_CAMERA_SPACE: u8 = 1;

const CO_MATERIAL1: u8 = 0;
const CO_MATERIAL2: u8 = 1;
const CO_MULTIPLY: u8 = 2;
const CO_ADD: u8 = 3;
const CO_SUBTRACT: u8 = 4;
const CO_ALPHA_BLEND_WITH_MASK: u8 = 5;
const CO_ADD_WITH_MASK_MODULATION: u8 = 6;

const AO_USE_MASK: u8 = 0;
const AO_MULTIPLY: u8 = 1;
const AO_ADD: u8 = 2;
const AO_ALPHA_FROM_MATERIAL1: u8 = 3;
const AO_ALPHA_FROM_MATERIAL2: u8 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendFactor {
    Zero,
    One,
    SrcColor,
    InvSrcColor,
    SrcAlpha,
    InvSrcAlpha,
    DstColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FogMode {
    Zone,
    Black,
    Gray127,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordSource {
    Stream(u8),
    CameraEnvMap,
    WorldEnvMap,
    Projection { matrix: [[f32; 4]; 4], divide_z: bool },
    ProjectorNormal { matrix: [[f32; 4]; 4] },
    ProjectorStream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorOp {
    SelectTexture,
    SelectCurrent,
    Modulate,
    Modulate2X,
    Modulate4X,
    Add,
    Subtract,
    BlendTextureAlpha,
    BlendDiffuseAlpha,
    BlendCurrentAlpha,
    BlendFactorAlpha,
    ModulateAlphaAddColor,
    MultiplyAdd,
    BlendCurrentFactorByTextureAlpha,
    BlendTextureFactorByDiffuseAlpha,
    ModulateCurrentByTextureAlpha,
    AddSigned,
    AddSigned2X,
    AddSmooth,
    BlendTextureAlphaPremultiplied,
    ModulateColorAddAlpha,
    ModulateInvAlphaAddColor,
    ModulateInvColorAddAlpha,
    Disable,
    PreModulate,
    MultiplyAddCurrent,
    DotProduct3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaOp {
    Texture,
    Current,
    Diffuse,
    TFactor,
    Modulate,
    Add,
    BlendDiffuseAlpha,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextureRef {
    pub path: String,
    pub frames: Vec<String>,
    pub rate: f32,
    pub total_frames: u32,
    pub size: [f32; 2],
    pub clamp: [bool; 2],
}

impl TextureRef {
    pub fn frame(&self, time: f32) -> u32 {
        frame_index(time, self.rate, self.total_frames)
    }

    pub fn frame_path(&self, time: f32) -> &str {
        let frame = self.frame(time) as usize;
        if frame == 0 || self.frames.is_empty() { &self.path } else { &self.frames[(frame - 1) % self.frames.len()] }
    }

    pub fn animated(&self) -> bool {
        self.total_frames > 0 && self.rate != 0.0 && !self.frames.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cubemap {
    pub faces: [String; 6],
    pub size: [f32; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub enum StageSource {
    Texture(TextureRef),
    Cubemap(Cubemap),
    Factor,
    Diffuse,
    ProjectorFacing,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UvModifier {
    Panner { direction: [i32; 3], rate: f32 },
    PannerTriggered { direction: [i32; 3], rate: f32, trigger: Trigger },
    Oscillator(Oscillator, [f32; 2]),
    OscillatorTriggered(Oscillator, [f32; 2], Trigger),
    Matrix(Mat3x2),
    Rotator(Rotator, [f32; 2]),
    Scaler { scale: [f32; 2], offset: [f32; 2], size: [f32; 2] },
}

impl UvModifier {
    pub fn matrix(self, time: f32) -> Mat3x2 {
        match self {
            Self::Panner { direction, rate } => panner(direction, rate, time),
            Self::PannerTriggered { direction, rate, trigger } => panner(direction, rate, trigger.time(rate, time)),
            Self::Oscillator(oscillator, size) => oscillator.matrix(size, time),
            Self::OscillatorTriggered(oscillator, size, trigger) => {
                oscillator.matrix(size, trigger.time(oscillator_trigger_rate(oscillator.rate), time))
            }
            Self::Matrix(matrix) => matrix,
            Self::Rotator(rotator, size) => rotator.matrix(size, time),
            Self::Scaler { scale, offset, size } => scaler(scale, offset, size),
        }
    }

    pub fn animated(self) -> bool {
        match self {
            Self::Panner { rate, .. } | Self::PannerTriggered { rate, .. } => rate != 0.0,
            Self::Oscillator(oscillator, _) | Self::OscillatorTriggered(oscillator, ..) => {
                oscillator.rate != [0.0, 0.0]
            }
            Self::Matrix(_) => false,
            Self::Rotator(rotator, _) => rotator.kind != TR_FIXED,
            Self::Scaler { .. } => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stage {
    pub source: StageSource,
    pub uv: Vec<UvModifier>,
    pub coord_source: CoordSource,
    pub color_op: ColorOp,
    pub alpha_op: AlphaOp,
    pub swapped: bool,
}

impl Stage {
    fn new(source: StageSource, info: &Info, color_op: ColorOp, alpha_op: AlphaOp, swapped: bool) -> Self {
        Self { source, uv: info.uv.clone(), coord_source: info.coord(), color_op, alpha_op, swapped }
    }

    fn factor(color_op: ColorOp, alpha_op: AlphaOp) -> Self {
        Self {
            source: StageSource::Factor,
            uv: Vec::new(),
            coord_source: CoordSource::Stream(0),
            color_op,
            alpha_op,
            swapped: false,
        }
    }

    pub fn matrix(&self, time: f32) -> Mat3x2 {
        self.uv.iter().fold(Mat3x2::IDENTITY, |matrix, modifier| matrix.then(modifier.matrix(time)))
    }

    pub fn frame(&self, time: f32) -> u32 {
        self.texture().map_or(0, |texture| texture.frame(time))
    }

    pub fn texture(&self) -> Option<&TextureRef> {
        match &self.source {
            StageSource::Texture(texture) => Some(texture),
            _ => None,
        }
    }

    pub fn animated(&self) -> bool {
        self.uv.iter().any(|modifier| modifier.animated()) || self.texture().is_some_and(TextureRef::animated)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pass {
    pub stages: Range<usize>,
    pub src: BlendFactor,
    pub dst: BlendFactor,
    pub alpha_blend: bool,
    pub z_write: bool,
    pub z_test: bool,
    pub alpha_test: Option<u8>,
    pub two_sided: bool,
    pub fog: FogMode,
    pub tfactor: Option<[u8; 4]>,
    pub fade: Option<Fade>,
    pub fade_alpha_replicate: bool,
    pub lit: bool,
    pub light_after: usize,
}

impl Pass {
    fn new(first_stage: usize) -> Self {
        Self {
            stages: first_stage..first_stage,
            src: BlendFactor::One,
            dst: BlendFactor::Zero,
            alpha_blend: false,
            z_write: true,
            z_test: true,
            alpha_test: None,
            two_sided: false,
            fog: FogMode::Zone,
            tfactor: None,
            fade: None,
            fade_alpha_replicate: false,
            lit: true,
            light_after: 0,
        }
    }

    pub fn tfactor_at(&self, time: f32) -> [u8; 4] {
        match (self.fade, self.tfactor) {
            (Some(fade), _) => {
                let color = fade.bytes(time);
                if self.fade_alpha_replicate { [color[3]; 4] } else { color }
            }
            (None, Some(color)) => color,
            (None, None) => [255; 4],
        }
    }

    pub fn animated(&self) -> bool {
        self.fade.is_some()
    }

    fn set_constant(&mut self, color: Option<[u8; 4]>, fade: Option<Fade>, replicate: bool) -> Option<()> {
        if self.tfactor.is_some() || self.fade.is_some() {
            return None;
        }
        self.tfactor = color.map(|color| if replicate { [color[3]; 4] } else { color });
        self.fade = fade;
        self.fade_alpha_replicate = replicate;
        Some(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub stages: Vec<Stage>,
    pub passes: Vec<Pass>,
    pub sorting_level: u8,
    pub transparent: bool,
    pub root_class: String,
    /// Object path of the `ColorModifier` whose `Color` became the TFACTOR, when the chain has one.
    pub color_modifier: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DynamicProjectorParameters {
    pub material_blend: u8,
    pub frame_blend: u8,
    pub projected: bool,
    pub on_alpha: bool,
    pub on_backfaces: bool,
    pub projection: [[f32; 4]; 4],
    pub normal: [[f32; 4]; 4],
    pub gradient: Option<(String, [[f32; 4]; 4])>,
}

impl Material {
    pub fn requires_sorting(&self) -> bool {
        self.sorting_level > 0
    }

    pub fn pass_stages(&self, pass: &Pass) -> &[Stage] {
        &self.stages[pass.stages.clone()]
    }

    pub fn first_texture(&self) -> Option<&TextureRef> {
        self.stages.iter().find_map(Stage::texture)
    }

    pub fn animated(&self) -> bool {
        self.stages.iter().any(Stage::animated) || self.passes.iter().any(Pass::animated)
    }

    pub fn texture_paths(&self) -> Vec<&str> {
        let mut paths = Vec::new();
        for stage in &self.stages {
            match &stage.source {
                StageSource::Texture(texture) => {
                    paths.push(texture.path.as_str());
                    if texture.animated() {
                        paths.extend(texture.frames.iter().map(String::as_str));
                    }
                }
                StageSource::Cubemap(cubemap) => {
                    paths.extend(cubemap.faces.iter().map(String::as_str).filter(|face| !face.is_empty()))
                }
                StageSource::Factor | StageSource::Diffuse | StageSource::ProjectorFacing => {}
            }
        }
        paths
    }

    pub fn summary(&self) -> String {
        let Some(pass) = self.passes.first() else { return "invisible".to_owned() };
        let ops: Vec<String> =
            self.pass_stages(pass).iter().map(|stage| format!("{:?}/{:?}", stage.color_op, stage.alpha_op)).collect();
        format!(
            "{:?}/{:?} alpha_test={:?} z_write={} two_sided={} fog={:?} passes={} stages=[{}] animated={}",
            pass.src,
            pass.dst,
            pass.alpha_test,
            u8::from(pass.z_write),
            u8::from(pass.two_sided),
            pass.fog,
            self.passes.len(),
            ops.join(" "),
            u8::from(self.animated())
        )
    }
}

#[derive(Debug, Clone, Default)]
struct Info {
    uv: Vec<UvModifier>,
    coord_source: Option<CoordSource>,
    modify_fb: bool,
    fb: u8,
    z_write: bool,
    z_test: bool,
    alpha_test: bool,
    two_sided: bool,
    alpha_ref: u8,
    modify_color: bool,
    tfactor: [u8; 4],
    alpha_blend: bool,
    opacity: Option<String>,
    color_modifier: Option<String>,
    l2_modifier: Option<Modifier>,
}

impl Info {
    fn new() -> Self {
        Self { z_write: true, z_test: true, ..Self::default() }
    }

    fn coord(&self) -> CoordSource {
        self.coord_source.unwrap_or(CoordSource::Stream(0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShaderFlags {
    pub two_sided: bool,
    pub treat_as_two_sided: bool,
    pub z_write: bool,
    pub alpha_test: bool,
    pub alpha_ref: u8,
}

fn output_blending(pass: &mut Pass, output_blending: u8, opacity: bool, shader: ShaderFlags) -> Option<()> {
    let (src, dst) = match output_blending {
        OB_NORMAL if opacity => (BlendFactor::SrcAlpha, BlendFactor::InvSrcAlpha),
        OB_NORMAL | OB_MASKED => (BlendFactor::One, BlendFactor::Zero),
        OB_MODULATE => (BlendFactor::DstColor, BlendFactor::SrcColor),
        OB_TRANSLUCENT => (BlendFactor::One, BlendFactor::One),
        OB_INVISIBLE => return None,
        OB_BRIGHTEN => (BlendFactor::One, BlendFactor::InvSrcColor),
        OB_DARKEN => (BlendFactor::Zero, BlendFactor::InvSrcColor),
        _ => return None,
    };
    pass.src = src;
    pass.dst = dst;
    pass.alpha_blend = !(output_blending == OB_MASKED || (output_blending == OB_NORMAL && !opacity));
    pass.alpha_test = match output_blending {
        OB_NORMAL if opacity && shader.z_write => shader.alpha_test.then_some(shader.alpha_ref),
        OB_NORMAL if opacity => Some(0),
        OB_MASKED => Some(ALPHA_REF_MASKED),
        _ => None,
    };
    pass.z_write = match output_blending {
        OB_NORMAL if opacity => shader.z_write,
        OB_NORMAL | OB_MASKED => true,
        _ => false,
    };
    pass.fog = match output_blending {
        OB_MODULATE => FogMode::Gray127,
        OB_TRANSLUCENT | OB_BRIGHTEN | OB_DARKEN => FogMode::Black,
        _ => FogMode::Zone,
    };
    Some(())
}

fn apply_final_blend(pass: &mut Pass, info: &Info) -> Option<()> {
    let (src, dst, fog) = match info.fb {
        FB_OVERWRITE => (BlendFactor::One, BlendFactor::Zero, FogMode::Zone),
        FB_MODULATE => (BlendFactor::DstColor, BlendFactor::SrcColor, FogMode::Gray127),
        FB_ALPHA_BLEND => (BlendFactor::SrcAlpha, BlendFactor::InvSrcAlpha, FogMode::Zone),
        FB_ALPHA_MODULATE => (BlendFactor::One, BlendFactor::InvSrcAlpha, FogMode::Black),
        FB_TRANSLUCENT => (BlendFactor::One, BlendFactor::InvSrcColor, FogMode::Black),
        FB_DARKEN => (BlendFactor::Zero, BlendFactor::InvSrcColor, FogMode::Black),
        FB_BRIGHTEN => (BlendFactor::SrcAlpha, BlendFactor::One, FogMode::Black),
        FB_INVISIBLE => return None,
        _ => return None,
    };
    pass.src = src;
    pass.dst = dst;
    pass.alpha_blend = info.fb != FB_OVERWRITE;
    pass.fog = fog;
    pass.z_write = info.z_write;
    pass.z_test = info.z_test;
    pass.two_sided = info.two_sided;
    pass.alpha_test = info.alpha_test.then_some(info.alpha_ref);
    Some(())
}

fn texture_flags(info: &mut Info, masked: bool, alpha_texture: bool, two_sided: bool) {
    if info.modify_fb {
        return;
    }
    if masked || alpha_texture {
        info.modify_fb = true;
        info.fb = if alpha_texture || (info.modify_color && info.alpha_blend) { FB_ALPHA_BLEND } else { FB_OVERWRITE };
        info.alpha_ref = if masked { ALPHA_REF_MASKED } else { 0 };
        info.z_write = true;
        info.z_test = true;
        info.alpha_test = true;
        if !info.modify_color {
            info.two_sided = two_sided;
        }
    } else if two_sided {
        info.modify_fb = true;
        if !info.modify_color {
            info.two_sided = true;
        }
    } else if !info.modify_color && info.l2_modifier.is_none() {
        info.two_sided = false;
    }
}

fn color_modifier(pass: &mut Pass, stages: &mut Vec<Stage>, info: &Info) {
    if info.alpha_blend && pass.src == BlendFactor::One && pass.dst == BlendFactor::Zero {
        pass.src = BlendFactor::SrcAlpha;
        pass.dst = BlendFactor::InvSrcAlpha;
    }
    stages.push(Stage::factor(ColorOp::Modulate, AlphaOp::Modulate));
    pass.tfactor = Some(info.tfactor);
    pass.fade = None;
    pass.alpha_blend |= info.alpha_blend;
    pass.two_sided = info.two_sided;
}

fn l2_color_modifier(pass: &mut Pass, stages: &mut Vec<Stage>, info: &Info, path: MaterialPath) -> Option<()> {
    let Some(modifier) = info.l2_modifier else { return Some(()) };
    if stages.len().checked_sub(pass.stages.start)? >= 8 {
        return None;
    }
    let packed = |[r, g, b, a]: [u8; 4]| u32::from_le_bytes([b, g, r, a]);
    let blend = |factor| match factor {
        BlendFactor::Zero => 1,
        BlendFactor::One => 2,
        BlendFactor::SrcColor => 3,
        BlendFactor::InvSrcColor => 4,
        BlendFactor::SrcAlpha => 5,
        BlendFactor::InvSrcAlpha => 6,
        BlendFactor::DstColor => 9,
    };
    let mut state = PassState {
        factor: packed(pass.tfactor.unwrap_or([0; 4])),
        src: blend(pass.src),
        dst: blend(pass.dst),
        alpha_blend: u32::from(pass.alpha_blend),
        alpha_ref: pass.alpha_test.unwrap_or(0),
    };
    let mut flags =
        PassFlags { alpha_test: u32::from(pass.alpha_test.is_some()), two_sided: u32::from(pass.two_sided) };
    let mut stage =
        StageState { texture: 0, color_op: 2, alpha_op: 2, color_args: [1; 2], color_arg0: 1, alpha_args: [1; 2] };
    l2_color_stage::apply_pass(
        modifier,
        ModifierFlags { alpha_blend: u32::from(info.alpha_blend), two_sided: u32::from(info.two_sided) },
        &mut state,
        &mut flags,
        &mut stage,
        path,
    );
    let color_op = match stage.color_op {
        1 => ColorOp::Disable,
        2 => ColorOp::SelectCurrent,
        3 => ColorOp::SelectTexture,
        4 => ColorOp::Modulate,
        5 => ColorOp::Modulate2X,
        6 => ColorOp::Modulate4X,
        7 => ColorOp::Add,
        8 => ColorOp::AddSigned,
        9 => ColorOp::AddSigned2X,
        10 => ColorOp::Subtract,
        11 => ColorOp::AddSmooth,
        12 => ColorOp::BlendDiffuseAlpha,
        14 => ColorOp::BlendFactorAlpha,
        16 => ColorOp::BlendCurrentAlpha,
        17 => ColorOp::PreModulate,
        18 => ColorOp::ModulateAlphaAddColor,
        19 => ColorOp::ModulateColorAddAlpha,
        20 => ColorOp::ModulateInvAlphaAddColor,
        21 => ColorOp::ModulateInvColorAddAlpha,
        24 => ColorOp::DotProduct3,
        25 => ColorOp::MultiplyAddCurrent,
        _ => return None,
    };
    let alpha_op = match stage.alpha_op {
        2 => AlphaOp::Current,
        3 => AlphaOp::TFactor,
        4 => AlphaOp::Modulate,
        _ => return None,
    };
    let mut translated = Stage::factor(color_op, alpha_op);
    translated.swapped = true;
    stages.push(translated);
    if path == MaterialPath::Simple
        || modifier.color_op != 1
        || (modifier.alpha_op != 1 && matches!((state.src, state.dst), (2, 4) | (5, 6)))
    {
        let [b, g, r, a] = state.factor.to_le_bytes();
        pass.tfactor = Some([r, g, b, a]);
        pass.fade = None;
        pass.fade_alpha_replicate = false;
    }
    if state.src == 5 && state.dst == 6 {
        pass.src = BlendFactor::SrcAlpha;
        pass.dst = BlendFactor::InvSrcAlpha;
    }
    pass.alpha_blend = state.alpha_blend != 0;
    pass.alpha_test = (flags.alpha_test != 0).then_some(state.alpha_ref);
    pass.two_sided = flags.two_sided != 0;
    Some(())
}

struct Leaf {
    path: String,
    node: Node,
    info: Info,
}

impl Leaf {
    fn is_bitmap(&self) -> bool {
        self.node.is_bitmap()
    }

    fn is_constant(&self) -> bool {
        matches!(self.node.class.as_str(), "constantcolor" | "fadecolor")
    }

    fn is_vertex_color(&self) -> bool {
        self.node.class == "vertexcolor"
    }
}

impl Catalog {
    pub fn dynamic_projector_material(
        &mut self,
        path: &str,
        base: Option<&str>,
        params: &DynamicProjectorParameters,
    ) -> Option<Material> {
        let projected = self.check_material(path, Info::new(), 0)?;
        if projected.path != path || !projected.is_bitmap() {
            return None;
        }
        let mut pass = Pass::new(0);
        let mut stages = Vec::new();
        let mut diffuse = base.map(str::to_owned);
        let mut opacity = None;
        if let Some(path) = base {
            let node = self.node(path)?;
            match node.class.as_str() {
                "shader" => {
                    diffuse = node.diffuse;
                    opacity = node.opacity;
                    pass.two_sided = node.two_sided;
                }
                "finalblend" => pass.two_sided = node.two_sided,
                "texture" | "wettexture" if node.masked || node.alpha_texture => opacity = Some(path.to_owned()),
                _ => {}
            }
        }
        if params.material_blend != 0
            && let Some(path) = diffuse
        {
            let leaf = self.check_material(&path, Info::new(), 0)?;
            self.handle_combined(&leaf, &mut pass, &mut stages, 0)?;
        }
        stages.push(Stage {
            source: self.bitmap_source(&projected)?,
            uv: Vec::new(),
            coord_source: CoordSource::Projection { matrix: params.projection, divide_z: params.projected },
            color_op: if params.material_blend == 1 { ColorOp::Modulate2X } else { ColorOp::SelectTexture },
            alpha_op: AlphaOp::Texture,
            swapped: false,
        });
        let attenuation = |source, coord_source, uv, alpha_replicate| Stage {
            source,
            coord_source,
            uv,
            swapped: false,
            color_op: match params.frame_blend {
                2 => ColorOp::SelectCurrent,
                3 if alpha_replicate => ColorOp::ModulateCurrentByTextureAlpha,
                3 => ColorOp::Modulate,
                _ => ColorOp::BlendCurrentFactorByTextureAlpha,
            },
            alpha_op: if params.frame_blend == 2 { AlphaOp::Modulate } else { AlphaOp::Current },
        };
        let mut attenuated = false;
        if let Some((path, matrix)) = &params.gradient {
            let leaf = self.check_material(path, Info::new(), 0)?;
            if leaf.path != *path {
                return None;
            }
            stages.push(attenuation(
                self.bitmap_source(&leaf)?,
                CoordSource::Projection { matrix: *matrix, divide_z: false },
                Vec::new(),
                true,
            ));
            attenuated = true;
        }
        if params.on_alpha
            && let Some(path) = opacity
        {
            let leaf = self.check_material(&path, Info::new(), 0)?;
            stages.push(attenuation(self.bitmap_source(&leaf)?, leaf.info.coord(), leaf.info.uv, true));
            attenuated = true;
        }
        if !params.on_backfaces {
            stages.push(attenuation(
                StageSource::ProjectorFacing,
                CoordSource::ProjectorNormal { matrix: params.normal },
                Vec::new(),
                false,
            ));
            attenuated = true;
        }
        pass.stages.end = stages.len();
        pass.light_after = stages.len();
        pass.lit = false;
        pass.z_write = false;
        pass.alpha_blend = true;
        (pass.src, pass.dst, pass.fog) = match params.frame_blend {
            2 => (BlendFactor::SrcAlpha, BlendFactor::InvSrcAlpha, FogMode::Zone),
            3 => (BlendFactor::One, BlendFactor::One, FogMode::Black),
            _ => (BlendFactor::DstColor, BlendFactor::SrcColor, FogMode::Gray127),
        };
        if !matches!(params.frame_blend, 2 | 3) && attenuated {
            pass.tfactor = Some([127, 127, 127, 0]);
        }
        Some(Material {
            stages,
            passes: vec![pass],
            sorting_level: 0,
            transparent: true,
            root_class: "dynamicprojectormaterial".into(),
            color_modifier: None,
        })
    }

    pub fn bsp_projector_material(
        &mut self,
        path: &str,
        base: Option<&str>,
        material_blend: u8,
        frame_blend: u8,
        projection: [[f32; 4]; 4],
    ) -> Option<Material> {
        let projected = self.check_material(path, Info::new(), 0)?;
        if projected.path != path || !projected.is_bitmap() {
            return None;
        }
        let mut pass = Pass::new(0);
        let mut stages = Vec::new();
        let base_node = base.and_then(|path| self.node(path));
        let mut opacity = None;
        let mut diffuse = None;
        if let (Some(path), Some(node)) = (base, base_node) {
            let texture_alpha =
                matches!(node.class.as_str(), "texture" | "wettexture") && (node.masked || node.alpha_texture);
            let shader = node.class == "shader";
            let final_alpha = node.class == "finalblend" && node.frame_buffer_blending == FB_ALPHA_BLEND;
            if material_blend != 0 || texture_alpha || (shader && node.opacity.is_some()) || final_alpha {
                diffuse = Some(path.to_owned());
                if texture_alpha {
                    opacity = Some(path.to_owned());
                } else if shader {
                    diffuse = node.diffuse;
                    opacity = node.opacity;
                }
            }
            if material_blend == 0 && (shader || final_alpha) {
                pass.two_sided = node.two_sided;
            }
        }
        let base_info = Info { coord_source: Some(CoordSource::Stream(1)), ..Info::new() };
        if material_blend != 0
            && let Some(path) = diffuse
        {
            let leaf = self.check_material(&path, base_info.clone(), 0)?;
            self.handle_combined(&leaf, &mut pass, &mut stages, 0)?;
        }
        let source = self.bitmap_source(&projected)?;
        let color_op = match material_blend {
            1 => ColorOp::Modulate2X,
            2 => ColorOp::BlendTextureAlpha,
            _ => match frame_blend {
                2 => ColorOp::SelectTexture,
                3 => ColorOp::Modulate,
                _ => ColorOp::BlendTextureFactorByDiffuseAlpha,
            },
        };
        let alpha_op = if material_blend == 0 && frame_blend == 2 { AlphaOp::Modulate } else { AlphaOp::Texture };
        stages.push(Stage {
            source,
            coord_source: CoordSource::Projection { matrix: projection, divide_z: true },
            uv: Vec::new(),
            color_op,
            alpha_op,
            swapped: false,
        });
        if let Some(path) = opacity {
            let leaf = self.check_material(&path, base_info, 0)?;
            let source = self.bitmap_source(&leaf)?;
            stages.push(Stage::new(
                source,
                &leaf.info,
                match frame_blend {
                    2 => ColorOp::SelectCurrent,
                    3 => ColorOp::ModulateCurrentByTextureAlpha,
                    _ => ColorOp::BlendCurrentFactorByTextureAlpha,
                },
                if frame_blend == 2 { AlphaOp::Modulate } else { AlphaOp::Current },
                false,
            ));
        }
        if material_blend != 0 {
            stages.push(Stage {
                source: StageSource::Diffuse,
                coord_source: CoordSource::Stream(0),
                uv: Vec::new(),
                swapped: false,
                color_op: match frame_blend {
                    2 => ColorOp::SelectCurrent,
                    3 => ColorOp::ModulateCurrentByTextureAlpha,
                    _ => ColorOp::BlendCurrentFactorByTextureAlpha,
                },
                alpha_op: if frame_blend == 2 { AlphaOp::Modulate } else { AlphaOp::Current },
            });
        }
        pass.stages.end = stages.len();
        pass.light_after = stages.len();
        pass.lit = false;
        pass.z_write = false;
        pass.alpha_blend = true;
        (pass.src, pass.dst, pass.fog) = match frame_blend {
            2 => (BlendFactor::SrcAlpha, BlendFactor::InvSrcAlpha, FogMode::Zone),
            3 => (BlendFactor::One, BlendFactor::One, FogMode::Black),
            _ => (BlendFactor::DstColor, BlendFactor::SrcColor, FogMode::Gray127),
        };
        if !matches!(frame_blend, 2 | 3) {
            pass.tfactor = Some([127, 127, 127, 0]);
        }
        Some(Material {
            stages,
            passes: vec![pass],
            sorting_level: 0,
            transparent: true,
            root_class: "projectormaterial".into(),
            color_modifier: None,
        })
    }

    pub fn material(&mut self, path: &str) -> Option<Material> {
        self.material_with_info(path, Info::new())
    }

    pub fn l2_modified_material(&mut self, path: &str, modifier: Modifier, flags: ModifierFlags) -> Option<Material> {
        self.material_with_info(
            path,
            Info {
                l2_modifier: Some(modifier),
                alpha_blend: flags.alpha_blend != 0,
                two_sided: flags.two_sided != 0,
                ..Info::new()
            },
        )
    }

    fn material_with_info(&mut self, path: &str, info: Info) -> Option<Material> {
        let sorting_level = self.sorting_level(path, 0)?;
        let transparent = self.is_transparent(path, 0)?;
        let root_class = self.node(path)?.class;
        let leaf = self.check_material(path, info, 0)?;
        let mut info = leaf.info.clone();
        let mut stages = Vec::new();
        let mut passes = Vec::new();
        match leaf.node.class.as_str() {
            "shader" => self.shader_material(&leaf, &info, &mut stages, &mut passes)?,
            _ => self.simple_material(&leaf, &mut info, &mut stages, &mut passes)?,
        }
        Some(Material {
            stages,
            passes,
            sorting_level,
            transparent,
            root_class,
            color_modifier: leaf.info.color_modifier,
        })
    }

    pub fn enchanted_material(&mut self, path: &str, colors: Option<[[u8; 3]; 2]>) -> Option<Material> {
        let fade_path = self.enchant_fade_path(path);
        self.enchant_color_override = fade_path.zip(colors);
        let material = self.material(path);
        self.enchant_color_override = None;
        material
    }

    fn enchant_fade_path(&mut self, path: &str) -> Option<String> {
        let shader = self.node(path)?;
        if shader.class != "shader" {
            return None;
        }
        let combiner = self.node(shader.diffuse.as_deref()?)?;
        if combiner.class != "combiner" {
            return None;
        }
        let fade_path = combiner.material1?;
        (self.node(&fade_path)?.class == "fadecolor").then_some(fade_path)
    }

    fn is_transparent(&mut self, path: &str, depth: usize) -> Option<bool> {
        if depth > MAX_MATERIAL_DEPTH {
            return None;
        }
        let node = self.node(path)?;
        Some(match node.class.as_str() {
            "texture" | "wettexture" => node.masked || node.alpha_texture,
            "finalblend" | "shader" => self.sorting_level(path, depth)? > 0,
            class if is_modifier(class) => self.is_transparent(node.material.as_deref()?, depth + 1)?,
            _ => false,
        })
    }

    fn sorting_level(&mut self, path: &str, depth: usize) -> Option<u8> {
        if depth > MAX_MATERIAL_DEPTH {
            return None;
        }
        let node = self.node(path)?;
        Some(match node.class.as_str() {
            "finalblend" => match node.frame_buffer_blending {
                FB_MODULATE..=FB_BRIGHTEN => {
                    if node.z_write {
                        1
                    } else {
                        2
                    }
                }
                _ => 0,
            },
            "shader" => {
                let sorts = (node.output_blending == OB_NORMAL && node.opacity.is_some())
                    || matches!(node.output_blending, OB_MODULATE | OB_TRANSLUCENT | OB_BRIGHTEN | OB_DARKEN);
                if sorts { 2 } else { 0 }
            }
            "texture" | "wettexture" => u8::from(node.alpha_texture),
            class if is_modifier(class) => self.sorting_level(node.material.as_deref()?, depth + 1)?,
            _ => 0,
        })
    }

    fn check_material(&mut self, path: &str, mut info: Info, depth: usize) -> Option<Leaf> {
        let mut path = path.to_owned();
        for _ in 0..=MAX_MATERIAL_DEPTH {
            let node = self.node(&path)?;
            let class = node.class.clone();
            if !is_modifier(&class) {
                return Some(Leaf { path, node, info });
            }
            let Some(child) = node.material.clone() else {
                return Some(Leaf { path, node: Node::null(), info });
            };
            match class.as_str() {
                "finalblend" => {
                    info.modify_fb = true;
                    info.fb = node.frame_buffer_blending;
                    info.z_write = node.z_write;
                    info.z_test = node.z_test;
                    info.alpha_test = node.alpha_test;
                    info.two_sided = node.two_sided;
                    info.alpha_ref = node.alpha_ref;
                }
                "colormodifier" => {
                    info.modify_color = true;
                    info.color_modifier = Some(path.clone());
                    info.tfactor = node.color.unwrap_or([255; 4]);
                    info.alpha_blend |= node.alpha_blend;
                    info.two_sided |= node.render_two_sided;
                }
                "opacitymodifier" => info.opacity = node.opacity.clone(),
                "l2colormodifier" => {
                    let [r, g, b, a] = node.color.unwrap_or([255; 4]);
                    info.l2_modifier = Some(Modifier {
                        color: u32::from_le_bytes([b, g, r, a]),
                        alpha_op: u32::from(node.l2_alpha_op),
                        color_op: u32::from(node.l2_color_op),
                    });
                    info.alpha_blend |= node.alpha_blend;
                    info.two_sided |= node.render_two_sided;
                }
                _ => {
                    let size = self.material_size(&child, depth + 1);
                    if let Some(modifier) = node.uv_modifier(&class, size) {
                        info.uv.push(modifier);
                    }
                    if info.coord_source.is_none() {
                        info.coord_source = node.coord_source(&class);
                    }
                }
            }
            path = child;
        }
        None
    }

    fn material_size(&mut self, path: &str, depth: usize) -> [f32; 2] {
        if depth > MAX_MATERIAL_DEPTH {
            return [0.0; 2];
        }
        let Some(node) = self.node(path) else { return [0.0; 2] };
        let child = |catalog: &mut Self, path: Option<&str>| {
            path.map_or([0.0; 2], |path| catalog.material_size(path, depth + 1))
        };
        match node.class.as_str() {
            "texture" | "cubemap" | "wettexture" => node.size,
            "combiner" => {
                let first = child(self, node.material1.as_deref());
                let second = child(self, node.material2.as_deref());
                [first[0].max(second[0]), first[1].max(second[1])]
            }
            "shader" => child(self, node.diffuse.as_deref().or(node.self_illumination.as_deref())),
            class if is_modifier(class) => child(self, node.material.as_deref()),
            _ => [0.0; 2],
        }
    }

    fn simple_material(
        &mut self,
        leaf: &Leaf,
        info: &mut Info,
        stages: &mut Vec<Stage>,
        passes: &mut Vec<Pass>,
    ) -> Option<()> {
        let mut pass = Pass::new(stages.len());
        if matches!(leaf.node.class.as_str(), "texture" | "wettexture") {
            texture_flags(info, leaf.node.masked, leaf.node.alpha_texture, leaf.node.two_sided);
        }
        self.handle_combined(leaf, &mut pass, stages, 0)?;
        pass.light_after = stages.len();
        if info.modify_fb {
            if info.fb == FB_INVISIBLE {
                stages.clear();
                return Some(());
            }
            apply_final_blend(&mut pass, info)?;
        }
        if info.modify_color {
            color_modifier(&mut pass, stages, info);
        }
        l2_color_modifier(&mut pass, stages, info, MaterialPath::Simple)?;
        pass.stages.end = stages.len();
        passes.push(pass);
        Some(())
    }

    fn handle_combined(&mut self, leaf: &Leaf, pass: &mut Pass, stages: &mut Vec<Stage>, depth: usize) -> Option<()> {
        if depth > MAX_MATERIAL_DEPTH {
            return None;
        }
        match leaf.node.class.as_str() {
            "texture" | "cubemap" | "wettexture" => {
                let source = self.bitmap_source(leaf)?;
                stages.push(Stage::new(source, &leaf.info, ColorOp::SelectTexture, AlphaOp::Texture, false));
            }
            "constantcolor" | "fadecolor" => {
                pass.set_constant(leaf.node.color, leaf.node.fade, false)?;
                stages.push(Stage::factor(ColorOp::SelectTexture, AlphaOp::TFactor));
            }
            "vertexcolor" => stages.push(Stage {
                source: StageSource::Diffuse,
                ..Stage::factor(ColorOp::SelectTexture, AlphaOp::Diffuse)
            }),
            "combiner" => self.combiner(leaf, pass, stages, depth)?,
            _ => {}
        }
        Some(())
    }

    fn combiner(&mut self, leaf: &Leaf, pass: &mut Pass, stages: &mut Vec<Stage>, depth: usize) -> Option<()> {
        let node = &leaf.node;
        let material1 = node.material1.clone()?;
        let first = self.check_material(&material1, leaf.info.clone(), depth + 1);
        let second = node.material2.as_deref().and_then(|path| self.check_material(path, leaf.info.clone(), depth + 1));
        let (simple, complex, swapped) = match (first, second) {
            (Some(first), Some(second)) if second.is_bitmap() => (second, first, false),
            (Some(first), Some(second)) if first.is_bitmap() => (first, second, true),
            _ => return None,
        };
        self.handle_combined(&complex, pass, stages, depth + 1)?;
        let (mask_alpha, blend_op) = match node.mask.as_deref() {
            None => (AlphaOp::Current, ColorOp::BlendCurrentAlpha),
            Some(mask) if same_object(mask, &simple.path) => (AlphaOp::Texture, ColorOp::BlendTextureAlpha),
            Some(mask) if same_object(mask, &complex.path) => (AlphaOp::Current, ColorOp::BlendCurrentAlpha),
            Some(mask) => {
                let mask_leaf = self.check_material(mask, leaf.info.clone(), depth + 1)?;
                if mask_leaf.is_vertex_color() {
                    (AlphaOp::Diffuse, ColorOp::BlendDiffuseAlpha)
                } else if mask_leaf.is_constant() {
                    pass.set_constant(mask_leaf.node.color, mask_leaf.node.fade, false)?;
                    (AlphaOp::TFactor, ColorOp::BlendFactorAlpha)
                } else if mask_leaf.is_bitmap() {
                    let source = self.bitmap_source(&mask_leaf)?;
                    stages.push(Stage::new(source, &mask_leaf.info, ColorOp::SelectCurrent, AlphaOp::Texture, false));
                    (AlphaOp::Current, ColorOp::BlendCurrentAlpha)
                } else {
                    return None;
                }
            }
        };
        let (color_op, stage_swapped) = match node.combine_operation {
            CO_MATERIAL1 => (if swapped { ColorOp::SelectTexture } else { ColorOp::SelectCurrent }, false),
            CO_MATERIAL2 => (if swapped { ColorOp::SelectCurrent } else { ColorOp::SelectTexture }, false),
            CO_MULTIPLY if node.modulate4x => (ColorOp::Modulate4X, false),
            CO_MULTIPLY if node.modulate2x => (ColorOp::Modulate2X, false),
            CO_MULTIPLY => (ColorOp::Modulate, false),
            CO_ADD => (ColorOp::Add, false),
            CO_SUBTRACT => (ColorOp::Subtract, swapped),
            CO_ALPHA_BLEND_WITH_MASK => (blend_op, swapped),
            CO_ADD_WITH_MASK_MODULATION => (ColorOp::ModulateAlphaAddColor, swapped),
            _ => return None,
        };
        let alpha_op = match node.alpha_operation {
            AO_USE_MASK => mask_alpha,
            AO_MULTIPLY => AlphaOp::Modulate,
            AO_ADD => AlphaOp::Add,
            AO_ALPHA_FROM_MATERIAL1 => {
                if swapped {
                    AlphaOp::Texture
                } else {
                    AlphaOp::Current
                }
            }
            AO_ALPHA_FROM_MATERIAL2 => {
                if swapped {
                    AlphaOp::Current
                } else {
                    AlphaOp::Texture
                }
            }
            _ => return None,
        };
        let source = self.bitmap_source(&simple)?;
        stages.push(Stage::new(source, &simple.info, color_op, alpha_op, stage_swapped));
        Some(())
    }

    fn opacity_stage(&mut self, opacity: &str, info: &Info, pass: &mut Pass) -> Option<Stage> {
        let leaf = self.check_material(opacity, info.clone(), 1)?;
        if leaf.is_bitmap() {
            let source = self.bitmap_source(&leaf)?;
            Some(Stage::new(source, &leaf.info, ColorOp::SelectCurrent, AlphaOp::Texture, false))
        } else if leaf.is_constant() {
            pass.set_constant(leaf.node.color, leaf.node.fade, false)?;
            Some(Stage::factor(ColorOp::SelectCurrent, AlphaOp::TFactor))
        } else if leaf.is_vertex_color() {
            Some(Stage { source: StageSource::Diffuse, ..Stage::factor(ColorOp::SelectCurrent, AlphaOp::Diffuse) })
        } else {
            None
        }
    }

    fn self_illumination(
        &mut self,
        node: &Node,
        info: &Info,
        opacity: Option<&str>,
        stages: &mut Vec<Stage>,
    ) -> Option<Option<(Pass, Vec<Stage>)>> {
        let self_illumination = node.self_illumination.as_deref()?;
        let si = self.check_material(self_illumination, info.clone(), 1)?;
        let sim = self.check_material(node.self_illumination_mask.as_deref()?, info.clone(), 1)?;
        if let Some(opacity) = opacity {
            let opacity_leaf = self.check_material(opacity, info.clone(), 1)?;
            if !opacity_leaf.is_bitmap() || !sim.is_bitmap() {
                return None;
            }
            let mut second = Pass::new(0);
            second.z_write = false;
            second.two_sided = node.two_sided;
            second.lit = false;
            let mut second_stages = Vec::new();
            self.handle_combined(&si, &mut second, &mut second_stages, 1)?;
            let mask_source = self.bitmap_source(&sim)?;
            second_stages.push(Stage::new(mask_source, &sim.info, ColorOp::SelectCurrent, AlphaOp::Texture, false));
            let opacity_source = self.bitmap_source(&opacity_leaf)?;
            second_stages.push(Stage::new(
                opacity_source,
                &opacity_leaf.info,
                ColorOp::SelectCurrent,
                AlphaOp::Modulate,
                false,
            ));
            second.src = BlendFactor::SrcAlpha;
            second.dst = BlendFactor::InvSrcAlpha;
            second.alpha_blend = true;
            second.light_after = second_stages.len();
            return Some(Some((second, second_stages)));
        }
        if si.is_bitmap() && sim.is_bitmap() {
            let mask_source = self.bitmap_source(&sim)?;
            stages.push(Stage::new(mask_source, &sim.info, ColorOp::SelectCurrent, AlphaOp::Texture, false));
            let source = self.bitmap_source(&si)?;
            stages.push(Stage::new(source, &si.info, ColorOp::BlendCurrentAlpha, AlphaOp::Current, false));
            return Some(None);
        }
        let mut second = Pass::new(0);
        second.z_write = false;
        second.two_sided = node.two_sided;
        second.lit = false;
        let mut second_stages = Vec::new();
        self.handle_combined(&si, &mut second, &mut second_stages, 1)?;
        if !same_object(&sim.path, &si.path) {
            if !sim.is_bitmap() {
                return None;
            }
            let mask_source = self.bitmap_source(&sim)?;
            second_stages.push(Stage::new(mask_source, &sim.info, ColorOp::SelectCurrent, AlphaOp::Texture, false));
        }
        second.src = BlendFactor::SrcAlpha;
        second.dst = BlendFactor::InvSrcAlpha;
        second.alpha_blend = true;
        second.light_after = second_stages.len();
        Some(Some((second, second_stages)))
    }

    fn specular(
        &mut self,
        node: &Node,
        info: &Info,
        self_illum_done: bool,
        with_opacity: bool,
        pass: &mut Pass,
        stages: &mut Vec<Stage>,
    ) -> Option<Option<(Pass, Vec<Stage>)>> {
        let specular = node.specular.as_deref()?;
        let spec = self.check_material(specular, info.clone(), 1)?;
        let mask = node.specularity_mask.as_deref().and_then(|mask| self.check_material(mask, info.clone(), 1));
        let mask_is_diffuse = node
            .specularity_mask
            .as_deref()
            .zip(node.diffuse.as_deref())
            .is_some_and(|(mask, diffuse)| same_object(mask, diffuse));
        let mask_constant = mask.as_ref().is_some_and(Leaf::is_constant);
        if !with_opacity
            && spec.is_bitmap()
            && (node.specularity_mask.is_none() || (mask_is_diffuse && !self_illum_done) || mask_constant)
        {
            let color_op = if mask_constant {
                let mask = mask.as_ref()?;
                pass.set_constant(mask.node.color, mask.node.fade, true)?;
                ColorOp::MultiplyAdd
            } else if node.specularity_mask.is_none() {
                ColorOp::Add
            } else {
                ColorOp::ModulateAlphaAddColor
            };
            let source = self.bitmap_source(&spec)?;
            stages.push(Stage::new(source, &spec.info, color_op, AlphaOp::Current, true));
            return Some(None);
        }
        if !with_opacity && spec.is_bitmap() && mask.as_ref().is_some_and(Leaf::is_bitmap) {
            let mask = mask.as_ref()?;
            let mask_source = self.bitmap_source(mask)?;
            stages.push(Stage::new(mask_source, &mask.info, ColorOp::SelectCurrent, AlphaOp::Texture, false));
            let source = self.bitmap_source(&spec)?;
            stages.push(Stage::new(source, &spec.info, ColorOp::ModulateAlphaAddColor, AlphaOp::Current, true));
            return Some(None);
        }
        if mask_constant {
            return None;
        }
        let mut second = Pass::new(0);
        second.src = BlendFactor::One;
        second.dst = BlendFactor::One;
        second.alpha_blend = true;
        second.z_write = false;
        second.two_sided = node.two_sided;
        second.lit = false;
        second.fog = FogMode::Black;
        let mut second_stages = Vec::new();
        self.handle_combined(&spec, &mut second, &mut second_stages, 1)?;
        second.light_after = second_stages.len();
        if let Some(mask) = mask.as_ref() {
            if !mask.is_bitmap() {
                return None;
            }
            let mask_source = self.bitmap_source(mask)?;
            second_stages.push(Stage::new(mask_source, &mask.info, ColorOp::SelectCurrent, AlphaOp::Texture, false));
            second.src = BlendFactor::SrcAlpha;
        }
        Some(Some((second, second_stages)))
    }

    fn shader_material(
        &mut self,
        leaf: &Leaf,
        info: &Info,
        stages: &mut Vec<Stage>,
        passes: &mut Vec<Pass>,
    ) -> Option<()> {
        let node = &leaf.node;
        let flags = ShaderFlags {
            two_sided: node.two_sided,
            treat_as_two_sided: node.treat_as_two_sided,
            z_write: node.z_write,
            alpha_test: node.alpha_test,
            alpha_ref: node.alpha_ref,
        };
        let mut pass = Pass::new(stages.len());
        let mut diffuse = node.diffuse.clone();
        let mut self_illum_done = false;
        if node.self_illumination.is_some() && node.self_illumination_mask.is_none() {
            diffuse = node.self_illumination.clone();
            self_illum_done = true;
            pass.lit = false;
        }
        let opacity = info.opacity.clone().or_else(|| node.opacity.clone());
        let diffuse = diffuse?;
        let diffuse_leaf = self.check_material(&diffuse, info.clone(), 1)?;
        self.handle_combined(&diffuse_leaf, &mut pass, stages, 1)?;
        pass.light_after = stages.len();
        if let Some(opacity) = opacity.as_deref().filter(|opacity| !same_object(opacity, &diffuse)) {
            let stage = self.opacity_stage(opacity, info, &mut pass)?;
            stages.push(stage);
        }
        let mut extra_passes = Vec::new();
        if !self_illum_done && node.self_illumination.is_some() {
            extra_passes.extend(self.self_illumination(node, info, opacity.as_deref(), stages)?);
        }
        if node.specular.is_some() {
            extra_passes.extend(self.specular(node, info, self_illum_done, opacity.is_some(), &mut pass, stages)?);
        }
        if (info.modify_fb && info.fb == FB_INVISIBLE) || (!info.modify_fb && node.output_blending == OB_INVISIBLE) {
            stages.clear();
            return Some(());
        }
        if info.modify_fb {
            apply_final_blend(&mut pass, info)?;
        } else {
            output_blending(&mut pass, node.output_blending, opacity.is_some(), flags)?;
            if !flags.treat_as_two_sided {
                pass.two_sided = flags.two_sided;
            }
        }
        if info.modify_color {
            color_modifier(&mut pass, stages, info);
        }
        l2_color_modifier(&mut pass, stages, info, MaterialPath::Shader)?;
        pass.stages.end = stages.len();
        passes.push(pass);
        for (mut second, second_stages) in extra_passes {
            second.stages = stages.len()..stages.len() + second_stages.len();
            stages.extend(second_stages);
            passes.push(second);
        }
        Some(())
    }

    fn bitmap_source(&mut self, leaf: &Leaf) -> Option<StageSource> {
        match leaf.node.class.as_str() {
            "texture" | "wettexture" => Some(StageSource::Texture(TextureRef {
                path: leaf.path.clone(),
                frames: self.frames(&leaf.path, &leaf.node),
                rate: leaf.node.max_frame_rate,
                total_frames: leaf.node.total_frame_num,
                size: leaf.node.size,
                clamp: leaf.node.clamp,
            })),
            "cubemap" => Some(StageSource::Cubemap(Cubemap { faces: leaf.node.faces.clone()?, size: leaf.node.size })),
            _ => None,
        }
    }

    fn frames(&mut self, first: &str, node: &Node) -> Vec<String> {
        let mut frames = Vec::new();
        let mut next = node.anim_next.clone();
        while let Some(path) = next.take() {
            if same_object(&path, first) || frames.len() >= MAX_TEXTURE_FRAMES {
                break;
            }
            next = self.node(&path).and_then(|node| node.anim_next);
            frames.push(path);
        }
        frames
    }

    fn node(&mut self, path: &str) -> Option<Node> {
        let enchant_colors = self
            .enchant_color_override
            .as_ref()
            .filter(|(target, _)| same_object(target, path))
            .map(|(_, colors)| *colors);
        let package_name = package_name(path)?;
        if let Some((class, from)) = self.imported_class_package(path)
            && from.eq_ignore_ascii_case("engine")
            && !self.engine_defines(&class)
        {
            return None;
        }
        let defaults = match self.indexed_class(path) {
            Some(class) => self.class_defaults(&class),
            None => Default::default(),
        };
        let found = self.object(path, "")?;
        let package = found.package;
        let objects = found.objects;
        let export = package.exports().get(found.index)?;
        let class = package.class_name(export).to_ascii_lowercase();
        let properties = object_properties(package, found.file, export).ok()?;
        let get = |name| find(&properties, name);
        let inherited =
            |name: &str| defaults.iter().find(|default| default.index == 0 && default.name.eq_ignore_ascii_case(name));
        let flag = |name, default: bool| {
            get(name).and_then(Property::bool).or_else(|| inherited(name)?.bool).unwrap_or(default)
        };
        let byte =
            |name, default: u8| get(name).and_then(Property::byte).or_else(|| inherited(name)?.byte).unwrap_or(default);
        let int =
            |name, default: i32| get(name).and_then(Property::int).or_else(|| inherited(name)?.int).unwrap_or(default);
        let float =
            |name, default| get(name).and_then(Property::float).or_else(|| inherited(name)?.float).unwrap_or(default);
        let rotator =
            |name| get(name).and_then(Property::rotator).or_else(|| inherited(name)?.rotator).unwrap_or_default();
        let alpha_ref =
            get("AlphaRef").and_then(|value| value.byte().or_else(|| u8::try_from(value.int()?).ok())).unwrap_or(0);
        let reference = |name| full_path(&package_name, package, objects, get(name)?.object(package)?);
        let mut faces: [Option<String>; 6] = Default::default();
        for property in properties.iter().filter(|property| property.name.eq_ignore_ascii_case("Faces")) {
            if let Some(slot) = faces.get_mut(property.index as usize)
                && let Some(face) = property.object(package).filter(|face| !matches!(face, ObjectRef::Null))
            {
                *slot = full_path(&package_name, package, objects, face);
            }
        }
        let mut node = Node {
            diffuse: reference("Diffuse"),
            opacity: reference("Opacity"),
            specular: reference("Specular"),
            specularity_mask: reference("SpecularityMask"),
            self_illumination: reference("SelfIllumination"),
            self_illumination_mask: reference("SelfIlluminationMask"),
            material: reference("Material"),
            material1: reference("Material1"),
            material2: reference("Material2"),
            mask: reference("Mask"),
            alpha_texture: flag("bAlphaTexture", false),
            masked: flag("bMasked", false),
            two_sided: flag("bTwoSided", false) || flag("TwoSided", false),
            treat_as_two_sided: flag("TreatAsTwoSided", false),
            z_write: flag("ZWrite", class == "finalblend"),
            z_test: flag("ZTest", true),
            alpha_test: flag("AlphaTest", false),
            alpha_ref,
            render_two_sided: flag("RenderTwoSided", matches!(class.as_str(), "colormodifier" | "l2colormodifier")),
            alpha_blend: flag("AlphaBlend", matches!(class.as_str(), "colormodifier" | "l2colormodifier")),
            modulate2x: flag("Modulate2X", false),
            modulate4x: flag("Modulate4X", false),
            output_blending: byte("OutputBlending", 0),
            frame_buffer_blending: byte("FrameBufferBlending", 0),
            combine_operation: byte("CombineOperation", 0),
            alpha_operation: byte("AlphaOperation", 0),
            l2_alpha_op: byte("AlphaOp", 0),
            l2_color_op: byte("ColorOp", 0),
            color: get("Color").and_then(Property::color),
            anim_next: reference("AnimNext"),
            max_frame_rate: float("MaxFrameRate", 0.0),
            total_frame_num: u32::try_from(int("TotalFrameNum", 0)).unwrap_or(0),
            size: [texels(int("USize", 0)), texels(int("VSize", 0))],
            clamp: [byte("UClampMode", 0) != 0, byte("VClampMode", 0) != 0],
            faces: (class == "cubemap").then(|| faces.map(Option::unwrap_or_default)),
            fade: get("Color1").and_then(Property::color).map(|color1| Fade {
                color1,
                color2: get("Color2").and_then(Property::color).unwrap_or_default(),
                period: float("FadePeriod", 0.0),
                phase: float("FadePhase", 0.0),
                sinusoidal: byte("ColorFadeType", 0) == 1,
            }),
            tex_coord_source: get("TexCoordSource").and_then(Property::byte),
            source_channel: int("SourceChannel", 0),
            env_map_type: byte("EnvMapType", EM_CAMERA_SPACE),
            pan_direction: rotator("PanDirection"),
            pan_rate: float("PanRate", 0.0),
            scale: [float("UScale", 1.0), float("VScale", 1.0)],
            offset: [float("UOffset", 0.0), float("VOffset", 0.0)],
            rotator: Rotator {
                kind: if flag("ConstantRotation", false) { TR_CONSTANTLY } else { byte("TexRotationType", 0) },
                rotation: rotator("Rotation"),
                offset: [float("UOffset", 0.0), float("VOffset", 0.0)],
                oscillation_rate: rotator("OscillationRate"),
                oscillation_amplitude: rotator("OscillationAmplitude"),
                oscillation_phase: rotator("OscillationPhase"),
            },
            oscillator: Oscillator {
                rate: [float("UOscillationRate", 0.0), float("VOscillationRate", 0.0)],
                phase: [float("UOscillationPhase", 0.0), float("VOscillationPhase", 0.0)],
                amplitude: [float("UOscillationAmplitude", 0.0), float("VOscillationAmplitude", 0.0)],
                kind: [byte("UOscillationType", 0), byte("VOscillationType", 0)],
                offset: [float("UOffset", 0.0), float("VOffset", 0.0)],
            },
            trigger: Trigger {
                stop_after: float("StopAfterPeriod", 0.0),
                triggered_time: float("TriggeredTime", 0.0),
                reverse: flag("Reverse", false),
            },
            matrix: get("Matrix").and_then(|matrix| matrix.fields(package)).map_or(Mat3x2::IDENTITY, |fields| {
                texture_matrix(["XPlane", "YPlane", "ZPlane"].map(|name| {
                    find(&fields, name).and_then(|plane| plane.plane(package)).map_or([0.0; 4], |plane| plane)
                }))
            }),
            class,
        };
        if let Some(colors) = enchant_colors
            && let Some(fade) = &mut node.fade
        {
            fade.color1[..3].copy_from_slice(&colors[0]);
            fade.color2[..3].copy_from_slice(&colors[1]);
        }
        Some(node)
    }

    fn class_defaults(&mut self, class: &str) -> std::sync::Arc<Vec<ClassDefault>> {
        let key = class.to_ascii_lowercase();
        if let Some(cached) = self.class_defaults.get(&key) {
            return cached.clone();
        }
        let mut chain: Vec<ClassDefault> = Vec::new();
        let mut next = Some(class.to_owned());
        let mut hops = 0;
        while let Some(name) = next.take() {
            hops += 1;
            if hops > MAX_CLASS_CHAIN {
                break;
            }
            let Some(found) = self.object(&format!("engine.{name}"), "Class") else { break };
            let Some(export) = found.package.exports().get(found.index) else { break };
            if let Ok(properties) = ue2_assets::class_default_properties(found.package, found.file, export) {
                for property in properties {
                    if !chain
                        .iter()
                        .any(|known| known.index == property.index && known.name.eq_ignore_ascii_case(property.name))
                    {
                        chain.push(ClassDefault {
                            name: property.name.to_owned(),
                            index: property.index,
                            float: property.float(),
                            int: property.int(),
                            byte: property.byte(),
                            bool: property.bool(),
                            rotator: property.rotator(),
                        });
                    }
                }
            }
            next = match export.super_class {
                ObjectRef::Export(_) => Some(found.package.object_name(export.super_class).to_owned()),
                _ => None,
            };
        }
        let chain = std::sync::Arc::new(chain);
        self.class_defaults.insert(key, chain.clone());
        chain
    }

    pub fn texture_size(&mut self, path: &str) -> Option<[f32; 2]> {
        matches!(self.indexed_class(path)?.to_ascii_lowercase().as_str(), "texture" | "wettexture")
            .then(|| self.node(path))?
            .map(|node| node.size)
    }
}

const MAX_CLASS_CHAIN: usize = 32;

#[derive(Debug, Clone)]
pub(crate) struct ClassDefault {
    name: String,
    index: u32,
    float: Option<f32>,
    int: Option<i32>,
    byte: Option<u8>,
    bool: Option<bool>,
    rotator: Option<[i32; 3]>,
}

fn texels(value: i32) -> f32 {
    value as f32
}

fn is_modifier(class: &str) -> bool {
    matches!(
        class,
        "finalblend"
            | "colormodifier"
            | "l2colormodifier"
            | "opacitymodifier"
            | "texpanner"
            | "texscaler"
            | "texrotator"
            | "texoscillator"
            | "texpannertriggered"
            | "texoscillatortriggered"
            | "texmatrix"
            | "texcoordsource"
            | "texenvmap"
            | "texmodifier"
            | "modifier"
    )
}

fn same_object(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

struct Node {
    class: String,
    diffuse: Option<String>,
    opacity: Option<String>,
    specular: Option<String>,
    specularity_mask: Option<String>,
    self_illumination: Option<String>,
    self_illumination_mask: Option<String>,
    material: Option<String>,
    material1: Option<String>,
    material2: Option<String>,
    mask: Option<String>,
    alpha_texture: bool,
    masked: bool,
    two_sided: bool,
    treat_as_two_sided: bool,
    z_write: bool,
    z_test: bool,
    alpha_test: bool,
    alpha_ref: u8,
    render_two_sided: bool,
    alpha_blend: bool,
    modulate2x: bool,
    modulate4x: bool,
    output_blending: u8,
    frame_buffer_blending: u8,
    combine_operation: u8,
    alpha_operation: u8,
    l2_alpha_op: u8,
    l2_color_op: u8,
    color: Option<[u8; 4]>,
    anim_next: Option<String>,
    max_frame_rate: f32,
    total_frame_num: u32,
    size: [f32; 2],
    clamp: [bool; 2],
    faces: Option<[String; 6]>,
    fade: Option<Fade>,
    tex_coord_source: Option<u8>,
    source_channel: i32,
    env_map_type: u8,
    pan_direction: [i32; 3],
    pan_rate: f32,
    scale: [f32; 2],
    offset: [f32; 2],
    rotator: Rotator,
    oscillator: Oscillator,
    trigger: Trigger,
    matrix: Mat3x2,
}

impl Node {
    fn null() -> Self {
        Self {
            class: "none".to_owned(),
            diffuse: None,
            opacity: None,
            specular: None,
            specularity_mask: None,
            self_illumination: None,
            self_illumination_mask: None,
            material: None,
            material1: None,
            material2: None,
            mask: None,
            alpha_texture: false,
            masked: false,
            two_sided: false,
            treat_as_two_sided: false,
            z_write: false,
            z_test: true,
            alpha_test: false,
            alpha_ref: 0,
            render_two_sided: false,
            alpha_blend: false,
            modulate2x: false,
            modulate4x: false,
            output_blending: 0,
            frame_buffer_blending: 0,
            combine_operation: 0,
            alpha_operation: 0,
            l2_alpha_op: 0,
            l2_color_op: 0,
            color: None,
            anim_next: None,
            max_frame_rate: 0.0,
            total_frame_num: 0,
            size: [0.0; 2],
            clamp: [false; 2],
            faces: None,
            fade: None,
            tex_coord_source: None,
            source_channel: 0,
            env_map_type: EM_CAMERA_SPACE,
            pan_direction: [0; 3],
            pan_rate: 0.0,
            scale: [1.0; 2],
            offset: [0.0; 2],
            rotator: Rotator {
                kind: TR_FIXED,
                rotation: [0; 3],
                offset: [0.0; 2],
                oscillation_rate: [0; 3],
                oscillation_amplitude: [0; 3],
                oscillation_phase: [0; 3],
            },
            oscillator: Oscillator {
                rate: [0.0; 2],
                phase: [0.0; 2],
                amplitude: [0.0; 2],
                kind: [0; 2],
                offset: [0.0; 2],
            },
            trigger: Trigger { stop_after: 0.0, triggered_time: 0.0, reverse: false },
            matrix: Mat3x2::IDENTITY,
        }
    }

    fn is_bitmap(&self) -> bool {
        matches!(self.class.as_str(), "texture" | "cubemap" | "wettexture")
    }

    fn uv_modifier(&self, class: &str, size: [f32; 2]) -> Option<UvModifier> {
        match class {
            "texpanner" => Some(UvModifier::Panner { direction: self.pan_direction, rate: self.pan_rate }),
            "texscaler" => Some(UvModifier::Scaler { scale: self.scale, offset: self.offset, size }),
            "texrotator" => Some(UvModifier::Rotator(self.rotator, size)),
            "texoscillator" => Some(UvModifier::Oscillator(self.oscillator, size)),
            "texpannertriggered" => Some(UvModifier::PannerTriggered {
                direction: self.pan_direction,
                rate: self.pan_rate,
                trigger: self.trigger,
            }),
            "texoscillatortriggered" => Some(UvModifier::OscillatorTriggered(self.oscillator, size, self.trigger)),
            "texmatrix" => Some(UvModifier::Matrix(self.matrix)),
            _ => None,
        }
    }

    fn coord_source(&self, class: &str) -> Option<CoordSource> {
        match class {
            "texcoordsource" => {
                u8::try_from(self.source_channel.clamp(0, i32::from(TCS_STREAM_MAX))).ok().map(CoordSource::Stream)
            }
            "texenvmap" => Some(if self.env_map_type == EM_WORLD_SPACE {
                CoordSource::WorldEnvMap
            } else {
                CoordSource::CameraEnvMap
            }),
            _ => match self.tex_coord_source? {
                source if source <= TCS_STREAM_MAX => Some(CoordSource::Stream(source)),
                TCS_WORLD_ENV_MAP => Some(CoordSource::WorldEnvMap),
                TCS_CAMERA_ENV_MAP => Some(CoordSource::CameraEnvMap),
                _ => None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn class_defaults_of_the_engine_modifiers_are_read_from_engine_u() {
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let mut catalog = Catalog::open(&root);
        catalog.register_package("Engine", &root.join("system").join("engine.u"));
        let value =
            |defaults: &[ClassDefault], name: &str| defaults.iter().find(|d| d.name == name).and_then(|d| d.float);
        let oscillator = catalog.class_defaults("TexOscillator");
        assert_eq!(value(&oscillator, "UOscillationRate"), Some(1.0));
        assert_eq!(value(&oscillator, "VOscillationRate"), Some(1.0));
        assert!((value(&oscillator, "UOscillationAmplitude").unwrap() - 0.1).abs() < 1.0e-6);
        assert!((value(&oscillator, "VOscillationAmplitude").unwrap() - 0.1).abs() < 1.0e-6);
        let panner = catalog.class_defaults("TexPanner");
        assert!((value(&panner, "PanRate").unwrap() - 0.1).abs() < 1.0e-6);
        let triggered = catalog.class_defaults("TexOscillatorTriggered");
        assert_eq!(value(&triggered, "StopAfterPeriod"), Some(0.5));
        assert_eq!(value(&triggered, "UOscillationRate"), Some(0.0));
        assert_eq!(value(&triggered, "VOscillationRate"), Some(0.5));
        assert_eq!(value(&triggered, "VOscillationAmplitude"), Some(0.5));
        assert_eq!(triggered.iter().find(|d| d.name == "RetriggerAction").and_then(|d| d.byte), Some(3));
        assert_eq!(value(&catalog.class_defaults("TexPannerTriggered"), "StopAfterPeriod"), Some(0.5));
    }

    #[test]
    #[ignore]
    fn the_triggered_oscillator_of_flameset_a_keeps_its_class_defaults_and_overrides() {
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let mut catalog = Catalog::open(&root);
        catalog.register_package("Engine", &root.join("system").join("engine.u"));
        let node = catalog.node("fx_e_t.Flameset_a.FX_E_under_pan03").unwrap();
        assert_eq!(node.class, "texoscillatortriggered");
        assert_eq!(node.trigger, Trigger { stop_after: 0.05, triggered_time: 0.0, reverse: false });
        assert_eq!(node.oscillator.rate[0], 0.0);
        let modifier = node.uv_modifier("texoscillatortriggered", [64.0, 64.0]).unwrap();
        assert!(matches!(modifier, UvModifier::OscillatorTriggered(..)));
        let panner = catalog.node("fx_e_t.Flameset_a.FX_E_under_pan04").unwrap();
        assert_eq!(panner.pan_rate, 0.01);
    }

    #[test]
    #[ignore]
    fn enchanted_material_1067ee30_preserves_everything_except_target_rgb() {
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let mut catalog = Catalog::open(&root);
        let path = "LineageEffectsTextures.Etc.Enchant_Aura001_Shader01";
        let fade_path = catalog.enchant_fade_path(path).expect("Shader.Diffuse.Combiner.Material1.FadeColor");
        let original_fade = catalog.node(&fade_path).unwrap().fade.unwrap();
        let original = catalog.material(path).unwrap();
        let colors = [[11, 22, 33], [44, 55, 66]];
        let tinted = catalog.enchanted_material(path, Some(colors)).unwrap();
        let mut expected = original.clone();
        let mut modified = 0;
        for pass in &mut expected.passes {
            if pass.fade == Some(original_fade) {
                let fade = pass.fade.as_mut().unwrap();
                fade.color1[..3].copy_from_slice(&colors[0]);
                fade.color2[..3].copy_from_slice(&colors[1]);
                modified += 1;
            }
        }
        assert!(modified > 0);
        assert_eq!(tinted, expected);
        assert_eq!(catalog.material(path).unwrap(), original);
        assert_eq!(catalog.enchanted_material(path, None).unwrap(), original);
    }

    #[test]
    fn wettexture_is_a_bitmap_and_keeps_its_own_flags_size_and_clamp() {
        let node = Node {
            class: "wettexture".to_owned(),
            size: [128.0, 256.0],
            clamp: [true, false],
            masked: true,
            two_sided: true,
            ..Node::null()
        };
        assert!(node.is_bitmap());
        let leaf = Leaf { path: "Synthetic.Wet".to_owned(), node, info: Info::new() };
        let mut catalog = Catalog::default();
        let mut info = Info::new();
        let mut stages = Vec::new();
        let mut passes = Vec::new();
        catalog.simple_material(&leaf, &mut info, &mut stages, &mut passes).unwrap();
        let StageSource::Texture(texture) = &stages[0].source else { panic!("bitmap esperado") };
        assert_eq!(texture.path, "Synthetic.Wet");
        assert_eq!(texture.size, [128.0, 256.0]);
        assert_eq!(texture.clamp, [true, false]);
        assert_eq!(passes.len(), 1);
        assert!(passes[0].two_sided);
        assert_eq!(passes[0].alpha_test, Some(ALPHA_REF_MASKED));
    }

    const SHADER: ShaderFlags =
        ShaderFlags { two_sided: false, treat_as_two_sided: false, z_write: false, alpha_test: false, alpha_ref: 10 };
    const Z_SHADER: ShaderFlags = ShaderFlags { z_write: true, alpha_test: true, ..SHADER };

    type Row = (BlendFactor, BlendFactor, bool, Option<u8>, bool, FogMode);

    fn tuple(pass: &Pass) -> Row {
        (pass.src, pass.dst, pass.alpha_blend, pass.alpha_test, pass.z_write, pass.fog)
    }

    fn shader_pass(ob: u8, opacity: bool, flags: ShaderFlags) -> Option<Row> {
        let mut pass = Pass::new(0);
        output_blending(&mut pass, ob, opacity, flags)?;
        Some(tuple(&pass))
    }

    #[test]
    fn output_blending_table_matches_the_d3ddrv() {
        use BlendFactor::*;
        assert_eq!(shader_pass(0, false, SHADER), Some((One, Zero, false, None, true, FogMode::Zone)));
        assert_eq!(shader_pass(0, true, SHADER), Some((SrcAlpha, InvSrcAlpha, true, Some(0), false, FogMode::Zone)));
        assert_eq!(shader_pass(0, true, Z_SHADER), Some((SrcAlpha, InvSrcAlpha, true, Some(10), true, FogMode::Zone)));
        assert_eq!(
            shader_pass(0, true, ShaderFlags { alpha_test: false, ..Z_SHADER }),
            Some((SrcAlpha, InvSrcAlpha, true, None, true, FogMode::Zone))
        );
        assert_eq!(shader_pass(1, false, SHADER), Some((One, Zero, false, Some(127), true, FogMode::Zone)));
        assert_eq!(shader_pass(2, false, SHADER), Some((DstColor, SrcColor, true, None, false, FogMode::Gray127)));
        assert_eq!(shader_pass(3, false, SHADER), Some((One, One, true, None, false, FogMode::Black)));
        assert_eq!(shader_pass(4, false, SHADER), None);
        assert_eq!(shader_pass(5, false, SHADER), Some((One, InvSrcColor, true, None, false, FogMode::Black)));
        assert_eq!(shader_pass(6, false, SHADER), Some((Zero, InvSrcColor, true, None, false, FogMode::Black)));
        assert_eq!(shader_pass(7, false, SHADER), None);
    }

    fn final_blend_pass(fb: u8) -> Option<Pass> {
        let info = Info {
            modify_fb: true,
            fb,
            z_write: true,
            z_test: false,
            alpha_test: true,
            two_sided: true,
            alpha_ref: 160,
            ..Info::default()
        };
        let mut pass = Pass::new(0);
        apply_final_blend(&mut pass, &info)?;
        Some(pass)
    }

    #[test]
    fn frame_buffer_blending_table_matches_the_d3ddrv() {
        use BlendFactor::*;
        let rows = [
            (0, (One, Zero, false, Some(160), true, FogMode::Zone)),
            (1, (DstColor, SrcColor, true, Some(160), true, FogMode::Gray127)),
            (2, (SrcAlpha, InvSrcAlpha, true, Some(160), true, FogMode::Zone)),
            (3, (One, InvSrcAlpha, true, Some(160), true, FogMode::Black)),
            (4, (One, InvSrcColor, true, Some(160), true, FogMode::Black)),
            (5, (Zero, InvSrcColor, true, Some(160), true, FogMode::Black)),
            (6, (SrcAlpha, One, true, Some(160), true, FogMode::Black)),
        ];
        for (fb, expected) in rows {
            let pass = final_blend_pass(fb).unwrap();
            assert_eq!(tuple(&pass), expected, "FB {fb}");
            assert!(!pass.z_test && pass.two_sided, "FB {fb}");
        }
        assert!(final_blend_pass(7).is_none());
        assert!(final_blend_pass(8).is_none());
    }

    #[test]
    fn texture_flags_turn_into_a_final_blend() {
        let mut masked = Info::new();
        texture_flags(&mut masked, true, false, true);
        assert!(masked.modify_fb && masked.alpha_test && masked.z_write && masked.z_test && masked.two_sided);
        assert_eq!((masked.fb, masked.alpha_ref), (FB_OVERWRITE, 127));

        let mut masked_tinted = Info { modify_color: true, alpha_blend: true, ..Info::new() };
        texture_flags(&mut masked_tinted, true, false, true);
        assert_eq!(masked_tinted.fb, FB_ALPHA_BLEND);
        assert!(!masked_tinted.two_sided);

        let mut alpha = Info::new();
        texture_flags(&mut alpha, false, true, false);
        assert_eq!(
            (alpha.fb, alpha.alpha_ref, alpha.alpha_test, alpha.z_write, alpha.two_sided),
            (FB_ALPHA_BLEND, 0, true, true, false)
        );

        let mut two_sided = Info::new();
        texture_flags(&mut two_sided, false, false, true);
        assert!(two_sided.modify_fb && two_sided.two_sided && !two_sided.alpha_test);
        assert_eq!(two_sided.fb, FB_OVERWRITE);

        let mut plain = Info { two_sided: true, ..Info::new() };
        texture_flags(&mut plain, false, false, false);
        assert!(!plain.modify_fb && !plain.two_sided);

        let mut kept = Info { modify_fb: true, fb: FB_BRIGHTEN, ..Info::new() };
        texture_flags(&mut kept, true, true, true);
        assert_eq!(kept.fb, FB_BRIGHTEN);
    }

    #[test]
    fn invisible_blends_leave_no_pass_to_draw() {
        let mut pass = Pass::new(0);
        assert!(output_blending(&mut pass, OB_INVISIBLE, false, SHADER).is_none());
        assert!(final_blend_pass(FB_INVISIBLE).is_none());
        let material = Material {
            stages: Vec::new(),
            passes: Vec::new(),
            sorting_level: 0,
            transparent: false,
            root_class: "shader".to_owned(),
            color_modifier: None,
        };
        assert_eq!(material.summary(), "invisible");
        assert!(material.texture_paths().is_empty());
    }

    #[test]
    fn a_color_modifier_adds_a_factor_stage_and_promotes_opaque_to_alpha() {
        let info =
            Info { modify_color: true, tfactor: [10, 20, 30, 40], alpha_blend: true, two_sided: true, ..Info::new() };
        let mut pass = Pass::new(0);
        let mut stages = Vec::new();
        color_modifier(&mut pass, &mut stages, &info);
        assert_eq!((pass.src, pass.dst), (BlendFactor::SrcAlpha, BlendFactor::InvSrcAlpha));
        assert!(pass.alpha_blend && pass.two_sided);
        assert_eq!(pass.tfactor, Some([10, 20, 30, 40]));
        assert_eq!(stages.len(), 1);
        assert_eq!((stages[0].color_op, stages[0].alpha_op), (ColorOp::Modulate, AlphaOp::Modulate));
        assert_eq!(stages[0].source, StageSource::Factor);

        let mut additive = Pass { src: BlendFactor::One, dst: BlendFactor::One, ..Pass::new(0) };
        color_modifier(&mut additive, &mut Vec::new(), &info);
        assert_eq!((additive.src, additive.dst), (BlendFactor::One, BlendFactor::One));
    }

    #[test]
    fn the_pass_factor_follows_the_fade_and_replicates_alpha_for_constant_masks() {
        let fade =
            Fade { color1: [255, 0, 0, 255], color2: [0, 0, 255, 51], period: 1.0, phase: 0.0, sinusoidal: false };
        let mut pass = Pass::new(0);
        assert_eq!(pass.tfactor_at(0.0), [255; 4]);
        pass.set_constant(None, Some(fade), false).unwrap();
        assert_eq!(pass.tfactor_at(1.0), [0, 0, 255, 51]);
        assert!(pass.set_constant(Some([1; 4]), None, false).is_none());
        let mut replicated = Pass::new(0);
        replicated.set_constant(None, Some(fade), true).unwrap();
        assert_eq!(replicated.tfactor_at(1.0), [51; 4]);
        let mut constant = Pass::new(0);
        constant.set_constant(Some([1, 2, 3, 4]), None, true).unwrap();
        assert_eq!(constant.tfactor_at(5.0), [4; 4]);
    }

    #[test]
    fn a_texture_frame_is_the_kth_successor_in_the_chain() {
        let texture = TextureRef {
            path: "P.a".into(),
            frames: vec!["P.b".into(), "P.c".into()],
            rate: 20.0,
            total_frames: 3,
            size: [1.0; 2],
            clamp: [false; 2],
        };
        assert_eq!(texture.frame_path(0.0), "P.a");
        assert_eq!(texture.frame_path(0.05), "P.b");
        assert_eq!(texture.frame_path(0.1), "P.c");
        assert_eq!(texture.frame_path(0.15), "P.a");
        assert!(texture.animated());
        let unanimated = TextureRef { total_frames: 0, ..texture.clone() };
        assert_eq!(unanimated.frame_path(0.1), "P.a");
        assert!(!unanimated.animated());
    }

    #[test]
    fn stage_matrices_compose_outer_modifiers_first() {
        let stage = Stage {
            source: StageSource::Factor,
            uv: vec![
                UvModifier::Panner { direction: [0, 0, 0], rate: -0.5 },
                UvModifier::Scaler { scale: [2.0, 2.0], offset: [0.0, 16.0], size: [64.0, 64.0] },
            ],
            coord_source: CoordSource::Stream(0),
            color_op: ColorOp::SelectTexture,
            alpha_op: AlphaOp::Texture,
            swapped: false,
        };
        let uv = stage.matrix(2.0).apply([1.0, 1.0]);
        assert!((uv[0] - 512.0).abs() < 1e-3 && (uv[1] - 0.75).abs() < 1e-5, "{uv:?}");
        assert!(stage.animated());
    }
}
