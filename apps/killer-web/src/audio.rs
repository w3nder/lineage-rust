#[path = "../../killer/src/ui_audio/lobby.rs"]
pub mod lobby;
#[path = "../../killer/src/ui_audio/ogg.rs"]
pub mod ogg;
#[path = "../../killer/src/ui_audio/spatial_state.rs"]
pub mod spatial_state;
use glam::{Mat4,Vec3};
#[path = "../../killer/src/ui_audio/music_state.rs"]
pub mod music_state;
pub type AudioError = Box<dyn std::error::Error + Send + Sync>;
#[path = "../../killer/src/ui_audio/types.rs"]
pub mod types;
#[path = "../../killer/src/ui_audio/state.rs"]
pub mod state;
#[path = "../../killer/src/ui_audio/config.rs"]
pub mod config;

pub use types::{AudioVolumes, SourceIdentity};
pub use state::{AudioConfigReader, AudioFrame, InitConfig};
use state::{SourceGain, native_clamp, nonspatial_priority};

pub struct Voice {
    id: u32,
    identity: SourceIdentity,
    gain: SourceGain,
    priority: f32,
    output: f32,
    spatial: Option<(Vec3,(f32,f32))>,
}

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct SpatialParams {
    pub position: [f32;3],
    pub reference: f32,
    pub rolloff: f32,
    pub pitch: f32,
}

pub enum Command {
    Play { id: u32, encoded: Vec<u8>, gain: f32, spatial: Option<SpatialParams>, looping: bool, stream: bool },
    Listener {position: [f32;3],front: [f32;3],up: [f32;3]},
    Position {id: u32,position: [f32;3],pitch: f32},
    Stop(u32),
    Gain { id: u32, value: f32 },
}

pub struct Audio {
    pub console: Option<lobby::LobbyConsole>,
    intro_started: bool,
    pub config: InitConfig,
    sources: Vec<Option<Voice>>,
    commands: Vec<Command>,
    next: u32,
    available: bool,
    previous_time: f64,
    listener: Vec3,
    spatial_config: Option<spatial_state::Config>,
    pub controller: music_state::MusicController,
}

impl Audio {
    pub fn new(config: InitConfig, channels: i32) -> Self {
        Self {console:None,intro_started:false,config,sources:(0..channels.clamp(0,32)).map(|_|None).collect(),commands:Vec::new(),next:0,available:false,previous_time:0.0,listener:Vec3::ZERO,spatial_config:None,controller:music_state::MusicController::Lineage {field_a34:0,flags_a38:0}}
    }

    pub fn inherit(&mut self, mut previous: Self, reset_lobby: bool) {
        previous.spatial_config=self.spatial_config;
        if reset_lobby {
            previous.stop();
            previous.console=self.console.take();
            previous.intro_started=false;
        }
        *self=previous;
    }

    pub fn configure_lobby(&mut self, ini: &killer_ui::ini::IniFile) {
        self.console=Some(lobby::LobbyConsole::new(lobby::lobby_sound_names(ini)).with_credit_names(lobby::credit_sound_names(ini)));
    }

    pub fn set_credits(&mut self, root: &std::path::Path, frame: AudioFrame, visible: bool, roll: u32) -> Result<(),AudioError> {
        let Some(mut console)=self.console.take() else {return Ok(());};
        let mut backend=MusicBackend {audio:self,root,frame};
        let result=console.set_credits(visible,roll,Some(&mut backend));
        self.console=Some(console);
        result
    }

    pub fn update_lobby(&mut self, root: &std::path::Path, frame: AudioFrame, state: i32, race: Option<i32>) -> Result<(),AudioError> {
        self.update(Some(frame));
        if !self.available {return Ok(());}
        let Some(mut console)=self.console.take() else {return Ok(());};
        let mut backend=MusicBackend {audio:self,root,frame};
        let result=(|| {
            if !backend.audio.intro_started {
                backend.audio.intro_started=true;
                console.play_intro_music(Some(&mut backend))?;
            }
            if console.state_38!=state {console.set_state(state,Some(&mut backend))?;}
            if state==3 && let Some(race)=race {console.set_current_making_race(race,Some(&mut backend))?;}
            Ok(())
        })();
        self.console=Some(console);
        result
    }

