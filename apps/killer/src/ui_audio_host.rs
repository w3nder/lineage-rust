use std::collections::HashMap;
use std::path::Path;

use killer_ui::ini::IniFile;
use killer_ui::option_audio::{AudioApplyBackend, AudioChannel};

use crate::ui_audio::AudioFrame;
use crate::zone_music::{MusicOut, MusicTimes, Pawn, ZoneMusic};
use crate::ui_audio::{AudioDevice, AudioError, LobbyMusicRequest, MusicController, ReplayAudioGate, restore_lobby_music};
use crate::ui_option_config::ConfigEdit;

pub use crate::ui_audio::lobby::{LobbyConsole, LobbyMusic, credit_sound_names, lobby_sound_names};
pub const MAX_RADIUS_MULTIPLIER: f32 = 50.0;
pub const NATIVE_GATE: ReplayAudioGate = ReplayAudioGate { is_editor: false, replay_mode: false, context_5228: false };

pub use crate::ui_audio::config::AudioIni;
#[cfg(test)]
use crate::ui_audio::config::native_atof;

pub fn channels(system: &IniFile) -> i32 {
    system.int("ALAudio.ALAudioSubsystem", "Channels").unwrap_or(0)
}

struct DeviceMusic<'a> {
    device: &'a mut AudioDevice,
    root: &'a Path,
    frame: Option<AudioFrame>,
}

impl LobbyMusic for DeviceMusic<'_> {
    fn music_playing(&self) -> bool {
        self.device.music_playing()
    }
    fn sound_volume(&self) -> f32 {
        self.device.volumes().sound
    }
    fn set_sound_volume(&mut self, volume: f32) {
        self.device.set_volume(AudioChannel::Sound, volume);
    }
    fn stop_music(&mut self, handle: i32, fade: f32) {
        self.device.stop_music(handle, fade, false, NATIVE_GATE);
    }
    fn play_music(&mut self, request: LobbyMusicRequest<'_>) -> Result<i32, AudioError> {
        self.device.play_music(self.root, &request, NATIVE_GATE, self.frame)
    }
}

struct WorldMusicOut<'a> {
    device: &'a mut AudioDevice,
    root: &'a Path,
    frame: Option<AudioFrame>,
    voice_suffix: &'a str,
    wav_voices: &'a HashMap<String, Vec<u8>>,
}

impl MusicOut for WorldMusicOut<'_> {
    fn music_playing(&self) -> bool {
        self.device.music_playing()
    }
    fn ogg_voice_playing(&self) -> bool {
        self.device.playing_channels().ogg_voice
    }
    fn wav_voice_playing(&self) -> bool {
        self.device.playing_channels().wav_voice
    }
    fn voice_finished(&self) -> bool {
        self.device.playing_channels().ogg_voice_finished
    }
    fn play_music(&mut self, name: &str, volume: f32) -> i32 {
        let request = LobbyMusicRequest { name, fade: 0.0, looping: 0, voice: 0, volume, force: 0 };
        self.device.play_music(self.root, &request, NATIVE_GATE, self.frame).unwrap_or_else(|error| {
            log::warn!("áudio: música {name}: {error}");
            0
        })
    }
    fn stop_music(&mut self, handle: i32, fade: f32) {
        self.device.stop_music(handle, fade, false, NATIVE_GATE);
    }
    fn set_music_volume(&mut self, handle: i32, volume: f32, fade: f32) {
        self.device.set_music_volume(handle, volume, fade);
    }
    fn play_voice(&mut self, name: &str, ogg: bool) -> i32 {
        if !ogg {
            let Some(encoded) = self.wav_voices.get(name) else { return 0 };
            return self.device.play_wav_voice(encoded.clone(), self.frame).unwrap_or_else(|error| {
                log::warn!("áudio: voz {name}: {error}");
                0
            });
        }
        self.device.play_ogg_voice(self.root, &format!("{name}{}", self.voice_suffix), self.frame).unwrap_or_else(|error| {
            log::warn!("áudio: voz {name}: {error}");
            0
        })
    }
}

