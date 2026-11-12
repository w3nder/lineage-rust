use crate::color::Argb;
use crate::draw::DrawList;

pub const ROTATION_FRAMES: usize = 360;
pub const END_FRAMES: usize = 9;
pub const FLASH_SECONDS: f64 = 0.5;
pub const FLASH_COLOR: Argb = Argb(0xFF008080);
pub const ROTATION_PREFIX: &str = "L2_SkillTime.Rotation_Type.cooltime";
pub const END_PREFIX: &str = "L2_SkillTime.CooltimeEnd.CooltimeEnd00";
pub const SOUND_COOL_TIME_END: usize = 9;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoolFrame {
    Running(usize),
    Flash(usize),
}

impl CoolFrame {
    pub fn texture(self) -> String {
        match self {
            CoolFrame::Running(index) => format!("{ROTATION_PREFIX}{index}"),
            CoolFrame::Flash(index) => format!("{END_PREFIX}{}", index + 1),
        }
    }
}

fn frame_at(time: f64, frame_time: f64, first: usize, count: usize) -> Option<usize> {
    let mut frame_start = 0.0;
    for index in first..count {
        if frame_start <= time && (time < frame_start + frame_time || index == count - 1) {
            return Some(index);
        }
        frame_start += frame_time;
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoolTime {
    pub total: f64,
    pub remaining: f64,
}

impl CoolTime {
    pub fn running(self, elapsed: f64) -> Option<CoolFrame> {
        if !self.total.is_finite() || self.total <= 0.0 || self.total < self.remaining {
            return None;
        }
        let elapsed = elapsed.max(0.0);
        let remaining = self.ends_at();
        if elapsed > remaining {
            return None;
        }
        let frame_time = self.total / ROTATION_FRAMES as f64;
        let start_frame = ((self.total - remaining) / frame_time) as usize;
        let index = frame_at(elapsed as f32 as f64, frame_time, start_frame, ROTATION_FRAMES)?;
        Some(CoolFrame::Running(index))
    }

    pub fn flash(since_flash_start: f64) -> Option<CoolFrame> {
        let elapsed = since_flash_start.max(0.0);
        if elapsed > FLASH_SECONDS {
            return None;
        }
        let index = frame_at(elapsed as f32 as f64, FLASH_SECONDS / END_FRAMES as f64, 0, END_FRAMES)?;
        Some(CoolFrame::Flash(index))
    }

    pub fn ends_at(self) -> f64 {
        if self.remaining == 0.0 { self.total } else { self.remaining }
    }

    pub fn flash_window_open(self, elapsed: f64) -> bool {
        elapsed > self.ends_at() && elapsed <= self.ends_at() + FLASH_SECONDS
    }
}

pub fn paint(list: &mut DrawList, x: i32, y: i32, size: i32, frame: CoolFrame) {
    let texture = frame.texture();
    match frame {
        CoolFrame::Running(_) => list.draw_texture(x, y, size, size, 0, 0, size, size, &texture, 1.0),
        CoolFrame::Flash(_) => list.draw_brighten_texture(x, y, size, size, &texture, FLASH_COLOR),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rotation_picks_the_frame_of_the_elapsed_time_from_the_frames_already_gone() {
        let full = CoolTime { total: 36.0, remaining: 36.0 };
        assert_eq!(full.running(0.0), Some(CoolFrame::Running(0)));
        assert_eq!(full.running(0.099), Some(CoolFrame::Running(0)));
        assert_eq!(full.running(0.1), Some(CoolFrame::Running(1)));
        assert_eq!(full.running(35.99), Some(CoolFrame::Running(359)));
        assert_eq!(full.running(36.01), None);
        let half = CoolTime { total: 36.0, remaining: 18.0 };
        assert_eq!(half.running(0.0), Some(CoolFrame::Running(180)));
        assert_eq!(half.running(1.0), Some(CoolFrame::Running(190)));
        let unset = CoolTime { total: 36.0, remaining: 0.0 };
        assert_eq!(unset.running(0.0), Some(CoolFrame::Running(0)));
        assert_eq!(unset.ends_at(), 36.0);
        assert_eq!(CoolTime { total: 0.0, remaining: 0.0 }.running(0.0), None);
        assert_eq!(CoolTime { total: 5.0, remaining: 6.0 }.running(0.0), None);
    }

    #[test]
    fn the_end_flash_runs_nine_frames_over_half_a_second_and_only_opens_for_half_a_second_after_the_end() {
        assert_eq!(CoolTime::flash(0.0), Some(CoolFrame::Flash(0)));
        assert_eq!(CoolTime::flash(0.2), Some(CoolFrame::Flash(3)));
        assert_eq!(CoolTime::flash(0.499), Some(CoolFrame::Flash(8)));
        assert_eq!(CoolTime::flash(0.51), None);
        let cool = CoolTime { total: 10.0, remaining: 10.0 };
        assert!(!cool.flash_window_open(9.9) && !cool.flash_window_open(10.0));
        assert!(cool.flash_window_open(10.2) && cool.flash_window_open(10.5));
        assert!(!cool.flash_window_open(10.51));
    }

    #[test]
    fn the_frame_search_of_1001f190_adds_the_frame_time_and_the_last_frame_takes_the_rest() {
        assert_eq!(frame_at(0.0, 0.1, 0, 360), Some(0));
        assert_eq!(frame_at(0.35, 0.1, 0, 360), Some(3));
        assert_eq!(frame_at(1000.0, 0.1, 0, 360), Some(359));
        assert_eq!(frame_at(0.0, 0.1, 180, 360), Some(180));
        assert_eq!(frame_at(-1.0, 0.1, 0, 360), None);
        assert_eq!(frame_at(0.0, 0.1, 360, 360), None);
    }

    #[test]
    fn the_frame_names_are_the_ones_the_icon_loads() {
        assert_eq!(CoolFrame::Running(7).texture(), "L2_SkillTime.Rotation_Type.cooltime7");
        assert_eq!(CoolFrame::Flash(0).texture(), "L2_SkillTime.CooltimeEnd.CooltimeEnd001");
        assert_eq!(CoolFrame::Flash(8).texture(), "L2_SkillTime.CooltimeEnd.CooltimeEnd009");
    }
}
