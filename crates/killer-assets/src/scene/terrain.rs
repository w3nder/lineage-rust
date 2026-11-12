use super::lights::{PointLight, TERRAIN_POINT_SCALE, point_term, sector_lights};
use super::{DrawKind, LightmapGroup, MeshDraw, Pass, SectorBounds, Vertex, flat_normals, resolve_material};
use crate::client::Client;
use crate::textures::{TextureFormatKind, alpha_map_image};
use glam::Vec3;
use l2_catalog::{BlendFactor, Material};
use std::collections::HashMap;
use ue2_level::{LAYER_ALL_UNKNOWN, Terrain, TerrainLayer, TerrainSector};

const HEIGHT_ZERO: f32 = 32767.0;
const HEIGHT_STEPS: f32 = 256.0;
const LAYER_ALL_OPAQUE: u16 = 0xff;
const SLICE_HOURS: f32 = 3.0;
const FIRST_SLICE_HOUR: f32 = 1.5;
const HOURS_PER_DAY: f32 = 24.0;
const SEAM_OFFSET: u32 = 240;
pub const SECTOR_QUADS: u32 = 16;
const SEAM_MASK_STEP: f32 = 1.0 / 4096.0;
const ROTATOR_TURN: f32 = 65536.0;
const TEXMAPAXIS_XZ: u8 = 1;
const TEXMAPAXIS_YZ: u8 = 2;
pub const INTENSITY_SLICES: usize = 8;

/// Heightmap samples turned into world heights; `width`/`height` may exceed the heightmap by the seam column and row.
#[derive(Clone, Debug)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub heightmap: [usize; 2],
    pub heights: Vec<f32>,
    pub location: [f32; 3],
    pub scale: [f32; 3],
}

impl Grid {
    pub fn from_samples(width: usize, height: usize, samples: &[u16], location: [f32; 3], scale: [f32; 3]) -> Self {
        let heights = samples.iter().take(width * height).map(|&h| location[2] + (f32::from(h) - HEIGHT_ZERO) * scale[2] / HEIGHT_STEPS).collect();
        Self { width, height, heightmap: [width, height], heights, location, scale }
    }

    pub fn z(&self, x: usize, y: usize) -> f32 {
        self.heights.get(y * self.width + x).copied().unwrap_or(self.location[2])
    }

    pub fn position(&self, x: usize, y: usize) -> [f32; 3] {
        [
            self.location[0] + (x as f32 - (self.heightmap[0] / 2) as f32) * self.scale[0],
            self.location[1] + (y as f32 - (self.heightmap[1] / 2) as f32) * self.scale[1],
            self.z(x, y),
        ]
    }

    /// Adds the seam column and row of the seamless client: the neighbour's column 0 / row 0 when loaded, else a copy of 255.
    pub fn seamed(&self, right: Option<&Grid>, top: Option<&Grid>, corner: Option<&Grid>) -> Self {
        let (w, h) = (self.width + 1, self.height + 1);
        let mut heights = vec![self.location[2]; w * h];
        for y in 0..self.height {
            for x in 0..self.width {
                heights[y * w + x] = self.z(x, y);
            }
            heights[y * w + self.width] = right.map_or(self.z(self.width - 1, y), |r| r.z(0, y));
        }
        for x in 0..self.width {
            heights[self.height * w + x] = top.map_or(self.z(x, self.height - 1), |t| t.z(x, 0));
        }
        heights[self.height * w + self.width] = corner.map_or(heights[self.height * w + self.width - 1], |c| c.z(0, 0));
        Self { width: w, height: h, heightmap: self.heightmap, heights, location: self.location, scale: self.scale }
    }
}

/// `PostLoad` in seamless mode: quad column 255 and quad row 255 are always visible.
pub fn seamless_visibility(bits: &[u32], width: usize, height: usize) -> Vec<u32> {
    let words_per_row = width / 32;
    let mut bits = bits.to_vec();
    if bits.is_empty() || words_per_row == 0 {
        return bits;
    }
    for (index, word) in bits.iter_mut().enumerate() {
        if index % words_per_row == words_per_row - 1 {
            *word |= 1 << 31;
        }
        if index >= (height - 1) * words_per_row {
            *word = u32::MAX;
        }
    }
    bits
}