pub struct AudioHost {
    pub device: Option<AudioDevice>,
    pub console: LobbyConsole,
    pub controller: MusicController,
    pub frame: Option<AudioFrame>,
    pub intro_started: bool,
    pub zone: ZoneMusic,
    pub music_times: MusicTimes,
    pub voice_suffix: String,
    pub wav_voices: HashMap<String, Vec<u8>>,
}

impl AudioHost {
    pub fn stop_all_music(&mut self) {
        if let Some(device) = &mut self.device {
            device.stop_channel_immediately(AudioChannel::Music, NATIVE_GATE);
        }
    }

    pub fn stop_all_voice(&mut self, fade: f32, ogg: bool) {
        if let Some(device) = &mut self.device {
            device.stop_all_voice(fade, ogg);
        }
    }

    pub fn reset_music(&mut self) {
        self.controller.reset_music_controll();
        self.zone.reset();
    }

    pub fn play_interface_sound(&mut self, client: &mut killer_assets::client::Client, name: &str) -> Result<i32, AudioError> {
        if name.is_empty() || name == "NAME_None" { return Ok(0); }
        let Some(device) = self.device.as_mut() else { return Ok(0) };
        if self.frame.is_none() { return Ok(0); }
        let Some((format, encoded)) = client.sound(name) else { return Ok(0) };
        if !format.eq_ignore_ascii_case("WAV") {
            return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, format!("USound {name}: formato {format}")).into());
        }
        device.play_interface_sound_10476e70(encoded, self.frame)
    }

    pub fn request_wav_voice(&mut self, client: &mut killer_assets::client::Client, name: &str) -> Result<(), AudioError> {
        if self.device.is_none() { return Ok(()) }
        let Some((format, encoded)) = client.sound(name) else { return Ok(()) };
        if !format.eq_ignore_ascii_case("WAV") {
            return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, format!("USound {name}: formato {format}")).into());
        }
        self.wav_voices.insert(name.to_owned(), encoded);
        self.zone.request_server_voice(name, 0.0, false);
        Ok(())
    }

    pub fn play_wav_voice(&mut self, client: &mut killer_assets::client::Client, name: &str) -> Result<i32, AudioError> {
        let Some(device) = self.device.as_mut() else { return Ok(0) };
        let Some((format, encoded)) = client.sound(name) else { return Ok(0) };
        if !format.eq_ignore_ascii_case("WAV") {
            return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, format!("USound {name}: formato {format}")).into());
        }
        device.play_wav_voice(encoded, self.frame)
    }

    pub fn volumes(&self) -> Option<[f32; 4]> {
        self.device.as_ref().map(|device| device.volumes().ui_values())
    }

    pub fn set_console_state(&mut self, root: &Path, state: i32) {
        let frame = self.frame;
        let mut music = self.device.as_mut().map(|device| DeviceMusic { device, root, frame });
        if let Err(error) = self.console.set_state(state, music.as_mut()) {
            log::warn!("áudio: música do lobby: {error}");
        }
        log::info!("áudio: estado do console {state}, música {:?} handle {}", usize::try_from(self.console.selected_48).ok().map(|i| &self.console.names[i]), self.console.handle_40);
    }

    pub fn play_intro_music(&mut self, root: &Path) {
        let frame = self.frame;
        let mut music = self.device.as_mut().map(|device| DeviceMusic { device, root, frame });
        if let Err(error) = self.console.play_intro_music(music.as_mut()) {
            log::warn!("áudio: PlayIntroMusic: {error}");
        }
        log::info!("áudio: PlayIntroMusic handle {}", self.console.handle_40);
    }

    pub fn set_credits(&mut self, root: &Path, visible: bool, roll: u32) {
        let frame = self.frame;
        let mut music = self.device.as_mut().map(|device| DeviceMusic { device, root, frame });
        if let Err(error) = self.console.set_credits(visible, roll, music.as_mut()) {
            log::warn!("áudio: música dos créditos: {error}");
        }
    }

    pub fn set_current_making_race(&mut self, root: &Path, race: i32) {
        let frame = self.frame;
        let mut music = self.device.as_mut().map(|device| DeviceMusic { device, root, frame });
        if let Err(error) = self.console.set_current_making_race(race, music.as_mut()) {
            log::warn!("áudio: SetCurrentMakingRace: {error}");
        }
    }

    pub fn tick_world_music(&mut self, root: &Path, dt: f32, pawn: Option<&Pawn>, name_of: impl FnMut(i32) -> Option<String>) {
        let frame = self.frame;
        let mut out = self.device.as_mut().map(|device| WorldMusicOut { device, root, frame, voice_suffix: &self.voice_suffix, wav_voices: &self.wav_voices });
        self.zone.tick(dt, self.music_times, pawn, out.as_mut().map(|out| out as &mut dyn MusicOut), name_of);
    }

    pub fn update(&mut self, frame: Option<AudioFrame>) {
        self.frame = frame;
        if let Some(device) = &mut self.device {
            device.update_nonspatial(frame);
        }
    }
}

