//! `TerrainInfo` actors: a heightmap placed and scaled in the world, painted by blended layers.

use ue2_assets::{Property, find, object_data};
use ue2_core::Reader;
use ue2_package::{ObjectRef, Package};

use crate::Range;
use crate::emitters::{range_of, range_vector};
use crate::lighting::{self, TerrainSector};

const LICENSEE_NATIVE_VERTICES: u16 = 17;
const VERSION_VERTEX_COLORS: u16 = 0x74;
const VERSION_OLD_HEIGHTMAP: u16 = 0x4c;
const VERSION_OLD_LAYERS_LOW: u16 = 0x4a;
const VERSION_OLD_LAYERS_HIGH: u16 = 0x52;
const VERSION_DUMMY_ARRAY: u16 = 0x53;

/// A scale-and-translate frame as `FCoords` stores it: `TransformPointBy(p) = ((p − origin)·x, (p − origin)·y, (p − origin)·z)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coords {
    pub origin: [f32; 3],
    pub x_axis: [f32; 3],
    pub y_axis: [f32; 3],
    pub z_axis: [f32; 3],
}

impl Coords {
    pub fn transform_point(&self, point: [f32; 3]) -> [f32; 3] {
        let d = [point[0] - self.origin[0], point[1] - self.origin[1], point[2] - self.origin[2]];
        let dot = |axis: [f32; 3]| axis[0] * d[0] + axis[1] * d[1] + axis[2] * d[2];
        [dot(self.x_axis), dot(self.y_axis), dot(self.z_axis)]
    }
}

