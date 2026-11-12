//! `StaticMesh` objects: the vertex, UV and index streams a renderer needs, plus section materials.

use ue2_package::{Export, ObjectRef, Package};

use super::{array, count, vector, MeshCollision};
use super::collision::{immediate_range, lazy_range};
use crate::Error;
use crate::properties::{Property, find, object_properties_reader};

/// A run of primitives drawn with one material.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Section {
    /// Offset into `StaticMesh::indices`.
    pub first_index: u32,
    /// `NumPrimitives`, the count the engine hands to `DrawPrimitive`; `NumTriangles` is only a statistic.
    pub primitives: u32,
    /// Drawn as a triangle strip instead of a list.
    pub strip: bool,
}

impl Section {
    pub fn index_count(&self) -> u32 {
        if self.strip { self.primitives + 2 } else { self.primitives * 3 }
    }
}

/// The L2 tail of `UStaticMesh::Serialize`: distance LODs, sway and billboard, by `LicenseeVer`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeshLod {
    pub mesh01: ObjectRef,
    pub mesh02: ObjectRef,
    /// `LodRange01/02`, in `option.ini` units (× 2048 world units).
    pub range01: f32,
    pub range02: f32,
    pub blend: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sway {
    pub frequency: f32,
    pub max_angle: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaticMesh {
    pub collision: MeshCollision,
    pub bounding_box: [[f32; 3]; 2],
    pub bounding_sphere: [f32; 4],
    pub positions: Vec<[f32; 3]>,
    /// One normal per position.
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[u8; 4]>,
    /// First UV channel; empty when the mesh has none.
    pub uvs: Vec<[f32; 2]>,
    /// Second UV channel; empty when the mesh has only one.
    pub uvs2: Vec<[f32; 2]>,
    /// Triangle list indices into the vertex streams.
    pub indices: Vec<u16>,
    pub sections: Vec<Section>,
    /// Material of each section, in section order; `Null` when unset.
    pub materials: Vec<ObjectRef>,
    pub no_dynamic_shadow_cast: Vec<bool>,
    /// `bStaticMeshLod` with its two meshes and ranges.
    pub lod: Option<MeshLod>,
    /// `bSwayObject` with `Frequency` and `MaxSwayAngle`.
    pub sway: Option<Sway>,
    /// `bUseBillBoard`: yawed toward the player every frame.
    pub billboard: bool,
}

const LICENSEE_LAZY_COLLISION: u16 = 0x11;
const VERSION_OLD_COLLISION_INTS: u16 = 0x72;
const LICENSEE_LOD: u16 = 5;
const LICENSEE_SWAY: u16 = 6;
const LICENSEE_SWAY_ANGLE: u16 = 10;
const LICENSEE_LOD_BLEND: u16 = 12;
const LICENSEE_TWO_SIDE_MESH: u16 = 13;
const LICENSEE_BILLBOARD: u16 = 14;

pub fn read_static_mesh<'a>(package: &'a Package, file: impl Into<ue2_core::ByteWindow<'a>>, export: &Export) -> Result<StaticMesh, Error> {
    let (properties, mut reader) = object_properties_reader(package, file, export)?;
    let (materials, no_dynamic_shadow_cast) = materials(package, find(&properties, "Materials"));

    let bounding_box = [vector(&mut reader)?, vector(&mut reader)?];
    reader.u8()?;
    let bounding_sphere = [reader.f32()?, reader.f32()?, reader.f32()?, reader.f32()?];
    let sections = array(&mut reader, |reader| {
        let strip = reader.u32()? != 0;
        let first_index = u32::from(reader.u16()?);
        reader.bytes(4)?; // vertex index range
        reader.u16()?; // triangle count, kept by the engine only for statistics
        let primitives = u32::from(reader.u16()?);
        Ok(Section { first_index, primitives, strip })
    })?;
    reader.bytes(25)?; // bounding box again

    let (positions, normals) = array(&mut reader, |reader| Ok((vector(reader)?, vector(reader)?)))?.into_iter().unzip();
    reader.u32()?; // revision
    let colors = array(&mut reader, |reader| { let [b, g, r, a] = [reader.u8()?, reader.u8()?, reader.u8()?, reader.u8()?]; Ok([r, g, b, a]) })?;
    reader.u32()?;
    let len = count(&mut reader)?;
    reader.bytes(len.checked_mul(4).ok_or(Error::BadMesh)?)?;
    reader.u32()?;
    let uv_streams = array(&mut reader, |reader| {
        let uvs = array(reader, |reader| Ok([reader.f32()?, reader.f32()?]))?;
        reader.bytes(8)?; // coordinate index and revision
        Ok(uvs)
    })?;
    let indices = array(&mut reader, |reader| Ok(reader.u16()?))?;
    let Tail { collision, lod, sway, billboard } = l2_tail(package, &mut reader, &properties)?;

    let mut uv_streams = uv_streams.into_iter();
    let uvs = uv_streams.next().unwrap_or_default();
    let uvs2 = uv_streams.next().filter(|uvs2| uvs2.len() == uvs.len()).unwrap_or_default();
    let mesh = StaticMesh { collision, bounding_box, bounding_sphere, positions, normals, colors, uvs, uvs2, indices, sections, materials, no_dynamic_shadow_cast, lod, sway, billboard };
    validate(&mesh)?;
    Ok(mesh)
}

