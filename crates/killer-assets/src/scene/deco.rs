use super::props::section_material;
use super::terrain::{Grid, TerrainSeam, mask_uv, quad_visible, runtime_quads, seamed_grid, seamless_visibility};
use super::transform::placement_matrix;
use super::{DecorationInstance, DrawKind, MeshDraw, Pass, Vertex, draw_pass, resolve_material};
use crate::client::Client;
use crate::textures::alpha_map_image;
use glam::Vec3;
use std::collections::HashMap;
use std::f32::consts::TAU;
use ue2_level::{DecoLayer, Range, Terrain, TerrainSector};

const SRAND_MULTIPLIER: u32 = 196_314_165;
const SRAND_INCREMENT: u32 = 907_633_515;
const SRAND_ONE: u32 = 0x3f80_0000;
const SRAND_MANTISSA: u32 = 0x007f_ffff;
const ROTATOR_TURN: f32 = 65536.0;
const PITCH_DOWN_QUARTER: i32 = 16384;
const YAW_RANDOM_SPAN: f32 = 65535.0;
const DENSITY_STEP: f32 = 1.0 / 255.0;

/// `appSRand`: the UE2 linear congruential generator seeded per layer and sector.
pub struct Srand(u32);

impl Srand {
    pub fn new(seed: i32) -> Self {
        Self(seed.cast_unsigned())
    }

    pub fn sample(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(SRAND_MULTIPLIER).wrapping_add(SRAND_INCREMENT);
        f32::from_bits(SRAND_ONE | (self.0 & SRAND_MANTISSA)).fract()
    }

