use super::AudioError;
use glam::{Mat4,Vec3};
use killer_ui::ini::IniFile;

#[derive(Clone, Copy)]
pub struct Config {
    pub rolloff: f32,
    pub doppler: f32,
    pub reverse: bool,
    pub disable_pitch: bool,
}

impl Config {
    pub fn read(ini: &IniFile) -> Result<Self, AudioError> {
        let float = |key| ini.get("ALAudio.ALAudioSubsystem",key).map(super::config::native_atof)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData,format!("ALAudio.{key} ausente")));
        Ok(Self { rolloff:float("Rolloff")?,doppler:float("DopplerFactor")?,
            reverse:ini.bool("ALAudio.ALAudioSubsystem","ReverseStereo").unwrap_or(false),
            disable_pitch:ini.bool("ALAudio.ALAudioSubsystem","DisablePitch").unwrap_or(false) })
    }
}

pub fn coordinates(position: Vec3) -> [f32;3] {
    [position.x*0.04,position.y*0.04,-(position.z*0.04)]
}

pub fn orientation(camera_to_world: Mat4, reverse: bool) -> ([f32;3],[f32;3]) {
    let matrix = camera_to_world.to_cols_array_2d();
    let transform = |point: [f64;4]| std::array::from_fn::<_,3,_>(|axis| {
        (matrix[0][axis] as f64*point[0]+matrix[1][axis] as f64*point[1]+matrix[2][axis] as f64*point[2]+matrix[3][axis] as f64*point[3]) as f32
    });
    let front = transform([0.0,1000.0,0.0,0.0]);
    let right = transform([if reverse { -1000.0 } else { 1000.0 },0.0,0.0,0.0]);
    let up = std::array::from_fn(|axis| {
        let a = (axis+1)%3;
        let b = (axis+2)%3;
        (right[a] as f64*front[b] as f64-right[b] as f64*front[a] as f64) as f32
    });
    let up = normalize(up);
    (normalize(front),[up[0],up[1],-up[2]])
}

fn normalize(vector: [f32;3]) -> [f32;3] {
    let [x,y,z] = vector.map(f64::from);
    let size = x*x+y*y+z*z;
    if size < 1e-8 { return vector; }
    let inverse = 1.0/size.sqrt();
    vector.map(|value| (value as f64*inverse) as f32)
}

pub fn source_velocity(listener_velocity: Vec3) -> [f32;3] {
    let relative = -listener_velocity*0.04;
    if relative.length() > 300.0 {[0.0;3]} else {(relative*0.04).to_array()}
}
