use crate::{Catalog, RawMip, RawTexture, full_path, package_name};
use ue2_assets::{
    Property, TextureFormat, find, object_properties,
    wet::{FireRandom, WaterSimulation},
};

#[derive(Clone, Debug)]
pub struct WetTexture {
    pub simulation: WaterSimulation,
    pub source: Vec<u8>,
    pub palette: Vec<[u8; 4]>,
    pub prime_count: u8,
    pub max_frame_rate: f32,
    pub total_frames: i32,
}

impl WetTexture {
    pub fn raw(&self) -> RawTexture {
        RawTexture {
            format: TextureFormat::P8,
            palette: Some(self.palette.clone()),
            mips: vec![RawMip {
                width: self.simulation.width as u32,
                height: self.simulation.height as u32,
                data: self.source.clone(),
            }],
        }
    }

    pub fn evaluate(&self, state: &mut WaterSimulation, prime_current: &mut u8, time: f64, random: &mut FireRandom) {
        self.evaluate_with_animation(state, prime_current, time, random, false);
    }

    pub fn evaluate_with_animation(
        &self,
        state: &mut WaterSimulation,
        prime_current: &mut u8,
        time: f64,
        random: &mut FireRandom,
        no_fractal_anim: bool,
    ) {
        let count = if no_fractal_anim { self.prime_count.max(48) } else { self.prime_count };
        if *prime_current == 0 {
            while *prime_current < count {
                *prime_current += 1;
                state.tick(&self.source, random);
            }
        }
        if no_fractal_anim {
            return;
        }
        if self.max_frame_rate == 0.0 {
            state.tick(&self.source, random);
        } else if self.total_frames != 0 && time.is_finite() {
            let step = 1.0 / f64::from(self.max_frame_rate.clamp(0.1, 100.0));
            let cycle = f64::from(self.total_frames) * step;
            let mut remaining = time - (time / cycle).trunc() * cycle;
            while remaining >= step {
                state.tick(&self.source, random);
                remaining -= step;
            }
        }
    }

    pub fn rgba(&self, pixels: &[u8]) -> Vec<u8> {
        pixels.iter().flat_map(|&index| self.palette[usize::from(index)]).collect()
    }
}

impl Catalog {
    pub fn wet_texture(&mut self, path: &str) -> Option<WetTexture> {
        let package_name = package_name(path)?;
        let found = self.object(path, "WetTexture")?;
        let export = found.package.exports().get(found.index)?;
        let props = object_properties(found.package, found.file, export).ok()?;
        let int = |name| find(&props, name).and_then(Property::int).unwrap_or(0);
        let width = usize::try_from(int("USize")).ok()?;
        let height = usize::try_from(int("VSize")).ok()?;
        let count = usize::try_from(int("NumDrops")).ok()?;
        if count > 256 {
            return None;
        }
        let amplitude = find(&props, "WaveAmp").and_then(Property::byte).unwrap_or(0);
        let prime_count = find(&props, "PrimeCount").and_then(Property::byte).unwrap_or(0);
        let max_frame_rate = find(&props, "MaxFrameRate").and_then(Property::float).unwrap_or(0.0);
        let total_frames = int("TotalFrameNum");
        let mut drops = vec![[0u8; 8]; count];
        for prop in props.iter().filter(|p| p.name == "Drops" && (p.index as usize) < count) {
            let fields = prop.fields(found.package)?;
            drops[prop.index as usize] = ["Type", "Depth", "X", "Y", "ByteA", "ByteB", "ByteC", "ByteD"]
                .map(|name| find(&fields, name).and_then(Property::byte).unwrap_or(0));
        }
        let source = find(&props, "SourceTexture")?.object(found.package)?;
        let source_path = full_path(&package_name, found.package, found.objects, source)?;
        let simulation = WaterSimulation::new(width, height, amplitude, drops)?;
        if !self.object_class(&source_path)?.eq_ignore_ascii_case("Texture") {
            return None;
        }
        let raw = self.texture_raw(&source_path)?;
        if raw.format != TextureFormat::P8 {
            return None;
        }
        let palette = raw.palette?;
        if palette.len() != 256 {
            return None;
        }
        let source = raw.mips.first()?;
        let (sw, sh) = (source.width as usize, source.height as usize);
        if !sw.is_power_of_two() || !sh.is_power_of_two() || sw > width || sh > height || source.data.len() < sw * sh {
            return None;
        }
        let dx = width.ilog2() - sw.ilog2();
        let dy = height.ilog2() - sh.ilog2();
        let source = (0..width * height).map(|i| source.data[((i / width) >> dy) * sw + ((i % width) >> dx)]).collect();
        Some(WetTexture { simulation, source, palette, prime_count, max_frame_rate, total_frames })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texture() -> WetTexture {
        WetTexture {
            simulation: WaterSimulation::new(8, 8, 128, vec![[8, 255, 0, 0, 0, 0, 0, 6]]).unwrap(),
            source: (0..64).collect(),
            palette: vec![[255; 4]; 256],
            prime_count: 0,
            max_frame_rate: 0.0,
            total_frames: 0,
        }
    }

    #[test]
    fn no_fractal_anim_primes_48_ticks_and_freezes() {
        let wet = texture();
        let mut state = wet.simulation.clone();
        let mut expected = state.clone();
        let mut random = FireRandom::from_seed(1);
        let mut prime = 0;
        wet.evaluate_with_animation(&mut state, &mut prime, 0.0, &mut random, true);
        for _ in 0..48 {
            expected.tick(&wet.source, &mut random);
        }
        assert_eq!(prime, 48);
        assert_eq!(state.pixels, expected.pixels);
        wet.evaluate_with_animation(&mut state, &mut prime, 10.0, &mut random, true);
        assert_eq!(state.global_phase, 48);
        assert_eq!(state.pixels, expected.pixels);
    }

    #[test]
    fn texture_tick_uses_get_calls_or_absolute_cycle_not_delta_time() {
        let mut wet = texture();
        let mut state = wet.simulation.clone();
        let mut random = FireRandom::from_seed(1);
        let mut prime = 0;
        for time in [0.0, 0.0, 100.0] {
            wet.evaluate(&mut state, &mut prime, time, &mut random);
        }
        assert_eq!(state.global_phase, 3);
        wet.max_frame_rate = 4.0;
        wet.evaluate(&mut state, &mut prime, 100.0, &mut random);
        assert_eq!(state.global_phase, 3);
        wet.total_frames = 8;
        wet.evaluate(&mut state, &mut prime, 0.75, &mut random);
        assert_eq!(state.global_phase, 6);
        wet.evaluate(&mut state, &mut prime, 2.75, &mut random);
        assert_eq!(state.global_phase, 9);
    }
}