    pub fn range(&mut self, range: Range) -> f32 {
        range[0] + (range[1] - range[0]) * self.sample()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DecoInstance {
    pub position: Vec3,
    pub rotation: [i32; 3],
    pub scale: Vec3,
    /// Heightmap sample of the quad corner the instance stands on.
    pub vertex: [usize; 2],
}

pub struct DensityMap {
    pub width: usize,
    pub height: usize,
    pub alpha: Vec<u8>,
}

impl DensityMap {
    pub fn at(&self, x: usize, y: usize, heightmap: [usize; 2]) -> u8 {
        let tx = (x * self.width / heightmap[0].max(1)).min(self.width.saturating_sub(1));
        let ty = (y * self.height / heightmap[1].max(1)).min(self.height.saturating_sub(1));
        self.alpha.get(ty * self.width + tx).copied().unwrap_or(0)
    }
}

fn rotator_units(radians: f32) -> i32 {
    (radians / TAU * ROTATOR_TURN) as i32
}

pub fn normal_rotation(normal: Vec3) -> [i32; 3] {
    let yaw = normal.y.atan2(normal.x);
    let pitch = normal.z.atan2(normal.truncate().length());
    [rotator_units(pitch) - PITCH_DOWN_QUARTER, rotator_units(yaw), 0]
}

pub fn fade(distance: f32, range: Range) -> f32 {
    if distance > range[1] {
        return -1.0;
    }
    if distance <= range[0] {
        return 1.0;
    }
    let k = if range[1] - range[0] != 0.0 { 1.0 / (range[1] - range[0]) } else { 1.0 };
    1.0 - (distance - range[0]) * k
}

fn quad_point(corners: [Vec3; 4], r1: f32, r2: f32) -> (Vec3, Vec3) {
    let [v00, v10, v01, v11] = corners;
    if r1 <= r2 {
        (v00 + (v10 - v00) * r1 + (v01 - v00) * (1.0 - r2), (v10 - v00).cross(v01 - v00))
    } else {
        (v11 + (v01 - v11) * (1.0 - r1) + (v10 - v11) * r2, (v01 - v11).cross(v10 - v11))
    }
}

pub fn instances(
    grid: &Grid,
    terrain: &Terrain,
    sector: &TerrainSector,
    layer: &DecoLayer,
    density: &DensityMap,
    scale_map: Option<&DensityMapColor>,
    rng: &mut Srand,
) -> Vec<DecoInstance> {
    let mut placed = Vec::new();
    let [hx, hy] = grid.heightmap;
    let visible = seamless_visibility(&terrain.visible_quads, hx, hy);
    let quads = runtime_quads(sector).map(|q| q as usize);
    let quads = [if sector.offset[0] as usize + quads[0] >= hx { quads[0] - 1 } else { quads[0] }, if sector.offset[1] as usize + quads[1] >= hy { quads[1] - 1 } else { quads[1] }];
    for y in 0..quads[1] {
        for x in 0..quads[0] {
            let gx = sector.offset[0] as usize + x;
            let gy = sector.offset[1] as usize + y;
            if gx + 1 >= grid.width || gy + 1 >= grid.height {
                continue;
            }
            if !layer.on_invisible_terrain && !quad_visible(&visible, hx, gx, gy) {
                continue;
            }
            let corners = [grid.position(gx, gy), grid.position(gx + 1, gy), grid.position(gx, gy + 1), grid.position(gx + 1, gy + 1)].map(Vec3::from);
            for _ in 0..layer.max_per_quad {
                let dens = f32::from(density.at(gx, gy, [hx, hy])) * DENSITY_STEP;
                if rng.sample() >= rng.range(layer.density_multiplier) * dens {
                    continue;
                }
                let r1 = rng.sample();
                let r2 = rng.sample();
                let (point, raw_normal) = quad_point(corners, r1, r2);
                let mut normal = raw_normal.normalize_or(Vec3::Z);
                if normal.z < 0.0 {
                    normal = -normal;
                }
                if terrain.inverted {
                    normal = -normal;
                }
                if !layer.align_to_terrain {
                    normal = if terrain.inverted { -Vec3::Z } else { Vec3::Z };
                }
                let mut rotation = normal_rotation(normal);
                if layer.random_yaw {
                    rotation[1] = (rng.sample() * YAW_RANDOM_SPAN) as i32;
                }
                let position = point + normal * terrain.deco_layer_offset;
                let mut scale = Vec3::new(rng.range(layer.scale[0]), rng.range(layer.scale[1]), rng.range(layer.scale[2]));
                if let Some(map) = scale_map {
                    let [r, g, b] = map.at(gx, gy, [hx, hy]);
                    scale *= Vec3::new(f32::from(b), f32::from(g), f32::from(r)) * DENSITY_STEP;
                }
                placed.push(DecoInstance { position, rotation, scale, vertex: [gx, gy] });
            }
        }
    }
    placed
}

pub struct DensityMapColor {
    pub width: usize,
    pub height: usize,
    pub rgb: Vec<[u8; 3]>,
}

impl DensityMapColor {
    pub fn at(&self, x: usize, y: usize, heightmap: [usize; 2]) -> [u8; 3] {
        let tx = (self.width * x / heightmap[0].max(1)).min(self.width.saturating_sub(1));
        let ty = (self.height * y / heightmap[1].max(1)).min(self.height.saturating_sub(1));
        self.rgb.get(ty * self.width + tx).copied().unwrap_or([255; 3])
    }
}

fn density_map(client: &mut Client, path: &str) -> Option<DensityMap> {
    let image = alpha_map_image(client.catalog().texture_raw(path)?)?;
    Some(DensityMap { width: image.width as usize, height: image.height as usize, alpha: image.rgba.as_chunks::<4>().0.iter().map(|px| px[3]).collect() })
}

fn color_map(client: &mut Client, path: &str) -> Option<DensityMapColor> {
    let image = client.catalog().texture(path)?;
    Some(DensityMapColor { width: image.width as usize, height: image.height as usize, rgb: image.rgba.as_chunks::<4>().0.iter().map(|px| [px[0], px[1], px[2]]).collect() })
}

struct SectionTopology {
    sources: Vec<usize>,
    indices: Vec<u32>,
}

impl SectionTopology {
    fn new(indices: &[u16]) -> Self {
        let mut sources = Vec::new();
        let mut remap = HashMap::new();
        let indices = indices.iter().map(|&index| *remap.entry(index).or_insert_with(|| {
            let local = sources.len() as u32;
            sources.push(index as usize);
            local
        })).collect();
        Self { sources, indices }
    }

    fn compact(&self, placed: &[DecoInstance], positions: &[[f32; 3]], uvs: &[[f32; 2]], colors: &[[u8; 4]], heightmap: [usize; 2]) -> (Vec<Vertex>, Vec<u32>, Vec<DecorationInstance>) {
        let vertices = self.sources.iter().map(|&source| Vertex {
            position: positions[source], normal: [0.0, 0.0, 1.0],
            uv: uvs.get(source).copied().unwrap_or([0.0; 2]), mask_uv: [0.0; 2], color: [1.0; 4],
        }).collect();
        let instances = placed.iter().map(|instance| DecorationInstance {
            world: placement_matrix(instance.position.to_array(), instance.rotation, instance.scale.to_array()).to_cols_array_2d(),
            color: colors.get(instance.vertex[1] * heightmap[0] + instance.vertex[0])
                .map_or([0.0, 0.0, 0.0, 1.0], |c| c.map(|c| f32::from(c) / 255.0)),
            mask_uv: mask_uv(instance.vertex[0], instance.vertex[1], heightmap[0], heightmap[1]),
        }).collect();
        (vertices, self.indices.clone(), instances)
    }

    #[cfg(test)]
    fn expand(&self, placed: &[DecoInstance], positions: &[[f32; 3]], uvs: &[[f32; 2]], colors: &[[u8; 4]], heightmap: [usize; 2]) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::with_capacity(placed.len() * self.sources.len());
        let mut indices = Vec::with_capacity(placed.len() * self.indices.len());
        for instance in placed {
            let world = placement_matrix(instance.position.to_array(), instance.rotation, instance.scale.to_array());
            let color = colors.get(instance.vertex[1] * heightmap[0] + instance.vertex[0])
                .map_or([0.0, 0.0, 0.0, 1.0], |c| [f32::from(c[0]) / 255.0, f32::from(c[1]) / 255.0, f32::from(c[2]) / 255.0, f32::from(c[3]) / 255.0]);
            let uv_mask = mask_uv(instance.vertex[0], instance.vertex[1], heightmap[0], heightmap[1]);
            let base = vertices.len() as u32;
            for &source in &self.sources {
                let position = world.transform_point3(Vec3::from(positions[source])).to_array();
                vertices.push(Vertex { position, normal: [0.0, 0.0, 1.0], uv: uvs.get(source).copied().unwrap_or([0.0; 2]), mask_uv: uv_mask, color });
            }
            indices.extend(self.indices.iter().map(|index| base + index));
        }
        (vertices, indices)
    }
}

pub struct GrassField {
    cells: HashMap<(i32, i32), Vec<[f32; 2]>>,
}

impl GrassField {
    pub const CELL: f32 = 64.0;
    pub const RADIUS: f32 = 64.0;

