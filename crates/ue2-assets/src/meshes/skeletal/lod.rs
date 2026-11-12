//! A skeletal mesh's level of detail, turned into one skinned vertex list.
//!
//! Most parts keep their vertices in two streams: soft wedges weighted to up to four bones through their
//! section's bone map, and rigid vertices bound whole to their section's bone. Some keep the older layout
//! instead: shared points, wedges that give each point a UV, faces, and a list of bone influences per point.

use ue2_core::Reader;

use super::{Bone, Layout, SkeletalLodInfo, SkinVertex};
use crate::Error;
use crate::meshes::{Section, array, vector};

const FALLBACK_RIGID_BONE: &str = "Bip01_head";
const STREAM_END: u32 = u32::MAX;
const REUSED_POINT: u32 = 0xF;
const INDEX_STRIDE: u32 = 6;
const WEIGHT_SCALE: f32 = 65535.0;
const NORMAL_SCALE: f32 = 511.0;

pub(super) struct Lod {
    pub(super) vertices: Vec<SkinVertex>,
    pub(super) indices: Vec<u16>,
    /// Each section with its index into the mesh's material slots.
    pub(super) sections: Vec<(u16, Section)>,
}

struct RawSection {
    material: u16,
    bone: u16,
    first_face: u16,
    faces: u16,
    bone_map: Vec<i32>,
}

type SoftWedge = ([f32; 3], [f32; 3], [f32; 2], [u8; 4], [f32; 4]);
type RigidVertex = ([f32; 3], [f32; 3], [f32; 2]);
type Influences = ([u16; 4], [f32; 4]);

pub(super) fn read(reader: &mut Reader<'_>, bones: &[Bone], layout: Layout) -> Result<(Lod, SkeletalLodInfo), Error> {
    let skinning = array(reader, |reader| Ok(reader.u32()?))?; // skinning data
    let skin_points = array(reader, |reader| Ok((vector(reader)?, reader.u32()?)))?; // skin points
    let soft_wedges = reader.i32()?; // soft wedge count
    let soft_sections = array(reader, |reader| raw_section(reader, layout))?;
    let rigid_sections = array(reader, |reader| raw_section(reader, layout))?;
    let soft_indices = index_buffer(reader)?;
    let rigid_indices = index_buffer(reader)?;
    reader.bytes(12)?; // stream revision and two counters
    let rigid = array(reader, |reader| Ok((vector(reader)?, vector(reader)?, uv(reader)?)))?;
    let influences = lazy(reader, |reader| Ok((reader.f32()?, reader.u16()?, reader.u16()?)))?;
    let wedges = lazy(reader, |reader| Ok((reader.u16()?, uv(reader)?)))?;
    let faces = lazy(reader, |reader| Ok(([reader.u16()?, reader.u16()?, reader.u16()?], reader.u16()?)))?;
    let points = lazy(reader, vector)?;
    let info = SkeletalLodInfo { soft_wedges, distance_factor: reader.f32()?, hysteresis: reader.f32()? };
    reader.bytes(4 * 4)?;
    let soft = match layout {
        Layout::HighFive | Layout::Interlude => {
            reader.i32()?; // uses new wedges
            array(reader, |reader| {
                let (position, normal, uv) = (vector(reader)?, vector(reader)?, uv(reader)?);
                let slots: [u8; 4] = reader.array()?;
                Ok((position, normal, uv, slots, [reader.f32()?, reader.f32()?, reader.f32()?, reader.f32()?]))
            })?
        }
        Layout::Chronicle4 | Layout::Unlicensed => Vec::new(),
    };

    let lod = if !matches!(layout, Layout::HighFive | Layout::Interlude) && !skin_points.is_empty() {
        let mut lod = packed_stream(&skinning, &skin_points, &soft_sections, &soft_indices)?;
        if usize::try_from(soft_wedges).ok() != Some(lod.vertices.len()) {
            return Err(Error::BadMesh);
        }
        let weights = rigid_weights(bones, rigid.len(), &wedges, &influences)?;
        rigid_stream(&mut lod, &rigid_sections, &rigid_indices, rigid, Some(weights))?;
        lod
    } else if soft.is_empty() && rigid.is_empty() {
        from_wedges(&points, &wedges, &faces, &influences)?
    } else {
        let mut lod = Lod { vertices: Vec::new(), indices: Vec::new(), sections: Vec::new() };
        soft_stream(&mut lod, &soft_sections, &soft_indices, soft)?;
        let weights = match layout {
            Layout::HighFive | Layout::Interlude => None,
            Layout::Chronicle4 | Layout::Unlicensed => Some(rigid_weights(bones, rigid.len(), &wedges, &influences)?),
        };
        rigid_stream(&mut lod, &rigid_sections, &rigid_indices, rigid, weights)?;
        lod
    };
    let bones_fit = lod.vertices.iter().all(|vertex| vertex.bones.iter().all(|&bone| usize::from(bone) < bones.len()));
    let indices_fit = lod.indices.iter().all(|&index| usize::from(index) < lod.vertices.len());
    if bones_fit && indices_fit { Ok((lod, info)) } else { Err(Error::BadMesh) }
}

