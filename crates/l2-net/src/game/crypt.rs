use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
#[error("KeyPacket inválido")]
pub struct KeyError;

pub struct GameCrypt {
    tx: [u8; 8],
    rx: [u8; 8],
}

impl GameCrypt {
    pub fn from_key_packet(body: &[u8]) -> Result<Self, KeyError> {
        if body.len() != 14 || body[0] != 0 || body[1] != 1 || body[6..10] != [1, 0, 0, 0] {
            return Err(KeyError);
        }
        let seed = u32::from_le_bytes([body[2], body[3], body[4], body[5]]);
        let tail: u32 = if seed & 0x8000_0000 != 0 { 0xffff_ffff } else { 0x8754_6ca1 };
        let mut key = [0u8; 8];
        key[..4].copy_from_slice(&seed.to_le_bytes());
        key[4..].copy_from_slice(&tail.to_le_bytes());
        Ok(Self { tx: key, rx: key })
    }

    pub fn encrypt(&mut self, data: &mut [u8]) {
        let mut previous = 0u8;
        for (i, byte) in data.iter_mut().enumerate() {
            *byte ^= self.tx[i & 7] ^ previous;
            previous = *byte;
        }
        advance(&mut self.tx, data.len());
    }

    pub fn decrypt(&mut self, data: &mut [u8]) {
        let mut previous = 0u8;
        for (i, byte) in data.iter_mut().enumerate() {
            let cipher = *byte;
            *byte ^= self.rx[i & 7] ^ previous;
            previous = cipher;
        }
        advance(&mut self.rx, data.len());
    }
}

fn advance(key: &mut [u8; 8], len: usize) {
    let value = u64::from_le_bytes(*key).wrapping_add(len as u64);
    *key = value.to_le_bytes();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_packet_shape_is_checked() {
        assert!(GameCrypt::from_key_packet(&[0; 13]).is_err());
        assert!(GameCrypt::from_key_packet(&[0, 0, 1, 2, 3, 4, 1, 0, 0, 0, 0, 0, 0, 0]).is_err());
        assert!(GameCrypt::from_key_packet(&[0, 1, 1, 2, 3, 4, 1, 0, 0, 0, 0, 0, 0, 0]).is_ok());
    }

    #[test]
    fn negative_seed_uses_all_ones_tail() {
        let c = GameCrypt::from_key_packet(&[0, 1, 0, 0, 0, 0x80, 1, 0, 0, 0, 0, 0, 0, 0]).unwrap();
        assert_eq!(c.tx, [0, 0, 0, 0x80, 0xff, 0xff, 0xff, 0xff]);
        let c = GameCrypt::from_key_packet(&[0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0]).unwrap();
        assert_eq!(c.tx, [1, 0, 0, 0, 0xa1, 0x6c, 0x54, 0x87]);
    }
}
