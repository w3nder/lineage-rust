pub const RADAR_SIZE: (i32, i32) = (102, 102);

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    (crate::window_restore::x87_to_int(f64::from(viewport.0 as f32) - f64::from(RADAR_SIZE.0)), 0)
}

pub fn initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    let (x, y) = default_position(viewport);
    crate::initial_position::InitialPosition {
        enabled: true, horizontal_mode: 1, vertical_mode: 3,
        x: viewport.0.wrapping_sub(x), y,
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RadarView {
    pub camera_pitch: i32,
    pub camera_yaw: i32,
    pub camera_roll: i32,
    pub party: Vec<[f32; 2]>,
    pub target: Option<[f32; 2]>,
    pub arrow_to: Option<[f32; 3]>,
    pub points: Vec<[f32; 2]>,
    pub show_time: bool,
}

impl RadarView {
    pub fn camera(&self) -> [i32; 3] {
        [self.camera_pitch, self.camera_yaw, self.camera_roll]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_window_sits_in_the_top_right_corner() {
        assert_eq!(default_position((1024, 768)), (922, 0));
        let initial = initial_position((1024, 768));
        assert_eq!((initial.x, initial.y, initial.horizontal_mode, initial.vertical_mode), (102, 0, 1, 3));
        let view = RadarView { camera_pitch: -0x1000, camera_yaw: 0x4000, camera_roll: 7, ..Default::default() };
        assert_eq!(view.camera(), [-0x1000, 0x4000, 7]);
    }
}
