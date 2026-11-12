use glam::Vec3;
use killer_assets::scene::{GlobalFog, ZoneLighting};
use ue2_level::{FogColorInfo, L2FogInfo};

/// `L2DistanceFog::GetOutdoorFogColor`: the base inside an `L2FogInfo` is 50 % gray.
pub const OUTDOOR_FOG_COLOR: [f32; 3] = [127.0 / 255.0; 3];
const HOURS_PER_DAY: f32 = 24.0;
const BYTE: f32 = 255.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FogValues {
    pub start: f32,
    pub end: f32,
    pub color: [f32; 3],
}

/// Sky, cloud and haze colors an `L2FogInfo` imposes over the hour's tables (`ApplySkyColors(…, 1, …)`).
#[derive(Clone, Debug, PartialEq)]
pub struct SkyOverride {
    pub sky: [f32; 3],
    pub clouds: Vec<[f32; 3]>,
    pub haze: Vec<[f32; 3]>,
}

/// What `FLevelSceneNode::Render` hands to `SetDistanceFog` this frame.
#[derive(Clone, Debug, PartialEq)]
pub struct FogFrame {
    pub enabled: bool,
    pub values: FogValues,
    pub sky: Option<SkyOverride>,
    /// `CloudTexture` of the nearest `L2FogInfo` within its `TextureDistance`.
    pub cloud_texture: Option<String>,
}

/// The per-frame inputs of the fog rule.
pub struct FogInput<'a> {
    pub zone: ZoneLighting,
    pub hour: f32,
    /// `HazeringColor(hour)` in 0..1.
    pub hazering: [f32; 3],
    pub sky: [f32; 3],
    pub clouds: Vec<[f32; 3]>,
    pub haze: Vec<[f32; 3]>,
    pub view: Vec3,
    pub dt: f32,
    pub global: GlobalFog,
    /// Index into `FogRange1..5` for the client's `ClippingRange.Terrain`.
    pub fog_level: usize,
    pub fog_infos: &'a [L2FogInfo],
}

/// `L2DistanceFog`: target, previous and current values with a linear blend over `blend_time`.
#[derive(Clone, Debug, PartialEq)]
pub struct FogState {
    target: FogValues,
    old: FogValues,
    current: FogValues,
    blend_time: f32,
    remaining: f32,
    color_snapped: bool,
    last_terrain: bool,
    inside: bool,
    /// `L2DistanceFog+0/+4`: the option values last applied, to notice a change of `GL2FogStart/End`.
    option: [f32; 2],
}

impl Default for FogState {
    fn default() -> Self {
        let zero = FogValues { start: 0.0, end: 0.0, color: [0.0; 3] };
        Self { target: zero, old: zero, current: zero, blend_time: 1.0, remaining: 0.0, color_snapped: false, last_terrain: false, inside: false, option: [f32::NAN; 2] }
    }
}

fn lerp(a: f32, b: f32, f: f32) -> f32 {
    a + (b - a) * f
}

/// `L2DistanceFog::Update` (0x1042fa40) while a blend runs: `f = 1 - remaining/BlendTime` is not clamped, so the frame that
/// finishes the blend overshoots `Target` by up to `dt/BlendTime` of the distance, and the color keeps the low byte of each `ftol`.
pub fn blend_update(old: FogValues, target: FogValues, blend_time: f32, remaining: f32, dt: f32) -> (FogValues, f32) {
    let remaining = remaining - dt;
    let f = 1.0 - remaining / blend_time;
    (FogValues { start: lerp(old.start, target.start, f), end: lerp(old.end, target.end, f), color: lerp_byte(old.color, target.color, f) }, remaining)
}

