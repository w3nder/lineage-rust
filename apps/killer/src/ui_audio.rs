pub mod lobby;
mod ogg;
mod state;
pub mod types;
pub mod config;
pub mod music_state;
#[cfg(not(any(target_os = "ios", target_os = "android")))]
pub mod spatial;
#[cfg(any(target_os = "ios", target_os = "android"))]
#[path = "ui_audio/spatial_stub.rs"]
pub mod spatial;
pub mod spatial_state;

#[cfg(not(any(target_os = "ios", target_os = "android")))]
use alto::Source as _;
use std::cell::RefCell;
use glam::{Mat4,Vec3};

pub use state::{AudioConfigReader, AudioFrame, InitConfig, PlayingChannels};
use state::{SourceGain, channel_gain, native_clamp, nonspatial_priority, update_delta};

use std::io::Cursor;
use std::path::Path;

use killer_ui::option_audio::AudioChannel;
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};

pub const OGG_VOICE_FLAGS: u32 = 0x114;
pub const WAV_VOICE_FLAGS: u32 = 0x110;
pub const VOICE_RADIUS: f32 = 1000.0;

pub type AudioError = Box<dyn std::error::Error + Send + Sync>;

pub use types::{AudioVolumes, SourceIdentity, ReplayAudioGate, rescaled_sound_gain};

pub fn normalize_client_ogg(encoded: &mut [u8]) -> Result<(), AudioError> {
    let offsets = ogg::normalization_offsets(&mut Cursor::new(&*encoded))?;
    for offset in offsets {
        encoded[offset as usize..offset as usize + 4].copy_from_slice(b"OggS");
    }
    Ok(())
}

enum Playback {
    Stream(Sink),
    Spatial(RefCell<spatial::Voice>),
}

impl Playback {
    fn set_volume(&self, volume: f32) {
        match self {
            Self::Stream(sink) => sink.set_volume(volume),
            Self::Spatial(source) => if let Err(error) = source.borrow_mut().set_gain(volume) { log::warn!("ganho OpenAL: {error}"); },
        }
    }
    fn volume(&self) -> f32 {
        match self { Self::Stream(sink) => sink.volume(),Self::Spatial(source) => source.borrow().gain() }
    }
    fn stop(&self) {
        match self { Self::Stream(sink) => sink.stop(),Self::Spatial(source) => source.borrow_mut().stop() }
    }
    fn empty(&self) -> bool {
        match self { Self::Stream(sink) => sink.empty(),Self::Spatial(source) => spatial::finished(&source.borrow()) }
    }
}

struct ActiveSource {
    identity: SourceIdentity,
    sink: Playback,
    attenuation: Option<(f32,f32)>,
    spatial_position: Option<Vec3>,
    gain: SourceGain,
    priority: f32,
    priority_radius: Option<f32>,
}

impl ActiveSource {
    fn set_channel_volume(&mut self, volumes: AudioVolumes) {
        if let Some(gain) = channel_gain(self.identity, volumes) {
            self.gain.base = native_clamp(gain, 0.0, 1.0);
            self.sink.set_volume(self.gain.base);
        }
    }

    fn update_gain(&mut self, delta: f32, volumes: AudioVolumes) -> bool {
        if let Some(gain) = self.gain.update(delta, channel_gain(self.identity, volumes), self.attenuation) {
            self.sink.set_volume(gain);
            true
        } else {
            self.sink.stop();
            false
        }
    }
}

pub struct AudioDevice {
    stream: OutputStream,
    spatial: Option<spatial::Spatial>,
    listener_position: Vec3,
    sources: Vec<Option<ActiveSource>>,
    config: InitConfig,
    pending_music: String,
    previous_time: f64,
    playing: PlayingChannels,
    next_sound_id: u64,
    realtime: bool,
    replay_music: Vec<(String, i32)>,
}