fn packed_stream(
    skinning: &[u32],
    skin_points: &[([f32; 3], u32)],
    sections: &[RawSection],
    indices: &[u16],
) -> Result<Lod, Error> {
    let mut lod = Lod { vertices: Vec::new(), indices: Vec::new(), sections: Vec::new() };
    for section in sections {
        push_section(&mut lod, section.material, faces_of(indices, section)?, 0)?;
    }
    let mut words = skinning.iter().copied();
    let mut shared = Vec::new();
    let mut influences: Vec<Influences> = Vec::new();
    loop {
        let word = words.next().ok_or(Error::BadMesh)?;
        if word == STREAM_END {
            break;
        }
        let point = if word >> 28 == REUSED_POINT {
            *shared.get(packed_index(word & 0x0FFF_FFFF)?).ok_or(Error::BadMesh)?
        } else {
            let point = influences.len();
            if word >> 31 != 0 {
                shared.push(point);
            }
            influences.push(packed_influences(word, &mut words)?);
            point
        };
        let uv = [next_f32(&mut words)?, next_f32(&mut words)?];
        let (&(position, normal), &(bones, weights)) =
            skin_points.get(point).zip(influences.get(point)).ok_or(Error::BadMesh)?;
        lod.vertices.push(SkinVertex { position, normal: unpack_normal(normal), uv, bones, weights });
    }
    if influences.len() != skin_points.len() || words.next().is_some() {
        return Err(Error::BadMesh);
    }
    Ok(lod)
}

fn packed_influences(first: u32, words: &mut impl Iterator<Item = u32>) -> Result<Influences, Error> {
    let extra = usize::try_from((first >> 28) & 7).map_err(|_| Error::BadMesh)?;
    let (mut bones, mut weights) = ([0; 4], [0.0; 4]);
    if extra >= bones.len() {
        return Err(Error::BadMesh);
    }
    for slot in 0..=extra {
        let word = if slot == 0 { first } else { words.next().ok_or(Error::BadMesh)? };
        bones[slot] = u16::try_from(packed_index(word & 0xFFF)?).map_err(|_| Error::BadMesh)?;
        weights[slot] = ((word >> 12) & 0xFFFF) as f32 / WEIGHT_SCALE;
    }
    Ok((bones, weights))
}

fn packed_index(scaled: u32) -> Result<usize, Error> {
    if !scaled.is_multiple_of(INDEX_STRIDE) {
        return Err(Error::BadMesh);
    }
    usize::try_from(scaled / INDEX_STRIDE).map_err(|_| Error::BadMesh)
}

fn next_f32(words: &mut impl Iterator<Item = u32>) -> Result<f32, Error> {
    words.next().map(f32::from_bits).ok_or(Error::BadMesh)
}

fn unpack_normal(packed: u32) -> [f32; 3] {
    [packed, packed >> 10, packed >> 20].map(|axis| ((axis & 0x3FF) as f32 - 512.0) / NORMAL_SCALE)
}