struct Tail {
    collision: MeshCollision,
    lod: Option<MeshLod>,
    sway: Option<Sway>,
    billboard: bool,
}

fn l2_tail(package: &Package, reader: &mut ue2_core::Reader<'_>, properties: &[Property<'_>]) -> Result<Tail, Error> {
    reader.u32()?; // index buffer revision
    let wireframe = count(reader)?;
    reader.bytes(wireframe.checked_mul(2).ok_or(Error::BadMesh)?)?;
    reader.u32()?; // wireframe revision
    let model = objref(reader.compact()?);
    let (triangles, nodes) = if package.licensee < LICENSEE_LAZY_COLLISION {
        (immediate_range(reader, true)?, immediate_range(reader, false)?)
    } else {
        (lazy_range(reader)?, lazy_range(reader)?)
    };
    let mut collision = MeshCollision { model, triangles, nodes,
        simple_line:find(properties,"UseSimpleLineCollision").and_then(Property::bool).unwrap_or(false),
        simple_box:find(properties,"UseSimpleBoxCollision").and_then(Property::bool).unwrap_or(true),
        simple_karma:find(properties,"UseSimpleKarmaCollision").and_then(Property::bool).unwrap_or(false),
    };
    if package.version < VERSION_OLD_COLLISION_INTS {
        collision.simple_line = reader.i32()? != 0;
        collision.simple_box = reader.i32()? != 0;
        collision.simple_karma = reader.i32()? != 0;
    }
    let mut lod = None;
    let mut sway = None;
    let mut billboard = false;
    if package.licensee > LICENSEE_LOD {
        let enabled = reader.i32()? != 0;
        let mesh01 = objref(reader.compact()?);
        let mesh02 = objref(reader.compact()?);
        let range01 = reader.f32()?;
        let range02 = reader.f32()?;
        let mut blend = false;
        let mut frequency = 0.0;
        let mut sways = false;
        let mut max_angle = 0.0;
        if package.licensee > LICENSEE_SWAY {
            sways = reader.i32()? != 0;
            frequency = reader.f32()?;
        }
        if package.licensee > LICENSEE_SWAY_ANGLE {
            max_angle = reader.f32()?;
        }
        if package.licensee > LICENSEE_LOD_BLEND {
            blend = reader.i32()? != 0;
        }
        if package.licensee > LICENSEE_TWO_SIDE_MESH {
            reader.bytes(8)?;
        }
        if package.licensee > LICENSEE_BILLBOARD {
            billboard = reader.i32()? != 0;
        }
        if enabled {
            lod = Some(MeshLod { mesh01, mesh02, range01, range02, blend });
        }
        if sways {
            sway = Some(Sway { frequency, max_angle });
        }
    }
    Ok(Tail { collision, lod, sway, billboard })
}

fn objref(index: i32) -> ObjectRef {
    match index {
        0 => ObjectRef::Null,
        i if i > 0 => ObjectRef::Export((i - 1) as usize),
        i => ObjectRef::Import((-i - 1) as usize),
    }
}

/// Every index and section must stay inside the streams, which a misread layout cannot satisfy.
fn validate(mesh: &StaticMesh) -> Result<(), Error> {
    let vertices = mesh.positions.len();
    let uvs_fit = mesh.uvs.is_empty() || mesh.uvs.len() == vertices;
    let indices_fit = mesh.indices.iter().all(|&index| usize::from(index) < vertices);
    let sections_fit = mesh.sections.iter().all(|section| {
        section.primitives == 0
            || (section.first_index as usize).checked_add(section.index_count() as usize).is_some_and(|end| end <= mesh.indices.len())
    });
    if uvs_fit && indices_fit && sections_fit { Ok(()) } else { Err(Error::BadMesh) }
}

