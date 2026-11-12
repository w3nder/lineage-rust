use crate::client::{AssetError, Client};
use l2_dat::animnames::decode_text;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
struct Entry {
    t: f32,
    values: Vec<f32>,
}

/// `UL2NEnvManager` day phase (`+0x130`): 0 before 6 h, 1 in 6–8 h, 2 in 8–22 h, 3 in 22–24 h.
pub const PHASE_NIGHT: u8 = 3;
const PHASE_DAWN_START: f32 = 6.0;
const PHASE_DAY_START: f32 = 8.0;
const PHASE_NIGHT_START: f32 = 22.0;
const SCALE_SECTION_SUN: &str = "SUNSCALE";
const SCALE_SECTION_MOON: &str = "MOONSCALE";

pub fn phase(hour: f32) -> u8 {
    let hour = hour.rem_euclid(24.0);
    if hour < PHASE_DAWN_START {
        0
    } else if hour < PHASE_DAY_START {
        1
    } else if hour < PHASE_NIGHT_START {
        2
    } else {
        PHASE_NIGHT
    }
}

#[derive(Debug, Clone)]
pub struct TimeEnv {
    sections: HashMap<String, Vec<Entry>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmbientKind {
    Terrain,
    Actor,
    StaticMesh,
    Bsp,
}

impl AmbientKind {
    fn section(self) -> &'static str {
        match self {
            AmbientKind::Terrain => "TERRAINAMBIENT",
            AmbientKind::Actor => "ACTORAMBIENT",
            AmbientKind::StaticMesh => "STATICMESHAMBIENT",
            AmbientKind::Bsp => "BSPAMBIENT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightKind {
    Terrain,
    Actor,
    StaticMesh,
    Bsp,
}

impl LightKind {
    fn section(self) -> &'static str {
        match self {
            LightKind::Terrain => "HSVTERRAINLIGHT",
            LightKind::Actor => "HSVACTORLIGHT",
            LightKind::StaticMesh => "HSVSTATICMESHLIGHT",
            LightKind::Bsp => "HSVBSPLIGHT",
        }
    }
}

impl TimeEnv {
    pub fn load(client: &mut Client, env_type: u8) -> Result<TimeEnv, AssetError> {
        let bytes = client.read_system_file(&format!("timeenv{env_type}.int"))?;
        Ok(TimeEnv::parse(&decode_text(&bytes)))
    }

