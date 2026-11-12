//! A level's BSP (its `Model`): the walls, floors and other brush geometry, as polygons with the material
//! and texture axes of the surface each lies on, plus the node tree and the zones it partitions space into.

use ue2_assets::{Error, object_data};
use ue2_core::Reader;
use ue2_package::{Export, ObjectRef, Package};

/// Polygon flags whose surfaces are never drawn: invisible, portal, sky backdrop, and one Lineage II uses
/// for hidden surfaces.
pub const HIDDEN: u32 = 0x0000_0001 | 0x0400_0000 | 0x0000_0080;
pub const PF_L2WATER: u32 = 0x0008_0000;
const VERSION_SECTIONS: u16 = 0x69;
const LICENSEE_TIME_LIGHTMAPS: u16 = 9;
const LICENSEE_SURF_EXTRA: u16 = 23;

#[derive(Debug, Clone, PartialEq)]
pub struct BspPolygon {
    pub node_index: usize,
    /// Object path of the material, when set.
    pub material: Option<String>,
    /// Convex polygon corners in world space, fanned from the first.
    pub corners: Vec<[f32; 3]>,
    /// Where texture coordinates are zero, and the texture's U and V axes; a corner's texel coordinate on
    /// an axis is its offset from the origin dotted with that axis.
    pub texture_origin: [f32; 3],
    pub texture_axes: [[f32; 3]; 2],
    /// Zone on the front side of the polygon's node, where it is seen from.
    pub zone: u8,
    /// The surface's `PolyFlags` as stored.
    pub flags: u32,
    /// Normal of the node's plane, the side the polygon faces.
    pub normal: [f32; 3],
}

/// A node of the BSP tree; child, leaf and zone references are kept as stored, -1 meaning none.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BspNode {
    pub flags: u8,
    pub vertices: u8,
    pub coplanar: i32,
    pub collision_bound: i32,
    pub render_bound: i32,
    pub surface: i32,
    /// Normal and distance: a point `p` is in front when `dot(plane.xyz, p) >= plane.w`.
    pub plane: [f32; 4],
    pub front: i32,
    pub back: i32,
    pub zone_front: u8,
    pub zone_back: u8,
    pub leaf_front: i32,
    pub leaf_back: i32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BspTree {
    pub nodes: Vec<BspNode>,
}

/// A vertex of a section's stream: texture and lightmap coordinates already computed by the editor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BspVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub lightmap_uv: [f32; 2],
    pub normal: [f32; 3],
}

/// The vertices the driver draws for a run of nodes sharing a material and flags.
#[derive(Debug, Clone, PartialEq)]
pub struct BspSection {
    pub vertices: Vec<BspVertex>,
    pub num_nodes: u32,
    pub material: Option<String>,
    pub poly_flags: u32,
    /// Index into `groups`; -1 for unlit sections.
    pub light_map_texture: i32,
}

/// Where a node's fan lives in its section; kept as stored, -1 meaning none.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeDraw {
    pub section: i32,
    pub first_vertex: i32,
    pub num_vertices: u8,
    pub light_map: i32,
    /// `PolyFlags` of the node's surface; nodes whose surface has a `HIDDEN` bit are not drawn.
    pub surface_flags: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightMap {
    pub texture: i32,
    pub surface: i32,
    pub zone: i32,
    pub offset: [i32; 2],
    pub size: [i32; 2],
    pub world_to_lightmap: [[f32; 4]; 4],
    pub base: [f32; 3],
    pub axis_x: [f32; 3],
    pub axis_y: [f32; 3],
    pub bitmaps: Vec<LightBitmapRecord>,
    pub revision: i32,
}

/// One light's shadow bits over a lightmap, as `FLightBitmap` is stored: one bit per texel in rows of `stride` bytes,
/// covering `min..=max`. The slice a bitmap belongs to is not stored.
#[derive(Debug, Clone, PartialEq)]
pub struct LightBitmapRecord {
    pub actor: Option<String>,
    pub bits: Vec<u8>,
    pub size: [i32; 2],
    pub stride: i32,
    pub min: [i32; 2],
    pub max: [i32; 2],
}

/// What a lightmap reads from its `FBspSurf`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderSurface {
    pub flags: u32,
    pub plane: [f32; 4],
}

