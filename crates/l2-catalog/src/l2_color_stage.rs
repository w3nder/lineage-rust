#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modifier {
    pub color: u32,
    pub alpha_op: u32,
    pub color_op: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PassState {
    pub factor: u32,
    pub src: u32,
    pub dst: u32,
    pub alpha_blend: u32,
    pub alpha_ref: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageState {
    pub texture: u32,
    pub color_op: u32,
    pub alpha_op: u32,
    pub color_args: [u32; 2],
    pub color_arg0: u32,
    pub alpha_args: [u32; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PassFlags {
    pub alpha_test: u32,
    pub two_sided: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModifierFlags {
    pub alpha_blend: u32,
    pub two_sided: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialPath {
    Shader,
    Simple,
}

pub fn apply_pass(
    modifier: Modifier,
    modifier_flags: ModifierFlags,
    pass: &mut PassState,
    flags: &mut PassFlags,
    stage: &mut StageState,
    path: MaterialPath,
) {
    if modifier_flags.alpha_blend != 0 && pass.src == 2 && pass.dst == 1 {
        pass.src = 5;
        pass.dst = 6;
    }
    apply(modifier, pass, stage);
    if path == MaterialPath::Simple {
        pass.factor = modifier.color;
    }
    pass.alpha_blend |= modifier_flags.alpha_blend;
    flags.two_sided |= modifier_flags.two_sided;
    if modifier.alpha_op != 1 {
        flags.alpha_test = 1;
        pass.alpha_ref = 0;
    }
}

pub fn texture_operation(operation: u32) -> u32 {
    TextureOperation::from_client(operation) as u32
}

pub fn apply(modifier: Modifier, pass: &mut PassState, stage: &mut StageState) {
    stage.color_args = [1, 3];
    stage.alpha_args = [1, 3];
    if modifier.alpha_op != 1 {
        if pass.src == 2 && pass.dst == 4 {
            let mut color = modifier.color.to_le_bytes();
            let scale = (f64::from(color[3]) * f64::from(f32::from_bits(0x3b808081))).clamp(0.0, 1.0);
            if modifier.color_op != 1 {
                for channel in &mut color[..3] {
                    *channel = (f64::from(*channel) * scale) as u8;
                }
            }
            pass.factor = u32::from_le_bytes(color);
            stage.color_op = 4;
            stage.alpha_op = 4;
        } else if pass.src == 5 && pass.dst == 6 {
            pass.factor = modifier.color;
            stage.alpha_op = if pass.alpha_blend != 0 || pass.alpha_ref == 127 { 4 } else { 3 };
            stage.color_op = 2;
        }
    }
    if modifier.color_op != 1 && stage.color_op == 2 {
        pass.factor = modifier.color;
        stage.color_op = texture_operation(modifier.color_op);
    }
    stage.texture = 0;
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TextureOperation {
    Disable = 1,
    SelectArg1,
    SelectArg2,
    Modulate,
    Modulate2X,
    Modulate4X,
    Add,
    AddSigned,
    AddSigned2X,
    Subtract,
    AddSmooth,
    BlendDiffuseAlpha,
    BlendTextureAlpha,
    BlendFactorAlpha,
    BlendTextureAlphaPm,
    BlendCurrentAlpha,
    PreModulate,
    ModulateAlphaAddColor,
    ModulateColorAddAlpha,
    ModulateInvAlphaAddColor,
    ModulateInvColorAddAlpha,
    BumpEnvMap,
    BumpEnvMapLuminance,
    DotProduct3,
    MultiplyAdd,
}

impl TextureOperation {
    pub fn from_client(value: u32) -> Self {
        match value {
            1 => Self::SelectArg1,
            2 => Self::SelectArg2,
            3 => Self::Modulate,
            4 => Self::Modulate2X,
            5 => Self::Modulate4X,
            6 => Self::Add,
            7 => Self::AddSigned,
            8 => Self::AddSigned2X,
            9 => Self::Subtract,
            10 => Self::AddSmooth,
            11 => Self::BlendDiffuseAlpha,
            12 => Self::BlendTextureAlpha,
            13 => Self::BlendFactorAlpha,
            14 => Self::BlendTextureAlphaPm,
            15 => Self::BlendCurrentAlpha,
            16 => Self::PreModulate,
            17 => Self::ModulateAlphaAddColor,
            18 => Self::ModulateColorAddAlpha,
            19 => Self::ModulateInvAlphaAddColor,
            20 => Self::ModulateInvColorAddAlpha,
            21 => Self::BumpEnvMap,
            22 => Self::BumpEnvMapLuminance,
            23 => Self::DotProduct3,
            24 => Self::MultiplyAdd,
            _ => Self::Disable,
        }
    }
}