/// One `VertexColors` per grid sample, with the seam column and row from the neighbours or copied from 255.
pub fn seamed_colors(colors: &[[u8; 4]], width: usize, height: usize, right: Option<&[[u8; 4]]>, top: Option<&[[u8; 4]]>, corner: Option<&[[u8; 4]]>) -> Vec<[u8; 4]> {
    let (w, h) = (width + 1, height + 1);
    let at = |source: &[[u8; 4]], x: usize, y: usize| source.get(y * width + x).copied().unwrap_or([0, 0, 0, 255]);
    let mut out = vec![[0, 0, 0, 255]; w * h];
    for y in 0..height {
        for x in 0..width {
            out[y * w + x] = at(colors, x, y);
        }
        out[y * w + width] = right.map_or(at(colors, width - 1, y), |r| at(r, 0, y));
    }
    for x in 0..width {
        out[height * w + x] = top.map_or(at(colors, x, height - 1), |t| at(t, x, 0));
    }
    out[height * w + width] = corner.map_or(out[height * w + width - 1], |c| at(c, 0, 0));
    out
}

pub struct TerrainSeam<'a> {
    pub right: Option<&'a Terrain>,
    pub top: Option<&'a Terrain>,
    pub corner: Option<&'a Terrain>,
}

impl TerrainSeam<'_> {
    pub const NONE: TerrainSeam<'static> = TerrainSeam { right: None, top: None, corner: None };
}

pub fn load_grid(client: &mut Client, terrain: &Terrain) -> Option<Grid> {
    let heightmap = client.catalog().heightmap_with_base(&terrain.heightmap, terrain.base_height)?;
    Some(Grid::from_samples(heightmap.width, heightmap.height, &heightmap.samples, terrain.location, terrain.scale))
}

pub fn seamed_grid(client: &mut Client, terrain: &Terrain, seam: &TerrainSeam<'_>) -> Option<Grid> {
    let own = load_grid(client, terrain)?;
    let right = seam.right.and_then(|t| load_grid(client, t));
    let top = seam.top.and_then(|t| load_grid(client, t));
    let corner = seam.corner.and_then(|t| load_grid(client, t));
    Some(own.seamed(right.as_ref(), top.as_ref(), corner.as_ref()))
}

pub fn runtime_quads(sector: &TerrainSector) -> [u32; 2] {
    [if sector.offset[0] >= SEAM_OFFSET { SECTOR_QUADS } else { sector.quads[0] }, if sector.offset[1] >= SEAM_OFFSET { SECTOR_QUADS } else { sector.quads[1] }]
}

pub fn quad_bit(bits: &[u32], width: usize, x: usize, y: usize) -> Option<bool> {
    let bit = y * width + x;
    bits.get(bit / 32).map(|word| word & (1 << (bit % 32)) != 0)
}

pub fn quad_visible(bits: &[u32], width: usize, x: usize, y: usize) -> bool {
    bits.is_empty() || quad_bit(bits, width, x, y).unwrap_or(false)
}

pub fn edge_turned(bits: &[u32], width: usize, x: usize, y: usize) -> bool {
    quad_bit(bits, width, x, y).unwrap_or(false)
}

fn rotate(point: [f32; 2], units: f32) -> [f32; 2] {
    if units == 0.0 {
        return point;
    }
    let (sin, cos) = (units / ROTATOR_TURN * std::f32::consts::TAU).sin_cos();
    [point[0] * cos - point[1] * sin, point[0] * sin + point[1] * cos]
}

pub fn layer_uv(world: [f32; 3], layer: &TerrainLayer, scale: [f32; 3]) -> [f32; 2] {
    let plane = match layer.texture_map_axis {
        TEXMAPAXIS_XZ => [world[0], world[2]],
        TEXMAPAXIS_YZ => [world[1], world[2]],
        _ => [world[0], world[1]],
    };
    let turned = rotate(plane, layer.layer_rotation[1] as f32);
    let panned = [turned[0] - layer.pan[0] * layer.scale[0] * scale[0], turned[1] - layer.pan[1] * layer.scale[1] * scale[1]];
    let local = rotate(panned, -(layer.texture_rotation as f32));
    [local[0] / (scale[0] * layer.scale[0]), local[1] / (scale[1] * layer.scale[1])]
}

fn mask_axis(index: usize, samples: usize) -> f32 {
    let x = index as f32;
    let seam = index.saturating_sub(SEAM_OFFSET as usize) as f32;
    (x + 0.5) / samples as f32 - seam * SEAM_MASK_STEP
}

pub fn mask_uv(x: usize, y: usize, width: usize, height: usize) -> [f32; 2] {
    [mask_axis(x, width), mask_axis(y, height)]
}

