#[derive(Debug, Clone, PartialEq)]
pub struct SeqNotify {
    pub time: f32,
    pub kind: NotifyKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotifyKind {
    AttackShot,
    AttackPreShot,
    AttackItem,
    AttackVoice(AttackVoice),
    Sound(Box<SoundNotify>),
    SwimSound,
    Effect(Box<EffectNotify>),
    ViewShake(ViewShake),
    ScreenFade(ScreenFade),
    Other(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AttackVoice {
    pub volume: f32,
    pub radius: f32,
    pub random: i32,
}

pub const SURFACES: usize = 8;
pub const SOUND_VARIANTS: usize = 3;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SoundNotify {
    pub sound: Option<String>,
    pub volume: f32,
    pub radius: i32,
    pub random: i32,
    pub walk_run: [[Option<String>; SOUND_VARIANTS]; SURFACES],
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EffectNotify {
    pub effect_class: Option<String>,
    pub bone: Option<String>,
    pub offset_location: [f32; 3],
    pub offset_rotation: [i32; 3],
    pub attach: bool,
    pub tag: Option<String>,
    pub draw_scale: f32,
    pub draw_scale_3d: [f32; 3],
    pub trail_camera: bool,
    pub independent_rotation: bool,
    pub effect_scale: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ViewShake {
    pub shake_type: u8,
    pub intensity: f32,
    pub vector: [f32; 3],
    pub range: f32,
    pub count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ScreenFade {
    pub fade_out_duration: f32,
    pub fade_out_color: [u8; 4],
    pub black_out_duration: f32,
    pub fade_in_duration: f32,
}