/// What `ATerrainInfo::Serialize` writes after the properties.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TerrainNative {
    pub vertices: Vec<[f32; 3]>,
    pub face_normals: Vec<[[f32; 3]; 2]>,
    pub sectors_x: i32,
    pub sectors_y: i32,
    pub to_world: Option<Coords>,
    pub to_heightmap: Option<Coords>,
    pub heightmap_x: i32,
    pub heightmap_y: i32,
    /// One color per heightmap sample, red first.
    pub vertex_colors: Vec<[u8; 4]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Terrain {
    pub name: String,
    pub base_height: i32,
    /// World position of the heightmap's center.
    pub location: [f32; 3],
    /// World units per heightmap sample on X and Y, and per 256 height steps on Z.
    pub scale: [f32; 3],
    /// Object path of the 16-bit heightmap texture.
    pub heightmap: String,
    /// Painted layers, bottom first.
    pub layers: Vec<TerrainLayer>,
    /// One bit per quad, row by row from the lowest bit of the first word; a clear bit is a hole.
    pub visible_quads: Vec<u32>,
    /// One bit per quad in the same layout; a set bit turns the quad's diagonal.
    pub edge_turn: Vec<u32>,
    /// Negates vertex normals.
    pub inverted: bool,
    /// Precomputed lighting, patch by patch.
    pub sectors: Vec<TerrainSector>,
    /// Static meshes scattered over the terrain, such as grass.
    pub deco_layers: Vec<DecoLayer>,
    /// Lifts each decoration along its normal, in world units.
    pub deco_layer_offset: f32,
    pub native: TerrainNative,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TerrainLayer {
    /// Object path of the material the layer paints with.
    pub material: String,
    /// Object path of the texture whose red channel weighs the layer; `None` paints everywhere.
    pub alpha_map: Option<String>,
    /// Heightmap samples covered by one repeat of the material.
    pub scale: [f32; 2],
    pub pan: [f32; 2],
    pub texture_map_axis: u8,
    pub texture_rotation: i32,
    pub layer_rotation: [i32; 3],
}

/// A static mesh scattered over the quads a density map paints.
#[derive(Debug, Clone, PartialEq)]
pub struct DecoLayer {
    /// Object path of the scattered static mesh.
    pub static_mesh: String,
    /// Object path of the texture whose red channel weighs how many land on each quad.
    pub density_map: String,
    /// Multiplies the density map, picked per placement.
    pub density_multiplier: Range,
    pub max_per_quad: u32,
    /// Scale on X, Y and Z.
    pub scale: [Range; 3],
    /// Distance from the camera where decorations begin and finish fading out.
    pub fadeout_radius: Range,
    pub seed: i32,
    pub random_yaw: bool,
    /// Also placed on quads that are holes.
    pub on_invisible_terrain: bool,
    /// Stands along the quad's normal instead of straight up.
    pub align_to_terrain: bool,
    /// Multiplies the scale by a texel of this texture.
    pub scale_map: Option<String>,
    /// 0 draws in sector order, 1 far to near, 2 near to far.
    pub draw_order: u8,
    pub lit_directional: bool,
    /// Drawn even when the client's `RenderDeco` is off.
    pub force_render: bool,
}

pub(crate) fn read(package: &Package, file: &[u8], export: usize, properties: &[Property<'_>], package_name: Option<&str>) -> Option<Terrain> {
    let path = |property: &Property<'_>| match property.object(package)? {
        ObjectRef::Null => None,
        ObjectRef::Export(index) if package_name.is_some() => package_name.map(|name| format!("{name}.{}#{index}", package.object_path(ObjectRef::Export(index)))),
        object => Some(package.object_path(object)),
    };
    let layers = properties
        .iter()
        .filter(|property| property.name.eq_ignore_ascii_case("Layers"))
        .filter_map(|layer| {
            let fields = layer.fields(package)?;
            let float = |name| find(&fields, name).and_then(Property::float).unwrap_or(1.0);
            let float0 = |name| find(&fields, name).and_then(Property::float).unwrap_or(0.0);
            Some(TerrainLayer {
                material: find(&fields, "Texture").and_then(path)?,
                alpha_map: find(&fields, "AlphaMap").and_then(path),
                scale: [float("UScale"), float("VScale")],
                pan: [float0("UPan"), float0("VPan")],
                texture_map_axis: find(&fields, "TextureMapAxis").and_then(Property::byte).unwrap_or(0),
                texture_rotation: find(&fields, "TextureRotation").and_then(Property::int).unwrap_or(0),
                layer_rotation: find(&fields, "LayerRotation").and_then(Property::rotator).unwrap_or_default(),
            })
        })
        .collect();
    Some(Terrain {
        name: package.object_name(ObjectRef::Export(export)).to_owned(),
        base_height: find(properties, "BaseHeight").and_then(Property::int).unwrap_or(0),
        location: find(properties, "Location").and_then(Property::vector).unwrap_or_default(),
        scale: find(properties, "TerrainScale").and_then(Property::vector)?,
        heightmap: find(properties, "TerrainMap").and_then(path)?,
        layers,
        visible_quads: find(properties, "QuadVisibilityBitmap")
            .and_then(Property::ints)
            .map(|words| words.into_iter().map(i32::cast_unsigned).collect())
            .unwrap_or_default(),
        edge_turn: find(properties, "EdgeTurnBitmap")
            .and_then(Property::ints)
            .map(|words| words.into_iter().map(i32::cast_unsigned).collect())
            .unwrap_or_default(),
        inverted: find(properties, "Inverted").and_then(Property::bool).unwrap_or(false),
        deco_layer_offset: find(properties, "DecoLayerOffset").and_then(Property::float).unwrap_or(0.0),
        native: package.exports().get(export).and_then(|export| native(package, file, export)).unwrap_or_default(),
        // Unlike Layers, a dynamic array.
        deco_layers: find(properties, "DecoLayers")
            .and_then(|layers| layers.structs(package))
            .unwrap_or_default()
            .iter()
            .filter_map(|layer| deco_layer(package, layer, path))
            .collect(),
        // Sectors are objects inside the terrain actor.
        sectors: package
            .exports()
            .iter()
            .filter(|sector| {
                sector.outer == ObjectRef::Export(export)
                    && package.class_name(sector).eq_ignore_ascii_case("TerrainSector")
            })
            .filter_map(|sector| lighting::sector(package, file, sector))
            .collect(),
    })
}

fn native(package: &Package, file: &[u8], export: &ue2_package::Export) -> Option<TerrainNative> {
    let (_, data) = object_data(package, file, export).ok()?;
    let mut reader = Reader::at(data, 0);
    let reader = &mut reader;
    let count = usize::try_from(reader.compact().ok()?).ok()?;
    for _ in 0..count {
        reader.compact().ok()?;
    }
    let mut native = TerrainNative::default();
    if package.licensee < LICENSEE_NATIVE_VERTICES {
        let vertices = usize::try_from(reader.compact().ok()?).ok()?;
        reader.bytes(vertices.checked_mul(12)?).ok()?.as_chunks::<12>().0.iter().for_each(|bytes| {
            native.vertices.push(std::array::from_fn(|i| f32::from_le_bytes([bytes[i * 4], bytes[i * 4 + 1], bytes[i * 4 + 2], bytes[i * 4 + 3]])));
        });
        native.sectors_x = reader.i32().ok()?;
        native.sectors_y = reader.i32().ok()?;
        let normals = usize::try_from(reader.compact().ok()?).ok()?;
        reader.bytes(normals.checked_mul(24)?).ok()?.as_chunks::<24>().0.iter().for_each(|bytes| {
            native.face_normals.push(std::array::from_fn(|face| std::array::from_fn(|axis| {
                let i = face * 12 + axis * 4;
                f32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]])
            })));
        });
    } else {
        native.sectors_x = reader.i32().ok()?;
        native.sectors_y = reader.i32().ok()?;
    }
    if package.version < VERSION_DUMMY_ARRAY {
        let dummy = usize::try_from(reader.compact().ok()?).ok()?;
        reader.bytes(dummy).ok()?;
    }
    let coords = |reader: &mut Reader<'_>| -> Option<Coords> {
        let mut vectors = [[0.0; 3]; 4];
        for vector in &mut vectors {
            for value in vector.iter_mut() {
                *value = reader.f32().ok()?;
            }
        }
        Some(Coords { origin: vectors[0], x_axis: vectors[1], y_axis: vectors[2], z_axis: vectors[3] })
    };
    native.to_world = Some(coords(reader)?);
    native.to_heightmap = Some(coords(reader)?);
    if package.version < VERSION_OLD_HEIGHTMAP {
        let count = usize::try_from(reader.compact().ok()?).ok()?;
        reader.bytes(count.checked_mul(2)?).ok()?;
    }
    native.heightmap_x = reader.i32().ok()?;
    native.heightmap_y = reader.i32().ok()?;
    if package.version > VERSION_OLD_LAYERS_LOW && package.version < VERSION_OLD_LAYERS_HIGH {
        return Some(native);
    }
    if package.version > VERSION_VERTEX_COLORS {
        let count = usize::try_from(reader.compact().ok()?).ok()?;
        let colors = reader.bytes(count.checked_mul(4)?).ok()?;
        native.vertex_colors = colors.as_chunks::<4>().0.to_vec();
    }
    Some(native)
}