    pub fn music(&mut self, root: &std::path::Path, request: music_state::LobbyMusicRequest<'_>, frame: AudioFrame) -> Result<i32,AudioError> {
        use std::io::Read;
        if !self.available || self.config.volumes.music<=0.0 || request.name.is_empty() {return Ok(0);}
        let path=root.join(if request.voice!=0 {"Voice"}else{"Music"}).join(format!("{}.ogg",request.name));
        let mut encoded=Vec::new();
        ogg::ClientOggReader::new(killer_platform::fs::File::open(path)?)?.read_to_end(&mut encoded)?;
        if self.sources.iter().flatten().filter(|voice|voice.identity.flags&4!=0).count()>=8 {return Ok(0);}
        let mut gain=SourceGain::new(native_clamp(self.config.volumes.music,0.0,1.0),request.fade);
        let priority=nonspatial_priority(frame,1000.0,gain.base,0x1c);
        let Some(index)=state::source_slot(self.config,self.sources.iter().map(|source|source.as_ref().map(|source|(source.identity.flags,source.priority))),priority) else {return Ok(0);};
        if let Some(old)=self.sources[index].take() {self.commands.push(Command::Stop(old.id));}
        self.next=self.next.wrapping_add(1);
        let id=self.next;
        let flags=0x1c | if request.looping!=0 {0x80}else{0} | if request.voice!=0 {0x100}else{0};
        let output=gain.initial_gain();
        gain.play_music_adjustment(request.volume);
        self.sources[index]=Some(Voice {id,identity:SourceIdentity {flags,actor:None,sound:Some(u64::from(id))},gain,priority,output,spatial:None});
        self.commands.push(Command::Play {id,encoded,gain:output,spatial:None,looping:request.looping!=0,stream:true});
        Ok(index as i32+1)
    }

    pub fn available(&self) -> bool {self.available}

    pub fn set_available(&mut self, available: bool) {
        self.available=available;
        if !available {
            self.stop();
            self.intro_started=false;
            if let Some(console)=self.console.take() {self.console=Some(lobby::LobbyConsole::new(console.names).with_credit_names(console.credit_names));}
        }
    }

    pub fn stop(&mut self) {
        for source in &mut self.sources {
            if let Some(voice)=source.take() {self.commands.push(Command::Stop(voice.id));}
        }
    }

    pub fn ended(&mut self,id: u32) {
        for source in &mut self.sources {
            if source.as_ref().is_some_and(|voice|voice.id==id) {*source=None;break;}
        }
    }

    pub fn drain(&mut self) -> Vec<Command> {std::mem::take(&mut self.commands)}

    pub fn interface(&mut self,encoded: Vec<u8>,frame: Option<AudioFrame>) -> i32 {
        if !self.available || self.config.volumes.sound<=0.0 {return 0;}
        let Some(frame)=frame else {return 0;};
        let gain=native_clamp(self.config.volumes.sound*self.config.volumes.sound,0.0,1.0);
        if gain==0.0 {return 0;}
        let priority=nonspatial_priority(frame,80.0,gain,0x10);
        let candidate=state::source_slot(self.config,self.sources.iter().map(|source|source.as_ref().map(|source|(source.identity.flags,source.priority))),priority);
        let Some(index)=candidate else{return 0;};
        if let Some(previous)=self.sources[index].take() {self.commands.push(Command::Stop(previous.id));}
        self.next=self.next.wrapping_add(1);
        let id=self.next;
        self.sources[index]=Some(Voice {id,identity:SourceIdentity {flags:0x10,actor:None,sound:Some(u64::from(id))},gain:SourceGain::new(gain,0.0),priority,output:gain,spatial:None});
        self.commands.push(Command::Play {id,encoded,gain,spatial:None,looping:false,stream:false});
        index as i32+1
    }

