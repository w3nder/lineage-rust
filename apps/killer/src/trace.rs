use glam::Vec3;
use std::borrow::Borrow;
use killer_assets::scene::{DrawKind, Pass, Scene};

const LEAF_TRIANGLES: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Node {
    min: Vec3,
    max: Vec3,
    first: u32,
    count: u32,
    right: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    pub distance: f32,
    pub terrain: bool,
}

pub struct TraceMesh {
    positions: Vec<[f32; 3]>,
    indices: Vec<u32>,
    terrain: Vec<bool>,
    nodes: Vec<Node>,
}

#[cfg(any(test, target_arch = "wasm32"))]
pub struct TraceBuilder {
    sky: [bool; 256],
    draws: Vec<TraceDraw>,
}

#[cfg(any(test, target_arch = "wasm32"))]
struct TraceDraw {
    positions: Vec<[f32; 3]>,
    indices: Vec<u32>,
    terrain: bool,
}

#[cfg(any(test, target_arch = "wasm32"))]
impl TraceBuilder {
    pub fn new(scene: &Scene) -> Self {
        Self { sky: std::array::from_fn(|zone| scene.zone(zone as u8).is_sky), draws: Vec::new() }
    }

    pub fn push(&mut self, draw: killer_assets::scene::MeshDraw) {
        if matches!(draw.pass, Pass::Translucent | Pass::Projector) || self.sky[draw.zone as usize] { return; }
        let mut positions = Vec::with_capacity(draw.world_vertex_count());
        positions.extend(draw.world_vertices().map(|vertex| vertex.position));
        let indices = if draw.instances.is_empty() { draw.indices } else {
            let mut indices = Vec::with_capacity(draw.world_index_count());
            indices.extend(draw.world_indices());
            indices
        };
        self.draws.push(TraceDraw { positions, indices, terrain: draw.kind == DrawKind::Terrain });
    }

    pub fn build(self) -> TraceMesh {
        let vertex_count = self.draws.iter().map(|draw| draw.positions.len()).sum();
        let index_count = self.draws.iter().map(|draw| draw.indices.len()).sum();
        let mut mesh = TraceMesh { positions: Vec::with_capacity(vertex_count), indices: Vec::with_capacity(index_count), terrain: Vec::with_capacity(index_count / 3), nodes: Vec::new() };
        for TraceDraw { positions, indices, terrain } in self.draws {
            let base = mesh.positions.len() as u32;
            mesh.positions.extend(positions);
            mesh.indices.extend(indices.iter().map(|index| base + index));
            mesh.terrain.extend(std::iter::repeat_n(terrain, indices.len() / 3));
        }
        mesh.build();
        mesh
    }
}

impl TraceMesh {
    pub fn from_scene(scene: &Scene) -> Self {
        Self::from_draws(solid_draws(scene))
    }

