#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameCursor {
    pub width: u16,
    pub height: u16,
    pub hotspot: [u16; 2],
    pub rgba: Vec<u8>,
}

impl GameCursor {
    pub fn encode(&self) -> Vec<u8> {
        let mut output = [self.width, self.height, self.hotspot[0], self.hotspot[1]].into_iter().flat_map(u16::to_le_bytes).collect::<Vec<_>>();
        output.extend_from_slice(&self.rgba);
        output
    }
}

pub fn from_texture(raw: &l2_catalog::RawTexture) -> Option<GameCursor> {
    use ue2_assets::{Mip, TextureFormat, decode_rgba};
    if !matches!(raw.format, TextureFormat::Dxt1 | TextureFormat::Dxt3) { return None; }
    let mip = raw.mips.first()?;
    let width = u16::try_from(mip.width).ok().filter(|width| *width > 0)?;
    let height = u16::try_from(mip.height).ok().filter(|height| *height > 0)?;
    let rgba = decode_rgba(raw.format, &Mip { width: mip.width, height: mip.height, data: &mip.data }, raw.palette.as_deref()).ok()?;
    Some(GameCursor { width, height, hotspot: [width / 2, height / 2], rgba })
}

pub const GROUPS: [u32; 7] = [102, 101, 106, 107, 127, 130, 131];

fn word(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(bytes.get(offset..offset.checked_add(2)?)?.try_into().ok()?))
}

fn dword(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(offset..offset.checked_add(4)?)?.try_into().ok()?))
}

pub fn load(bytes: &[u8]) -> Option<Vec<GameCursor>> {
    let pe = dword(bytes, 0x3c)? as usize;
    if bytes.get(pe..pe.checked_add(4)?)? != b"PE\0\0" { return None; }
    bytes.get(pe..pe.checked_add(24 + 120)?)?;
    let count = word(bytes, pe + 6)? as usize;
    let optional = word(bytes, pe + 20)? as usize;
    let sections = pe.checked_add(24 + optional)?;
    bytes.get(sections..sections.checked_add(count.checked_mul(40)?)?)?;
    let rva = |address: u32| -> Option<usize> {
        for index in 0..count {
            let section = sections.checked_add(index.checked_mul(40)?)?;
            let start = dword(bytes, section + 12)?;
            let size = dword(bytes, section + 16)?;
            if let Some(delta) = address.checked_sub(start).filter(|&delta| delta < size) {
                return (dword(bytes, section + 20)? as usize).checked_add(delta as usize);
            }
        }
        None
    };
    if word(bytes, pe + 24)? != 0x10b { return None; }
    let base = rva(dword(bytes, pe + 24 + 112)?)?;
    let resource = |kind: u32, id: u32| -> Option<&[u8]> {
        let mut directory = base;
        for key in [Some(kind), Some(id), None] {
            bytes.get(directory..directory.checked_add(16)?)?;
            let count = usize::from(word(bytes, directory + 12)?) + usize::from(word(bytes, directory + 14)?);
            let entry = (0..count).find_map(|index| {
                let entry = directory.checked_add(16 + index.checked_mul(8)?)?;
                let name = dword(bytes, entry)?;
                key.is_none_or(|key| key == name).then(|| dword(bytes, entry + 4)).flatten()
            })?;
            directory = base.checked_add((entry & 0x7fff_ffff) as usize)?;
            if key.is_some() && entry & 0x8000_0000 == 0 { return None; }
        }
        bytes.get(directory..directory.checked_add(16)?)?;
        let start = rva(dword(bytes, directory)?)?;
        let size = dword(bytes, directory + 4)? as usize;
        bytes.get(start..start.checked_add(size)?)
    };
    GROUPS.into_iter().map(|id| {
        let group = resource(12, id)?;
        if word(group, 2)? != 2 || word(group, 4)? != 1 { return None; }
        decode(resource(1, u32::from(word(group, 18)?))?)
    }).collect()
}

