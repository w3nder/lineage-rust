//! Precomputed lighting stored with a level: vertex colors of each static mesh actor and intensity
//! maps of each terrain sector.

use ue2_assets::{Error, object_data};
use ue2_core::Reader;
use ue2_package::{Export, ObjectRef, Package};

pub const SECTOR_MAP_SIDE: usize = 17;
pub const SECTOR_MAP_LEN: usize = SECTOR_MAP_SIDE * SECTOR_MAP_SIDE;
pub const LAYER_ALL_UNKNOWN: u16 = 0xffff;
pub const LAYER_ALL_MIXED: u16 = 0xfffe;
const LAYER_CACHE_LEN: usize = 32;
const LICENSEE_INT_MAP: u16 = 4;
const LICENSEE_LAYER_CACHE: u16 = 8;
const LICENSEE_PASS_MASK: u16 = 10;
const VERSION_BOX: u16 = 0x75;

/// A terrain sector: the patch of heightmap quads it covers and its intensity maps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainSector {
    /// First quad on X and Y.
    pub offset: [u32; 2],
    /// Quads on X and Y; the sector has one more vertex than quads on each axis.
    pub quads: [u32; 2],
    /// One 17×17 map per time-of-day slice, row by row; column and row 16 are copies when the
    /// sector has 15 quads on that axis.
    pub intensities: Vec<Vec<u8>>,
    /// Per layer: 0 when the layer's alpha is 0 all over the sector, 0xff when 255 all over,
    /// `LAYER_ALL_MIXED` otherwise, `LAYER_ALL_UNKNOWN` when the editor did not compute it.
    pub layer_all: [u16; LAYER_CACHE_LEN],
    /// Per quad (`y * 16 + x`), bit `p` set when the quad is drawn in render pass `p`.
    pub pass_mask: Vec<u16>,
    /// Static lights reaching the sector, with a one-bit-per-vertex visibility bitmap each.
    pub lights: Vec<(String, Vec<u8>)>,
}

/// A `StaticMeshInstance`'s lighting: one color per vertex of the actor's mesh.
pub(crate) fn vertex_colors(package: &Package, file: &[u8], instance: &Export) -> Result<Vec<[u8; 4]>, Error> {
    let (_, data) = object_data(package, file, instance)?;
    let mut reader = Reader::at(data, 0);
    let count = usize::try_from(reader.compact()?).unwrap_or(0);
    // Serialized as red, green, blue, alpha, unlike Color struct properties.
    let colors = reader.bytes(count.checked_mul(4).ok_or(Error::BadMesh)?)?;
    Ok(colors.as_chunks::<4>().0.to_vec())
}

/// A `TerrainSector` as `UTerrainSector::Serialize` writes it, or `None` when the bytes do not fit.
pub(crate) fn sector(package: &Package, file: &[u8], export: &Export) -> Option<TerrainSector> {
    let (_, data) = object_data(package, file, export).ok()?;
    let mut reader = Reader::at(data, 0);
    let reader = &mut reader;
    reader.compact().ok()?;
    let quads = [reader.u32().ok()?, reader.u32().ok()?];
    let offset = [reader.u32().ok()?, reader.u32().ok()?];
    if package.version >= VERSION_BOX {
        reader.bytes(25).ok()?;
    } else {
        reader.bytes(8 * 12).ok()?;
    }
    let light_count = usize::try_from(reader.compact().ok()?).ok()?;
    let mut lights = Vec::with_capacity(light_count);
    for _ in 0..light_count {
        let actor = package.object_at(reader.compact().ok()?).ok()?;
        let len = usize::try_from(reader.compact().ok()?).ok()?;
        let bits = reader.bytes(len).ok()?.to_vec();
        if !matches!(actor, ObjectRef::Null) {
            lights.push((package.object_name(actor).to_owned(), bits));
        }
    }
    let mut intensities = Vec::new();
    if package.licensee > LICENSEE_INT_MAP && reader.u32().ok()? != 0 {
        let count = usize::try_from(reader.u32().ok()?).ok()?;
        for _ in 0..count {
            let len = usize::try_from(reader.compact().ok()?).ok()?;
            intensities.push(reader.bytes(len).ok()?.to_vec());
        }
    }
    let mut layer_all = [LAYER_ALL_UNKNOWN; LAYER_CACHE_LEN];
    if package.licensee >= LICENSEE_LAYER_CACHE {
        for cache in &mut layer_all {
            *cache = reader.u16().ok()?;
        }
    }
    let mut pass_mask = Vec::new();
    if package.licensee >= LICENSEE_PASS_MASK {
        let count = usize::try_from(reader.compact().ok()?).ok()?;
        for _ in 0..count {
            pass_mask.push(reader.u16().ok()?);
        }
    }
    if !reader.remaining().is_empty() {
        return None;
    }
    Some(TerrainSector { offset, quads, intensities, layer_all, pass_mask, lights })
}