/// One time slice of a lightmap atlas, DXT1 as stored.
#[derive(Debug, Clone, PartialEq)]
pub struct StaticLightMap {
    pub width: u32,
    pub height: u32,
    pub format: u8,
    pub mips: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightMapGroup {
    pub textures: Vec<StaticLightMap>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Leaf {
    pub zone: i32,
    pub permeating: i32,
    pub volumetric: i32,
}

/// What the client renders the model from: sections, lightmaps and their time slices.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BspRender {
    pub collision_bounds: Vec<([[f32; 3]; 2], u8)>,
    pub leaf_hulls: Vec<i32>,
    pub root_outside: bool,
    pub linked: bool,
    pub sections: Vec<BspSection>,
    /// One entry per node of the tree.
    pub node_draws: Vec<NodeDraw>,
    pub light_maps: Vec<LightMap>,
    pub surfaces: Vec<RenderSurface>,
    pub groups: Vec<LightMapGroup>,
    pub leaves: Vec<Leaf>,
    /// Light actors by leaf, runs ended by `None`.
    pub lights: Vec<Option<String>>,
    /// Whether the reader consumed the object to its last byte.
    pub complete: bool,
}

impl BspTree {
    /// The zone a point lies in, walking from the root; 0 is solid space or a level without zones.
    pub fn zone_at(&self, point: [f32; 3]) -> u8 {
        let mut zone = 0;
        let mut next = 0;
        for _ in 0..self.nodes.len() {
            let Some(node) = get(&self.nodes, next) else { break };
            let [x, y, z, w] = node.plane;
            let in_front = x * point[0] + y * point[1] + z * point[2] >= w;
            (zone, next) = if in_front { (node.zone_front, node.front) } else { (node.zone_back, node.back) };
        }
        zone
    }
}

/// A zone of the model, named by the `ZoneInfo` actor placed in it; none for solid space and zones
/// left to the level's defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct Zone {
    pub actor: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BspModel {
    pub bounding_box: [[f32;3];2],
    pub polygons: Vec<BspPolygon>,
    pub tree: BspTree,
    pub zones: Vec<Zone>,
    pub render: BspRender,
}

/// References are kept as stored: -1 means none.
struct Node {
    first_vertex: i32,
    surface: i32,
    vertices: usize,
    tree: BspNode,
    draw: NodeDraw,
}

struct Surface {
    material: ObjectRef,
    flags: u32,
    origin: i32,
    axes: [i32; 2],
    plane: [f32; 4],
}

/// The drawable polygons, tree and zones of a `Model` export; `None` when it holds no geometry, as
/// brush models do.
pub fn read<'a>(package: &'a Package, file: impl Into<ue2_core::ByteWindow<'a>>, export: &Export) -> Result<Option<BspModel>, Error> {
    let (_, data) = object_data(package, file, export)?;
    let mut reader = Reader::at(data, 0);
    let reader = &mut reader;
    let bounding_box = [vector(reader)?,vector(reader)?];
    reader.bytes(1 + 16)?;
    let vectors = array(reader, vector)?;
    let points = array(reader, vector)?;
    let nodes = array(reader, |reader| {
        let plane = [reader.f32()?, reader.f32()?, reader.f32()?, reader.f32()?];
        reader.bytes(8)?;
        let flags = reader.u8()?;
        let first_vertex = reader.compact()?;
        let surface = reader.compact()?;
        let back = reader.compact()?;
        let front = reader.compact()?;
        let coplanar = reader.compact()?;
        let collision_bound = reader.compact()?;
        let render_bound = reader.compact()?;
        reader.bytes(32)?; // bounding spheres
        let zone_back = reader.u8()?;
        let zone_front = reader.u8()?;
        let vertices = usize::from(reader.u8()?);
        let leaf_back = reader.i32()?;
        let leaf_front = reader.i32()?;
        let section = reader.i32()?;
        let section_vertex = reader.i32()?;
        let light_map = reader.i32()?;
        let tree = BspNode { flags, vertices: vertices as u8, coplanar, collision_bound, render_bound, surface, plane, front, back, zone_front, zone_back, leaf_front, leaf_back };
        let draw = NodeDraw { section, first_vertex: section_vertex, num_vertices: u8::try_from(vertices).unwrap_or(u8::MAX), light_map, surface_flags: 0 };
        Ok(Node { first_vertex, surface, vertices, tree, draw })
    })?;
    let lineage = package.licensee >= LICENSEE_SURF_EXTRA;
    let surfaces = array(reader, |reader| {
        let material = package.object_at(reader.compact()?)?;
        let flags = reader.u32()?;
        let origin = reader.compact()?;
        reader.compact()?; // normal
        let axes = [reader.compact()?, reader.compact()?];
        reader.compact()?; // brush polygon
        reader.compact()?; // brush actor
        let plane = [reader.f32()?, reader.f32()?, reader.f32()?, reader.f32()?];
        reader.bytes(4)?; // light map scale
        if lineage {
            reader.i32()?;
        }
        Ok(Surface { material, flags, origin, axes, plane })
    })?;
    let vertices = array(reader, |reader| {
        let point = reader.compact()?;
        reader.compact()?; // side
        Ok(point)
    })?;
    reader.i32()?; // shared sides
    let zones = usize::try_from(reader.i32()?).map_err(|_| Error::BadMesh)?;
    if zones > usize::from(u8::MAX) + 1 {
        return Err(Error::BadMesh);
    }
    let zones = (0..zones)
        .map(|_| {
            let actor = package.object_at(reader.compact()?)?;
            reader.bytes(8 + 8 + 4)?; // connectivity, visibility and last render time
            Ok(Zone { actor: (!matches!(actor, ObjectRef::Null)).then(|| package.object_name(actor).to_owned()) })
        })
        .collect::<Result<_, Error>>()?;

