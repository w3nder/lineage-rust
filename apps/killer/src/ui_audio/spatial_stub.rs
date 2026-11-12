use super::AudioError;
use glam::{Mat4, Vec3};

pub use super::spatial_state::{Config,coordinates,orientation};

pub struct Voice;

impl Voice {
    pub fn set_gain(&mut self, _: f32) -> Result<(), AudioError> { Ok(()) }
    pub fn gain(&self) -> f32 { 0.0 }
    pub fn stop(&mut self) {}
    pub fn set_position(&mut self, _: [f32;3]) -> Result<(), AudioError> { Ok(()) }
    pub fn set_velocity(&mut self, _: [f32;3]) -> Result<(), AudioError> { Ok(()) }
}

pub fn finished(_: &Voice) -> bool { true }

pub struct Spatial;

impl Spatial {
    pub fn open(_: Config) -> Result<Self, AudioError> {
        Err(std::io::Error::new(std::io::ErrorKind::Unsupported,"OpenAL indisponível no iOS").into())
    }

    pub fn listener(&self, _: Vec3, _: Mat4) -> Result<(), AudioError> { Ok(()) }

    pub fn voice(&mut self, _: &str, _: &[u8], _: Vec3, _: f32, _: f32, _: f32) -> Result<Voice, AudioError> {
        Err(std::io::Error::new(std::io::ErrorKind::Unsupported,"OpenAL indisponível no iOS").into())
    }
}