    pub fn from_draws<'a>(draws: impl Iterator<Item = &'a killer_assets::scene::MeshDraw>) -> Self {
        Self::from_meshes(draws.collect())
    }

    #[cfg(any(test, target_arch = "wasm32"))]
    pub fn from_owned_scene(mut scene: Scene) -> Self {
        let mut draws = std::mem::take(&mut scene.draws);
        draws.retain(|draw| !matches!(draw.pass, Pass::Translucent | Pass::Projector) && !scene.zone(draw.zone).is_sky);
        drop(scene);
        Self::from_meshes(draws)
    }

    fn from_meshes<T: Borrow<killer_assets::scene::MeshDraw>>(draws: Vec<T>) -> Self {
        let vertex_count = draws.iter().map(|draw| draw.borrow().world_vertex_count()).sum();
        let index_count = draws.iter().map(|draw| draw.borrow().world_index_count()).sum();
        let mut positions = Vec::with_capacity(vertex_count);
        let mut indices = Vec::with_capacity(index_count);
        let mut terrain = Vec::with_capacity(index_count / 3);
        for draw in draws {
            let draw = draw.borrow();
            let base = positions.len() as u32;
            positions.extend(draw.world_vertices().map(|v| v.position));
            indices.extend(draw.world_indices().map(|i| base + i));
            terrain.extend(std::iter::repeat_n(draw.kind == DrawKind::Terrain, draw.world_index_count() / 3));
        }
        let mut mesh = Self { positions, indices, terrain, nodes: Vec::new() };
        mesh.build();
        mesh
    }

    pub fn triangles(&self) -> usize {
        self.indices.len() / 3
    }

    #[cfg(test)]
    pub fn nearest_hit(&self, from: Vec3, to: Vec3) -> Option<f32> {
        self.nearest(from, to).map(|hit| hit.distance)
    }

    pub fn nearest(&self, from: Vec3, to: Vec3) -> Option<Hit> {
        let dir = to - from;
        let length = dir.length();
        if length <= 0.0 || self.nodes.is_empty() {
            return None;
        }
        let dir = dir / length;
        let inv = Vec3::new(invert(dir.x), invert(dir.y), invert(dir.z));
        let mut nearest = length;
        let mut found = None;
        let mut stack = vec![0u32];
        while let Some(index) = stack.pop() {
            let node = self.nodes[index as usize];
            if !slab(from, inv, node.min, node.max, nearest) {
                continue;
            }
            if node.count > 0 {
                for index in node.first..node.first + node.count {
                    let tri = self.triangle(index);
                    if let Some(t) = ray_triangle(from, dir, tri[0], tri[1], tri[2])
                        && t <= nearest
                    {
                        nearest = t;
                        found = Some(index);
                    }
                }
            } else {
                stack.push(node.right);
                stack.push(index + 1);
            }
        }
        found.map(|index| Hit { distance: nearest, terrain: self.terrain[index as usize] })
    }

    fn triangle(&self, index: u32) -> [Vec3; 3] {
        let i = index as usize * 3;
        [self.indices[i], self.indices[i + 1], self.indices[i + 2]].map(|v| Vec3::from(self.positions[v as usize]))
    }

    fn build(&mut self) {
        let count = self.triangles();
        if count == 0 {
            return;
        }
        self.nodes = Vec::with_capacity(node_count(count));
        let centroids: Vec<Vec3> = (0..count as u32).map(|t| self.triangle(t).iter().sum::<Vec3>() / 3.0).collect();
        let mut order: Vec<u32> = (0..count as u32).collect();
        let mut pending = vec![(0usize, count, usize::MAX)];
        while let Some((start, end, parent)) = pending.pop() {
            let index = self.nodes.len() as u32;
            if parent != usize::MAX {
                self.nodes[parent].right = index;
            }
            let (min, max) = order[start..end].iter().fold((Vec3::INFINITY, Vec3::NEG_INFINITY), |(lo, hi), &t| {
                let tri = self.triangle(t);
                (lo.min(tri[0]).min(tri[1]).min(tri[2]), hi.max(tri[0]).max(tri[1]).max(tri[2]))
            });
            let span = end - start;
            if span <= LEAF_TRIANGLES {
                self.nodes.push(Node { min, max, first: start as u32, count: span as u32, right: 0 });
                continue;
            }
            let (lo, hi) = order[start..end].iter().fold((Vec3::INFINITY, Vec3::NEG_INFINITY), |(lo, hi), &t| (lo.min(centroids[t as usize]), hi.max(centroids[t as usize])));
            let extent = hi - lo;
            let axis = if extent.x >= extent.y && extent.x >= extent.z {
                0
            } else if extent.y >= extent.z {
                1
            } else {
                2
            };
            let mid = start + span / 2;
            order[start..end].select_nth_unstable_by(span / 2, |&a, &b| centroids[a as usize][axis].total_cmp(&centroids[b as usize][axis]));
            self.nodes.push(Node { min, max, first: 0, count: 0, right: 0 });
            pending.push((mid, end, index as usize));
            pending.push((start, mid, usize::MAX));
        }
        drop(centroids);
        for start in 0..count {
            if order[start] as usize == start { continue; }
            let first = [self.indices[start * 3], self.indices[start * 3 + 1], self.indices[start * 3 + 2]];
            let terrain = self.terrain[start];
            let mut current = start;
            loop {
                let source = order[current] as usize;
                order[current] = current as u32;
                if source == start {
                    self.indices[current * 3..current * 3 + 3].copy_from_slice(&first);
                    self.terrain[current] = terrain;
                    break;
                }
                self.indices.copy_within(source * 3..source * 3 + 3, current * 3);
                self.terrain[current] = self.terrain[source];
                current = source;
            }
        }
    }
}