/// `lerpByte` (0x1042fc60): per channel `ftol(float(b - a) * f + a)`; `_ftol2` (0x10740cb4) truncates toward zero and the store keeps the low byte.
pub fn lerp_byte(a: [f32; 3], b: [f32; 3], f: f32) -> [f32; 3] {
    let mut out = [0.0; 3];
    for (channel, slot) in out.iter_mut().enumerate() {
        let (from, to) = ((a[channel] * BYTE).round(), (b[channel] * BYTE).round());
        *slot = f32::from((((to - from) * f + from).trunc() as i32 & 0xff) as u8) / BYTE;
    }
    out
}

fn bytes(color: [u8; 4]) -> [f32; 3] {
    [f32::from(color[0]) / BYTE, f32::from(color[1]) / BYTE, f32::from(color[2]) / BYTE]
}

/// `AL2FogInfo::GetAffectFactor`.
pub fn affect_factor(fog: &L2FogInfo, view: Vec3) -> f32 {
    if fog.colors.len() < 2 {
        return 0.0;
    }
    let d = Vec3::from(fog.location).distance(view);
    let [min, max] = fog.affect_range;
    if d >= max {
        0.0
    } else if d < min {
        1.0
    } else {
        (max - d) / (max - min)
    }
}

/// `AL2FogInfo::IsActive`.
pub fn is_active(fog: &L2FogInfo, view: Vec3) -> bool {
    let d = Vec3::from(fog.location).distance(view);
    d <= fog.affect_range[1] || d < fog.texture_distance
}

/// `FindColorInterval` (0x10431080) plus the fraction of 0x1042f700: `hour` wraps into [0, 24) and is never shifted afterwards;
/// the first entry `i` with `A.time <= hour <= B.time` wins, where `B.time` gets +24 until it is not below `A.time`.
/// No entry means the original returns false and `GetFogColor` and its siblings fall back to black.
pub fn color_interval(colors: &[FogColorInfo], hour: f32) -> Option<(&FogColorInfo, &FogColorInfo, f32)> {
    let t = hour.rem_euclid(HOURS_PER_DAY);
    let n = colors.len();
    (0..n).find_map(|i| {
        let a = &colors[i];
        let b = &colors[(i + 1) % n];
        let mut b_time = b.time;
        while a.time > b_time {
            b_time += HOURS_PER_DAY;
        }
        (t >= a.time && t <= b_time).then(|| (a, b, (t - a.time) / (b_time - a.time)))
    })
}

/// `GetFogColor`/`GetSkyColor`/`GetCloudColors`/`GetHazeRingColors` of an `L2FogInfo` at `hour`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FogInfoColors {
    pub fog: [f32; 3],
    pub sky: [f32; 3],
    pub clouds: Vec<[f32; 3]>,
    pub haze: Vec<[f32; 3]>,
}

pub fn fog_colors(fog: &L2FogInfo, hour: f32) -> FogInfoColors {
    let Some((a, b, f)) = color_interval(&fog.colors, hour) else { return FogInfoColors::default() };
    let pairs = |x: &[[u8; 4]], y: &[[u8; 4]]| x.iter().zip(y).map(|(x, y)| lerp_byte(bytes(*x), bytes(*y), f)).collect::<Vec<_>>();
    FogInfoColors { fog: lerp_byte(bytes(a.fog), bytes(b.fog), f), sky: lerp_byte(bytes(a.sky), bytes(b.sky), f), clouds: pairs(&a.cloud, &b.cloud), haze: pairs(&a.haze_ring, &b.haze_ring) }
}

impl FogState {
    fn set_target(&mut self, start: f32, end: f32) {
        self.target.start = start;
        self.target.end = end;
    }

    fn snap(&mut self) {
        self.old = self.target;
        self.current = self.target;
        self.remaining = 0.0;
    }

    /// `L2DistanceFog::SnapColor`: the first terrain frame takes the hazering color without a transition.
    fn snap_color(&mut self) {
        self.old.color = self.target.color;
        self.current.color = self.target.color;
    }

    fn start_blend(&mut self, blend_time: f32) {
        self.blend_time = blend_time;
        self.old = self.current;
        self.remaining = blend_time;
    }