fn rigid_weights(
    bones: &[Bone],
    vertices: usize,
    wedges: &[(u16, [f32; 2])],
    influences: &[(f32, u16, u16)],
) -> Result<Vec<Influences>, Error> {
    if vertices == 0 {
        return Ok(Vec::new());
    }
    if wedges.is_empty() && influences.is_empty() {
        return fallback_rigid_weights(bones, vertices);
    }
    if wedges.len() != vertices {
        return Err(Error::BadMesh);
    }
    let points = wedges.iter().map(|&(point, _)| point).max().map_or(0, |point| usize::from(point) + 1);
    let weights = heaviest_influences(points, influences)?;
    wedges
        .iter()
        .map(|&(point, _)| weights.get(usize::from(point)).copied().ok_or(Error::BadMesh))
        .map(|slots| slots.map(|slots| (slots.map(|slot| slot.0), slots.map(|slot| slot.1))))
        .collect()
}

fn fallback_rigid_weights(bones: &[Bone], vertices: usize) -> Result<Vec<Influences>, Error> {
    let bone = match bones {
        [_] => 0,
        _ => bones
            .iter()
            .position(|bone| bone.name.replace(' ', "_").eq_ignore_ascii_case(FALLBACK_RIGID_BONE))
            .ok_or(Error::BadMesh)?,
    };
    let bone = u16::try_from(bone).map_err(|_| Error::BadMesh)?;
    Ok(vec![([bone, 0, 0, 0], [1.0, 0.0, 0.0, 0.0]); vertices])
}

fn soft_stream(lod: &mut Lod, sections: &[RawSection], indices: &[u16], soft: Vec<SoftWedge>) -> Result<(), Error> {
    // A wedge takes its bones from the section whose faces use it.
    let mut maps = vec![None; soft.len()];
    for section in sections {
        let faces = faces_of(indices, section)?;
        for &index in faces {
            *maps.get_mut(usize::from(index)).ok_or(Error::BadMesh)? = Some(&section.bone_map);
        }
        push_section(lod, section.material, faces, 0)?;
    }
    for ((position, normal, uv, slots, weights), map) in soft.into_iter().zip(maps) {
        let mut vertex = SkinVertex { position, normal, uv, bones: [0; 4], weights: [0.0; 4] };
        let targets = vertex.bones.iter_mut().zip(vertex.weights.iter_mut());
        for ((&local, &weight), (bone_slot, weight_slot)) in slots.iter().zip(&weights).zip(targets) {
            if local == 255 || weight <= 0.0 {
                continue;
            }
            let bone = map.and_then(|map| map.get(usize::from(local))).and_then(|&bone| u16::try_from(bone).ok());
            (*bone_slot, *weight_slot) = (bone.ok_or(Error::BadMesh)?, weight);
        }
        lod.vertices.push(vertex);
    }
    Ok(())
}

fn rigid_stream(
    lod: &mut Lod,
    sections: &[RawSection],
    indices: &[u16],
    rigid: Vec<RigidVertex>,
    weights: Option<Vec<Influences>>,
) -> Result<(), Error> {
    let offset = u16::try_from(lod.vertices.len()).map_err(|_| Error::BadMesh)?;
    let mut bones = vec![0; rigid.len()];
    for section in sections {
        let faces = faces_of(indices, section)?;
        for &index in faces {
            *bones.get_mut(usize::from(index)).ok_or(Error::BadMesh)? = section.bone;
        }
        push_section(lod, section.material, faces, offset)?;
    }
    let weights =
        weights.unwrap_or_else(|| bones.iter().map(|&bone| ([bone, 0, 0, 0], [1.0, 0.0, 0.0, 0.0])).collect());
    if weights.len() != rigid.len() {
        return Err(Error::BadMesh);
    }
    for ((position, normal, uv), (bones, weights)) in rigid.into_iter().zip(weights) {
        lod.vertices.push(SkinVertex { position, normal, uv, bones, weights });
    }
    Ok(())
}