impl AudioDevice {
    pub fn init(device: &mut Option<Self>, config: &impl AudioConfigReader, channels: i32) -> Result<(), AudioError> {
        if device.is_some() {
            return Ok(());
        }
        let initialized = Self::open(InitConfig::read(config), channels)?;
        *device = Some(initialized);
        Ok(())
    }

    pub fn open(config: InitConfig, channels: i32) -> Result<Self, AudioError> {
        let stream = OutputStreamBuilder::from_default_device()?.with_sample_rate(44100).open_stream()?;
        let source_count = channels.clamp(0, 32) as usize;
        if source_count == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "nenhum canal áudio configurado").into());
        }
        Ok(Self {
            stream,
            spatial: None,
            listener_position: Vec3::ZERO,
            sources: (0..source_count).map(|_| None).collect(),
            config,
            pending_music: String::new(),
            previous_time: 0.0,
            playing: PlayingChannels::default(),
            next_sound_id: 1,
            realtime: false,
            replay_music: Vec::new(),
        })
    }

    pub fn configure_spatial(&mut self, config: spatial::Config) -> Result<(),AudioError> {
        self.spatial = Some(spatial::Spatial::open(config)?);
        Ok(())
    }

    pub fn update_spatial(&mut self, frame: Option<AudioFrame>, camera_to_world: Mat4, positions: &std::collections::HashMap<u64,Vec3>) -> Result<(),AudioError> {
        let Some(frame) = frame else {
            for slot in &mut self.sources {
                if slot.as_ref().is_some_and(|source| source.spatial_position.is_some()) { *slot = None; }
            }
            return Ok(());
        };
        let listener = Vec3::from(frame.view_actor_location);
        let delta = update_delta(frame.time,self.previous_time);
        let velocity = (listener-self.listener_position)/delta;
        self.listener_position = listener;
        if let Some(spatial) = &self.spatial { spatial.listener(listener,camera_to_world)?; }
        for slot in &mut self.sources {
            let Some(source) = slot else { continue; };
            if source.spatial_position.is_none() { continue; }
            let Some(position) = source.identity.actor.and_then(|id| positions.get(&id)).copied() else { *slot = None; continue; };
            source.spatial_position = Some(position);
            let radius = source.priority_radius.unwrap_or(0.0)*frame.max_radius_multiplier;
            source.attenuation = Some((listener.distance(position),radius));
            if let Playback::Spatial(voice) = &source.sink {
                let mut voice = voice.borrow_mut();
                voice.set_position(spatial::coordinates(position))?;
                voice.set_velocity(spatial_state::source_velocity(velocity))?;
            }
        }
        Ok(())
    }

    pub fn play_pickup(&mut self, request: &crate::items::SoundRequest, frame: Option<AudioFrame>) -> Result<i32,AudioError> {
        let key = self.next_sound_id;
        self.next_sound_id = self.next_sound_id.wrapping_add(1);
        self.play_spatial(request.actor,key,&request.sound.name,&request.sound.encoded,request.position,request.volume,50.0,1.0,frame)
    }

    pub fn play_world(&mut self, sound: &crate::world_sound::WorldSound, frame: Option<AudioFrame>) -> Result<i32,AudioError> {
        self.play_spatial(sound.actor,sound.key,&sound.name,&sound.encoded,sound.position,sound.volume,sound.radius,sound.pitch,frame)
    }

    pub fn is_playing(&self, actor: u64, sound: u64) -> bool {
        self.sources.iter().flatten().any(|source| source.identity.matches_sound(Some(actor),Some(sound)))
    }

    #[allow(clippy::too_many_arguments)]
    fn play_spatial(&mut self, actor: u64, key: u64, name: &str, encoded: &[u8], position: Vec3, volume: f32, radius: f32, pitch: f32, frame: Option<AudioFrame>) -> Result<i32,AudioError> {
        let Some(frame) = frame else { return Ok(0); };
        let gain = native_clamp(volume*self.config.volumes.sound,0.0,1.0);
        if gain == 0.0 { return Ok(0); }
        let priority = state::sound_priority(frame,position.to_array(),radius,gain,0);
        let Some(index) = self.source_slot(priority) else { return Ok(0); };
        self.sources[index] = None;
        let Some(spatial) = &mut self.spatial else { return Err(std::io::Error::new(std::io::ErrorKind::NotConnected,"backend espacial indisponível").into()); };
        let scaled = radius*frame.max_radius_multiplier;
        let distance = Vec3::from(frame.view_actor_location).distance(position);
        let initial = native_clamp((scaled-distance)*gain/scaled,0.0,1.0);
        let voice = spatial.voice(name,encoded,position,initial,radius,pitch)?;
        self.sources[index] = Some(ActiveSource {
            identity:SourceIdentity { flags:0,actor:Some(actor),sound:Some(key) },
            sink:Playback::Spatial(RefCell::new(voice)),attenuation:Some((distance,scaled)),spatial_position:Some(position),
            gain:SourceGain::new(gain,0.0),priority,priority_radius:Some(radius),
        });
        Ok(index as i32+1)
    }

    fn source_slot(&self, priority: f32) -> Option<usize> {
        state::source_slot(self.config,self.sources.iter().map(|source|source.as_ref().map(|source|(source.identity.flags,source.priority))),priority)
    }

    pub fn config(&self) -> InitConfig {
        self.config
    }

    pub fn volumes(&self) -> AudioVolumes {
        self.config.volumes
    }

    pub fn playing_channels(&self) -> PlayingChannels {
        self.playing
    }

    pub fn set_pending_music(&mut self, name: String) {
        self.pending_music = name;
    }

    pub fn pending_music(&self) -> &str {
        &self.pending_music
    }

    pub fn source_gain(&self, handle: i32) -> Option<f32> {
        self.source(handle).map(|source| source.sink.volume())
    }

    fn source(&self, handle: i32) -> Option<&ActiveSource> {
        usize::try_from(handle.checked_sub(1)?).ok().and_then(|index| self.sources.get(index)).and_then(Option::as_ref)
    }

    fn source_mut(&mut self, handle: i32) -> Option<&mut ActiveSource> {
        usize::try_from(handle.checked_sub(1)?)
            .ok()
            .and_then(|index| self.sources.get_mut(index))
            .and_then(Option::as_mut)
    }

    pub fn play_file(
        &mut self,
        path: &Path,
        identity: SourceIdentity,
        effective_gain: f32,
        looping: bool,
        priority: f32,
    ) -> Result<i32, AudioError> {
        let input = std::io::BufReader::new(ogg::ClientOggReader::new(std::fs::File::open(path)?)?);
        let source: Box<dyn Source<Item = f32> + Send> =
            if looping { Box::new(Decoder::new_looped(input)?) } else { Box::new(Decoder::new(input)?) };
        self.append(source, identity, SourceGain::new(effective_gain, 0.0), priority, None)
    }

    pub fn play_interface_sound_10476e70(&mut self, encoded: Vec<u8>, frame: Option<AudioFrame>) -> Result<i32, AudioError> {
        let Some(frame) = frame else { return Ok(0) };
        if self.config.volumes.sound <= 0.0 { return Ok(0); }
        let gain = native_clamp(self.config.volumes.sound * self.config.volumes.sound, 0.0, 1.0);
        if gain == 0.0 { return Ok(0); }
        let source = Decoder::new(Cursor::new(encoded))?;
        let identity = SourceIdentity { flags: 0x10, actor: None, sound: Some(self.next_sound_id) };
        self.next_sound_id = self.next_sound_id.wrapping_add(1);
        let priority = nonspatial_priority(frame, 80.0, gain, 0x10);
        self.append(source, identity, SourceGain::new(gain, 0.0), priority, Some(80.0))
    }

    pub fn play_encoded(
        &mut self,
        mut encoded: Vec<u8>,
        identity: SourceIdentity,
        effective_gain: f32,
        looping: bool,
        priority: f32,
    ) -> Result<i32, AudioError> {
        normalize_client_ogg(&mut encoded)?;
        let input = Cursor::new(encoded);
        let source: Box<dyn Source<Item = f32> + Send> =
            if looping { Box::new(Decoder::new_looped(input)?) } else { Box::new(Decoder::new(input)?) };
        self.append(source, identity, SourceGain::new(effective_gain, 0.0), priority, None)
    }

    fn append(
        &mut self,
        source: impl Source<Item = f32> + Send + 'static,
        identity: SourceIdentity,
        gain: SourceGain,
        priority: f32,
        priority_radius: Option<f32>,
    ) -> Result<i32, AudioError> {
        if identity.actor.is_some() || identity.flags & 0x10 == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "fonte espacial requer backend de listener/doppler",
            )
            .into());
        }
        if identity.flags & 4 != 0
            && self.sources.iter().flatten().filter(|source| source.identity.flags & 4 != 0).count() >= 8
        {
            return Ok(0);
        }
        let Some(index) = self.source_slot(priority) else {
            return Ok(0);
        };
        if let Some(previous) = self.sources[index].take() {
            previous.sink.stop();
        }
        let sink = Sink::connect_new(self.stream.mixer());
        sink.set_volume(gain.initial_gain());
        sink.append(source);
        self.sources[index] = Some(ActiveSource { identity, sink:Playback::Stream(sink), gain, priority, priority_radius,attenuation:None,spatial_position:None });
        Ok(index as i32 + 1)
    }

    pub fn play_music(
        &mut self,
        client_root: &Path,
        request: &LobbyMusicRequest<'_>,
        gate: ReplayAudioGate,
        frame: Option<AudioFrame>,
    ) -> Result<i32, AudioError> {
        if !gate.permits_music(request.force != 0) || self.config.volumes.music <= 0.0 {
            return Ok(0);
        }
        self.pending_music.clear();
        if request.name.is_empty() {
            return Ok(0);
        }
        let path =
            client_root.join(if request.voice != 0 { "Voice" } else { "Music" }).join(format!("{}.ogg", request.name));
        let input = std::io::BufReader::new(ogg::ClientOggReader::new(std::fs::File::open(path)?)?);
        let source: Box<dyn Source<Item = f32> + Send> =
            if request.looping != 0 { Box::new(Decoder::new_looped(input)?) } else { Box::new(Decoder::new(input)?) };
        let flags = 0x1c | if request.looping != 0 { 0x80 } else { 0 } | if request.voice != 0 { 0x100 } else { 0 };
        let identity = SourceIdentity { flags, actor: None, sound: Some(self.next_sound_id) };
        self.next_sound_id = self.next_sound_id.wrapping_add(1);
        let gain = SourceGain::new(native_clamp(self.config.volumes.music, 0.0, 1.0), request.fade);
        let Some(frame) = frame else {
            return Ok(0);
        };
        let priority = nonspatial_priority(frame, 1000.0, gain.base, 0x1c);
        let handle = self.append(source, identity, gain, priority, Some(1000.0))?;
        if let Some(source) = self.source_mut(handle) {
            source.gain.play_music_adjustment(request.volume);
        }
        if handle != 0 && !gate.is_editor && frame.replay_recording {
            self.replay_music.push((request.name.to_owned(), request.voice));
        }
        Ok(handle)
    }

    pub fn play_ogg_voice(
        &mut self,
        client_root: &Path,
        file_name: &str,
        frame: Option<AudioFrame>,
    ) -> Result<i32, AudioError> {
        if self.config.volumes.ogg_voice <= 0.0 || file_name.is_empty() {
            return Ok(0);
        }
        let path = client_root.join("Voice").join(format!("{file_name}.ogg"));
        let input = std::io::BufReader::new(ogg::ClientOggReader::new(std::fs::File::open(path)?)?);
        let source: Box<dyn Source<Item = f32> + Send> = Box::new(Decoder::new(input)?);
        let flags = OGG_VOICE_FLAGS;
        let identity = SourceIdentity { flags, actor: None, sound: Some(self.next_sound_id) };
        self.next_sound_id = self.next_sound_id.wrapping_add(1);
        let gain = SourceGain::new(native_clamp(self.config.volumes.ogg_voice, 0.0, 1.0), 0.0);
        let Some(frame) = frame else {
            return Ok(0);
        };
        let priority = nonspatial_priority(frame, 1000.0, gain.base, flags);
        self.append(source, identity, gain, priority, Some(1000.0))
    }

    pub fn play_wav_voice(&mut self, encoded: Vec<u8>, frame: Option<AudioFrame>) -> Result<i32, AudioError> {
        let gain = native_clamp(self.config.volumes.wav_voice, 0.0, 1.0);
        if self.config.volumes.wav_voice <= 0.0 || gain == 0.0 {
            return Ok(0);
        }
        let Some(frame) = frame else { return Ok(0) };
        let source = Decoder::new(Cursor::new(encoded))?;
        let identity = SourceIdentity { flags: WAV_VOICE_FLAGS, actor: None, sound: Some(self.next_sound_id) };
        self.next_sound_id = self.next_sound_id.wrapping_add(1);
        let priority = nonspatial_priority(frame, VOICE_RADIUS, gain, WAV_VOICE_FLAGS);
        self.append(source, identity, SourceGain::new(gain, 0.0), priority, Some(VOICE_RADIUS))
    }

    pub fn music_playing(&self) -> bool {
        self.playing.music
    }

    pub fn take_replay_music(&mut self) -> Vec<(String, i32)> {
        std::mem::take(&mut self.replay_music)
    }

    pub fn set_volumes(
        &mut self,
        client_root: &Path,
        gate: ReplayAudioGate,
        frame: Option<AudioFrame>,
    ) -> Result<(), AudioError> {
        let result = if !self.pending_music.is_empty() && self.config.volumes.music > 0.0 {
            let name = self.pending_music.clone();
            self.play_music(
                client_root,
                &LobbyMusicRequest { name: &name, fade: 0.0, looping: 0, voice: 0, volume: 1.0, force: 0 },
                gate,
                frame,
            )
            .map(|_| ())
        } else {
            Ok(())
        };
        for source in self.sources.iter_mut().flatten() {
            source.set_channel_volume(self.config.volumes);
        }
        result
    }

    pub fn update_nonspatial(&mut self, frame: Option<AudioFrame>) {
        let Some(frame) = frame else {
            return;
        };
        if self.config.volumes.ui_values().iter().all(|volume| *volume <= 0.0) {
            return;
        }
        let delta = update_delta(frame.time, self.previous_time);
        if !frame.realtime && self.realtime {
            self.stop_sound(None, None);
        }
        self.realtime = frame.realtime;
        self.playing = PlayingChannels::default();
        for slot in &mut self.sources {
            let Some(source) = slot else {
                continue;
            };
            if source.identity.flags & 2 != 0 {
                continue;
            }
            if source.sink.empty() {
                if source.identity.flags & 0x104 == 0x104 {
                    self.playing.ogg_voice_finished = true;
                }
                *slot = None;
            } else {
                self.playing.observe(source.identity.flags);
            }
        }
        self.previous_time = frame.time;
        for slot in &mut self.sources {
            if let Some(source) = slot {
                if !source.update_gain(delta, self.config.volumes) {
                    *slot = None;
                } else if let Some(radius) = source.priority_radius {
                    source.priority = state::sound_priority(frame,source.spatial_position.unwrap_or(Vec3::ZERO).to_array(),radius,source.gain.base,source.identity.flags);
                }
            }
        }
    }

    pub fn set_volume(&mut self, channel: AudioChannel, value: f32) {
        match channel {
            AudioChannel::Sound => {
                let previous = self.config.volumes.sound;
                self.config.volumes.sound = value;
                for source in self.sources.iter_mut().flatten() {
                    if source.identity.is_sound() {
                        source.gain.base = rescaled_sound_gain(previous, value, source.gain.base);
                        source.sink.set_volume(source.gain.base);
                    }
                }
            }
            AudioChannel::Music => self.config.volumes.music = value,
            AudioChannel::WavVoice => self.config.volumes.wav_voice = value,
            AudioChannel::OggVoice => self.config.volumes.ogg_voice = value,
        }
    }

    pub fn set_music_volume(&mut self, handle: i32, volume: f32, fade: f32) {
        let music = self.config.volumes.music;
        if let Some(source) = self.source_mut(handle) {
            source.gain.set_music_volume(music, volume, fade);
        }
    }

    pub fn stop_music(&mut self, handle: i32, fade: f32, force: bool, gate: ReplayAudioGate) -> bool {
        if !gate.permits_music(force) {
            return false;
        }
        let Some(source) = self.source_mut(handle) else {
            return false;
        };
        if fade > 0.0 {
            source.gain.stop(fade);
        } else {
            source.sink.stop();
            self.sources[(handle - 1) as usize] = None;
        }
        true
    }

    pub fn stop_sound(&mut self, actor: Option<u64>, sound: Option<u64>) -> bool {
        let mut stopped = actor.is_none() && sound.is_none();
        for slot in &mut self.sources {
            if slot.as_ref().is_some_and(|source| source.identity.matches_sound(actor, sound)) {
                if let Some(source) = slot.take() {
                    source.sink.stop();
                }
                stopped = true;
            }
        }
        stopped
    }

    pub fn stop_all_voice(&mut self, fade: f32, ogg: bool) {
        let channel = if ogg { AudioChannel::OggVoice } else { AudioChannel::WavVoice };
        for slot in &mut self.sources {
            if !slot.as_ref().is_some_and(|source| source.identity.matches_channel(channel)) {
                continue;
            }
            if fade > 0.0 {
                if let Some(source) = slot.as_mut() {
                    source.gain.stop(fade);
                }
            } else if let Some(source) = slot.take() {
                source.sink.stop();
            }
        }
    }

    pub fn stop_channel_immediately(&mut self, channel: AudioChannel, gate: ReplayAudioGate) {
        if channel == AudioChannel::Sound {
            self.stop_sound(None, None);
            return;
        }
        if !gate.permits_music(false) {
            return;
        }
        for slot in &mut self.sources {
            if slot.as_ref().is_some_and(|source| source.identity.matches_channel(channel))
                && let Some(source) = slot.take()
            {
                source.sink.stop();
            }
        }
    }
}

