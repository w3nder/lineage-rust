//! Decryption of Lineage 2 client files protected by a `Lineage2VerXXX` header.
//!
//! High Five ships three schemes: `Ver111` and `Ver121` (single-byte XOR) and
//! `Ver413` (RSA-1024 blocks over a zlib stream). Pure: bytes in, bytes out.

mod cafef00d;
mod ver413;

use std::borrow::Cow;
use std::fmt;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

/// `Lineage2VerXXX` as UTF-16LE.
const HEADER_LEN: usize = 28;
/// Optional footer `[0, ?, ?, crc32, 0]` as little-endian u32s. On XOR files the CRC
/// covers header + payload; on `Ver413` files it does not reliably, so block alignment decides.
const TRAILER_LEN: usize = 20;
const MAGIC: &str = "Lineage2Ver";
pub const PROPRIETARY_CONTAINER_MAGIC: [u8; 4] = [0xCA, 0xFE, 0xF0, 0x0D];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scheme {
    /// No `Lineage2Ver` header; the file is used as-is.
    Plain,
    Ver111,
    Ver121,
    Ver413,
    CafeF00d,
}

#[derive(Debug)]
pub enum Error {
    UnsupportedVersion(u16),
    ProprietaryContainer,
    Truncated,
    MisalignedRsaPayload(usize),
    NoMatchingRsaKey,
    InvalidRsaBlock(usize),
    Inflate(std::io::Error),
    SizeMismatch { declared: usize, actual: usize },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion(v) => write!(f, "unsupported Lineage2Ver{v}"),
            Self::ProprietaryContainer => f.write_str("CAFEF00D container: known, not decodable by this port"),
            Self::Truncated => f.write_str("file is truncated"),
            Self::MisalignedRsaPayload(len) => write!(f, "RSA payload of {len} bytes is not a multiple of 128"),
            Self::NoMatchingRsaKey => f.write_str("no known Ver413 RSA key opens this file"),
            Self::InvalidRsaBlock(i) => write!(f, "RSA block {i} is malformed"),
            Self::Inflate(e) => write!(f, "zlib stream is corrupt: {e}"),
            Self::SizeMismatch { declared, actual } => {
                write!(f, "declared {declared} decompressed bytes, got {actual}")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Identifies the protection scheme from the file header.
pub fn detect(data: &[u8]) -> Result<Scheme, Error> {
    if cafef00d::matches(data) {
        return Ok(Scheme::CafeF00d);
    }
    match header_version(data) {
        None => Ok(Scheme::Plain),
        Some(111) => Ok(Scheme::Ver111),
        Some(121) => Ok(Scheme::Ver121),
        Some(413) => Ok(Scheme::Ver413),
        Some(other) => Err(Error::UnsupportedVersion(other)),
    }
}

/// Decrypts a client file. `path` is only used for its file name, which keys `Ver121`.
pub fn decrypt<'a>(data: &'a [u8], path: &Path) -> Result<Cow<'a, [u8]>, Error> {
    Ok(Cow::Owned(match detect(data)? {
        Scheme::Plain => return Ok(Cow::Borrowed(data)),
        Scheme::Ver111 => xor(xor_payload(data)?, 0xAC),
        Scheme::Ver121 => xor(xor_payload(data)?, ver121_key(path)),
        Scheme::Ver413 => ver413::decrypt(data.get(HEADER_LEN..).ok_or(Error::Truncated)?)?,
        Scheme::CafeF00d => cafef00d::decrypt(data)?,
    }))
}

/// Decrypts a client file in place, without a second copy of it. `Ver413` files are decrypted whole.
pub fn decrypt_owned(mut data: Vec<u8>, path: &Path) -> Result<Vec<u8>, Error> {
    let key = match detect(&data)? {
        Scheme::Plain => return Ok(data),
        Scheme::Ver111 => 0xAC,
        Scheme::Ver121 => ver121_key(path),
        Scheme::Ver413 | Scheme::CafeF00d => return decrypt(&data, path).map(Cow::into_owned),
    };
    let payload = xor_payload(&data)?.len();
    data.truncate(HEADER_LEN + payload);
    data.drain(..HEADER_LEN);
    for byte in &mut data {
        *byte ^= key;
    }
    Ok(data)
}