/// The older layout: a vertex per wedge, weighted by its point's four heaviest influences, with sections
/// made of consecutive faces that share a material. It carries no normals.
fn from_wedges(
    points: &[[f32; 3]],
    wedges: &[(u16, [f32; 2])],
    faces: &[([u16; 3], u16)],
    influences: &[(f32, u16, u16)],
) -> Result<Lod, Error> {
    let weights = heaviest_influences(points.len(), influences)?;
    let mut lod = Lod { vertices: Vec::with_capacity(wedges.len()), indices: Vec::new(), sections: Vec::new() };
    for &(point, uv) in wedges {
        let point = usize::from(point);
        let (&position, &slots) = points.get(point).zip(weights.get(point)).ok_or(Error::BadMesh)?;
        let (bones, weights) = (slots.map(|slot| slot.0), slots.map(|slot| slot.1));
        lod.vertices.push(SkinVertex { position, normal: [0.0; 3], uv, bones, weights });
    }
    for run in faces.chunk_by(|a, b| a.1 == b.1) {
        let indices: Vec<u16> = run.iter().flat_map(|face| face.0).collect();
        push_section(&mut lod, run.first().map_or(0, |face| face.1), &indices, 0)?;
    }
    Ok(lod)
}

fn heaviest_influences(points: usize, influences: &[(f32, u16, u16)]) -> Result<Vec<[(u16, f32); 4]>, Error> {
    let mut weights = vec![[(0_u16, 0.0_f32); 4]; points];
    for &(weight, point, bone) in influences {
        let slots = weights.get_mut(usize::from(point)).ok_or(Error::BadMesh)?;
        let lightest = slots.iter_mut().min_by(|a, b| a.1.total_cmp(&b.1)).ok_or(Error::BadMesh)?;
        if weight > lightest.1 {
            *lightest = (bone, weight);
        }
    }
    Ok(weights)
}

fn push_section(lod: &mut Lod, material: u16, indices: &[u16], offset: u16) -> Result<(), Error> {
    let first_index = u32::try_from(lod.indices.len()).map_err(|_| Error::BadMesh)?;
    let triangles = u32::try_from(indices.len() / 3).map_err(|_| Error::BadMesh)?;
    for &index in indices {
        lod.indices.push(index.checked_add(offset).ok_or(Error::BadMesh)?);
    }
    lod.sections.push((material, Section { first_index, primitives: triangles, strip: false }));
    Ok(())
}

fn raw_section(reader: &mut Reader<'_>, layout: Layout) -> Result<RawSection, Error> {
    let material = reader.u16()?;
    reader.bytes(8)?; // stream and wedge index ranges
    let bone = reader.u16()?;
    reader.u16()?;
    let first_face = reader.u16()?;
    let faces = reader.u16()?;
    let bone_map = match layout {
        Layout::HighFive | Layout::Interlude => array(reader, |reader| Ok(reader.i32()?))?,
        Layout::Chronicle4 | Layout::Unlicensed => Vec::new(),
    };
    Ok(RawSection { material, bone, first_face, faces, bone_map })
}

/// The indices of `section`'s faces.
fn faces_of<'a>(indices: &'a [u16], section: &RawSection) -> Result<&'a [u16], Error> {
    let first = usize::from(section.first_face) * 3;
    indices.get(first..first + usize::from(section.faces) * 3).ok_or(Error::BadMesh)
}

fn index_buffer(reader: &mut Reader<'_>) -> Result<Vec<u16>, Error> {
    let indices = array(reader, |reader| Ok(reader.u16()?))?;
    reader.u32()?; // revision
    Ok(indices)
}

/// An array stored after the offset that lets a loader skip it.
fn lazy<'a, T>(
    reader: &mut Reader<'a>,
    item: impl FnMut(&mut Reader<'a>) -> Result<T, Error>,
) -> Result<Vec<T>, Error> {
    reader.u32()?;
    array(reader, item)
}

