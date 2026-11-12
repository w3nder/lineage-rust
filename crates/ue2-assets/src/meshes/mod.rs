//! Mesh objects: static meshes, skinned body parts and the animations that move them.

mod animation;
mod collision;
mod skeletal;
mod static_mesh;
mod vertex;
mod vertex_animation;

use ue2_core::Reader;

use crate::Error;

pub use collision::{CollisionNode, CollisionTriangle, MeshCollision};
pub use animation::{AnimBone, MeshAnimation, Notify, NotifyClass, NotifyProperty, NotifyValue, Sequence, Track, read_mesh_animation, read_class_notify_defaults, read_notify_properties};
pub use skeletal::{Alias, Bone, SkeletalLodInfo, SkeletalLodSettings, SkeletalMesh, SkinVertex, Layout, read_skeletal_mesh, read_skeletal_mesh_as, read_skeletal_mesh_lod, read_skeletal_mesh_lod_with_info};
pub use static_mesh::{Section, StaticMesh, read_static_mesh};
pub use vertex::{VertexMesh, VertexPoint, VertexPose, VertexSequence, VertexWedge, read_vertex_mesh};
pub use vertex_animation::{VertexAnimation, VertexAnimEvent};

pub(crate) fn count(reader: &mut Reader<'_>) -> Result<usize, Error> {
    usize::try_from(reader.compact()?).map_err(|_| Error::BadMesh)
}

pub(crate) fn array<'a, T>(
    reader: &mut Reader<'a>,
    mut item: impl FnMut(&mut Reader<'a>) -> Result<T, Error>,
) -> Result<Vec<T>, Error> {
    let len = count(reader)?;
    // A count larger than the bytes left is a misread, not a reason to allocate.
    if len > reader.remaining().len() {
        return Err(Error::BadMesh);
    }
    (0..len).map(|_| item(reader)).collect()
}

pub(crate) fn vector(reader: &mut Reader<'_>) -> Result<[f32; 3], Error> {
    Ok([reader.f32()?, reader.f32()?, reader.f32()?])
}

/// Skips an array of fixed-size elements.
pub(crate) fn skip_array(reader: &mut Reader<'_>, size: usize) -> Result<(), Error> {
    let len = count(reader)?.checked_mul(size).ok_or(Error::BadMesh)?;
    reader.bytes(len)?;
    Ok(())
}
