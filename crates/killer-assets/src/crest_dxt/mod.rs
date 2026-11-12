#[allow(clippy::all, unused_mut, unused_assignments, unreachable_code, unused_variables, dead_code, unused_parens, unused_comparisons)]
mod lifted;
mod runtime;

use runtime::{CALLBACK, Cpu, Fault};

pub use runtime::Fault as DxtFault;

const OPTIONS_SIZE: usize = 0x38;
const FORMAT_DXT1: i16 = 10;
const FORMAT_DXT1_ALPHA: i16 = 11;
const QUALITY: u16 = 0x50;
const MIP_FILTER: u16 = 0x86;
const TEXTURE_FORMAT: u16 = 0x21;

pub fn options(transparent: bool) -> [u8; OPTIONS_SIZE] {
    let mut table = [0u8; OPTIONS_SIZE];
    table[6..8].copy_from_slice(&TEXTURE_FORMAT.to_le_bytes());
    table[0xA..0xC].copy_from_slice(&MIP_FILTER.to_le_bytes());
    table[0xC] = u8::from(transparent);
    table[0x14..0x18].copy_from_slice(&u32::MAX.to_le_bytes());
    table[0x1C..0x20].copy_from_slice(&u32::MAX.to_le_bytes());
    table[0x30..0x32].copy_from_slice(&QUALITY.to_le_bytes());
    let format = if transparent { FORMAT_DXT1_ALPHA } else { FORMAT_DXT1 };
    table[0x32..0x34].copy_from_slice(&format.to_le_bytes());
    table
}

pub fn compress(bgra: &[u8], width: u32, height: u32, transparent: bool) -> Result<Vec<u8>, Fault> {
    let mut cpu = Cpu::new();
    let pixels = cpu.alloc_bytes(bgra);
    let table = cpu.alloc_bytes(&options(transparent));
    for value in [CALLBACK, 4, table, width * 4, height, width, pixels, 0] {
        cpu.push(value)?;
    }
    lifted::dispatch(&mut cpu, 0x1072c4f0)?;
    if cpu.r[0] != 0 {
        return Err(Fault("status", cpu.r[0]));
    }
    cpu.output.pop().ok_or(Fault("output", 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lifted_compressor_reproduces_the_oracle_on_a_gradient() {
        let mut bgra = Vec::new();
        for y in 0..16u8 {
            for x in 0..16u8 {
                bgra.extend_from_slice(&[x * 15, y * 15, 100, 255]);
            }
        }
        let data = compress(&bgra, 16, 16, false).unwrap();
        assert_eq!(&data[..16], &[0x63, 0x61, 0x03, 0x60, 0x55, 0xff, 0xaa, 0x00, 0x6a, 0x61, 0x0a, 0x60, 0x55, 0xff, 0xaa, 0x00]);
        assert_eq!(data.len(), 128);
    }
}
