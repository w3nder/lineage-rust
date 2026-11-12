use std::f32::consts::{FRAC_PI_2, TAU};

pub const OT_PAN: u8 = 0;
pub const OT_STRETCH: u8 = 1;
pub const OT_STRETCH_REPEAT: u8 = 2;
pub const OT_JITTER: u8 = 3;
pub const TR_FIXED: u8 = 0;
pub const TR_CONSTANTLY: u8 = 1;
pub const TR_OSCILLATING: u8 = 2;

const PAN_WRAP: f32 = 1024.0;
const ROTATOR_TURN: f32 = 65536.0;
const ROTATOR_MASK: i32 = 0xffff;
const FRAME_RATE_MIN: f32 = 0.1;
const FRAME_RATE_MAX: f32 = 100.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat3x2(pub [[f32; 3]; 3]);

impl Mat3x2 {
    pub const IDENTITY: Self = Self([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);

    pub fn translation(u: f32, v: f32) -> Self {
        Self([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [u, v, 1.0]])
    }

    pub fn apply(self, [u, v]: [f32; 2]) -> [f32; 2] {
        let m = self.0;
        [u * m[0][0] + v * m[1][0] + m[2][0], u * m[0][1] + v * m[1][1] + m[2][1]]
    }

    pub fn apply3(self, [x, y, z]: [f32; 3]) -> [f32; 3] {
        let m = self.0;
        std::array::from_fn(|j| x * m[0][j] + y * m[1][j] + z * m[2][j])
    }

    pub fn then(self, next: Self) -> Self {
        let (a, b) = (self.0, next.0);
        Self(std::array::from_fn(|i| {
            std::array::from_fn(|j| a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j])
        }))
    }

    fn recentered(self, offset: [f32; 2], size: [f32; 2]) -> Self {
        if offset == [0.0, 0.0] {
            return self;
        }
        let c = [texels(offset[0], size[0]), texels(offset[1], size[1])];
        Self::translation(-c[0], -c[1]).then(self).then(Self::translation(c[0], c[1]))
    }
}

fn texels(offset: f32, size: f32) -> f32 {
    if size == 0.0 { 0.0 } else { offset / size }
}

fn frac(x: f32) -> f32 {
    x - x.floor()
}

fn turns(units: i32) -> f32 {
    units as f32 * TAU / ROTATOR_TURN
}

fn rotator_sin_cos(units: i32) -> (f32, f32) {
    match units.rem_euclid(ROTATOR_TURN as i32) {
        0 => (0.0, 1.0),
        16384 => (1.0, 0.0),
        32768 => (0.0, -1.0),
        49152 => (-1.0, 0.0),
        wrapped => turns(wrapped).sin_cos(),
    }
}

fn ftol(x: f32) -> i32 {
    x as i32
}