    /// `L2DistanceFog::Update`.
    fn advance(&mut self, dt: f32) -> FogValues {
        if self.remaining > 0.0 {
            let (current, remaining) = blend_update(self.old, self.target, self.blend_time, self.remaining, dt);
            self.current = current;
            self.remaining = remaining;
        } else {
            self.snap();
        }
        self.current
    }

    /// `FLevelSceneNode::Render` (b)–(f) for one frame.
    pub fn update(&mut self, input: &FogInput<'_>) -> FogFrame {
        let mut active: Vec<&L2FogInfo> = input.fog_infos.iter().filter(|fog| is_active(fog, input.view)).collect();
        let terrain = input.zone.terrain_zone;
        let mut sky = None;
        let mut cloud_texture = None;
        if active.is_empty() {
            if terrain {
                self.target.color = input.hazering;
                if !self.color_snapped {
                    self.color_snapped = true;
                    self.snap_color();
                }
            }
            if !self.inside {
                if (terrain && !self.last_terrain) || self.option != [input.global.start, input.global.end] {
                    self.retarget_global(input);
                } else if !terrain && let Some(fog) = input.zone.fog {
                    self.set_target(fog.start, fog.end);
                    self.target.color = fog.color;
                    if self.last_terrain {
                        self.start_blend(fog.blend_time);
                    }
                }
            } else {
                self.retarget_global(input);
            }
            self.last_terrain = terrain;
            self.inside = false;
        } else {
            let mut start = input.global.start;
            let mut end = input.global.end;
            let mut color = OUTDOOR_FOG_COLOR;
            let mut sky_color = input.sky;
            let mut clouds = input.clouds.clone();
            let mut haze = input.haze.clone();
            for fog in &active {
                let f = affect_factor(fog, input.view);
                let range = fog.fog_range.get(input.fog_level).copied().unwrap_or([0.0; 2]);
                let of = fog_colors(fog, input.hour);
                start = lerp(start, range[0], f);
                end = lerp(end, range[1], f);
                color = lerp_byte(color, of.fog, f);
                sky_color = lerp_byte(sky_color, of.sky, f);
                for (cloud, theirs) in clouds.iter_mut().zip(&of.clouds) {
                    *cloud = lerp_byte(*cloud, *theirs, f);
                }
                for (ring, theirs) in haze.iter_mut().zip(&of.haze) {
                    *ring = lerp_byte(*ring, *theirs, f);
                }
            }
            self.set_target(start, end);
            self.target.color = color;
            if !self.inside {
                self.start_blend(input.global.speed);
            }
            self.inside = true;
            sky = Some(SkyOverride { sky: sky_color, clouds, haze });
            active.sort_by(|a, b| Vec3::from(a.location).distance(input.view).total_cmp(&Vec3::from(b.location).distance(input.view)));
            cloud_texture = active.iter().find(|fog| Vec3::from(fog.location).distance(input.view) < fog.texture_distance).and_then(|fog| fog.cloud_texture.clone());
        }
        let values = self.advance(input.dt);
        FogFrame { enabled: input.zone.fog.is_some(), values, sky, cloud_texture }
    }

    fn retarget_global(&mut self, input: &FogInput<'_>) {
        self.option = [input.global.start, input.global.end];
        self.set_target(input.global.start, input.global.end);
        self.start_blend(input.global.speed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_assets::scene::FogParams;

    fn terrain_zone() -> ZoneLighting {
        ZoneLighting { terrain_zone: true, fog: Some(FogParams { color: [0.0; 3], start: 3000.0, end: 15000.0, blend_time: 1.0 }), ..ZoneLighting::default() }
    }

    fn interior() -> ZoneLighting {
        ZoneLighting { terrain_zone: false, fog: Some(FogParams { color: [57.0 / 255.0, 198.0 / 255.0, 176.0 / 255.0], start: 500.0, end: 16000.0, blend_time: 1.0 }), ..ZoneLighting::default() }
    }

    fn input<'a>(zone: ZoneLighting, view: Vec3, dt: f32, fogs: &'a [L2FogInfo]) -> FogInput<'a> {
        FogInput {
            zone,
            hour: 12.0,
            hazering: [188.0 / 255.0, 203.0 / 255.0, 224.0 / 255.0],
            sky: [66.0 / 255.0, 124.0 / 255.0, 176.0 / 255.0],
            clouds: vec![[1.0; 3]],
            haze: vec![[188.0 / 255.0, 203.0 / 255.0, 224.0 / 255.0]],
            view,
            dt,
            global: GlobalFog { start: 8192.0, end: 20480.0, speed: 1.0, level: 4 },
            fog_level: 4,
            fog_infos: fogs,
        }
    }

