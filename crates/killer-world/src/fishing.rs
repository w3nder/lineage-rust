pub const TOOK_BAIT_SYSSTRING: u32 = 1261;
pub const SUCCESS_SYSSTRING: u32 = 1264;
pub const PUMPING_SUCCESS_SYSSTRING: u32 = 1256;
pub const REELING_SUCCESS_SYSSTRING: u32 = 1257;
pub const PUMPING_FAILED_SYSSTRING: u32 = 1258;
pub const REELING_FAILED_SYSSTRING: u32 = 1259;
pub const PUMPING_PENALTY_SYSMSG: u32 = 1672;
pub const REELING_PENALTY_SYSMSG: u32 = 1671;
pub const EXHAUSTED_SYSMSG: u32 = 1665;
pub const RESISTING_SYSMSG: u32 = 1664;
pub const ACTION_PUMPING: u8 = 1;
pub const ACTION_REELING: u8 = 2;
pub const ACTION_BAIT: u8 = 3;
pub const ACTION_SUCCESS: u8 = 4;
const MESSAGE_SECONDS: f32 = 2.0;
const CLOSE_SECONDS: f32 = 2.0;
const START_DELAY: i32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FishText {
    SysString(u32),
    SystemMessage { id: u32, penalty: i32 },
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FishWindow {
    pub visible: bool,
    pub start_hp: i32,
    pub hp: i32,
    pub start_time: i32,
    pub time: i32,
    pub started: bool,
    pub effect_alpha: u8,
    pub effect_on: bool,
    pub effect_time: f32,
    pub close_timer: f32,
    pub message: Option<FishText>,
    pub value: i32,
    pub message_timer: f32,
    pub message_positive: bool,
    pub hint: bool,
    pub hint_time: f32,
    pub deceptive: bool,
    pub mode: u8,
}

impl FishWindow {
    pub fn reset(&mut self) {
        let visible = self.visible;
        *self = Self { visible, ..Self::default() };
    }

    pub fn combat(&mut self, hp: i32, time: i32, deceptive: bool) {
        self.start_hp = hp;
        self.hp = hp;
        self.start_time = time;
        self.time = time;
        self.deceptive = deceptive;
        self.started = false;
        self.value = 0;
        self.message_timer = 0.0;
        self.message_positive = false;
        self.hint = false;
        self.hint_time = 0.0;
        self.mode = 0;
        self.message = None;
    }

    fn effect(&mut self) {
        self.effect_on = true;
        self.effect_alpha = 255;
        self.effect_time = 0.0;
    }

    fn sysstring(&mut self, id: u32, value: i32) {
        self.message = Some(FishText::SysString(id));
        self.value = value;
        self.message_timer = MESSAGE_SECONDS;
        self.message_positive = value > 0;
    }

    fn system_message(&mut self, id: u32, penalty: i32, value: i32) {
        self.message = Some(FishText::SystemMessage { id, penalty });
        self.value = value;
        self.message_timer = MESSAGE_SECONDS;
        self.message_positive = true;
    }

    pub fn update(&mut self, hp: i32, time: i32, hit: bool, mode: u8, action: u8, penalty: i32) {
        let delta = hp - self.hp;
        if mode != self.mode {
            self.mode = mode;
            self.hint_time = 0.0;
        }
        if action != ACTION_SUCCESS {
            self.hp = hp;
            self.time = time;
        }
        if !self.started && self.start_time - self.time == START_DELAY {
            self.started = true;
            if !self.deceptive {
                self.hint = true;
            }
        }
        match action {
            ACTION_BAIT => self.sysstring(TOOK_BAIT_SYSSTRING, 0),
            ACTION_SUCCESS => {
                self.hint = false;
                self.sysstring(SUCCESS_SYSSTRING, 0);
            }
            _ => {}
        }
        if !self.started {
            return;
        }
        if hit {
            self.effect();
        }
        let (penalty_message, success, failed) = match action {
            ACTION_PUMPING => (PUMPING_PENALTY_SYSMSG, PUMPING_SUCCESS_SYSSTRING, PUMPING_FAILED_SYSSTRING),
            ACTION_REELING => (REELING_PENALTY_SYSMSG, REELING_SUCCESS_SYSSTRING, REELING_FAILED_SYSSTRING),
            _ => return,
        };
        if delta < 0 {
            if penalty > 0 {
                self.system_message(penalty_message, penalty, delta);
            } else {
                self.sysstring(success, delta);
            }
        } else {
            self.sysstring(failed, delta);
        }
    }

    pub fn success(&mut self) {
        self.close_timer = CLOSE_SECONDS;
        if self.started {
            self.effect();
        }
    }

    pub fn tick(&mut self, dt: f32) {
        if !self.visible {
            return;
        }
        if self.effect_on {
            self.effect_time += dt;
            let t = self.effect_time;
            if t <= 0.04 {
                self.effect_alpha = 255;
            } else if t <= 0.08 {
                self.effect_alpha = 0;
            } else if t <= 0.095 {
                self.effect_alpha = ((255.0 - (t - 0.05) * 10000.0) as i32) as u8;
            } else if t <= 0.2 {
                self.effect_alpha = ((105.0 - (t - 0.08) * 500.0) as i32) as u8;
            } else {
                self.effect_on = false;
                self.effect_time = 0.0;
                self.effect_alpha = 0;
            }
        }
        if self.hint {
            self.hint_time += dt;
        }
        if self.message_timer > 0.0 {
            self.message_timer = (self.message_timer - dt).max(0.0);
        }
        if self.close_timer > 0.0 {
            self.close_timer -= dt;
            if self.close_timer <= 0.0 {
                self.close_timer = 0.0;
                self.visible = false;
            }
        }
    }

    pub fn hint_shown(&self) -> bool {
        let t = self.hint_time;
        self.hint && ((0.0..0.1).contains(&t) || (0.2..0.3).contains(&t) || t >= 0.4)
    }

    pub fn message_alpha(&self) -> u8 {
        if self.message_timer >= 1.0 { 255 } else { (self.message_timer * 255.0) as u8 }
    }

    pub fn bar_width(&self) -> i32 {
        if self.start_hp == 0 { 0 } else { self.hp * 115 / self.start_hp }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combat_starts_only_when_the_timer_passed_exactly_three_seconds() {
        let mut window = FishWindow { visible: true, ..FishWindow::default() };
        window.combat(100, 30, false);
        window.update(100, 30, false, 0, ACTION_BAIT, 0);
        assert_eq!(window.message, Some(FishText::SysString(TOOK_BAIT_SYSSTRING)));
        assert!(!window.started);
        window.update(100, 28, false, 0, 0, 0);
        assert!(!window.started);
        window.update(100, 27, false, 0, 0, 0);
        assert!(window.started && window.hint);
        window.update(90, 26, true, 0, ACTION_PUMPING, 0);
        assert_eq!((window.message, window.value, window.effect_on), (Some(FishText::SysString(PUMPING_SUCCESS_SYSSTRING)), -10, true));
        window.update(95, 25, false, 1, ACTION_REELING, 0);
        assert_eq!((window.message, window.value, window.mode), (Some(FishText::SysString(REELING_FAILED_SYSSTRING)), 5, 1));
        window.update(80, 24, false, 1, ACTION_REELING, 15);
        assert_eq!(window.message, Some(FishText::SystemMessage { id: REELING_PENALTY_SYSMSG, penalty: 15 }));
        assert_eq!(window.bar_width(), 80 * 115 / 100);
    }

    #[test]
    fn deceptive_fish_hide_the_hint_and_success_closes_after_two_seconds() {
        let mut window = FishWindow { visible: true, ..FishWindow::default() };
        window.combat(50, 10, true);
        window.update(50, 7, false, 0, 0, 0);
        assert!(window.started && !window.hint);
        window.update(0, 0, false, 0, ACTION_SUCCESS, 0);
        assert_eq!((window.hp, window.time), (50, 7));
        window.success();
        window.tick(1.0);
        assert!(window.visible);
        window.tick(1.0);
        assert!(!window.visible);
    }

    #[test]
    fn hint_blinks_three_times_then_stays() {
        let mut window = FishWindow { visible: true, hint: true, ..FishWindow::default() };
        let mut shown = Vec::new();
        for _ in 0..10 {
            shown.push(window.hint_shown());
            window.tick(0.05);
        }
        assert_eq!(shown, [true, true, false, false, true, true, false, false, true, true]);
    }

    #[test]
    fn effect_flashes_and_fades_out() {
        let mut window = FishWindow { visible: true, ..FishWindow::default() };
        window.effect();
        window.tick(0.03);
        assert_eq!(window.effect_alpha, 255);
        window.tick(0.03);
        assert_eq!(window.effect_alpha, 0);
        window.tick(0.04);
        assert_eq!(window.effect_alpha, ((105.0 - (window.effect_time - 0.08) * 500.0) as i32) as u8);
        window.tick(0.2);
        assert!(!window.effect_on);
    }
}