pub use music_state::{MusicController, LobbyMusicRequest, restore_lobby_music};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_masks_and_sound_filters_match_alaudio() {
        for flags in 0..512 {
            let source = SourceIdentity { flags, actor: Some(7), sound: Some(9) };
            assert_eq!(source.matches_channel(AudioChannel::Sound), flags & 0x108 == 0);
            assert_eq!(source.matches_channel(AudioChannel::Music), flags & 8 != 0);
            assert_eq!(source.matches_channel(AudioChannel::WavVoice), flags & 0x104 == 0x100);
            assert_eq!(source.matches_channel(AudioChannel::OggVoice), flags & 0x104 == 0x104);
            assert!(source.matches_sound(Some(7), Some(9)));
            assert!(source.matches_sound(None, Some(9)));
            assert!(source.matches_sound(Some(7), None));
            assert!(!source.matches_sound(Some(8), Some(9)));
            assert!(!source.matches_sound(Some(7), Some(10)));
            assert_eq!(source.matches_sound(None, None), source.is_sound());
        }
    }

    #[test]
    fn replay_gate_keeps_all_native_branches() {
        for bits in 0..16 {
            let gate =
                ReplayAudioGate { is_editor: bits & 1 != 0, replay_mode: bits & 2 != 0, context_5228: bits & 4 != 0 };
            assert_eq!(gate.permits_music(bits & 8 != 0), bits != 6);
        }
    }

    #[test]
    fn sound_setter_rescales_existing_gain() {
        assert_eq!(rescaled_sound_gain(0.8, 0.4, 0.6), 0.3);
        for old in [0.0, -0.0, -1.0, f32::NEG_INFINITY] {
            assert_eq!(rescaled_sound_gain(old, 0.4, 0.5), 0.2);
        }
        assert!(rescaled_sound_gain(f32::NAN, 0.4, 0.5).is_nan());
        assert_eq!(rescaled_sound_gain(0.0, 0.4, 0.0), 0.0);
    }

    #[test]
    fn controller_reset_does_not_choose_music() {
        let mut base = MusicController::Base;
        base.reset_music_controll();
        assert_eq!(base, MusicController::Base);
        let mut lineage = MusicController::Lineage { field_a34: 99, flags_a38: 0xa4 };
        lineage.reset_music_controll();
        assert_eq!(lineage, MusicController::Lineage { field_a34: 0, flags_a38: 0xa5 });
    }

    #[test]
    fn lobby_restore_uses_selected_track_and_skips_world() {
        let names = std::array::from_fn(|index| index.to_string());
        for state in 0..8 {
            for selected in -1..8 {
                let mut handle = -1;
                let mut calls = 0;
                restore_lobby_music(state, selected, &names, &mut handle, |request| {
                    calls += 1;
                    assert_eq!(request.name, &names[selected as usize]);
                    assert_eq!(
                        (request.fade, request.looping, request.voice, request.volume, request.force),
                        (1.5, 1, 0, 1.0, 0)
                    );
                    Ok(42)
                })
                .unwrap();
                let expected = state != 6 && selected >= 0;
                assert_eq!(calls, i32::from(expected));
                assert_eq!(handle, if expected { 42 } else { -1 });
            }
        }
    }

    #[test]
    fn l2sd_normalization_preserves_payload_and_all_pages() {
        let mut page = vec![0_u8; 32];
        page[..4].copy_from_slice(b"L2SD");
        page[26] = 1;
        page[27] = 4;
        page[28..].copy_from_slice(b"L2SD");
        page[..4].copy_from_slice(b"OggS");
        let crc = ogg::checksum(&page);
        page[22..26].copy_from_slice(&crc.to_le_bytes());
        page[..4].copy_from_slice(b"L2SD");
        let mut encoded = [page.clone(), page].concat();
        normalize_client_ogg(&mut encoded).unwrap();
        assert_eq!(&encoded[..4], b"OggS");
        assert_eq!(&encoded[32..36], b"OggS");
        assert_eq!(&encoded[28..32], b"L2SD");
        assert_eq!(&encoded[60..64], b"L2SD");
        assert!(normalize_client_ogg(&mut encoded[..63]).is_ok());
        let mut wav = b"RIFFpayload".to_vec();
        normalize_client_ogg(&mut wav).unwrap();
        assert_eq!(wav, b"RIFFpayload");
    }

    #[test]
    #[ignore]
    fn client_intro_decodes_real_audio_without_a_device() {
        let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let path = Path::new(&root).join("music/intro.ogg");
        let mut encoded = std::fs::read(path).unwrap();
        normalize_client_ogg(&mut encoded).unwrap();
        let mut source = Decoder::try_from(Cursor::new(encoded)).unwrap();
        assert!(source.sample_rate() > 0);
        assert!(source.channels() > 0);
        assert!(source.any(|sample| sample != 0.0 && sample.is_finite()));
    }
}

