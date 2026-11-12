use super::{AudioVolumes, SourceIdentity};

pub trait AudioConfigReader {
    fn float(&self, file: &str, section: &str, key: &str) -> Option<f32>;
    fn integer(&self, file: &str, section: &str, key: &str) -> Option<i32>;
    fn boolean(&self, file: &str, section: &str, key: &str) -> Option<bool>;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InitConfig {
    pub volumes: AudioVolumes,
    pub use_ambient_slot: bool,
    pub ambient_sound_slot: i32,
    pub use_voice_slot: bool,
    pub voice_sound_slot: i32,
    pub use_release: bool,
}

impl InitConfig {
    pub fn read(config: &impl AudioConfigReader) -> Self {
        Self {
            volumes: AudioVolumes {
                sound: config.float("Option.ini", "Audio", "SoundVolume").unwrap_or(0.8),
                music: config.float("Option.ini", "Audio", "MusicVolume").unwrap_or(1.0),
                ogg_voice: config.float("Option.ini", "Audio", "OggVoiceVolume").unwrap_or(0.6),
                wav_voice: config.float("Option.ini", "Audio", "WavVoiceVolume").unwrap_or(0.6),
            },
            use_ambient_slot: config.boolean("Alaudio.int", "AmbientSound", "UseAmbientSlot").unwrap_or(false),
            ambient_sound_slot: config.integer("Alaudio.int", "AmbientSound", "AmbientSoundSlot").unwrap_or(0),
            use_voice_slot: config.boolean("Alaudio.int", "VoiceSound", "UseVoiceSlot").unwrap_or(false),
            voice_sound_slot: config.integer("Alaudio.int", "VoiceSound", "VoiceSoundSlot").unwrap_or(0),
            use_release: config.boolean("Alaudio.int", "SoundResource", "UseRelease").unwrap_or(true),
        }
    }
}

pub fn source_slot(config: InitConfig, sources: impl ExactSizeIterator<Item=Option<(u32,f32)>>, priority: f32) -> Option<usize> {
    let start=if config.use_ambient_slot {config.ambient_sound_slot.clamp(0,sources.len() as i32) as usize}else{0};
    let mut candidate=None;
    let mut lowest=priority;
    for (index,existing) in sources.enumerate().skip(start) {
        let existing_priority=existing.map_or(0.0,|(_,priority)|priority);
        if config.use_ambient_slot && existing.is_some_and(|(flags,_)|flags&8!=0) {continue;}
        if existing_priority<lowest {candidate=Some(index);lowest=existing_priority;}
    }
    candidate
}

pub fn native_clamp(value: f32, lower: f32, upper: f32) -> f32 {
    if value < lower {
        lower
    } else if value < upper {
        value
    } else {
        upper
    }
}

pub fn update_delta(now: f64, previous: f64) -> f32 {
    let difference = now - previous;
    if difference < f64::from(0.0001_f32) { 0.0001 } else { native_clamp(difference as f32, 0.0001, 1.0) }
}

pub fn channel_gain(identity: SourceIdentity, volumes: AudioVolumes) -> Option<f32> {
    if identity.flags & 8 != 0 {
        Some(volumes.music)
    } else if identity.flags & 0x100 != 0 {
        Some(if identity.flags & 4 != 0 { volumes.ogg_voice } else { volumes.wav_voice })
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FadeKind {
    None,
    In,
    Out,
    Relative,
    Restore,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceGain {
    pub base: f32,
    pub adjustment: f32,
    pub duration: f32,
    pub elapsed: f32,
    pub fade: FadeKind,
}

impl SourceGain {
    pub fn new(base: f32, fade: f32) -> Self {
        Self {
            base,
            adjustment: 0.0,
            duration: fade,
            elapsed: 0.0,
            fade: if fade > 0.0 { FadeKind::In } else { FadeKind::None },
        }
    }

    pub fn initial_gain(self) -> f32 {
        if self.fade == FadeKind::In { 0.0 } else { self.base }
    }

    pub fn stop(&mut self, fade: f32) {
        self.elapsed = 0.0;
        self.duration = fade;
        self.fade = FadeKind::Out;
    }

    pub fn play_music_adjustment(&mut self, volume: f32) {
        if volume != 1.0 {
            self.fade = FadeKind::Relative;
            self.adjustment = native_clamp((volume - 1.0) * self.base, -1.0, 1.0);
            self.elapsed = 1.0;
            self.duration = 1.0;
        }
    }

    pub fn set_music_volume(&mut self, music: f32, volume: f32, fade: f32) {
        if self.fade == FadeKind::Out {
            return;
        }
        if volume == 1.0 {
            if self.fade != FadeKind::Relative {
                return;
            }
            self.fade = FadeKind::Restore;
            self.adjustment += music;
        } else {
            self.fade = FadeKind::Relative;
            self.adjustment = native_clamp((volume - 1.0) * self.base, -1.0, 1.0);
        }
        self.elapsed = 0.0;
        self.duration = fade;
    }

    fn advance(&mut self, delta: f32) {
        if delta < 1.0 {
            self.elapsed += delta;
        }
    }

    pub fn update(&mut self, delta: f32, channel: Option<f32>, attenuation: Option<(f32, f32)>) -> Option<f32> {
        match self.fade {
            FadeKind::In => {
                self.advance(delta);
                if self.elapsed >= self.duration {
                    self.fade = FadeKind::None;
                }
            }
            FadeKind::Out => {
                self.advance(delta);
                if self.elapsed >= self.duration {
                    return None;
                }
            }
            _ => {}
        }
        if let Some((distance, radius)) = attenuation {
            return Some(native_clamp(((radius - distance) * self.base) / radius, 0.0, 1.0));
        }
        let base = channel.unwrap_or(self.base);
        let value = match self.fade {
            FadeKind::None => base,
            FadeKind::In => (self.elapsed / self.duration) * base,
            FadeKind::Out => (1.0 - self.elapsed / self.duration) * base,
            FadeKind::Relative => {
                self.advance(delta);
                if self.elapsed >= self.duration {
                    base + self.adjustment
                } else {
                    (self.adjustment / self.duration) * self.elapsed + base
                }
            }
            FadeKind::Restore => {
                self.advance(delta);
                if self.elapsed >= self.duration {
                    self.fade = FadeKind::None;
                    base
                } else {
                    ((base - self.adjustment) / self.duration) * self.elapsed + self.adjustment
                }
            }
        };
        Some(native_clamp(value, 0.0, 1.0))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlayingChannels {
    pub music: bool,
    pub ogg_voice: bool,
    pub wav_voice: bool,
    pub ogg_voice_finished: bool,
}

impl PlayingChannels {
    pub fn observe(&mut self, flags: u32) {
        if flags & 8 != 0 {
            self.music = true;
        } else if flags & 0x100 != 0 {
            if flags & 4 != 0 {
                self.ogg_voice = true;
            } else {
                self.wav_voice = true;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AudioFrame {
    pub time: f64,
    pub view_actor_location: [f32; 3],
    pub max_radius_multiplier: f32,
    pub realtime: bool,
    pub replay_recording: bool,
}

pub fn nonspatial_priority(frame: AudioFrame, radius: f32, gain: f32, flags: u32) -> f32 {
    sound_priority(frame,[0.0;3],radius,gain,flags)
}

pub fn sound_priority(frame: AudioFrame, position: [f32;3], radius: f32, gain: f32, flags: u32) -> f32 {
    let distance_squared = frame.view_actor_location.into_iter().zip(position).map(|(listener,source)| (f64::from(listener)-f64::from(source)).powi(2)).sum::<f64>();
    let scaled_radius = f64::from(radius) * f64::from(frame.max_radius_multiplier);
    let attenuation = if radius == 0.0 {
        1.0
    } else {
        let value = 1.0 - distance_squared / (scaled_radius * scaled_radius);
        if value < f64::from(0.01_f32) {
            f64::from(0.01_f32)
        } else if value < 1.0 {
            value
        } else {
            1.0
        }
    };
    (attenuation * f64::from(gain)
        + f64::from((flags >> 4) & 1)
        + f64::from((flags >> 2) & 2)
        + f64::from((flags >> 2) & 1)) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EmptyConfig;
    impl AudioConfigReader for EmptyConfig {
        fn float(&self, _: &str, _: &str, _: &str) -> Option<f32> {
            None
        }
        fn integer(&self, _: &str, _: &str, _: &str) -> Option<i32> {
            None
        }
        fn boolean(&self, _: &str, _: &str, _: &str) -> Option<bool> {
            None
        }
    }

    #[test]
    fn init_native_defaults() {
        let config = InitConfig::read(&EmptyConfig);
        assert_eq!(config.volumes.ui_values(), [0.8, 1.0, 0.6, 0.6]);
        assert!(config.use_release);
        assert!(!config.use_ambient_slot && !config.use_voice_slot);
        assert_eq!((config.ambient_sound_slot, config.voice_sound_slot), (0, 0));
    }

    #[test]
    fn x87_clamps_and_delta_boundaries() {
        for (input, result) in
            [(f32::NAN, 1.0), (f32::NEG_INFINITY, 0.0), (f32::INFINITY, 1.0), (-0.2, 0.0), (0.4, 0.4)]
        {
            assert_eq!(native_clamp(input, 0.0, 1.0), result);
        }
        assert_eq!(update_delta(f64::NAN, 0.0), 1.0);
        assert_eq!(update_delta(0.0, 1.0), 0.0001);
        assert_eq!(update_delta(1.0, 0.0), 1.0);
    }

    #[test]
    fn all_channel_masks_keep_music_precedence() {
        let volumes = AudioVolumes { sound: 0.1, music: 0.2, wav_voice: 0.3, ogg_voice: 0.4 };
        for flags in 0..512 {
            let identity = SourceIdentity { flags, actor: None, sound: None };
            let expected = if flags & 8 != 0 {
                Some(0.2)
            } else if flags & 0x100 == 0 {
                None
            } else if flags & 4 != 0 {
                Some(0.4)
            } else {
                Some(0.3)
            };
            assert_eq!(channel_gain(identity, volumes), expected);
        }
    }

    #[test]
    fn fade_in_uses_current_channel_and_exact_end() {
        let mut gain = SourceGain::new(0.8, 1.0);
        assert_eq!(gain.initial_gain(), 0.0);
        assert_eq!(gain.update(0.25, Some(0.4), None), Some(0.1));
        assert_eq!(gain.update(1.0, Some(0.8), None), Some(0.2));
        assert_eq!(gain.elapsed, 0.25);
        assert_eq!(gain.update(0.75, Some(0.4), None), Some(0.4));
        assert_eq!(gain.fade, FadeKind::None);
    }

    #[test]
    fn fade_out_stops_and_spatial_gain_bypasses_fade_only_until_end() {
        let mut gain = SourceGain::new(0.8, 0.0);
        gain.stop(0.5);
        assert_eq!(gain.update(0.25, Some(0.2), Some((5.0, 10.0))), Some(0.4));
        assert_eq!(gain.update(0.25, Some(0.2), None), None);
    }

    #[test]
    fn relative_and_restore_follow_native_states() {
        let mut gain = SourceGain::new(0.8, 1.5);
        gain.play_music_adjustment(0.5);
        assert_eq!(gain.update(0.25, Some(0.8), None), Some(0.4));
        assert_eq!(gain.fade, FadeKind::Relative);
        gain.set_music_volume(0.8, 1.0, 0.5);
        assert_eq!(gain.update(0.25, Some(0.8), None), Some(0.6));
        assert_eq!(gain.update(0.25, Some(0.8), None), Some(0.8));
        assert_eq!(gain.fade, FadeKind::None);
        gain.stop(0.5);
        let before = gain;
        gain.set_music_volume(1.0, 0.2, 0.25);
        assert_eq!(gain, before);
    }

    #[test]
    fn zero_duration_relative_and_restore_do_not_divide_by_zero() {
        let mut gain = SourceGain::new(0.8, 0.0);
        gain.set_music_volume(0.8, 0.5, 0.0);
        assert_eq!(gain.update(1.0, Some(0.8), None), Some(0.4));
        gain.set_music_volume(0.8, 1.0, 0.0);
        assert_eq!(gain.update(1.0, Some(0.8), None), Some(0.8));
    }
    #[test]
    fn priority_uses_real_view_location_and_native_flags() {
        let mut frame = AudioFrame {
            time: 0.0,
            view_actor_location: [0.0; 3],
            max_radius_multiplier: 1.0,
            realtime: false,
            replay_recording: false,
        };
        assert_eq!(nonspatial_priority(frame, 1000.0, 0.8, 0x1c), 4.8);
        frame.view_actor_location = [1000.0, 0.0, 0.0];
        assert!((nonspatial_priority(frame, 1000.0, 0.8, 0x1c) - 4.008).abs() < 0.000001);
        assert_eq!(nonspatial_priority(frame, 0.0, 0.8, 0x1c), 4.8);
    }

    #[test]
    fn unordered_fade_duration_keeps_state_and_clamps_gain_high() {
        for kind in [FadeKind::In, FadeKind::Out, FadeKind::Relative, FadeKind::Restore] {
            let mut gain = SourceGain::new(0.8, 0.0);
            gain.fade = kind;
            gain.duration = f32::NAN;
            assert_eq!(gain.update(0.25, Some(0.8), None), Some(1.0));
            assert_eq!(gain.fade, kind);
        }
    }
}
