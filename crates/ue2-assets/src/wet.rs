#[derive(Clone, Debug)]
pub struct FireRandom {
    words: [u32; 64],
    index: usize,
}

impl FireRandom {
    pub fn from_seed(mut seed: u32) -> Self {
        let mut bytes = [0u8; 256];
        for byte in &mut bytes {
            seed = seed.wrapping_mul(214013).wrapping_add(2531011);
            *byte = (seed >> 16) as u8;
        }
        Self {
            words: std::array::from_fn(|i| {
                u32::from_le_bytes([bytes[i * 4], bytes[i * 4 + 1], bytes[i * 4 + 2], bytes[i * 4 + 3]])
            }),
            index: 0,
        }
    }

    fn next(&mut self) -> u8 {
        let value = self.words[(self.index + 32) & 63];
        self.index = (self.index + 1) & 63;
        self.words[self.index] ^= value;
        value as u8
    }
}

pub fn fire_sine() -> [u8; 256] {
    std::array::from_fn(|i| {
        (((i as f64 / 256.0 * f64::from(std::f32::consts::TAU)).sin() * 127.5 + 127.44999694824219) as f32)
            .round_ties_even() as u8
    })
}

#[derive(Clone, Debug)]
pub struct WaterSimulation {
    pub width: usize,
    pub height: usize,
    pub fields: Vec<u8>,
    pub pixels: Vec<u8>,
    pub drops: Vec<[u8; 8]>,
    pub parity: u8,
    pub global_phase: u32,
    pub refraction: [u8; 1024],
    sine: [u8; 256],
}

impl WaterSimulation {
    pub fn new(width: usize, height: usize, amplitude: u8, drops: Vec<[u8; 8]>) -> Option<Self> {
        if width < 8
            || height < 8
            || !width.is_power_of_two()
            || !height.is_power_of_two()
            || drops.len() > 256
        {
            return None;
        }
        if drops.iter().any(|d| usize::from(d[2]) >= width / 2 || usize::from(d[3]) >= height / 2) {
            return None;
        }
        let count=width.checked_mul(height)?;
        let mut fields=Vec::new();
        fields.try_reserve_exact(count/2).ok()?;
        fields.resize(count/2,128);
        let mut pixels=Vec::new();
        pixels.try_reserve_exact(count).ok()?;
        pixels.resize(count,0);
        Some(Self {
            width,
            height,
            fields,
            pixels,
            drops,
            parity: 0,
            global_phase: 0,
            refraction: std::array::from_fn(|i| {
                (((i as i32 - 511) * i32::from(amplitude) / 512).clamp(-128, 127)) as u8
            }),
            sine: fire_sine(),
        })
    }