impl TerrainSector {
    /// Vertices the sector draws on each axis.
    pub fn vertices(&self) -> [usize; 2] {
        [self.quads[0] as usize + 1, self.quads[1] as usize + 1]
    }

    /// Intensity of the local vertex `(x, y)` in slice `slice`, from the 17×17 map.
    pub fn intensity(&self, slice: usize, x: usize, y: usize) -> Option<u8> {
        self.intensities.get(slice)?.get(y * SECTOR_MAP_SIDE + x).copied()
    }

    /// Whether the quad `(x, y)` of the sector is drawn in render pass `pass` (§3a.4.5).
    pub fn in_pass(&self, pass: usize, x: usize, y: usize) -> bool {
        if pass == 0 {
            return true;
        }
        self.pass_mask.get(y * 16 + x).is_some_and(|mask| mask & (1 << pass) != 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sector_bytes(info: &[u8], quads: [u32; 2], offset: [u32; 2]) -> Vec<u8> {
        let mut data = vec![0];
        data.extend_from_slice(info);
        for value in [quads[0], quads[1], offset[0], offset[1]] {
            data.extend(value.to_le_bytes());
        }
        data.extend([0; 25]);
        data.push(1);
        data.extend([1, 2, 0xAA, 0x55]);
        data.extend(1u32.to_le_bytes());
        data.extend(8u32.to_le_bytes());
        for slice in 0..8u8 {
            data.extend([0x61, 0x04]);
            data.extend(std::iter::repeat_n(slice * 10, SECTOR_MAP_LEN));
        }
        for cache in 0..32u16 {
            data.extend(cache.to_le_bytes());
        }
        data.extend([0x40, 0x04]);
        for quad in 0..256u16 {
            data.extend(quad.to_le_bytes());
        }
        data
    }

    fn package(object: &[u8]) -> Vec<u8> {
        let mut file = Vec::new();
        for value in [0x9E2A_83C1, 123 | (25 << 16), 0, 2, 36, 1, 0, 0, 0] {
            file.extend(u32::to_le_bytes(value));
        }
        file.extend([5, b'N', b'o', b'n', b'e', 0, 0, 0, 0, 0]);
        file.extend([4, b'S', b'u', b'n', 0, 0, 0, 0, 0]);
        let export_offset = u32::try_from(file.len()).unwrap();
        let (offset, size) = (file.len() + 15, object.len());
        file[24..28].copy_from_slice(&export_offset.to_le_bytes());
        file.extend([0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0]);
        for value in [size, offset] {
            file.push(u8::try_from(value & 0x3F).unwrap() | 0x40);
            file.push(u8::try_from(value >> 6).unwrap());
        }
        file.extend(object);
        file
    }

    #[test]
    fn a_sector_with_a_two_byte_info_reference_reads_to_its_end() {
        let file = package(&sector_bytes(&[0x64, 0x01], [15, 16], [240, 32]));
        let package = Package::parse(&file).unwrap();
        let sector = sector(&package, &file, &package.exports()[0]).unwrap();
        assert_eq!((sector.quads, sector.offset), ([15, 16], [240, 32]));
        assert_eq!(sector.vertices(), [16, 17]);
        assert_eq!(sector.lights, vec![("Sun".to_owned(), vec![0xAA, 0x55])]);
        assert_eq!(sector.intensities.len(), 8);
        assert!(sector.intensities.iter().all(|map| map.len() == SECTOR_MAP_LEN));
        assert_eq!(sector.intensity(3, 16, 16), Some(30));
        assert_eq!(sector.layer_all[5], 5);
        assert_eq!(sector.pass_mask.len(), 256);
        assert!(sector.in_pass(0, 0, 0));
        assert!(sector.in_pass(1, 2, 0));
        assert!(!sector.in_pass(1, 1, 0));
        assert!(sector.in_pass(3, 8, 0));
    }

    #[test]
    fn a_sector_with_bytes_left_over_is_rejected() {
        let mut bytes = sector_bytes(&[7], [16, 16], [0, 0]);
        bytes.push(0);
        let file = package(&bytes);
        let package = Package::parse(&file).unwrap();
        assert!(sector(&package, &file, &package.exports()[0]).is_none());
    }
}
