use crate::client::{AssetError, Client};
use std::collections::HashMap;

pub const L2_INI_RENDER: &str = include_str!("l2.ini.render.txt");
pub const ENV_INT_FIXTURE: &str = include_str!("env.int.txt");

const RANGE_MIN: f32 = 1.0;
const RANGE_MAX: f32 = 16.0;
const RANGE_UNITS: f32 = 2048.0;
const FOG_LEVELS: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClippingRange {
    pub actor: f32,
    pub static_mesh: f32,
    pub static_mesh_lod: f32,
    pub pawn: f32,
    pub terrain: f32,
    pub projector: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FogRanges {
    pub start: [f32; FOG_LEVELS],
    pub end: [f32; FOG_LEVELS],
    pub speed: f32,
}

/// `env.int [EnvSetup]`: the color modifiers `UpdateSkyColors` tints every frame.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SkyNames {
    pub skybox: String,
    pub hazering: String,
    pub clouds: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientOptions {
    pub clipping: ClippingRange,
    pub keep_min_frame_rate: bool,
    pub render_deco: bool,
    pub post_proc: u8,
    pub pawn_shadow: bool,
    pub projectors: bool,
    pub water_effect: bool,
    pub water_effect_type: u8,
    pub seamless: bool,
    pub l2_mark: i32,
    pub shared_sky: bool,
    pub coronas: bool,
    pub no_fractal_anim: bool,
    pub thread_filter: bool,
    pub purge_level: bool,
    pub max_pawn_resource_load: f32,
    pub fog: FogRanges,
    pub slope_sun_angle: f32,
    pub start_time: f32,
    pub sky: SkyNames,
    pub enchant_mesh_show: Option<i32>,
    pub enchant_mesh_texture: String,
    pub hero_weapon_texture: String,
    pub enchant_mesh_colors: HashMap<u8, [[u8; 3]; 2]>,
    pub enchant_effect_show: Option<i32>,
    pub enchant_effect_particles: HashMap<u8, EnchantParticles>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnchantParticles {
    pub opacity: f32,
    pub num: f32,
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self::parse("", L2_INI_RENDER, ENV_INT_FIXTURE)
    }
}

impl ClientOptions {
    pub fn load(client: &mut Client) -> Result<Self, AssetError> {
        let option_ini = client.read_system_text("option.ini")?;
        let l2_ini = client.read_system_text("l2.ini").unwrap_or_else(|_| L2_INI_RENDER.to_owned());
        let env_int = client.read_system_text("env.int")?;
        Ok(Self::parse(&option_ini, &l2_ini, &env_int))
    }

    pub fn parse(option_ini: &str, l2_ini: &str, env_int: &str) -> Self {
        let option = Ini::parse(option_ini);
        let l2 = Ini::parse(l2_ini);
        let env = Ini::parse(env_int);
        let fog_at = |prefix: &str| {
            let mut values = [0.0; FOG_LEVELS];
            for (level, value) in values.iter_mut().enumerate() {
                *value = env.float("FOG", &format!("{prefix}{}", level + 1), 0.0);
            }
            values
        };
        Self {
            clipping: ClippingRange {
                actor: option.float("ClippingRange", "Actor", 1.0),
                static_mesh: option.float("ClippingRange", "StaticMesh", 1.0),
                static_mesh_lod: option.float("ClippingRange", "StaticMeshLod", 1.0),
                pawn: option.float("ClippingRange", "Pawn", 1.0),
                terrain: option.float("ClippingRange", "Terrain", 1.0),
                projector: l2.float("ClippingRange", "Projector", 0.2),
            },
            keep_min_frame_rate: option.bool("Video", "IsKeepMinFrameRate", false),
            render_deco: option.bool("Video", "RenderDeco", false),
            post_proc: option.int("Video", "PostProc", 0) as u8,
            pawn_shadow: option.bool("Video", "PawnShadow", false),
            projectors: l2.bool("WinDrv.WindowsClient", "Projectors", true),
            water_effect: option.bool("L2WaterEffect", "IsUseEffect", false),
            water_effect_type: option.int("L2WaterEffect", "EffectType", 0) as u8,
            seamless: l2.bool("URL", "IsL2Seamless", false),
            l2_mark: l2.get("URL", "IsL2Mark").map_or(0, app_atoi),
            shared_sky: l2.bool("URL", "SharedSky", false),
            coronas: l2.bool("WinDrv.WindowsClient", "Coronas", true),
            no_fractal_anim: l2.bool("WinDrv.WindowsClient", "NoFractalAnim", false),
            thread_filter: l2.int("URL", "L2ThreadFilter", 0) != 0,
            purge_level: l2.bool("URL", "EnablePurgeLevel", false),
            max_pawn_resource_load: l2.float("URL", "L2MaxPawnResourceLoad", 0.0),
            fog: FogRanges { start: fog_at("StartRange"), end: fog_at("EndRange"), speed: env.float("FOG", "FogSpeed", 1.0) },
            slope_sun_angle: env.float("EnvSetup", "SlopeSunAngle", 0.0),
            start_time: env.float("EnvSetup", "StartTime", 0.0),
            sky: SkyNames {
                skybox: env.text("EnvSetup", "SKYBOX"),
                hazering: env.text("EnvSetup", "HAZERING"),
                clouds: (1..=env.int("EnvSetup", "NumCloudColor", 0).max(0)).map(|i| env.text("EnvSetup", &format!("CLOUD{i}"))).collect(),
            },
            enchant_mesh_show: env.get("EnchantEffect", "EnchantMeshShow").and_then(|v| v.parse().ok()),
            enchant_mesh_texture: env.get("EnchantEffect", "CubeTexName").unwrap_or("LineageEffectsTextures.Etc.Enchant_Aura001_Shader01").to_owned(),
            hero_weapon_texture: env.get("EnchantEffect", "HeroWeaponTex").unwrap_or("LineageEffectsTextures.Etc.Enchant_Aura001_Shader01").to_owned(),
            enchant_mesh_colors: (0..=255).map(|level| {
                let value = env.get("EnchantEffect", &format!("Enchant{level}")).or_else(|| env.get("EnchantEffect", "Enchant"));
                (level as u8, enchant_colors(value))
            }).collect(),
            enchant_effect_show: env.get("EnchantEffect", "EnchantEffectShow").and_then(|v| v.parse().ok()),
            enchant_effect_particles: (0..=255).map(|level| {
                let value = env.get("EnchantEffect", &format!("Enchant{level}")).or_else(|| env.get("EnchantEffect", "Enchant"));
                (level as u8, enchant_particles(value))
            }).collect(),
        }
    }

    pub fn fog_range(&self) -> (f32, f32) {
        let level = fog_index(self.clipping.terrain, self.keep_min_frame_rate) - 1;
        (self.fog.start[level] * RANGE_UNITS, self.fog.end[level] * RANGE_UNITS)
    }
}

fn enchant_colors(value: Option<&str>) -> [[u8; 3]; 2] {
    let Some(value) = value else { return [[0; 3]; 2] };
    let value: String = value.chars().take(63).collect();
    let value = value.to_ascii_lowercase();
    [["r1=", "g1=", "b1="], ["r2=", "g2=", "b2="]].map(|keys| keys.map(|key| {
        value.match_indices(key).find(|(at, _)| *at == 0 || !value.as_bytes()[at - 1].is_ascii_alphanumeric()).map_or(100, |(at, _)| {
            let number = value[at + key.len()..].trim_start_matches(|c: char| c.is_ascii_whitespace());
            let sign = usize::from(number.starts_with(['-', '+']));
            let digits = number[sign..].bytes().take_while(u8::is_ascii_digit).count();
            number[..sign + digits].parse::<i32>().unwrap_or(0) as u8
        })
    }))
}

fn enchant_particles(value: Option<&str>) -> EnchantParticles {
    let Some(value) = value else { return EnchantParticles { opacity: 1.0, num: 1.0 } };
    let value: String = value.chars().take(63).collect();
    EnchantParticles { opacity: parse_float(&value, "Opacity=").unwrap_or(1.0), num: parse_float(&value, "Num=").unwrap_or(1.0) }
}

fn parse_float(stream: &str, key: &str) -> Option<f32> {
    let lower = stream.to_ascii_lowercase();
    let key = key.to_ascii_lowercase();
    let (at, _) = lower.match_indices(&key).find(|(at, _)| *at == 0 || !lower.as_bytes()[at - 1].is_ascii_alphanumeric())?;
    Some(atof(&stream[at + key.len()..]))
}

fn atof(text: &str) -> f32 {
    let text = text.trim_start_matches([' ', '\t', '\n', '\r', '\x0b', '\x0c']);
    let bytes = text.as_bytes();
    let mut end = usize::from(bytes.first().is_some_and(|b| matches!(b, b'+' | b'-')));
    let integer = bytes[end..].iter().take_while(|b| b.is_ascii_digit()).count();
    end += integer;
    let mut fraction = 0;
    if bytes.get(end) == Some(&b'.') {
        fraction = bytes[end + 1..].iter().take_while(|b| b.is_ascii_digit()).count();
        if integer + fraction > 0 {
            end += 1 + fraction;
        }
    }
    if integer + fraction == 0 {
        return 0.0;
    }
    if matches!(bytes.get(end), Some(b'e' | b'E')) {
        let sign = usize::from(bytes.get(end + 1).is_some_and(|b| matches!(b, b'+' | b'-')));
        let digits = bytes[(end + 1 + sign).min(bytes.len())..].iter().take_while(|b| b.is_ascii_digit()).count();
        if digits > 0 {
            end += 1 + sign + digits;
        }
    }
    text[..end].parse::<f64>().unwrap_or(0.0) as f32
}

pub fn range_units(clipping_range: f32) -> f32 {
    clipping_range.clamp(RANGE_MIN, RANGE_MAX) * RANGE_UNITS
}

pub fn fog_index(terrain_range: f32, keep_min_frame_rate: bool) -> usize {
    if keep_min_frame_rate || terrain_range < 5.0 {
        1
    } else if terrain_range < 6.0 {
        2
    } else if terrain_range < 7.0 {
        3
    } else if terrain_range < 8.0 {
        4
    } else {
        5
    }
}

const DIGIT_BASES: [u32; 18] = [0x30,0x660,0x6f0,0x966,0x9e6,0xa66,0xae6,0xb66,0xc66,0xce6,0xd66,0xe50,0xed0,0xf20,0x1040,0x17e0,0x1810,0xff10];

pub fn app_atoi(text: &str) -> i32 {
    let units: Vec<u32> = text.encode_utf16().map(u32::from).collect();
    let mut index = units.iter().take_while(|&&unit| matches!(unit,0x09..=0x0d|0x20|0xa0)).count();
    let negative = units.get(index) == Some(&0x2d);
    if matches!(units.get(index),Some(0x2b | 0x2d)) { index += 1; }
    let mut value: i32 = 0;
    while let Some(digit) = units.get(index).and_then(|&unit| DIGIT_BASES.iter().find(|&&base| (base..base+10).contains(&unit)).map(|&base| unit-base)) {
        value = value.wrapping_mul(10).wrapping_add(digit as i32);
        index += 1;
    }
    if negative { value.wrapping_neg() } else { value }
}

pub(crate) struct Ini {
    values: HashMap<(String, String), String>,
}

impl Ini {
    pub(crate) fn parse(text: &str) -> Self {
        let mut values = HashMap::new();
        let mut section = String::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            if let Some(name) = line.strip_prefix('[').and_then(|rest| rest.strip_suffix(']')) {
                section = name.trim().to_ascii_lowercase();
            } else if let Some((key, value)) = line.split_once('=') {
                values.insert((section.clone(), key.trim().to_ascii_lowercase()), value.trim().to_owned());
            }
        }
        Self { values }
    }

    pub(crate) fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.values.get(&(section.to_ascii_lowercase(), key.to_ascii_lowercase())).map(String::as_str)
    }

    pub(crate) fn text(&self, section: &str, key: &str) -> String {
        self.get(section, key).unwrap_or_default().to_owned()
    }

    fn float(&self, section: &str, key: &str, default: f32) -> f32 {
        self.get(section, key).and_then(|value| value.parse().ok()).unwrap_or(default)
    }

    pub(crate) fn int(&self, section: &str, key: &str, default: i32) -> i32 {
        self.get(section, key).and_then(|value| value.parse().ok()).unwrap_or(default)
    }

    pub(crate) fn bool(&self, section: &str, key: &str, default: bool) -> bool {
        match self.get(section, key).map(str::to_ascii_lowercase).as_deref() {
            Some("true" | "1") => true,
            Some("false" | "0") => false,
            _ => default,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_atoi_follows_the_crt_wtol() {
        for (text,value) in [("2",2),("1tail",1),("+01",1),("1.5",1),("4294967297",1),("-1",-1),("TrueX",0),("",0),(" \u{a0}7",7),("\u{0662}",2)] {
            assert_eq!(app_atoi(text),value,"{text:?}");
        }
        assert_eq!(ClientOptions::parse("", "[URL]\nISL2Mark=2\n", "").l2_mark, 2);
        assert_eq!(ClientOptions::parse("", "", "").l2_mark, 0);
    }

    #[test]
    fn enchant_material_colors_keep_native_defaults_and_first_match() {
        assert_eq!(enchant_colors(None), [[0; 3]; 2]);
        assert_eq!(enchant_colors(Some("(Opacity=0,Num=6.5)")), [[100; 3]; 2]);
        assert_eq!(enchant_colors(Some("(R1=1,G1=2,B1=3,R1=9,G1=8,B1=7)")), [[1, 2, 3], [100; 3]]);
        assert_eq!(enchant_colors(Some("(XR1=99,R1=+12xyz,G1=-1.5,B1=no)")), [[12, 255, 0], [100; 3]]);
        let options = ClientOptions::parse("", "", "[EnchantEffect]\nEnchant=(R1=4,G1=5,B1=6,R2=7,G2=8,B2=9)\nEnchant20=(R1=11)\nEnchantMeshShow=-1\n");
        assert_eq!(options.enchant_mesh_colors[&19], [[4, 5, 6], [7, 8, 9]]);
        assert_eq!(options.enchant_mesh_colors[&20], [[11, 100, 100], [100; 3]]);
        assert_eq!(options.enchant_mesh_show, Some(-1));
        assert_eq!(ClientOptions::parse("", "", "").enchant_mesh_show, None);
    }

    const OPTION_INI: &str = "[Video]\nGamePlayViewportX=1558\nIsKeepMinFrameRate=false\nPostProc=0\nRenderDeco=false\nPawnShadow=true\n\n[ClippingRange]\nPawn=1.500000\nTerrain=8.000000\nActor=6.000000\nStaticMesh=4.000000\nStaticMeshLod=6.000000\n\n[L2WaterEffect]\nIsUseEffect=false\nEffectType=0\n";

    #[test]
    fn option_ini_values_land_in_the_struct() {
        let options = ClientOptions::parse(OPTION_INI, L2_INI_RENDER, ENV_INT_FIXTURE);
        assert_eq!(options.clipping.actor, 6.0);
        assert_eq!(options.sky.skybox, "SkybackgroundColor");
        assert_eq!(options.sky.hazering, "HazeRing_Final");
        assert_eq!(options.sky.clouds, ["Cloud_Final", "StarField_Final01", "StarField_Final02"]);
        assert_eq!(options.clipping.static_mesh, 4.0);
        assert_eq!(options.clipping.static_mesh_lod, 6.0);
        assert_eq!(options.clipping.pawn, 1.5);
        assert_eq!(options.clipping.terrain, 8.0);
        assert_eq!(options.clipping.projector, 0.2);
        assert!(!options.keep_min_frame_rate);
        assert!(!options.render_deco);
        assert_eq!(options.post_proc, 0);
        assert!(options.pawn_shadow);
        assert!(!options.water_effect);
        assert!(!options.no_fractal_anim);
        assert!(ClientOptions::parse(OPTION_INI,"[WinDrv.WindowsClient]\nNoFractalAnim=True",ENV_INT_FIXTURE).no_fractal_anim);
        assert_eq!(options.water_effect_type, 0);
        assert!(options.seamless);
        assert!(options.shared_sky);
        assert!(!options.thread_filter);
        assert!(options.purge_level);
        assert_eq!(options.max_pawn_resource_load, 1.0);
        assert_eq!(options.fog.start, [1.0, 1.5, 2.0, 2.5, 4.0]);
        assert_eq!(options.fog.end, [4.0, 5.0, 6.0, 7.0, 10.0]);
        assert_eq!(options.fog.speed, 1.0);
        assert_eq!(options.slope_sun_angle, 30.0);
        assert_eq!(options.start_time, 12.0);
    }

    #[test]
    fn ranges_scale_by_2048_after_the_engine_clamp() {
        assert_eq!(range_units(6.0), 12288.0);
        assert_eq!(range_units(0.5), 2048.0);
        assert_eq!(range_units(20.0), 32768.0);
    }

    #[test]
    fn fog_index_follows_the_terrain_range_unless_the_frame_rate_is_kept() {
        assert_eq!(fog_index(8.0, false), 5);
        assert_eq!(fog_index(8.0, true), 1);
        assert_eq!(fog_index(6.5, false), 3);
        assert_eq!(fog_index(4.9, false), 1);
        assert_eq!(fog_index(5.0, false), 2);
        assert_eq!(fog_index(7.0, false), 4);
    }

    #[test]
    fn the_fog_ranges_in_use_come_from_the_selected_index() {
        let options = ClientOptions::parse(OPTION_INI, L2_INI_RENDER, ENV_INT_FIXTURE);
        assert_eq!(options.fog_range(), (8192.0, 20480.0));
    }

    #[test]
    fn enchant_mesh_colors_use_native_first_match_and_missing_component_defaults() {
        let options = ClientOptions::default();
        assert_eq!(options.enchant_mesh_colors[&4], [[91, 112, 255], [66, 99, 255]]);
        assert_eq!(options.enchant_mesh_colors[&20], [[255, 0, 0], [195, 0, 0]]);
        assert_eq!(options.enchant_mesh_colors[&50][1], [100; 3]);
    }

    #[test]
    fn weapon_effect_init_10493790_reads_opacity_and_num_like_parse_10155100() {
        let options = ClientOptions::default();
        assert_eq!(options.enchant_effect_show, Some(1));
        assert_eq!(options.enchant_effect_particles[&4], EnchantParticles { opacity: 1.0, num: 0.0 });
        assert_eq!(options.enchant_effect_particles[&8], EnchantParticles { opacity: 0.5, num: 6.5 });
        assert_eq!(options.enchant_effect_particles[&15], EnchantParticles { opacity: 0.65, num: 6.5 });
        assert_eq!(options.enchant_effect_particles[&50], EnchantParticles { opacity: 0.0, num: 6.5 });
        assert_eq!(enchant_particles(None), EnchantParticles { opacity: 1.0, num: 1.0 });
        assert_eq!(enchant_particles(Some("(R1=1)")), EnchantParticles { opacity: 1.0, num: 1.0 });
        assert_eq!(enchant_particles(Some("(XOpacity=3,Num= 2.5e1x)")), EnchantParticles { opacity: 1.0, num: 25.0 });
        assert_eq!(enchant_particles(Some("(Opacity=abc,Num=.5)")), EnchantParticles { opacity: 0.0, num: 0.5 });
        let missing = ClientOptions::parse("", "", "[EnchantEffect]\nEnchant=(Opacity=0.25)\n");
        assert_eq!(missing.enchant_effect_particles[&9], EnchantParticles { opacity: 0.25, num: 1.0 });
        assert_eq!(missing.enchant_effect_show, None);
    }
}