pub fn quad_triangles(i00: u32, i10: u32, i01: u32, i11: u32, turned: bool) -> [u32; 6] {
    if turned { [i00, i01, i10, i01, i11, i10] } else { [i00, i01, i11, i00, i11, i10] }
}

pub fn terrain_normals(vertices: &mut [Vertex], indices: &[u32], inverted: bool) {
    flat_normals(vertices, indices);
    if !inverted {
        for vertex in vertices {
            vertex.normal = vertex.normal.map(|n| -n);
        }
    }
}

pub fn grid_indices(width: usize, height: usize, visible: &[u32], edge_turn: &[u32]) -> Vec<u32> {
    let mut indices = Vec::new();
    if width < 2 || height < 2 {
        return indices;
    }
    for y in 0..height - 1 {
        for x in 0..width - 1 {
            if !quad_visible(visible, width, x, y) {
                continue;
            }
            let i00 = (y * width + x) as u32;
            let w = width as u32;
            indices.extend_from_slice(&quad_triangles(i00, i00 + 1, i00 + w, i00 + w + 1, edge_turned(edge_turn, width, x, y)));
        }
    }
    indices
}

pub fn time_slices(hour: f32) -> (usize, usize, f32) {
    let hour = hour.rem_euclid(HOURS_PER_DAY);
    let first = if hour < FIRST_SLICE_HOUR { INTENSITY_SLICES - 1 } else { (((hour - FIRST_SLICE_HOUR) / SLICE_HOURS) as usize).min(INTENSITY_SLICES - 1) };
    let second = (first + 1) % INTENSITY_SLICES;
    let start = FIRST_SLICE_HOUR + SLICE_HOURS * first as f32;
    let shifted = if hour < start { hour + HOURS_PER_DAY } else { hour };
    (first, second, (shifted - start) / SLICE_HOURS)
}

pub fn intensity(sector: &TerrainSector, first: usize, second: usize, fraction: f32, x: usize, y: usize) -> u8 {
    let a = f32::from(sector.intensity(first, x, y).unwrap_or(0));
    let b = f32::from(sector.intensity(second, x, y).unwrap_or(0));
    (a * (1.0 - fraction) + b * fraction) as u8
}

pub fn intensity_group(terrain: &Terrain, width: usize, height: usize) -> LightmapGroup {
    let side = width.max(height) + 1;
    let mut slices = vec![vec![0u8; side * side * 4]; INTENSITY_SLICES];
    for sector in &terrain.sectors {
        let [columns, rows] = sector.vertices();
        for (slice, pixels) in slices.iter_mut().enumerate() {
            for y in 0..rows {
                for x in 0..columns {
                    let gx = sector.offset[0] as usize + x;
                    let gy = sector.offset[1] as usize + y;
                    if gx >= side || gy >= side {
                        continue;
                    }
                    let value = sector.intensity(slice, x, y).unwrap_or(0);
                    let at = (gy * side + gx) * 4;
                    pixels[at..at + 4].copy_from_slice(&[value, value, value, 255]);
                }
            }
        }
    }
    LightmapGroup { width: side as u32, height: side as u32, format: TextureFormatKind::Rgba8, slices: slices.into_iter().map(|pixels| vec![pixels]).collect() }
}

struct AlphaMap {
    width: usize,
    height: usize,
    alpha: Vec<u8>,
}

fn alpha_map(client: &mut Client, path: &str) -> Option<AlphaMap> {
    let image = alpha_map_image(client.catalog().texture_raw(path)?)?;
    Some(AlphaMap { width: image.width as usize, height: image.height as usize, alpha: image.rgba.as_chunks::<4>().0.iter().map(|px| px[3]).collect() })
}

fn sector_all(map: &AlphaMap, sector: &TerrainSector, heightmap: usize, value: u8) -> bool {
    let su = map.width as f32 / heightmap as f32;
    let sv = map.height as f32 / heightmap as f32;
    let quads = runtime_quads(sector);
    let x0 = (sector.offset[0] as f32 * su).floor() as usize;
    let x1 = (((sector.offset[0] + quads[0]) as f32) * su).ceil() as usize;
    let y0 = (sector.offset[1] as f32 * sv).floor() as usize;
    let y1 = (((sector.offset[1] + quads[1]) as f32) * sv).ceil() as usize;
    for y in y0..=y1.min(map.height - 1) {
        for x in x0..=x1.min(map.width - 1) {
            if map.alpha[y * map.width + x] != value {
                return false;
            }
        }
    }
    true
}

