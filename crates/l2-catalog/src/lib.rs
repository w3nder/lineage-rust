//! Assets of an installed client found by path (`Package.Name` or `Package.Group.Name`) and read in
//! place: textures, static and skeletal meshes, animations and the texture a material draws with. A
//! package's index of names and exports stays loaded once read; the bytes of an object are read from
//! the file when it is asked for, so no package is ever held whole.

mod fade;
pub mod heightmap;
pub mod l2_color_stage;
mod material;
mod uv;
mod wet;
pub use wet::WetTexture;

use killer_platform::fs::{self, File};
use std::collections::HashMap;
use std::io::{self, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use l2_crypto::Window;
use ue2_assets::{Image, MeshAnimation, Mip, SkeletalMesh, StaticMesh, TextureFormat};
use ue2_package::{Export, ObjectRef, Package};

pub use fade::Fade;
pub use material::{
    ALPHA_REF_MASKED, AlphaOp, BlendFactor, ColorOp, CoordSource, Cubemap, DynamicProjectorParameters, FogMode,
    Material, Pass, ShaderFlags, Stage, StageSource, TextureRef, UvModifier,
};
pub use uv::{Mat3x2, Oscillator, Rotator, Trigger, frame_index};

/// Folders of the client that hold asset packages, with the extension used there.
const FOLDERS: [(&str, &str); 5] =
    [("SysTextures", "utx"), ("Textures", "utx"), ("StaticMeshes", "usx"), ("Animations", "ukx"), ("Maps", "unr")];

/// Deeper material chains than this are treated as cycles.
const MAX_MATERIAL_DEPTH: usize = 8;
/// Frames of one animated texture to follow before giving up on a chain that never closes.
const MAX_TEXTURE_FRAMES: usize = 64;

#[derive(Default)]
pub struct Catalog {
    /// Lower-case package name to file.
    files: HashMap<String, PathBuf>,
    /// Packages of the base client, tried when an object is missing from `files` (keys carry a leading `~`).
    fallback: HashMap<String, PathBuf>,
    /// Lower-case package name to its contents, `None` when it failed to load.
    packages: HashMap<String, Option<Loaded>>,
    scratch: Vec<u8>,
    enchant_color_override: Option<(String, [[u8; 3]; 2])>,
    class_defaults: HashMap<String, std::sync::Arc<Vec<material::ClassDefault>>>,
    engine_classes: HashMap<String, bool>,
}

struct Loaded {
    path: PathBuf,
    /// How to decrypt any range of the file, or `None` when it only decrypts whole.
    window: Option<Window>,
    /// The decrypted file, kept only when it has no `window`.
    whole: Option<Vec<u8>>,
    package: Package,
    /// Lower-case `Group.Name` and bare `Name` to export index.
    objects: HashMap<String, usize>,
}

/// An export found by path, with the bytes of its package that hold it.
struct Found<'a> {
    package: &'a Package,
    file: ue2_core::ByteWindow<'a>,
    index: usize,
    objects: &'a HashMap<String, usize>,
}

/// 16-bit terrain heights, row by row.
#[derive(Debug, Clone)]
pub struct Heightmap {
    pub width: usize,
    pub height: usize,
    pub samples: Vec<u16>,
}

/// A texture with every mip's raw bytes untouched (still DXT-compressed for `Dxt1`/`Dxt3`/`Dxt5`),
/// largest mip first.
#[derive(Debug, Clone)]
pub struct RawTexture {
    pub format: TextureFormat,
    pub palette: Option<Vec<[u8; 4]>>,
    pub mips: Vec<RawMip>,
}

#[derive(Debug, Clone)]
pub struct RawMip {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageStatus {
    Missing,
    Encrypted,
    Unreadable,
    Loaded,
}

const STANDARD_COPY_MARK: &str = "std";

fn is_proprietary_container(path: &Path) -> bool {
    let mut magic = [0u8; 4];
    File::open(path).and_then(|mut f| f.read_exact(&mut magic)).is_ok()
        && magic == l2_crypto::PROPRIETARY_CONTAINER_MAGIC
}

fn standard_copy(path: &Path) -> Option<PathBuf> {
    let stem = path.file_stem()?.to_str()?;
    let extension = path.extension()?.to_str()?;
    let copy = path.with_file_name(format!("{stem}.{STANDARD_COPY_MARK}.{extension}"));
    copy.is_file().then_some(copy)
}

/// Why an asset path did not resolve, with the words the log lines use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MissingAsset {
    BadPath,
    Package,
    Encrypted,
    UnreadablePackage,
    Object,
    WrongClass(String),
    Undecodable { version: u16, licensee: u16, error: String },
}

impl std::fmt::Display for MissingAsset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadPath => write!(f, "caminho sem pacote"),
            Self::Package => write!(f, "pacote ausente"),
            Self::Encrypted => write!(f, "pacote CAFEF00D: contêiner conhecido, não decifrável pelo porte"),
            Self::UnreadablePackage => write!(f, "pacote ilegível"),
            Self::Object => write!(f, "objeto ausente"),
            Self::WrongClass(class) => write!(f, "objeto é {class}"),
            Self::Undecodable { version, licensee, error } => {
                write!(f, "indecodificável (v{version} licensee {licensee}: {error})")
            }
        }
    }
}

/// A skeletal mesh with its default animation and each section's material as client-wide paths.
#[derive(Debug, Clone)]
pub struct Skinned {
    pub mesh: SkeletalMesh,
    pub animation: Option<String>,
    pub materials: Vec<Option<String>>,
    pub textures: Vec<Option<String>>,
}

