//! `SkeletalMesh` objects as Lineage II High Five stores them: a skinned body part with its bind skeleton.
//!

mod lod;

use ue2_core::Reader;
use ue2_package::{Export, ObjectRef, Package};

use super::{Section, array, count, skip_array, vector};
use crate::Error;
use crate::properties::object_properties_reader;

/// The layout these readers follow: package version 123, licensee 37.
const LAYOUT: (u16, u16) = (123, 37);
const CHRONICLE4_LAYOUT: (u16, u16) = (123, 27);
const UNLICENSED_LAYOUT: (u16, u16) = (123, 0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    HighFive,
    Chronicle4,
    Unlicensed,
    Interlude,
}

/// Refuses packages saved with another layout, such as custom content made with other tools.
pub(super) fn check_layout(package: &Package) -> Result<Layout, Error> {
    match (package.version, package.licensee) {
        LAYOUT => Ok(Layout::HighFive),
        (123, 28 | 30 | 31) => Ok(Layout::Interlude),
        CHRONICLE4_LAYOUT | (123, 26) => Ok(Layout::Chronicle4),
        UNLICENSED_LAYOUT | (117, 0) => Ok(Layout::Unlicensed),
        (version, licensee) => Err(Error::UnsupportedLayout { version, licensee }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package(version: u16, licensee: u16) -> Package {
        let mut bytes = vec![0xC1, 0x83, 0x2A, 0x9E];
        bytes.extend(version.to_le_bytes());
        bytes.extend(licensee.to_le_bytes());
        bytes.extend([0; 4]);
        bytes.extend([0; 32]);
        Package::parse(&bytes).unwrap()
    }

    #[test]
    fn licensees_28_30_and_31_select_the_interlude_layout_and_the_others_keep_theirs() {
        for licensee in [28, 30, 31] {
            assert_eq!(check_layout(&package(123, licensee)).unwrap(), Layout::Interlude);
        }
        assert_eq!(check_layout(&package(123, 37)).unwrap(), Layout::HighFive);
        assert_eq!(check_layout(&package(123, 27)).unwrap(), Layout::Chronicle4);
        assert_eq!(check_layout(&package(123, 26)).unwrap(), Layout::Chronicle4);
        assert!(check_layout(&package(123, 29)).is_err());
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bone {
    pub name: String,
    /// Index of the parent bone; the root is its own parent.
    pub parent: usize,
    /// Bind pose relative to the parent: rotation as a quaternion (x, y, z, w), then translation.
    ///
    /// Composing the pose follows the ActorX convention: the root's quaternion is used as stored and every other
    /// bone's is conjugated before `parent * child` (`world = parent_rotation * conj(rotation)`,
    /// `origin = parent_origin + parent_rotation.rotate(position)`). Animation keys (`Track::rotations`) replace
    /// `rotation` under the same convention.
    pub rotation: [f32; 4],
    pub position: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkinVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    /// Indices into `SkeletalMesh::bones`; unused slots have weight 0.
    pub bones: [u16; 4],
    pub weights: [f32; 4],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkeletalLodInfo {
    pub soft_wedges: i32,
    pub distance_factor: f32,
    pub hysteresis: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkeletalLodSettings {
    pub scale_max: f32,
    pub hysteresis: f32,
    pub strength: f32,
    pub min_vertices: i32,
    pub morph: f32,
    pub z_displace: f32,
    pub impostor: Option<i32>,
    pub reduction: Option<i32>,
}

/// An attachment socket: the alias name the engine resolves before searching the skeleton
/// (`USkeletalMesh::MatchRefBone`), the bone it stands for and the socket coordinates (origin and axes).
#[derive(Debug, Clone, PartialEq)]
pub struct Alias {
    pub name: String,
    pub bone: String,
    pub origin: [f32; 3],
    pub axes: [[f32; 3]; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkeletalMesh {
    pub lod_settings: SkeletalLodSettings,
    pub bounding_box: [[f32; 3]; 2],
    pub bounding_sphere: [f32; 4],
    pub bones: Vec<Bone>,
    pub vertices: Vec<SkinVertex>,
    /// Triangle list indices into `vertices`.
    pub indices: Vec<u16>,
    pub sections: Vec<Section>,
    pub section_material_slots: Vec<u16>,
    pub material_texture_indices: Vec<i32>,
    pub textures: Vec<ObjectRef>,
    /// Material of each section, in section order; `Null` when unset.
    pub materials: Vec<ObjectRef>,
    /// The `MeshAnimation` the mesh plays by default.
    pub animation: ObjectRef,
    pub scale: [f32; 3],
    pub origin: [f32; 3],
    /// Pitch, yaw and roll in Unreal rotation units.
    pub rotation: [i32; 3],
    pub aliases: Vec<Alias>,
    /// The word the engine keeps at `mesh+0x14c`: 0 on bodies, 4 on hair skinned like the body and 6 on hair
    /// with its own skeleton, which `USkeletalMeshInstance::Render` sends through `RenderDynamicHairMesh`.
    pub hair_kind: i32,
    pub floats_150: Vec<f32>,
}

impl SkeletalMesh {
    /// `USkeletalMesh::MatchRefBone` (@ 0x1030f146): resolves an alias first, then finds the bone by name,
    /// comparing like `FName` does (case-insensitively, without any other normalisation).
    pub fn match_ref_bone(&self, name: &str) -> Option<usize> {
        let name = self.aliases.iter().find(|a| a.name.eq_ignore_ascii_case(name)).map_or(name, |a| a.bone.as_str());
        self.bones.iter().position(|b| b.name.eq_ignore_ascii_case(name))
    }
}

pub fn read_skeletal_mesh<'a>(package: &'a Package, file: impl Into<ue2_core::ByteWindow<'a>>, export: &Export) -> Result<SkeletalMesh, Error> {
    read_skeletal_mesh_lod(package, file, export, 0)
}

pub fn read_skeletal_mesh_lod<'a>(package: &'a Package, file: impl Into<ue2_core::ByteWindow<'a>>, export: &Export, selected_lod: usize) -> Result<SkeletalMesh, Error> {
    read_skeletal_mesh_lod_with_info(package, file, export, selected_lod).map(|(mesh, _)| mesh)
}

pub fn read_skeletal_mesh_lod_with_info<'a>(package: &'a Package, file: impl Into<ue2_core::ByteWindow<'a>>, export: &Export, selected_lod: usize) -> Result<(SkeletalMesh, Vec<SkeletalLodInfo>), Error> {
    let layout = check_layout(package)?;
    read_skeletal_mesh_as(package, file, export, selected_lod, layout)
}

pub fn read_skeletal_mesh_as<'a>(package: &'a Package, file: impl Into<ue2_core::ByteWindow<'a>>, export: &Export, selected_lod: usize, layout: Layout) -> Result<(SkeletalMesh, Vec<SkeletalLodInfo>), Error> {
    let (_, mut reader) = object_properties_reader(package, file, export)?;
    let reader = &mut reader;
    let bounding_box = [vector(reader)?, vector(reader)?];
    reader.u8()?;
    let bounding_sphere = [reader.f32()?, reader.f32()?, reader.f32()?, reader.f32()?];

    // LodMesh
    let version = reader.i32()?;
    reader.i32()?; // vertex count
    skip_array(reader, 4)?; // packed vertices of the older mesh format
    let textures = array(reader, |reader| Ok(package.object_at(reader.compact()?)?))?;
    let scale = vector(reader)?;
    let origin = vector(reader)?;
    let rotation = [reader.i32()?, reader.i32()?, reader.i32()?];
    skip_array(reader, 2)?; // face levels
    skip_array(reader, 8)?; // faces
    skip_array(reader, 2)?; // collapse wedges
    skip_array(reader, 10)?; // wedges
    let slots = array(reader, |reader| {
        reader.u32()?; // poly flags
        Ok(reader.i32()?)
    })?;
    let mut lod_settings = SkeletalLodSettings {
        scale_max: reader.f32()?,
        hysteresis: reader.f32()?,
        strength: reader.f32()?,
        min_vertices: reader.i32()?,
        morph: reader.f32()?,
        z_displace: reader.f32()?,
        impostor: None,
        reduction: None,
    };
    if version >= 3 {
        lod_settings.impostor = Some(reader.i32()?);
        reader.compact()?; // sprite material
        reader.bytes(12 + 12 + 12 + 4 + 12)?; // impostor location, rotation, scale, color and modes
    }
    if version >= 4 {
        reader.f32()?; // skin tessellation factor
    }
    if version >= 5 {
        lod_settings.reduction = Some(reader.i32()?);
    }
    if version >= 6 {
        reader.u8()?;
    }

    // SkeletalMesh
    skip_array(reader, 12)?; // points of the older mesh format
    let bones = array(reader, |reader| bone(package, reader))?;
    let animation = package.object_at(reader.compact()?)?;
    reader.i32()?; // skeletal depth
    array(reader, |reader| {
        skip_array(reader, 2)?;
        Ok(reader.i32()?)
    })?; // weight indices
    skip_array(reader, 4)?; // bone influences
    let alias_names = array(reader, |reader| Ok(package.name_at(reader.compact()?)?.to_owned()))?;
    let alias_bones = array(reader, |reader| Ok(package.name_at(reader.compact()?)?.to_owned()))?;
    let alias_coords =
        array(reader, |reader| Ok((vector(reader)?, [vector(reader)?, vector(reader)?, vector(reader)?])))?;
    if alias_names.len() != alias_bones.len() || alias_names.len() != alias_coords.len() {
        return Err(Error::BadMesh);
    }
    let aliases = alias_names
        .into_iter()
        .zip(alias_bones)
        .zip(alias_coords)
        .map(|((name, bone), (origin, axes))| Alias { name, bone, origin, axes })
        .collect();
    let lod_count = count(reader)?;
    if selected_lod >= lod_count { return Err(Error::BadMesh); }
    let (mut lod, info) = lod::read(reader, &bones, layout)?;
    let mut lod_info = vec![info];
    for index in 1..lod_count {
        let (candidate, info) = lod::read(reader, &bones, layout)?;
        lod_info.push(info);
        if index == selected_lod { lod = candidate; }
    }
    reader.compact()?;
    for size in [12, 10, 12, 8, 2, 2] {
        reader.u32()?; // lazy array skip offset
        skip_array(reader, size)?; // points, wedges, triangles, influences and collapse tables of the older format
    }
    let hair_kind = if package.version > 117 && package.licensee > 2 { reader.i32()? } else { 0 };
    let mut floats_150 = Vec::new();
    if layout != Layout::Unlicensed {
        floats_150 = array(reader, |reader| Ok(reader.f32()?))?;
    }
    if package.version > 119 { reader.i32()?; }
    if layout == Layout::HighFive {
        reader.i32()?;
    }
    if !reader.remaining().is_empty() {
        return Err(Error::TrailingBytes(reader.remaining().len()));
    }

    if !bones.iter().enumerate().all(|(index, bone)| bone.parent <= index) {
        return Err(Error::BadMesh);
    }
    let material = |&(slot, _): &(u16, Section)| {
        let texture = slots.get(usize::from(slot)).and_then(|&texture| usize::try_from(texture).ok());
        texture.and_then(|texture| textures.get(texture)).copied().unwrap_or(ObjectRef::Null)
    };
    let materials = lod.sections.iter().map(material).collect();
    let sections = lod.sections.iter().map(|&(_, section)| section).collect();
    let section_material_slots = lod.sections.iter().map(|&(slot, _)| slot).collect();
    Ok((SkeletalMesh {
        section_material_slots,
        material_texture_indices: slots,
        textures,
        lod_settings,
        bounding_box,
        bounding_sphere,
        bones,
        vertices: lod.vertices,
        indices: lod.indices,
        sections,
        materials,
        animation,
        scale,
        origin,
        rotation,
        aliases,
        hair_kind,
        floats_150,
    }, lod_info))
}

fn bone(package: &Package, reader: &mut Reader<'_>) -> Result<Bone, Error> {
    let name = package.name_at(reader.compact()?)?.to_owned();
    reader.u32()?; // flags
    let rotation = [reader.f32()?, reader.f32()?, reader.f32()?, reader.f32()?];
    let position = vector(reader)?;
    reader.bytes(16)?; // length and size
    reader.i32()?; // children
    let parent = usize::try_from(reader.i32()?).map_err(|_| Error::BadMesh)?;
    Ok(Bone { name, parent, rotation, position })
}