    pub fn redraw_drops(&mut self, random: &mut FireRandom) {
        let w = self.width;
        let mx = (w / 2 - 1)&255;
        let my = (self.height / 2 - 1)&255;
        let fields = &mut self.fields;
        let sine = &self.sine;
        let mut write = |x: usize, y: usize, first: u8, second: u8| {
            let i = (y & my) * w + (x & mx);
            fields[i] = first;
            fields[i + w / 2] = second;
        };
        self.global_phase = self.global_phase.wrapping_add(1);
        for d in &mut self.drops {
            let x = usize::from(d[2]);
            let y = usize::from(d[3]);
            match d[0] {
                0 => write(x, y, d[7], d[7]),
                1..=3 => {
                    d[1] = d[1].wrapping_add(d[7]);
                    let value = sine[usize::from(d[1])];
                    let value = match d[0] {
                        2 => (value >> 1) + 64,
                        3 => value.max(128),
                        _ => value,
                    };
                    write(x, y, value, value);
                }
                4 => {
                    d[2] = d[2].wrapping_add(random.next() & 3).wrapping_sub(random.next() & 3) & mx as u8;
                    d[3] = d[3].wrapping_add(random.next() & 3).wrapping_sub(random.next() & 3) & my as u8;
                    write(x, y, 185, 71);
                }
                5 => write(x, y, random.next(), random.next()),
                6 | 7 | 64 | 65 => {
                    let angle = u16::from_le_bytes([d[4], d[5]]);
                    let step = u16::from_le_bytes([d[6], d[7]]);
                    let angle = if d[0] < 64 { angle.wrapping_add(step) } else { angle.wrapping_sub(step) };
                    [d[4], d[5]] = angle.to_le_bytes();
                    let shift = if d[0] == 6 || d[0] == 64 { 4 } else { 3 };
                    write(
                        x + usize::from(sine[usize::from(d[5])] >> shift),
                        y + usize::from(sine[usize::from(d[5].wrapping_add(64))] >> shift),
                        d[1],
                        d[1],
                    );
                }
                8..=15 => {
                    let value = if d[0] >= 12 {
                        d[1] = d[1].wrapping_add(d[6]);
                        sine[usize::from(d[1])]
                    } else {
                        d[1]
                    };
                    for j in 0..=usize::from(d[7] >> 1) {
                        let (dx, dy) = match d[0] & 3 {
                            0 => (x + j, y),
                            1 => (x, y + j),
                            2 => (x.wrapping_sub(j), y + j),
                            _ => (x + j, y + j),
                        };
                        write(dx, dy, value, value);
                    }
                }
                16 if random.next() & 15 == 0 => {
                    let dy = (usize::from(random.next()) * usize::from(d[7])) >> 8;
                    let dx = (usize::from(random.next()) * usize::from(d[7])) >> 8;
                    write(x + dx, y + dy, d[1], !d[1]);
                }
                17 => {
                    for dy in 0..usize::from(d[7] >> 1) {
                        for dx in 0..usize::from(d[7] >> 1) {
                            write(x + dx, y + dy, d[1], d[1]);
                        }
                    }
                }
                18 | 19 => {
                    d[4] = d[4].wrapping_add(d[7]);
                    if d[4] <= d[7] {
                        if d[0] == 19 {
                            d[4] = random.next();
                        }
                        write(x, y, d[1], !d[1]);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn calculate_water(&mut self) {
        self.parity = self.parity.wrapping_add(1);
        let odd = self.parity & 1 != 0;
        let w = self.width / 2;
        let h = self.height / 2;
        let phase = if odd { 0 } else { w };
        for y in 0..h {
            for x in 0..w {
                let sample =
                    |x: usize, y: usize| i32::from(self.fields[(y & (h - 1)) * self.width + (x & (w - 1)) + phase]);
                let neighbor = if odd { (x + 1, y + 1) } else { (x.wrapping_sub(1), y.wrapping_sub(1)) };
                let old_index = y * self.width + x + (w - phase);
                let delta =
                    sample(x, y) + sample(neighbor.0, y) + sample(x, neighbor.1) + sample(neighbor.0, neighbor.1)
                        - 2 * i32::from(self.fields[old_index]);
                let (a, b, c, d, px, py) = if odd {
                    let grad = |x: usize, y: usize| sample(x.wrapping_add(1), y) - sample(x.wrapping_sub(1), y);
                    (
                        grad(x, y + 1),
                        grad(x, y),
                        grad(x.wrapping_sub(1), y + 1),
                        grad(x.wrapping_sub(1), y),
                        2 * x + 1,
                        2 * y + 1,
                    )
                } else {
                    let grad = |x: usize, y: usize| sample(x, y) - sample(x.wrapping_sub(2), y);
                    (
                        grad(x, y),
                        grad(x, y.wrapping_sub(1)),
                        grad(x.wrapping_sub(1), y),
                        grad(x.wrapping_sub(1), y.wrapping_sub(1)),
                        2 * x,
                        2 * y,
                    )
                };
                self.fields[old_index] = ((512 + delta) / 2 - 256 + i32::from(delta < 256)).clamp(0, 255) as u8;
                for (dx, dy, g) in [(0, 0, 2 * a), (0, 1, a + b), (1, 0, a + c), (1, 1, (a + b + c + d) / 2)] {
                    let i = (py.wrapping_sub(dy) & (self.height - 1)) * self.width
                        + (px.wrapping_sub(dx) & (self.width - 1));
                    self.pixels[i] = self.refraction[(512 + g) as usize];
                }
            }
        }
    }

    pub fn apply_wet_texture(&mut self, source: &[u8]) -> bool {
        if source.len() < self.pixels.len() {
            return false;
        }
        for (i, pixel) in self.pixels.iter_mut().enumerate() {
            *pixel = source[(i / self.width) * self.width + ((i + usize::from(*pixel)) & (self.width - 1))];
        }
        true
    }

    pub fn tick(&mut self, source: &[u8], random: &mut FireRandom) -> bool {
        if source.len() < self.pixels.len() {
            return false;
        }
        self.redraw_drops(random);
        self.calculate_water();
        self.apply_wet_texture(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_field_stays_at_rest_through_parity_wrap() {
        let mut water = WaterSimulation::new(16, 32, 128, vec![]).unwrap();
        let mut random = FireRandom::from_seed(1);
        let source: Vec<u8> = (0..512).map(|i| i as u8).collect();
        for _ in 0..512 {
            assert!(water.tick(&source, &mut random));
            assert!(water.fields.iter().all(|&v| v == 128));
            assert_eq!(water.pixels, source);
        }
        assert_eq!(water.parity, 0);
        assert_eq!(water.global_phase, 512);
    }

    #[test]
    fn reject_invalid_layouts_and_truncated_sources_without_mutating_state() {
        for (w, h) in [(0, 8), (4, 8), (9, 16), (8, 1023)] {
            assert!(WaterSimulation::new(w, h, 128, vec![]).is_none());
        }
        assert!(WaterSimulation::new(8, 8, 128, vec![[0, 0, 4, 0, 0, 0, 0, 0]]).is_none());
        let mut water = WaterSimulation::new(8, 8, 128, vec![]).unwrap();
        assert!(!water.tick(&[0; 63], &mut FireRandom::from_seed(1)));
        assert_eq!(water.global_phase, 0);
    }

    #[test]
    fn refraction_has_native_offset_and_signed_clamping() {
        let water = WaterSimulation::new(8, 8, 255, vec![]).unwrap();
        assert_eq!(water.refraction[0], 128);
        assert_eq!(water.refraction[1023], 127);
        assert_eq!(water.refraction[511], 0);
        assert_eq!(water.refraction[515], 1);
    }
}