/// A static mesh with each section's material as a client-wide path.
#[derive(Debug, Clone)]
pub struct Mesh {
    pub mesh: StaticMesh,
    /// Material of each section, `None` when unset.
    pub materials: Vec<Option<String>>,
    /// Client paths of `StaticMeshLod01/02` when the mesh has distance LODs.
    pub lod_meshes: [Option<String>; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub struct VertexAnimated {
    pub name: String,
    pub mesh: ue2_assets::VertexMesh,
    pub textures: Vec<Option<String>>,
}

#[derive(Debug, Clone)]
pub struct StaticCollision {
    pub bounding_box: [[f32; 3]; 2],
    pub simple_line: bool,
    pub simple_box: bool,
    pub positions: Vec<[f32; 3]>,
    pub triangles: Vec<ue2_assets::CollisionTriangle>,
    pub nodes: Vec<ue2_assets::CollisionNode>,
    pub model: Option<String>,
}

impl Catalog {
    pub fn register_package(&mut self, name: &str, path: &Path) {
        let key = name.to_lowercase();
        self.files.insert(key.clone(), path.to_owned());
        self.packages.remove(&key);
    }

    /// Indexes the asset packages under a client root; missing folders are skipped.
    pub fn open(client_root: &Path) -> Self {
        Self { files: scan(client_root), ..Self::default() }
    }

    /// Like `open`, with a second client whose packages answer when the first has no such package or object.
    pub fn open_with_fallback(client_root: &Path, base_root: &Path) -> Self {
        Self { files: scan(client_root), fallback: scan(base_root), ..Self::default() }
    }

    pub fn source_paths(&self) -> impl Iterator<Item = &Path> {
        self.files.values().chain(self.fallback.values()).map(PathBuf::as_path)
    }

    /// Bytes of package files currently held.
    pub fn cached_bytes(&self) -> usize {
        self.scratch.capacity()
            + self
                .packages
                .values()
                .flatten()
                .filter_map(|loaded| loaded.whole.as_ref())
                .map(Vec::capacity)
                .sum::<usize>()
    }

    /// Drops every byte read from the packages, keeping the indexes.
    pub fn release_files(&mut self) {
        self.scratch = Vec::new();
        for loaded in self.packages.values_mut().flatten() {
            loaded.whole = None;
        }
    }

    /// Decodes the top mip of the texture at `path`, or `None` when it does not exist or cannot be read.
    pub fn texture(&mut self, path: &str) -> Option<Image> {
        let raw = self.texture_raw(path)?;
        let mip = raw.mips.first()?;
        let source = Mip { width: mip.width, height: mip.height, data: &mip.data };
        let rgba = ue2_assets::decode_rgba(raw.format, &source, raw.palette.as_deref()).ok()?;
        Some(Image { width: mip.width, height: mip.height, rgba })
    }

    /// The texture at `path` with every mip's raw bytes, for GPU upload without a full RGBA decode.
    pub fn texture_raw(&mut self, path: &str) -> Option<RawTexture> {
        if self.object_class(path).is_some_and(|class| class.eq_ignore_ascii_case("WetTexture")) {
            return self.wet_texture(path).map(|wet| wet.raw());
        }
        let resolved = self.static_texture_path(path)?;
        let path = resolved.as_str();
        let package_name = package_name(path)?;
        let found = self.object(path, "Texture")?;
        let export = found.package.exports().get(found.index)?;
        let texture = ue2_assets::read_texture(found.package, found.file, export).ok()?;
        let format = texture.format;
        let mips = texture
            .mips
            .iter()
            .map(|mip| RawMip { width: mip.width, height: mip.height, data: mip.data.to_vec() })
            .collect();
        let palette = match texture.palette {
            ObjectRef::Export(palette) => {
                let found = self.export(&package_name, palette)?;
                let export = found.package.exports().get(found.index)?;
                ue2_assets::read_palette(found.package, found.file, export).ok()
            }
            ObjectRef::Null | ObjectRef::Import(_) => None,
        };
        Some(RawTexture { format, palette, mips })
    }

    pub fn heightmap(&mut self, path: &str) -> Option<Heightmap> {
        self.heightmap_with_base(path, 0)
    }

    pub fn heightmap_with_base(&mut self, path: &str, base: i32) -> Option<Heightmap> {
        let found = self.object(path, "Texture")?;
        let texture =
            ue2_assets::read_texture(found.package, found.file, found.package.exports().get(found.index)?).ok()?;
        let mip = texture.mips.first()?;
        let (width, height) = (mip.width as usize, mip.height as usize);
        let samples = heightmap::samples(texture.format, mip.data, base, width * height)?;
        Some(Heightmap { width, height, samples })
    }

    pub fn static_mesh(&mut self, path: &str) -> Option<Mesh> {
        let package_name = package_name(path)?;
        let found = self.object(path, "StaticMesh")?;
        let export = found.package.exports().get(found.index)?;
        let mesh = ue2_assets::read_static_mesh(found.package, found.file, export).ok()?;
        let materials = mesh
            .materials
            .iter()
            .map(|&material| full_path(&package_name, found.package, found.objects, material))
            .collect();
        let lod_meshes = mesh.lod.map_or([None, None], |lod| {
            [
                full_path(&package_name, found.package, found.objects, lod.mesh01),
                full_path(&package_name, found.package, found.objects, lod.mesh02),
            ]
        });
        Some(Mesh { mesh, materials, lod_meshes })
    }

    pub fn vertex_mesh(&mut self, path: &str) -> Option<VertexAnimated> {
        let package_name = package_name(path)?;
        let found = self.object(path, "VertMesh")?;
        let export = found.package.exports().get(found.index)?;
        let mesh = ue2_assets::read_vertex_mesh(found.package, found.file, export).ok()?;
        let textures = mesh
            .textures
            .iter()
            .map(|&material| full_path(&package_name, found.package, found.objects, material))
            .collect();
        Some(VertexAnimated { name: found.package.name(export.name).to_owned(), mesh, textures })
    }

    pub fn inspect_object<T>(
        &mut self,
        path: &str,
        class: &str,
        read: impl FnOnce(&Package, ue2_core::ByteWindow<'_>, &Export) -> Option<T>,
    ) -> Option<T> {
        let found = self.object(path, class)?;
        read(found.package, found.file, found.package.exports().get(found.index)?)
    }

    pub fn static_mesh_collision(&mut self, path: &str) -> Option<StaticCollision> {
        let package_name = package_name(path)?;
        let found = self.object(path, "StaticMesh")?;
        let export = found.package.exports().get(found.index)?;
        let mesh = ue2_assets::read_static_mesh(found.package, found.file, export).ok()?;
        let triangles = mesh.collision.read_triangles(found.file).ok()?;
        let nodes = mesh.collision.read_nodes(found.file).ok()?;
        let model = full_path(&package_name, found.package, found.objects, mesh.collision.model);
        Some(StaticCollision {
            bounding_box: mesh.bounding_box,
            simple_line: mesh.collision.simple_line,
            simple_box: mesh.collision.simple_box,
            positions: mesh.positions,
            triangles,
            nodes,
            model,
        })
    }

    pub fn static_mesh_collision_bounds(&mut self, path: &str) -> Option<([[f32; 3]; 2], Option<String>)> {
        let package_name = package_name(path)?;
        let found = self.object(path, "StaticMesh")?;
        let export = found.package.exports().get(found.index)?;
        let mesh = ue2_assets::read_static_mesh(found.package, found.file, export).ok()?;
        let model = full_path(&package_name, found.package, found.objects, mesh.collision.model);
        Some((mesh.bounding_box, model))
    }

    /// The skeletal mesh at `path`, with its default animation and each section's material as client paths.
    pub fn skeletal_mesh(&mut self, path: &str) -> Option<Skinned> {
        self.try_skeletal_mesh(path).ok()
    }

    /// Like `skeletal_mesh`, but says why the mesh could not be produced.
    pub fn try_skeletal_mesh(&mut self, path: &str) -> Result<Skinned, MissingAsset> {
        self.try_skeletal_mesh_lod(path, 0)
    }

    pub fn try_skeletal_mesh_lod(&mut self, path: &str, lod: usize) -> Result<Skinned, MissingAsset> {
        self.try_skeletal_mesh_lod_with_info(path, lod).map(|(mesh, _)| mesh)
    }

    pub fn try_skeletal_mesh_lod_with_info(
        &mut self,
        path: &str,
        lod: usize,
    ) -> Result<(Skinned, Vec<ue2_assets::SkeletalLodInfo>), MissingAsset> {
        let package_name = package_name(path).ok_or(MissingAsset::BadPath)?;
        let status = self.package_status(&package_name);
        let found = match self.object(path, "SkeletalMesh") {
            Some(found) => found,
            None => {
                return Err(match status {
                    PackageStatus::Missing => MissingAsset::Package,
                    PackageStatus::Encrypted => MissingAsset::Encrypted,
                    PackageStatus::Unreadable => MissingAsset::UnreadablePackage,
                    PackageStatus::Loaded => match self.object_class(path) {
                        Some(class) => MissingAsset::WrongClass(class),
                        None => MissingAsset::Object,
                    },
                });
            }
        };
        let export = found.package.exports().get(found.index).ok_or(MissingAsset::Object)?;
        let (version, licensee) = (found.package.version, found.package.licensee);
        let (mesh, lod_info) = ue2_assets::read_skeletal_mesh_lod_with_info(found.package, found.file, export, lod)
            .map_err(|e| MissingAsset::Undecodable { version, licensee, error: format!("{e:?}") })?;
        let animation = full_path(&package_name, found.package, found.objects, mesh.animation);
        let materials = mesh
            .materials
            .iter()
            .map(|&material| full_path(&package_name, found.package, found.objects, material))
            .collect();
        let textures = mesh
            .textures
            .iter()
            .map(|&texture| full_path(&package_name, found.package, found.objects, texture))
            .collect();
        Ok((Skinned { mesh, animation, materials, textures }, lod_info))
    }

    pub fn sound(&mut self, path: &str) -> Option<(String, Vec<u8>)> {
        let path = path.to_lowercase();
        let (package, object) = path.split_once('.')?;
        let (scope, name) = object.rsplit_once('.').map_or((None, object), |(scope, name)| (Some(scope), name));
        let loaded = self.package(package)?;
        let index = loaded.package.exports().iter().enumerate().find_map(|(index, export)| {
            if !loaded.package.name(export.name).eq_ignore_ascii_case(name)
                || !loaded.package.class_name(export).eq_ignore_ascii_case("Sound")
            {
                return None;
            }
            let full = loaded.package.object_path(ObjectRef::Export(index)).to_lowercase();
            scope
                .is_none_or(|scope| full.strip_prefix(scope).is_some_and(|tail| tail.starts_with('.')))
                .then_some(index)
        })?;
        let found = self.export(package, index)?;
        let export = found.package.exports().get(found.index)?;
        let sound = ue2_assets::read_sound(found.package, found.file, export).ok()?;
        Some((sound.format.to_owned(), sound.encoded.to_vec()))
    }

    pub fn mesh_animation(&mut self, path: &str) -> Option<MeshAnimation> {
        let found = self.object(path, "MeshAnimation")?;
        let export = found.package.exports().get(found.index)?;
        let mut animation = ue2_assets::read_mesh_animation(found.package, found.file, export).ok()?;
        let package = path.split_once('.')?.0.to_lowercase();
        for sequence in &mut animation.sequences {
            for notify in &mut sequence.notifies {
                let Some(index) = notify.export else { continue };
                let mut properties: Vec<ue2_assets::NotifyProperty> = Vec::new();
                for class in notify.classes.iter().rev() {
                    let defaults = match class.export {
                        Some(class_index) => {
                            let Some(found) = self.export(&package, class_index) else { continue };
                            let Some(export) = found.package.exports().get(found.index) else { continue };
                            ue2_assets::read_class_notify_defaults(found.package, found.file, export)
                        }
                        None => {
                            let Some(found) = self.object(&class.path, "Class") else { continue };
                            let Some(export) = found.package.exports().get(found.index) else { continue };
                            ue2_assets::read_class_notify_defaults(found.package, found.file, export)
                        }
                    };
                    merge_properties(&mut properties, defaults);
                }
                let Some(found) = self.export(&package, index) else { continue };
                let Some(export) = found.package.exports().get(found.index) else { continue };
                merge_properties(
                    &mut properties,
                    ue2_assets::read_notify_properties(found.package, found.file, export),
                );
                notify.properties = properties;
            }
        }
        Some(animation)
    }

    /// The export at `path`, when its class is `class` (any when empty), with its bytes read.
    fn object(&mut self, path: &str, class: &str) -> Option<Found<'_>> {
        let path = path.to_lowercase();
        let (package_name, object) = path.split_once('.')?;
        let mut key = package_name.to_owned();
        let mut index = self.locate(&key, object, class);
        if index.is_none() && !key.starts_with('~') && self.fallback.contains_key(&key) {
            key = format!("~{key}");
            index = self.locate(&key, object, class);
        }
        let index = index?;
        self.export(&key, index)
    }

    fn locate(&mut self, package_name: &str, object: &str, class: &str) -> Option<usize> {
        let loaded = self.package(package_name)?;
        let bare = object.rsplit_once('.').map(|(_, name)| name);
        let key = if loaded.objects.contains_key(object) { object } else { bare? };
        let first = *loaded.objects.get(key)?;
        let matches =
            |export: &Export| class.is_empty() || loaded.package.class_name(export).eq_ignore_ascii_case(class);
        if matches(loaded.package.exports().get(first)?) {
            Some(first)
        } else if key.contains('#') {
            None
        } else {
            let exact_scope = loaded.package.object_path(ObjectRef::Export(first)).eq_ignore_ascii_case(key);
            loaded.package.exports().iter().enumerate().find_map(|(index, export)| {
                if !matches(export) {
                    return None;
                }
                let same_path = loaded.package.object_path(ObjectRef::Export(index)).eq_ignore_ascii_case(key);
                let same_bare =
                    !exact_scope && !key.contains('.') && loaded.package.name(export.name).eq_ignore_ascii_case(key);
                (same_path || same_bare).then_some(index)
            })
        }
    }

    /// Export `index` of the package called `name`, with its bytes read into place.
    fn export(&mut self, name: &str, index: usize) -> Option<Found<'_>> {
        let loaded = self.packages.get_mut(name)?.as_mut()?;
        let export = loaded.package.exports().get(index)?;
        if let Some(window) = loaded.window {
            read_export(&loaded.path, window, export, &mut self.scratch).ok()?;
        } else if loaded.whole.is_none() {
            loaded.whole = Some(l2_crypto::decrypt_owned(fs::read(&loaded.path).ok()?, &loaded.path).ok()?);
        }
        let loaded = self.packages.get(name)?.as_ref()?;
        let file = match loaded.whole.as_deref() {
            Some(whole) => ue2_core::ByteWindow::from(whole),
            None => {
                ue2_core::ByteWindow { data: &self.scratch, base: loaded.package.exports().get(index)?.serial_offset }
            }
        };
        Some(Found { package: &loaded.package, file, index, objects: &loaded.objects })
    }

    /// Whether the package called `name` is indexed and readable, for messages about missing assets.
    pub fn package_status(&mut self, name: &str) -> PackageStatus {
        let name = name.to_lowercase();
        let Some(path) = self.files.get(&name).cloned() else { return PackageStatus::Missing };
        if self.package(&name).is_some() {
            return PackageStatus::Loaded;
        }
        if is_proprietary_container(&path) { PackageStatus::Encrypted } else { PackageStatus::Unreadable }
    }

    pub fn texture_lod(&mut self, path: &str) -> Option<(i32, i32)> {
        let resolved = self.static_texture_path(path)?;
        let found = self.object(&resolved, "Texture")?;
        let export = found.package.exports().get(found.index)?;
        let properties = ue2_assets::object_properties(found.package, found.file, export).ok()?;
        let minimum = ue2_assets::find(&properties, "MinLOD").and_then(ue2_assets::Property::int).unwrap_or(0);
        let texture = ue2_assets::read_texture(found.package, found.file, export).ok()?;
        let maximum = texture
            .mips
            .iter()
            .position(|mip| !(mip.width >= 64 && (mip.height >= 64 || mip.width >= 256 || mip.height >= 256)))
            .unwrap_or(texture.mips.len().saturating_sub(1));
        Some((minimum, maximum as i32))
    }

    /// Like `texture_raw`, but says why the texture could not be produced.
    pub fn try_texture_raw(&mut self, path: &str) -> Result<RawTexture, MissingAsset> {
        if self.object_class(path).is_some_and(|class| class.eq_ignore_ascii_case("WetTexture")) {
            return self.wet_texture(path).map(|wet| wet.raw()).ok_or(MissingAsset::Object);
        }
        let resolved = self.static_texture_path(path).ok_or(MissingAsset::Object)?;
        let path = resolved.as_str();
        let package_name = package_name(path).ok_or(MissingAsset::BadPath)?;
        let status = self.package_status(&package_name);
        if self.object(path, "Texture").is_none() {
            return Err(match status {
                PackageStatus::Missing => MissingAsset::Package,
                PackageStatus::Encrypted => MissingAsset::Encrypted,
                PackageStatus::Unreadable => MissingAsset::UnreadablePackage,
                PackageStatus::Loaded => match self.object_class(path) {
                    Some(class) => MissingAsset::WrongClass(class),
                    None => MissingAsset::Object,
                },
            });
        }
        let found = self.object(path, "Texture").ok_or(MissingAsset::Object)?;
        let export = found.package.exports().get(found.index).ok_or(MissingAsset::Object)?;
        let (version, licensee) = (found.package.version, found.package.licensee);
        ue2_assets::read_texture(found.package, found.file, export).map_err(|e| MissingAsset::Undecodable {
            version,
            licensee,
            error: format!("{e:?}"),
        })?;
        self.texture_raw(path).ok_or(MissingAsset::Object)
    }

    fn static_texture_path(&mut self, path: &str) -> Option<String> {
        let mut path = path.to_owned();
        let mut visited = Vec::<String>::new();
        while self.object_class(&path).is_some_and(|class| class.eq_ignore_ascii_case("WetTexture")) {
            if visited.iter().any(|previous| previous.eq_ignore_ascii_case(&path)) {
                return None;
            }
            visited.push(path.clone());
            let package_name = package_name(&path)?;
            let found = self.object(&path, "WetTexture")?;
            let export = found.package.exports().get(found.index)?;
            let properties = ue2_assets::object_properties(found.package, found.file, export).ok()?;
            let source = ue2_assets::find(&properties, "SourceTexture")?.object(found.package)?;
            path = full_path(&package_name, found.package, found.objects, source)?;
        }
        Some(path)
    }

    /// The class of the export at `path` from the package index alone, without reading its bytes.
    fn indexed_class(&mut self, path: &str) -> Option<String> {
        let path = path.to_lowercase();
        let (package_name, object) = path.split_once('.')?;
        let loaded = self.package(package_name)?;
        let bare = object.rsplit_once('.').map(|(_, name)| name);
        let index = *loaded.objects.get(object).or_else(|| loaded.objects.get(bare?))?;
        let export = loaded.package.exports().get(index)?;
        Some(loaded.package.class_name(export).to_string())
    }

    /// The package the class of the export at `path` is imported from, when the class is an import.
    fn imported_class_package(&mut self, path: &str) -> Option<(String, String)> {
        let path = path.to_lowercase();
        let (package_name, object) = path.split_once('.')?;
        let loaded = self.package(package_name)?;
        let bare = object.rsplit_once('.').map(|(_, name)| name);
        let index = *loaded.objects.get(object).or_else(|| loaded.objects.get(bare?))?;
        let export = loaded.package.exports().get(index)?;
        let ObjectRef::Import(class) = export.class else { return None };
        let import = loaded.package.imports().get(class)?;
        Some((loaded.package.name(import.name).to_owned(), loaded.package.object_path(import.outer)))
    }

    /// `StaticLoadObject` cannot build an object whose class `Engine.u` does not define.
    fn engine_defines(&mut self, class: &str) -> bool {
        if !self.files.contains_key("engine") {
            return true;
        }
        let key = class.to_ascii_lowercase();
        if let Some(known) = self.engine_classes.get(&key) {
            return *known;
        }
        let known = self.object(&format!("engine.{class}"), "Class").is_some();
        self.engine_classes.insert(key, known);
        known
    }

    /// The class of the export at `path`, when it exists in an indexed package.
    pub fn object_class(&mut self, path: &str) -> Option<String> {
        self.indexed_class(path)
    }

    pub fn default_material_path(&mut self) -> Option<String> {
        let found = self.object("Engine.Material", "Class")?;
        let export = found.package.exports().get(found.index)?;
        let properties = ue2_assets::class_default_properties(found.package, found.file, export).ok()?;
        let object = ue2_assets::find(&properties, "DefaultMaterial")?.object(found.package)?;
        full_path("Engine", found.package, found.objects, object)
    }

    /// The index of the package called `name`, read on first use.
    fn package(&mut self, name: &str) -> Option<&Loaded> {
        if !self.packages.contains_key(name) {
            let source = match name.strip_prefix('~') {
                Some(bare) => self.fallback.get(bare),
                None => self.files.get(name),
            };
            let loaded = source.and_then(|path| {
                load(path).or_else(|| if is_proprietary_container(path) { load(&standard_copy(path)?) } else { None })
            });
            self.packages.insert(name.to_owned(), loaded);
        }
        self.packages.get(name)?.as_ref()
    }
}

fn scan(client_root: &Path) -> HashMap<String, PathBuf> {
    let mut files = HashMap::new();
    let folders: Vec<PathBuf> =
        fs::read_dir(client_root).into_iter().flatten().flatten().map(|entry| entry.path()).collect();
    for (folder, extension) in FOLDERS {
        let found = folders.iter().filter(|path| path.file_name().is_some_and(|name| name.eq_ignore_ascii_case(folder)));
        for entry in found.flat_map(|path| fs::read_dir(path).into_iter().flatten().flatten()) {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
                && let Some(stem) = path.file_stem()
            {
                files.insert(stem.to_string_lossy().to_lowercase(), path);
            }
        }
    }
    files
}

fn read_export(path: &Path, window: Window, export: &Export, scratch: &mut Vec<u8>) -> std::io::Result<()> {
    export.serial_offset.checked_add(export.serial_size).ok_or(std::io::ErrorKind::InvalidData)?;
    scratch.resize(export.serial_size, 0);
    let range = scratch.as_mut_slice();
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(window.file_offset(export.serial_offset as u64)))?;
    file.read_exact(range)?;
    window.decrypt_in_place(range);
    Ok(())
}