    pub fn new(points: &[[f32; 2]]) -> Self {
        let mut cells: HashMap<(i32, i32), Vec<[f32; 2]>> = HashMap::new();
        for &point in points {
            cells.entry(Self::cell(point[0], point[1])).or_default().push(point);
        }
        Self { cells }
    }

    fn cell(x: f32, y: f32) -> (i32, i32) {
        ((x / Self::CELL).floor() as i32, (y / Self::CELL).floor() as i32)
    }

    pub fn count_within(&self, x: f32, y: f32, radius: f32) -> usize {
        let (cx, cy) = Self::cell(x, y);
        let reach = (radius / Self::CELL).ceil() as i32;
        let mut count = 0;
        for gx in cx - reach..=cx + reach {
            for gy in cy - reach..=cy + reach {
                if let Some(points) = self.cells.get(&(gx, gy)) {
                    count += points.iter().filter(|p| ((p[0] - x).powi(2) + (p[1] - y).powi(2)).sqrt() < radius).count();
                }
            }
        }
        count
    }
}

pub fn points(client: &mut Client, terrain: &Terrain, render_deco: bool) -> Vec<[f32; 2]> {
    let mut points = Vec::new();
    let layers: Vec<&DecoLayer> = terrain.deco_layers.iter().filter(|layer| render_deco || layer.force_render).collect();
    if layers.is_empty() {
        return points;
    }
    let Some(grid) = seamed_grid(client, terrain, &TerrainSeam::NONE) else { return points };
    for layer in layers {
        let Some(density) = density_map(client, &layer.density_map) else { continue };
        let scale_map = layer.scale_map.as_deref().and_then(|path| color_map(client, path));
        for (sector_index, sector) in terrain.sectors.iter().enumerate() {
            let mut rng = Srand::new(layer.seed.wrapping_add(sector_index as i32));
            points.extend(instances(&grid, terrain, sector, layer, &density, scale_map.as_ref(), &mut rng).iter().map(|placed| [placed.position.x, placed.position.y]));
        }
    }
    points
}

pub fn build(client: &mut Client, terrain: &Terrain, zone: u8, sun_affect: bool, intensity: Option<usize>, render_deco: bool) -> Vec<MeshDraw> {
    let layers: Vec<&DecoLayer> = terrain.deco_layers.iter().filter(|layer| render_deco || layer.force_render).collect();
    if layers.is_empty() {
        return Vec::new();
    }
    let Some(grid) = seamed_grid(client, terrain, &TerrainSeam::NONE) else { return Vec::new() };
    let mut materials: HashMap<String, l2_catalog::Material> = HashMap::new();
    let mut draws = Vec::new();
    for layer in layers {
        let Some(mesh) = client.catalog().static_mesh(&layer.static_mesh) else { continue };
        let Some(density) = density_map(client, &layer.density_map) else { continue };
        let scale_map = layer.scale_map.as_deref().and_then(|path| color_map(client, path));
        let mesh_materials: Vec<l2_catalog::Material> = (0..mesh.mesh.sections.len())
            .map(|index| {
                let path = section_material(&[], &mesh.materials, index).unwrap_or_default();
                materials.entry(path.to_lowercase()).or_insert_with(|| resolve_material(client, &path)).clone()
            })
            .collect();
        let topology: Vec<_> = mesh.mesh.sections.iter().map(|section| {
            let first = section.first_index as usize;
            mesh.mesh.indices.get(first..first + section.index_count() as usize).map(SectionTopology::new)
        }).collect();
        for (sector_index, sector) in terrain.sectors.iter().enumerate() {
            let mut rng = Srand::new(layer.seed.wrapping_add(sector_index as i32));
            let placed = instances(&grid, terrain, sector, layer, &density, scale_map.as_ref(), &mut rng);
            if placed.is_empty() {
                continue;
            }
            for (section_index, topology) in topology.iter().enumerate() {
                let Some(topology) = topology else { continue };
                if topology.indices.is_empty() { continue; }
                let (vertices, indices, instances) = topology.compact(&placed, &mesh.mesh.positions, &mesh.mesh.uvs, &terrain.native.vertex_colors, grid.heightmap);
                let material = mesh_materials[section_index].clone();
                let pass = match draw_pass(&material) {
                    Pass::Translucent => Pass::Translucent,
                    _ => Pass::Layer(u16::MAX),
                };
                draws.push(MeshDraw {
                    instances, vertices,
                    indices,
                    material,
                    mask: None,
                    kind: DrawKind::Terrain,
                    sun_affect,
                    pass,
                    zone,
                    lightmap: intensity,
                    modulate_2x: true,
                    sector: None,
                    fade: Some(layer.fadeout_radius),
                    tint: None,
                    actor: None,
                });
            }
        }
    }
    draws
}

#[cfg(test)]
mod tests {
    use super::*;
    use ue2_level::{SECTOR_MAP_SIDE, TerrainNative};