    pub fn parse(text: &str) -> TimeEnv {
        let mut sections: HashMap<String, Vec<Entry>> = HashMap::new();
        let mut current: Option<String> = None;
        for raw_line in text.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            if let Some(name) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                current = Some(name.trim().to_uppercase());
                continue;
            }
            let Some(section) = current.as_ref() else { continue };
            let Some(eq) = line.find('=') else { continue };
            let Some(open_rel) = line[eq..].find('(') else { continue };
            let Some(close) = line.rfind(')') else { continue };
            let open = eq + open_rel;
            if close <= open {
                continue;
            }
            let inner = &line[open + 1..close];
            if let Some(entry) = parse_entry(inner) {
                sections.entry(section.clone()).or_default().push(entry);
            }
        }
        for entries in sections.values_mut() {
            entries.sort_by(|a, b| a.t.total_cmp(&b.t));
        }
        TimeEnv { sections }
    }

    pub fn bsp_ambient_color(&self, hour: f32) -> [u8; 3] {
        self.interpolate_color(AmbientKind::Bsp.section(), hour).map(|channel| crate::pawn_rotation::ftol(f64::from(channel)) as u8)
    }

    pub fn ambient(&self, kind: AmbientKind, hour: f32) -> [f32; 3] {
        let rgb = self.interpolate_color(kind.section(), hour);
        rgb.map(|c| c / 255.0)
    }

    pub fn sun_light(&self, kind: LightKind, hour: f32) -> ([f32; 3], f32) {
        let Some(entries) = self.sections.get(kind.section()) else {
            return ([0.0; 3], 0.0);
        };
        let Some((a, b, f)) = surrounding(entries, hour) else {
            return ([0.0; 3], 0.0);
        };
        let (a, b) = (rgb(&a.values), rgb(&b.values));
        let ca = fgethsv(a[0] as u8, a[1] as u8, 255);
        let cb = fgethsv(b[0] as u8, b[1] as u8, 255);
        let color = lerp3(ca, cb, f);
        let brightness = lerp(a[2], b[2], f) / 255.0;
        (color, brightness)
    }

    /// `[SunColor]` in bytes, as `GetSunColor` hands it to `RenderSun`.
    pub fn sun_color(&self, hour: f32) -> [f32; 3] {
        self.interpolate_color("SUNCOLOR", hour)
    }

    /// `[SkyBoxColor]` in bytes for the `SKYBOX` color modifier.
    pub fn sky_box_color(&self, hour: f32) -> [f32; 3] {
        self.interpolate_color("SKYBOXCOLOR", hour)
    }

    /// `[HazeringColor]` in bytes for the `HAZERING` color modifier.
    pub fn hazering_color(&self, hour: f32) -> [f32; 3] {
        self.interpolate_color("HAZERINGCOLOR", hour)
    }

    /// `[CloudColor{i+1}]` in bytes for the `CLOUD{i+1}` color modifier.
    pub fn cloud_color(&self, index: usize, hour: f32) -> [f32; 3] {
        self.interpolate_color(&format!("CLOUDCOLOR{}", index + 1), hour)
    }

    /// `UL2NTimeLight::GetScale`: linear between `T_i <= t < T_{i+1}`, untouched (`None`) when no interval covers `t`.
    fn scale(&self, section: &str, hour: f32) -> Option<f32> {
        let entries = self.sections.get(section)?;
        let hour = hour.rem_euclid(24.0);
        entries.windows(2).find(|pair| hour >= pair[0].t && hour < pair[1].t).map(|pair| {
            let f = (hour - pair[0].t) / (pair[1].t - pair[0].t);
            lerp(pair[0].values[0], pair[1].values[0], f)
        })
    }

    pub fn sun_scale(&self, hour: f32) -> Option<f32> {
        self.scale(SCALE_SECTION_SUN, hour)
    }

    pub fn moon_scale(&self, hour: f32) -> Option<f32> {
        self.scale(SCALE_SECTION_MOON, hour)
    }

    fn interpolate_color(&self, section: &str, hour: f32) -> [f32; 3] {
        let Some(entries) = self.sections.get(section) else { return [0.0; 3] };
        let Some((a, b, f)) = surrounding(entries, hour) else { return [0.0; 3] };
        lerp3(rgb(&a.values), rgb(&b.values), f)
    }
}

fn rgb(values: &[f32]) -> [f32; 3] {
    [values.first().copied().unwrap_or(0.0), values.get(1).copied().unwrap_or(0.0), values.get(2).copied().unwrap_or(0.0)]
}

fn surrounding(entries: &[Entry], hour: f32) -> Option<(&Entry, &Entry, f32)> {
    if entries.is_empty() {
        return None;
    }
    if entries.len() == 1 {
        return Some((&entries[0], &entries[0], 0.0));
    }
    let hour = hour.rem_euclid(24.0);
    for pair in entries.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        if hour >= a.t && hour < b.t {
            let f = (hour - a.t) / (b.t - a.t);
            return Some((a, b, f));
        }
    }
    if hour < entries[0].t {
        Some((&entries[0], &entries[0], 0.0))
    } else {
        let last = &entries[entries.len() - 1];
        Some((last, last, 0.0))
    }
}

fn lerp(a: f32, b: f32, f: f32) -> f32 {
    a + (b - a) * f
}

fn lerp3(a: [f32; 3], b: [f32; 3], f: f32) -> [f32; 3] {
    [lerp(a[0], b[0], f), lerp(a[1], b[1], f), lerp(a[2], b[2], f)]
}

pub fn fgethsv(h: u8, s: u8, v: u8) -> [f32; 3] {
    let hf = h as f32;
    let mut b = v as f32 * 1.4 / 255.0;
    b = b / (b.sqrt() + 0.01) * 0.7;
    b = b.clamp(0.0, 1.0);
    let hue = if h < 86 {
        [(85.0 - hf) / 85.0, hf / 85.0, 0.0]
    } else if h < 171 {
        [0.0, (170.0 - hf) / 85.0, (hf - 85.0) / 85.0]
    } else {
        [(hf - 170.0) / 85.0, 0.0, (255.0 - hf) / 84.0]
    };
    let sf = s as f32 / 255.0;
    [
        (hue[0] + (1.0 - hue[0]) * sf) * b,
        (hue[1] + (1.0 - hue[1]) * sf) * b,
        (hue[2] + (1.0 - hue[2]) * sf) * b,
    ]
}