    // A polygon whose references lead nowhere is left out rather than failing the level.
    let polygons = nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| node.vertices >= 3)
        .filter_map(|(node_index, node)| {
            let surface = get(&surfaces, node.surface)?;
            if surface.flags & HIDDEN != 0 {
                return None;
            }
            let first = usize::try_from(node.first_vertex).ok()?;
            let corners = vertices
                .get(first..first + node.vertices)?
                .iter()
                .map(|&point| get(&points, point).copied())
                .collect::<Option<_>>()?;
            Some(BspPolygon {
                node_index,
                material: (!matches!(surface.material, ObjectRef::Null)).then(|| package.object_path(surface.material)),
                corners,
                texture_origin: *get(&points, surface.origin)?,
                texture_axes: [*get(&vectors, surface.axes[0])?, *get(&vectors, surface.axes[1])?],
                zone: node.tree.zone_front,
                flags: surface.flags,
                normal: [node.tree.plane[0], node.tree.plane[1], node.tree.plane[2]],
            })
        })
        .collect();
    let tree = BspTree { nodes: nodes.iter().map(|node| node.tree).collect() };
    let mut render = read_render(package, reader)?;
    render.surfaces = surfaces.iter().map(|surface| RenderSurface { flags: surface.flags, plane: surface.plane }).collect();
    render.node_draws = nodes
        .iter()
        .map(|node| NodeDraw { surface_flags: get(&surfaces, node.surface).map_or(HIDDEN, |surface| surface.flags), ..node.draw })
        .collect();
    Ok(Some(BspModel { bounding_box, polygons, tree, zones, render }))
}