    fn close(a: [f32; 3], b: [u8; 3]) -> bool {
        (0..3).all(|i| (a[i] * 255.0 - f32::from(b[i])).abs() <= 1.0)
    }

    #[test]
    fn a_terrain_zone_takes_the_global_ranges_over_fog_speed_and_the_hazering_color_at_once() {
        let mut state = FogState::default();
        let first = state.update(&input(terrain_zone(), Vec3::ZERO, 0.5, &[]));
        assert!(first.enabled);
        assert!((first.values.start - 4096.0).abs() < 1.0 && (first.values.end - 10240.0).abs() < 1.0, "{:?}", first.values);
        assert!(close(first.values.color, [188, 203, 224]), "{:?}", first.values.color);
        let frame = state.update(&input(terrain_zone(), Vec3::ZERO, 0.5, &[]));
        assert_eq!((frame.values.start, frame.values.end), (8192.0, 20480.0));
        assert!(frame.sky.is_none());
        let off = state.update(&input(ZoneLighting { terrain_zone: false, fog: None, ..ZoneLighting::default() }, Vec3::ZERO, 0.1, &[]));
        assert!(!off.enabled);
    }

    #[test]
    fn an_interior_blends_from_the_terrain_fog_over_its_blend_time() {
        let mut state = FogState::default();
        state.update(&input(terrain_zone(), Vec3::ZERO, 1.0, &[]));
        let half = state.update(&input(interior(), Vec3::ZERO, 0.5, &[]));
        assert!((half.values.start - lerp(8192.0, 500.0, 0.5)).abs() < 1.0, "{}", half.values.start);
        assert!((half.values.end - lerp(20480.0, 16000.0, 0.5)).abs() < 1.0);
        let done = state.update(&input(interior(), Vec3::ZERO, 0.5, &[]));
        assert_eq!((done.values.start, done.values.end), (500.0, 16000.0));
        assert!(close(done.values.color, [57, 198, 176]), "{:?}", done.values.color);
    }

    #[test]
    fn the_frame_that_finishes_a_blend_overshoots_by_the_excess_dt() {
        let mut state = FogState::default();
        state.update(&input(terrain_zone(), Vec3::ZERO, 0.5, &[]));
        let frame = state.update(&input(terrain_zone(), Vec3::ZERO, 0.6, &[]));
        assert!((frame.values.start - lerp(0.0, 8192.0, 1.1)).abs() < 1.0, "{:?}", frame.values);
        let settled = state.update(&input(terrain_zone(), Vec3::ZERO, 0.016, &[]));
        assert_eq!((settled.values.start, settled.values.end), (8192.0, 20480.0));
    }

    fn kruma() -> L2FogInfo {
        let color = |time: f32, fog: [u8; 3], sky: [u8; 3]| FogColorInfo { time, fog: [fog[0], fog[1], fog[2], 255], sky: [sky[0], sky[1], sky[2], 255], cloud: vec![[10, 20, 30, 255]], haze_ring: vec![[40, 50, 60, 255]] };
        L2FogInfo {
            location: [0.0; 3],
            affect_range: [2000.0, 4000.0],
            fog_range: [[0.0, 2000.0], [0.0, 4000.0], [0.0, 6000.0], [0.0, 8000.0], [0.0, 10000.0]],
            texture_distance: 3000.0,
            colors: vec![color(0.0, [83, 83, 70], [91, 85, 74]), color(6.0, [97, 114, 100], [110, 120, 130]), color(18.0, [50, 50, 50], [60, 60, 60])],
            cloud_texture: Some("Fog.Cloud".into()),
        }
    }

