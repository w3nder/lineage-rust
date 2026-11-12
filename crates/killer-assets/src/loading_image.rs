use ue2_assets::{Mip, decode_rgba};

/// `NCLoadingWnd::OnPaint` (0x1009a910) desenha a textura com UV `(0, 0, 1024, 768)` esticada sobre a janela
/// (`killer_ui::loading::LOADING_TEXTURE_UV`): a textura do cliente tem 1024×1024 e só essa faixa é usada.
pub const REGION: (u32, u32) = (1024, 768);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadingImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl LoadingImage {
    pub fn encode(&self) -> Vec<u8> {
        let mut output = [self.width, self.height].into_iter().flat_map(u32::to_le_bytes).collect::<Vec<_>>();
        output.extend_from_slice(&self.rgba);
        output
    }
}

pub fn from_texture(raw: &l2_catalog::RawTexture) -> Option<LoadingImage> {
    let mip = raw.mips.first()?;
    let (width, height) = REGION;
    if mip.width < width || mip.height < height {
        return None;
    }
    let decoded = decode_rgba(raw.format, &Mip { width: mip.width, height: mip.height, data: &mip.data }, raw.palette.as_deref()).ok()?;
    let stride = mip.width as usize * 4;
    let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
    for row in 0..height as usize {
        rgba.extend_from_slice(decoded.get(row * stride..row * stride + width as usize * 4)?);
    }
    Some(LoadingImage { width, height, rgba })
}

#[cfg(test)]
mod tests {
    use super::*;
    use l2_catalog::{RawMip, RawTexture};
    use ue2_assets::TextureFormat;

    #[test]
    fn only_the_1024_by_768_region_of_the_texture_is_kept() {
        let (width, height) = (1024_u32, 1024_u32);
        let mut data = vec![0_u8; (width * height * 4) as usize];
        for y in 0..height as usize {
            for x in 0..width as usize {
                let offset = (y * width as usize + x) * 4;
                data[offset..offset + 4].copy_from_slice(&[x as u8, y as u8, (y >> 8) as u8, 255]);
            }
        }
        let raw = RawTexture { format: TextureFormat::Rgba8, palette: None, mips: vec![RawMip { width, height, data }] };
        let image = from_texture(&raw).expect("imagem");
        assert_eq!((image.width, image.height), REGION);
        assert_eq!(image.rgba.len(), 1024 * 768 * 4);
        let pixel = |x: usize, y: usize| image.rgba[(y * 1024 + x) * 4..(y * 1024 + x) * 4 + 4].to_vec();
        assert_eq!(pixel(5, 7), decode_rgba(TextureFormat::Rgba8, &Mip { width, height, data: &raw.mips[0].data }, None).unwrap()[(7 * 1024 + 5) * 4..(7 * 1024 + 5) * 4 + 4]);
        assert_eq!(pixel(1023, 767)[1], 767_u32 as u8);
    }

    #[test]
    fn a_texture_smaller_than_the_region_is_refused() {
        let raw = RawTexture { format: TextureFormat::Rgba8, palette: None, mips: vec![RawMip { width: 512, height: 512, data: vec![0; 512 * 512 * 4] }] };
        assert!(from_texture(&raw).is_none());
    }
}
