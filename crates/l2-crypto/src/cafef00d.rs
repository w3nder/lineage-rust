use crate::{Error, HEADER_LEN};

pub(crate) const MAGIC: [u8; 4] = [0xCA, 0xFE, 0xF0, 0x0D];
const VERSION: [u8; 2] = [0x6F, 0x00];
const KEY_START: usize = 6;
const KEY_LEN: usize = 6;
const TABLE_MASK: usize = 0x7F;
const SIGN_BIT: usize = 0x80;

const TABLE: [u8; 128] = [
    0x0c, 0xad, 0xe2, 0xfc, 0xf1, 0xd6, 0xea, 0x65, 0xf6, 0x4d, 0x6c, 0xd1, 0x70, 0x14, 0x80, 0x34, 0x5c, 0x9d, 0xd2,
    0xac, 0x21, 0xe6, 0x3a, 0x95, 0xa6, 0x3d, 0xbc, 0xe1, 0x80, 0xc4, 0xd0, 0xe4, 0x6c, 0xcd, 0x02, 0x5c, 0x11, 0x36,
    0x4a, 0xc5, 0x56, 0xed, 0x8c, 0x31, 0xd0, 0xf4, 0xe0, 0x94, 0xbc, 0x3d, 0x72, 0x0c, 0x41, 0x46, 0x9a, 0xf5, 0x06,
    0xdd, 0xdc, 0x41, 0xe0, 0xa4, 0x30, 0x44, 0x4c, 0xed, 0x22, 0xbc, 0x31, 0x16, 0x2a, 0xa5, 0xb6, 0x0d, 0xac, 0x11,
    0xb0, 0xd4, 0xc0, 0xf4, 0x9c, 0xdd, 0x12, 0x6c, 0x61, 0x26, 0x7a, 0xd5, 0x66, 0xfd, 0xfc, 0x21, 0xc0, 0x84, 0x10,
    0xa4, 0xac, 0x0d, 0x42, 0x1c, 0x51, 0x76, 0x8a, 0x05, 0x16, 0xad, 0xcc, 0x71, 0x10, 0xb4, 0x20, 0x54, 0xfc, 0x7d,
    0xb2, 0xcc, 0x81, 0x86, 0xda, 0x35, 0xc6, 0x9d, 0x1c, 0x81, 0x20, 0x64, 0x70, 0x04,
];

pub(crate) fn matches(data: &[u8]) -> bool {
    data.starts_with(&MAGIC)
}

pub(crate) fn version(data: &[u8]) -> Option<u16> {
    data.get(MAGIC.len()..HEADER_LEN.min(data.len()))?.first_chunk::<2>().map(|v| u16::from_le_bytes(*v))
}

pub(crate) fn decrypt(data: &[u8]) -> Result<Vec<u8>, Error> {
    if data.len() < HEADER_LEN {
        return Err(Error::Truncated);
    }
    if data[MAGIC.len()..KEY_START] != VERSION {
        return Err(Error::UnsupportedVersion(version(data).unwrap_or(0)));
    }
    let key = &data[KEY_START..KEY_START + KEY_LEN];
    Ok(data
        .iter()
        .enumerate()
        .skip(HEADER_LEN)
        .map(|(offset, &byte)| byte ^ TABLE[offset & TABLE_MASK] ^ (offset & SIGN_BIT) as u8 ^ key[offset % KEY_LEN])
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(plain: &[u8], key: [u8; KEY_LEN]) -> Vec<u8> {
        let mut file = vec![0; HEADER_LEN];
        file[..MAGIC.len()].copy_from_slice(&MAGIC);
        file[MAGIC.len()..KEY_START].copy_from_slice(&VERSION);
        file[KEY_START..KEY_START + KEY_LEN].copy_from_slice(&key);
        for (index, &byte) in plain.iter().enumerate() {
            let offset = HEADER_LEN + index;
            file.push(byte ^ TABLE[offset & TABLE_MASK] ^ (offset & SIGN_BIT) as u8 ^ key[offset % KEY_LEN]);
        }
        file
    }

    #[test]
    fn round_trips_across_the_whole_key_period() {
        let plain: Vec<u8> = (0..3000u32).map(|i| (i * 7 + i / 5) as u8).collect();
        for key in [[0; KEY_LEN], [0xEC, 0x8B, 0x52, 0xB6, 0xA2, 0x76]] {
            assert_eq!(decrypt(&encode(&plain, key)).unwrap(), plain);
        }
    }

    #[test]
    fn rejects_truncated_files_and_other_versions() {
        assert!(matches!(decrypt(&MAGIC), Err(Error::Truncated)));
        let mut file = encode(b"x", [1; KEY_LEN]);
        file[MAGIC.len()] = 0x79;
        assert!(matches!(decrypt(&file), Err(Error::UnsupportedVersion(0x79))));
    }
}