fn parse_entry(inner: &str) -> Option<Entry> {
    let pairs = scan_pairs(inner);
    let mut t = None;
    let mut values = Vec::new();
    for (key, value) in pairs {
        if key.eq_ignore_ascii_case("T") {
            t = Some(value);
        } else {
            values.push(value);
        }
    }
    if values.is_empty() {
        return None;
    }
    Some(Entry { t: t?, values })
}

fn scan_pairs(s: &str) -> Vec<(String, f32)> {
    let chars: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_ascii_alphabetic() {
            i += 1;
            continue;
        }
        let key_start = i;
        while i < chars.len() && chars[i].is_ascii_alphabetic() {
            i += 1;
        }
        let key: String = chars[key_start..i].iter().collect();
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() || chars[i] != '=' {
            continue;
        }
        i += 1;
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        let value_start = i;
        if i < chars.len() && chars[i] == '-' {
            i += 1;
        }
        while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
            i += 1;
        }
        if let Ok(value) = chars[value_start..i].iter().collect::<String>().parse::<f32>() {
            out.push((key, value));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = "\
[StaticMeshAmbient]
NUM=25
;noite
COLOR9=(T=8,R=67,G=71,B=88)
COLOR10=(T=9,R=120,G=120,B=120)
;centro
COLOR11=(T=10,R=120,G=120,B=120)

[HSVActorLight]
NUM=25
;noite
Light1=( T=0, Hue=138, Sat=182, Bri=0 )
Light2=( T=1, Hue=138, Sat=182, Bri=120 )
Light3=( T=2, Hue=138, Sat=100, Bri=120 )

[HSVStaticMeshLight]
NUM=25
Light10=( T=9, Hue=29, Sat=200, Bri=180)
Light11=( T=10  Hue=1, Sat=255, Bri=180 )
Light12=( T=11, Hue=1, Sat=255, Bri=180 )
";

    #[test]
    fn parser_reads_colors_and_lights_from_real_sections() {
        let env = TimeEnv::parse(FIXTURE);
        let colors = env.sections.get("STATICMESHAMBIENT").expect("seção STATICMESHAMBIENT");
        assert_eq!(colors.len(), 3);
        assert_eq!(colors[0].t, 8.0);
        assert_eq!(colors[0].values, [67.0, 71.0, 88.0]);
        assert_eq!(colors[1].t, 9.0);
        assert_eq!(colors[1].values, [120.0, 120.0, 120.0]);
        let lights = env.sections.get("HSVACTORLIGHT").expect("seção HSVACTORLIGHT");
        assert_eq!(lights.len(), 3);
        assert_eq!(lights[1].t, 1.0);
        assert_eq!(lights[1].values, [138.0, 182.0, 120.0]);
        let missing_comma = env.sections.get("HSVSTATICMESHLIGHT").expect("seção HSVSTATICMESHLIGHT");
        assert_eq!(missing_comma.len(), 3);
        assert_eq!(missing_comma[1].t, 10.0);
        assert_eq!(missing_comma[1].values, [1.0, 255.0, 180.0]);
    }

    #[test]
    fn ambient_lerps_between_the_hours_around_it() {
        let env = TimeEnv::parse(FIXTURE);
        let rgb = env.ambient(AmbientKind::StaticMesh, 9.5);
        for c in rgb {
            assert!((c - 120.0 / 255.0).abs() < 1e-3, "{rgb:?}");
        }
    }

    const EDGES_FIXTURE: &str = "\
[StaticMeshAmbient]
NUM=2
COLOR1=(T=6,R=90,G=91,B=92)
COLOR2=(T=18,R=10,G=11,B=12)
";

    const SKY_FIXTURE: &str = "\
[SunColor]
Color1=(T=6,R=255,G=122,B=23)
Color2=(T=8,R=255,G=219,B=151)
Color3=(T=15,R=255,G=219,B=151)

[SunScale]
Scale1=(T=6,S=2.0)
Scale2=(T=8,S=0.5)
Scale3=(T=15,S=0.2)
Scale4=(T=20,S=0.2)
Scale5=(T=24,S=2.0)

[MoonScale]
Scale1=(T=0,S=4.5)
Scale2=(T=2,S=3.5)
Scale3=(T=3,S=2.5)
Scale4=(T=4,S=3.5)
Scale5=(T=6,S=4.5)

[SkyBoxColor]
COLOR11=(T=10,R=81,G=147,B=206)
COLOR12=(T=11,R=66,G=124,B=176)
COLOR13=(T=12,R=66,G=124,B=176)

[HazeringColor]
COLOR1=(T=0,R=48,G=62,B=84)
COLOR2=(T=1,R=48,G=62,B=84)

[CloudColor2]
COLOR1=(T=0,R=10,G=20,B=30)
";

    #[test]
    fn sky_scales_follow_get_scale_and_stay_unset_outside_the_table() {
        let env = TimeEnv::parse(SKY_FIXTURE);
        assert!((env.sun_scale(12.0).unwrap() - 0.329).abs() < 1e-3);
        assert_eq!(env.sun_scale(3.0), None);
        assert_eq!(env.sun_scale(6.0), Some(2.0));
        assert_eq!(env.moon_scale(0.0), Some(4.5));
        assert_eq!(env.moon_scale(23.0), None);
        assert_eq!(env.sun_color(8.0), [255.0, 219.0, 151.0]);
        assert_eq!(env.sky_box_color(12.0), [66.0, 124.0, 176.0]);
        assert_eq!(env.sky_box_color(10.5), [73.5, 135.5, 191.0]);
        assert_eq!(env.hazering_color(0.5), [48.0, 62.0, 84.0]);
        assert_eq!(env.cloud_color(1, 5.0), [10.0, 20.0, 30.0]);
        assert_eq!(env.cloud_color(0, 5.0), [0.0; 3]);
        assert_eq!([phase(0.0), phase(6.0), phase(8.0), phase(21.9), phase(22.0), phase(23.99)], [0, 1, 2, 2, 3, 3]);
    }

    #[test]
    fn ambient_before_the_first_hour_clamps_to_the_first_entry() {
        let env = TimeEnv::parse(EDGES_FIXTURE);
        let rgb = env.ambient(AmbientKind::StaticMesh, 3.0);
        assert_eq!(rgb, [90.0 / 255.0, 91.0 / 255.0, 92.0 / 255.0]);
    }

    #[test]
    fn ambient_after_the_last_hour_clamps_to_the_last_entry() {
        let env = TimeEnv::parse(EDGES_FIXTURE);
        let rgb = env.ambient(AmbientKind::StaticMesh, 22.0);
        assert_eq!(rgb, [10.0 / 255.0, 11.0 / 255.0, 12.0 / 255.0]);
    }

    #[test]
    fn sun_light_matches_fgethsv_times_brightness_at_an_exact_hour() {
        let env = TimeEnv::parse(FIXTURE);
        let (color, brightness) = env.sun_light(LightKind::Actor, 1.0);
        let expected = fgethsv(138, 182, 255);
        for i in 0..3 {
            assert!((color[i] - expected[i]).abs() < 1e-4, "{color:?} vs {expected:?}");
        }
        assert!((brightness - 120.0 / 255.0).abs() < 1e-4);
        for i in 0..3 {
            assert!((color[i] * brightness - expected[i] * 120.0 / 255.0).abs() < 1e-4);
        }
    }

    #[test]
    fn fgethsv_magnitude_at_full_saturation_and_value_is_documented() {
        for h in [0u8, 40, 100, 138, 200, 255] {
            let rgb = fgethsv(h, 255, 255);
            for c in rgb {
                assert!((c - 0.8213).abs() < 1e-3, "h={h} rgb={rgb:?}");
            }
        }
    }

    #[test]
    fn decode_text_reads_ascii_and_utf16le_with_bom_the_same_way() {
        let ascii: Vec<u8> = FIXTURE.bytes().collect();
        let mut utf16: Vec<u8> = vec![0xff, 0xfe];
        utf16.extend(FIXTURE.encode_utf16().flat_map(u16::to_le_bytes));
        let env_ascii = TimeEnv::parse(&decode_text(&ascii));
        let env_utf16 = TimeEnv::parse(&decode_text(&utf16));
        assert_eq!(env_ascii.sections.get("STATICMESHAMBIENT"), env_utf16.sections.get("STATICMESHAMBIENT"));
        assert_eq!(env_ascii.sections.get("HSVACTORLIGHT"), env_utf16.sections.get("HSVACTORLIGHT"));
    }

    fn test_client() -> Option<Client> {
        let root = std::env::var_os("KILLER_CLIENT_DIR").map(std::path::PathBuf::from)?;
        Client::open(&root).ok()
    }

    #[test]
    #[ignore]
    fn giran_timeenv0_has_twenty_five_static_mesh_ambient_entries() {
        let Some(mut client) = test_client() else { return };
        let env = TimeEnv::load(&mut client, 0).unwrap();
        let entries = env.sections.get("STATICMESHAMBIENT").expect("seção STATICMESHAMBIENT");
        assert_eq!(entries.len(), 25);
    }
}