    fn legacy_geometry(placed: &[DecoInstance], section_indices: &[u16], positions: &[[f32; 3]], uvs: &[[f32; 2]], colors: &[[u8; 4]], heightmap: [usize; 2]) -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for instance in placed {
            let world = placement_matrix(instance.position.to_array(), instance.rotation, instance.scale.to_array());
            let color = colors.get(instance.vertex[1] * heightmap[0] + instance.vertex[0])
                .map_or([0.0, 0.0, 0.0, 1.0], |c| [f32::from(c[0]) / 255.0, f32::from(c[1]) / 255.0, f32::from(c[2]) / 255.0, f32::from(c[3]) / 255.0]);
            let uv_mask = mask_uv(instance.vertex[0], instance.vertex[1], heightmap[0], heightmap[1]);
            let mut remap: HashMap<u16, u32> = HashMap::new();
            for &index in section_indices {
                let local = *remap.entry(index).or_insert_with(|| {
                    let i = index as usize;
                    let next = vertices.len() as u32;
                    let position = world.transform_point3(Vec3::from(positions[i])).to_array();
                    vertices.push(Vertex { position, normal: [0.0, 0.0, 1.0], uv: uvs.get(i).copied().unwrap_or([0.0; 2]), mask_uv: uv_mask, color });
                    next
                });
                indices.push(local);
            }
        }
        (vertices, indices)
    }

