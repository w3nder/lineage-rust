use killer_ui::option_audio::AudioChannel;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AudioVolumes {
    pub sound: f32,
    pub music: f32,
    pub wav_voice: f32,
    pub ogg_voice: f32,
}

impl AudioVolumes {
    pub fn get(self, channel: AudioChannel) -> f32 {
        match channel {
            AudioChannel::Sound => self.sound,
            AudioChannel::Music => self.music,
            AudioChannel::WavVoice => self.wav_voice,
            AudioChannel::OggVoice => self.ogg_voice,
        }
    }

    pub fn ui_values(self) -> [f32; 4] {
        [self.sound, self.music, self.wav_voice, self.ogg_voice]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceIdentity {
    pub flags: u32,
    pub actor: Option<u64>,
    pub sound: Option<u64>,
}

impl SourceIdentity {
    pub fn is_sound(self) -> bool {
        self.flags & (8 | 0x100) == 0
    }

    pub fn matches_channel(self, channel: AudioChannel) -> bool {
        match channel {
            AudioChannel::Sound => self.is_sound(),
            AudioChannel::Music => self.flags & 8 != 0,
            AudioChannel::WavVoice => self.flags & 0x100 != 0 && self.flags & 4 == 0,
            AudioChannel::OggVoice => self.flags & 0x100 != 0 && self.flags & 4 != 0,
        }
    }

    pub fn matches_sound(self, actor: Option<u64>, sound: Option<u64>) -> bool {
        match (actor, sound) {
            (None, None) => self.is_sound(),
            (Some(actor), None) => self.actor == Some(actor),
            (None, Some(sound)) => self.sound == Some(sound),
            (Some(actor), Some(sound)) => self.actor == Some(actor) && self.sound == Some(sound),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplayAudioGate {
    pub is_editor: bool,
    pub replay_mode: bool,
    pub context_5228: bool,
}

impl ReplayAudioGate {
    pub fn permits_music(self, force: bool) -> bool {
        self.is_editor || !self.replay_mode || !self.context_5228 || force
    }
}

pub fn rescaled_sound_gain(old_volume: f32, new_volume: f32, gain: f32) -> f32 {
    let divisor = if old_volume <= 0.0 { 1.0 } else { old_volume };
    (new_volume / divisor) * gain
}
