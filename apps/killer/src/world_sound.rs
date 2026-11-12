use glam::Vec3;
use killer_assets::client::Client;
use std::collections::HashMap;
use std::sync::Arc;

pub const PAWN_ACTOR: u64 = 1 << 40;
pub const VOLUME_SCALE: f32 = f32::from_bits(0x3b808081);
pub const DEFAULT_RADIUS: f32 = 80.0;

pub fn pawn_actor(object_id: i32) -> u64 {
    PAWN_ACTOR | u64::from(object_id as u32)
}

pub fn sound_key(name: &str) -> u64 {
    name.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| (hash ^ u64::from(byte.to_ascii_lowercase())).wrapping_mul(0x0000_0100_0000_01b3))
}

#[derive(Clone, Copy, Debug)]
pub struct CrtRand(u32);

impl CrtRand {
    pub fn seeded(seed: u32) -> Self {
        Self(seed)
    }

    pub fn fraction(&mut self) -> f32 {
        (f64::from(self.roll()) * f64::from(f32::from_bits(0x3800_0100))) as f32
    }

    pub fn roll(&mut self) -> i32 {
        self.0 = self.0.wrapping_mul(214013).wrapping_add(2531011);
        ((self.0 >> 16) & 0x7fff) as i32
    }
}

pub const FOOTSTEP_SECTION: &str = "L2FootStepSound";
pub const FOOTSTEP_KEYS: [&str; 8] = [
    "DefWalkSound",
    "DefRunSound",
    "GrassWalkSound",
    "GrassRunSound",
    "WaterWalkSound",
    "WaterRunSound",
    "DefActorWalkSound",
    "DefActorRunSound",
];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SwimSet {
    pub names: [Option<String>; 3],
    pub volume: f32,
    pub radius: f32,
    pub random: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EtcSound {
    pub name: String,
    pub volume: f32,
    pub radius: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SoundConfig {
    pub critical: EtcSound,
    pub soul_shot: EtcSound,
    pub item_volume: f32,
    pub item_radius: f32,
    pub char_volume: f32,
    pub char_radius: f32,
    pub char_random: i32,
    pub footsteps: [[Option<String>; 3]; 8],
    pub surface_swim: SwimSet,
    pub under_water: SwimSet,
}

impl Default for SoundConfig {
    fn default() -> Self {
        let swim = SwimSet { names: Default::default(), volume: 255.0, radius: 50.0, random: 100 };
        Self {
            critical: EtcSound { name: "SkillSound.Critical_Hit.Critical_Hit".into(), volume: 255.0, radius: 50.0 },
            soul_shot: EtcSound { name: "skillsound.soul_shot_shot".into(), volume: 255.0, radius: 50.0 },
            item_volume: 255.0,
            item_radius: 50.0,
            char_volume: 255.0,
            char_radius: 50.0,
            char_random: 100,
            footsteps: Default::default(),
            surface_swim: swim.clone(),
            under_water: swim,
        }
    }
}

impl SoundConfig {
    pub fn read(ini: &killer_ui::ini::IniFile) -> Self {
        let float = |section: &str, key: &str, default: f32| ini.get(section, key).and_then(|value| value.trim().parse::<f32>().ok()).unwrap_or(default);
        let name = |section: &str, key: &str| ini.get(section, key).map(str::trim).filter(|value| !value.is_empty()).map(str::to_owned);
        let defaults = Self::default();
        let swim = |prefix: &str, fallback: &SwimSet| SwimSet {
            names: std::array::from_fn(|index| name("SwimSound", &format!("{prefix}Sound{}", index + 1))),
            volume: float("SwimSound", &format!("{prefix}SoundVol"), fallback.volume),
            radius: float("SwimSound", &format!("{prefix}SoundRadius"), fallback.radius),
            random: float("SwimSound", &format!("{prefix}SoundRandom"), fallback.random as f32) as i32,
        };
        let etc = |key: &str, fallback: &EtcSound| EtcSound {
            name: ini.get("EtcSound", key).map_or_else(|| fallback.name.clone(), |value| value.trim().to_owned()),
            volume: float("EtcSound", &format!("{key}_Vol"), fallback.volume),
            radius: float("EtcSound", &format!("{key}_Radius"), fallback.radius),
        };
        Self {
            critical: etc("CriticalSound", &defaults.critical),
            soul_shot: etc("SoulShotSound", &defaults.soul_shot),
            item_volume: float("ItemSound", "Vol", defaults.item_volume),
            item_radius: float("ItemSound", "Radius", defaults.item_radius),
            char_volume: float("CharSound", "Vol", defaults.char_volume),
            char_radius: float("CharSound", "Radius", defaults.char_radius),
            char_random: ini.get("CharSound", "Random").and_then(|value| value.trim().parse().ok()).unwrap_or(defaults.char_random),
            footsteps: std::array::from_fn(|table| std::array::from_fn(|index| name(FOOTSTEP_SECTION, &format!("{}{}", FOOTSTEP_KEYS[table], index + 1)))),
            surface_swim: swim("WaterSurface", &defaults.surface_swim),
            under_water: swim("UnderWater", &defaults.under_water),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SoundPlan {
    pub actor: i32,
    pub name: String,
    pub position: Vec3,
    pub volume: f32,
    pub radius: f32,
    pub skip_if_playing: bool,
}

#[derive(Clone)]
pub struct WorldSound {
    pub actor: u64,
    pub key: u64,
    pub name: String,
    pub encoded: Arc<[u8]>,
    pub position: Vec3,
    pub volume: f32,
    pub radius: f32,
    pub pitch: f32,
    pub skip_if_playing: bool,
}

pub fn pawn_positions(world: &killer_world::World) -> HashMap<u64, Vec3> {
    world.me.iter().chain(world.entities.values()).map(|entity| (pawn_actor(entity.object_id), entity.position)).collect()
}

pub const SOUND_BANK_BYTES: usize = 64 * 1024 * 1024;

#[derive(Default)]
pub struct SoundBank {
    loaded: HashMap<String, Option<Arc<[u8]>>>,
    bytes: usize,
}

impl SoundBank {
    fn trim(&mut self) {
        self.loaded.retain(|_, encoded| encoded.as_ref().is_none_or(|encoded| Arc::strong_count(encoded) > 1));
        self.bytes = self.loaded.values().flatten().map(|encoded| encoded.len()).sum();
    }

    pub fn resolve(&mut self, client: &mut Client, plan: &SoundPlan) -> Option<WorldSound> {
        let key = plan.name.to_ascii_lowercase();
        if !self.loaded.contains_key(&key) && self.bytes > SOUND_BANK_BYTES {
            self.trim();
        }
        let bytes = &mut self.bytes;
        let encoded = self
            .loaded
            .entry(key)
            .or_insert_with(|| {
                let encoded = client.sound(&plan.name).map(|(_, encoded)| Arc::<[u8]>::from(encoded));
                *bytes += encoded.as_ref().map_or(0, |encoded| encoded.len());
                encoded
            })
            .clone()?;
        Some(WorldSound {
            actor: pawn_actor(plan.actor),
            key: sound_key(&plan.name),
            name: plan.name.clone(),
            encoded,
            position: plan.position,
            volume: plan.volume,
            radius: plan.radius,
            pitch: 1.0,
            skip_if_playing: plan.skip_if_playing,
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn sound_bank_trim_keeps_the_sounds_still_playing_and_the_missing_marks() {
        let mut bank = SoundBank::default();
        let playing: Arc<[u8]> = Arc::from(vec![1u8; 8]);
        bank.loaded.insert("playing".into(), Some(playing.clone()));
        bank.loaded.insert("idle".into(), Some(Arc::from(vec![2u8; 8])));
        bank.loaded.insert("missing".into(), None);
        bank.bytes = 16;
        bank.trim();
        assert_eq!(bank.bytes, 8);
        assert!(bank.loaded.contains_key("playing") && bank.loaded.contains_key("missing") && !bank.loaded.contains_key("idle"));
    }

    #[test]
    fn etc_sound_105edc30_reads_critical_and_soulshot_from_alaudio_with_the_engine_defaults() {
        let ini = killer_ui::ini::parse_ini("[EtcSound]\nCriticalSound=Mine.crit\nCriticalSound_Vol=100.0\n");
        let config = SoundConfig::read(&ini);
        assert_eq!(config.critical, EtcSound { name: "Mine.crit".into(), volume: 100.0, radius: 50.0 });
        assert_eq!(config.soul_shot, SoundConfig::default().soul_shot);
        assert_eq!(SoundConfig::default().soul_shot.name, "skillsound.soul_shot_shot");
    }

    use super::*;

    #[test]
    fn volume_scale_is_the_engine_constant_one_over_255() {
        assert_eq!(VOLUME_SCALE.to_bits(), 0x3b808081);
        assert!((VOLUME_SCALE * 255.0 - 1.0).abs() < 1e-6);
    }

    #[test]
    fn crt_rand_is_the_msvc_linear_congruential_generator() {
        let mut generator = CrtRand::seeded(1);
        assert_eq!([generator.roll(), generator.roll(), generator.roll()], [41, 18467, 6334]);
    }

    #[test]
    fn crt_fraction_is_app_frand_one_over_32767() {
        let mut generator = CrtRand::seeded(1);
        assert_eq!(generator.fraction(), (41.0_f64 * f64::from(f32::from_bits(0x3800_0100))) as f32);
    }

    #[test]
    fn sound_key_ignores_case_and_separates_actors() {
        assert_eq!(sound_key("ChrSound.Breath_1"), sound_key("chrsound.breath_1"));
        assert_ne!(sound_key("a"), sound_key("b"));
        assert_ne!(pawn_actor(5), pawn_actor(6));
        assert!(pawn_actor(5) > u64::from(u32::MAX));
    }
}