pub fn sector_passes(layers: &[(&TerrainLayer, &Material)], is_all: impl FnMut(usize, u16) -> bool) -> Vec<usize> {
    let mut is_all = is_all;
    let usable = |k: usize| layers.get(k).is_some_and(|(layer, _)| layer.alpha_map.is_some());
    let mut passes = Vec::new();
    let mut k = 0;
    while usable(k) {
        if !is_all(k, 0) {
            let mut j = k;
            loop {
                j += 1;
                if !usable(j) {
                    passes.push(k);
                    break;
                }
                if layers[j].1.transparent {
                    continue;
                }
                if is_all(j, LAYER_ALL_OPAQUE) {
                    break;
                }
            }
        }
        k += 1;
    }
    passes
}

pub fn sector_bounds(vertices: &[Vertex]) -> SectorBounds {
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);
    for vertex in vertices {
        min = min.min(Vec3::from(vertex.position));
        max = max.max(Vec3::from(vertex.position));
    }
    SectorBounds { location: [0.0; 3], min: min.to_array(), max: max.to_array() }
}

pub fn sector_location(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let (a, b, c) = (Vec3::from(a), Vec3::from(b), Vec3::from(c));
    (a + (b - a) / 2.0 + (c - a) / 2.0).to_array()
}

pub fn light_vertices(vertices: &mut [Vertex], lights: &[PointLight]) {
    if lights.is_empty() {
        return;
    }
    for vertex in vertices {
        let [r, g, b] = point_term(Vec3::from(vertex.position), Vec3::from(vertex.normal), lights, TERRAIN_POINT_SCALE);
        vertex.color[0] += r;
        vertex.color[1] += g;
        vertex.color[2] += b;
    }
}

pub fn build(client: &mut Client, terrain: &Terrain, zone: u8, sun_affect: bool, intensity: Option<usize>, lights: &[PointLight], seam: &TerrainSeam<'_>) -> Vec<MeshDraw> {
    let Some(grid) = seamed_grid(client, terrain, seam) else { return Vec::new() };
    if grid.width < 3 || grid.height < 3 {
        return Vec::new();
    }
    let [hx, hy] = grid.heightmap;
    let colors = seamed_colors(
        &terrain.native.vertex_colors,
        hx,
        hy,
        seam.right.map(|t| t.native.vertex_colors.as_slice()),
        seam.top.map(|t| t.native.vertex_colors.as_slice()),
        seam.corner.map(|t| t.native.vertex_colors.as_slice()),
    );
    let visible = seamless_visibility(&terrain.visible_quads, hx, hy);
    let materials: Vec<Material> = terrain.layers.iter().map(|layer| resolve_material(client, &layer.material)).collect();
    let layers: Vec<(&TerrainLayer, &Material)> = terrain.layers.iter().zip(&materials).collect();
    let mut alpha_maps: HashMap<String, Option<AlphaMap>> = HashMap::new();
    let mut draws = Vec::new();
    for sector in &terrain.sectors {
        let quads = runtime_quads(sector).map(|quads| quads as usize);
        let columns = quads[0] + 1;
        let rows = quads[1] + 1;
        if sector.offset[0] as usize + quads[0] >= grid.width || sector.offset[1] as usize + quads[1] >= grid.height {
            continue;
        }
        let mut scanned: HashMap<(usize, u16), bool> = HashMap::new();
        let is_all = |k: usize, value: u16| -> bool {
            let cached = sector.layer_all.get(k).copied().unwrap_or(LAYER_ALL_UNKNOWN);
            if cached != LAYER_ALL_UNKNOWN {
                return cached == value;
            }
            *scanned.entry((k, value)).or_insert_with(|| {
                let Some(path) = terrain.layers[k].alpha_map.as_deref() else { return false };
                let map = alpha_maps.entry(path.to_lowercase()).or_insert_with(|| alpha_map(client, path));
                map.as_ref().is_some_and(|map| sector_all(map, sector, hx, value as u8))
            })
        };
        let passes = sector_passes(&layers, is_all);
        let corner = |x: usize, y: usize| grid.position(sector.offset[0] as usize + x, sector.offset[1] as usize + y);
        let location = sector_location(corner(0, 0), corner(quads[0], 0), corner(0, quads[1]));
        for (pass_index, &layer_index) in passes.iter().enumerate() {
            let (layer, material) = layers[layer_index];
            let mut vertices = Vec::with_capacity(columns * rows);
            for y in 0..rows {
                for x in 0..columns {
                    let gx = sector.offset[0] as usize + x;
                    let gy = sector.offset[1] as usize + y;
                    let position = grid.position(gx, gy);
                    let uv = layer_uv(position, layer, terrain.scale);
                    let mask_uv = mask_uv(gx, gy, hx, hy);
                    let color = colors.get(gy * grid.width + gx).map_or([0.0, 0.0, 0.0, 1.0], |c| [f32::from(c[0]) / 255.0, f32::from(c[1]) / 255.0, f32::from(c[2]) / 255.0, 1.0]);
                    vertices.push(Vertex { position, normal: [0.0; 3], uv, mask_uv, color });
                }
            }
            let mut indices = Vec::new();
            for y in 0..quads[1] {
                for x in 0..quads[0] {
                    let gx = sector.offset[0] as usize + x;
                    let gy = sector.offset[1] as usize + y;
                    if !quad_visible(&visible, hx, gx, gy) || !sector.in_pass(pass_index, x, y) {
                        continue;
                    }
                    let i00 = (y * columns + x) as u32;
                    let w = columns as u32;
                    indices.extend_from_slice(&quad_triangles(i00, i00 + 1, i00 + w, i00 + w + 1, edge_turned(&terrain.edge_turn, hx, gx, gy)));
                }
            }
            if indices.is_empty() {
                continue;
            }
            terrain_normals(&mut vertices, &indices, terrain.inverted);
            let bounds = SectorBounds { location, ..sector_bounds(&vertices) };
            light_vertices(&mut vertices, &sector_lights(lights, Vec3::from(bounds.min), Vec3::from(bounds.max)));
            let mut material = material.clone();
            if pass_index > 0 && let Some(pass) = material.passes.first_mut() {
                pass.src = BlendFactor::SrcAlpha;
                pass.dst = BlendFactor::InvSrcAlpha;
                pass.alpha_blend = true;
                pass.z_write = false;
            }
            draws.push(MeshDraw {
                instances: Vec::new(), vertices,
                indices,
                material,
                mask: if pass_index == 0 { None } else { layer.alpha_map.clone() },
                kind: DrawKind::Terrain,
                sun_affect,
                pass: if pass_index == 0 { Pass::Opaque } else { Pass::Layer(pass_index as u16) },
                zone,
                lightmap: intensity,
                modulate_2x: true,
                sector: Some(bounds),
                fade: None,
                tint: None,
                actor: None,
            });
        }
    }
    draws
}

