use super::AudioError;
use alto::{Alto, Buffer, Context, DistanceModel, Mono, Source as _, StaticSource, Stereo};
use glam::{Mat4, Vec3};
use rodio::{Decoder, Source as _};
use std::{collections::HashMap, io::Cursor, sync::{Arc, Weak}};

pub use super::spatial_state::{Config,coordinates,orientation};

pub type Voice = StaticSource;

pub fn finished(voice: &Voice) -> bool { voice.state() == alto::SourceState::Stopped }

pub struct Spatial {
    context: Context,
    config: Config,
    buffers: HashMap<String,Weak<Buffer>>,
}

impl Spatial {
    pub fn open(config: Config) -> Result<Self, AudioError> {
        let library = Alto::load_default();
        #[cfg(target_os = "macos")]
        let library = library.or_else(|_| Alto::load("/System/Library/Frameworks/OpenAL.framework/OpenAL"));
        let context = library?.open(None)?.new_context(None)?;
        context.set_distance_model(DistanceModel::InverseClamped);
        context.set_doppler_factor(config.doppler)?;
        Ok(Self { context,config,buffers:HashMap::new() })
    }

    pub fn listener(&self, position: Vec3, camera_to_world: Mat4) -> Result<(), AudioError> {
        let (front,up) = orientation(camera_to_world,self.config.reverse);
        self.context.set_position(coordinates(position))?;
        self.context.set_orientation((front,up))?;
        self.context.set_velocity([0.0;3])?;
        Ok(())
    }