fn read_render(package: &Package, reader: &mut Reader<'_>) -> Result<BspRender, Error> {
    reader.compact()?;
    let collision_bounds = array(reader, |reader| Ok(([vector(reader)?, vector(reader)?], reader.u8()?)))?;
    let leaf_hulls = array(reader, |reader| Ok(reader.i32()?))?;
    let leaves = array(reader, |reader| {
        let zone = reader.compact()?;
        let permeating = reader.compact()?;
        let volumetric = reader.compact()?;
        reader.bytes(8)?;
        Ok(Leaf { zone, permeating, volumetric })
    })?;
    let lights = array(reader, |reader| {
        let actor = package.object_at(reader.compact()?)?;
        Ok((!matches!(actor, ObjectRef::Null)).then(|| package.object_name(actor).to_owned()))
    })?;
    let root_outside = reader.i32()? != 0;
    let linked = reader.i32()? != 0;
    if package.version < VERSION_SECTIONS {
        return Err(Error::BadMesh);
    }
    let sections = array(reader, |reader| {
        let vertices = array(reader, |reader| {
            let position = vector(reader)?;
            let uv = [reader.f32()?, reader.f32()?];
            let lightmap_uv = [reader.f32()?, reader.f32()?];
            let normal = vector(reader)?;
            Ok(BspVertex { position, uv, lightmap_uv, normal })
        })?;
        let num_nodes = reader.u32()?;
        let material = package.object_at(reader.compact()?)?;
        reader.i32()?;
        let poly_flags = reader.u32()?;
        let light_map_texture = reader.i32()?;
        Ok(BspSection {
            vertices,
            num_nodes,
            material: (!matches!(material, ObjectRef::Null)).then(|| package.object_path(material)),
            poly_flags,
            light_map_texture,
        })
    })?;
    let light_maps = array(reader, |reader| {
        let texture = reader.compact()?;
        let surface = reader.compact()?;
        let zone = reader.compact()?;
        let offset = [reader.compact()?, reader.compact()?];
        let size = [reader.compact()?, reader.compact()?];
        let mut world_to_lightmap = [[0.0; 4]; 4];
        for row in &mut world_to_lightmap {
            for value in row.iter_mut() {
                *value = reader.f32()?;
            }
        }
        let base = vector(reader)?;
        let axis_x = vector(reader)?;
        let axis_y = vector(reader)?;
        let bitmaps = array(reader, |reader| {
            let actor = package.object_at(reader.compact()?)?;
            let bits = array(reader, |reader| Ok(reader.u8()?))?;
            let mut ints = [0; 7];
            for value in &mut ints {
                *value = reader.i32()?;
            }
            Ok(LightBitmapRecord {
                actor: (!matches!(actor, ObjectRef::Null)).then(|| package.object_name(actor).to_owned()),
                bits,
                size: [ints[0], ints[1]],
                stride: ints[2],
                min: [ints[3], ints[4]],
                max: [ints[5], ints[6]],
            })
        })?;
        reader.compact()?;
        let revision = reader.i32()?;
        Ok(LightMap { texture, surface, zone, offset, size, world_to_lightmap, base, axis_x, axis_y, bitmaps, revision })
    })?;
    let texture = |reader: &mut Reader<'_>| {
        reader.compact()?;
        array(reader, |reader| Ok(reader.i32()?))?;
        reader.bytes(8)?;
        reader.i32()?;
        let mut mips = Vec::with_capacity(2);
        for _ in 0..2 {
            reader.i32()?;
            mips.push(array(reader, |reader| Ok(reader.u8()?))?);
        }
        let format = reader.u8()?;
        let width = reader.u32()?;
        let height = reader.u32()?;
        reader.i32()?;
        Ok(StaticLightMap { width, height, format, mips })
    };
    let groups = if package.licensee < LICENSEE_TIME_LIGHTMAPS {
        array(reader, texture)?.into_iter().map(|texture| LightMapGroup { textures: vec![texture] }).collect()
    } else {
        array(reader, |reader| {
            let textures = array(reader, texture)?;
            array(reader, |reader| Ok(reader.i32()?))?;
            Ok(LightMapGroup { textures })
        })?
    };
    let complete = reader.remaining().is_empty();
    Ok(BspRender { collision_bounds, leaf_hulls, root_outside, linked, sections, node_draws: Vec::new(), light_maps, surfaces: Vec::new(), groups, leaves, lights, complete })
}

fn get<T>(items: &[T], index: i32) -> Option<&T> {
    items.get(usize::try_from(index).ok()?)
}

fn vector(reader: &mut Reader<'_>) -> Result<[f32; 3], Error> {
    Ok([reader.f32()?, reader.f32()?, reader.f32()?])
}