#[cfg(test)]
mod tests {
    use super::*;
    use ue2_level::SECTOR_MAP_SIDE;

    #[test]
    fn sample_positions_follow_the_ue2_terrain_formula() {
        let grid = Grid::from_samples(4, 4, &[32767; 16], [1000.0, 2000.0, -100.0], [128.0, 128.0, 76.0]);
        let p = grid.position(0, 0);
        assert_eq!(p, [1000.0 - 2.0 * 128.0, 2000.0 - 2.0 * 128.0, -100.0]);
        let mut samples = [32767u16; 16];
        samples[5] = 32767 + 256;
        let raised = Grid::from_samples(4, 4, &samples, [1000.0, 2000.0, -100.0], [128.0, 128.0, 76.0]);
        assert_eq!(raised.position(1, 1)[2], -100.0 + 76.0);
        samples[0] = 32768;
        let above = Grid::from_samples(4, 4, &samples, [1000.0, 2000.0, -100.0], [128.0, 128.0, 76.0]);
        assert!((above.position(0, 0)[2] - (-100.0 + 76.0 / 256.0)).abs() < 1e-4);
        let odd = Grid::from_samples(5, 5, &[32767; 25], [0.0; 3], [128.0, 128.0, 76.0]);
        assert_eq!(odd.position(0, 0)[0], -2.0 * 128.0);
    }