fn uv(reader: &mut Reader<'_>) -> Result<[f32; 2], Error> {
    Ok([reader.f32()?, reader.f32()?])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_lod_model_preserves_serialized_distance_and_hysteresis_bits() {
        for layout in [Layout::Chronicle4, Layout::Unlicensed, Layout::HighFive, Layout::Interlude] {
            for (distance, hysteresis) in [(0x3f800000u32, 0x3dcccccd_u32), (0x80000000, 0), (0x7fc01234, 0xff800000)] {
                let mut bytes = vec![0; 51];
                bytes.extend(distance.to_le_bytes());
                bytes.extend(hysteresis.to_le_bytes());
                bytes.extend([0; 16]);
                if matches!(layout, Layout::HighFive | Layout::Interlude) { bytes.extend([0; 5]); }
                let mut reader = Reader::at(&bytes, 0);
                let (lod, info) = read(&mut reader, &[], layout).unwrap();
                assert_eq!(info.distance_factor.to_bits(), distance);
                assert_eq!(info.hysteresis.to_bits(), hysteresis);
                assert_eq!(info.soft_wedges, 0);
                assert!(lod.vertices.is_empty() && lod.indices.is_empty() && lod.sections.is_empty());
                assert!(reader.remaining().is_empty());
                for length in 51..bytes.len() {
                    assert!(read(&mut Reader::at(&bytes[..length], 0), &[], layout).is_err());
                }
            }
        }
    }

    #[test]
    #[expect(clippy::float_cmp, reason = "weights are copied, not computed")]
    fn older_layout_keeps_the_four_heaviest_influences_and_splits_sections_by_material() {
        let points = [[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let wedges = [(0, [0.0, 0.0]), (1, [1.0, 0.0]), (2, [0.0, 1.0])];
        let faces = [([0, 1, 2], 0), ([2, 1, 0], 0), ([0, 2, 1], 1)];
        let influences = [(0.1, 0, 1), (0.4, 0, 2), (0.2, 0, 3), (0.3, 0, 4), (0.5, 0, 5), (1.0, 1, 1), (1.0, 2, 2)];

        let lod = from_wedges(&points, &wedges, &faces, &influences).unwrap();

        let mut first: Vec<(u16, f32)> = lod.vertices[0].bones.into_iter().zip(lod.vertices[0].weights).collect();
        first.sort_by_key(|&(bone, _)| bone);
        assert_eq!(first, [(2, 0.4), (3, 0.2), (4, 0.3), (5, 0.5)]);
        assert_eq!(lod.vertices[1].uv, [1.0, 0.0]);
        let sections: Vec<(u16, u32, u32)> = lod
            .sections
            .iter()
            .map(|&(material, section)| (material, section.first_index, section.primitives))
            .collect();
        assert_eq!(sections, [(0, 0, 2), (1, 6, 1)]);
    }

    #[test]
    #[expect(clippy::float_cmp, reason = "weights and coordinates are copied, not computed")]
    fn packed_stream_reuses_shared_points_and_unpacks_influences() {
        let bone = |index: u32| index * 6;
        let weight = |scaled: u32| scaled << 12;
        let words = [
            0x8000_0000 | weight(0xFFFF) | bone(0),
            0.0_f32.to_bits(),
            0.0_f32.to_bits(),
            0x1000_0000 | weight(0x8000) | bone(2),
            weight(0x7FFF) | bone(3),
            1.0_f32.to_bits(),
            0.0_f32.to_bits(),
            0x8000_0000 | weight(0xFFFF) | bone(1),
            0.0_f32.to_bits(),
            1.0_f32.to_bits(),
            0xF000_0000 | 6,
            0.5_f32.to_bits(),
            0.5_f32.to_bits(),
            u32::MAX,
        ];
        let up = (512 + 511) << 20 | 512 << 10 | 512;
        let points = [([0.0; 3], up), ([1.0, 0.0, 0.0], up), ([0.0, 1.0, 0.0], up)];
        let sections = [RawSection { material: 4, bone: 0, first_face: 0, faces: 2, bone_map: Vec::new() }];
        let indices = [0, 1, 2, 2, 1, 3];

        let lod = packed_stream(&words, &points, &sections, &indices).unwrap();

        assert_eq!(lod.vertices.len(), 4);
        assert_eq!(lod.vertices[0].bones, [0, 0, 0, 0]);
        assert_eq!(lod.vertices[0].weights, [1.0, 0.0, 0.0, 0.0]);
        assert_eq!(lod.vertices[0].normal, [0.0, 0.0, 1.0]);
        assert_eq!(lod.vertices[1].bones, [2, 3, 0, 0]);
        assert!((lod.vertices[1].weights[0] - 0.5).abs() < 1e-4 && (lod.vertices[1].weights[1] - 0.5).abs() < 1e-4);
        assert_eq!(lod.vertices[3].position, [0.0, 1.0, 0.0]);
        assert_eq!(lod.vertices[3].uv, [0.5, 0.5]);
        assert_eq!(lod.vertices[3].bones, [1, 0, 0, 0]);
        assert_eq!(lod.indices, [0, 1, 2, 2, 1, 3]);
        assert_eq!(lod.sections.len(), 1);
        assert_eq!(lod.sections[0].0, 4);
    }

    #[test]
    fn interlude_sections_carry_a_bone_map_like_high_five_and_chronicle4_ones_do_not() {
        let mut bytes = vec![7, 0];
        bytes.extend([0; 8]);
        bytes.extend([3, 0, 0, 0, 5, 0, 2, 0]);
        let mut with_map = bytes.clone();
        with_map.extend([2, 9, 0, 0, 0, 8, 0, 0, 0]);
        for layout in [Layout::HighFive, Layout::Interlude] {
            let mut reader = Reader::at(&with_map, 0);
            let section = raw_section(&mut reader, layout).unwrap();
            assert_eq!((section.material, section.bone, section.first_face, section.faces), (7, 3, 5, 2));
            assert_eq!(section.bone_map, [9, 8]);
            assert!(reader.remaining().is_empty());
            assert!(raw_section(&mut Reader::at(&with_map[..with_map.len() - 1], 0), layout).is_err());
        }
        let mut reader = Reader::at(&bytes, 0);
        assert!(raw_section(&mut reader, Layout::Chronicle4).unwrap().bone_map.is_empty());
        assert!(reader.remaining().is_empty());
    }

    #[test]
    fn packed_stream_rejects_a_reused_point_that_was_never_shared() {
        let words = [0x0FFF_F000, 0, 0, 0xF000_0000, 0, 0, u32::MAX];
        let points = [([0.0; 3], 0)];
        assert!(packed_stream(&words, &points, &[], &[]).is_err());
    }

    #[test]
    fn packed_stream_requires_the_terminator() {
        let words = [0x8FFF_F000, 0, 0];
        let points = [([0.0; 3], 0)];
        assert!(packed_stream(&words, &points, &[], &[]).is_err());
        assert!(packed_stream(&[u32::MAX], &[], &[], &[]).unwrap().vertices.is_empty());
    }

    #[test]
    fn rigid_vertices_without_tables_fall_back_to_the_head() {
        let bone = |name: &str| Bone { name: name.to_owned(), parent: 0, rotation: [0.0; 4], position: [0.0; 3] };
        let bones = [bone("Bip01"), bone("Bip01_Pelvis"), bone("Bip01_head")];

        let weights = rigid_weights(&bones, 2, &[], &[]).unwrap();
        assert_eq!(weights, vec![([2, 0, 0, 0], [1.0, 0.0, 0.0, 0.0]); 2]);
        assert!(rigid_weights(&bones[..2], 2, &[], &[]).is_err());
        assert_eq!(rigid_weights(&bones[..1], 2, &[], &[]).unwrap(), vec![([0, 0, 0, 0], [1.0, 0.0, 0.0, 0.0]); 2]);

        let weights = rigid_weights(&bones, 2, &[(1, [0.0; 2]), (0, [0.0; 2])], &[(1.0, 0, 1), (1.0, 1, 2)]).unwrap();
        assert_eq!(weights[0].0[0], 2);
        assert_eq!(weights[1].0[0], 1);
        assert!(rigid_weights(&bones, 3, &[(1, [0.0; 2]), (0, [0.0; 2])], &[(1.0, 0, 1)]).is_err());
    }
}