/// A byte-wise cipher, which lets any range of the payload be decrypted without the rest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    header: usize,
    key: u8,
}

impl Window {
    pub fn payload_len(self, input: &mut (impl Read + Seek)) -> io::Result<u64> {
        let size = input.seek(SeekFrom::End(0))?;
        let body = size.checked_sub(self.header as u64).ok_or(io::ErrorKind::UnexpectedEof)?;
        if self.header == 0 || body < TRAILER_LEN as u64 {
            return Ok(body);
        }
        let signed = size - TRAILER_LEN as u64;
        input.seek(SeekFrom::Start(signed + 12))?;
        let mut crc = [0; 4];
        input.read_exact(&mut crc)?;
        input.seek(SeekFrom::Start(0))?;
        let mut hasher = flate2::Crc::new();
        let mut buffer = [0; 65536];
        let mut remaining = signed;
        while remaining > 0 {
            let count = remaining.min(buffer.len() as u64) as usize;
            input.read_exact(&mut buffer[..count])?;
            hasher.update(&buffer[..count]);
            remaining -= count as u64;
        }
        Ok(if hasher.sum() == u32::from_le_bytes(crc) { body - TRAILER_LEN as u64 } else { body })
    }

    /// The cipher of a file from its header, `None` for `Ver413`, which must be decrypted whole.
    pub fn of(data: &[u8], path: &Path) -> Result<Option<Self>, Error> {
        Ok(match detect(data)? {
            Scheme::Plain => Some(Self { header: 0, key: 0 }),
            Scheme::Ver111 => Some(Self { header: HEADER_LEN, key: 0xAC }),
            Scheme::Ver121 => Some(Self { header: HEADER_LEN, key: ver121_key(path) }),
            Scheme::Ver413 | Scheme::CafeF00d => None,
        })
    }

    /// Where byte `offset` of the decrypted payload sits in the file.
    pub fn file_offset(self, offset: u64) -> u64 {
        self.header as u64 + offset
    }

    /// Decrypts bytes read from the file at any payload offset.
    pub fn decrypt_in_place(self, bytes: &mut [u8]) {
        for byte in bytes {
            *byte ^= self.key;
        }
    }
}

/// Official XOR files end with a CRC-signed trailer; some custom encoders omit it,
/// so it is only stripped when its checksum matches.
fn xor_payload(data: &[u8]) -> Result<&[u8], Error> {
    let body = data.get(HEADER_LEN..).ok_or(Error::Truncated)?;
    let Some((signed, trailer)) = data.split_last_chunk::<TRAILER_LEN>() else {
        return Ok(body);
    };
    let [.., c0, c1, c2, c3, _, _, _, _] = *trailer;
    let crc = u32::from_le_bytes([c0, c1, c2, c3]);
    let mut hasher = flate2::Crc::new();
    hasher.update(signed);
    match signed.get(HEADER_LEN..) {
        Some(payload) if hasher.sum() == crc => Ok(payload),
        _ => Ok(body),
    }
}

fn header_version(data: &[u8]) -> Option<u16> {
    let units = data.get(..HEADER_LEN)?.as_chunks::<2>().0.iter().map(|&unit| u16::from_le_bytes(unit));
    let text: String = char::decode_utf16(units).collect::<Result<_, _>>().ok()?;
    text.strip_prefix(MAGIC)?.parse().ok()
}

/// Low byte of the sum of the lower-cased file name's UTF-16 units.
fn ver121_key(path: &Path) -> u8 {
    let name = path.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
    let [low, ..] = name.encode_utf16().fold(0u32, |sum, unit| sum.wrapping_add(unit.into())).to_le_bytes();
    low
}