    #[test]
    fn the_seam_takes_the_neighbours_first_column_row_and_corner_or_copies_the_last() {
        let own = Grid::from_samples(4, 4, &(0..16).map(|i| 32767 + i as u16 * 256).collect::<Vec<_>>(), [0.0; 3], [128.0, 128.0, 76.0]);
        let alone = own.seamed(None, None, None);
        assert_eq!((alone.width, alone.height, alone.heightmap), (5, 5, [4, 4]));
        assert_eq!(alone.position(4, 1)[0], own.position(3, 1)[0] + 128.0);
        assert_eq!(alone.z(4, 1), own.z(3, 1));
        assert_eq!(alone.z(1, 4), own.z(1, 3));
        assert_eq!(alone.z(4, 4), own.z(3, 3));
        let right = Grid::from_samples(4, 4, &[40000; 16], [512.0, 0.0, 1000.0], [128.0, 128.0, 76.0]);
        let top = Grid::from_samples(4, 4, &[20000; 16], [0.0, 512.0, -1000.0], [128.0, 128.0, 76.0]);
        let corner = Grid::from_samples(4, 4, &[32767; 16], [512.0, 512.0, 7.0], [128.0, 128.0, 76.0]);
        let stitched = own.seamed(Some(&right), Some(&top), Some(&corner));
        assert_eq!(stitched.z(4, 2), right.z(0, 2));
        assert_eq!(stitched.z(2, 4), top.z(2, 0));
        assert_eq!(stitched.z(4, 4), 7.0);
        assert_eq!(stitched.z(3, 3), own.z(3, 3));
    }

    #[test]
    fn seamless_visibility_forces_the_last_quad_column_and_row() {
        let bits = vec![0u32; 8 * 256];
        let forced = seamless_visibility(&bits, 256, 256);
        assert!(quad_visible(&forced, 256, 255, 0));
        assert!(quad_visible(&forced, 256, 255, 100));
        assert!(!quad_visible(&forced, 256, 254, 100));
        assert!(quad_visible(&forced, 256, 7, 255));
        assert!(!quad_visible(&forced, 256, 7, 254));
        assert!(seamless_visibility(&[], 256, 256).is_empty());
    }

    #[test]
    fn seamed_colors_follow_the_same_rule_as_the_heights() {
        let own: Vec<[u8; 4]> = (0..4).map(|i| [i, 0, 0, 255]).collect();
        let right = vec![[9, 9, 9, 255]; 4];
        let alone = seamed_colors(&own, 2, 2, None, None, None);
        assert_eq!(alone.len(), 9);
        assert_eq!(alone[2], [1, 0, 0, 255]);
        assert_eq!(alone[8], [3, 0, 0, 255]);
        let stitched = seamed_colors(&own, 2, 2, Some(&right), None, None);
        assert_eq!(stitched[2], [9, 9, 9, 255]);
        assert_eq!(stitched[5], [9, 9, 9, 255]);
        assert_eq!(stitched[7], [3, 0, 0, 255]);
    }

    #[test]
    fn quad_visibility_bit_is_row_major_over_the_heightmap_width() {
        let mut bits = vec![0u32; 2];
        bits[0] |= 1 << 5;
        assert!(quad_visible(&bits, 4, 1, 1));
        assert!(!quad_visible(&bits, 4, 0, 0));
        assert!(quad_visible(&[], 4, 0, 0));
        assert!(edge_turned(&bits, 4, 1, 1));
        assert!(!edge_turned(&[], 4, 1, 1));
    }

    fn layer_with(scale: [f32; 2], pan: [f32; 2], texture_rotation: i32, layer_yaw: i32, axis: u8) -> TerrainLayer {
        TerrainLayer { material: "P.m".into(), alpha_map: Some("P.a".into()), scale, pan, texture_map_axis: axis, texture_rotation, layer_rotation: [0, layer_yaw, 0] }
    }

    #[test]
    fn layer_uv_follows_calc_layer_tex_coords_over_the_world_position() {
        let scale = [128.0, 128.0, 76.0];
        let plain = layer_with([4.0, 4.0], [0.0, 0.0], 0, 0, 0);
        assert_eq!(layer_uv([1024.0, 2560.0, 7.0], &plain, scale), [2.0, 5.0]);
        let panned = layer_with([4.0, 4.0], [1.0, 0.5], 0, 0, 0);
        assert_eq!(layer_uv([1024.0, 2560.0, 7.0], &panned, scale), [1.0, 4.5]);
        let quarter = layer_with([4.0, 4.0], [0.0, 0.0], 16384, 0, 0);
        let uv = layer_uv([512.0, 0.0, 0.0], &quarter, scale);
        assert!((uv[0]).abs() < 1e-5 && (uv[1] + 1.0).abs() < 1e-5, "{uv:?}");
        let turned = layer_with([4.0, 4.0], [0.0, 0.0], 0, 16384, 0);
        let uv = layer_uv([512.0, 0.0, 0.0], &turned, scale);
        assert!((uv[0]).abs() < 1e-5 && (uv[1] - 1.0).abs() < 1e-5, "{uv:?}");
        let xz = layer_with([1.0, 1.0], [0.0, 0.0], 0, 0, 1);
        assert_eq!(layer_uv([128.0, 256.0, 384.0], &xz, scale), [1.0, 3.0]);
        let yz = layer_with([1.0, 1.0], [0.0, 0.0], 0, 0, 2);
        assert_eq!(layer_uv([128.0, 256.0, 384.0], &yz, scale), [2.0, 3.0]);
    }