fn decode(bytes: &[u8]) -> Option<GameCursor> {
    let hotspot = [word(bytes, 0)?, word(bytes, 2)?];
    let dib = bytes.get(4..)?;
    if dword(dib, 0)? != 40 || word(dib, 12)? != 1 || word(dib, 14)? != 8 || dword(dib, 16)? != 0 { return None; }
    let width = u16::try_from(dword(dib, 4)?).ok()?;
    let full_height = dword(dib, 8)?;
    if full_height % 2 != 0 { return None; }
    let height = u16::try_from(full_height / 2).ok()?;
    if width == 0 || height == 0 || hotspot[0] >= width || hotspot[1] >= height { return None; }
    let (w, h) = (usize::from(width), usize::from(height));
    let colors = match dword(dib, 32)? { 0 => 256, count if count <= 256 => count as usize, _ => return None };
    let xor = 40 + colors * 4;
    let stride = w.div_ceil(4) * 4;
    let mask = xor + stride * h;
    let mask_stride = w.div_ceil(32) * 4;
    dib.get(..mask.checked_add(mask_stride * h)?)?;
    let length = w.checked_mul(h)?.checked_mul(4)?;
    let mut rgba = Vec::new();
    rgba.try_reserve_exact(length).ok()?;
    for y in (0..h).rev() {
        for x in 0..w {
            let index = usize::from(*dib.get(xor + y * stride + x)?);
            if index >= colors { return None; }
            let color = dib.get(40 + index * 4..44 + index * 4)?;
            let opaque = dib[mask + y * mask_stride + x / 8] & (0x80 >> (x % 8)) == 0;
            rgba.extend([color[2], color[1], color[0], if opaque { 255 } else { 0 }]);
        }
    }
    Some(GameCursor { width, height, hotspot, rgba })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexed_cursor_preserves_hotspot_bottom_up_palette_and_mask() {
        let mut bytes = vec![0; 4 + 40 + 8 + 8 + 8];
        bytes[0..2].copy_from_slice(&1u16.to_le_bytes());
        let dib = &mut bytes[4..];
        for (offset, value) in [(0, 40u32), (4, 2), (8, 4), (32, 2)] {
            dib[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        dib[12..14].copy_from_slice(&1u16.to_le_bytes());
        dib[14..16].copy_from_slice(&8u16.to_le_bytes());
        dib[40..48].copy_from_slice(&[3, 2, 1, 0, 6, 5, 4, 0]);
        dib[48] = 1;
        dib[53] = 1;
        dib[60] = 0x40;
        let cursor = decode(&bytes).unwrap();
        assert_eq!(cursor.hotspot, [1, 0]);
        assert_eq!(cursor.rgba, [1, 2, 3, 255, 4, 5, 6, 0, 4, 5, 6, 255, 1, 2, 3, 255]);
        for cut in 0..bytes.len() { assert!(decode(&bytes[..cut]).is_none()); }
        bytes[0..2].copy_from_slice(&2u16.to_le_bytes());
        assert!(decode(&bytes).is_none());
    }

    #[test]
    fn create_texture_cursor_10039640_uses_mip_zero_and_center_hotspot() {
        use l2_catalog::{RawMip, RawTexture};
        use ue2_assets::TextureFormat;
        for (format, data) in [
            (TextureFormat::Dxt1, vec![0x00, 0xf8, 0, 0, 0, 0, 0, 0]),
            (TextureFormat::Dxt3, vec![255, 255, 255, 255, 255, 255, 255, 255, 0, 0xf8, 0, 0, 0, 0, 0, 0]),
        ] {
            let raw = RawTexture { format, palette: None, mips: vec![RawMip { width: 4, height: 4, data }] };
            let cursor = from_texture(&raw).unwrap();
            assert_eq!(cursor.hotspot, [2, 2]);
            assert_eq!(cursor.rgba.len(), 64);
            assert!(cursor.rgba.as_chunks::<4>().0.iter().all(|pixel| *pixel == [255, 0, 0, 255]));
            assert_eq!(&cursor.encode()[..8], &[4, 0, 4, 0, 2, 0, 2, 0]);
        }
    }

    #[test]
    fn invalid_pe_offsets_are_rejected() {
        for offset in [0, 64, u32::MAX] {
            let mut bytes = vec![0; 64];
            bytes[0x3c..].copy_from_slice(&offset.to_le_bytes());
            assert!(load(&bytes).is_none());
        }
    }

    #[test]
    #[ignore]
    fn create_texture_cursor_10039640_audits_all_original_item_skill_action_icons() {
        let root = std::env::var("KILLER_CLIENT_DIR").unwrap();
        let mut client = crate::client::Client::open(std::path::Path::new(&root)).unwrap();
        let tables = client.tables().unwrap();
        let paths = tables.item_icons.items.values().flat_map(|item| item.icons.iter().map(String::as_str))
            .chain(tables.skillgrp.icons()).chain(tables.actions.iter().map(|action| action.icon.as_str()))
            .map(str::trim).filter(|path| !path.is_empty() && path.contains('.'))
            .map(str::to_lowercase).collect::<std::collections::BTreeSet<_>>();
        let mut supported = 0;
        let mut unsupported = 0;
        let mut missing = Vec::new();
        for path in &paths {
            let Some(raw) = client.catalog().texture_raw(path) else { missing.push(path); continue; };
            if matches!(raw.format, ue2_assets::TextureFormat::Dxt1 | ue2_assets::TextureFormat::Dxt3) {
                let cursor = from_texture(&raw).unwrap_or_else(|| panic!("cursor: {path}"));
                assert_eq!(cursor.rgba.len(), usize::from(cursor.width) * usize::from(cursor.height) * 4);
                assert_eq!(cursor.hotspot, [cursor.width / 2, cursor.height / 2]);
                supported += 1;
            } else {
                assert!(from_texture(&raw).is_none(), "{path}");
                unsupported += 1;
            }
        }
        println!("Ícones: {} únicos; {} DXT1/DXT3; {} formatos não aceitos pelo original; {} ausentes: {:?}", paths.len(), supported, unsupported, missing.len(), missing);
        assert!(supported > 0);
        assert_eq!(paths.len(), supported + unsupported + missing.len());
    }

    #[test]
    #[ignore]
    fn load_cursors_10038350_reads_all_original_resources() {
        let root = std::env::var("KILLER_CLIENT_DIR").unwrap();
        let mut client = crate::client::Client::open(std::path::Path::new(&root)).unwrap();
        let bytes = client.read_system_file("nwindow.dll").unwrap();
        let cursors = load(&bytes).unwrap();
        assert_eq!(cursors.len(), 7);
        for (index, cursor) in cursors.iter().enumerate() {
            assert_eq!((cursor.width, cursor.height), (32, 32));
            assert!(cursor.rgba.as_chunks::<4>().0.iter().any(|pixel| pixel[3] == 0));
            assert!(cursor.rgba.as_chunks::<4>().0.iter().any(|pixel| pixel[3] == 255));
            crate::textures::write_png(&std::env::temp_dir().join(format!("killer-cursor-{index}.png")), 32, 32, &cursor.rgba).unwrap();
        }
        for cut in [0, 63, 128, bytes.len() / 2] { assert!(load(&bytes[..cut]).is_none()); }
    }
}
