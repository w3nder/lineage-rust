//! Unreal Engine 2 objects read in place from a decrypted package (see `ue2-package`).
//!
//! Nothing is converted or rewritten: parsed objects borrow the original bytes, and
//! pixels are decoded to RGBA only when a caller asks for them.

mod meshes;
mod properties;
mod script;
mod sound;
pub use sound::{Sound, read_sound};
mod texture;
pub mod wet;

use std::fmt;

use ue2_core::ReadError;

pub use meshes::{
    Alias, AnimBone, Bone, CollisionNode, CollisionTriangle, MeshCollision, MeshAnimation, Notify, NotifyClass, NotifyProperty, NotifyValue, Section, Sequence, SkeletalLodInfo, SkeletalLodSettings, SkeletalMesh, SkinVertex, StaticMesh, Track,
    read_mesh_animation, read_class_notify_defaults, read_notify_properties, read_skeletal_mesh, read_skeletal_mesh_as, read_skeletal_mesh_lod, read_skeletal_mesh_lod_with_info, read_static_mesh, Layout,
};
pub use properties::{Property, class_default_properties, find, object_data, object_properties};
pub use meshes::{VertexAnimEvent, VertexAnimation, VertexMesh, VertexPoint, VertexPose, VertexSequence, VertexWedge, read_vertex_mesh};
pub use texture::{Image, Mip, Texture, TextureFormat, decode_rgba, decode_texture, read_palette, read_texture};

#[derive(Debug)]
pub enum Error {
    BadScript,
    BadSound,
    UnknownScriptToken(u8),
    Read(ReadError),
    Package(ue2_package::Error),
    ExportOutOfRange,
    UnknownPropertyType(u8),
    UnknownTextureFormat(u8),
    UnsupportedFormat(TextureFormat),
    MissingPalette,
    NoMipArray,
    /// Mesh streams that do not fit together.
    BadMesh,
    MipTooShort {
        needed: usize,
        actual: usize,
    },
    TrailingBytes(usize),
    ScriptedClass(i32),
    /// A package whose object layout this reader does not follow.
    UnsupportedLayout {
        version: u16,
        licensee: u16,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadSound => f.write_str("invalid serialized sound data"),
            Self::BadScript => f.write_str("script logical size or alignment is invalid"),
            Self::UnknownScriptToken(token) => write!(f, "unknown script token {token:#x}"),
            Self::Read(error) => error.fmt(f),
            Self::Package(error) => error.fmt(f),
            Self::ExportOutOfRange => f.write_str("export data lies outside the file"),
            Self::UnknownPropertyType(kind) => write!(f, "unknown property type {kind}"),
            Self::UnknownTextureFormat(format) => write!(f, "unknown texture format {format}"),
            Self::UnsupportedFormat(format) => write!(f, "decoding {format:?} is not supported"),
            Self::MissingPalette => f.write_str("paletted texture without a palette in this package"),
            Self::NoMipArray => f.write_str("no mip array ends at the end of the object"),
            Self::BadMesh => f.write_str("mesh streams do not fit together"),
            Self::MipTooShort { needed, actual } => write!(f, "mip needs {needed} bytes, has {actual}"),
            Self::TrailingBytes(len) => write!(f, "{len} unread bytes at the end of the object"),
            Self::ScriptedClass(size) => write!(f, "class with {size} bytes of script before its defaults"),
            Self::UnsupportedLayout { version, licensee } => {
                write!(f, "package version {version} licensee {licensee} is not supported")
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<ReadError> for Error {
    fn from(error: ReadError) -> Self {
        Self::Read(error)
    }
}

impl From<ue2_package::Error> for Error {
    fn from(error: ue2_package::Error) -> Self {
        Self::Package(error)
    }
}