    #[test]
    fn inside_a_fog_info_the_ranges_and_colors_lerp_by_the_affect_factor() {
        let fogs = [kruma()];
        let mut state = FogState::default();
        let at_min = state.update(&input(terrain_zone(), Vec3::new(2000.0, 0.0, 0.0), 1.0, &fogs));
        assert_eq!((at_min.values.start, at_min.values.end), (0.0, 10000.0));
        let of = fog_colors(&fogs[0], 12.0);
        let (fog_color, sky_color) = (of.fog, of.sky);
        assert!(close(fog_color, [74, 82, 75]), "{fog_color:?}");
        assert!(close(at_min.values.color, [74, 82, 75]), "{:?}", at_min.values.color);
        let sky = at_min.sky.clone().unwrap();
        assert!(close(sky.sky, [85, 90, 95]), "{:?}", sky.sky);
        assert_eq!(sky.sky, sky_color);
        assert_eq!(at_min.cloud_texture.as_deref(), Some("Fog.Cloud"));
        let mut state = FogState::default();
        let half = state.update(&input(terrain_zone(), Vec3::new(3000.0, 0.0, 0.0), 1.0, &fogs));
        assert!((half.values.start - 4096.0).abs() < 1.0 && (half.values.end - 15240.0).abs() < 1.0, "{:?}", half.values);
        let expected = lerp_byte(OUTDOOR_FOG_COLOR, fog_color, 0.5);
        assert!((0..3).all(|i| (half.values.color[i] - expected[i]).abs() <= 1.0 / 255.0), "{:?} vs {expected:?}", half.values.color);
        assert!(half.cloud_texture.is_none());
        let mut state = FogState::default();
        let outside = state.update(&input(terrain_zone(), Vec3::new(4001.0, 0.0, 0.0), 1.0, &fogs));
        assert_eq!((outside.values.start, outside.values.end), (8192.0, 20480.0));
        assert!(outside.sky.is_none());
    }

    #[test]
    fn color_intervals_wrap_around_midnight() {
        let fog = kruma();
        let (a, b, f) = color_interval(&fog.colors, 21.0).unwrap();
        assert_eq!((a.time, b.time), (18.0, 0.0));
        assert!((f - 0.5).abs() < 1e-6);
        let (a, b, f) = color_interval(&fog.colors, 3.0).unwrap();
        assert_eq!((a.time, b.time), (0.0, 6.0));
        assert!((f - 0.5).abs() < 1e-6);
    }

    #[test]
    fn the_interval_search_is_inclusive_and_never_shifts_the_hour() {
        let fog = kruma();
        let (a, b, f) = color_interval(&fog.colors, 6.0).unwrap();
        assert_eq!((a.time, b.time, f), (0.0, 6.0, 1.0));
        let mut late = fog;
        late.colors.remove(0);
        assert!(color_interval(&late.colors, 3.0).is_none());
        assert!(color_interval(&late.colors, 21.0).is_some());
        assert_eq!(fog_colors(&late, 3.0), FogInfoColors::default());
        assert!(color_interval(&[], 3.0).is_none());
    }

    #[test]
    fn lerp_byte_truncates_toward_zero_like_ftol() {
        let half = lerp_byte([0.0; 3], [1.0; 3], 0.5);
        assert_eq!(half, [127.0 / 255.0; 3]);
        assert_eq!(lerp_byte([1.0; 3], [0.0; 3], 0.5), [127.0 / 255.0; 3]);
    }
}