    pub fn configure_spatial(&mut self,config: spatial_state::Config) -> Result<(),AudioError> {
        if !config.doppler.is_finite() || config.doppler<0.0 || !config.rolloff.is_finite() {
            return Err(std::io::Error::other("configuração espacial inválida").into());
        }
        self.spatial_config=Some(config);
        Ok(())
    }

    pub fn pickup(&mut self,actor: u64,encoded: &[u8],position: Vec3,volume: f32,frame: Option<AudioFrame>) -> Result<i32,AudioError> {
        self.spatial(actor,None,encoded,position,volume,50.0,1.0,frame)
    }

    pub fn world(&mut self,sound: &crate::world_sound::WorldSound,frame: Option<AudioFrame>) -> Result<i32,AudioError> {
        self.spatial(sound.actor,Some(sound.key),&sound.encoded,sound.position,sound.volume,sound.radius,sound.pitch,frame)
    }

    pub fn is_playing(&self,actor: u64,sound: u64) -> bool {
        self.sources.iter().flatten().any(|voice|voice.identity.matches_sound(Some(actor),Some(sound)))
    }

    #[allow(clippy::too_many_arguments)]
    fn spatial(&mut self,actor: u64,key: Option<u64>,encoded: &[u8],position: Vec3,volume: f32,radius: f32,pitch: f32,frame: Option<AudioFrame>) -> Result<i32,AudioError> {
        if !self.available {return Ok(0);}
        let Some(frame)=frame else {return Ok(0);};
        let gain=native_clamp(volume*self.config.volumes.sound,0.0,1.0);
        if gain==0.0 {return Ok(0);}
        let priority=state::sound_priority(frame,position.to_array(),radius,gain,0);
        let candidate=state::source_slot(self.config,self.sources.iter().map(|source|source.as_ref().map(|source|(source.identity.flags,source.priority))),priority);
        let Some(index)=candidate else{return Ok(0);};
        if let Some(previous)=self.sources[index].take() {self.commands.push(Command::Stop(previous.id));}
        let config=self.spatial_config.ok_or_else(||std::io::Error::other("backend espacial indisponível"))?;
        let scaled=radius*frame.max_radius_multiplier;
        let distance=Vec3::from(frame.view_actor_location).distance(position);
        let initial=native_clamp((scaled-distance)*gain/scaled,0.0,1.0);
        self.next=self.next.wrapping_add(1);
        let id=self.next;
        self.sources[index]=Some(Voice {id,identity:SourceIdentity {flags:0,actor:Some(actor),sound:Some(key.unwrap_or(u64::from(id)))},gain:SourceGain::new(gain,0.0),priority,output:initial,spatial:Some((position,(distance,scaled)))});
        self.commands.push(Command::Play {id,encoded:encoded.to_vec(),gain:initial,looping:false,stream:false,spatial:Some(SpatialParams {position:spatial_state::coordinates(position),reference:radius.max(1.0)*0.04,rolloff:if config.rolloff<=0.0 {1.0}else{config.rolloff},pitch:pitch.clamp(0.5,2.0)})});
        Ok(index as i32+1)
    }