#[derive(Default)]
pub struct MovingTrace {
    entries: std::collections::HashMap<(String, String), MovingEntry>,
}

struct MovingEntry {
    meshes: [TraceMesh; 8],
    normals: [Vec<Vec3>; 8],
    matrix: glam::Mat4,
    inverse: glam::Mat4,
    grade: usize,
}

impl MovingTrace {
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub fn sync(&mut self, renderer: &killer_render::renderer::Renderer) {
        let mut seen = std::collections::HashSet::new();
        for (tile, asset, matrix, grade) in renderer.moving_actors() {
            let key = (tile.to_owned(), asset.name.clone());
            seen.insert(key.clone());
            let entry = self.entries.entry(key).or_insert_with(|| MovingEntry {
                meshes: std::array::from_fn(|i| TraceMesh::from_draws(if i == 0 { &asset.draws } else { &asset.grades[i - 1] }.iter().filter(|d| d.pass != Pass::Translucent))),
                normals: std::array::from_fn(|i| if i == 0 { &asset.draws } else { &asset.grades[i - 1] }.iter().filter(|d| d.pass != Pass::Translucent).flat_map(|d| d.vertices.iter().map(|v| Vec3::from(v.normal))).collect()),
                matrix, inverse: matrix.inverse(), grade: 0,
            });
            entry.matrix = matrix;
            entry.inverse = matrix.inverse();
            entry.grade = grade.map_or(0, |g| g + 1);
        }
        self.entries.retain(|key, _| seen.contains(key));
    }

    pub fn nearest(&self, from: Vec3, to: Vec3) -> Option<Hit> {
        self.entries.values().filter_map(|e| e.meshes[e.grade].nearest(e.inverse.transform_point3(from), e.inverse.transform_point3(to))).min_by(|a, b| a.distance.total_cmp(&b.distance))
    }

    pub fn ground_at(&self, x: f32, y: f32, above: f32, below: f32) -> Option<f32> {
        let from = Vec3::new(x, y, above);
        let to = Vec3::new(x, y, below);
        let mut floor: Option<f32> = None;
        for entry in self.entries.values() {
            let mesh = &entry.meshes[entry.grade];
            if mesh.nearest(entry.inverse.transform_point3(from), entry.inverse.transform_point3(to)).is_none() { continue; }
            for tri in 0..mesh.indices.len() / 3 {
                let [a, b, c] = mesh.triangle(tri as u32).map(|p| entry.matrix.transform_point3(p));
                let facing = mesh.indices[tri * 3..tri * 3 + 3].iter().map(|&i| entry.normals[entry.grade][i as usize]).sum::<Vec3>();
                if !crate::ground::walkable(a, b, c, entry.matrix.transform_vector3(facing)) { continue; }
                if let Some(distance) = ray_triangle(from, -Vec3::Z, a, b, c) {
                    let z = above - distance;
                    if z >= below && floor.is_none_or(|f| z > f) { floor = Some(z); }
                }
            }
        }
        floor
    }
}

fn node_count(triangles: usize) -> usize {
    if triangles == 0 {return 0;}
    if triangles <= LEAF_TRIANGLES {return 1;}
    let half = triangles / 2;
    if triangles.is_multiple_of(2) {1 + 2 * node_count(half)} else {1 + node_count(half) + node_count(half + 1)}
}

fn invert(component: f32) -> f32 {
    1.0 / if component.abs() < f32::EPSILON { f32::EPSILON.copysign(component) } else { component }
}

fn slab(origin: Vec3, inv: Vec3, min: Vec3, max: Vec3, tmax: f32) -> bool {
    let t0 = (min - origin) * inv;
    let t1 = (max - origin) * inv;
    let near = t0.min(t1);
    let far = t0.max(t1);
    let enter = near.x.max(near.y).max(near.z).max(0.0);
    let exit = far.x.min(far.y).min(far.z).min(tmax);
    enter <= exit
}

fn solid_draws(scene: &Scene) -> impl Iterator<Item = &killer_assets::scene::MeshDraw> {
    scene.draws.iter().filter(|d| !matches!(d.pass, Pass::Translucent | Pass::Projector) && !scene.zone(d.zone).is_sky)
}