/// The `Material` of each `StaticMeshMaterial` in the `Materials` property.
fn materials(package: &Package, property: Option<&Property<'_>>) -> (Vec<ObjectRef>, Vec<bool>) {
    property
        .and_then(|materials| materials.structs(package))
        .map(|materials| {
            let entries = materials
                .iter()
                .map(|material| (
                    find(material, "Material").and_then(|material| material.object(package)).unwrap_or(ObjectRef::Null),
                    find(material, "bNoDynamicShadowCast").and_then(Property::bool).unwrap_or(false),
                ))
                .collect::<Vec<_>>();
            entries.into_iter().unzip()
        })
        .unwrap_or_else(|| (Vec::new(), Vec::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f32s(values: &[f32]) -> Vec<u8> {
        values.iter().flat_map(|value| value.to_le_bytes()).collect()
    }

    /// A package whose only export is a quad with one section, laid out like `L2_Lobby.BloodLobbyMoon_S`.
    fn quad(indices: &[u16]) -> Vec<u8> {
        let mut mesh = vec![0]; // properties: None
        mesh.extend([0; 41]); // bounding box and sphere
        mesh.push(1); // one section
        mesh.extend([0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 9, 0, 2, 0]);
        mesh.extend([0; 25]);
        mesh.push(4);
        for corner in [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]] {
            mesh.extend(f32s(&[corner[0], 0.0, corner[1], 0.0, -1.0, 0.0]));
        }
        mesh.extend([2, 0, 0, 0]);
        for _ in 0..2 {
            mesh.push(4);
            mesh.extend([10, 20, 30, 40].repeat(4));
            mesh.extend([2, 0, 0, 0]);
        }
        mesh.extend([1, 4]);
        mesh.extend(f32s(&[0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0]));
        mesh.extend([0; 8]);
        mesh.push(u8::try_from(indices.len()).unwrap());
        mesh.extend(indices.iter().flat_map(|index| index.to_le_bytes()));
        mesh.extend([2, 0, 0, 0]);
        mesh.extend([0; 8]);

        let mut file = Vec::new();
        for value in [0x9E2A_83C1, 123, 0, 1, 36, 1, 0, 0, 0] {
            file.extend(u32::to_le_bytes(value));
        }
        file.extend([5, b'N', b'o', b'n', b'e', 0, 0, 0, 0, 0]);
        let export_offset = u32::try_from(file.len()).unwrap();
        // The export entry is 11 bytes of fixed fields plus two 2-byte compact indices.
        let (offset, size) = (file.len() + 15, mesh.len());
        file[24..28].copy_from_slice(&export_offset.to_le_bytes());
        file.extend([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // class, super, outer, name, flags
        for value in [size, offset] {
            file.push(u8::try_from(value & 0x3F).unwrap() | 0x40);
            file.push(u8::try_from(value >> 6).unwrap());
        }
        file.extend(mesh);
        file
    }

    #[test]
    #[expect(clippy::float_cmp, reason = "values are written and read back as the same bits")]
    fn reads_streams_and_rejects_indices_outside_them() {
        let file = quad(&[0, 1, 2, 2, 3, 0]);
        let package = Package::parse(&file).unwrap();
        let mesh = read_static_mesh(&package, &file, &package.exports()[0]).unwrap();
        assert_eq!(mesh.positions[2], [1.0, 0.0, 1.0]);
        assert_eq!(mesh.normals[2], [0.0, -1.0, 0.0]);
        assert_eq!(mesh.colors, [[30, 20, 10, 40]; 4]);
        assert_eq!(mesh.uvs[1], [1.0, 1.0]);
        assert_eq!(mesh.indices, [0, 1, 2, 2, 3, 0]);
        assert_eq!(mesh.sections, [Section { first_index: 0, primitives: 2, strip: false }]);
        assert_eq!(mesh.sections[0].index_count(), 6);
        assert_eq!(Section { first_index: 0, primitives: 4, strip: true }.index_count(), 6);

        let file = quad(&[0, 1, 2, 2, 3, 4]);
        let package = Package::parse(&file).unwrap();
        assert!(matches!(read_static_mesh(&package, &file, &package.exports()[0]), Err(Error::BadMesh)));
    }
}