    pub fn update_spatial(&mut self,frame: Option<AudioFrame>,camera: Mat4,positions: &std::collections::HashMap<u64,Vec3>) -> Result<(),AudioError> {
        let Some(frame)=frame else {
            for slot in &mut self.sources {
                if slot.as_ref().is_some_and(|voice|voice.spatial.is_some())
                    && let Some(voice)=slot.take() {self.commands.push(Command::Stop(voice.id));}
            }
            return Ok(());
        };
        let listener=Vec3::from(frame.view_actor_location);
        let velocity=(listener-self.listener)/state::update_delta(frame.time,self.previous_time);
        self.listener=listener;
        let Some(config)=self.spatial_config else {return Ok(());};
        if self.available {
            let (front,up)=spatial_state::orientation(camera,config.reverse);
            self.commands.push(Command::Listener {position:spatial_state::coordinates(listener),front,up});
        }
        for slot in &mut self.sources {
            let Some(voice)=slot.as_mut().filter(|voice|voice.spatial.is_some()) else {continue;};
            let Some(position)=voice.identity.actor.and_then(|id|positions.get(&id)).copied() else {
                self.commands.push(Command::Stop(voice.id));
                *slot=None;
                continue;
            };
            voice.spatial=Some((position,(listener.distance(position),50.0*frame.max_radius_multiplier)));
            let pitch=match crate::doppler::pitch(spatial_state::coordinates(listener),spatial_state::coordinates(position),spatial_state::source_velocity(velocity),config.doppler) {
                Ok(pitch)=>pitch,
                Err(error)=>{
                    self.commands.push(Command::Stop(voice.id));
                    *slot=None;
                    return Err(std::io::Error::other(error).into());
                }
            };
            self.commands.push(Command::Position {id:voice.id,position:spatial_state::coordinates(position),pitch});
        }
        Ok(())
    }

    pub fn update(&mut self,frame: Option<AudioFrame>) {
        let Some(frame)=frame else {return;};
        if self.config.volumes.ui_values().iter().all(|value|*value<=0.0) {return;}
        let delta=state::update_delta(frame.time,self.previous_time);
        self.previous_time=frame.time;
        for slot in &mut self.sources {
            let Some(voice)=slot else {continue;};
            match voice.gain.update(delta,state::channel_gain(voice.identity,self.config.volumes),voice.spatial.map(|(_,attenuation)|attenuation)) {
                Some(value) if value!=voice.output => {
                    voice.output=value;
                    self.commands.push(Command::Gain {id:voice.id,value});
                }
                Some(_) => {}
                None => {
                    self.commands.push(Command::Stop(voice.id));
                    *slot=None;
                    continue;
                }
            }
            voice.priority=state::sound_priority(frame,voice.spatial.map_or(Vec3::ZERO,|(position,_)|position).to_array(),if voice.spatial.is_some(){50.0}else if voice.identity.flags&8!=0 {1000.0}else{80.0},voice.gain.base,voice.identity.flags);
        }
    }

    pub fn set_volume(&mut self,channel: killer_ui::option_audio::AudioChannel,value: f32) {
        use killer_ui::option_audio::AudioChannel;
        match channel {
            AudioChannel::Sound=>self.set_sound_volume(value),
            AudioChannel::Music=>self.config.volumes.music=value,
            AudioChannel::WavVoice=>self.config.volumes.wav_voice=value,
            AudioChannel::OggVoice=>self.config.volumes.ogg_voice=value,
        }
    }

    pub fn stop_channel(&mut self,channel: killer_ui::option_audio::AudioChannel) {
        for slot in &mut self.sources {
            if slot.as_ref().is_some_and(|source|source.identity.matches_channel(channel))
                && let Some(source)=slot.take() {self.commands.push(Command::Stop(source.id));}
        }
    }

    pub fn set_sound_volume(&mut self,value: f32) {
        let old=self.config.volumes.sound;
        self.config.volumes.sound=value;
        for voice in self.sources.iter_mut().flatten() {
            if voice.identity.is_sound() {
                voice.gain.base=types::rescaled_sound_gain(old,value,voice.gain.base);
                voice.output=voice.gain.base;
                self.commands.push(Command::Gain {id:voice.id,value:voice.gain.base});
            }
        }
    }
}


struct MusicBackend<'a> {
    audio: &'a mut Audio,
    root: &'a std::path::Path,
    frame: AudioFrame,
}