pub struct AudioApplyHost<'a> {
    pub host: &'a mut AudioHost,
    pub root: &'a Path,
    pub controller_present: bool,
    pub edits: &'a mut Vec<ConfigEdit>,
}

impl AudioApplyBackend for AudioApplyHost<'_> {
    fn available(&mut self) -> bool { self.host.device.is_some() }
    fn volume(&mut self, channel: AudioChannel) -> f32 {
        self.host.device.as_ref().map_or(0.0, |device| device.volumes().get(channel))
    }
    fn set_volume(&mut self, channel: AudioChannel, value: f32) {
        if let Some(device) = &mut self.host.device { device.set_volume(channel, value); }
    }
    fn stop(&mut self, channel: AudioChannel) {
        if let Some(device) = &mut self.host.device { device.stop_channel_immediately(channel, NATIVE_GATE); }
    }
    fn has_music_controller(&mut self) -> bool { self.controller_present }
    fn reset_music_controller(&mut self) { self.host.controller.reset_music_controll(); self.host.zone.reset(); }
    fn restore_lobby_music(&mut self) {
        let frame = self.host.frame;
        let root = self.root;
        let console = &mut self.host.console;
        let Some(device) = self.host.device.as_mut() else { return };
        let names = console.names.clone();
        if let Err(error) = restore_lobby_music(console.state_38, console.selected_48, &names, &mut console.handle_40, |request| {
            device.play_music(root, &request, NATIVE_GATE, frame)
        }) {
            log::warn!("áudio: RestoreLobbyMusic: {error}");
        }
    }
    fn write_volume(&mut self, file: &str, section: &str, key: &str, value: f32) {
        if file.eq_ignore_ascii_case("Option.ini") {
            self.edits.push(ConfigEdit::new(section, key, format!("{value:.6}")));
        } else {
            log::error!("áudio: gravação em arquivo inesperado {file}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::ini::parse_ini;

    #[test]
    fn lobby_sound_1042de40_reads_eight_keys_with_intro_default() {
        let ini = parse_ini("[LobbySound]\nLogInSound=S28_F\nCharacterSelect=NT_Oren\nCharacterOrc=CT_Orc\n");
        let names = lobby_sound_names(&ini);
        assert_eq!(names[0], "S28_F");
        assert_eq!(names[1], "NT_Oren");
        assert_eq!(names[6], "CT_Orc");
        for index in [2, 3, 4, 5, 7] { assert_eq!(names[index], "intro"); }
        assert_eq!(lobby_sound_names(&parse_ini("")), std::array::from_fn(|_| "intro".to_owned()));
    }

    #[test]
    #[ignore]
    fn interface_sound_10476e70_loads_the_client_wave_and_uses_squared_sound_volume() {
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").unwrap());
        let mut client = killer_assets::client::Client::open(&root).unwrap();
        let option = parse_ini("[Audio]
SoundVolume=0.5
");
        let alaudio = parse_ini(&client.read_system_text("alaudio.int").unwrap());
        let system = parse_ini(&client.read_system_text("l2.ini").unwrap());
        let mut device = None;
        AudioDevice::init(&mut device, &AudioIni { option: &option, alaudio: &alaudio }, channels(&system)).unwrap();
        let mut host = AudioHost {
            device, console: LobbyConsole::new(lobby_sound_names(&alaudio)).with_credit_names(credit_sound_names(&alaudio)), controller: MusicController::Base, zone: ZoneMusic::default(), music_times: MusicTimes::default(), voice_suffix: String::new(), wav_voices: HashMap::new(),
            frame: None, intro_started: false,
        };
        host.device.as_mut().unwrap().set_volume(AudioChannel::Sound, 0.5);
        assert_eq!(host.play_interface_sound(&mut client, "itemsound.window_close").unwrap(), 0);
        host.frame = Some(AudioFrame { time: 0.0, view_actor_location: [83400.0, 148000.0, -3400.0], max_radius_multiplier: MAX_RADIUS_MULTIPLIER, realtime: true, replay_recording: false });
        assert_eq!(host.play_interface_sound(&mut client, "").unwrap(), 0);
        assert_eq!(host.play_interface_sound(&mut client, "NAME_None").unwrap(), 0);
        let handle = host.play_interface_sound(&mut client, "itemsound.window_close").unwrap();
        assert!(handle > 0);
        assert_eq!(host.device.as_ref().unwrap().source_gain(handle), Some(0.25));
        std::thread::sleep(std::time::Duration::from_millis(350));
        host.device.as_mut().unwrap().stop_sound(None, None);
        host.device.as_mut().unwrap().set_volume(AudioChannel::Sound, 0.0);
        assert_eq!(host.play_interface_sound(&mut client, "itemsound.window_close").unwrap(), 0);
    }

    #[test]
    #[ignore]
    fn every_system_message_second_sound_resolves_to_a_wave_and_plays_as_wav_voice() {
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").unwrap());
        let mut client = killer_assets::client::Client::open(&root).unwrap();
        let tables = client.tables().unwrap();
        let voices = tables.systemmsg.values().filter(|m| !m.second_sound.is_empty() && m.second_sound != "None").map(|m| m.second_sound.clone()).collect::<Vec<_>>();
        assert_eq!(voices.len(), 73);
        let (korean, english): (Vec<_>, Vec<_>) = voices.iter().partition(|name| name.starts_with("SystemMsg_k."));
        assert_eq!((korean.len(), english.len()), (2, 71));
        for name in &korean {
            assert!(client.sound(name).is_none(), "{name}");
        }
        for name in &english {
            let (format, encoded) = client.sound(name).unwrap_or_else(|| panic!("{name}"));
            assert!(format.eq_ignore_ascii_case("WAV"), "{name}: {format}");
            assert!(!encoded.is_empty(), "{name}");
        }
        let option = parse_ini("[Audio]
WavVoiceVolume=0.5
");
        let alaudio = parse_ini(&client.read_system_text("alaudio.int").unwrap());
        let system = parse_ini(&client.read_system_text("l2.ini").unwrap());
        let mut device = None;
        AudioDevice::init(&mut device, &AudioIni { option: &option, alaudio: &alaudio }, channels(&system)).unwrap();
        let mut host = AudioHost {
            device, console: LobbyConsole::new(lobby_sound_names(&alaudio)), controller: MusicController::Base, zone: ZoneMusic::default(), music_times: MusicTimes::default(), voice_suffix: String::new(), wav_voices: HashMap::new(),
            frame: Some(AudioFrame { time: 0.0, view_actor_location: [0.0; 3], max_radius_multiplier: MAX_RADIUS_MULTIPLIER, realtime: true, replay_recording: false }),
            intro_started: false,
        };
        host.request_wav_voice(&mut client, english[0]).unwrap();
        host.tick_world_music(&root, 0.1, None, |_| None);
        assert!(host.zone.voice_handle() > 0);
        assert_eq!(host.device.as_ref().unwrap().source_gain(host.zone.voice_handle()), Some(0.5));
        host.device.as_mut().unwrap().stop_sound(None, None);
    }

    #[test]
    #[ignore]
    fn client_lobby_music_plays_on_the_real_device_through_the_console_states() {
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let mut client = killer_assets::client::Client::open(&root).unwrap();
        let read = |client: &mut killer_assets::client::Client, name: &str| parse_ini(&client.read_system_text(name).unwrap());
        let (option, alaudio, system) = (read(&mut client, "option.ini"), read(&mut client, "alaudio.int"), read(&mut client, "l2.ini"));
        assert_eq!(channels(&system), 32);
        let names = lobby_sound_names(&alaudio);
        assert_eq!(names[..3], ["S28_F".to_owned(), "NT_Oren".to_owned(), "S28_F".to_owned()]);
        let mut device = None;
        AudioDevice::init(&mut device, &AudioIni { option: &option, alaudio: &alaudio }, channels(&system)).unwrap();
        let mut host = AudioHost {
            device, console: LobbyConsole::new(names), controller: MusicController::Base, zone: ZoneMusic::default(), music_times: MusicTimes::default(), voice_suffix: String::new(), wav_voices: HashMap::new(),
            frame: Some(AudioFrame { time: 0.0, view_actor_location: [0.0; 3], max_radius_multiplier: MAX_RADIUS_MULTIPLIER, realtime: false, replay_recording: false }),
            intro_started: false,
        };
        host.device.as_mut().unwrap().set_volume(AudioChannel::Music, 1.0);
        host.play_intro_music(&root);
        let login = host.console.handle_40;
        assert!(login > 0, "PlayMusic devolveu {login}");
        host.set_console_state(&root, 4);
        assert!(host.console.handle_40 > 0);
        assert_eq!(host.console.selected_48, 1);
        for step in 1..=20 {
            host.update(host.frame.map(|frame| AudioFrame { time: f64::from(step) * 0.1, ..frame }));
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        let gain = host.device.as_ref().unwrap().source_gain(host.console.handle_40).unwrap();
        assert!(gain > 0.9, "ganho após o fade de 1,5 s: {gain}");
        let mut edits = Vec::new();
        let mut page = killer_ui::option_audio::AudioPage::new(|_| None);
        page.fill(host.volumes(), |_| None);
        page.combos[1].selected = 5;
        page.apply(&mut AudioApplyHost { host: &mut host, root: &root, controller_present: false, edits: &mut edits });
        assert!(host.device.as_ref().unwrap().source_gain(host.console.handle_40).is_none());
        assert!(edits.iter().any(|edit| edit.key == "MusicVolume" && edit.value == "0.000000"));
        page.combos[1].selected = 0;
        page.apply(&mut AudioApplyHost { host: &mut host, root: &root, controller_present: false, edits: &mut edits });
        assert!(host.console.handle_40 > 0, "RestoreLobbyMusic deveria voltar a tocar no lobby");
    }

    #[test]
    fn channels_property_d8_has_no_default() {
        assert_eq!(channels(&parse_ini("[ALAudio.ALAudioSubsystem]\nChannels=32\n")), 32);
        assert_eq!(channels(&parse_ini("")), 0);
    }

    #[test]
    fn native_atof_accepts_numeric_prefix_like_appatof() {
        for (text, value) in [("0.000000", 0.0), ("0.8", 0.8), (" 1.5x", 1.5), ("-2e1", -20.0), ("abc", 0.0), ("3.", 3.0), ("1e", 1.0)] {
            assert_eq!(native_atof(text), value, "{text}");
        }
    }

    #[derive(Default)]
    struct Music {
        events: Vec<String>,
        next: i32,
        fail: bool,
    }

    impl LobbyMusic for Music {
        fn music_playing(&self) -> bool { false }
        fn sound_volume(&self) -> f32 { 0.0 }
        fn set_sound_volume(&mut self, _volume: f32) {}
        fn stop_music(&mut self, handle: i32, _fade: f32) { self.events.push(format!("stop {handle}")); }
        fn play_music(&mut self, request: LobbyMusicRequest<'_>) -> Result<i32, AudioError> {
            assert_eq!((request.fade, request.looping, request.voice, request.volume, request.force), (1.5, 1, 0, 1.0, 0));
            self.events.push(format!("play {}", request.name));
            if self.fail { return Err(std::io::Error::other("sem arquivo").into()); }
            self.next += 1;
            Ok(self.next)
        }
    }

    fn new_console() -> LobbyConsole {
        LobbyConsole::new(std::array::from_fn(|index| format!("m{index}")))
    }

    #[derive(Default)]
    struct CreditMusic {
        events: Vec<String>,
        next: i32,
        playing: bool,
        sound: f32,
    }

    impl LobbyMusic for CreditMusic {
        fn music_playing(&self) -> bool { self.playing }
        fn sound_volume(&self) -> f32 { self.sound }
        fn set_sound_volume(&mut self, volume: f32) {
            self.events.push(format!("sfx {volume}"));
            self.sound = volume;
        }
        fn stop_music(&mut self, handle: i32, fade: f32) {
            self.events.push(format!("stop {handle} {fade}"));
            self.playing = false;
        }
        fn play_music(&mut self, request: LobbyMusicRequest<'_>) -> Result<i32, AudioError> {
            assert_eq!((request.fade, request.voice, request.volume, request.force), (1.5, 0, 1.0, 0));
            assert_eq!(request.looping, i32::from(!request.name.starts_with('c')));
            self.events.push(format!("play {}", request.name));
            self.next += 1;
            self.playing = true;
            Ok(self.next)
        }
    }

    #[test]
    fn credits_replace_the_intro_music_pick_one_of_five_when_nothing_plays_and_bring_the_intro_back() {
        let mut console = new_console().with_credit_names(std::array::from_fn(|index| format!("c{index}")));
        let mut music = CreditMusic { sound: 0.75, ..CreditMusic::default() };
        console.play_intro_music(Some(&mut music)).unwrap();
        assert_eq!(console.handle_40, 1);
        music.events.clear();
        console.set_credits(true, 7, Some(&mut music)).unwrap();
        assert_eq!(music.events, ["stop 1 0", "sfx 0", "play c2"]);
        assert_eq!((console.handle_40, console.credit_handle, console.credits_active), (-1, 2, true));
        console.set_credits(true, 9, Some(&mut music)).unwrap();
        assert_eq!(music.events.len(), 3);
        music.playing = false;
        console.set_credits(true, 4, Some(&mut music)).unwrap();
        assert_eq!(&music.events[3..], ["stop 2 0", "play c4"]);
        console.set_credits(false, 0, Some(&mut music)).unwrap();
        assert_eq!(&music.events[5..], ["stop 3 2", "play m0", "sfx 0.75"]);
        assert!(!console.credits_active);
        assert_eq!(music.sound, 0.75);
        console.set_credits(false, 0, Some(&mut music)).unwrap();
        assert_eq!(music.events.len(), 8);
        let ini = killer_ui::ini::parse_ini("[Credit]\nCreditSound2=track\n");
        let names = credit_sound_names(&ini);
        assert_eq!((names[0].as_str(), names[1].as_str()), ("intro", "track"));
    }

    #[test]
    fn set_state_1042e050_plays_per_state_and_stops_previous_handle() {
        let mut console = new_console();
        let mut music = Music::default();
        for state in [2, 2, 4, 3, 4, 6, 2] {
            console.set_state(state, Some(&mut music)).unwrap();
        }
        assert_eq!(music.events, ["play m1", "stop 1", "play m2", "stop 2", "play m1", "stop 3", "play m0"]);
        assert_eq!((console.state_38, console.selected_48, console.handle_40), (2, 0, 4));
    }

    #[test]
    fn set_state_1042e050_initial_state_two_does_not_replay_login_music() {
        let mut console = new_console();
        let mut music = Music::default();
        console.set_state(2, Some(&mut music)).unwrap();
        assert!(music.events.is_empty());
        assert_eq!(console.selected_48, -1);
    }

    #[test]
    fn set_state_1042e050_loading_saves_previous_and_redirects_next_state() {
        let mut console = new_console();
        let mut music = Music::default();
        console.set_state(4, Some(&mut music)).unwrap();
        console.set_state(7, Some(&mut music)).unwrap();
        assert_eq!((console.state_38, console.previous_3c), (7, 4));
        console.set_state(6, Some(&mut music)).unwrap();
        assert_eq!((console.state_38, console.previous_3c), (7, 6));
        assert_eq!(music.events, ["play m1"]);
    }

    #[test]
    fn set_state_1042e050_failure_keeps_old_handle_but_records_selection() {
        let mut console = new_console();
        let mut music = Music { fail: true, ..Default::default() };
        console.handle_40 = 5;
        assert!(console.set_state(4, Some(&mut music)).is_err());
        assert_eq!(music.events, ["stop 5", "play m1"]);
        assert_eq!((console.state_38, console.selected_48, console.handle_40), (4, 1, 5));
    }

    #[test]
    fn play_intro_music_1042d930_only_without_a_handle() {
        let mut console = new_console();
        let mut music = Music::default();
        console.play_intro_music(Some(&mut music)).unwrap();
        console.play_intro_music(Some(&mut music)).unwrap();
        assert_eq!(music.events, ["play m0"]);
        assert_eq!((console.handle_40, console.selected_48), (1, 0));
        let mut silent = new_console();
        silent.play_intro_music(None::<&mut Music>).unwrap();
        assert_eq!(silent.handle_40, -1);
    }

    #[test]
    fn set_current_making_race_1042da20_keeps_the_native_selection_table() {
        let mut console = new_console();
        let mut music = Music::default();
        console.set_current_making_race(1, Some(&mut music)).unwrap();
        assert!(music.events.is_empty());
        assert_eq!(console.create_44, -1);
        console.set_state(3, Some(&mut music)).unwrap();
        music.events.clear();
        for (race, index, selected) in [(0, 3, 3), (1, 4, 4), (2, 5, 2), (3, 6, 3), (4, 7, 4)] {
            console.set_current_making_race(race, Some(&mut music)).unwrap();
            assert_eq!(music.events.last().unwrap(), &format!("play m{index}"));
            assert_eq!((console.selected_48, console.create_44), (selected, race));
            let before = music.events.len();
            console.set_current_making_race(race, Some(&mut music)).unwrap();
            assert_eq!(music.events.len(), before);
        }
        console.set_current_making_race(9, Some(&mut music)).unwrap();
        assert_eq!(console.create_44, 9);
        console.set_current_making_race(0, None::<&mut Music>).unwrap();
        assert_eq!(console.create_44, 0);
    }

    #[test]
    fn set_state_1042e050_without_audio_only_moves_state() {
        let mut console = new_console();
        for state in [2, 4, 3, 6, 7, 4] {
            console.set_state(state, None::<&mut Music>).unwrap();
        }
        assert_eq!((console.state_38, console.previous_3c), (7, 4));
        assert_eq!((console.selected_48, console.handle_40, console.create_44), (-1, -1, -1));
    }
}

#[cfg(test)]
mod credit_exit_tests {
    use super::*;

    struct Silent {
        events: Vec<String>,
    }

    impl LobbyMusic for Silent {
        fn music_playing(&self) -> bool { false }
        fn sound_volume(&self) -> f32 { 0.5 }
        fn set_sound_volume(&mut self, volume: f32) { self.events.push(format!("sfx {volume}")); }
        fn stop_music(&mut self, handle: i32, fade: f32) { self.events.push(format!("stop {handle} {fade}")); }
        fn play_music(&mut self, _request: LobbyMusicRequest<'_>) -> Result<i32, AudioError> {
            self.events.push("play".to_owned());
            Ok(0)
        }
    }

    #[test]
    fn without_a_credit_handle_the_exit_block_of_10083fe0_is_skipped_and_the_sfx_stays_muted() {
        let mut console = LobbyConsole::new(std::array::from_fn(|index| format!("m{index}")));
        let mut music = Silent { events: Vec::new() };
        console.set_credits(true, 0, Some(&mut music)).unwrap();
        console.set_credits(false, 0, Some(&mut music)).unwrap();
        assert_eq!(music.events, ["sfx 0", "play"]);
        assert!(!console.credits_active);
    }
}