    fn same_geometry(expected: &(Vec<Vertex>, Vec<u32>), actual: &(Vec<Vertex>, Vec<u32>)) {
        assert!(bytemuck::cast_slice::<Vertex, u8>(&expected.0) == bytemuck::cast_slice::<Vertex, u8>(&actual.0), "bytes dos vértices alterados");
        assert!(expected.1 == actual.1, "índices alterados");
        assert_eq!(actual.0.capacity(), actual.0.len());
        assert_eq!(actual.1.capacity(), actual.1.len());
    }

    fn same_compact(expected: &(Vec<Vertex>, Vec<u32>), compact: &(Vec<Vertex>, Vec<u32>, Vec<DecorationInstance>)) {
        assert_eq!(expected.0.len(), compact.0.len() * compact.2.len());
        assert_eq!(expected.1.len(), compact.1.len() * compact.2.len());
        let vertices = compact.2.iter().flat_map(|instance| compact.0.iter().map(|&vertex| instance.vertex(vertex)));
        for (expected, actual) in expected.0.iter().zip(vertices) {
            assert_eq!(bytemuck::bytes_of(expected), bytemuck::bytes_of(&actual));
        }
        let indices = (0..compact.2.len()).flat_map(|index| compact.1.iter().map(move |local| (index * compact.0.len()) as u32 + local));
        assert!(expected.1.iter().copied().eq(indices));
    }

    #[test]
    fn shared_decoration_topology_preserves_first_use_order_and_exact_geometry() {
        let positions = [[2.0, -3.0, 1.0], [-0.0, 4.0, 8.0], [7.0, 1.0, -2.0], [5.0, 6.0, 7.0]];
        let uvs = [[0.2, 0.5], [0.7, 0.9]];
        let colors = [[7, 19, 230, 255]; 4];
        let placed: Vec<_> = (0..17).map(|index| DecoInstance {
            position: Vec3::new(index as f32 * 11.0, -2.0, 17.0),
            rotation: [index * 93, index * 381, -index * 77],
            scale: Vec3::new(-1.25, 0.75, 3.0),
            vertex: [index as usize % 3, index as usize % 2],
        }).collect();
        for indices in [&[][..], &[2, 0, 2, 3, 0, 1, 2, 3, 1][..]] {
            let topology = SectionTopology::new(indices);
            let recovered: Vec<_> = topology.indices.iter().map(|&index| topology.sources[index as usize] as u16).collect();
            assert_eq!(recovered, indices);
            for count in [0, 1, placed.len()] {
                let expected = legacy_geometry(&placed[..count], indices, &positions, &uvs, &colors, [3, 3]);
                let actual = topology.expand(&placed[..count], &positions, &uvs, &colors, [3, 3]);
                same_geometry(&expected, &actual);
                let compact = topology.compact(&placed[..count], &positions, &uvs, &colors, [3, 3]);
                same_compact(&expected, &compact);
            }
        }
    }

    #[test]
    #[ignore]
    fn original_decoration_topology_preserves_all_lobby_and_heine_instances() {
        verify_decoration_tiles(["lobby".to_owned(), "23_24".to_owned()], 2);
    }

    #[test]
    #[ignore]
    fn all_143_tiles_preserve_every_decoration_instance() {
        verify_decoration_tiles((15..=26).flat_map(|x| (10..=26).map(move |y| format!("{x}_{y}"))), 143);
    }