fn ray_triangle(origin: Vec3, dir: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Option<f32> {
    let e1 = b - a;
    let e2 = c - a;
    let p = dir.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-6 {
        return None;
    }
    let inv = 1.0 / det;
    let s = origin - a;
    let u = s.dot(p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(e1);
    let v = dir.dot(q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = e2.dot(q) * inv;
    (t > 0.0).then_some(t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_assets::scene::{DrawKind, MeshDraw, Vertex, ZoneLighting, texture_material};

    fn wall_in(x: f32, pass: Pass, zone: u8) -> MeshDraw {
        let vertex = |y: f32, z: f32| Vertex { position: [x, y, z], normal: [1.0, 0.0, 0.0], uv: [0.0; 2], mask_uv: [0.0; 2], color: [1.0; 4] };
        MeshDraw {
            instances: Vec::new(), vertices: vec![vertex(-100.0, -100.0), vertex(100.0, -100.0), vertex(100.0, 100.0), vertex(-100.0, 100.0)],
            indices: vec![0, 1, 2, 0, 2, 3],
            material: texture_material(""),
            mask: None,
            kind: DrawKind::StaticMesh,
            sun_affect: true,
            pass,
            zone,
            lightmap: None,
            modulate_2x: true,
            sector: None,
            fade: None,
            tint: None,
            actor: None,
        }
    }

    fn wall(x: f32, pass: Pass) -> MeshDraw {
        wall_in(x, pass, 0)
    }

    #[test]
    fn instanced_decorations_preserve_entire_trace_and_consuming_builder() {
        let mut draw = wall(10.0, Pass::Opaque);
        draw.kind = DrawKind::Terrain;
        draw.instances = (0..17).map(|i| killer_assets::scene::DecorationInstance {
            world: glam::Mat4::from_scale_rotation_translation(
                Vec3::new(-1.0, 0.75, 2.0), glam::Quat::from_rotation_z(i as f32 * 0.1),
                Vec3::new(i as f32 * 30.0, i as f32 * -10.0, 20.0),
            ).to_cols_array_2d(), mask_uv: [0.0; 2], color: [1.0; 4],
        }).collect();
        let scene = Scene { draws: vec![draw.clone()], ..Default::default() };
        let actual = TraceMesh::from_scene(&scene);
        let mut builder = TraceBuilder::new(&scene);
        builder.push(draw.clone());
        let consumed = builder.build();
        let vertices = draw.world_vertices().collect();
        let indices = draw.world_indices().collect();
        draw.vertices = vertices;
        draw.indices = indices;
        draw.instances.clear();
        let expected = TraceMesh::from_draws(std::iter::once(&draw));
        assert!(!expected.positions.is_empty());
        for result in [actual, consumed] {
            assert_eq!(result.positions, expected.positions);
            assert_eq!(result.indices, expected.indices);
            assert_eq!(result.terrain, expected.terrain);
            assert_eq!(result.nodes, expected.nodes);
        }
    }

    fn brute_force(mesh: &TraceMesh, from: Vec3, to: Vec3) -> Option<f32> {
        let dir = to - from;
        let length = dir.length();
        let dir = dir / length;
        (0..mesh.triangles() as u32)
            .filter_map(|t| {
                let tri = mesh.triangle(t);
                ray_triangle(from, dir, tri[0], tri[1], tri[2]).filter(|&t| t <= length)
            })
            .min_by(f32::total_cmp)
    }

    fn assert_geometry_preserved(scene: &Scene, mesh: &TraceMesh) {
        let mut expected = Vec::new();
        let mut base = 0u32;
        for draw in solid_draws(scene) {
            for (actual, vertex) in mesh.positions[base as usize..].iter().zip(&draw.vertices) {
                assert_eq!(*actual, vertex.position);
            }
            expected.extend(draw.indices.as_chunks::<3>().0.iter().map(|tri| (tri.map(|index| index + base), draw.kind == DrawKind::Terrain)));
            base += draw.vertices.len() as u32;
        }
        assert_eq!(mesh.positions.len(), base as usize);
        let mut actual: Vec<_> = mesh.indices.as_chunks::<3>().0.iter().copied().zip(mesh.terrain.iter().copied()).collect();
        expected.sort_unstable();
        actual.sort_unstable();
        assert_eq!(actual, expected);
        assert_eq!(mesh.positions.capacity(), mesh.positions.len());
        assert_eq!(mesh.indices.capacity(), mesh.indices.len());
        assert_eq!(mesh.nodes.capacity(), mesh.nodes.len());
    }

    #[test]
    fn in_place_bvh_permutation_keeps_every_triangle_and_terrain_flag() {
        let draws = (0..257).rev().map(|index| {
            let mut draw = wall(index as f32 * 10.0, Pass::Opaque);
            if index % 3 == 0 {draw.kind = DrawKind::Terrain;}
            draw
        }).collect();
        let scene = Scene {draws, ..Default::default()};
        let mesh = TraceMesh::from_scene(&scene);
        assert_geometry_preserved(&scene, &mesh);
        for index in 0..257 {
            let from = Vec3::new(index as f32 * 10.0 - 1.0, 0.0, 0.0);
            assert_eq!(mesh.nearest(from, from + Vec3::X * 2.0), Some(Hit {distance:1.0, terrain:index % 3 == 0}));
        }
    }

    #[test]
    #[ignore]
    fn heine_trace_keeps_all_original_triangles_with_exact_buffers() {
        let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let mut client = killer_assets::client::Client::open(std::path::Path::new(&root)).unwrap();
        let loaded = client.load_level("23_24").unwrap();
        let scene = killer_assets::scene::build_scene(&mut client, &loaded.level);
        client.catalog().release_files();
        let mesh = TraceMesh::from_scene(&scene);
        assert!(mesh.triangles() > 100_000);
        assert_geometry_preserved(&scene, &mesh);
        let source_bytes: usize = scene.draws.iter().map(|draw| draw.vertices.capacity() * size_of::<Vertex>() + draw.indices.capacity() * size_of::<u32>()).sum();
        let owned = TraceMesh::from_owned_scene(scene);
        assert_eq!(owned.positions, mesh.positions);
        assert_eq!(owned.indices, mesh.indices);
        assert_eq!(owned.terrain, mesh.terrain);
        assert_eq!(owned.nodes, mesh.nodes);
        assert_eq!(owned.nodes.capacity(), owned.nodes.len());
        eprintln!("Heine: {} triângulos; {} bytes de geometria CPU consumidos antes da BVH; {} nós, {} bytes exatos", mesh.triangles(), source_bytes, owned.nodes.len(), owned.nodes.capacity() * size_of::<Node>());
    }

    #[test]
    fn streamed_scene_keeps_the_same_tree_and_all_collision_filters() {
        let draws = (0..257).map(|index| {
            let pass = match index % 7 { 0 => Pass::Translucent, 1 => Pass::Projector, _ => Pass::Opaque };
            let mut draw = wall_in(index as f32, pass, u8::from(index % 11 == 0));
            if index % 3 == 0 { draw.kind = DrawKind::Terrain; }
            draw
        }).collect();
        let scene = Scene { draws, zones: vec![ZoneLighting::default(), ZoneLighting { is_sky: true, ..Default::default() }], ..Default::default() };
        let expected = TraceMesh::from_scene(&scene);
        let mut builder = TraceBuilder::new(&scene);
        killer_assets::cache::encode_consuming(std::io::sink(), &killer_assets::cache::SourceStamp { size: 0, hash: 0 }, scene, |draw| builder.push(draw)).unwrap();
        let actual = builder.build();
        assert_eq!(actual.positions, expected.positions);
        assert_eq!(actual.indices, expected.indices);
        assert_eq!(actual.terrain, expected.terrain);
        assert_eq!(actual.nodes, expected.nodes);
        let empty = TraceBuilder::new(&Scene::default()).build();
        assert_eq!(empty.triangles(), 0);
        assert!(empty.nodes.is_empty());
    }

    #[test]
    fn owned_scene_keeps_the_same_tree_and_exact_node_capacity() {
        for count in [0, 1, 7, 8, 9, 16, 17, 31, 65, 257, 1025] {
            let draws = (0..count).map(|index| {
                let mut draw = wall(index as f32, Pass::Opaque);
                draw.indices.truncate(3);
                if index % 3 == 0 {draw.kind = DrawKind::Terrain;}
                draw
            }).collect();
            let scene = Scene {draws, ..Default::default()};
            let borrowed = TraceMesh::from_scene(&scene);
            let owned = TraceMesh::from_owned_scene(scene);
            assert_eq!(owned.positions, borrowed.positions);
            assert_eq!(owned.indices, borrowed.indices);
            assert_eq!(owned.terrain, borrowed.terrain);
            assert_eq!(owned.nodes, borrowed.nodes);
            assert_eq!(owned.nodes.capacity(), owned.nodes.len());
        }
    }

    #[test]
    fn nearest_opaque_wall_along_the_segment_is_reported() {
        let scene = Scene { draws: vec![wall(80.0, Pass::Opaque), wall(50.0, Pass::Opaque), wall(20.0, Pass::Translucent)], ..Default::default() };
        let mesh = TraceMesh::from_scene(&scene);
        assert_eq!(mesh.triangles(), 4);
        let hit = mesh.nearest_hit(Vec3::ZERO, Vec3::new(230.0, 0.0, 0.0)).unwrap();
        assert!((hit - 50.0).abs() < 1e-4);
    }

    #[test]
    fn moving_floor_keeps_walkable_normal_and_current_height() {
        let mut draw = wall(0.0, Pass::Opaque);
        for vertex in &mut draw.vertices {
            let [x, y, z] = vertex.position;
            vertex.position = [y, z, x];
            vertex.normal = [0.0, 0.0, 1.0];
        }
        let matrix = glam::Mat4::from_translation(Vec3::new(0.0, 0.0, 80.0));
        let key = ("tile".to_owned(), "floor".to_owned());
        let mut moving = MovingTrace::default();
        moving.entries.insert(key.clone(), MovingEntry {
            meshes: std::array::from_fn(|i| if i == 0 { TraceMesh::from_draws(std::iter::once(&draw)) } else { TraceMesh::from_scene(&Scene::default()) }),
            normals: std::array::from_fn(|i| if i == 0 { draw.vertices.iter().map(|v| Vec3::from(v.normal)).collect() } else { Vec::new() }),
            matrix, inverse: matrix.inverse(), grade: 0,
        });
        assert_eq!(moving.ground_at(0.0, 0.0, 100.0, -100.0), Some(80.0));
        assert_eq!(moving.ground_at(0.0, 0.0, 70.0, -100.0), None);
        assert_eq!(moving.ground_at(200.0, 0.0, 100.0, -100.0), None);
        let entry = moving.entries.get_mut(&key).unwrap();
        entry.normals[0].fill(-Vec3::Z);
        assert_eq!(moving.ground_at(0.0, 0.0, 100.0, -100.0), None);
        moving.entries.get_mut(&key).unwrap().grade = 7;
        assert_eq!(moving.ground_at(0.0, 0.0, 100.0, -100.0), None);
    }

    #[test]
    fn moving_mesh_trace_uses_current_transform_and_damage_mesh_without_old_wall() {
        let draw = wall(100.0, Pass::Opaque);
        let matrix = glam::Mat4::from_translation(Vec3::new(80.0, 0.0, 0.0));
        let mut moving = MovingTrace::default();
        let key = ("tile".to_owned(), "door".to_owned());
        moving.entries.insert(key.clone(), MovingEntry {
            meshes: std::array::from_fn(|i| if i == 0 { TraceMesh::from_draws(std::iter::once(&draw)) } else { TraceMesh::from_scene(&Scene::default()) }),
            normals: std::array::from_fn(|i| if i == 0 { draw.vertices.iter().map(|v| Vec3::from(v.normal)).collect() } else { Vec::new() }),
            matrix, inverse: matrix.inverse(), grade: 0,
        });
        assert_eq!(moving.nearest(Vec3::ZERO, Vec3::new(300.0, 0.0, 0.0)).unwrap().distance, 180.0);
        assert!(moving.nearest(Vec3::ZERO, Vec3::new(150.0, 0.0, 0.0)).is_none());
        moving.entries.get_mut(&key).unwrap().grade = 7;
        assert!(moving.nearest(Vec3::ZERO, Vec3::new(300.0, 0.0, 0.0)).is_none());
    }

    #[test]
    fn walls_beyond_the_eye_or_behind_the_target_do_not_count() {
        let scene = Scene { draws: vec![wall(300.0, Pass::Opaque), wall(-10.0, Pass::Opaque)], ..Default::default() };
        let mesh = TraceMesh::from_scene(&scene);
        assert_eq!(mesh.nearest_hit(Vec3::ZERO, Vec3::new(230.0, 0.0, 0.0)), None);
        assert_eq!(mesh.nearest_hit(Vec3::ZERO, Vec3::new(0.0, 230.0, 0.0)), None);
        assert_eq!(TraceMesh::from_scene(&Scene::default()).nearest_hit(Vec3::ZERO, Vec3::X), None);
    }

    #[test]
    fn a_hit_says_whether_the_triangle_is_terrain() {
        let mut ground = wall(50.0, Pass::Opaque);
        ground.kind = DrawKind::Terrain;
        let scene = Scene { draws: vec![wall(80.0, Pass::Opaque), ground], ..Default::default() };
        let mesh = TraceMesh::from_scene(&scene);
        assert_eq!(mesh.nearest(Vec3::ZERO, Vec3::new(230.0, 0.0, 0.0)), Some(Hit { distance: 50.0, terrain: true }));
        assert_eq!(mesh.nearest(Vec3::new(60.0, 0.0, 0.0), Vec3::new(230.0, 0.0, 0.0)), Some(Hit { distance: 20.0, terrain: false }));
    }

    #[test]
    fn sky_draws_are_left_out_of_the_trace_mesh() {
        let sky = ZoneLighting { is_sky: true, ..Default::default() };
        let scene = Scene { draws: vec![wall_in(30.0, Pass::Opaque, 1), wall(50.0, Pass::Opaque)], zones: vec![ZoneLighting::default(), sky], ..Default::default() };
        let mesh = TraceMesh::from_scene(&scene);
        assert_eq!(mesh.triangles(), 2);
        let hit = mesh.nearest_hit(Vec3::ZERO, Vec3::new(230.0, 0.0, 0.0)).unwrap();
        assert!((hit - 50.0).abs() < 1e-4);
    }

    #[test]
    fn the_bvh_reports_the_same_hit_as_brute_force_on_200_random_rays() {
        let mut state = 0x9e3779b97f4a7c15u64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 11) as f32 / (1u64 << 53) as f32
        };
        let mut draws = Vec::new();
        for _ in 0..300 {
            let center = Vec3::new(next() * 4000.0 - 2000.0, next() * 4000.0 - 2000.0, next() * 1000.0 - 500.0);
            let size = 20.0 + next() * 300.0;
            let vertex = |p: Vec3| Vertex { position: p.to_array(), normal: [0.0, 0.0, 1.0], uv: [0.0; 2], mask_uv: [0.0; 2], color: [1.0; 4] };
            let a = center + Vec3::new(next() - 0.5, next() - 0.5, next() - 0.5) * size;
            let b = center + Vec3::new(next() - 0.5, next() - 0.5, next() - 0.5) * size;
            let c = center + Vec3::new(next() - 0.5, next() - 0.5, next() - 0.5) * size;
            let mut draw = wall(0.0, Pass::Opaque);
            draw.vertices = vec![vertex(a), vertex(b), vertex(c)];
            draw.indices = vec![0, 1, 2];
            draws.push(draw);
        }
        let mesh = TraceMesh::from_scene(&Scene { draws, ..Default::default() });
        assert_eq!(mesh.triangles(), 300);
        assert!(mesh.nodes.len() > 1);
        let mut hits = 0;
        for _ in 0..200 {
            let from = Vec3::new(next() * 4000.0 - 2000.0, next() * 4000.0 - 2000.0, next() * 1000.0 - 500.0);
            let to = from + Vec3::new(next() - 0.5, next() - 0.5, next() - 0.5) * 3000.0;
            let expected = brute_force(&mesh, from, to);
            let got = mesh.nearest_hit(from, to);
            match (expected, got) {
                (Some(e), Some(g)) => assert!((e - g).abs() < 1e-3, "{e} vs {g}"),
                (None, None) => {}
                other => panic!("{other:?}"),
            }
            hits += usize::from(got.is_some());
        }
        assert!(hits > 0);
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
#[path = "trace/cache.rs"]
mod cache;
