use crate::crest_dxt;

pub const SYSMSG_CLAN_FORMAT: u32 = 211;
pub const SYSMSG_ALLY_FORMAT: u32 = 529;
pub const SYSMSG_LARGE_FORMAT: u32 = 1478;
pub const SYSMSG_FILE_NOT_FOUND: u32 = 528;
pub const MAX_FILE_BYTES: u32 = 0xC00;
const BMP_MAGIC: u16 = 0x4D42;
const HEADER_BYTES: usize = 0xE;
const INFO_BYTES: usize = 0x28;
const PALETTE_AT: usize = 0x36;
const PALETTE_BYTES: usize = 0x400;
const PIXELS_AT: usize = 0x436;
const SOURCE_ROWS: usize = 12;
const CREST_ROWS: i32 = 16;
const LARGE_SIZE: i32 = 64;
const DDS_HEADER_BYTES: usize = 0x80;
const DDS_MAGIC: u32 = 0x2053_4444;
const DDS_FOURCC_DXT1: u32 = 0x3154_5844;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrestKind {
    Clan,
    Ally,
    Large,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrestError(pub u32);

struct Small {
    width: i32,
    message: u32,
    file_bytes: usize,
    image_bytes: u32,
}

fn small(kind: CrestKind) -> Option<Small> {
    match kind {
        CrestKind::Clan => Some(Small { width: 16, message: SYSMSG_CLAN_FORMAT, file_bytes: 0x536, image_bytes: 0x100 }),
        CrestKind::Ally => Some(Small { width: 8, message: SYSMSG_ALLY_FORMAT, file_bytes: 0x4B6, image_bytes: 0x80 }),
        CrestKind::Large => None,
    }
}

fn byte_at(data: &[u8], at: usize) -> u8 {
    data.get(at).copied().unwrap_or(0)
}

fn u16_at(data: &[u8], at: usize) -> u16 {
    u16::from(byte_at(data, at)) | u16::from(byte_at(data, at + 1)) << 8
}

fn u32_at(data: &[u8], at: usize) -> u32 {
    u32::from(u16_at(data, at)) | u32::from(u16_at(data, at + 2)) << 16
}

fn put_u32(data: &mut [u8], at: usize, value: u32) {
    data[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

fn slice_of(data: &[u8], at: usize, length: usize) -> Vec<u8> {
    (0..length).map(|offset| byte_at(data, at + offset)).collect()
}

fn fits(file: &[u8], width: i32, height: i32) -> bool {
    u16_at(file, 0) == BMP_MAGIC && u32_at(file, 0x12) as i32 == width && u32_at(file, 0x16) as i32 == height && u16_at(file, 0x1C) <= 8
}

pub fn rebuild_small(file: &[u8], kind: CrestKind) -> Result<Vec<u8>, CrestError> {
    let spec = small(kind).ok_or(CrestError(SYSMSG_LARGE_FORMAT))?;
    if u16_at(file, 0) != BMP_MAGIC || u32_at(file, 2) > MAX_FILE_BYTES || !fits(file, spec.width, SOURCE_ROWS as i32) {
        return Err(CrestError(spec.message));
    }
    let mut crest = vec![0xFFu8; spec.file_bytes];
    let mut info = slice_of(file, HEADER_BYTES, INFO_BYTES);
    put_u32(&mut info, 0x12 - HEADER_BYTES, spec.width as u32);
    put_u32(&mut info, 0x16 - HEADER_BYTES, CREST_ROWS as u32);
    put_u32(&mut info, 0x22 - HEADER_BYTES, spec.image_bytes);
    crest[..HEADER_BYTES].copy_from_slice(&slice_of(file, 0, HEADER_BYTES));
    put_u32(&mut crest, 2, spec.file_bytes as u32);
    crest[HEADER_BYTES..HEADER_BYTES + INFO_BYTES].copy_from_slice(&info);
    crest[PALETTE_AT..PALETTE_AT + PALETTE_BYTES].copy_from_slice(&slice_of(file, PALETTE_AT, PALETTE_BYTES));
    let width = spec.width as usize;
    for row in 0..SOURCE_ROWS {
        let at = PIXELS_AT + row * width;
        crest[at..at + width].copy_from_slice(&slice_of(file, at, width));
    }
    Ok(crest)
}

pub fn check_large(file: &[u8]) -> Result<(), CrestError> {
    if fits(file, LARGE_SIZE, LARGE_SIZE) { Ok(()) } else { Err(CrestError(SYSMSG_LARGE_FORMAT)) }
}

pub fn import_bgra(bmp: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let (width, height) = (u32_at(bmp, 0x12), u32_at(bmp, 0x16));
    let pow2 = |value: u32| value & value.wrapping_sub(1) == 0;
    if bmp.len() < PALETTE_AT || u16_at(bmp, 0) != BMP_MAGIC || width == 0 || height == 0 || !pow2(width) || !pow2(height) {
        return None;
    }
    if u32_at(bmp, 0x1E) != 0 || u16_at(bmp, 0x1A) != 1 || u16_at(bmp, 0x1C) != 8 {
        return None;
    }
    let used = match u32_at(bmp, 0x2E) {
        0 | 0x100.. => 0x100,
        count => count as usize,
    };
    let has_alpha = (0..used).any(|index| byte_at(bmp, 0x39 + index * 4) != 0);
    let mut palette = vec![[0u8, 0, 0, 0xFF]; 0x100];
    for (index, entry) in palette.iter_mut().enumerate().take(used) {
        let alpha = if has_alpha { byte_at(bmp, 0x39 + index * 4) } else { 0xFF };
        *entry = [byte_at(bmp, PALETTE_AT + index * 4), byte_at(bmp, 0x37 + index * 4), byte_at(bmp, 0x38 + index * 4), alpha];
    }
    let data_at = u32_at(bmp, 10) as usize;
    let mut bgra = Vec::with_capacity((width * height * 4) as usize);
    for row in (0..height as usize).rev() {
        for column in 0..width as usize {
            bgra.extend_from_slice(&palette[usize::from(byte_at(bmp, data_at + row * width as usize + column))]);
        }
    }
    Some((width, height, bgra))
}

pub fn dds(width: u32, height: u32, reserved: u32, data: &[u8]) -> Vec<u8> {
    let mut file = vec![0u8; DDS_HEADER_BYTES];
    let fields = [
        (0, DDS_MAGIC),
        (1, 0x7C),
        (2, 0x81007),
        (3, height),
        (4, width),
        (5, data.len() as u32),
        (9, reserved),
        (0x13, 0x20),
        (0x14, 4),
        (0x15, DDS_FOURCC_DXT1),
    ];
    for (index, value) in fields {
        put_u32(&mut file, index * 4, value);
    }
    file.extend_from_slice(data);
    file
}

pub fn bmp_to_dxt1(bmp: &[u8], extra: i32, kind: CrestKind) -> Result<Vec<u8>, CrestError> {
    let format = if kind == CrestKind::Large { SYSMSG_LARGE_FORMAT } else { small(kind).map_or(0, |spec| spec.message) };
    let (width, height, mut bgra) = import_bgra(bmp).ok_or(CrestError(format))?;
    if extra != 0 {
        for pixel in bgra.chunks_exact_mut(4) {
            pixel[3] = if pixel == [0xFF, 0x00, 0xFF, 0xFF] { 0 } else { 0xFF };
        }
    }
    let data = crest_dxt::compress(&bgra, width, height, extra != 0).map_err(|_| CrestError(format))?;
    let (out_width, out_height) = match kind {
        CrestKind::Large => (width, height),
        CrestKind::Clan => (16, 16),
        CrestKind::Ally => (8, 16),
    };
    Ok(dds(out_width, out_height, extra as u32, &data))
}

pub fn prepare(file: &[u8], kind: CrestKind, extra: i32) -> Result<Vec<u8>, CrestError> {
    match kind {
        CrestKind::Large => {
            check_large(file)?;
            bmp_to_dxt1(file, extra, kind)
        }
        _ => bmp_to_dxt1(&rebuild_small(file, kind)?, extra, kind),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bmp(width: i32, height: i32, bits: u16, pixel: impl Fn(usize, usize) -> u8) -> Vec<u8> {
        let image = (width * height) as usize;
        let mut file = vec![0u8; PIXELS_AT + image];
        file[0..2].copy_from_slice(b"BM");
        let length = file.len() as u32;
        put_u32(&mut file, 2, length);
        put_u32(&mut file, 10, PIXELS_AT as u32);
        put_u32(&mut file, 14, 40);
        put_u32(&mut file, 0x12, width as u32);
        put_u32(&mut file, 0x16, height as u32);
        file[0x1A] = 1;
        file[0x1C] = bits as u8;
        put_u32(&mut file, 0x22, image as u32);
        for index in 0..256usize {
            file[PALETTE_AT + index * 4..PALETTE_AT + index * 4 + 3].copy_from_slice(&[index as u8, (index * 2) as u8, 255 - index as u8]);
        }
        for row in 0..height as usize {
            for column in 0..width as usize {
                file[PIXELS_AT + row * width as usize + column] = pixel(column, row);
            }
        }
        file
    }

    #[test]
    fn the_import_flips_the_rows_and_reads_the_palette_as_blue_green_red() {
        let source = bmp(16, 16, 8, |column, row| (row * 16 + column) as u8);
        let (width, height, bgra) = import_bgra(&source).unwrap();
        assert_eq!((width, height, bgra.len()), (16, 16, 1024));
        assert_eq!(&bgra[..4], &[240, 224, 15, 255]);
        assert_eq!(&bgra[1020..1024], &[15, 30, 240, 255]);
    }

    #[test]
    fn a_clan_crest_is_rebuilt_as_16_by_16_with_the_last_rows_white_and_sent_as_dxt1() {
        let source = bmp(16, 12, 8, |column, row| (column + row * 3) as u8);
        let crest = rebuild_small(&source, CrestKind::Clan).unwrap();
        assert_eq!((crest.len(), u32_at(&crest, 0x16), u32_at(&crest, 0x22), u32_at(&crest, 2)), (0x536, 16, 0x100, 0x536));
        assert!(crest[PIXELS_AT + 12 * 16..].iter().all(|byte| *byte == 0xFF));
        let file = prepare(&source, CrestKind::Clan, 0).unwrap();
        assert_eq!(file.len(), DDS_HEADER_BYTES + 128);
        assert_eq!((u32_at(&file, 0), u32_at(&file, 4), u32_at(&file, 8), u32_at(&file, 12), u32_at(&file, 16), u32_at(&file, 20)), (DDS_MAGIC, 0x7C, 0x81007, 16, 16, 128));
        assert_eq!((u32_at(&file, 0x4C), u32_at(&file, 0x50), u32_at(&file, 0x54)), (0x20, 4, DDS_FOURCC_DXT1));
    }

    #[test]
    fn the_alliance_crest_is_8_by_16_and_the_large_one_keeps_its_own_size() {
        let ally = bmp(8, 12, 8, |column, row| (column * row) as u8);
        let file = prepare(&ally, CrestKind::Ally, 0).unwrap();
        assert_eq!((file.len(), u32_at(&file, 12), u32_at(&file, 16)), (DDS_HEADER_BYTES + 64, 16, 8));
        let large = bmp(64, 64, 8, |column, row| ((column / 4 + row / 4) % 7) as u8);
        let file = prepare(&large, CrestKind::Large, 0).unwrap();
        assert_eq!((file.len(), u32_at(&file, 12), u32_at(&file, 16), u32_at(&file, 20)), (DDS_HEADER_BYTES + 2048, 64, 64, 2048));
    }

    #[test]
    fn files_that_do_not_fit_answer_with_the_message_of_their_kind() {
        let wrong_size = bmp(16, 16, 8, |_, _| 0);
        assert_eq!(rebuild_small(&wrong_size, CrestKind::Clan), Err(CrestError(SYSMSG_CLAN_FORMAT)));
        let deep = bmp(16, 12, 24, |_, _| 0);
        assert_eq!(rebuild_small(&deep, CrestKind::Clan), Err(CrestError(SYSMSG_CLAN_FORMAT)));
        let ally = bmp(16, 12, 8, |_, _| 0);
        assert_eq!(rebuild_small(&ally, CrestKind::Ally), Err(CrestError(SYSMSG_ALLY_FORMAT)));
        assert_eq!(prepare(&bmp(32, 64, 8, |_, _| 0), CrestKind::Large, 0), Err(CrestError(SYSMSG_LARGE_FORMAT)));
        assert_eq!(prepare(b"not a bitmap", CrestKind::Large, 0), Err(CrestError(SYSMSG_LARGE_FORMAT)));
        let mut oversize = bmp(16, 12, 8, |_, _| 0);
        put_u32(&mut oversize, 2, MAX_FILE_BYTES + 1);
        assert_eq!(rebuild_small(&oversize, CrestKind::Clan), Err(CrestError(SYSMSG_CLAN_FORMAT)));
    }

    #[test]
    fn the_transparent_key_color_becomes_alpha_zero_and_selects_the_one_bit_alpha_format() {
        let mut source = bmp(16, 12, 8, |column, _| u8::from(column == 0));
        source[PALETTE_AT..PALETTE_AT + 3].copy_from_slice(&[255, 0, 255]);
        let crest = rebuild_small(&source, CrestKind::Clan).unwrap();
        let (_, _, mut bgra) = import_bgra(&crest).unwrap();
        for pixel in bgra.chunks_exact_mut(4) {
            pixel[3] = if pixel == [0xFF, 0x00, 0xFF, 0xFF] { 0 } else { 0xFF };
        }
        assert!(bgra.chunks_exact(4).any(|pixel| pixel[3] == 0));
        let plain = prepare(&source, CrestKind::Clan, 0).unwrap();
        let keyed = prepare(&source, CrestKind::Clan, 1).unwrap();
        assert_eq!(u32_at(&keyed, 0x24), 1);
        assert_ne!(plain[DDS_HEADER_BYTES..], keyed[DDS_HEADER_BYTES..]);
    }
}