    fn verify_decoration_tiles(tiles: impl IntoIterator<Item = String>, expected_tiles: usize) {
        let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let mut total_vertices = 0usize;
        let mut checked_tiles = 0usize;
        let mut unavailable = Vec::new();
        for tile in tiles {
            let mut client = Client::open(std::path::Path::new(&root)).unwrap();
            let Ok(loaded) = client.load_level(&tile) else { continue };
            checked_tiles += 1;
            let (mut count, mut vertices, mut indices, mut old_bytes, mut new_bytes) = (0usize, 0usize, 0usize, 0usize, 0usize);
            let mut compact_bytes = 0usize;
            let (mut old_time, mut new_time) = (std::time::Duration::ZERO, std::time::Duration::ZERO);
            for terrain in &loaded.level.terrains {
                let Some(grid) = seamed_grid(&mut client, terrain, &TerrainSeam::NONE) else { continue };
                for layer in &terrain.deco_layers {
                    let Some(mesh) = client.catalog().static_mesh(&layer.static_mesh) else {
                        unavailable.push(format!("{tile}: malha {}", layer.static_mesh));
                        continue;
                    };
                    let Some(density) = density_map(&mut client, &layer.density_map) else {
                        unavailable.push(format!("{tile}: densidade {}", layer.density_map));
                        continue;
                    };
                    let scale_map = layer.scale_map.as_deref().and_then(|path| color_map(&mut client, path));
                    for section in &mesh.mesh.sections {
                        let first = section.first_index as usize;
                        let Some(source) = mesh.mesh.indices.get(first..first + section.index_count() as usize) else { continue };
                        if source.is_empty() { continue; }
                        let topology = SectionTopology::new(source);
                        for (sector_index, sector) in terrain.sectors.iter().enumerate() {
                            let mut rng = Srand::new(layer.seed.wrapping_add(sector_index as i32));
                            let placed = instances(&grid, terrain, sector, layer, &density, scale_map.as_ref(), &mut rng);
                            let started = std::time::Instant::now();
                            let expected = legacy_geometry(&placed, source, &mesh.mesh.positions, &mesh.mesh.uvs, &terrain.native.vertex_colors, grid.heightmap);
                            old_time += started.elapsed();
                            let started = std::time::Instant::now();
                            let actual = topology.expand(&placed, &mesh.mesh.positions, &mesh.mesh.uvs, &terrain.native.vertex_colors, grid.heightmap);
                            new_time += started.elapsed();
                            same_geometry(&expected, &actual);
                            let compact = topology.compact(&placed, &mesh.mesh.positions, &mesh.mesh.uvs, &terrain.native.vertex_colors, grid.heightmap);
                            same_compact(&expected, &compact);
                            compact_bytes += compact.0.capacity() * size_of::<Vertex>() + compact.1.capacity() * size_of::<u32>() + compact.2.capacity() * size_of::<DecorationInstance>();
                            count += placed.len();
                            vertices += actual.0.len();
                            indices += actual.1.len();
                            old_bytes += expected.0.capacity() * size_of::<Vertex>() + expected.1.capacity() * size_of::<u32>();
                            new_bytes += actual.0.capacity() * size_of::<Vertex>() + actual.1.capacity() * size_of::<u32>();
                        }
                    }
                }
            }
            eprintln!("{tile}: compact geometry {compact_bytes} bytes");
            total_vertices += vertices;
            eprintln!("{tile}: {count} instâncias×seções; {vertices} vértices/{indices} índices idênticos; capacidade {old_bytes}→{new_bytes} bytes; montagem {:?}→{:?}", old_time, new_time);
        }
        assert_eq!(checked_tiles, expected_tiles);
        assert!(total_vertices > 0);
        eprintln!("{checked_tiles} mapas, {total_vertices} vértices conferidos; recursos indisponíveis: {unavailable:?}");
        assert!(unavailable.is_empty(), "cobertura incompleta: {unavailable:?}");
    }

    #[test]
    fn srand_stays_in_the_unit_interval_and_repeats_from_the_same_seed() {
        let mut a = Srand::new(1234);
        let mut b = Srand::new(1234);
        let values: Vec<f32> = (0..100).map(|_| a.sample()).collect();
        assert!(values.iter().all(|value| (0.0..1.0).contains(value)));
        assert!(values.windows(2).any(|pair| pair[0] != pair[1]));
        assert_eq!(values[7], { for _ in 0..7 { b.sample(); } b.sample() });
        let mut r = Srand::new(9);
        let ranged = r.range([2.0, 4.0]);
        assert!((2.0..4.0).contains(&ranged));
    }

    #[test]
    fn fade_is_linear_between_min_and_max_and_negative_beyond() {
        assert_eq!(fade(50.0, [100.0, 300.0]), 1.0);
        assert!((fade(200.0, [100.0, 300.0]) - 0.5).abs() < 1e-6);
        assert_eq!(fade(301.0, [100.0, 300.0]), -1.0);
        assert_eq!(fade(300.0, [300.0, 300.0]), 1.0);
    }