#[cfg(target_arch = "wasm32")]
pub struct WorldMusic<'a> {
    pub audio: &'a mut Audio,
    pub root: &'a std::path::Path,
    pub frame: AudioFrame,
}

#[cfg(target_arch = "wasm32")]
impl WorldMusic<'_> {
    fn playing(&self, mask: u32) -> bool {
        self.audio.sources.iter().flatten().any(|voice|voice.identity.flags&mask!=0)
    }

    fn voice_slot(&mut self, handle: i32) -> Option<&mut Option<Voice>> {
        handle.checked_sub(1).and_then(|index|usize::try_from(index).ok()).and_then(|index|self.audio.sources.get_mut(index))
    }
}

#[cfg(target_arch = "wasm32")]
impl crate::zone_music::MusicOut for WorldMusic<'_> {
    fn music_playing(&self) -> bool {self.playing(8)}
    fn ogg_voice_playing(&self) -> bool {self.playing(0x100)}
    fn wav_voice_playing(&self) -> bool {false}
    fn voice_finished(&self) -> bool {!self.playing(0x100)}
    fn play_music(&mut self, name: &str, volume: f32) -> i32 {
        let request=music_state::LobbyMusicRequest {name,fade:0.0,looping:0,voice:0,volume,force:0};
        self.audio.music(self.root,request,self.frame).unwrap_or_else(|error|{log::warn!("áudio: música {name}: {error}");0})
    }
    fn stop_music(&mut self, handle: i32, fade: f32) {
        let Some(slot)=self.voice_slot(handle) else {return;};
        if fade>0.0 {
            if let Some(voice)=slot.as_mut() {voice.gain.stop(fade);}
        } else if let Some(voice)=slot.take() {
            self.audio.commands.push(Command::Stop(voice.id));
        }
    }
    fn set_music_volume(&mut self, handle: i32, volume: f32, fade: f32) {
        let music=self.audio.config.volumes.music;
        if let Some(Some(voice))=self.voice_slot(handle) {voice.gain.set_music_volume(music,volume,fade);}
    }
    fn play_voice(&mut self, name: &str, ogg: bool) -> i32 {
        if !ogg {return 0;}
        let request=music_state::LobbyMusicRequest {name,fade:0.0,looping:0,voice:1,volume:1.0,force:0};
        self.audio.music(self.root,request,self.frame).unwrap_or_else(|error|{log::warn!("áudio: voz {name}: {error}");0})
    }
}