fn merge_properties(into: &mut Vec<ue2_assets::NotifyProperty>, from: Vec<ue2_assets::NotifyProperty>) {
    for property in from {
        match into
            .iter_mut()
            .find(|current| current.index == property.index && current.name.eq_ignore_ascii_case(&property.name))
        {
            Some(current) => *current = property,
            None => into.push(property),
        }
    }
}

fn package_name(path: &str) -> Option<String> {
    path.split_once('.').map(|(package, _)| package.to_lowercase())
}

/// A reference read inside `package` as a client-wide path: imports already start with their
/// package, exports get the package's own name in front. An export whose `Group.Name` another
/// export of the package also uses is told apart by its index (`#n`), as the index does.
fn full_path(
    package_name: &str,
    package: &Package,
    objects: &HashMap<String, usize>,
    object: ObjectRef,
) -> Option<String> {
    match object {
        ObjectRef::Null => None,
        ObjectRef::Import(_) => Some(package.object_path(object)),
        ObjectRef::Export(index) => {
            let object_path = package.object_path(object);
            let unique = objects.get(&object_path.to_lowercase()) == Some(&index);
            Some(if unique {
                format!("{package_name}.{object_path}")
            } else {
                format!("{package_name}.{object_path}#{index}")
            })
        }
    }
}

