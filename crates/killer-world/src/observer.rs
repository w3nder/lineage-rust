pub const TELEPORT_FADE_ID: i32 = 1;
pub const FADE_MS: f32 = 1000.0;
pub const FADE_COLOR: [u8; 4] = [0, 0, 0, 255];
pub const TELEPORT_RISE: f32 = 10.0;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObserverControl {
    pub player_observer_mode: bool,
    pub teleport_loc: Option<[f32; 3]>,
    pub ctrl_observer: bool,
    pub ctrl_broadcast: bool,
    pub fade_out_pending: bool,
    pub camera_default_pending: bool,
}

impl ObserverControl {
    pub fn teleport(&mut self, x: f32, y: f32, z: f32, collision_height: f32) {
        self.teleport_loc = Some([x, y, z + collision_height + TELEPORT_RISE]);
        self.fade_out_pending = true;
    }

    pub fn start(&mut self, values: [i32; 5], collision_height: f32) -> i32 {
        let [x, y, z, yaw, _] = values;
        self.teleport(x as f32, y as f32, z as f32, collision_height);
        self.player_observer_mode = true;
        self.ctrl_observer = true;
        self.camera_default_pending = true;
        yaw
    }

    pub fn end(&mut self, values: [i32; 3], collision_height: f32) {
        let [x, y, z] = values;
        self.teleport(x as f32, y as f32, z as f32, collision_height);
        self.player_observer_mode = false;
    }

    pub fn olympiad_mode(&mut self, mode: u8) {
        match mode {
            3 => self.ctrl_broadcast = true,
            0 => self.ctrl_broadcast = false,
            _ => {}
        }
    }

    pub fn fade_done(&mut self) -> Option<[f32; 3]> {
        self.ctrl_observer = self.player_observer_mode;
        self.teleport_loc.take()
    }

    pub fn hides_player(&self) -> bool {
        self.ctrl_observer || self.ctrl_broadcast
    }

    pub fn skips_physics(&self) -> bool {
        self.ctrl_observer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_teleports_above_the_target_by_collision_height_plus_ten_and_flags_the_controller_at_once() {
        let mut control = ObserverControl::default();
        let yaw = control.start([100, 200, 300, 16384, 7], 23.0);
        assert_eq!(yaw, 16384);
        assert_eq!(control.teleport_loc, Some([100.0, 200.0, 333.0]));
        assert!(control.player_observer_mode && control.ctrl_observer && control.fade_out_pending && control.camera_default_pending);
        assert!(control.hides_player() && control.skips_physics());
    }

    #[test]
    fn end_keeps_the_controller_flag_until_the_fade_finishes() {
        let mut control = ObserverControl::default();
        control.start([0, 0, 0, 0, 0], 10.0);
        control.fade_out_pending = false;
        control.end([5, 6, 7], 10.0);
        assert!(!control.player_observer_mode && control.ctrl_observer && control.fade_out_pending);
        assert_eq!(control.fade_done(), Some([5.0, 6.0, 27.0]));
        assert!(!control.ctrl_observer);
        assert_eq!(control.fade_done(), None);
    }

    #[test]
    fn the_olympiad_spectator_only_hides_the_player_without_stopping_physics() {
        let mut control = ObserverControl::default();
        control.olympiad_mode(3);
        assert!(control.hides_player() && !control.skips_physics());
        control.olympiad_mode(2);
        assert!(control.ctrl_broadcast);
        control.olympiad_mode(0);
        assert!(!control.hides_player());
    }
}