impl lobby::LobbyMusic for MusicBackend<'_> {
    fn music_playing(&self) -> bool {
        self.audio.sources.iter().flatten().any(|voice|voice.identity.flags&8!=0)
    }
    fn sound_volume(&self) -> f32 {
        self.audio.config.volumes.sound
    }
    fn set_sound_volume(&mut self, volume: f32) {
        self.audio.set_sound_volume(volume);
    }
    fn stop_music(&mut self, handle: i32, fade: f32) {
        let Some(slot)=handle.checked_sub(1).and_then(|index|usize::try_from(index).ok()).and_then(|index|self.audio.sources.get_mut(index)) else {return;};
        if fade>0.0 {
            if let Some(voice)=slot.as_mut() {voice.gain.stop(fade);}
        } else if let Some(voice)=slot.take() {
            self.audio.commands.push(Command::Stop(voice.id));
        }
    }
    fn play_music(&mut self, request: music_state::LobbyMusicRequest<'_>) -> Result<i32,AudioError> {
        self.audio.music(self.root,request,self.frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::ini::parse_ini;
    fn audio(channels: i32) -> Audio {
        let option=parse_ini("[Audio]\nSoundVolume=0.5\n");
        let alaudio=parse_ini("");
        let mut audio=Audio::new(InitConfig::read(&config::AudioIni {option:&option,alaudio:&alaudio}),channels);
        audio.set_available(true);
        audio
    }
    fn frame() -> AudioFrame {AudioFrame {time:0.0,view_actor_location:[0.0;3],max_radius_multiplier:50.0,realtime:false,replay_recording:false}}
    #[test]
    #[ignore]
    fn all_original_lobby_tracks_normalize_decode_and_follow_desktop_transitions() {
        let root=std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").unwrap());
        let mut client=killer_assets::client::Client::open(&root).unwrap();
        let ini=parse_ini(&client.read_system_text("alaudio.int").unwrap());
        let names=lobby::lobby_sound_names(&ini);
        let mut audio=audio(32);
        audio.config.volumes.music=0.8;
        let request=|name|music_state::LobbyMusicRequest {name,fade:1.5,looping:1,voice:0,volume:1.0,force:0};
        for name in &names {
            audio.stop();audio.drain();
            assert!(audio.music(&root,request(name),frame()).unwrap()>0);
            let commands=audio.drain();
            let [Command::Play {encoded,gain,looping,stream,..}]=commands.as_slice() else {panic!("comando de música ausente")};
            assert_eq!(*gain,0.0);
            assert!(*looping && *stream);
            assert_eq!(&encoded[..4],b"OggS");
            use std::io::Write;
            let mut decoder=std::process::Command::new("ffmpeg").args(["-v","error","-i","pipe:0","-f","null","-"])
                .stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::piped()).spawn().unwrap();
            decoder.stdin.take().unwrap().write_all(encoded).unwrap();
            let result=decoder.wait_with_output().unwrap();
            assert!(result.status.success(),"{name}: {}",String::from_utf8_lossy(&result.stderr));
        }
        audio.stop();audio.drain();
        audio.configure_lobby(&ini);
        audio.update_lobby(&root,frame(),2,None).unwrap();
        assert_eq!(audio.console.as_ref().unwrap().selected_48,0);
        assert!(matches!(&audio.drain()[..],[Command::Play {stream:true,..}]));
        for step in 1..=20 {audio.update(Some(AudioFrame {time:f64::from(step)*0.1,..frame()}));}
        let voice=audio.sources.iter().flatten().next().unwrap();
        assert!((voice.output-0.8).abs()<1e-6);
        audio.config.volumes.music=0.4;
        audio.update(Some(AudioFrame {time:2.1,..frame()}));
        assert!((audio.sources.iter().flatten().next().unwrap().output-0.4).abs()<1e-6);
        audio.drain();
        audio.update_lobby(&root,AudioFrame {time:2.2,..frame()},4,None).unwrap();
        assert_eq!(audio.console.as_ref().unwrap().selected_48,1);
        assert!(matches!(&audio.drain()[..],[Command::Stop(_),Command::Play {stream:true,..}]));
        let mut world=Audio::new(audio.config,32);
        world.inherit(audio,false);
        assert_eq!(world.sources.iter().flatten().count(),1);
        assert!(world.drain().is_empty());
        let mut lobby=Audio::new(world.config,32);
        lobby.configure_lobby(&ini);
        lobby.inherit(world,true);
        assert!(matches!(&lobby.drain()[..],[Command::Stop(_)]));
        lobby.update_lobby(&root,frame(),2,None).unwrap();
        assert!(matches!(&lobby.drain()[..],[Command::Play {stream:true,..}]));
    }

    #[test]
    fn interface_10476e70_obeys_frame_gain_channels_and_strict_priority() {
        let mut audio=audio(1);
        assert_eq!(audio.interface(vec![1],None),0);
        assert_eq!(audio.interface(vec![1],Some(frame())),1);
        assert!(matches!(&audio.drain()[..],[Command::Play {gain,..}] if *gain==0.25));
        assert_eq!(audio.interface(vec![2],Some(frame())),0);
        audio.config.volumes.sound=0.8;
        assert_eq!(audio.interface(vec![3],Some(frame())),1);
        assert!(matches!(&audio.drain()[..],[Command::Stop(1),Command::Play {id:2,..}]));
        audio.ended(1);
        assert_eq!(audio.interface(vec![4],Some(frame())),0);
        audio.ended(2);
        assert_eq!(audio.interface(vec![5],Some(frame())),1);
        audio.set_available(false);
        assert_eq!(audio.interface(vec![6],Some(frame())),0);
    }
    #[test]
    fn reserved_slots_and_zero_volume_do_not_play() {
        for channels in [-1,0,1,32,33] {
            let mut audio=audio(channels);
            audio.config.use_ambient_slot=true;
            audio.config.ambient_sound_slot=channels.max(0);
            assert_eq!(audio.interface(vec![1],Some(frame())),0);
            audio.config.use_ambient_slot=false;
            audio.config.volumes.sound=0.0;
            assert_eq!(audio.interface(vec![1],Some(frame())),0);
            assert!(audio.drain().is_empty());
        }
    }
    #[test]
    fn pickups_share_all_channels_follow_actors_and_release_without_stopping_interface() {
        use std::collections::HashMap;
        let mut audio=audio(32);
        audio.configure_spatial(spatial_state::Config {rolloff:0.5,doppler:1.0,reverse:false,disable_pitch:false}).unwrap();
        let mut positions=HashMap::new();
        for actor in 0..32 {
            positions.insert(actor,Vec3::X*100.0);
            assert_eq!(audio.pickup(actor,&[1],Vec3::X*100.0,1.0,Some(frame())).unwrap(),actor as i32+1);
        }
        assert_eq!(audio.pickup(32,&[1],Vec3::X*100.0,1.0,Some(frame())).unwrap(),0);
        assert!(audio.drain().iter().all(|command|matches!(command,Command::Play {gain,spatial:Some(spatial),..} if (*gain-0.48).abs()<1e-6 && spatial.position==[4.0,0.0,-0.0] && spatial.reference==2.0 && spatial.rolloff==0.5)));
        assert_eq!(audio.interface(vec![2],Some(frame())),1);
        assert!(matches!(&audio.drain()[..],[Command::Stop(1),Command::Play {id:33,spatial:None,..}]));
        let moved=AudioFrame {time:1.0,view_actor_location:[10.0,0.0,0.0],..frame()};
        audio.update_spatial(Some(moved),Mat4::IDENTITY,&positions).unwrap();
        audio.update(Some(moved));
        let commands=audio.drain();
        assert!(commands.iter().any(|command|matches!(command,Command::Position {id:2,position,pitch} if *position==[4.0,0.0,-0.0] && *pitch>1.0)));
        assert!(commands.iter().any(|command|matches!(command,Command::Gain {id:2,value} if (*value-0.482).abs()<1e-6)));
        positions.remove(&1);
        audio.update_spatial(Some(AudioFrame {time:2.0,..moved}),Mat4::IDENTITY,&positions).unwrap();
        assert!(audio.drain().iter().any(|command|matches!(command,Command::Stop(2))));
        audio.update_spatial(None,Mat4::IDENTITY,&positions).unwrap();
        assert_eq!(audio.sources.iter().flatten().count(),1);
        assert!(audio.sources.iter().flatten().all(|voice|voice.id==33));
        assert!(!audio.drain().iter().any(|command|matches!(command,Command::Stop(33))));
    }

    #[test]
    fn source_velocity_preserves_original_second_scale_and_teleport_limit() {
        assert_eq!(spatial_state::source_velocity(Vec3::new(100.0,-100.0,50.0)),[-0.16,0.16,-0.08]);
        assert_eq!(spatial_state::source_velocity(Vec3::X*8000.0),[0.0;3]);
        let mut audio=audio(1);
        assert!(audio.configure_spatial(spatial_state::Config {rolloff:0.5,doppler:-1.0,reverse:false,disable_pitch:false}).is_err());
    }

}
