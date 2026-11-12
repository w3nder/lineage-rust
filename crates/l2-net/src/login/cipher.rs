use blowfish::BlowfishLE;
use cipher::{BlockDecrypt, BlockEncrypt, KeyInit};
use thiserror::Error;

const C4_KEY: [u8; 21] = [
    0x5f, 0x3b, 0x35, 0x2e, 0x5d, 0x39, 0x34, 0x2d, 0x33, 0x31, 0x3d, 0x3d, 0x2d, 0x25, 0x78, 0x54, 0x21, 0x5e, 0x5b,
    0x24, 0x00,
];

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CipherError {
    #[error("tamanho não é múltiplo de 8")]
    Length,
    #[error("checksum inválido")]
    Checksum,
}

pub struct LoginCipher {
    blowfish: BlowfishLE,
}

impl LoginCipher {
    pub fn c4() -> Self {
        let blowfish = BlowfishLE::new_from_slice(&C4_KEY).expect("chave de 21 bytes");
        Self { blowfish }
    }

    pub fn seal(&self, payload: &[u8]) -> Vec<u8> {
        let aligned = (payload.len() + 7) & !7;
        let mut out = vec![0u8; aligned + 8];
        out[..payload.len()].copy_from_slice(payload);
        let checksum = xor_words(&out[..aligned]);
        out[aligned..aligned + 4].copy_from_slice(&checksum.to_le_bytes());
        for block in out.as_chunks_mut::<8>().0 {
            self.blowfish.encrypt_block(block.into());
        }
        out
    }

    pub fn open(&self, sealed: &[u8]) -> Result<Vec<u8>, CipherError> {
        if sealed.is_empty() || !sealed.len().is_multiple_of(8) {
            return Err(CipherError::Length);
        }
        let mut out = sealed.to_vec();
        for block in out.as_chunks_mut::<8>().0 {
            self.blowfish.decrypt_block(block.into());
        }
        if xor_words(&out[..out.len() - 4]) != 0 {
            return Err(CipherError::Checksum);
        }
        out.truncate(out.len() - 8);
        Ok(out)
    }
}

fn xor_words(bytes: &[u8]) -> u32 {
    bytes.as_chunks::<4>().0.iter().map(|w| u32::from_le_bytes(*w)).fold(0, |acc, w| acc ^ w)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_rejects_bad_checksum() {
        let cipher = LoginCipher::c4();
        let mut sealed = cipher.seal(&[1, 2, 3]);
        sealed[0] ^= 1;
        assert!(matches!(cipher.open(&sealed), Err(CipherError::Checksum)));
    }

    #[test]
    fn open_rejects_unaligned_length() {
        let cipher = LoginCipher::c4();
        assert!(matches!(cipher.open(&[0u8; 12]), Err(CipherError::Length)));
    }
}