fn deco_layer(
    package: &Package,
    fields: &[Property<'_>],
    path: impl Fn(&Property<'_>) -> Option<String>,
) -> Option<DecoLayer> {
    // The editor stores these flags as integers.
    let int = |name| find(fields, name).and_then(|field| field.int().or_else(|| field.byte().map(i32::from)));
    if int("ShowOnTerrain") == Some(0) {
        return None;
    }
    Some(DecoLayer {
        static_mesh: find(fields, "StaticMesh").and_then(&path)?,
        density_map: find(fields, "DensityMap").and_then(&path)?,
        density_multiplier: find(fields, "DensityMultiplier").map_or([1.0; 2], |range| range_of(package, range)),
        max_per_quad: u32::try_from(int("MaxPerQuad").unwrap_or(0)).unwrap_or(0),
        scale: find(fields, "ScaleMultiplier").map_or([[1.0; 2]; 3], |ranges| range_vector(package, ranges)),
        fadeout_radius: find(fields, "FadeoutRadius").map_or([0.0; 2], |range| range_of(package, range)),
        seed: int("Seed").unwrap_or(0),
        random_yaw: int("RandomYaw").unwrap_or(0) != 0,
        on_invisible_terrain: int("ShowOnInvisibleTerrain").unwrap_or(0) != 0,
        align_to_terrain: int("AlignToTerrain").unwrap_or(0) != 0,
        scale_map: find(fields, "ScaleMap").and_then(&path),
        draw_order: u8::try_from(int("DrawOrder").unwrap_or(0)).unwrap_or(0),
        lit_directional: int("LitDirectional").unwrap_or(0) != 0,
        force_render: int("bForceRender").unwrap_or(0) != 0,
    })
}