pub fn panner(direction: [i32; 3], rate: f32, t: f32) -> Mat3x2 {
    let cp = rotator_sin_cos(direction[0]).1;
    let (sy, cy) = rotator_sin_cos(direction[1]);
    let dir = [cp * cy, cp * sy];
    let wrap = |axis: f32| frac(axis * rate * t / PAN_WRAP) * PAN_WRAP;
    Mat3x2::translation(wrap(dir[0]), wrap(dir[1]))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trigger {
    pub stop_after: f32,
    pub triggered_time: f32,
    pub reverse: bool,
}

impl Trigger {
    pub fn time(&self, rate: f32, time: f32) -> f32 {
        if 0.0 <= self.triggered_time {
            let duration = self.stop_after / rate;
            let elapsed = time - self.triggered_time;
            if !self.reverse {
                if duration < elapsed { duration } else { elapsed }
            } else {
                let remaining = duration - elapsed;
                if remaining < 0.0 { 0.0 } else { remaining }
            }
        } else {
            0.0
        }
    }
}

pub fn oscillator_trigger_rate(rate: [f32; 2]) -> f32 {
    if rate[1] <= rate[0] { rate[0] } else { rate[1] }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Oscillator {
    pub rate: [f32; 2],
    pub phase: [f32; 2],
    pub amplitude: [f32; 2],
    pub kind: [u8; 2],
    pub offset: [f32; 2],
}

impl Oscillator {
    pub fn matrix(&self, size: [f32; 2], t: f32) -> Mat3x2 {
        let mut m = Mat3x2::IDENTITY;
        for axis in 0..2 {
            let s = t * self.rate[axis];
            let (phase, amplitude) = (self.phase[axis], self.amplitude[axis]);
            match self.kind[axis] {
                OT_PAN => m.0[2][axis] = ((frac(s) + phase) * TAU).sin() * amplitude,
                OT_STRETCH => m.0[axis][axis] = ((frac(s) + phase) * TAU).sin() * amplitude + 1.0,
                OT_STRETCH_REPEAT => m.0[axis][axis] = (phase * TAU + frac(s) * FRAC_PI_2).sin() * amplitude + 1.0,
                OT_JITTER => m.0[2][axis] = jitter(s, phase) * amplitude,
                _ => {}
            }
        }
        m.recentered(self.offset, size)
    }
}

fn jitter(s: f32, phase: f32) -> f32 {
    let period = (s - phase).floor() as i64;
    let mut hash = (period as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    hash ^= hash >> 31;
    hash = hash.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    hash ^= hash >> 29;
    (hash >> 40) as f32 / (1u64 << 24) as f32
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rotator {
    pub kind: u8,
    pub rotation: [i32; 3],
    pub offset: [f32; 2],
    pub oscillation_rate: [i32; 3],
    pub oscillation_amplitude: [i32; 3],
    pub oscillation_phase: [i32; 3],
}

impl Rotator {
    pub fn matrix(&self, size: [f32; 2], t: f32) -> Mat3x2 {
        let rotation = match self.kind {
            TR_CONSTANTLY => self.rotation.map(|c| ftol(c as f32 * t) & ROTATOR_MASK),
            TR_OSCILLATING => {
                let mut rotation = self.rotation;
                for (axis, component) in rotation.iter_mut().enumerate() {
                    let f = self.oscillation_rate[axis] as f32 * t / ROTATOR_TURN;
                    let swing = ((frac(f) + self.oscillation_phase[axis] as f32) * TAU).sin()
                        * self.oscillation_amplitude[axis] as f32;
                    *component = component.wrapping_add(ftol(swing));
                }
                rotation
            }
            _ => self.rotation,
        };
        rotation_matrix(rotation).recentered(self.offset, size)
    }
}

fn rotation_matrix([pitch, yaw, roll]: [i32; 3]) -> Mat3x2 {
    let (sp, cp) = rotator_sin_cos(pitch);
    let (sy, cy) = rotator_sin_cos(yaw);
    let (sr, cr) = rotator_sin_cos(roll);
    Mat3x2([
        [cy * cp, cp * sy, sp],
        [cy * sp * sr - cr * sy, sy * sp * sr + cy * cr, -cp * sr],
        [-(sy * sr + cy * cr * sp), cy * sr - cr * sy * sp, cp * cr],
    ])
}

pub fn scaler(scale: [f32; 2], offset: [f32; 2], size: [f32; 2]) -> Mat3x2 {
    Mat3x2([
        [1.0 / scale[0], 0.0, 0.0],
        [0.0, 1.0 / scale[1], 0.0],
        [texels(offset[0], size[0]), texels(offset[1], size[1]), 1.0],
    ])
}

pub fn texture_matrix(rows: [[f32; 4]; 3]) -> Mat3x2 {
    Mat3x2(std::array::from_fn(|r| std::array::from_fn(|c| rows[r][c])))
}

pub fn frame_index(t: f32, max_frame_rate: f32, total_frames: u32) -> u32 {
    if max_frame_rate == 0.0 || total_frames == 0 {
        return 0;
    }
    let rate = f64::from(max_frame_rate.clamp(FRAME_RATE_MIN, FRAME_RATE_MAX));
    let step = 1.0 / rate;
    let cycle = f64::from(total_frames) * step;
    let time = f64::from(t);
    let mut remainder = time - (time / cycle).trunc() * cycle;
    let mut frame = 0;
    while remainder >= step {
        frame += 1;
        remainder -= step;
    }
    frame % total_frames
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::{FRAC_PI_2, TAU};

    fn close(a: [f32; 2], b: [f32; 2]) -> bool {
        (a[0] - b[0]).abs() < 1e-4 && (a[1] - b[1]).abs() < 1e-4
    }

    #[test]
    fn a_panner_moves_by_direction_times_rate_times_time() {
        let m = panner([0, 0, 0], 0.5, 2.0);
        assert!(close(m.apply([0.0, 0.0]), [1.0, 0.0]), "{m:?}");
        assert!(close(m.apply([0.25, 0.75]), [1.25, 0.75]));
        let m = panner([0, 16384, 0], 0.5, 2.0);
        assert!(close(m.apply([0.0, 0.0]), [0.0, 1.0]), "{m:?}");
        let m = panner([0, 0, 0], 3.0, 400.0);
        assert!(close(m.apply([0.0, 0.0]), [1200.0 - 1024.0, 0.0]), "{m:?}");
        let m = panner([0, 0, 0], -0.25, 1.0);
        assert!(close(m.apply([0.0, 0.0]), [1023.75, 0.0]), "{m:?}");
    }

    fn oscillator(u_type: u8, v_type: u8, offset: [f32; 2]) -> Oscillator {
        Oscillator { rate: [0.25, 0.0], phase: [0.0, 0.0], amplitude: [0.1, 0.0], kind: [u_type, v_type], offset }
    }

    #[test]
    fn an_oscillating_pan_is_a_sine_of_the_fractional_turn() {
        let m = oscillator(OT_PAN, OT_PAN, [0.0, 0.0]).matrix([64.0, 64.0], 1.0);
        assert!(close(m.apply([0.0, 0.0]), [(TAU * 0.25).sin() * 0.1, 0.0]), "{m:?}");
        let m = oscillator(OT_PAN, OT_PAN, [0.0, 0.0]).matrix([64.0, 64.0], 3.0);
        assert!(close(m.apply([0.5, 0.5]), [0.5 + (TAU * 0.75).sin() * 0.1, 0.5]), "{m:?}");
    }

    #[test]
    fn an_oscillating_stretch_scales_around_the_offset() {
        let m = oscillator(OT_STRETCH, OT_PAN, [32.0, 0.0]).matrix([64.0, 64.0], 1.0);
        let scale = (TAU * 0.25).sin() * 0.1 + 1.0;
        assert!(close(m.apply([0.5, 0.25]), [0.5, 0.25]), "{m:?}");
        assert!(close(m.apply([1.0, 0.25]), [0.5 + 0.5 * scale, 0.25]), "{m:?}");
        let m = oscillator(OT_STRETCH_REPEAT, OT_PAN, [0.0, 0.0]).matrix([64.0, 64.0], 0.5);
        let scale = (0.125 * FRAC_PI_2).sin() * 0.1 + 1.0;
        assert!(close(m.apply([1.0, 0.0]), [scale, 0.0]), "{m:?}");
    }

    #[test]
    fn a_jitter_holds_one_random_offset_per_rate_period() {
        let osc = Oscillator {
            rate: [2.0, 0.0],
            phase: [0.0, 0.0],
            amplitude: [0.5, 0.0],
            kind: [OT_JITTER, OT_PAN],
            offset: [0.0; 2],
        };
        let a = osc.matrix([64.0, 64.0], 0.6).apply([0.0, 0.0]);
        let b = osc.matrix([64.0, 64.0], 0.9).apply([0.0, 0.0]);
        let c = osc.matrix([64.0, 64.0], 1.1).apply([0.0, 0.0]);
        assert_eq!(a, b);
        assert!((0.0..=0.5).contains(&a[0]) && (0.0..=0.5).contains(&c[0]));
    }

    fn rotator(kind: u8, rotation: [i32; 3], offset: [f32; 2]) -> Rotator {
        Rotator {
            kind,
            rotation,
            offset,
            oscillation_rate: [0; 3],
            oscillation_amplitude: [0; 3],
            oscillation_phase: [0; 3],
        }
    }

    #[test]
    fn a_constant_rotator_turns_rotation_per_second_counterclockwise() {
        let m = rotator(TR_CONSTANTLY, [0, 16384, 0], [0.0, 0.0]).matrix([64.0, 64.0], 1.0);
        assert!(close(m.apply([1.0, 0.0]), [0.0, 1.0]), "{m:?}");
        let m = rotator(TR_CONSTANTLY, [0, 16384, 0], [0.0, 0.0]).matrix([64.0, 64.0], 4.0);
        assert!(close(m.apply([1.0, 0.0]), [1.0, 0.0]), "{m:?}");
        let m = rotator(TR_FIXED, [0, 32768, 0], [32.0, 32.0]).matrix([64.0, 64.0], 7.0);
        assert!(close(m.apply([1.0, 0.5]), [0.0, 0.5]), "{m:?}");
        let m = rotator(TR_OSCILLATING, [0, 0, 0], [0.0, 0.0]).matrix([64.0, 64.0], 1.0);
        assert!(close(m.apply([1.0, 0.0]), [1.0, 0.0]), "{m:?}");
    }

    #[test]
    fn the_rotator_keeps_the_third_column_of_frotationmatrix_104f51d0() {
        let m = rotator(TR_FIXED, [-16384, 0, 0], [0.0, 0.0]).matrix([0.0, 0.0], 3.0);
        assert_eq!(m.apply3([1.0, 2.0, 3.0]), [3.0, 2.0, -1.0], "{m:?}");
        let yaw = rotator(TR_FIXED, [0, 16384, 0], [0.0, 0.0]).matrix([0.0, 0.0], 0.0);
        assert_eq!(yaw.apply3([1.0, 0.0, 5.0]), [0.0, 1.0, 5.0], "{yaw:?}");
        let roll = rotator(TR_FIXED, [0, 0, 16384], [0.0, 0.0]).matrix([0.0, 0.0], 0.0);
        assert_eq!(roll.apply3([0.0, 1.0, 0.0]), [0.0, 0.0, -1.0], "{roll:?}");
    }

    #[test]
    fn fmatrix_products_use_every_column() {
        let pitch = rotator(TR_FIXED, [16384, 0, 0], [0.0, 0.0]).matrix([0.0, 0.0], 0.0);
        let shift = Mat3x2::translation(0.5, 0.25);
        let product = pitch.then(shift);
        assert_eq!(product.apply3([1.0, 0.0, 0.0]), [0.5, 0.25, 1.0], "{product:?}");
        assert_eq!(product.apply([1.0, 0.0]), [-0.5, 0.25]);
    }

    #[test]
    fn a_scaler_divides_and_offsets_in_texels() {
        let m = scaler([2.0, 4.0], [32.0, 16.0], [64.0, 64.0]);
        assert!(close(m.apply([1.0, 1.0]), [1.0, 0.5]), "{m:?}");
    }

    #[test]
    fn matrices_compose_outer_first() {
        let outer = Mat3x2::translation(1.0, 0.0);
        let inner = scaler([2.0, 2.0], [0.0, 0.0], [64.0, 64.0]);
        assert!(close(outer.then(inner).apply([1.0, 1.0]), [1.0, 0.5]));
        assert!(close(inner.then(outer).apply([1.0, 1.0]), [1.5, 0.5]));
    }

    #[test]
    fn frames_follow_the_total_frame_count_and_the_clamped_rate() {
        assert_eq!(frame_index(0.55, 20.0, 10), 1);
        assert_eq!(frame_index(0.5, 0.0, 10), 0);
        assert_eq!(frame_index(0.5, 20.0, 0), 0);
        assert_eq!(frame_index(0.49, 20.0, 10), 9);
        assert_eq!(frame_index(0.5, 20.0, 10), 0);
        assert_eq!(frame_index(3.0, 1000.0, 4), 0);
        assert_eq!(frame_index(0.0125, 1000.0, 4), 1);
        assert_eq!(frame_index(30.0, 0.01, 3), 0);
        assert_eq!(frame_index(10.0, 0.01, 3), 1);
    }
}