fn xor(payload: &[u8], key: u8) -> Vec<u8> {
    payload.iter().map(|b| b ^ key).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(version: u16, key: u8, plain: &[u8], with_trailer: bool) -> Vec<u8> {
        let mut file: Vec<u8> = format!("{MAGIC}{version}").encode_utf16().flat_map(u16::to_le_bytes).collect();
        file.extend(xor(plain, key));
        if with_trailer {
            let mut hasher = flate2::Crc::new();
            hasher.update(&file);
            file.extend([0; 12]);
            file.extend(hasher.sum().to_le_bytes());
            file.extend([0; 4]);
        }
        file
    }

    #[test]
    fn ver121_keys_match_real_client_files() {
        assert_eq!(ver121_key(Path::new("SysTextures/BaseballTex.utx")), 0x16);
        assert_eq!(ver121_key(Path::new("ui.ugx")), 0x60);
    }

    #[test]
    fn xor_schemes_round_trip() {
        let plain = b"\xC1\x83\x2A\x9E package with enough bytes to span a trailer";
        let path = Path::new("BaseballTex.utx");
        for with_trailer in [true, false] {
            assert_eq!(*decrypt(&encode(111, 0xAC, plain, with_trailer), path).unwrap(), *plain);
            assert_eq!(*decrypt(&encode(121, 0x16, plain, with_trailer), path).unwrap(), *plain);
        }
    }

    #[test]
    fn decrypt_owned_matches_decrypt_for_every_scheme() {
        let plain = b"\xC1\x83\x2A\x9E package with enough bytes to span a trailer";
        let path = Path::new("BaseballTex.utx");
        for with_trailer in [true, false] {
            for (version, key) in [(111, 0xAC), (121, 0x16)] {
                let file = encode(version, key, plain, with_trailer);
                assert_eq!(decrypt_owned(file.clone(), path).unwrap(), *decrypt(&file, path).unwrap());
            }
        }
        assert_eq!(decrypt_owned(b"OggS".to_vec(), path).unwrap(), b"OggS");
    }

    #[test]
    fn window_decrypts_any_slice_of_the_payload_on_its_own() {
        let plain = b"\xC1\x83\x2A\x9E package with enough bytes to span a trailer";
        let path = Path::new("BaseballTex.utx");
        for (version, key) in [(111, 0xAC), (121, 0x16)] {
            let file = encode(version, key, plain, true);
            let window = Window::of(&file, path).unwrap().unwrap();
            let start = window.file_offset(10) as usize;
            let mut slice = file[start..start + 7].to_vec();
            window.decrypt_in_place(&mut slice);
            assert_eq!(slice, plain[10..17]);
        }
        let window = Window::of(b"OggS", path).unwrap().unwrap();
        assert_eq!(window.file_offset(3), 3);
        let mut slice = *b"gS";
        window.decrypt_in_place(&mut slice);
        assert_eq!(&slice, b"gS");
    }

    #[test]
    fn streamed_payload_bounds_match_full_decryption_with_valid_and_invalid_trailers() {
        let path = Path::new("BaseballTex.utx");
        for plain in [b"short".as_slice(), b"package with enough bytes to span the optional trailer".as_slice()] {
            for (version, key) in [(111, 0xAC), (121, 0x16)] {
                for trailer in [false, true] {
                    let mut file = encode(version, key, plain, trailer);
                    for corrupt in [false, true] {
                        if corrupt {
                            let index = file.len() - if trailer { 8 } else { 1 };
                            file[index] ^= 1;
                        }
                        let window = Window::of(&file, path).unwrap().unwrap();
                        let size = window.payload_len(&mut io::Cursor::new(&file)).unwrap();
                        assert_eq!(size as usize, decrypt(&file, path).unwrap().len());
                    }
                }
            }
        }
        let bytes = b"plain";
        let window = Window::of(bytes, path).unwrap().unwrap();
        assert_eq!(window.payload_len(&mut io::Cursor::new(bytes)).unwrap(), bytes.len() as u64);
    }

    #[test]
    fn headerless_files_are_borrowed() {
        assert!(matches!(decrypt(b"OggS", Path::new("a.ogg")).unwrap(), Cow::Borrowed(_)));
        assert!(matches!(detect(&encode(212, 0, b"", true)), Err(Error::UnsupportedVersion(212))));
    }
}
