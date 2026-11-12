#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GammaSettings {
    pub gamma: f32,
    pub brightness: f32,
    pub contrast: f32,
}

impl GammaSettings {
    pub fn ramp(self) -> [u16; 256] {
        std::array::from_fn(|index| {
            let power = (index as f64 * f64::from(f32::from_bits(0x3b808081))).powf(1.0 / f64::from(self.gamma));
            let value = ((f64::from(self.contrast) + 0.5) * power * 65535.0
                + (f64::from(self.brightness) - 0.5) * 32768.0
                - f64::from(self.contrast) * 32768.0
                + 16384.0) as f32;
            let rounded = (f64::from(value) + 0.5).floor();
            if !rounded.is_finite() || rounded < f64::from(i32::MIN) || rounded >= 2147483648.0 {
                0
            } else {
                (rounded as i32).clamp(0, 65535) as u16
            }
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextureFiltering {
    pub trilinear: bool,
    pub mipmaps: bool,
}

impl TextureFiltering {
    pub fn sampler(self, address: wgpu::AddressMode) -> wgpu::SamplerDescriptor<'static> {
        let filter = if self.trilinear { wgpu::FilterMode::Linear } else { wgpu::FilterMode::Nearest };
        wgpu::SamplerDescriptor {
            address_mode_u: address,
            address_mode_v: address,
            mag_filter: filter,
            min_filter: filter,
            mipmap_filter: if self.trilinear {
                wgpu::MipmapFilterMode::Linear
            } else {
                wgpu::MipmapFilterMode::Nearest
            },
            lod_max_clamp: if self.mipmaps { 32.0 } else { 0.0 },
            ..Default::default()
        }
    }
}

pub fn multisample_capability(adapter: &wgpu::Adapter, device: &wgpu::Device) -> i32 {
    let flags = |format: wgpu::TextureFormat| {
        if device.features().contains(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES) {
            adapter.get_texture_format_features(format).flags
        } else {
            format.guaranteed_format_features(device.features()).flags
        }
    };
    let color = flags(crate::pipeline::COLOR_FORMAT);
    let depth = flags(crate::pipeline::DEPTH_FORMAT);
    if !color.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_RESOLVE)
        || !color.sample_count_supported(2)
        || !depth.sample_count_supported(2)
    {
        return 0;
    }
    if color.sample_count_supported(4) && depth.sample_count_supported(4) { 2 } else { 1 }
}

pub(crate) struct MultisampleFrame {
    pub size: (u32, u32),
    pub color: wgpu::TextureView,
    pub depth: wgpu::TextureView,
}

impl MultisampleFrame {
    pub fn new(device: &wgpu::Device, size: (u32, u32), samples: u32) -> Self {
        let texture = |format| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some("video multisample"),
                    size: wgpu::Extent3d { width: size.0.max(1), height: size.1.max(1), depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: samples,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                })
                .create_view(&Default::default())
        };
        Self { size, color: texture(crate::pipeline::COLOR_FORMAT), depth: texture(crate::pipeline::DEPTH_FORMAT) }
    }
}

pub fn first_texture_mip(width: u32, height: u32, detail: i32, minimum: i32, maximum: i32) -> usize {
    let mut level = if detail < minimum {
        minimum
    } else if detail < maximum {
        detail
    } else {
        maximum
    };
    while level > 0 && (width >> (level as u32 & 31)) < 64 && (height >> (level as u32 & 31)) < 64 {
        level -= 1;
    }
    level.max(minimum).max(0) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_gamma_10012190_maps_all_256_entries() {
        let identity = GammaSettings { gamma: 1.0, brightness: 0.5, contrast: 0.5 }.ramp();
        for (index, value) in identity.into_iter().enumerate() {
            assert_eq!(value, index as u16 * 257);
        }
        for gamma in [1.2, 1.0, 0.8, 0.6, 0.4] {
            let ramp = GammaSettings { gamma, brightness: 0.8, contrast: 0.7 }.ramp();
            assert!(ramp.windows(2).all(|pair| pair[0] <= pair[1]));
            assert_eq!(ramp[255], 65535);
        }
        assert_eq!(GammaSettings { gamma: f32::NAN, brightness: 0.5, contrast: 0.5 }.ramp(), [0; 256]);
    }

    #[test]
    fn first_mip_106f5800_preserves_small_textures_and_minimum() {
        for width in [1, 16, 32, 64, 128, 256, 512, 1024] {
            for height in [1, 16, 32, 64, 128, 256, 512, 1024] {
                for detail in 0..=2 {
                    let mip = first_texture_mip(width, height, detail, 0, 10);
                    assert!(mip == 0 || (width >> mip) >= 64 || (height >> mip) >= 64);
                    assert!(mip <= detail as usize);
                    if mip < detail as usize {
                        assert!((width >> (mip + 1)) < 64 && (height >> (mip + 1)) < 64);
                    }
                }
            }
        }
        assert_eq!(first_texture_mip(32, 32, 0, 2, 5), 2);
        assert_eq!(first_texture_mip(1024, 1024, 9, 0, 2), 2);
        assert_eq!(first_texture_mip(256, 16, 2, 0, 8), 2);
    }

    #[test]
    fn trilinear_10010580_changes_all_three_filters() {
        for trilinear in [false, true] {
            for mipmaps in [false, true] {
                let sampler = TextureFiltering { trilinear, mipmaps }.sampler(wgpu::AddressMode::Repeat);
                let filter = if trilinear { wgpu::FilterMode::Linear } else { wgpu::FilterMode::Nearest };
                assert_eq!(sampler.mag_filter, filter);
                assert_eq!(sampler.min_filter, filter);
                assert_eq!(
                    sampler.mipmap_filter,
                    if trilinear { wgpu::MipmapFilterMode::Linear } else { wgpu::MipmapFilterMode::Nearest }
                );
                assert_eq!(sampler.lod_max_clamp == 0.0, !mipmaps);
            }
        }
    }
}