struct PackageFile {
    file: File,
    window: Window,
}

impl Read for PackageFile {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let count = self.file.read(bytes)?;
        self.window.decrypt_in_place(&mut bytes[..count]);
        Ok(count)
    }
}

impl Seek for PackageFile {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        let from = match from {
            SeekFrom::Start(position) => {
                SeekFrom::Start(position.checked_add(self.window.file_offset(0)).ok_or(io::ErrorKind::InvalidInput)?)
            }
            other => other,
        };
        self.file.seek(from)?.checked_sub(self.window.file_offset(0)).ok_or_else(|| io::ErrorKind::InvalidInput.into())
    }
}

fn load(path: &Path) -> Option<Loaded> {
    let mut file = File::open(path).ok()?;
    let mut header = [0; 28];
    let count = usize::try_from(file.metadata().ok()?.len().min(header.len() as u64)).ok()?;
    file.read_exact(&mut header[..count]).ok()?;
    let window = Window::of(&header[..count], path).ok()?;
    let (package, whole) = match window {
        Some(window) => {
            let size = usize::try_from(window.payload_len(&mut file).ok()?).ok()?;
            let reader = BufReader::new(PackageFile { file, window });
            (Package::read(reader, size).ok()?, None)
        }
        None => {
            let file = l2_crypto::decrypt_owned(fs::read(path).ok()?, path).ok()?;
            (Package::parse(&file).ok()?, Some(file))
        }
    };
    let mut objects = HashMap::new();
    for index in 0..package.exports().len() {
        let object_path = package.object_path(ObjectRef::Export(index)).to_lowercase();
        objects.insert(format!("{object_path}#{index}"), index);
        objects.entry(object_path).or_insert(index);
    }
    for (index, export) in package.exports().iter().enumerate() {
        objects.entry(package.name(export.name).to_lowercase()).or_insert(index);
    }
    Some(Loaded { path: path.to_path_buf(), window, whole, package, objects })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn small_export_at_one_gib_reads_only_its_own_bytes() {
        let body = vec![0xEE; 1000];
        let (root, _) = client("sparse", &body);
        let path = root.join("Textures/alpha.utx");
        let (mut bytes, _) = package_bytes(&body);
        bytes.truncate(45 + 11 + compact(body.len()).len());
        let offset = 1 << 30;
        bytes.extend_from_slice(&compact(offset));
        let mut file = File::create(&path).unwrap();
        file.write_all(&bytes).unwrap();
        file.seek(SeekFrom::Start(offset as u64)).unwrap();
        file.write_all(&body).unwrap();
        drop(file);
        let mut catalog = Catalog::open(&root);
        assert_eq!(catalog.object_class("Alpha.Foo").as_deref(), Some("Class"));
        assert_eq!(catalog.cached_bytes(), 0);
        let found = catalog.object("Alpha.Foo", "").unwrap();
        assert_eq!(found.file.base, offset);
        assert_eq!(found.file.data, body);
        assert_eq!(found.file.reader().pos(), offset);
        assert_eq!(catalog.cached_bytes(), body.len());
        catalog.release_files();
        assert_eq!(catalog.cached_bytes(), 0);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore]
    fn real_package_indexes_and_every_export_match_whole_file_reads() {
        let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").unwrap());
        let mut catalog = Catalog::open(&root);
        for name in ["fighter", "lineageweapons", "l2ui_ch3"] {
            let path = catalog.files.get(name).unwrap().clone();
            let bytes = l2_crypto::decrypt_owned(fs::read(&path).unwrap(), &path).unwrap();
            let package = Package::parse(&bytes).unwrap();
            assert_eq!(catalog.package(name).unwrap().package, package, "{name}");
            let mut checked = 0;
            for (index, export) in package.exports().iter().enumerate() {
                let found = catalog.export(name, index).unwrap();
                assert_eq!(found.file.base, export.serial_offset);
                assert_eq!(
                    found.file.data,
                    &bytes[export.serial_offset..export.serial_offset + export.serial_size],
                    "{name}#{index}"
                );
                checked += 1;
            }
            eprintln!(
                "{name}: {checked} exports iguais; pacote={} bytes; buffer={} bytes",
                bytes.len(),
                catalog.cached_bytes()
            );
            catalog.release_files();
            assert_eq!(catalog.cached_bytes(), 0);
        }
    }

    fn compact(value: usize) -> Vec<u8> {
        let mut bytes = vec![(value & 0x3F) as u8 | if value >= 0x40 { 0x40 } else { 0 }];
        let mut rest = value >> 6;
        while rest > 0 {
            bytes.push((rest & 0x7F) as u8 | if rest >= 0x80 { 0x80 } else { 0 });
            rest >>= 7;
        }
        bytes
    }

    fn package_bytes(body: &[u8]) -> (Vec<u8>, usize) {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&ue2_package::TAG.to_le_bytes());
        bytes.extend_from_slice(&123u16.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        let names_at = 36usize;
        let exports_at = names_at + 9;
        let fixed = exports_at + 1 + 1 + 4 + 1 + 4 + compact(body.len()).len();
        let body_at = (fixed + 1..).find(|&at| fixed + compact(at).len() == at).unwrap();
        for value in [1, names_at, 1, exports_at, 0, 0] {
            bytes.extend_from_slice(&(value as u32).to_le_bytes());
        }
        bytes.push(4);
        bytes.extend_from_slice(b"Foo\0");
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&[0; 11]);
        bytes.extend_from_slice(&compact(body.len()));
        bytes.extend_from_slice(&compact(body_at));
        assert_eq!(bytes.len(), body_at);
        bytes.extend_from_slice(body);
        (bytes, body_at)
    }

    fn client(name: &str, body: &[u8]) -> (PathBuf, usize) {
        let root = std::env::temp_dir().join(format!("l2-catalog-{name}-{}", std::process::id()));
        let textures = root.join("Textures");
        fs::create_dir_all(&textures).unwrap();
        let (bytes, body_at) = package_bytes(body);
        fs::write(textures.join("alpha.utx"), bytes).unwrap();
        (root, body_at)
    }

    #[test]
    fn only_the_object_asked_for_is_read_and_release_drops_it() {
        let body = vec![0xEE; 1000];
        let (root, _) = client("release", &body);
        let mut catalog = Catalog::open(&root);
        assert_eq!(catalog.cached_bytes(), 0);
        assert!(catalog.texture_size("Alpha.Foo").is_none());
        assert_eq!(catalog.cached_bytes(), 0);
        assert!(catalog.material("Alpha.Foo").is_none());
        let end = 1000;
        assert_eq!(catalog.cached_bytes(), end);
        assert_eq!(catalog.scratch, body);
        catalog.release_files();
        assert_eq!(catalog.cached_bytes(), 0);
        assert!(catalog.material("Alpha.Foo").is_none());
        assert_eq!(catalog.cached_bytes(), end);
        assert!(catalog.texture("Alpha.Missing").is_none());
        assert!(catalog.texture("Nope.Foo").is_none());
    }

    #[test]
    fn the_index_is_kept_after_the_file_bytes_go() {
        let (root, _) = client("index", &[0; 10]);
        let mut catalog = Catalog::open(&root);
        assert!(catalog.material("Alpha.Foo").is_none());
        let loaded = catalog.packages.get("alpha").unwrap().as_ref().unwrap();
        assert_eq!(loaded.objects.get("foo"), Some(&0));
        assert!(loaded.whole.is_none());
        assert!(loaded.window.is_some());
    }

    #[test]
    fn explicitly_registered_packages_replace_negative_lookups_and_keep_export_indices() {
        let (root, _) = client("registered", &[0; 10]);
        let mut catalog = Catalog::open(&root);
        let source = catalog.files["alpha"].clone();
        assert!(catalog.object("Map.Foo", "").is_none());
        catalog.register_package("Map", &source);
        assert_eq!(catalog.object("Map.Foo#0", "").unwrap().index, 0);
        catalog.release_files();
        assert_eq!(catalog.object("Map.Foo#0", "").unwrap().index, 0);
    }

    #[test]
    fn a_cafef00d_package_that_does_not_decode_falls_back_to_a_standard_copy_beside_it() {
        let (root, _) = client("cafef00d", &[0; 10]);
        let textures = root.join("Textures");
        let (standard, _) = package_bytes(&[0; 10]);
        let mut container = l2_crypto::PROPRIETARY_CONTAINER_MAGIC.to_vec();
        container.extend_from_slice(&[0x6F, 0x00, 0xEC, 0x8B, 1, 2, 3, 4]);
        fs::write(textures.join("beta.utx"), &container).unwrap();
        let mut catalog = Catalog::open(&root);
        assert_eq!(catalog.package_status("Beta"), PackageStatus::Encrypted);
        assert_eq!(catalog.try_texture_raw("Beta.Foo").unwrap_err(), MissingAsset::Encrypted);
        fs::write(textures.join("beta.std.utx"), standard).unwrap();
        let mut catalog = Catalog::open(&root);
        assert_eq!(catalog.package_status("Beta"), PackageStatus::Loaded);
        assert!(catalog.object("Beta.Foo", "").is_some());
    }

    #[test]
    fn a_group_that_the_package_does_not_have_falls_back_to_the_bare_name() {
        let (root, _) = client("group", &[0; 10]);
        let mut catalog = Catalog::open(&root);
        assert!(catalog.object("Alpha.Etc.Foo", "").is_some());
        assert!(catalog.object("Alpha.Etc.Bar", "").is_none());
        assert!(catalog.object("Alpha.Foo", "").is_some());
    }
}