    pub fn voice(&mut self, name: &str, encoded: &[u8], position: Vec3, gain: f32, radius: f32, pitch: f32) -> Result<StaticSource,AudioError> {
        self.buffers.retain(|_,buffer| buffer.strong_count() > 0);
        let buffer = match self.buffers.get(name).and_then(Weak::upgrade) {
            Some(buffer) => buffer,
            None => {
                let decoder = Decoder::new(Cursor::new(encoded.to_vec()))?;
                let frequency = decoder.sample_rate() as i32;
                let channels = decoder.channels();
                let samples: Vec<i16> = decoder.map(|sample| (sample*32768.0).round().clamp(-32768.0,32767.0) as i16).collect();
                let buffer = match channels {
                    1 => self.context.new_buffer::<Mono<i16>,_>(&samples,frequency)?,
                    2 => self.context.new_buffer::<Stereo<i16>,_>(&samples,frequency)?,
                    _ => return Err(std::io::Error::new(std::io::ErrorKind::Unsupported,format!("USound {name}: {channels} canais")).into()),
                };
                let buffer = Arc::new(buffer);
                self.buffers.insert(name.into(),Arc::downgrade(&buffer));
                buffer
            }
        };
        let mut voice = self.context.new_static_source()?;
        voice.set_buffer(buffer)?;
        voice.set_relative(false);
        voice.set_gain(gain)?;
        voice.set_reference_distance(radius.max(1.0)*0.04)?;
        voice.set_rolloff_factor(if self.config.rolloff <= 0.0 { 1.0 } else { self.config.rolloff })?;
        if !self.config.disable_pitch { voice.set_pitch(pitch.clamp(0.5,2.0))?; }
        voice.set_position(coordinates(position))?;
        voice.set_velocity([0.0;3])?;
        voice.play();
        Ok(voice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn listener_and_priority_match_original_alaudio() {
        use std::{io::Write,process::{Command,Stdio}};
        use serde_json::{json,Value};
        let mut cases = Vec::new();
        for pitch in [-1.2_f32,0.0,1.2] {
            for yaw in [0.0_f32,0.3,1.5,3.7] {
                let matrix = Mat4::from_rotation_z(yaw)*Mat4::from_rotation_y(pitch);
                for reverse in [false,true] { cases.push(json!({"matrix":matrix.to_cols_array(),"reverse":reverse})); }
            }
        }
        for listener in [[0.0;3],[83400.0,148000.0,-3430.0]] {
            for position in [[0.0;3],[83420.0,148100.0,-3450.0]] {
                for radius in [0.0,50.0,1000.0] {
                    for gain in [0.0,0.5,1.0] {
                        for flags in 0..=0x1ff { cases.push(json!({"listener":listener,"position":position,"radius":radius,"gain":gain,"flags":flags})); }
                    }
                }
            }
        }
        let script = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/pickup_audio_reference.py");
        let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
            .arg(script).arg(std::env::var("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"))
            .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
        writeln!(child.stdin.take().unwrap(),"{}",json!(cases)).unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        let expected: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(expected.len(),cases.len());
        for (index,(case,expected)) in cases.iter().zip(expected).enumerate() {
            if case.get("matrix").is_some() {
                let matrix = Mat4::from_cols_array(&serde_json::from_value(case["matrix"].clone()).unwrap());
                let (front,up) = orientation(matrix,case["reverse"].as_bool().unwrap());
                let values: [f32;6] = serde_json::from_value(expected).unwrap();
                for (axis,(got,want)) in front.into_iter().chain(up).zip(values).enumerate() {
                    assert_eq!(got.to_bits(),want.to_bits(),"orientation {index}, axis {axis}: {got} vs {want}");
                }
            } else {
                let frame = super::super::AudioFrame { time:0.0,view_actor_location:serde_json::from_value(case["listener"].clone()).unwrap(),max_radius_multiplier:50.0,realtime:true,replay_recording:false };
                let got = super::super::state::sound_priority(frame,serde_json::from_value(case["position"].clone()).unwrap(),case["radius"].as_f64().unwrap() as f32,case["gain"].as_f64().unwrap() as f32,case["flags"].as_u64().unwrap() as u32);
                assert_eq!(got.to_bits(),expected.as_u64().unwrap() as u32,"priority {index}: {case}");
            }
        }
        eprintln!("{} casos de orientação/prioridade comparados à alaudio.dll",cases.len());
    }

    #[test]
    #[ignore]
    fn pickup_voices_share_channel_limit_follow_actor_and_release_on_world_exit() {
        use super::super::{AudioDevice,AudioFrame,AudioVolumes,SourceIdentity,state::{InitConfig,SourceGain}};
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let mut client = killer_assets::client::Client::open(&root).unwrap();
        let ini = killer_ui::ini::parse_ini(&client.read_system_text("l2.ini").unwrap());
        let mut audio = AudioDevice::open(InitConfig { volumes:AudioVolumes { sound:1.0,music:1.0,wav_voice:1.0,ogg_voice:1.0 },use_ambient_slot:false,ambient_sound_slot:0,use_voice_slot:false,voice_sound_slot:0,use_release:true },32).unwrap();
        audio.configure_spatial(Config::read(&ini).unwrap()).unwrap();
        audio.spatial.as_ref().unwrap().context.set_gain(0.0).unwrap();
        let name = killer_assets::pickup::drop::THROW_SOUND;
        let (_,encoded) = client.sound(name).unwrap();
        let sound = Arc::new(killer_assets::pickup::assets::Sound { name:name.into(),encoded:encoded.into() });
        let frame = AudioFrame { time:1.0,view_actor_location:[0.0;3],max_radius_multiplier:50.0,realtime:true,replay_recording:false };
        for actor in 1..=32 {
            let request = crate::items::SoundRequest { actor,sound:sound.clone(),position:Vec3::ZERO,volume:1.0 };
            assert_eq!(audio.play_pickup(&request,Some(frame)).unwrap(),actor as i32);
        }
        assert_eq!(audio.play_pickup(&crate::items::SoundRequest { actor:33,sound:sound.clone(),position:Vec3::ZERO,volume:1.0 },Some(frame)).unwrap(),0);
        let music = rodio::buffer::SamplesBuffer::new(1,44100,vec![0.0;44100]);
        let handle = audio.append(music,SourceIdentity { flags:0x1c,actor:None,sound:Some(100) },SourceGain::new(1.0,0.0),4.0,None).unwrap();
        assert_eq!(handle,1);
        assert_eq!(audio.sources.iter().flatten().count(),32);
        let positions = (2..=32).map(|actor| (actor,Vec3::new(25.0,50.0,75.0))).collect();
        audio.update_spatial(Some(frame),Mat4::IDENTITY,&positions).unwrap();
        let source = audio.sources[1].as_ref().unwrap();
        let super::super::Playback::Spatial(voice) = &source.sink else { panic!("voz espacial"); };
        assert_eq!(voice.borrow().position::<[f32;3]>(),[1.0,2.0,-3.0]);
        assert_eq!(source.attenuation,Some((Vec3::new(25.0,50.0,75.0).length(),2500.0)));
        audio.update_spatial(None,Mat4::IDENTITY,&HashMap::new()).unwrap();
        assert_eq!(audio.sources.iter().flatten().count(),1);
        assert!(audio.sources[0].as_ref().unwrap().spatial_position.is_none());
        assert!(audio.spatial.as_ref().unwrap().buffers.values().all(|buffer| buffer.strong_count() == 0));
    }

    #[test]
    #[ignore]
    fn original_pickup_sound_uses_openal_position_gain_and_shared_pcm() {
        let root = std::path::PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let mut client = killer_assets::client::Client::open(&root).unwrap();
        let system = killer_ui::ini::parse_ini(&client.read_system_text("l2.ini").unwrap());
        let config = Config::read(&system).unwrap();
        let mut spatial = Spatial::open(config).unwrap();
        let name = killer_assets::pickup::drop::THROW_SOUND;
        let (_,encoded) = client.sound(name).unwrap();
        let location = Vec3::new(83400.0,148000.0,-3430.0);
        spatial.listener(location,Mat4::IDENTITY).unwrap();
        let mut first = spatial.voice(name,&encoded,location,0.0,50.0,1.0).unwrap();
        let mut second = spatial.voice(name,&encoded,location+Vec3::X*50.0,0.0,50.0,1.0).unwrap();
        assert_eq!(first.position::<[f32;3]>(),coordinates(location));
        assert_eq!(first.reference_distance(),2.0);
        assert_eq!(first.rolloff_factor(),config.rolloff);
        assert!(!first.relative());
        assert_eq!(first.gain(),0.0);
        assert_eq!(spatial.buffers.len(),1);
        assert_eq!(spatial.buffers[name].strong_count(),2);
        first.stop();
        second.stop();
        drop(first);
        drop(second);
        assert_eq!(spatial.buffers[name].strong_count(),0);
    }
}