#[cfg(test)]
mod runtime_tests {
    use super::*;
    use rodio::buffer::SamplesBuffer;

    fn source(flags: u32, base: f32, fade: f32) -> (ActiveSource, rodio::queue::SourcesQueueOutput) {
        let (sink, output) = Sink::new();
        let gain = SourceGain::new(base, fade);
        sink.set_volume(gain.initial_gain());
        sink.append(SamplesBuffer::new(1, 1000, vec![0.8; 1000]));
        (
            ActiveSource {
                identity: SourceIdentity { flags, actor: None, sound: Some(1) },
                sink:Playback::Stream(sink),
                attenuation:None,
                spatial_position:None,
                gain,
                priority: 1.0,
                priority_radius: None,
            },
            output,
        )
    }

    fn settled_sample(output: &mut rodio::queue::SourcesQueueOutput) -> f32 {
        output.by_ref().take(10).last().unwrap()
    }

    #[test]
    fn update_changes_real_rodio_pcm_and_stop_ends_source() {
        let volumes = AudioVolumes { sound: 1.0, music: 0.8, wav_voice: 0.6, ogg_voice: 0.4 };
        let (mut source, mut output) = source(0x1c, 0.8, 1.0);
        assert_eq!(settled_sample(&mut output), 0.0);
        assert!(source.update_gain(0.25, volumes));
        assert!((settled_sample(&mut output) - 0.16).abs() < 0.00001);
        source.gain.stop(0.5);
        assert!(source.update_gain(0.25, volumes));
        assert!((settled_sample(&mut output) - 0.32).abs() < 0.00001);
        assert!(!source.update_gain(0.25, volumes));
        assert_eq!(settled_sample(&mut output), 0.0);
        assert!(source.sink.empty());
    }

