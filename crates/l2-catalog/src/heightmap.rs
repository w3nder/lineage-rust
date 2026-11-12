use ue2_assets::TextureFormat;

pub fn samples(format: TextureFormat, data: &[u8], base: i32, count: usize) -> Option<Vec<u16>> {
    let samples: Vec<_> = match format {
        TextureFormat::G16 => data.as_chunks::<2>().0.iter().map(|&bytes| u16::from_le_bytes(bytes)).collect(),
        TextureFormat::P8 => data.iter().map(|&byte| u16::from(byte) << 8).collect(),
        TextureFormat::Rgba8 => data
            .as_chunks::<4>()
            .0
            .iter()
            .map(|&[b, g, r, _]| {
                if b == 0 && g == 0 && r == 0 {
                    return 0;
                }
                let value = if b > r.max(g) {
                    u16::from(b) + 510
                } else if g > r {
                    u16::from(g) + 255
                } else {
                    u16::from(r)
                };
                (base as u16).wrapping_add(value).wrapping_mul(85)
            })
            .collect(),
        _ => vec![0; count],
    };
    (samples.len() >= count).then_some(samples)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_heightmap_106d2130_formats_ties_zero_and_wrapping() {
        assert_eq!(samples(TextureFormat::P8, &[0, 1, 255], 32, 3).unwrap(), [0, 256, 65280]);
        assert_eq!(samples(TextureFormat::G16, &[1, 2, 255, 255], 32, 2).unwrap(), [513, 65535]);
        let rgba = [0, 0, 0, 255, 3, 3, 3, 0, 4, 4, 3, 0, 5, 4, 3, 255];
        assert_eq!(samples(TextureFormat::Rgba8, &rgba, 1, 4).unwrap(), [0, 340, 22100, 43860]);
        assert_eq!(samples(TextureFormat::Rgba8, &[0, 0, 1, 0], -1, 1).unwrap(), [0]);
        assert_eq!(samples(TextureFormat::Dxt1, &[], 0, 3).unwrap(), [0, 0, 0]);
        assert!(samples(TextureFormat::Rgba8, &[1, 2, 3], 0, 1).is_none());
    }
}