fn array<'a, T>(
    reader: &mut Reader<'a>,
    mut item: impl FnMut(&mut Reader<'a>) -> Result<T, Error>,
) -> Result<Vec<T>, Error> {
    let len = usize::try_from(reader.compact()?).map_err(|_| Error::BadMesh)?;
    if len > reader.remaining().len() {
        return Err(Error::BadMesh);
    }
    (0..len).map(|_| item(reader)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn floats(values: &[f32]) -> Vec<u8> {
        values.iter().flat_map(|value| value.to_le_bytes()).collect()
    }

    /// A model with a triangle on a node that splits at z = 0 with zone 1 in front and 2 behind, a node
    /// whose surface is missing, one surface, and three zones.
    fn model() -> Vec<u8> {
        let mut data = vec![0]; // properties: None
        data.extend([0; 41]);
        data.push(2);
        data.extend(floats(&[0.5, 0.0, 0.0, 0.0, 0.25, 0.0]));
        data.push(4);
        data.extend(floats(&[10.0, 0.0, 0.0, 12.0, 0.0, 0.0, 10.0, 4.0, 0.0, 10.0, 0.0, 0.0]));
        data.push(2);
        for (surface, zones, draw) in [(0, [2, 1], [0i32, 0, 0]), (0x81, [0, 3], [-1, 0, -1])] {
            data.extend(floats(&[0.0, 0.0, 1.0, 0.0]));
            data.extend([0; 9]);
            data.extend([0, surface, 0x81, 0x81, 0x81, 0x81, 0x81]);
            data.extend([0; 32]);
            data.extend(zones);
            data.push(3);
            data.extend([0xFF; 8]);
            data.extend(draw.iter().flat_map(|value| value.to_le_bytes()));
        }
        data.push(1);
        data.extend([0, 0x28, 0x80, 0x08, 0x00, 3, 0, 0, 1, 0, 0]);
        data.extend([0; 20]);
        data.push(3);
        data.extend([0, 0, 1, 0, 2, 0]);
        data.extend([0; 4]);
        data.extend([3, 0, 0, 0]);
        for actor in [0, 1, 0] {
            data.push(actor);
            data.extend([0; 20]);
        }
        data
    }

    /// The same model followed by what the client's levels store after the zones: polys, bounds,
    /// leaf hulls, one leaf, two lights, one section with the node's three vertices, one lightmap
    /// and one time group of eight tiny textures.
    fn model_with_render_data() -> Vec<u8> {
        let mut data = model();
        data.push(0);
        data.push(1);
        data.extend(floats(&[10.0, 0.0, 0.0, 12.0, 4.0, 0.0]));
        data.push(1);
        data.push(0);
        data.push(1);
        data.extend([1, 0x81, 0x81]);
        data.extend([0xFF; 8]);
        data.extend([2, 1, 0]);
        data.extend([1, 0, 0, 0, 0, 0, 0, 0]);
        data.push(1);
        data.push(3);
        for (x, y, u, v, u2, v2) in [(10.0, 0.0, 0.0, 0.0, 0.125, 0.25), (12.0, 0.0, 1.0, 0.0, 0.25, 0.25), (10.0, 4.0, 0.0, 1.0, 0.125, 0.5)] {
            data.extend(floats(&[x, y, 0.0, u, v, u2, v2, 0.0, 0.0, 1.0]));
        }
        data.extend([1, 0, 0, 0]);
        data.push(0);
        data.extend([1, 0, 0, 0]);
        data.extend([0, 1, 0, 0]);
        data.extend([0, 0, 0, 0]);
        data.push(1);
        data.extend([0, 0, 1, 0x40, 0x01, 0x40, 0x02, 8, 16]);
        data.extend(floats(&[0.25, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 2.0, 0.0, 1.0]));
        data.extend(floats(&[0.0; 9]));
        data.push(0);
        data.push(0);
        data.extend([1, 0, 0, 0]);
        data.push(1);
        data.push(8);
        for slice in 0u8..8 {
            data.push(0);
            data.push(0);
            data.extend([0; 8]);
            data.extend([1, 0, 0, 0]);
            data.extend([0; 4]);
            data.extend([4, slice, slice, slice, slice]);
            data.extend([0; 4]);
            data.extend([2, 9, 9]);
            data.push(3);
            data.extend([2, 0, 0, 0]);
            data.extend([2, 0, 0, 0]);
            data.extend([1, 0, 0, 0]);
        }
        data.push(1);
        data.extend([0, 0, 0, 0]);
        data
    }

    fn package(object: &[u8]) -> Vec<u8> {
        package_with_licensee(object, 0)
    }

    fn package_with_licensee(object: &[u8], licensee: u32) -> Vec<u8> {
        let mut file = Vec::new();
        for value in [0x9E2A_83C1, 123 | (licensee << 16), 0, 2, 36, 1, 0, 0, 0] {
            file.extend(u32::to_le_bytes(value));
        }
        file.extend([5, b'N', b'o', b'n', b'e', 0, 0, 0, 0, 0]);
        file.extend([5, b'H', b'a', b'l', b'l', 0, 0, 0, 0, 0]);
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
    #[expect(clippy::float_cmp, reason = "values are written and read back as the same bits")]
    fn reads_polygons_and_leaves_out_broken_references() {
        let file = package_with_licensee(&model_with_render_data(),20);
        let package = Package::parse(&file).unwrap();
        let model = read(&package, &file, &package.exports()[0]).unwrap().unwrap();

        assert_eq!(model.polygons.len(), 1);
        let polygon = &model.polygons[0];
        assert_eq!(polygon.corners, [[10.0, 0.0, 0.0], [12.0, 0.0, 0.0], [10.0, 4.0, 0.0]]);
        assert_eq!(polygon.texture_origin, [10.0, 0.0, 0.0]);
        assert_eq!(polygon.texture_axes, [[0.5, 0.0, 0.0], [0.0, 0.25, 0.0]]);
        assert_eq!(polygon.zone, 1);
        assert_eq!(polygon.flags, 0x0008_8028);
    }

    #[test]
    #[expect(clippy::float_cmp, reason = "values are written and read back as the same bits")]
    fn reads_sections_lightmaps_and_time_groups_to_the_end_of_the_object() {
        let file = package_with_licensee(&model_with_render_data(), 20);
        let package = Package::parse(&file).unwrap();
        let model = read(&package, &file, &package.exports()[0]).unwrap().unwrap();
        let render = &model.render;
        assert_eq!(render.leaves.len(), 1);
        assert_eq!(render.leaves[0], Leaf { zone: 1, permeating: -1, volumetric: -1 });
        assert_eq!(render.lights, vec![Some("Hall".to_owned()), None]);
        assert_eq!(render.sections.len(), 1);
        let section = &render.sections[0];
        assert_eq!(section.vertices.len(), 3);
        assert_eq!(section.vertices[2], BspVertex { position: [10.0, 4.0, 0.0], uv: [0.0, 1.0], lightmap_uv: [0.125, 0.5], normal: [0.0, 0.0, 1.0] });
        assert_eq!((section.num_nodes, section.poly_flags, section.light_map_texture), (1, 0x100, 0));
        assert_eq!(section.material, None);
        assert_eq!(render.node_draws.len(), 2);
        assert_eq!(render.node_draws[0], NodeDraw { section: 0, first_vertex: 0, num_vertices: 3, light_map: 0, surface_flags: 0x0008_8028 });
        assert_eq!(render.node_draws[1], NodeDraw { section: -1, first_vertex: 0, num_vertices: 3, light_map: -1, surface_flags: HIDDEN });
        assert_eq!(render.light_maps.len(), 1);
        let light_map = &render.light_maps[0];
        assert_eq!((light_map.texture, light_map.surface, light_map.zone), (0, 0, 1));
        assert_eq!((light_map.offset, light_map.size), ([64, 128], [8, 16]));
        assert_eq!(light_map.world_to_lightmap[3], [1.0, 2.0, 0.0, 1.0]);
        assert_eq!(render.groups.len(), 1);
        assert_eq!(render.groups[0].textures.len(), 8);
        let texture = &render.groups[0].textures[5];
        assert_eq!((texture.width, texture.height, texture.format), (2, 2, 3));
        assert_eq!(texture.mips, vec![vec![5, 5, 5, 5], vec![9, 9]]);
        assert!(render.complete);
    }

    #[test]
    fn a_model_that_stops_after_the_zones_cannot_hide_missing_collision_data() {
        let file = package(&model());
        let package = Package::parse(&file).unwrap();
        assert!(read(&package, &file, &package.exports()[0]).is_err());
    }

    #[test]
    fn reads_the_tree_and_the_zones() {
        let file = package_with_licensee(&model_with_render_data(),20);
        let package = Package::parse(&file).unwrap();
        let model = read(&package, &file, &package.exports()[0]).unwrap().unwrap();

        assert_eq!(model.tree.nodes.len(), 2);
        let node = &model.tree.nodes[0];
        assert_eq!(node.plane, [0.0, 0.0, 1.0, 0.0]);
        assert_eq!((node.front, node.back), (-1, -1));
        assert_eq!((node.zone_front, node.zone_back), (1, 2));
        assert_eq!((node.leaf_front, node.leaf_back), (-1, -1));
        assert_eq!(model.tree.zone_at([3.0, 3.0, 5.0]), 1);
        assert_eq!(model.tree.zone_at([3.0, 3.0, -5.0]), 2);
        assert_eq!(model.zones.len(), 3);
        assert_eq!(model.zones[0].actor, None);
        assert_eq!(model.zones[1].actor.as_deref(), Some("Hall"));
        assert_eq!(model.zones[2].actor, None);
    }
}