    #[test]
    fn set_volumes_changes_channel_pcm_but_keeps_fade_and_effects_base() {
        let volumes = AudioVolumes { sound: 0.2, music: 0.4, wav_voice: 0.6, ogg_voice: 0.8 };
        for (flags, expected) in [(0x10, 0.3), (0x18, 0.4), (0x110, 0.6), (0x114, 0.8), (0x11c, 0.4)] {
            let (mut source, mut output) = source(flags, 0.3, 0.5);
            source.set_channel_volume(volumes);
            assert_eq!(source.gain.base, expected);
            assert_eq!(source.gain.fade, state::FadeKind::In);
            let actual = settled_sample(&mut output);
            let expected_pcm = if flags == 0x10 { 0.0 } else { expected * 0.8 };
            assert!((actual - expected_pcm).abs() < 0.00001);
        }
    }

    #[test]
    fn fades_do_not_advance_on_one_second_delta() {
        let volumes = AudioVolumes { sound: 1.0, music: 0.8, wav_voice: 0.6, ogg_voice: 0.4 };
        for kind in [state::FadeKind::In, state::FadeKind::Out, state::FadeKind::Relative, state::FadeKind::Restore] {
            let (mut source, mut output) = source(0x1c, 0.8, 2.0);
            source.gain.fade = kind;
            source.gain.elapsed = 0.25;
            source.gain.adjustment = 0.2;
            assert!(source.update_gain(1.0, volumes));
            assert_eq!(source.gain.elapsed, 0.25);
            assert!(settled_sample(&mut output).is_finite());
        }
    }

    #[test]
    #[ignore]
    fn entire_original_intro_stream_decodes_without_loading_pcm() {
        let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let file = std::fs::File::open(Path::new(&root).join("Music/intro.ogg")).unwrap();
        let input = std::io::BufReader::new(ogg::ClientOggReader::new(file).unwrap());
        let decoder = Decoder::new(input).unwrap();
        let mut samples = 0_u64;
        let mut nonzero = false;
        for sample in decoder {
            assert!(sample.is_finite());
            nonzero |= sample != 0.0;
            samples += 1;
        }
        assert!(samples > 0 && nonzero);
    }
}
