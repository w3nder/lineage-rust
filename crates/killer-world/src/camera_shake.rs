pub const STATE_WORDS: usize = 0x9c / 4;

const SLOW_FRAME_RATE: f64 = 30.0;
const SLOW_FRAME_SCALE: f32 = f32::from_bits(0x3d088889);
const FREQUENCY_PER_INTENSITY: f64 = 50.0;
const EXTRA_SECONDS: f64 = 5.0;
const COUNT_MARGIN: f64 = 2.0;

const EPSILON: f32 = 0.0001;
const SAFE_NORMAL_LIMIT: f64 = 1e-8;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewShake {
    words: [u32; STATE_WORDS],
}

fn truncate(value: f64) -> i32 {
    value as i32
}

impl ViewShake {
    pub fn from_words(words: [u32; STATE_WORDS]) -> Self {
        Self { words }
    }

    pub fn words(&self) -> &[u32; STATE_WORDS] {
        &self.words
    }

    pub fn kind(&self) -> u8 {
        self.words[2] as u8
    }

    fn get(&self, offset: usize) -> f32 {
        f32::from_bits(self.words[offset / 4])
    }

    fn set(&mut self, offset: usize, value: f32) {
        self.words[offset / 4] = value.to_bits();
    }

    fn vector_is_zero(&self, offset: usize) -> bool {
        (0..3).all(|axis| self.get(offset + axis * 4) == 0.0)
    }

    pub fn rotation_offset(&self) -> [i32; 3] {
        [self.words[0x78 / 4] as i32, self.words[0x7c / 4] as i32, self.words[0x80 / 4] as i32]
    }

    pub fn location_offset(&self) -> [f32; 3] {
        [self.get(0x6c), self.get(0x70), self.get(0x74)]
    }

    fn check_shake(&mut self, phase: &mut f32) -> bool {
        let kind = self.kind();
        let (amplitude, frequency, remaining) = (self.get(0x3c), self.get(0x4c), self.get(0x5c));
        if amplitude <= 0.0 {
            if amplitude < *phase {
                return false;
            }
        } else if *phase < amplitude {
            return false;
        }
        *phase = amplitude;
        if remaining <= 1.0 {
            self.set(0x3c, 0.0);
            *phase = 0.0;
            self.set(0x4c, 0.0);
            return false;
        }
        let next = if amplitude <= 0.0 {
            Some((-f64::from(amplitude)) as f32)
        } else {
            match kind {
                0 => Some((((1.0 / (f64::from(self.words[0x2c / 4] as i32) - f64::from(remaining))) - 1.0) * f64::from(amplitude)) as f32),
                4 => Some((((1.0 / f64::from(remaining)) - 1.0) * f64::from(amplitude)) as f32),
                3 | 5 => Some((f64::from(amplitude) * f64::from(-1.1_f32)) as f32),
                _ => None,
            }
        };
        if let Some(value) = next {
            self.set(0x3c, value);
        }
        self.set(0x5c, remaining - 1.0);
        self.set(0x4c, -frequency);
        true
    }

    fn safe_normal(&self, offset: usize) -> [f32; 3] {
        let [x, y, z] = [0, 4, 8].map(|axis| f64::from(self.get(offset + axis)));
        let squared = x * x + y * y + z * z;
        if squared < SAFE_NORMAL_LIMIT {
            return [0.0; 3];
        }
        let scale = f64::from((1.0 / squared.sqrt()) as f32);
        [(x * scale) as f32, (y * scale) as f32, (z * scale) as f32]
    }