    #[test]
    fn mask_uv_sits_on_texel_centers_and_squeezes_the_seam_columns() {
        assert_eq!(mask_uv(3, 5, 256, 256), [3.5 / 256.0, 5.5 / 256.0]);
        assert_eq!(mask_uv(240, 0, 256, 256), [240.5 / 256.0, 0.5 / 256.0]);
        let [u, _] = mask_uv(256, 0, 256, 256);
        assert!((u - 255.5 / 256.0).abs() < 1e-6, "{u}");
    }

    #[test]
    fn sector_location_is_the_corner_plus_half_of_both_edges() {
        assert_eq!(sector_location([0.0, 0.0, 10.0], [2048.0, 0.0, 30.0], [0.0, 2048.0, 50.0]), [1024.0, 1024.0, 40.0]);
    }

    #[test]
    fn sector_lights_add_the_full_point_term_to_the_vertex_color() {
        let lamp = PointLight { position: Vec3::new(0.0, 0.0, 100.0), radius: 400.0, color: [0.8, 0.4, 0.2], brightness: 64.0, ..Default::default() };
        let mut vertices = vec![Vertex { position: [0.0; 3], normal: [0.0, 0.0, 1.0], uv: [0.0; 2], mask_uv: [0.0; 2], color: [0.1, 0.1, 0.1, 1.0] }];
        light_vertices(&mut vertices, &[lamp]);
        let expected = 0.1 + 0.8 * super::super::lights::point_falloff(100.0, 400.0);
        assert!((vertices[0].color[0] - expected).abs() < 1e-5, "{:?}", vertices[0].color);
        assert!((vertices[0].color[2] - (0.1 + 0.2 * super::super::lights::point_falloff(100.0, 400.0))).abs() < 1e-5);
        assert_eq!(vertices[0].color[3], 1.0);
    }

    #[test]
    fn the_edge_turn_bit_picks_the_quad_diagonal() {
        assert_eq!(quad_triangles(0, 1, 4, 5, false), [0, 4, 5, 0, 5, 1]);
        assert_eq!(quad_triangles(0, 1, 4, 5, true), [0, 4, 1, 4, 5, 1]);
        let turn = vec![1u32];
        let indices = grid_indices(3, 2, &[], &turn);
        assert_eq!(&indices[..6], &[0, 3, 1, 3, 4, 1]);
        assert_eq!(&indices[6..], &[1, 4, 5, 1, 5, 2]);
    }

    #[test]
    fn grid_indices_wind_against_the_face_normal_like_the_engine_and_static_meshes() {
        let grid = Grid::from_samples(3, 3, &[32767; 9], [0.0, 0.0, 0.0], [100.0, 100.0, 76.0]);
        for turn in [vec![], vec![0xFFFF_FFFFu32]] {
            let indices = grid_indices(grid.width, grid.height, &[], &turn);
            let mut vertices: Vec<Vertex> = (0..grid.height)
                .flat_map(|y| (0..grid.width).map(move |x| (x, y)))
                .map(|(x, y)| Vertex { position: grid.position(x, y), normal: [0.0; 3], uv: [0.0; 2], mask_uv: [0.0; 2], color: [1.0; 4] })
                .collect();
            flat_normals(&mut vertices, &indices);
            assert!(vertices.iter().all(|v| v.normal[2] < -0.99), "{:?}", vertices[0].normal);
            let reset = |vertices: &mut Vec<Vertex>| vertices.iter_mut().for_each(|v| v.normal = [0.0; 3]);
            reset(&mut vertices);
            terrain_normals(&mut vertices, &indices, false);
            assert!(vertices.iter().all(|v| v.normal[2] > 0.99), "{:?}", vertices[0].normal);
            reset(&mut vertices);
            terrain_normals(&mut vertices, &indices, true);
            assert!(vertices.iter().all(|v| v.normal[2] < -0.99), "{:?}", vertices[0].normal);
        }
    }

    #[test]
    fn grid_indices_are_empty_below_a_two_by_two_grid() {
        assert!(grid_indices(1, 5, &[], &[]).is_empty());
        assert!(grid_indices(5, 1, &[], &[]).is_empty());
    }

