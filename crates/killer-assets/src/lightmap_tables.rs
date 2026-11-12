pub const ATTENUATION_ENTRIES: usize = 0x1000;
pub const ATTENUATION_SCALE: f64 = 0.000_244_140_625;
pub const SHADOW_INDICES: usize = 128;
pub const SHADOW_ROWS: usize = 4;
pub const KERNEL: [[i32; 8]; 4] = [[0, 0x18, 0x28, 0x18, 0, 0, 0, 0], [0, 0x28, 0x40, 0x28, 0, 0, 0, 0], [0, 0x18, 0x28, 0x18, 0, 0, 0, 0], [0; 8]];

pub fn attenuation() -> Vec<f32> {
    (0..ATTENUATION_ENTRIES as u32)
        .map(|index| {
            let root = (f64::from(index + 1) * ATTENUATION_SCALE).sqrt();
            let cube = root * root * root;
            (((cube + cube) - root * root * 3.0 + 1.0) / root) as f32
        })
        .collect()
}

pub fn shadow_colors() -> Vec<[u32; SHADOW_ROWS]> {
    let total: i32 = KERNEL.iter().flatten().sum();
    (0..SHADOW_INDICES as u32)
        .map(|value| {
            std::array::from_fn(|row| {
                let mut packed = 0u32;
                for shift in 0..4 {
                    let mut sum = 0;
                    for (bit, weight) in KERNEL[row].iter().enumerate() {
                        if value & (1 << (bit + shift)) != 0 {
                            sum += weight;
                        }
                    }
                    packed += ((sum * 0xff / total) as u32) << (shift * 8);
                }
                packed
            })
        })
        .collect()
}

pub const SQRT_HALF_ENTRIES: usize = 0x400;
pub const INVERSE_ROOT_MANTISSA_ENTRIES: usize = 0x400;
pub const INVERSE_ROOT_EXPONENT_ENTRIES: usize = 0x200;
const INVERSE_ROOT_POWER: f64 = -0.5;

pub fn sqrt_table() -> Vec<u32> {
    let halves: Vec<f32> = (0..SQRT_HALF_ENTRIES as u32)
        .map(|index| {
            let value = f32::from_bits(0x3f80_0000 + index * 0x2000);
            let root = f64::from(value).sqrt() as f32;
            f32::from_bits(root.to_bits().wrapping_sub(0x2000_0000))
        })
        .collect();
    let scaled = halves.iter().map(|shifted| (f64::from(*shifted) * 2.0_f64.sqrt()) as f32);
    let doubled = halves.iter().map(|shifted| *shifted + *shifted);
    scaled.chain(doubled).map(f32::to_bits).collect()
}

fn power_or_zero(value: f32) -> f32 {
    let result = f64::from(value).abs().powf(INVERSE_ROOT_POWER) as f32;
    if result.is_nan() { 0.0 } else { result }
}

pub fn inverse_root_mantissa() -> Vec<f32> {
    (0..INVERSE_ROOT_MANTISSA_ENTRIES as u32).map(|index| power_or_zero(f32::from_bits(0x3f80_0000 + index * 0x2000))).collect()
}

pub fn inverse_root_exponent() -> Vec<f32> {
    (0..INVERSE_ROOT_EXPONENT_ENTRIES as u32).map(|index| power_or_zero(f32::from_bits(index << 23))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_attenuation_curve_starts_at_64_and_ends_at_zero() {
        let table = attenuation();
        assert!((table[0] - 63.953_613).abs() < 1.0e-3);
        assert_eq!(table[4095], 0.0);
    }
}