    pub fn update(&mut self, dt: f32) -> bool {
        let time = self.get(0x0c);
        if dt == 0.0 || time == 0.0 {
            return false;
        }
        let time = (f64::from(time) - f64::from(dt)) as f32;
        self.set(0x0c, time);
        if time <= EPSILON {
            return false;
        }
        if (self.vector_is_zero(0x50) && self.get(0x4c) == 0.0)
            || (self.vector_is_zero(0x60) && self.get(0x5c) == 0.0)
            || (self.vector_is_zero(0x40) && self.get(0x3c) == 0.0)
        {
            return false;
        }
        let mut pitch = 0_i32;
        let mut yaw = 0_i32;
        if self.get(0x4c) != 0.0 {
            let previous = truncate(f64::from(self.get(0x88))) & 0xffff;
            let advance = truncate(f64::from(dt) * f64::from(self.get(0x4c)));
            let wrapped = (advance.wrapping_add(previous) & 0xffff) as f32;
            self.set(0x88, wrapped);
            let mut phase = if wrapped > 32768.0 { wrapped - 65536.0 } else { wrapped };
            if phase != wrapped {
                self.set(0x88, phase);
            }
            match self.kind() {
                0 | 3 | 4 => {
                    self.check_shake(&mut phase);
                }
                5 => {
                    if self.get(0x3c) > self.get(0x98) {
                        let peak = self.get(0x98);
                        self.words[2] = 4;
                        self.set(0x3c, peak);
                        self.words[0x2c / 4] = truncate(f64::from(self.get(0x5c)) + 2.0) as u32;
                    }
                    self.check_shake(&mut phase);
                }
                _ => {}
            }
            self.set(0x88, phase);
            let direction = self.safe_normal(0x30);
            if self.get(0x30) != 0.0 {
                yaw = truncate(f64::from(direction[0]).abs() * f64::from(self.get(0x88)));
            }
            if self.get(0x34) != 0.0 {
                pitch = truncate(f64::from(direction[1]).abs() * f64::from(self.get(0x88)));
            }
        }
        for axis in 0..3 {
            self.words[0x6c / 4 + axis] = 0;
        }
        self.words[0x78 / 4] = pitch as u32;
        self.words[0x7c / 4] = yaw as u32;
        self.words[0x80 / 4] = 0;
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AddShake {
    pub kind: u8,
    pub time: f32,
    pub scale: f32,
    pub frequency: f32,
    pub count: f32,
    pub direction: [f32; 3],
    pub second: [f32; 3],
    pub third: [f32; 3],
    pub origin: [f32; 3],
    pub intensity: f32,
    pub range: f32,
}

fn single(value: f64) -> f32 {
    value as f32
}

impl ViewShake {
    pub fn added(add: &AddShake, viewer: [f32; 3], frame_time: f32) -> Self {
        let mut shake = Self { words: [0; STATE_WORDS] };
        shake.words[2] = u32::from(add.kind);
        shake.set(0x0c, add.time);
        let falloff = if add.origin == [0.0; 3] || add.range == 0.0 {
            f64::from(add.intensity)
        } else {
            let distance = viewer.iter().zip(add.origin).map(|(a, b)| (f64::from(*a) - f64::from(b)).powi(2)).sum::<f64>().sqrt();
            let ratio = distance / f64::from(add.range);
            2.0 * f64::from(add.intensity) / ((-ratio).exp() + ratio.exp())
        };
        let frames_per_second = 1.0 / f64::from(frame_time);
        let (mut third, mut count) = (add.third, add.count);
        if frames_per_second < SLOW_FRAME_RATE {
            let scale = single(frames_per_second * f64::from(SLOW_FRAME_SCALE));
            third = third.map(|value| single(f64::from(value) * f64::from(scale)));
            count = single(frames_per_second * f64::from(count) * f64::from(SLOW_FRAME_SCALE));
        }
        let scaled = add.direction.map(|value| single(f64::from(value) * falloff));
        shake.set_vector(0x60, third);
        shake.set_vector(0x50, add.second);
        shake.set(0x3c, single(falloff * f64::from(add.scale)));
        shake.set_vector(0x40, scaled);
        shake.set(0x5c, count);
        shake.set(0x4c, add.frequency);
        shake.set_vector(0x10, add.origin);
        shake.set(0x1c, single(falloff * f64::from(add.scale)));
        shake.set_vector(0x30, add.direction);
        shake.set_vector(0x20, scaled);
        let largest = third.into_iter().fold(f32::MIN, f32::max);
        let largest = if largest < count { count } else { largest };
        shake.words[0x2c / 4] = truncate(f64::from(largest) + COUNT_MARGIN) as u32;
        if add.kind == 5 {
            shake.words[0x98 / 4] = shake.words[0x3c / 4];
            shake.set(0x3c, 1.0);
        }
        shake
    }

    fn set_vector(&mut self, offset: usize, value: [f32; 3]) {
        for (axis, component) in value.into_iter().enumerate() {
            self.set(offset + axis * 4, component);
        }
    }
}

pub const EARTHQUAKE_SOUND: &str = "MonSound3.antaras_earthquake";
pub const EARTHQUAKE_SOUND_LONG: &str = "MonSound3.antaras_earthquake_2";
pub const EARTHQUAKE_LONG_SECONDS: i32 = 7;
pub const EARTHQUAKE_SOUND_VOLUME: f32 = 255.0;
pub const EARTHQUAKE_SOUND_RADIUS: f32 = 500.0;
const EARTHQUAKE_KIND: u8 = 5;
const EARTHQUAKE_FREQUENCY_PER_INTENSITY: f32 = 60.0;
const EARTHQUAKE_FRAMES_PER_SECOND: i32 = 30;
const EARTHQUAKE_EXTRA_SECONDS: i32 = 2;
const EARTHQUAKE_DIRECTION: [f32; 3] = [0.0, f32::from_bits(0x3dcccccd), 0.0];
const EARTHQUAKE_SECOND: [f32; 3] = [0.0, 100.0, 0.0];

pub fn earthquake(origin: [i32; 3], intensity: i32, time: i32) -> (AddShake, &'static str) {
    let count = time.wrapping_mul(EARTHQUAKE_FRAMES_PER_SECOND) as f32;
    let shake = AddShake {
        kind: EARTHQUAKE_KIND,
        time: time.wrapping_add(EARTHQUAKE_EXTRA_SECONDS) as f32,
        scale: 1.0,
        frequency: intensity as f32 * EARTHQUAKE_FREQUENCY_PER_INTENSITY,
        count,
        direction: EARTHQUAKE_DIRECTION,
        second: EARTHQUAKE_SECOND,
        third: [count, count, 0.0],
        origin: origin.map(|value| value as f32),
        intensity: intensity as f32,
        range: 0.0,
    };
    (shake, if time > EARTHQUAKE_LONG_SECONDS { EARTHQUAKE_SOUND_LONG } else { EARTHQUAKE_SOUND })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShakeNotify {
    pub kind: u8,
    pub intensity: f32,
    pub direction: [f32; 3],
    pub range: f32,
    pub count: i32,
}

#[derive(Clone, Debug, Default)]
pub struct ShakeSet {
    states: Vec<ViewShake>,
    frame_time: f32,
}

impl ShakeSet {
    pub fn states(&self) -> &[ViewShake] {
        &self.states
    }

    pub fn frame_time(&self) -> f32 {
        self.frame_time
    }

    pub fn measure(&mut self, dt: f32) {
        self.frame_time = if self.frame_time == 0.0 {
            dt
        } else {
            single(f64::from(dt) * f64::from(0.002_f32) + f64::from(self.frame_time) * f64::from(0.998_f32))
        };
    }

    pub fn add(&mut self, add: &AddShake, viewer: [f32; 3]) {
        self.states.insert(0, ViewShake::added(add, viewer, self.frame_time));
    }

    pub fn add_notify(&mut self, notify: &ShakeNotify, pawn: [f32; 3], viewer: [f32; 3], randoms: (f32, f32)) {
        let direction = if notify.direction == [0.0; 3] { [randoms.1, randoms.0, 0.0] } else { notify.direction };
        let [x, y] = [f64::from(direction[0]), f64::from(direction[1])];
        let inverse = 1.0 / (y * y + x * x).sqrt();
        let normal = [single(x * inverse), single(y * inverse), 0.0];
        let count = notify.count as f32;
        let first = normal.map(|component| single(f64::from(notify.intensity) * f64::from(component)));
        let second = first.map(|component| single(f64::from(component) + f64::from(component)));
        self.add(
            &AddShake {
                kind: notify.kind,
                time: single(f64::from(count) * f64::from(self.frame_time) + EXTRA_SECONDS),
                scale: 1.0,
                frequency: single(f64::from(notify.intensity) * FREQUENCY_PER_INTENSITY),
                count,
                direction: first,
                second,
                third: [count, count, 0.0],
                origin: pawn,
                intensity: notify.intensity,
                range: notify.range,
            },
            viewer,
        );
    }

    pub fn update(&mut self, dt: f32) {
        let mut index = self.states.len();
        while index > 0 {
            index -= 1;
            if !self.states[index].update(dt) {
                self.states.remove(index);
            }
        }
    }

    pub fn rotation_offset(&self) -> [i32; 3] {
        let mut sum = [0_i32; 3];
        for state in self.states.iter().rev() {
            let offset = state.rotation_offset();
            for axis in 0..3 {
                sum[axis] = sum[axis].wrapping_add(offset[axis]);
            }
        }
        sum.map(|component| component & 0xffff)
    }

    pub fn location_offset(&self, view_distance: f32) -> [f32; 3] {
        let mut sum = [0.0_f32; 3];
        for state in self.states.iter().rev() {
            let offset = state.location_offset();
            for axis in 0..3 {
                sum[axis] += offset[axis];
            }
        }
        let scale = (view_distance + 250.0) * 0.01;
        sum.map(|component| component * scale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_104ef7d0_equals_the_original_on_random_states_and_steps() {
        let cases: serde_json::Value = serde_json::from_str(include_str!("../tests/data/view_shake_oracle.json")).expect("fixture");
        let words = |value: &serde_json::Value| -> [u32; STATE_WORDS] {
            let list: Vec<u32> = value.as_array().expect("lista").iter().map(|word| word.as_u64().expect("inteiro") as u32).collect();
            list.try_into().expect("tamanho")
        };
        let mut checked = 0;
        for (index, case) in cases.as_array().expect("casos").iter().enumerate() {
            let mut shake = ViewShake::from_words(words(&case["initial"]));
            for (step, expected) in case["steps"].as_array().expect("passos").iter().enumerate() {
                let dt = f32::from_bits(expected["dt"].as_u64().expect("dt") as u32);
                let alive = shake.update(dt);
                assert_eq!(u64::from(alive), expected["result"].as_u64().expect("resultado"), "caso {index} passo {step}");
                let want = words(&expected["state"]);
                for (slot, (got, want)) in shake.words().iter().zip(want).enumerate() {
                    assert_eq!(*got, want, "caso {index} passo {step} palavra {slot:#x}: {} != {}", f32::from_bits(*got), f32::from_bits(want));
                }
                checked += 1;
            }
        }
        assert!(checked > 2000);
    }

    fn notify(kind: u8) -> ShakeNotify {
        ShakeNotify { kind, intensity: 4.0, direction: [3.0, 4.0, 9.0], range: 0.0, count: 10 }
    }

    #[test]
    fn add_view_shake_state_10539d50_builds_the_state_from_the_notify_arguments() {
        let mut set = ShakeSet::default();
        set.measure(0.02);
        set.add_notify(&notify(0), [0.0; 3], [0.0; 3], (0.1, 0.2));
        let state = &set.states()[0];
        assert_eq!(state.kind(), 0);
        assert_eq!(f32::from_bits(state.words()[0x0c / 4]), (10.0_f64 * 0.02_f32 as f64 + 5.0) as f32);
        assert_eq!(f32::from_bits(state.words()[0x3c / 4]), 4.0);
        assert_eq!(f32::from_bits(state.words()[0x4c / 4]), 200.0);
        assert_eq!(f32::from_bits(state.words()[0x5c / 4]), 10.0);
        assert_eq!(f32::from_bits(state.words()[0x40 / 4]), (f64::from((4.0_f64 * f64::from(0.6_f32)) as f32) * 4.0) as f32);
        assert_eq!(f32::from_bits(state.words()[0x50 / 4]), 2.0 * f32::from_bits(state.words()[0x30 / 4]));
        assert_eq!(f32::from_bits(state.words()[0x60 / 4]), 10.0);
        assert_eq!(f32::from_bits(state.words()[0x64 / 4]), 10.0);
        assert_eq!(f32::from_bits(state.words()[0x68 / 4]), 0.0);
        assert_eq!(state.words()[0x2c / 4], 12);
    }

    #[test]
    fn add_view_shake_state_10539d50_scales_by_the_cosh_falloff_with_distance_and_range() {
        let mut set = ShakeSet::default();
        set.measure(0.02);
        let mut shaken = notify(0);
        shaken.range = 100.0;
        set.add_notify(&shaken, [100.0, 0.0, 0.0], [0.0; 3], (0.0, 0.0));
        let expected = (4.0_f64 / 1.0_f64.cosh()) as f32;
        let got = f32::from_bits(set.states()[0].words()[0x3c / 4]);
        assert!((got - expected).abs() < 1e-5, "{got} != {expected}");
    }

    #[test]
    fn add_view_shake_state_10539d50_compensates_slow_frame_rates_below_thirty() {
        let mut set = ShakeSet::default();
        set.measure(0.1);
        set.add_notify(&notify(0), [0.0; 3], [0.0; 3], (0.0, 0.0));
        let state = &set.states()[0];
        let scale = 10.0_f64 * f64::from(SLOW_FRAME_SCALE);
        assert_eq!(f32::from_bits(state.words()[0x60 / 4]), (10.0 * f64::from(scale as f32)) as f32);
        assert_eq!(f32::from_bits(state.words()[0x5c / 4]), (10.0 * 10.0 * f64::from(SLOW_FRAME_SCALE)) as f32);
    }

    #[test]
    fn add_view_shake_state_10539d50_keeps_the_peak_of_type_five_and_inserts_the_newest_first() {
        let mut set = ShakeSet::default();
        set.measure(0.02);
        set.add_notify(&notify(0), [0.0; 3], [0.0; 3], (0.0, 0.0));
        set.add_notify(&notify(5), [0.0; 3], [0.0; 3], (0.0, 0.0));
        assert_eq!(set.states().iter().map(ViewShake::kind).collect::<Vec<_>>(), [5, 0]);
        assert_eq!(f32::from_bits(set.states()[0].words()[0x98 / 4]), 4.0);
        assert_eq!(f32::from_bits(set.states()[0].words()[0x3c / 4]), 1.0);
    }

    #[test]
    fn a_random_direction_is_used_only_when_the_notify_vector_is_zero() {
        let mut set = ShakeSet::default();
        set.measure(0.02);
        let mut random = notify(0);
        random.direction = [0.0; 3];
        set.add_notify(&random, [0.0; 3], [0.0; 3], (0.3, 0.4));
        let direction = &set.states()[0].words()[0x30 / 4..0x3c / 4];
        assert!((f32::from_bits(direction[0]) - 3.2).abs() < 1e-5);
        assert!((f32::from_bits(direction[1]) - 2.4).abs() < 1e-5);
        assert_eq!(f32::from_bits(direction[2]), 0.0);
    }

    #[test]
    fn the_shake_decays_and_the_set_drops_finished_states() {
        let mut set = ShakeSet::default();
        set.measure(0.02);
        set.add_notify(&notify(0), [0.0; 3], [0.0; 3], (0.0, 0.0));
        let mut peak = 0;
        for _ in 0..2000 {
            set.update(0.02);
            peak = peak.max(set.rotation_offset()[1].min(65536 - set.rotation_offset()[1]));
            if set.states().is_empty() {
                break;
            }
        }
        assert!(peak > 0);
        assert!(set.states().is_empty());
        assert_eq!(set.rotation_offset(), [0, 0, 0]);
    }

    #[test]
    fn the_earthquake_builds_the_engine_view_shake_and_picks_the_sound_by_duration() {
        let (shake, sound) = earthquake([10, -20, 30], 12, 5);
        assert_eq!((shake.kind, shake.time, shake.scale), (5, 7.0, 1.0));
        assert_eq!((shake.frequency, shake.intensity, shake.range), (720.0, 12.0, 0.0));
        assert_eq!((shake.count, shake.third), (150.0, [150.0, 150.0, 0.0]));
        assert_eq!((shake.direction, shake.second, shake.origin), ([0.0, 0.1, 0.0], [0.0, 100.0, 0.0], [10.0, -20.0, 30.0]));
        assert_eq!(sound, EARTHQUAKE_SOUND);
        assert_eq!(earthquake([0; 3], 1, 7).1, EARTHQUAKE_SOUND);
        assert_eq!(earthquake([0; 3], 1, 8).1, EARTHQUAKE_SOUND_LONG);
        let view = ViewShake::added(&shake, [0.0; 3], 1.0 / 60.0);
        assert_eq!(view.kind(), 5);
    }
}