    #[test]
    fn time_slices_follow_the_intensity_map_hours() {
        assert_eq!(time_slices(12.0), (3, 4, 0.5));
        let (first, second, f) = time_slices(23.0);
        assert_eq!((first, second), (7, 0));
        assert!((f - 0.5 / 3.0).abs() < 1e-6);
        let (first, second, f) = time_slices(0.5);
        assert_eq!((first, second), (7, 0));
        assert!((f - 2.0 / 3.0).abs() < 1e-6);
        assert_eq!(time_slices(1.5), (0, 1, 0.0));
        assert_eq!(time_slices(22.5), (7, 0, 0.0));
        assert_eq!(time_slices(4.5), (1, 2, 0.0));
    }

    fn sector(maps: Vec<Vec<u8>>, layer_all: [u16; 32], pass_mask: Vec<u16>) -> TerrainSector {
        TerrainSector { offset: [0, 0], quads: [16, 16], intensities: maps, layer_all, pass_mask, lights: Vec::new() }
    }

    #[test]
    fn intensity_interpolates_and_truncates_like_ftol() {
        let mut maps = vec![vec![0u8; SECTOR_MAP_SIDE * SECTOR_MAP_SIDE]; 8];
        maps[3][0] = 153;
        maps[4][0] = 131;
        maps[7][0] = 52;
        maps[0][0] = 153;
        let sector = sector(maps, [LAYER_ALL_UNKNOWN; 32], Vec::new());
        assert_eq!(intensity(&sector, 3, 4, 0.0, 0, 0), 153);
        assert_eq!(intensity(&sector, 3, 4, 1.0, 0, 0), 131);
        assert_eq!(intensity(&sector, 3, 4, 0.5, 0, 0), 142);
        assert_eq!(intensity(&sector, 7, 0, 0.5 / 3.0, 0, 0), 68);
    }

    fn layer(alpha: bool) -> TerrainLayer {
        TerrainLayer { material: "P.m".into(), alpha_map: alpha.then(|| "P.a".to_owned()), scale: [1.0; 2], pan: [0.0; 2], texture_map_axis: 0, texture_rotation: 0, layer_rotation: [0; 3] }
    }

    #[test]
    fn passes_skip_empty_layers_and_layers_covered_by_a_later_opaque_one() {
        let opaque = super::super::texture_material("P.t");
        let mut transparent = opaque.clone();
        transparent.transparent = true;
        let layers = [layer(true), layer(true), layer(true), layer(true), layer(false)];
        let materials = [opaque.clone(), opaque.clone(), transparent, opaque.clone(), opaque.clone()];
        let pairs: Vec<(&TerrainLayer, &Material)> = layers.iter().zip(&materials).collect();
        let cache = [0xfffe, 0, 0xfffe, 0xff, 0xfffe];
        assert_eq!(sector_passes(&pairs, |k, value| cache[k] == value), vec![3]);
        let cache = [0xfffe, 0, 0xfffe, 0xfffe, 0xfffe];
        assert_eq!(sector_passes(&pairs, |k, value| cache[k] == value), vec![0, 2, 3]);
        let cache = [0xfffe; 5];
        assert_eq!(sector_passes(&pairs, |k, value| cache[k] == value), vec![0, 1, 2, 3]);
    }

    #[test]
    fn the_intensity_group_places_each_sector_map_at_its_offset() {
        let mut maps = vec![vec![0u8; SECTOR_MAP_SIDE * SECTOR_MAP_SIDE]; 8];
        maps[2][SECTOR_MAP_SIDE + 2] = 99;
        let mut sector = sector(maps, [LAYER_ALL_UNKNOWN; 32], Vec::new());
        sector.offset = [16, 32];
        let terrain = Terrain {
            name: String::new(),
            base_height: 0,
            location: [0.0; 3],
            scale: [128.0, 128.0, 76.0],
            heightmap: "P.h".into(),
            layers: Vec::new(),
            visible_quads: Vec::new(),
            edge_turn: Vec::new(),
            inverted: false,
            sectors: vec![sector],
            deco_layers: Vec::new(),
            deco_layer_offset: 0.0,
            native: ue2_level::TerrainNative::default(),
        };
        let group = intensity_group(&terrain, 256, 256);
        assert_eq!((group.width, group.height, group.slices.len()), (257, 257, 8));
        let at = ((32 + 1) * 257 + 16 + 2) * 4;
        assert_eq!(group.slices[2][0][at], 99);
        assert_eq!(group.slices[1][0][at], 0);
    }
}