#[cfg(test)]
mod collision_tests {
    use super::*;

    #[test]
    #[ignore]
    fn static_find_object_1015e550_filters_class_before_accepting_all_static_mesh_exports() {
        let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let mut catalog = Catalog::open(&root);
        let paths: Vec<_> = catalog
            .source_paths()
            .filter(|path| path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("usx")))
            .map(Path::to_path_buf)
            .collect();
        let mut total = 0;
        let mut hidden = Vec::new();
        for path in &paths {
            let name = path.file_stem().unwrap().to_str().unwrap().to_lowercase();
            let loaded = catalog.package(&name).unwrap();
            let mut expected = std::collections::BTreeMap::new();
            for (index, export) in loaded.package.exports().iter().enumerate() {
                if loaded.package.class_name(export).eq_ignore_ascii_case("StaticMesh") {
                    let object = loaded.package.object_path(ObjectRef::Export(index)).to_lowercase();
                    expected.entry(object).or_insert(index);
                }
            }
            for (object, index) in expected {
                let reference = format!("{name}.{object}");
                let first = catalog.object(&reference, "").unwrap().index;
                assert_eq!(catalog.object(&reference, "StaticMesh").unwrap().index, index, "{reference}");
                assert_eq!(
                    catalog.object(&reference.to_uppercase(), "staticmesh").unwrap().index,
                    index,
                    "{reference}"
                );
                assert_eq!(catalog.object(&format!("{reference}#{index}"), "StaticMesh").unwrap().index, index);
                if first != index {
                    assert!(catalog.object(&format!("{reference}#{first}"), "StaticMesh").is_none());
                    let mesh = catalog.static_mesh(&reference).unwrap();
                    let indexed = catalog.static_mesh(&format!("{reference}#{index}")).unwrap();
                    assert_eq!(mesh.mesh, indexed.mesh, "{reference}");
                    assert_eq!(mesh.materials, indexed.materials, "{reference}");
                    let collision = catalog.static_mesh_collision(&reference).unwrap();
                    assert_eq!(collision.positions, mesh.mesh.positions);
                    hidden.push((reference, first, index, mesh.mesh.positions.len(), mesh.mesh.indices.len()));
                }
                total += 1;
            }
            catalog.packages.clear();
            catalog.release_files();
        }
        assert!(total > 0);
        for name in [
            "sp_lighthouse.sp_lighthouse001",
            "sp_lighthouse.sp_lighthouse002",
            "gludio_dungeon_enterance_s.gludio_dungeon_enterance01",
            "gludio_dungeon_enterance_s.gludio_dungeon_enterance03",
        ] {
            assert!(hidden.iter().any(|entry| entry.0 == name), "{name}");
        }
        eprintln!(
            "{} pacotes, {total} caminhos StaticMesh, {} homônimos recuperados: {hidden:?}",
            paths.len(),
            hidden.len()
        );
    }

    #[test]
    #[ignore]
    fn all_installed_static_mesh_collision_arrays_preserve_exact_serialized_ranges() {
        let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
        let catalog = Catalog::open(&root);
        let mut totals = [0_usize; 6];
        let mut scratch = Vec::new();
        for path in
            catalog.source_paths().filter(|path| path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("usx")))
        {
            let loaded = load(path).expect("pacote estático legível");
            let mut meshes = 0;
            for export in loaded
                .package
                .exports()
                .iter()
                .filter(|export| loaded.package.class_name(export).eq_ignore_ascii_case("StaticMesh"))
            {
                let bytes = if let Some(window) = loaded.window {
                    read_export(path, window, export, &mut scratch).unwrap();
                    ue2_core::ByteWindow { data: &scratch, base: export.serial_offset }
                } else {
                    ue2_core::ByteWindow::from(loaded.whole.as_ref().unwrap())
                };
                let mesh = ue2_assets::read_static_mesh(&loaded.package, bytes, export).unwrap_or_else(|error| {
                    panic!("{}: {}: {error}", path.display(), loaded.package.name(export.name))
                });
                let collision = mesh.collision;
                totals[4] += usize::from(collision.model != ObjectRef::Null);
                if collision.model != ObjectRef::Null {
                    eprintln!(
                        "Modelo simples: {}.{} -> {}",
                        path.file_stem().unwrap().to_string_lossy(),
                        loaded.package.name(export.name),
                        loaded.package.object_path(collision.model)
                    );
                }
                totals[5] += usize::from(!collision.simple_box);
                let triangles = collision.read_triangles(bytes).unwrap();
                let nodes = collision.read_nodes(bytes).unwrap();
                for triangle in &triangles {
                    assert!(
                        triangle.indices[..3].iter().all(|&i| i >= 0 && (i as usize) < mesh.positions.len()),
                        "{}: vértice de colisão",
                        loaded.package.name(export.name)
                    );
                }
                for node in &nodes {
                    assert!(
                        node.indices[0] >= 0 && (node.indices[0] as usize) < triangles.len(),
                        "{}: face do nó",
                        loaded.package.name(export.name)
                    );
                    assert!(
                        node.indices[1..].iter().all(|&i| i == -1 || (i >= 0 && (i as usize) < nodes.len())),
                        "{}: filho do nó",
                        loaded.package.name(export.name)
                    );
                }
                totals[2] += triangles.len();
                totals[3] += nodes.len();
                meshes += 1;
            }
            totals[0] += 1;
            totals[1] += meshes;
        }
        assert!(totals[..4].iter().all(|count| *count > 0));
        eprintln!(
            "Colisão: {} pacotes, {} meshes, {} triângulos, {} nós, {} modelos simples, {} meshes sem simple box",
            totals[0], totals[1], totals[2], totals[3], totals[4], totals[5]
        );
    }
}