    #[test]
    fn the_up_normal_yields_a_level_rotation_after_the_quarter_pitch() {
        assert_eq!(normal_rotation(Vec3::Z), [0, 0, 0]);
        let tilted = normal_rotation(Vec3::new(1.0, 0.0, 1.0).normalize());
        assert_eq!(tilted[0], 8192 - 16384);
        assert_eq!(tilted[1], 0);
    }

    fn flat_terrain() -> (Grid, Terrain) {
        let grid = Grid::from_samples(4, 4, &[32767; 16], [0.0; 3], [128.0, 128.0, 76.0]);
        let sector = TerrainSector { offset: [0, 0], quads: [3, 3], intensities: vec![vec![0; SECTOR_MAP_SIDE * SECTOR_MAP_SIDE]; 8], layer_all: [0xffff; 32], pass_mask: Vec::new(), lights: Vec::new() };
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
            deco_layer_offset: 10.0,
            native: TerrainNative::default(),
        };
        (grid, terrain)
    }

    fn grass(max_per_quad: u32) -> DecoLayer {
        DecoLayer {
            static_mesh: "P.grass".into(),
            density_map: "P.dens".into(),
            density_multiplier: [1.0, 1.0],
            max_per_quad,
            scale: [[1.0, 1.0]; 3],
            fadeout_radius: [100.0, 300.0],
            seed: 5,
            random_yaw: false,
            on_invisible_terrain: false,
            align_to_terrain: false,
            scale_map: None,
            draw_order: 0,
            lit_directional: false,
            force_render: false,
        }
    }

    #[test]
    fn a_full_density_map_places_max_per_quad_instances_inside_each_quad_lifted_by_the_offset() {
        let (grid, terrain) = flat_terrain();
        let density = DensityMap { width: 4, height: 4, alpha: vec![255; 16] };
        let mut rng = Srand::new(5);
        let placed = instances(&grid, &terrain, &terrain.sectors[0], &grass(3), &density, None, &mut rng);
        assert_eq!(placed.len(), 9 * 3);
        for instance in &placed {
            let corner = Vec3::from(grid.position(instance.vertex[0], instance.vertex[1]));
            assert!(instance.position.x >= corner.x - 1e-3 && instance.position.x <= corner.x + 128.0 + 1e-3, "{instance:?}");
            assert!(instance.position.y >= corner.y - 1e-3 && instance.position.y <= corner.y + 128.0 + 1e-3, "{instance:?}");
            assert!((instance.position.z - 10.0).abs() < 1e-3);
            assert_eq!(instance.rotation, [0, 0, 0]);
            assert_eq!(instance.scale, Vec3::ONE);
        }
    }

    #[test]
    fn an_empty_density_map_or_hidden_quads_place_nothing() {
        let (grid, mut terrain) = flat_terrain();
        let mut rng = Srand::new(5);
        let empty = DensityMap { width: 4, height: 4, alpha: vec![0; 16] };
        assert!(instances(&grid, &terrain, &terrain.sectors[0], &grass(3), &empty, None, &mut rng).is_empty());
        terrain.visible_quads = vec![0];
        let full = DensityMap { width: 4, height: 4, alpha: vec![255; 16] };
        assert!(instances(&grid, &terrain, &terrain.sectors[0], &grass(3), &full, None, &mut rng).is_empty());
        let anywhere = DecoLayer { on_invisible_terrain: true, ..grass(1) };
        assert_eq!(instances(&grid, &terrain, &terrain.sectors[0], &anywhere, &full, None, &mut rng).len(), 9);
    }

    #[test]
    fn half_density_places_about_half_as_many_over_many_quads() {
        let grid = Grid::from_samples(65, 65, &[32767; 65 * 65], [0.0; 3], [128.0, 128.0, 76.0]);
        let (_, mut terrain) = flat_terrain();
        terrain.sectors[0].quads = [64, 64];
        let density = DensityMap { width: 65, height: 65, alpha: vec![128; 65 * 65] };
        let mut rng = Srand::new(77);
        let placed = instances(&grid, &terrain, &terrain.sectors[0], &grass(4), &density, None, &mut rng);
        let expected = 64.0 * 64.0 * 4.0 * 128.0 / 255.0;
        assert!((placed.len() as f32 - expected).abs() < expected * 0.05, "{} vs {expected}", placed.len());
    }
}
